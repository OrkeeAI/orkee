// ABOUTME: TDD Task View component displaying test strategies, execution steps, and file references
// ABOUTME: Shows comprehensive task execution information with copyable test commands

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  CheckCircle2,
  Clock,
  FileText,
  Copy,
  Check,
  Terminal,
  Link as LinkIcon,
} from 'lucide-react';
import type { TaskExecutionSteps } from '@/services/tasks';

interface TDDTaskViewProps {
  executionSteps: TaskExecutionSteps | null;
  onGenerateSteps?: () => void;
  isGenerating?: boolean;
}

export function TDDTaskView({
  executionSteps,
  onGenerateSteps,
  isGenerating = false,
}: TDDTaskViewProps) {
  const [copiedCommands, setCopiedCommands] = useState<Set<number>>(new Set());

  if (!executionSteps) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center py-12">
          <Terminal className="h-12 w-12 text-muted-foreground mb-4" />
          <p className="text-lg font-medium text-muted-foreground mb-4">
            No execution steps available
          </p>
          {onGenerateSteps && (
            <Button onClick={onGenerateSteps} disabled={isGenerating}>
              {isGenerating ? 'Generating...' : 'Generate Execution Steps'}
            </Button>
          )}
        </CardContent>
      </Card>
    );
  }

  const handleCopyCommand = async (stepNumber: number, command: string) => {
    try {
      await navigator.clipboard.writeText(command);
      setCopiedCommands((prev) => new Set(prev).add(stepNumber));
      setTimeout(() => {
        setCopiedCommands((prev) => {
          const next = new Set(prev);
          next.delete(stepNumber);
          return next;
        });
      }, 2000);
    } catch (err) {
      console.error('Failed to copy command:', err);
    }
  };

  const getOperationBadgeColor = (operation: string) => {
    switch (operation) {
      case 'create':
        return 'bg-green-500';
      case 'modify':
        return 'bg-blue-500';
      case 'delete':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  const totalTime = executionSteps.steps.reduce(
    (sum, step) => sum + step.estimatedMinutes,
    0
  );

  return (
    <div className="space-y-4">
      {/* Test Strategy Card */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5" />
            Test Strategy
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm whitespace-pre-wrap">{executionSteps.testStrategy}</p>
        </CardContent>
      </Card>

      {/* Acceptance Criteria Card */}
      {executionSteps.acceptanceCriteria.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Acceptance Criteria</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2">
              {executionSteps.acceptanceCriteria.map((criterion, index) => (
                <li key={index} className="flex items-start gap-2">
                  <CheckCircle2 className="h-4 w-4 text-green-600 mt-0.5 flex-shrink-0" />
                  <span className="text-sm">{criterion}</span>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      {/* Execution Steps Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg flex items-center gap-2">
              <Terminal className="h-5 w-5" />
              Execution Steps
            </CardTitle>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Clock className="h-4 w-4" />
              <span>~{totalTime} min total</span>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {executionSteps.steps.map((step) => (
              <div
                key={step.stepNumber}
                className="border rounded-lg p-4 space-y-3"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-3">
                    <Badge variant="outline" className="mt-0.5">
                      {step.stepNumber}
                    </Badge>
                    <div className="space-y-1">
                      <p className="font-medium">{step.action}</p>
                      <p className="text-sm text-muted-foreground">
                        Expected: {step.expectedOutput}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Clock className="h-3 w-3" />
                    <span>~{step.estimatedMinutes}m</span>
                  </div>
                </div>

                {step.testCommand && (
                  <div className="bg-muted/50 rounded p-3">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-xs font-medium text-muted-foreground">
                        Command
                      </span>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-6 px-2"
                        onClick={() =>
                          handleCopyCommand(step.stepNumber, step.testCommand!)
                        }
                      >
                        {copiedCommands.has(step.stepNumber) ? (
                          <>
                            <Check className="h-3 w-3 mr-1" />
                            Copied
                          </>
                        ) : (
                          <>
                            <Copy className="h-3 w-3 mr-1" />
                            Copy
                          </>
                        )}
                      </Button>
                    </div>
                    <code className="text-xs font-mono">{step.testCommand}</code>
                  </div>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* File References Card */}
      {executionSteps.relevantFiles.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base flex items-center gap-2">
              <FileText className="h-5 w-5" />
              Files to Work On
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {executionSteps.relevantFiles.map((file, index) => (
                <div
                  key={index}
                  className="flex items-start gap-3 p-2 rounded border"
                >
                  <Badge
                    className={`${getOperationBadgeColor(file.operation)} text-white border-0`}
                  >
                    {file.operation}
                  </Badge>
                  <div className="flex-1 min-w-0">
                    <p className="font-mono text-sm truncate">{file.path}</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      {file.reason}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Similar Implementations Card */}
      {executionSteps.similarImplementations.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base flex items-center gap-2">
              <LinkIcon className="h-5 w-5" />
              Similar Implementations
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Alert>
              <AlertDescription>
                <p className="text-sm mb-2">
                  Reference these existing implementations for guidance:
                </p>
                <ul className="space-y-1">
                  {executionSteps.similarImplementations.map((impl, index) => (
                    <li key={index} className="text-sm font-mono">
                      • {impl}
                    </li>
                  ))}
                </ul>
              </AlertDescription>
            </Alert>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
