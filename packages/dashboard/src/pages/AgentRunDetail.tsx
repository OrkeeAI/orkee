// ABOUTME: Detail view for a single agent run with live event streaming
// ABOUTME: Shows story board, live agent feed, run controls, and iteration summaries

import { useParams, useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { StoryBoard } from '@/components/agent-runs/StoryBoard'
import { AgentFeed } from '@/components/agent-runs/AgentFeed'
import { RunStatusBadge } from '@/components/agent-runs/RunStatusBadge'
import { useAgentRunEvents } from '@/hooks/useAgentRunEvents'
import { stopRun, deleteRun } from '@/services/agent-runs'
import { ChevronLeft, Square, Trash2 } from 'lucide-react'
import { useState, useMemo } from 'react'
import { toast } from 'sonner'

export default function AgentRunDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { events, run, connectionMode, refetchRun } = useAgentRunEvents(id || null)
  const [stopping, setStopping] = useState(false)

  // Derive active story from the latest iteration_started event
  const activeStoryId = useMemo(() => {
    for (let i = events.length - 1; i >= 0; i--) {
      const e = events[i]
      if (e.type === 'iteration_started') {
        return e.story_id
      }
      if (e.type === 'iteration_completed' || e.type === 'iteration_failed') {
        return undefined // iteration ended, no active story between iterations
      }
    }
    return undefined
  }, [events])

  const handleStop = async () => {
    if (!id) return
    setStopping(true)
    try {
      await stopRun(id)
      await refetchRun()
      toast.success('Run stopped')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to stop run')
    } finally {
      setStopping(false)
    }
  }

  const handleDelete = async () => {
    if (!id) return
    if (!confirm('Delete this run? This cannot be undone.')) return
    try {
      await deleteRun(id)
      toast.success('Run deleted')
      navigate('/agent-runs')
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to delete run')
    }
  }

  if (!run) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>
    )
  }

  const isActive = run.status === 'running' || run.status === 'pending'

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={() => navigate('/agent-runs')}>
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <div>
            <h1 className="text-xl font-bold">{run.prdJson?.description || run.id}</h1>
            <p className="text-sm text-muted-foreground">
              {run.prdJson?.project} &middot; {connectionMode === 'sse' ? 'Live' : connectionMode}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <RunStatusBadge status={run.status} />
          <span className="text-sm font-mono">${run.totalCost.toFixed(2)}</span>
          {isActive && (
            <Button variant="destructive" size="sm" onClick={handleStop} disabled={stopping}>
              <Square className="h-4 w-4 mr-1" />
              Stop
            </Button>
          )}
          {!isActive && (
            <Button variant="ghost" size="sm" onClick={handleDelete}>
              <Trash2 className="h-4 w-4" />
            </Button>
          )}
        </div>
      </div>

      {/* Story board */}
      <Card>
        <CardContent className="pt-4">
          <StoryBoard
            stories={run.prdJson?.userStories || []}
            activeStoryId={activeStoryId}
          />
        </CardContent>
      </Card>

      {/* Stats row */}
      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-1 pt-3 px-4">
            <CardTitle className="text-xs text-muted-foreground">Iteration</CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-3">
            <span className="text-lg font-bold">{run.currentIteration}</span>
            <span className="text-sm text-muted-foreground">/{run.maxIterations}</span>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-1 pt-3 px-4">
            <CardTitle className="text-xs text-muted-foreground">Stories</CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-3">
            <span className="text-lg font-bold">{run.storiesCompleted}</span>
            <span className="text-sm text-muted-foreground">/{run.storiesTotal}</span>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-1 pt-3 px-4">
            <CardTitle className="text-xs text-muted-foreground">Cost</CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-3">
            <span className="text-lg font-bold">${run.totalCost.toFixed(2)}</span>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-1 pt-3 px-4">
            <CardTitle className="text-xs text-muted-foreground">Tokens</CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-3">
            <span className="text-lg font-bold">{run.totalTokens.toLocaleString()}</span>
          </CardContent>
        </Card>
      </div>

      {/* Live feed */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base">Live Feed</CardTitle>
        </CardHeader>
        <CardContent>
          <AgentFeed events={events} />
        </CardContent>
      </Card>

      {/* Error display */}
      {run.error && (
        <Card className="border-destructive">
          <CardHeader className="pb-2">
            <CardTitle className="text-base text-destructive">Error</CardTitle>
          </CardHeader>
          <CardContent>
            <pre className="text-sm whitespace-pre-wrap text-destructive/80">{run.error}</pre>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
