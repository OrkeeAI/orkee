// Work stream analysis component for parallel execution planning
import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { WorkAnalysis } from '@/services/epics';
import { AlertCircle, CheckCircle2, Layers } from 'lucide-react';

interface WorkStreamAnalysisProps {
  workAnalysis: WorkAnalysis | null;
}

export function WorkStreamAnalysis({ workAnalysis }: WorkStreamAnalysisProps) {
  if (!workAnalysis) {
    return (
      <Card>
        <CardContent className="p-6">
          <p className="text-sm text-muted-foreground">
            No work stream analysis available. Analyze work streams to view parallelization strategy.
          </p>
        </CardContent>
      </Card>
    );
  }

  const confidencePercent = (workAnalysis.confidenceScore || 0) * 100;

  return (
    <div className="space-y-6">
      {/* Analysis Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Work Stream Analysis</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Work Streams</p>
                <p className="text-2xl font-bold">{workAnalysis.parallelStreams.length}</p>
              </div>
              <div>
                <p className="text-sm font-medium text-muted-foreground">Total Tasks</p>
                <p className="text-2xl font-bold">
                  {workAnalysis.parallelStreams.reduce((sum, stream) => sum + stream.tasks.length, 0)}
                </p>
              </div>
              <div>
                <p className="text-sm font-medium text-muted-foreground">Conflicts</p>
                <p className="text-2xl font-bold">{workAnalysis.conflictAnalysis?.conflicts.length || 0}</p>
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <p className="text-sm font-medium">Confidence Score</p>
                <p className="text-sm font-bold">{confidencePercent.toFixed(0)}%</p>
              </div>
              <Progress value={confidencePercent} className="h-2" />
            </div>

            <div className="text-xs text-muted-foreground">
              Analyzed on {new Date(workAnalysis.analyzedAt).toLocaleDateString()}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Parallel Streams */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Layers className="h-5 w-5" />
            Parallel Work Streams
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {workAnalysis.parallelStreams.map((stream, index) => (
              <div key={index} className="p-4 rounded-lg border bg-muted/30">
                <div className="flex items-start justify-between mb-3">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <Badge variant="default">Stream {index + 1}</Badge>
                      <h3 className="font-medium">{stream.name}</h3>
                    </div>
                    <p className="text-sm text-muted-foreground">{stream.description}</p>
                  </div>
                  <Badge variant="outline">{stream.tasks.length} tasks</Badge>
                </div>

                {stream.filePatterns && stream.filePatterns.length > 0 && (
                  <div className="mt-3 pt-3 border-t">
                    <p className="text-xs font-medium text-muted-foreground mb-2">File Patterns:</p>
                    <div className="flex flex-wrap gap-1">
                      {stream.filePatterns.map((pattern, i) => (
                        <Badge key={i} variant="secondary" className="text-xs">
                          {pattern}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}

                <div className="mt-3 pt-3 border-t">
                  <p className="text-xs font-medium text-muted-foreground mb-2">Tasks in this stream:</p>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {stream.tasks.slice(0, 6).map((taskId, i) => (
                      <div key={i} className="text-xs p-2 rounded border bg-background">
                        Task ID: {taskId.substring(0, 8)}...
                      </div>
                    ))}
                    {stream.tasks.length > 6 && (
                      <div className="text-xs p-2 rounded border bg-background text-center text-muted-foreground">
                        +{stream.tasks.length - 6} more
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Parallelization Strategy */}
      {workAnalysis.parallelizationStrategy && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <CheckCircle2 className="h-5 w-5 text-green-600" />
              Recommended Strategy
            </CardTitle>
          </CardHeader>
          <CardContent>
            <pre className="text-sm whitespace-pre-wrap font-mono bg-muted p-4 rounded-lg">
              {workAnalysis.parallelizationStrategy}
            </pre>
          </CardContent>
        </Card>
      )}

      {/* Conflicts */}
      {workAnalysis.conflictAnalysis && workAnalysis.conflictAnalysis.conflicts.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <AlertCircle className="h-5 w-5 text-orange-600" />
              Detected Conflicts
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {workAnalysis.conflictAnalysis.conflicts.map((conflict, index) => (
                <div key={index} className="p-3 rounded-lg border border-orange-200 bg-orange-50">
                  <div className="flex items-start gap-2">
                    <AlertCircle className="h-4 w-4 mt-0.5 text-orange-600" />
                    <div className="flex-1">
                      <p className="text-sm font-medium">
                        Tasks {conflict.task1.substring(0, 8)}... ↔ {conflict.task2.substring(0, 8)}...
                      </p>
                      <p className="text-xs text-muted-foreground mt-1">{conflict.reason}</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
