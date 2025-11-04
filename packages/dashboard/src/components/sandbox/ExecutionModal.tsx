// ABOUTME: Modal dialog for configuring and starting new sandbox executions
// ABOUTME: Allows selecting agent, sandbox provider, resource limits, and environment variables

import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  PlayCircle,
  Settings,
  Cpu,
  HardDrive,
  Clock,
  AlertCircle,
  Loader2,
} from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import { agentsService } from '@/services/agents';
import { sandboxService, type SandboxConfig } from '@/services/sandbox';
import { executionsService, type AgentExecutionCreateInput } from '@/services/executions';
import { toast } from 'sonner';

interface ExecutionModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  taskId: string;
  defaultPrompt?: string;
  onExecutionCreated?: (executionId: string) => void;
}

export function ExecutionModal({
  open,
  onOpenChange,
  taskId,
  defaultPrompt = '',
  onExecutionCreated,
}: ExecutionModalProps) {
  const [agentId, setAgentId] = useState<string>('');
  const [sandboxId, setSandboxId] = useState<string>('local-docker');
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [memoryLimit, setMemoryLimit] = useState('2048');
  const [cpuLimit, setCpuLimit] = useState('2.0');
  const [timeoutSeconds, setTimeoutSeconds] = useState('3600');
  const [envVars, setEnvVars] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  // Load agents
  const { data: agentsData, isLoading: loadingAgents } = useQuery({
    queryKey: ['agents'],
    queryFn: () => agentsService.listAgents(),
  });

  const agents = agentsData?.items || [];

  // Load sandbox configs
  const { data: sandboxConfigs = [], isLoading: loadingSandboxes } = useQuery({
    queryKey: ['sandbox-configs'],
    queryFn: () => sandboxService.loadSandboxConfigs(),
  });

  const selectedSandbox = sandboxConfigs.find((s) => s.id === sandboxId);

  // Set default agent when loaded
  useEffect(() => {
    if (agents.length > 0 && !agentId) {
      setAgentId(agents[0].id);
    }
  }, [agents, agentId]);

  // Reset form when dialog closes
  useEffect(() => {
    if (!open) {
      setPrompt(defaultPrompt);
      setMemoryLimit('2048');
      setCpuLimit('2.0');
      setTimeoutSeconds('3600');
      setEnvVars('');
      setIsCreating(false);
    }
  }, [open, defaultPrompt]);

  const handleSubmit = async () => {
    if (!agentId) {
      toast.error('Please select an agent');
      return;
    }

    if (!prompt.trim()) {
      toast.error('Please provide a prompt for the agent');
      return;
    }

    setIsCreating(true);
    try {
      // Parse environment variables
      let parsedEnvVars: Record<string, string> | undefined;
      if (envVars.trim()) {
        try {
          const lines = envVars.split('\n').filter((line) => line.trim());
          parsedEnvVars = Object.fromEntries(
            lines.map((line) => {
              const [key, ...valueParts] = line.split('=');
              return [key.trim(), valueParts.join('=').trim()];
            })
          );
        } catch (err) {
          toast.error('Invalid environment variables format');
          setIsCreating(false);
          return;
        }
      }

      // Create execution request
      const input: AgentExecutionCreateInput = {
        taskId,
        agentId,
        prompt,
        // Note: sandbox-specific fields will be added to the execution record
        // by the backend based on the sandbox configuration
      };

      const execution = await executionsService.createExecution(input);

      toast.success('Execution started successfully');
      onExecutionCreated?.(execution.id);
      onOpenChange(false);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to start execution');
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <PlayCircle className="h-5 w-5" />
            Execute with AI Agent
          </DialogTitle>
          <DialogDescription>
            Configure sandbox environment and agent settings to run this task
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* Agent Selection */}
          <div className="space-y-2">
            <Label htmlFor="agent">Agent</Label>
            <Select value={agentId} onValueChange={setAgentId} disabled={loadingAgents}>
              <SelectTrigger id="agent">
                <SelectValue placeholder="Select an agent" />
              </SelectTrigger>
              <SelectContent>
                {agents.map((agent) => (
                  <SelectItem key={agent.id} value={agent.id}>
                    <div className="flex items-center gap-2">
                      <span>{agent.name}</span>
                      {agent.capabilities && agent.capabilities.length > 0 && (
                        <Badge variant="outline" className="text-xs">
                          {agent.capabilities[0]}
                        </Badge>
                      )}
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {agents.find((a) => a.id === agentId)?.description && (
              <p className="text-xs text-muted-foreground">
                {agents.find((a) => a.id === agentId)?.description}
              </p>
            )}
          </div>

          {/* Sandbox Provider */}
          <div className="space-y-2">
            <Label htmlFor="sandbox">Sandbox Provider</Label>
            <Select value={sandboxId} onValueChange={setSandboxId} disabled={loadingSandboxes}>
              <SelectTrigger id="sandbox">
                <SelectValue placeholder="Select sandbox provider" />
              </SelectTrigger>
              <SelectContent>
                {sandboxConfigs.map((sandbox) => (
                  <SelectItem key={sandbox.id} value={sandbox.id} disabled={!sandbox.is_available}>
                    <div className="flex items-center gap-2">
                      <span>{sandbox.name}</span>
                      {!sandbox.is_available && (
                        <Badge variant="destructive">Unavailable</Badge>
                      )}
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {selectedSandbox && (
              <p className="text-xs text-muted-foreground">{selectedSandbox.description}</p>
            )}
          </div>

          <Separator />

          {/* Prompt */}
          <div className="space-y-2">
            <Label htmlFor="prompt">Task Prompt</Label>
            <Textarea
              id="prompt"
              placeholder="Describe what the agent should do..."
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={6}
              className="font-mono text-sm"
            />
            <p className="text-xs text-muted-foreground">
              Provide clear instructions for the AI agent to execute this task
            </p>
          </div>

          {/* Resource Limits */}
          {selectedSandbox && (
            <>
              <Separator />
              <div className="space-y-4">
                <div className="flex items-center gap-2">
                  <Settings className="h-4 w-4" />
                  <h4 className="text-sm font-medium">Resource Limits</h4>
                </div>

                <div className="grid grid-cols-3 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="memory" className="text-xs">
                      <HardDrive className="inline h-3 w-3 mr-1" />
                      Memory (MB)
                    </Label>
                    <Input
                      id="memory"
                      type="number"
                      value={memoryLimit}
                      onChange={(e) => setMemoryLimit(e.target.value)}
                      min="512"
                      max="8192"
                    />
                    <p className="text-xs text-muted-foreground">
                      Default: {selectedSandbox.resource_limits.memory_mb}MB
                    </p>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="cpu" className="text-xs">
                      <Cpu className="inline h-3 w-3 mr-1" />
                      CPU Cores
                    </Label>
                    <Input
                      id="cpu"
                      type="number"
                      value={cpuLimit}
                      onChange={(e) => setCpuLimit(e.target.value)}
                      min="0.5"
                      max="8"
                      step="0.5"
                    />
                    <p className="text-xs text-muted-foreground">
                      Default: {selectedSandbox.resource_limits.cpu_cores}
                    </p>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="timeout" className="text-xs">
                      <Clock className="inline h-3 w-3 mr-1" />
                      Timeout (s)
                    </Label>
                    <Input
                      id="timeout"
                      type="number"
                      value={timeoutSeconds}
                      onChange={(e) => setTimeoutSeconds(e.target.value)}
                      min="60"
                      max="7200"
                    />
                    <p className="text-xs text-muted-foreground">
                      Default: {selectedSandbox.resource_limits.timeout_seconds}s
                    </p>
                  </div>
                </div>
              </div>
            </>
          )}

          {/* Environment Variables */}
          <div className="space-y-2">
            <Label htmlFor="envVars">Environment Variables (Optional)</Label>
            <Textarea
              id="envVars"
              placeholder="KEY=value&#10;ANOTHER_KEY=another value"
              value={envVars}
              onChange={(e) => setEnvVars(e.target.value)}
              rows={3}
              className="font-mono text-xs"
            />
            <p className="text-xs text-muted-foreground">
              One variable per line in KEY=value format
            </p>
          </div>

          {/* Warning Alert */}
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription className="text-xs">
              This will execute code in an isolated Docker container. Review the task prompt
              carefully before starting.
            </AlertDescription>
          </Alert>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isCreating}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isCreating || !agentId || !prompt.trim()}>
            {isCreating ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Starting...
              </>
            ) : (
              <>
                <PlayCircle className="mr-2 h-4 w-4" />
                Start Execution
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
