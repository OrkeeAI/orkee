// ABOUTME: Agent runs list page showing all autonomous coding sessions
// ABOUTME: Displays run cards with status, progress, cost and links to detail view

import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { RunStatusBadge } from '@/components/agent-runs/RunStatusBadge'
import { listRuns, type AgentRun } from '@/services/agent-runs'
import { RefreshCw, Plus } from 'lucide-react'

export default function AgentRuns() {
  const navigate = useNavigate()
  const [runs, setRuns] = useState<AgentRun[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadRuns = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await listRuns()
      // listRuns returns PaginatedResponse; extract items
      const items = Array.isArray(result) ? result : (result as { items?: AgentRun[] }).items || []
      setRuns(items)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load runs')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadRuns()
  }, [loadRuns])

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Agent Runs</h1>
          <p className="text-muted-foreground">Autonomous coding sessions powered by Ralph</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={loadRuns} disabled={loading}>
            <RefreshCw className={`h-4 w-4 mr-1 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button size="sm" onClick={() => navigate('/agent-runs/new')}>
            <Plus className="h-4 w-4 mr-1" />
            New Run
          </Button>
        </div>
      </div>

      {error && (
        <div className="bg-destructive/10 text-destructive rounded-md p-3 text-sm">
          {error}
        </div>
      )}

      {!loading && runs.length === 0 && !error && (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <p className="text-muted-foreground mb-4">No agent runs yet</p>
            <Button onClick={() => navigate('/agent-runs/new')}>
              <Plus className="h-4 w-4 mr-1" />
              Start your first run
            </Button>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-4">
        {runs.map(run => {
          const progress = run.storiesTotal > 0
            ? (run.storiesCompleted / run.storiesTotal) * 100
            : 0

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
                  {run.prdJson?.project || 'Unknown project'} &middot; Started {run.startedAt ? new Date(run.startedAt).toLocaleString() : 'pending'}
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
          )
        })}
      </div>
    </div>
  )
}
