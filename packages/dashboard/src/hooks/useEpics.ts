// ABOUTME: React hooks for Epic data fetching and mutations
// ABOUTME: Provides useEpics, useEpicsByPRD, useCreateEpic, useUpdateEpic, useDeleteEpic hooks

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { epicsService, type Epic, type CreateEpicInput, type UpdateEpicInput } from '@/services/epics';
import { useToast } from '@/hooks/use-toast';

// Query keys
export const epicKeys = {
  all: ['epics'] as const,
  lists: () => [...epicKeys.all, 'list'] as const,
  list: (projectId: string) => [...epicKeys.lists(), projectId] as const,
  byPRD: (projectId: string, prdId: string) => [...epicKeys.all, 'by-prd', projectId, prdId] as const,
  details: () => [...epicKeys.all, 'detail'] as const,
  detail: (projectId: string, epicId: string) => [...epicKeys.details(), projectId, epicId] as const,
  progress: (projectId: string, epicId: string) => [...epicKeys.all, 'progress', projectId, epicId] as const,
};

/**
 * Hook to fetch all epics for a project
 */
export function useEpics(projectId: string) {
  return useQuery({
    queryKey: epicKeys.list(projectId),
    queryFn: () => epicsService.listEpics(projectId),
    enabled: !!projectId,
  });
}

/**
 * Hook to fetch epics for a specific PRD
 */
export function useEpicsByPRD(projectId: string, prdId: string) {
  return useQuery({
    queryKey: epicKeys.byPRD(projectId, prdId),
    queryFn: () => epicsService.getEpicsByPRD(projectId, prdId),
    enabled: !!projectId && !!prdId,
  });
}

/**
 * Hook to fetch a single epic
 */
export function useEpic(projectId: string, epicId: string) {
  return useQuery({
    queryKey: epicKeys.detail(projectId, epicId),
    queryFn: () => epicsService.getEpic(projectId, epicId),
    enabled: !!projectId && !!epicId,
  });
}

/**
 * Hook to create a new epic
 */
export function useCreateEpic(projectId: string) {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (input: CreateEpicInput) => epicsService.createEpic(projectId, input),
    onSuccess: (newEpic) => {
      // Invalidate and refetch epic lists
      queryClient.invalidateQueries({ queryKey: epicKeys.list(projectId) });
      queryClient.invalidateQueries({ queryKey: epicKeys.byPRD(projectId, newEpic.prdId) });

      toast({
        title: 'Epic created',
        description: `Successfully created epic: ${newEpic.name}`,
      });
    },
    onError: (error: Error) => {
      toast({
        title: 'Failed to create epic',
        description: error.message,
        variant: 'destructive',
      });
    },
  });
}

/**
 * Hook to update an epic
 */
export function useUpdateEpic(projectId: string, epicId: string) {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (updates: UpdateEpicInput) => epicsService.updateEpic(projectId, epicId, updates),
    onSuccess: (updatedEpic) => {
      // Invalidate and refetch
      queryClient.invalidateQueries({ queryKey: epicKeys.list(projectId) });
      queryClient.invalidateQueries({ queryKey: epicKeys.detail(projectId, epicId) });
      queryClient.invalidateQueries({ queryKey: epicKeys.byPRD(projectId, updatedEpic.prdId) });

      toast({
        title: 'Epic updated',
        description: `Successfully updated epic: ${updatedEpic.name}`,
      });
    },
    onError: (error: Error) => {
      toast({
        title: 'Failed to update epic',
        description: error.message,
        variant: 'destructive',
      });
    },
  });
}

/**
 * Hook to delete an epic
 */
export function useDeleteEpic(projectId: string) {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (epicId: string) => epicsService.deleteEpic(projectId, epicId),
    onSuccess: () => {
      // Invalidate all epic queries
      queryClient.invalidateQueries({ queryKey: epicKeys.all });

      toast({
        title: 'Epic deleted',
        description: 'Successfully deleted epic',
      });
    },
    onError: (error: Error) => {
      toast({
        title: 'Failed to delete epic',
        description: error.message,
        variant: 'destructive',
      });
    },
  });
}

/**
 * Hook to calculate epic progress
 */
export function useEpicProgress(projectId: string, epicId: string) {
  return useQuery({
    queryKey: epicKeys.progress(projectId, epicId),
    queryFn: () => epicsService.calculateProgress(projectId, epicId),
    enabled: !!projectId && !!epicId,
  });
}
