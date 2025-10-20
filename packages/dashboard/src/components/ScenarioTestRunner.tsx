// ABOUTME: Component for running and validating tasks against spec scenarios
// ABOUTME: Allows users to test task implementations against defined WHEN/THEN/AND scenarios

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Play, CheckCircle, XCircle, Loader2, AlertCircle } from 'lucide-react';
import { useMutation } from '@tanstack/react-query';
import { aiSpecService } from '@/lib/ai/services';
import { ValidationResultsPanel } from './ValidationResultsPanel';
import type { TaskValidation } from '@/lib/ai/schemas';

interface Scenario {
  name: string;
  when: string;
  then: string;
  and?: string[];
}

interface ScenarioTestRunnerProps {
  taskId: string;
  taskTitle: string;
  taskDescription: string;
  scenarios: Scenario[];
}

export function ScenarioTestRunner({
  taskId,
  taskTitle,
  taskDescription,
  scenarios,
}: ScenarioTestRunnerProps) {
  const [implementation, setImplementation] = useState('');
  const [validationResult, setValidationResult] = useState<TaskValidation | null>(null);

  const validateMutation = useMutation({
    mutationFn: async (impl: string) => {
      const result = await aiSpecService.validateTaskCompletion(
        {
          title: taskTitle,
          description: taskDescription,
          implementation: impl,
        },
        scenarios
      );
      return result;
    },
    onSuccess: (result) => {
      setValidationResult(result.data);
    },
  });

  const handleRunTests = () => {
    if (!implementation.trim()) {
      return;
    }
    validateMutation.mutate(implementation);
  };

  const canRun = implementation.trim() && !validateMutation.isPending;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Scenario Test Runner</CardTitle>
          <CardDescription>
            Test task implementation against {scenarios.length} scenario{scenarios.length !== 1 ? 's' : ''}
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* Task Info */}
          <div className="rounded-lg border p-4 space-y-2">
            <h4 className="font-medium">{taskTitle}</h4>
            <p className="text-sm text-muted-foreground">{taskDescription}</p>
          </div>

          {/* Scenarios to Test */}
          <div className="space-y-2">
            <h4 className="font-medium text-sm">Scenarios to Test:</h4>
            <div className="space-y-2">
              {scenarios.map((scenario, idx) => (
                <div key={idx} className="rounded-lg border p-3 space-y-1 text-sm">
                  <div className="font-medium">{scenario.name}</div>
                  <div className="text-muted-foreground">
                    <span className="font-mono">WHEN</span> {scenario.when}
                  </div>
                  <div className="text-muted-foreground">
                    <span className="font-mono">THEN</span> {scenario.then}
                  </div>
                  {scenario.and && scenario.and.length > 0 && (
                    <div className="text-muted-foreground pl-4">
                      {scenario.and.map((clause, cidx) => (
                        <div key={cidx}>
                          <span className="font-mono">AND</span> {clause}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Implementation Input */}
          <div className="space-y-2">
            <h4 className="font-medium text-sm">Implementation Details:</h4>
            <Textarea
              placeholder="Describe what you implemented, how it works, and any relevant details...

Example:
- Implemented user authentication using JWT tokens
- Added middleware to validate tokens on protected routes
- Tokens expire after 24 hours
- Invalid tokens return 401 Unauthorized
- etc."
              value={implementation}
              onChange={(e) => setImplementation(e.target.value)}
              className="min-h-[200px] font-mono text-sm"
            />
            <p className="text-xs text-muted-foreground">
              Provide implementation details to validate against scenarios
            </p>
          </div>

          {/* Error Messages */}
          {validateMutation.isError && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                {validateMutation.error instanceof Error
                  ? validateMutation.error.message
                  : 'Failed to validate task implementation'}
              </AlertDescription>
            </Alert>
          )}

          {/* Run Button */}
          <div className="flex justify-end">
            <Button
              onClick={handleRunTests}
              disabled={!canRun}
              size="lg"
            >
              {validateMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Running Tests...
                </>
              ) : (
                <>
                  <Play className="mr-2 h-4 w-4" />
                  Run Scenario Tests
                </>
              )}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Results */}
      {validationResult && (
        <ValidationResultsPanel validation={validationResult} />
      )}

      {/* Cost Info */}
      {validateMutation.isSuccess && validateMutation.data && (
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription className="flex items-center justify-between">
            <span>AI validation completed</span>
            <Badge variant="secondary">
              Cost: ${validateMutation.data.cost.estimatedCost.toFixed(4)}
            </Badge>
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
}
