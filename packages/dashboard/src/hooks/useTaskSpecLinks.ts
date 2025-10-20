// ABOUTME: React Query hooks for task-spec linking operations
// ABOUTME: Provides hooks for linking tasks to requirements, validation, and orphan detection
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { taskSpecLinksService } from '@/services/task-spec-links';
import { queryKeys } from '@/lib/queryClient';
import type {
  TaskSpecLinkCreateInput,
  TaskValidationResult,
  SuggestSpecResponse,
  GenerateTasksInput,
  GenerateTasksResponse,
  OrphanTasksResponse,
} from '@/services/task-spec-links';
import type { SpecRequirement } from '@/services/specs';

interface ApiError {
  message?: string;
  status?: number;
}

export function useTaskSpecLinks(taskId: string) {
  return useQuery({
    queryKey: ['task-spec-links', taskId],
    queryFn: () => taskSpecLinksService.getTaskSpecLinks(taskId),
    enabled: !!taskId,
    staleTime: 5 * 60 * 1000,
  });
}

export function useLinkTaskToRequirement(taskId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (linkData: TaskSpecLinkCreateInput) =>
      taskSpecLinksService.linkTaskToRequirement(taskId, linkData),
    onMutate: async (newLink) => {
      await queryClient.cancelQueries({ queryKey: ['task-spec-links', taskId] });

      const previousLinks = queryClient.getQueryData<SpecRequirement[]>([
        'task-spec-links',
        taskId,
      ]);

      return { previousLinks };
    },
    onError: (_err, _newLink, context) => {
      if (context?.previousLinks) {
        queryClient.setQueryData(['task-spec-links', taskId], context.previousLinks);
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['task-spec-links', taskId] });
    },
  });
}

export function useValidateTask(taskId: string) {
  return useMutation({
    mutationFn: () => taskSpecLinksService.validateTaskAgainstSpec(taskId),
  });
}

export function useSuggestSpec(taskId: string) {
  return useMutation({
    mutationFn: () => taskSpecLinksService.suggestSpecFromTask(taskId),
  });
}

export function useGenerateTasksFromSpec(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (taskData: GenerateTasksInput) =>
      taskSpecLinksService.generateTasksFromSpec(projectId, taskData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.specsList(projectId) });
    },
  });
}

export function useOrphanTasks(projectId: string) {
  return useQuery({
    queryKey: ['orphan-tasks', projectId],
    queryFn: () => taskSpecLinksService.getOrphanTasks(projectId),
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}
