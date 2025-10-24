// ABOUTME: React hooks for managing OpenSpec change task completion
// ABOUTME: Provides state management, mutations, and real-time updates for task tracking

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { changesService, type ChangeTask, type TaskStats } from '@/services/changes';
import { toast } from 'sonner';

interface UseChangeTasksOptions {
  projectId: string;
  changeId: string;
  enabled?: boolean;
}

export function useChangeTasks({ projectId, changeId, enabled = true }: UseChangeTasksOptions) {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ['changeTasks', projectId, changeId],
    queryFn: () => changesService.getChangeTasks(projectId, changeId),
    enabled,
    staleTime: 30000, // 30 seconds
  });

  const stats: TaskStats = query.data
    ? changesService.calculateTaskStats(query.data)
    : { total: 0, completed: 0, pending: 0, percentage: 0 };

  return {
    tasks: query.data || [],
    stats,
    isLoading: query.isLoading,
    error: query.error,
    refetch: query.refetch,
  };
}

interface UseUpdateTaskOptions {
  projectId: string;
  changeId: string;
  onSuccess?: (task: ChangeTask) => void;
  onError?: (error: Error) => void;
}

export function useUpdateTask({ projectId, changeId, onSuccess, onError }: UseUpdateTaskOptions) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      taskId,
      isCompleted,
      completedBy,
    }: {
      taskId: string;
      isCompleted: boolean;
      completedBy?: string;
    }) => {
      return changesService.updateTask(projectId, changeId, taskId, isCompleted, completedBy);
    },
    onMutate: async ({ taskId, isCompleted }) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: ['changeTasks', projectId, changeId] });

      // Snapshot previous value
      const previousTasks = queryClient.getQueryData<ChangeTask[]>(['changeTasks', projectId, changeId]);

      // Optimistically update
      queryClient.setQueryData<ChangeTask[]>(['changeTasks', projectId, changeId], (old) =>
        old?.map((task) =>
          task.id === taskId
            ? { ...task, isCompleted, completedAt: isCompleted ? new Date().toISOString() : undefined }
            : task
        )
      );

      // Also invalidate the change to update completion percentage
      queryClient.invalidateQueries({ queryKey: ['change', projectId, changeId] });

      return { previousTasks };
    },
    onError: (error, _variables, context) => {
      // Rollback on error
      if (context?.previousTasks) {
        queryClient.setQueryData(['changeTasks', projectId, changeId], context.previousTasks);
      }
      toast.error(`Failed to update task: ${error.message}`);
      onError?.(error as Error);
    },
    onSuccess: (task) => {
      toast.success(task.isCompleted ? 'Task marked as complete' : 'Task marked as incomplete');
      onSuccess?.(task);
    },
    onSettled: () => {
      // Refetch to ensure data is in sync
      queryClient.invalidateQueries({ queryKey: ['changeTasks', projectId, changeId] });
      queryClient.invalidateQueries({ queryKey: ['change', projectId, changeId] });
    },
  });
}

interface UseBulkUpdateTasksOptions {
  projectId: string;
  changeId: string;
  onSuccess?: (tasks: ChangeTask[]) => void;
  onError?: (error: Error) => void;
}

export function useBulkUpdateTasks({
  projectId,
  changeId,
  onSuccess,
  onError,
}: UseBulkUpdateTasksOptions) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (tasks: Array<{ taskId: string; isCompleted: boolean; completedBy?: string }>) => {
      return changesService.bulkUpdateTasks(projectId, changeId, tasks);
    },
    onMutate: async (updates) => {
      await queryClient.cancelQueries({ queryKey: ['changeTasks', projectId, changeId] });

      const previousTasks = queryClient.getQueryData<ChangeTask[]>(['changeTasks', projectId, changeId]);

      // Optimistically update all tasks
      queryClient.setQueryData<ChangeTask[]>(['changeTasks', projectId, changeId], (old) =>
        old?.map((task) => {
          const update = updates.find((u) => u.taskId === task.id);
          if (update) {
            return {
              ...task,
              isCompleted: update.isCompleted,
              completedAt: update.isCompleted ? new Date().toISOString() : undefined,
            };
          }
          return task;
        })
      );

      queryClient.invalidateQueries({ queryKey: ['change', projectId, changeId] });

      return { previousTasks };
    },
    onError: (error, _variables, context) => {
      if (context?.previousTasks) {
        queryClient.setQueryData(['changeTasks', projectId, changeId], context.previousTasks);
      }
      toast.error(`Failed to update tasks: ${error.message}`);
      onError?.(error as Error);
    },
    onSuccess: (tasks) => {
      toast.success(`Updated ${tasks.length} tasks`);
      onSuccess?.(tasks);
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['changeTasks', projectId, changeId] });
      queryClient.invalidateQueries({ queryKey: ['change', projectId, changeId] });
    },
  });
}

interface UseParseChangeTasksOptions {
  projectId: string;
  changeId: string;
  onSuccess?: (tasks: ChangeTask[]) => void;
  onError?: (error: Error) => void;
}

export function useParseChangeTasks({
  projectId,
  changeId,
  onSuccess,
  onError,
}: UseParseChangeTasksOptions) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => changesService.parseChangeTasks(projectId, changeId),
    onSuccess: (tasks) => {
      queryClient.setQueryData(['changeTasks', projectId, changeId], tasks);
      queryClient.invalidateQueries({ queryKey: ['change', projectId, changeId] });
      toast.success(`Parsed ${tasks.length} tasks from markdown`);
      onSuccess?.(tasks);
    },
    onError: (error) => {
      toast.error(`Failed to parse tasks: ${error.message}`);
      onError?.(error as Error);
    },
  });
}
