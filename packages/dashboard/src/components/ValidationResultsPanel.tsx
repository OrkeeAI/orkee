// ABOUTME: Panel component for displaying task validation results against scenarios
// ABOUTME: Shows pass/fail status for each scenario with confidence levels and notes

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import { CheckCircle, XCircle, AlertCircle, Info } from 'lucide-react';
import type { TaskValidation } from '@/lib/ai/schemas';

interface ValidationResultsPanelProps {
  validation: TaskValidation;
  className?: string;
}

export function ValidationResultsPanel({ validation, className }: ValidationResultsPanelProps) {
  const passedCount = validation.scenarioResults.filter((r) => r.passed).length;
  const totalCount = validation.scenarioResults.length;
  const passRate = totalCount > 0 ? (passedCount / totalCount) * 100 : 0;
  const overallPassed = validation.overallAssessment.passed;

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Validation Results</CardTitle>
            <CardDescription>Task validation against spec scenarios</CardDescription>
          </div>
          <Badge variant={overallPassed ? 'default' : 'destructive'} className="text-lg px-4 py-2">
            {overallPassed ? (
              <>
                <CheckCircle className="mr-2 h-5 w-5" />
                Passed
              </>
            ) : (
              <>
                <XCircle className="mr-2 h-5 w-5" />
                Failed
              </>
            )}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-6">
        {/* Overall Progress */}
        <div className="space-y-2">
          <div className="flex justify-between text-sm">
            <span className="font-medium">Overall Pass Rate</span>
            <span className="text-muted-foreground">
              {passedCount} / {totalCount} scenarios ({Math.round(passRate)}%)
            </span>
          </div>
          <Progress value={passRate} className="h-2" />
        </div>

        {/* Overal Assessment */}
        <Alert variant={overallPassed ? 'default' : 'destructive'}>
          <Info className="h-4 w-4" />
          <AlertTitle>Overall Assessment</AlertTitle>
          <AlertDescription>{validation.overallAssessment.notes}</AlertDescription>
        </Alert>

        {/* Recommendations */}
        {validation.recommendations && validation.recommendations.length > 0 && (
          <div className="space-y-2">
            <h4 className="font-medium text-sm">Recommendations</h4>
            <ul className="space-y-1">
              {validation.recommendations.map((rec, index) => (
                <li key={index} className="text-sm text-muted-foreground flex items-start gap-2">
                  <AlertCircle className="h-4 w-4 mt-0.5 flex-shrink-0" />
                  <span>{rec}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        <Separator />

        {/* Scenario Results */}
        <div className="space-y-4">
          <h4 className="font-medium">Scenario Results</h4>
          {validation.scenarioResults.map((result, index) => (
            <div
              key={index}
              className={`rounded-lg border p-4 space-y-3 ${
                result.passed ? 'border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950' : 'border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950'
              }`}
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 space-y-1">
                  <div className="flex items-center gap-2">
                    <h5 className="font-medium">{result.scenarioName}</h5>
                    <Badge variant={result.passed ? 'default' : 'destructive'}>
                      {result.passed ? (
                        <>
                          <CheckCircle className="mr-1 h-3 w-3" />
                          Passed
                        </>
                      ) : (
                        <>
                          <XCircle className="mr-1 h-3 w-3" />
                          Failed
                        </>
                      )}
                    </Badge>
                  </div>
                  {result.notes && (
                    <p className="text-sm text-muted-foreground">{result.notes}</p>
                  )}
                </div>
                <div className="text-right">
                  <div className="text-xs text-muted-foreground">Confidence</div>
                  <div className="text-xl font-bold">
                    {Math.round(result.confidence * 100)}%
                  </div>
                </div>
              </div>

              {/* Confidence Bar */}
              <div className="space-y-1">
                <Progress
                  value={result.confidence * 100}
                  className={`h-1.5 ${
                    result.confidence > 0.8
                      ? 'bg-green-200 dark:bg-green-900'
                      : result.confidence > 0.5
                      ? 'bg-yellow-200 dark:bg-yellow-900'
                      : 'bg-red-200 dark:bg-red-900'
                  }`}
                />
                <p className="text-xs text-muted-foreground">
                  {result.confidence > 0.8
                    ? 'High confidence'
                    : result.confidence > 0.5
                    ? 'Medium confidence'
                    : 'Low confidence'}
                </p>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
