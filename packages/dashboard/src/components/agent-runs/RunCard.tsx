// ABOUTME: Reusable card component for displaying an agent run summary
// ABOUTME: Shows run title, status badge, progress bar, cost, and iteration info

import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { RunStatusBadge } from '@/components/agent-runs/RunStatusBadge'
import type { AgentRun } from '@/services/agent-runs'

interface RunCardProps {
  run: AgentRun
  onClick: () => void
  showProject?: boolean
}

export function RunCard({ run, onClick, showProject }: RunCardProps) {
  const progress = run.storiesTotal > 0
    ? (run.storiesCompleted / run.storiesTotal) * 100
    : 0

  return (
    <Card
      className="cursor-pointer hover:border-primary/50 transition-colors"
      onClick={onClick}
    >
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">
            {run.prdJson?.description || run.id}
          </CardTitle>
          <RunStatusBadge status={run.status} />
        </div>
        <CardDescription>
          {showProject && <>{run.prdJson?.project || 'Unknown project'} &middot; </>}
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
  )
}
