// ABOUTME: Streaming activity feed displaying real-time agent text and tool usage events
// ABOUTME: Auto-scrolls to bottom, groups consecutive text events, and highlights tool invocations

import { useRef, useEffect } from 'react'
import { ScrollArea } from '@/components/ui/scroll-area'
import type { RunEvent } from '@/services/agent-runs'
import { cn } from '@/lib/utils'

interface AgentFeedProps {
  events: RunEvent[]
  className?: string
}

const toolIcons: Record<string, string> = {
  Read: '\uD83D\uDCD6',
  Edit: '\u270F\uFE0F',
  Write: '\uD83D\uDCDD',
  Bash: '\uD83D\uDCBB',
  Glob: '\uD83D\uDD0D',
  Grep: '\uD83D\uDD0D',
  Task: '\uD83D\uDCCB',
  WebFetch: '\uD83C\uDF10',
  WebSearch: '\uD83C\uDF10',
}

function FeedItem({ event }: { event: RunEvent }) {
  switch (event.type) {
    case 'agent_text':
      return (
        <div className="text-sm text-foreground/80 whitespace-pre-wrap">
          {event.text}
        </div>
      )
    case 'agent_tool': {
      const icon = toolIcons[event.tool] || '\u2699\uFE0F'
      return (
        <div className="flex items-center gap-2 text-xs text-muted-foreground font-mono py-0.5">
          <span>{icon}</span>
          <span className="font-semibold">{event.tool}</span>
          <span className="truncate">{event.detail}</span>
        </div>
      )
    }
    case 'iteration_started':
      return (
        <div className="border-t pt-2 mt-2 text-sm font-medium">
          Iteration #{event.iteration}: {event.story_id} &mdash; {event.story_title}
        </div>
      )
    case 'iteration_completed':
      return (
        <div className="text-xs text-green-600 dark:text-green-400 py-1">
          Iteration #{event.iteration} completed ({event.story_id}) &mdash; ${event.cost.toFixed(2)}, {Math.round(event.duration_secs)}s
        </div>
      )
    case 'iteration_failed':
      return (
        <div className="text-xs text-red-600 dark:text-red-400 py-1">
          Iteration #{event.iteration} failed ({event.story_id}): {event.error}
        </div>
      )
    case 'branch_created':
      return (
        <div className="text-xs text-blue-600 dark:text-blue-400 py-0.5">
          Branch created: {event.branch}
        </div>
      )
    case 'pr_created':
      return (
        <div className="text-xs text-purple-600 dark:text-purple-400 py-0.5">
          PR #{event.pr_number} created
        </div>
      )
    case 'pr_merged':
      return (
        <div className="text-xs text-green-600 dark:text-green-400 py-0.5">
          PR #{event.pr_number} merged
        </div>
      )
    case 'story_completed':
      return (
        <div className="text-xs text-green-600 dark:text-green-400 font-medium py-1">
          Story {event.story_id} completed ({event.passed}/{event.total} stories done)
        </div>
      )
    default:
      return null
  }
}

export function AgentFeed({ events, className }: AgentFeedProps) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [events.length])

  if (events.length === 0) {
    return (
      <div className={cn('flex items-center justify-center text-sm text-muted-foreground h-32', className)}>
        Waiting for agent activity...
      </div>
    )
  }

  return (
    <ScrollArea className={cn('h-[400px]', className)}>
      <div className="space-y-1 p-4 font-mono text-sm">
        {events.map((event, i) => (
          <FeedItem key={i} event={event} />
        ))}
        <div ref={bottomRef} />
      </div>
    </ScrollArea>
  )
}
