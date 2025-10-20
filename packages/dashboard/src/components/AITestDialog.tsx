// ABOUTME: Test dialog for AI integration - demonstrates all AI operations
// ABOUTME: Use this component to test PRD analysis, spec generation, and task suggestions

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import {
  useAnalyzePRD,
  useGenerateSpec,
  useSuggestTasks,
  useAIConfiguration,
  usePRDWorkflow,
} from '@/hooks/useAI';
import { Loader2, CheckCircle, XCircle, DollarSign, Sparkles } from 'lucide-react';

interface AITestDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function AITestDialog({ open, onOpenChange }: AITestDialogProps) {
  const [prdContent, setPrdContent] = useState(`# User Authentication System

## Overview
Build a secure authentication system for web applications.

## Requirements
1. Users must be able to register with email and password
2. Users must be able to log in with credentials
3. Users must be able to reset forgotten passwords
4. Sessions must expire after 24 hours of inactivity
5. Support OAuth login with Google and GitHub

## Technical Constraints
- Use JWT tokens for session management
- Passwords must be hashed with bcrypt
- Rate limit login attempts to prevent brute force
`);

  const [workflowProgress, setWorkflowProgress] = useState({ step: '', progress: 0 });

  const { data: config } = useAIConfiguration();
  const analyzePRDMutation = useAnalyzePRD();
  const prdWorkflowMutation = usePRDWorkflow();

  const handleAnalyzePRD = () => {
    analyzePRDMutation.mutate(prdContent);
  };

  const handleFullWorkflow = () => {
    prdWorkflowMutation.mutate({
      prdId: 'test-prd-1',
      prdContent,
      projectId: 'test-project',
      onProgress: (step, progress) => {
        setWorkflowProgress({ step, progress });
      },
    });
  };

  const formatCost = (cost: number) => `$${cost.toFixed(4)}`;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>AI Integration Test</DialogTitle>
          <DialogDescription>
            Test OpenSpec AI integration with PRD analysis and spec generation.
          </DialogDescription>
        </DialogHeader>

        {/* Configuration Status */}
        <Alert>
          <Sparkles className="h-4 w-4" />
          <AlertDescription>
            {config?.isConfigured ? (
              <div className="space-y-1">
                <div className="font-medium text-green-600">✓ AI is configured</div>
                <div className="text-sm">
                  Provider: {config.preferredProvider === 'anthropic' ? 'Anthropic Claude' : 'OpenAI'} (
                  {config.openaiConfigured && 'OpenAI'}{' '}
                  {config.anthropicConfigured && config.openaiConfigured && '+ '}
                  {config.anthropicConfigured && 'Anthropic'})
                </div>
              </div>
            ) : (
              <div className="space-y-1">
                <div className="font-medium text-amber-600">⚠ AI not configured</div>
                <div className="text-sm">
                  Please set VITE_OPENAI_API_KEY or VITE_ANTHROPIC_API_KEY in your .env file
                </div>
              </div>
            )}
          </AlertDescription>
        </Alert>

        <Tabs defaultValue="analyze" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="analyze">Quick Analysis</TabsTrigger>
            <TabsTrigger value="workflow">Full Workflow</TabsTrigger>
          </TabsList>

          {/* Quick Analysis Tab */}
          <TabsContent value="analyze" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="prd-content">PRD Content</Label>
              <Textarea
                id="prd-content"
                value={prdContent}
                onChange={(e) => setPrdContent(e.target.value)}
                placeholder="Paste your PRD content here..."
                className="min-h-[200px] font-mono text-sm"
              />
            </div>

