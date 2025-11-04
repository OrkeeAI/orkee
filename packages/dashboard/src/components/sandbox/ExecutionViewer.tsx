// ABOUTME: Component for viewing sandbox execution status and metrics
// ABOUTME: Shows execution state, resource usage gauges, and control actions

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  PlayCircle,
  StopCircle,
  RefreshCw,
  Clock,
  Cpu,
  HardDrive,
  CheckCircle,
  XCircle,
  Loader2,
  AlertCircle,
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { sandboxService, type SandboxExecution } from '@/services/sandbox';
import { executionsService } from '@/services/executions';
import { toast } from 'sonner';

interface ExecutionViewerProps {
  executionId: string;
  onRetry?: () => void;
}

export function ExecutionViewer({ executionId, onRetry }: ExecutionViewerProps) {
  const [isStopping, setIsStopping] = useState(false);

  // Fetch execution details (poll every 5 seconds if running)
  const {
    data: execution,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['execution', executionId],
    queryFn: () => executionsService.getExecution(executionId),
    refetchInterval: (query) => {
      const data = query.state.data;
      // Poll every 5 seconds if execution is running
      return data?.status === 'running' ? 5000 : false;
    },
  });

  const handleStop = async () => {
    if (!execution) return;

    // Get container_id from execution (cast to SandboxExecution type)
    const sandboxExec = execution as unknown as SandboxExecution;
    if (!sandboxExec.container_id) {
      toast.error('No container ID found for this execution');
      return;
    }

    setIsStopping(true);
    try {
      await sandboxService.stopExecution(executionId, sandboxExec.container_id);
      toast.success('Execution stopped successfully');
      refetch();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to stop execution');
    } finally {
      setIsStopping(false);
    }
  };

  const handleRetry = () => {
    onRetry?.();
  };

  const formatDuration = (startedAt: string, completedAt?: string | null) => {
    const start = new Date(startedAt).getTime();
    const end = completedAt ? new Date(completedAt).getTime() : Date.now();
    const durationMs = end - start;

    const seconds = Math.floor((durationMs / 1000) % 60);
    const minutes = Math.floor((durationMs / (1000 * 60)) % 60);
    const hours = Math.floor(durationMs / (1000 * 60 * 60));

    if (hours > 0) {
      return `${hours}h ${minutes}m ${seconds}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    } else {
      return `${seconds}s`;
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
        return <Loader2 className="h-4 w-4 animate-spin" />;
      case 'succeeded':
        return <CheckCircle className="h-4 w-4" />;
      case 'failed':
        return <XCircle className="h-4 w-4" />;
      case 'cancelled':
        return <StopCircle className="h-4 w-4" />;
      case 'pending':
        return <Clock className="h-4 w-4" />;
      default:
        return <AlertCircle className="h-4 w-4" />;
    }
  };

  const getStatusVariant = (status: string) => {
    switch (status) {
      case 'running':
        return 'default';
      case 'succeeded':
        return 'default';
      case 'failed':
        return 'destructive';
      case 'cancelled':
        return 'secondary';
      case 'pending':
        return 'outline';
      default:
        return 'outline';
    }
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  if (error || !execution) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          {error instanceof Error ? error.message : 'Failed to load execution'}
        </AlertDescription>
      </Alert>
    );
  }

  const sandboxExec = execution as unknown as SandboxExecution;
  const isRunning = execution.status === 'running';
  const isFailed = execution.status === 'failed';
  const memoryPercent = sandboxExec.memory_limit_mb && sandboxExec.memory_used_mb
    ? (sandboxExec.memory_used_mb / sandboxExec.memory_limit_mb) * 100
    : 0;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2">
              Execution Status
              {getStatusIcon(execution.status)}
            </CardTitle>
            <CardDescription>
              {execution.startedAt && (
                <span>
                  Running for {formatDuration(execution.startedAt, execution.completedAt)}
                </span>
              )}
            </CardDescription>
          </div>
          <Badge variant={getStatusVariant(execution.status)}>
            {execution.status}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Container Info */}
        {sandboxExec.container_id && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Container</h4>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <span className="text-muted-foreground">ID:</span>{' '}
                <code className="text-xs">{sandboxExec.container_id.substring(0, 12)}</code>
              </div>
              {sandboxExec.container_image && (
                <div>
                  <span className="text-muted-foreground">Image:</span>{' '}
                  <code className="text-xs">{sandboxExec.container_image}</code>
                </div>
              )}
              {sandboxExec.container_status && (
                <div>
                  <span className="text-muted-foreground">Status:</span>{' '}
                  <Badge variant="outline" className="ml-1 text-xs">
                    {sandboxExec.container_status}
                  </Badge>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Resource Usage */}
        {(sandboxExec.memory_used_mb || sandboxExec.cpu_usage_percent) && (
          <div className="space-y-4">
            <h4 className="text-sm font-medium">Resource Usage</h4>

            {/* Memory */}
            {sandboxExec.memory_used_mb && sandboxExec.memory_limit_mb && (
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <div className="flex items-center gap-2">
                    <HardDrive className="h-4 w-4 text-muted-foreground" />
                    <span>Memory</span>
                  </div>
                  <span className="text-muted-foreground">
                    {sandboxExec.memory_used_mb}MB / {sandboxExec.memory_limit_mb}MB
                  </span>
                </div>
                <Progress value={memoryPercent} className="h-2" />
              </div>
            )}

            {/* CPU */}
            {sandboxExec.cpu_usage_percent !== undefined && (
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <div className="flex items-center gap-2">
                    <Cpu className="h-4 w-4 text-muted-foreground" />
                    <span>CPU</span>
                  </div>
                  <span className="text-muted-foreground">
                    {sandboxExec.cpu_usage_percent.toFixed(1)}%
                    {sandboxExec.cpu_limit_cores && ` (${sandboxExec.cpu_limit_cores} cores)`}
                  </span>
                </div>
                <Progress value={sandboxExec.cpu_usage_percent} className="h-2" />
              </div>
            )}
          </div>
        )}

        {/* Error Message */}
        {execution.errorMessage && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{execution.errorMessage}</AlertDescription>
          </Alert>
        )}

        {/* Action Buttons */}
        <div className="flex gap-2">
          {isRunning && (
            <Button
              variant="destructive"
              size="sm"
              onClick={handleStop}
              disabled={isStopping}
            >
              {isStopping ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <StopCircle className="mr-2 h-4 w-4" />
              )}
              Stop Execution
            </Button>
          )}

          {isFailed && onRetry && (
            <Button variant="outline" size="sm" onClick={handleRetry}>
              <RefreshCw className="mr-2 h-4 w-4" />
              Retry
            </Button>
          )}

          <Button variant="outline" size="sm" onClick={() => refetch()}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
