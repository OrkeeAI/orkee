// ABOUTME: Task completion tracker for OpenSpec changes
// ABOUTME: Displays task list with progress tracking and interactive completion checkboxes

import { useState } from 'react';
import { CheckCircle2, Circle, RefreshCw, ListTodo, AlertCircle } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { TaskItem } from './TaskItem';
import { useChangeTasks, useUpdateTask, useParseChangeTasks } from '@/hooks/useChangeTasks';
import type { ChangeStatus } from '@/services/changes';
import { cn } from '@/lib/utils';

interface TaskCompletionTrackerProps {
  projectId: string;
  changeId: string;
  status: ChangeStatus;
  currentUser?: string;
}

export function TaskCompletionTracker({
  projectId,
  changeId,
  status,
  currentUser,
}: TaskCompletionTrackerProps) {
  const { tasks, stats, isLoading, error } = useChangeTasks({
    projectId,
    changeId,
  });

  const updateTaskMutation = useUpdateTask({ projectId, changeId });
  const parseTasksMutation = useParseChangeTasks({ projectId, changeId });

  // Determine if tasks can be edited based on change status
  const canEdit = status === 'implementing';

  const handleToggleTask = (taskId: string, isCompleted: boolean) => {
    if (!canEdit) return;
    updateTaskMutation.mutate({ taskId, isCompleted, completedBy: currentUser });
  };

  const handleRefreshTasks = () => {
    parseTasksMutation.mutate();
  };

  // Group tasks by parent number for hierarchical display
  const groupedTasks = tasks.reduce((groups, task) => {
    const key = task.parentNumber || 'root';
    if (!groups[key]) {
      groups[key] = [];
    }
    groups[key].push(task);
    return groups;
  }, {} as Record<string, typeof tasks>);

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ListTodo className="h-5 w-5" />
            Implementation Tasks
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <div className="text-center">
              <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
              <p className="text-sm text-muted-foreground">Loading tasks...</p>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ListTodo className="h-5 w-5" />
            Implementation Tasks
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              Failed to load tasks: {error instanceof Error ? error.message : 'Unknown error'}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  const getProgressColor = (percentage: number) => {
    if (percentage < 30) return 'bg-red-500';
    if (percentage < 70) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <ListTodo className="h-5 w-5" />
              Implementation Tasks
            </CardTitle>
            <CardDescription className="mt-2">
              Track progress through implementation steps
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleRefreshTasks}
              disabled={parseTasksMutation.isPending}
            >
              {parseTasksMutation.isPending ? (
                <>
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  Parsing...
                </>
              ) : (
                <>
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Re-parse
                </>
              )}
            </Button>
          </div>
        </div>

        {/* Progress Bar */}
        <div className="space-y-2 mt-4">
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">
              {stats.completed} of {stats.total} tasks complete
            </span>
            <span className="font-semibold">{stats.percentage}%</span>
          </div>
          <Progress value={stats.percentage} className="h-2">
            <div
              className={cn('h-full rounded-full transition-all', getProgressColor(stats.percentage))}
              style={{ width: `${stats.percentage}%` }}
            />
          </Progress>
        </div>

        {/* Status Info */}
        {!canEdit && (
          <Alert className="mt-4">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              {status === 'draft' || status === 'review' || status === 'approved'
                ? 'Tasks can only be edited when the change status is "Implementing"'
                : 'Tasks are read-only in this status'}
            </AlertDescription>
          </Alert>
        )}

        {stats.percentage === 100 && status === 'implementing' && (
          <Alert className="mt-4">
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertDescription className="text-green-900 dark:text-green-100">
              All tasks completed! You can now mark the change as complete.
            </AlertDescription>
          </Alert>
        )}
      </CardHeader>

      <CardContent>
        {tasks.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <Circle className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No tasks found</p>
            <Button variant="outline" size="sm" className="mt-4" onClick={handleRefreshTasks}>
              Parse Tasks from Markdown
            </Button>
          </div>
        ) : (
          <div className="space-y-1">
            {/* Root level tasks (no parent) */}
            {(groupedTasks.root || []).map((task) => (
              <TaskItem
                key={task.id}
                task={task}
                disabled={!canEdit || updateTaskMutation.isPending}
                onToggle={handleToggleTask}
              />
            ))}

            {/* Nested tasks grouped by parent */}
            {Object.entries(groupedTasks)
              .filter(([key]) => key !== 'root')
              .map(([parentNumber, childTasks]) => (
                <div key={parentNumber} className="ml-6 border-l-2 border-accent pl-2">
                  {childTasks.map((task) => (
                    <TaskItem
                      key={task.id}
                      task={task}
                      disabled={!canEdit || updateTaskMutation.isPending}
                      onToggle={handleToggleTask}
                    />
                  ))}
                </div>
              ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