            <Button
              onClick={handleAnalyzePRD}
              disabled={!config?.isConfigured || analyzePRDMutation.isPending || !prdContent.trim()}
            >
              {analyzePRDMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Analyzing...
                </>
              ) : (
                'Analyze PRD'
              )}
            </Button>

            {/* Analysis Results */}
            {analyzePRDMutation.data && (
              <div className="space-y-4 rounded-lg border p-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold">Analysis Results</h3>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <DollarSign className="h-4 w-4" />
                    {formatCost(analyzePRDMutation.data.cost.estimatedCost)}
                    <span className="text-xs">
                      ({analyzePRDMutation.data.usage.totalTokens} tokens)
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <div>
                    <div className="text-sm font-medium">Summary</div>
                    <div className="text-sm text-muted-foreground">
                      {analyzePRDMutation.data.data.summary}
                    </div>
                  </div>

                  <div>
                    <div className="text-sm font-medium mb-2">
                      Capabilities ({analyzePRDMutation.data.data.capabilities.length})
                    </div>
                    {analyzePRDMutation.data.data.capabilities.map((cap) => (
                      <div key={cap.id} className="ml-4 mb-3 space-y-1">
                        <div className="font-medium text-sm">
                          {cap.name} <span className="text-muted-foreground">({cap.id})</span>
                        </div>
                        <div className="text-sm text-muted-foreground">{cap.purpose}</div>
                        <div className="text-xs text-muted-foreground">
                          {cap.requirements.length} requirements,{' '}
                          {cap.requirements.reduce((sum, r) => sum + r.scenarios.length, 0)} scenarios
                        </div>
                      </div>
                    ))}
                  </div>

                  {analyzePRDMutation.data.data.dependencies &&
                    analyzePRDMutation.data.data.dependencies.length > 0 && (
                      <div>
                        <div className="text-sm font-medium">Dependencies</div>
                        <div className="text-sm text-muted-foreground">
                          {analyzePRDMutation.data.data.dependencies.join(', ')}
                        </div>
                      </div>
                    )}
                </div>
              </div>
            )}

            {analyzePRDMutation.error && (
              <Alert variant="destructive">
                <XCircle className="h-4 w-4" />
                <AlertDescription>
                  Error: {analyzePRDMutation.error.message}
                </AlertDescription>
              </Alert>
            )}
          </TabsContent>

          {/* Full Workflow Tab */}
          <TabsContent value="workflow" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="workflow-prd">PRD Content</Label>
              <Textarea
                id="workflow-prd"
                value={prdContent}
                onChange={(e) => setPrdContent(e.target.value)}
                placeholder="Paste your PRD content here..."
                className="min-h-[200px] font-mono text-sm"
              />
            </div>

            <Button
              onClick={handleFullWorkflow}
              disabled={!config?.isConfigured || prdWorkflowMutation.isPending || !prdContent.trim()}
            >
              {prdWorkflowMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Running Workflow...
                </>
              ) : (
                'Run Full PRD → Spec → Task Workflow'
              )}
            </Button>

            {/* Workflow Progress */}
            {prdWorkflowMutation.isPending && (
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">{workflowProgress.step}</span>
                  <span className="font-medium">{workflowProgress.progress}%</span>
                </div>
                <Progress value={workflowProgress.progress} />
              </div>
            )}

            {/* Workflow Results */}
            {prdWorkflowMutation.data && (
              <div className="space-y-4 rounded-lg border p-4">
                <div className="flex items-center gap-2">
                  <CheckCircle className="h-5 w-5 text-green-600" />
                  <h3 className="font-semibold">Workflow Complete</h3>
                </div>

                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span>Capabilities Generated:</span>
                    <span className="font-medium">{prdWorkflowMutation.data.capabilities.length}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Tasks Suggested:</span>
                    <span className="font-medium">{prdWorkflowMutation.data.suggestedTasks.length}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Total Cost:</span>
                    <span className="font-medium">{formatCost(prdWorkflowMutation.data.totalCost)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Steps Executed:</span>
                    <span className="font-medium">{prdWorkflowMutation.data.steps.length}</span>
                  </div>
                </div>

                <div className="space-y-2">
                  <div className="text-sm font-medium">Generated Capabilities</div>
                  {prdWorkflowMutation.data.capabilities.map((cap) => {
                    // Find tasks for this capability
                    const capabilityTasks = prdWorkflowMutation.data.suggestedTasks.filter(
                      task => task.capabilityId === cap.capability.id
                    );

                    return (
                      <details key={cap.id} className="ml-4">
                        <summary className="cursor-pointer text-sm font-medium">
                          {cap.capability.name}
                          {capabilityTasks.length > 0 && (
                            <span className="ml-2 text-xs text-muted-foreground">
                              ({capabilityTasks.length} {capabilityTasks.length === 1 ? 'task' : 'tasks'})
                            </span>
                          )}
                        </summary>
                        <div className="mt-2 space-y-3">
                          <pre className="max-h-[200px] overflow-auto rounded bg-muted p-2 text-xs">
                            {cap.specMarkdown}
                          </pre>

                          {capabilityTasks.length > 0 && (
                            <div className="space-y-2">
                              <div className="text-xs font-medium text-muted-foreground">Implementation Tasks:</div>
                              {capabilityTasks.map((task, idx) => (
                                <div key={idx} className="p-2 rounded-md border bg-card text-card-foreground">
                                  <p className="font-medium text-xs">{task.title}</p>
                                  <p className="text-xs text-muted-foreground mt-1">{task.description}</p>
                                  <div className="flex gap-2 mt-1">
                                    <span className="text-xs px-1.5 py-0.5 rounded bg-secondary">
                                      Complexity: {task.complexity}/10
                                    </span>
                                    {task.priority && (
                                      <span className="text-xs px-1.5 py-0.5 rounded bg-secondary">
                                        Priority: {task.priority}
                                      </span>
                                    )}
                                  </div>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      </details>
                    );
                  })}
                </div>
              </div>
            )}

            {prdWorkflowMutation.error && (
              <Alert variant="destructive">
                <XCircle className="h-4 w-4" />
                <AlertDescription>Error: {prdWorkflowMutation.error.message}</AlertDescription>
              </Alert>
            )}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
