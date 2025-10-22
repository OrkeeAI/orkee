// ABOUTME: React Query hooks for context generation and OpenSpec integration
// ABOUTME: Provides hooks for file listing, context generation, PRD/task context, and spec validation

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { contextService } from '@/services/context';
import type {
  ContextGenerationRequest,
  SpecValidationReport,
  ContextConfiguration,
} from '@/services/context';

interface ApiError {
  message?: string;
  status?: number;
}

// Query keys for context operations
export const contextQueryKeys = {
  all: ['context'] as const,
  files: (projectId: string) => ['context', 'files', projectId] as const,
  configurations: (projectId: string) =>
    ['context', 'configurations', projectId] as const,
  history: (projectId: string) => ['context', 'history', projectId] as const,
  stats: (projectId: string) => ['context', 'stats', projectId] as const,
};

// List project files
export function useProjectFiles(projectId: string, maxDepth?: number) {
  return useQuery({
    queryKey: [...contextQueryKeys.files(projectId), maxDepth],
    queryFn: async () => {
      const response = await contextService.listProjectFiles(projectId, maxDepth);
      return response.data;
    },
    enabled: !!projectId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

// List saved configurations
export function useContextConfigurations(projectId: string) {
  return useQuery({
    queryKey: contextQueryKeys.configurations(projectId),
    queryFn: async () => {
      const response = await contextService.listConfigurations(projectId);
      return response.data;
    },
    enabled: !!projectId,
    staleTime: 5 * 60 * 1000,
  });
}

// Get context history
export function useContextHistory(projectId: string) {
  return useQuery({
    queryKey: contextQueryKeys.history(projectId),
    queryFn: async () => {
      const response = await contextService.getContextHistory(projectId);
      return response.data.snapshots;
    },
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
}

// Get context statistics
export function useContextStats(projectId: string) {
  return useQuery({
    queryKey: contextQueryKeys.stats(projectId),
    queryFn: async () => {
      const response = await contextService.getContextStats(projectId);
      return response.data;
    },
    enabled: !!projectId,
    staleTime: 5 * 60 * 1000,
  });
}

// Generate context from files
export function useGenerateContext(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: ContextGenerationRequest) => {
      const response = await contextService.generateContext(projectId, request);
      if (response.error) {
        throw new Error(response.error);
      }
      return response.data;
    },
    onSuccess: () => {
      // Invalidate history and stats after generating new context
      queryClient.invalidateQueries({ queryKey: contextQueryKeys.history(projectId) });
      queryClient.invalidateQueries({ queryKey: contextQueryKeys.stats(projectId) });
    },
  });
}

// Save context configuration
export function useSaveContextConfiguration(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (config: Partial<ContextConfiguration>) => {
      const response = await contextService.saveConfiguration(projectId, config);
      if (response.error) {
        throw new Error(response.error);
      }
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: contextQueryKeys.configurations(projectId),
      });
    },
  });
}

// Generate context from PRD
export function useGenerateContextFromPRD(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (prdId: string) => {
      const response = await contextService.generateContextFromPRD(projectId, prdId);
      if (response.error) {
        throw new Error(response.error);
      }
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: contextQueryKeys.history(projectId) });
    },
  });
}

// Generate context from task
export function useGenerateContextFromTask(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (taskId: string) => {
      const response = await contextService.generateContextFromTask(projectId, taskId);
      if (response.error) {
        throw new Error(response.error);
      }
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: contextQueryKeys.history(projectId) });
    },
  });
}

// Validate spec implementation
export function useValidateSpecImplementation(projectId: string) {
  return useMutation({
    mutationFn: async (capabilityId: string) => {
      const response = await contextService.validateSpecImplementation(
        projectId,
        capabilityId
      );
      if (response.error) {
        throw new Error(response.error);
      }
      return response.data as SpecValidationReport;
    },
  });
}

// Restore context snapshot
export function useRestoreContextSnapshot(projectId: string) {
  return useMutation({
    mutationFn: async (snapshotId: string) => {
      const response = await contextService.restoreSnapshot(projectId, snapshotId);
      if (response.error) {
        throw new Error(response.error);
      }
      return response.data;
    },
  });
}
