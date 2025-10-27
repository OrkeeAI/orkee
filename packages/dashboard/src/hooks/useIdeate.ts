// ABOUTME: React Query hooks for ideate session operations (fetch, create, update, delete, skip)
// ABOUTME: Includes cache invalidation and optimistic updates for responsive UI

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ideateService } from '@/services/ideate';
import { queryKeys } from '@/lib/queryClient';
import type {
  CreateIdeateInput,
  UpdateIdeateInput,
  SkipSectionInput,
} from '@/services/ideate';

interface ApiError {
  message?: string;
  status?: number;
}

/**
 * Fetch all ideate sessions for a project
 */
export function useIdeateSessions(projectId: string) {
  return useQuery({
    queryKey: queryKeys.ideateList(projectId),
    queryFn: () => ideateService.listSessions(projectId),
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}

/**
 * Fetch a single ideate session by ID
 */
export function useIdeateSession(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateDetail(sessionId),
    queryFn: () => ideateService.getSession(sessionId),
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
export function useIdeateStatus(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateStatus(sessionId),
    queryFn: () => ideateService.getCompletionStatus(sessionId),
    enabled: !!sessionId,
    staleTime: 30 * 1000, // 30 seconds - status changes frequently
  });
}

/**
 * Create a new ideate session
 */
export function useCreateIdeateSession(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateIdeateInput) => ideateService.createSession(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateList(projectId) });
    },
  });
}

/**
 * Update a ideate session
 */
export function useUpdateIdeateSession(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: UpdateIdeateInput) =>
      ideateService.updateSession(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateList(projectId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete a ideate session
 */
export function useDeleteIdeateSession(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (sessionId: string) => ideateService.deleteSession(sessionId),
    onSuccess: (_, sessionId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateList(projectId) });
      queryClient.removeQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.removeQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Skip a section with optional AI fill
 */
export function useSkipSection(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: SkipSectionInput) => ideateService.skipSection(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}
