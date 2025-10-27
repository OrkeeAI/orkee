// ABOUTME: React Query hooks for brainstorm session operations (fetch, create, update, delete, skip)
// ABOUTME: Includes cache invalidation and optimistic updates for responsive UI

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { brainstormService } from '@/services/brainstorm';
import { queryKeys } from '@/lib/queryClient';
import type {
  CreateBrainstormInput,
  UpdateBrainstormInput,
  SkipSectionInput,
} from '@/services/brainstorm';

interface ApiError {
  message?: string;
  status?: number;
}

/**
 * Fetch all brainstorm sessions for a project
 */
export function useBrainstormSessions(projectId: string) {
  return useQuery({
    queryKey: queryKeys.brainstormList(projectId),
    queryFn: () => brainstormService.listSessions(projectId),
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}

/**
 * Fetch a single brainstorm session by ID
 */
export function useBrainstormSession(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.brainstormDetail(sessionId),
    queryFn: () => brainstormService.getSession(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
    retry: (failureCount, error) => {
      const apiError = error as ApiError;
      if (apiError?.status === 404) return false;
      return failureCount < 2;
    },
  });
}

/**
 * Fetch session completion status
 */
export function useBrainstormStatus(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.brainstormStatus(sessionId),
    queryFn: () => brainstormService.getCompletionStatus(sessionId),
    enabled: !!sessionId,
    staleTime: 30 * 1000, // 30 seconds - status changes frequently
  });
}

/**
 * Create a new brainstorm session
 */
export function useCreateBrainstormSession(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateBrainstormInput) => brainstormService.createSession(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.brainstormList(projectId) });
    },
  });
}

/**
 * Update a brainstorm session
 */
export function useUpdateBrainstormSession(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: UpdateBrainstormInput) =>
      brainstormService.updateSession(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.brainstormDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.brainstormList(projectId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.brainstormStatus(sessionId) });
    },
  });
}

/**
 * Delete a brainstorm session
 */
export function useDeleteBrainstormSession(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (sessionId: string) => brainstormService.deleteSession(sessionId),
    onSuccess: (_, sessionId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.brainstormList(projectId) });
      queryClient.removeQueries({ queryKey: queryKeys.brainstormDetail(sessionId) });
      queryClient.removeQueries({ queryKey: queryKeys.brainstormStatus(sessionId) });
    },
  });
}

/**
 * Skip a section with optional AI fill
 */
export function useSkipSection(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: SkipSectionInput) => brainstormService.skipSection(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.brainstormDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.brainstormStatus(sessionId) });
    },
  });
}
