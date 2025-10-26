// ABOUTME: Individual task item component for OpenSpec change tasks
// ABOUTME: Displays task with checkbox, completion state, and metadata

import { CheckCircle2, Circle } from 'lucide-react';
import { Checkbox } from '@/components/ui/checkbox';
import { cn } from '@/lib/utils';
import type { ChangeTask } from '@/services/changes';
import { format } from 'date-fns';

interface TaskItemProps {
  task: ChangeTask;
  disabled?: boolean;
  onToggle?: (taskId: string, isCompleted: boolean) => void;
  showMetadata?: boolean;
}

export function TaskItem({ task, disabled = false, onToggle, showMetadata = true }: TaskItemProps) {
  const handleToggle = () => {
    if (!disabled && onToggle) {
      onToggle(task.id, !task.isCompleted);
    }
  };

  // Calculate indentation level from parent number
  const indentLevel = task.parentNumber ? task.parentNumber.split('.').length : 0;

  return (
    <div
      className={cn(
        'flex items-start gap-3 py-2 px-3 rounded-md transition-colors',
        'hover:bg-accent/50',
        task.isCompleted && 'opacity-60'
      )}
      style={{ paddingLeft: `${12 + indentLevel * 24}px` }}
    >
      <div className="flex items-center pt-0.5">
        <Checkbox
          checked={task.isCompleted}
          onCheckedChange={handleToggle}
          disabled={disabled}
          className="h-4 w-4"
        />
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-start gap-2">
          <span
            className={cn(
              'font-mono text-xs text-muted-foreground flex-shrink-0',
              task.isCompleted && 'line-through'
            )}
          >
            {task.taskNumber}
          </span>
          <p
            className={cn(
              'text-sm leading-relaxed',
              task.isCompleted && 'line-through text-muted-foreground'
            )}
          >
            {task.taskText}
          </p>
        </div>

        {showMetadata && task.isCompleted && task.completedAt && (
          <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
            <CheckCircle2 className="h-3 w-3 text-green-600" />
            <span>
              Completed {format(new Date(task.completedAt), 'MMM d, yyyy')}
              {task.completedBy && ` by ${task.completedBy}`}
            </span>
          </div>
        )}

        {showMetadata && !task.isCompleted && (
          <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
            <Circle className="h-3 w-3" />
            <span>Pending</span>
          </div>
        )}
      </div>
    </div>
  );
}
