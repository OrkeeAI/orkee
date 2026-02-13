// ABOUTME: Visual story progress board showing completion status of each user story
// ABOUTME: Displays story chips with color-coded states (completed, in-progress, pending)

import { Badge } from '@/components/ui/badge'
import type { UserStory } from '@/services/agent-runs'
import { cn } from '@/lib/utils'

interface StoryBoardProps {
  stories: UserStory[]
  activeStoryId?: string
}

function storyStatus(story: UserStory, activeStoryId?: string): 'completed' | 'active' | 'pending' {
  if (story.passes) return 'completed'
  if (story.id === activeStoryId) return 'active'
  return 'pending'
}

const statusStyles = {
  completed: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
  active: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300 animate-pulse',
  pending: 'bg-muted text-muted-foreground',
}

const statusIcons = {
  completed: '\u2705',
  active: '\u23F3',
  pending: '\u25CB',
}

export function StoryBoard({ stories, activeStoryId }: StoryBoardProps) {
  const sortedStories = [...stories].sort((a, b) => a.priority - b.priority)
  const completed = stories.filter(s => s.passes).length

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Story Progress</h3>
        <span className="text-sm text-muted-foreground">
          {completed}/{stories.length} completed
        </span>
      </div>
      <div className="flex flex-wrap gap-2">
        {sortedStories.map(story => {
          const status = storyStatus(story, activeStoryId)
          return (
            <Badge
              key={story.id}
              variant="outline"
              className={cn('cursor-default text-xs', statusStyles[status])}
              title={`${story.id}: ${story.title}`}
            >
              {statusIcons[status]} {story.id}
            </Badge>
          )
        })}
      </div>
    </div>
  )
}
