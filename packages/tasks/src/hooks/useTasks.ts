import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Task, TaskProvider, TaskProviderConfig, TaskProviderType } from '../types';
import { TaskProviderFactory } from '../providers/factory';

interface UseTasksOptions {
  projectId: string;
  projectPath: string;
  providerType: TaskProviderType;
  enabled?: boolean;
  apiBaseUrl?: string;
  apiToken?: string;
}

export function useTasks({
  projectId,
  projectPath,
  providerType,
  enabled = true,
  apiBaseUrl,
  apiToken,
}: UseTasksOptions) {
  const queryClient = useQueryClient();
  const [provider, setProvider] = useState<TaskProvider | null>(null);

  useEffect(() => {
    if (!enabled) return;
    
    const config: TaskProviderConfig = {
      type: providerType,
      projectPath,
      options: {
        ...(apiBaseUrl && { apiBaseUrl }),
        ...(apiToken && { apiToken }),
      },
    };
    
    const taskProvider = TaskProviderFactory.create(config);
    taskProvider.initialize().then(() => {
      setProvider(taskProvider);
    });
    
    return () => {
      // Cleanup if needed
    };
  }, [providerType, projectPath, enabled, apiBaseUrl, apiToken]);

  const { data: tasks = [], isLoading, error } = useQuery({
    queryKey: ['tasks', projectId, providerType],
    queryFn: async () => {
      if (!provider) throw new Error('Provider not initialized');
      return provider.getTasks(projectPath);
    },
    enabled: enabled && !!provider,
    refetchInterval: 5000, // Poll every 5 seconds
  });

  const createTaskMutation = useMutation({
    mutationFn: async (taskData: Partial<Task>) => {
      if (!provider) throw new Error('Provider not initialized');
      return provider.createTask(projectPath, taskData);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks', projectId, providerType] });
    },
  });

  const updateTaskMutation = useMutation({
    mutationFn: async ({ taskId, updates }: { taskId: string; updates: Partial<Task> }) => {
      if (!provider) throw new Error('Provider not initialized');
      return provider.updateTask(projectPath, taskId, updates);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks', projectId, providerType] });
    },
  });

  const deleteTaskMutation = useMutation({
    mutationFn: async (taskId: string) => {
      if (!provider) throw new Error('Provider not initialized');
      return provider.deleteTask(projectPath, taskId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks', projectId, providerType] });
    },
  });

  const createTask = useCallback((task: Partial<Task>) => {
    return createTaskMutation.mutate(task);
  }, [createTaskMutation]);

  const updateTask = useCallback((taskId: string, updates: Partial<Task>) => {
    return updateTaskMutation.mutate({ taskId, updates });
  }, [updateTaskMutation]);

  const deleteTask = useCallback((taskId: string) => {
    return deleteTaskMutation.mutate(taskId);
  }, [deleteTaskMutation]);

  return {
    tasks,
    isLoading: isLoading || !provider,
    error,
    createTask,
    updateTask,
    deleteTask,
  };
}