// ABOUTME: Project-scoped agent runs list showing all autonomous coding sessions for a project
// ABOUTME: Displays run cards with status, progress, cost and navigation to detail view
import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { RunStatusBadge } from '@/components/agent-runs/RunStatusBadge';
import { listRuns, type AgentRun } from '@/services/agent-runs';
import { RefreshCw, Bot } from 'lucide-react';

interface RunsTabProps {
  projectId: string;
}

export function RunsTab({ projectId }: RunsTabProps) {
  const navigate = useNavigate();
  const [runs, setRuns] = useState<AgentRun[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadRuns = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const items = await listRuns(projectId);
      setRuns(items);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load runs');
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => {
    loadRuns();
  }, [loadRuns]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading runs...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Agent Runs</h3>
          <p className="text-sm text-muted-foreground">
            {runs.length} {runs.length === 1 ? 'run' : 'runs'} in this project
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={loadRuns} disabled={loading}>
          <RefreshCw className={`h-4 w-4 mr-1 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {error && (
        <div className="bg-destructive/10 text-destructive rounded-md p-3 text-sm">
          {error}
        </div>
      )}

      {runs.length === 0 && !error && (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Bot className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground mb-2">No agent runs yet</p>
            <p className="text-sm text-muted-foreground">
              Select a PRD and click &ldquo;Run Agent&rdquo; to start an autonomous coding session.
            </p>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-4">
        {runs.map(run => {
          const progress = run.storiesTotal > 0
            ? (run.storiesCompleted / run.storiesTotal) * 100
            : 0;

          return (
            <Card
              key={run.id}
              className="cursor-pointer hover:border-primary/50 transition-colors"
              onClick={() => navigate(`/agent-runs/${run.id}`)}
            >
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base">
                    {run.prdJson?.description || run.id}
                  </CardTitle>
                  <RunStatusBadge status={run.status} />
                </div>
                <CardDescription>
                  Started {run.startedAt ? new Date(run.startedAt).toLocaleString() : 'pending'}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span>
                      {run.storiesCompleted}/{run.storiesTotal} stories
                    </span>
                    <span className="text-muted-foreground">
                      ${run.totalCost.toFixed(2)} &middot; Iteration {run.currentIteration}/{run.maxIterations}
                    </span>
                  </div>
                  <Progress value={progress} className="h-2" />
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
