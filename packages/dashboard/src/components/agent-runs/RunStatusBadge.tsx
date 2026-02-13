// ABOUTME: Color-coded status badge for agent run states
// ABOUTME: Maps run status to visual indicators (pending, running, completed, failed, cancelled)

import { Badge } from '@/components/ui/badge'
import type { AgentRunStatus } from '@/services/agent-runs'

const statusConfig: Record<AgentRunStatus, { label: string; variant: 'default' | 'secondary' | 'destructive' | 'outline' }> = {
  pending: { label: 'Pending', variant: 'secondary' },
  running: { label: 'Running', variant: 'default' },
  completed: { label: 'Completed', variant: 'outline' },
  failed: { label: 'Failed', variant: 'destructive' },
  cancelled: { label: 'Cancelled', variant: 'secondary' },
}

interface RunStatusBadgeProps {
  status: AgentRunStatus
}

export function RunStatusBadge({ status }: RunStatusBadgeProps) {
  const config = statusConfig[status]
  return (
    <Badge variant={config.variant}>
      {status === 'running' && (
        <span className="mr-1 inline-block h-2 w-2 rounded-full bg-green-500 animate-pulse" />
      )}
      {config.label}
    </Badge>
  )
}
