// ABOUTME: React Query hooks for PRD operations (fetch, create, update, delete, analyze)
// ABOUTME: Includes optimistic updates and cache invalidation for responsive UI
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { prdsService } from '@/services/prds';
import { queryKeys, invalidatePRDQueries, invalidatePRD } from '@/lib/queryClient';
import type {
  PRD,
  PRDCreateInput,
  PRDUpdateInput,
  PRDAnalysisResult,
} from '@/services/prds';

interface ApiError {
  message?: string;
  status?: number;
}

export function usePRDs(projectId: string) {
  return useQuery({
    queryKey: queryKeys.prdsList(projectId),
    queryFn: () => prdsService.listPRDs(projectId),
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}

export function usePRD(projectId: string, prdId: string) {
  return useQuery({
    queryKey: queryKeys.prdDetail(projectId, prdId),
    queryFn: () => prdsService.getPRD(projectId, prdId),
    enabled: !!projectId && !!prdId,
    staleTime: 5 * 60 * 1000,
    retry: (failureCount, error) => {
      const apiError = error as ApiError;
      if (apiError?.message?.includes('not found')) {
        return false;
      }
      return failureCount < 2;
    },
  });
}

export function useCreatePRD(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: PRDCreateInput) => prdsService.createPRD(projectId, input),
    onMutate: async (newPRD) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.prdsList(projectId) });

      const previousPRDs = queryClient.getQueryData<PRD[]>(queryKeys.prdsList(projectId));

      queryClient.setQueryData<PRD[]>(queryKeys.prdsList(projectId), (old = []) => {
        const optimisticPRD: PRD = {
          id: `temp-${Date.now()}`,
          projectId,
          title: newPRD.title,
          contentMarkdown: newPRD.contentMarkdown,
          version: 1,
          status: newPRD.status || 'draft',
          source: newPRD.source || 'manual',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          createdBy: newPRD.createdBy,
        };
        return [optimisticPRD, ...old];
      });

      return { previousPRDs };
    },
    onError: (_err, _newPRD, context) => {
      if (context?.previousPRDs) {
        queryClient.setQueryData(queryKeys.prdsList(projectId), context.previousPRDs);
      }
    },
    onSuccess: (createdPRD) => {
      queryClient.setQueryData<PRD[]>(queryKeys.prdsList(projectId), (old = []) => {
        const filtered = old.filter(p => !p.id.startsWith('temp-'));
        return [createdPRD, ...filtered];
      });

      queryClient.setQueryData(
        queryKeys.prdDetail(projectId, createdPRD.id),
        createdPRD
      );
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.prdsList(projectId) });
    },
  });
}

export function useUpdatePRD(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ prdId, input }: { prdId: string; input: PRDUpdateInput }) =>
      prdsService.updatePRD(projectId, prdId, input),
    onMutate: async ({ prdId, input }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.prdDetail(projectId, prdId) });
      await queryClient.cancelQueries({ queryKey: queryKeys.prdsList(projectId) });

      const previousPRD = queryClient.getQueryData<PRD>(
        queryKeys.prdDetail(projectId, prdId)
      );
      const previousPRDs = queryClient.getQueryData<PRD[]>(queryKeys.prdsList(projectId));

      if (previousPRD) {
        const updatedPRD = {
          ...previousPRD,
          ...input,
          updatedAt: new Date().toISOString(),
        };
        queryClient.setQueryData(queryKeys.prdDetail(projectId, prdId), updatedPRD);
      }

      if (previousPRDs) {
        queryClient.setQueryData<PRD[]>(queryKeys.prdsList(projectId), (old = []) =>
          old.map(prd =>
            prd.id === prdId
              ? { ...prd, ...input, updatedAt: new Date().toISOString() }
              : prd
          )
        );
      }

      return { previousPRD, previousPRDs };
    },
    onError: (_err, { prdId }, context) => {
      if (context?.previousPRD) {
        queryClient.setQueryData(
          queryKeys.prdDetail(projectId, prdId),
          context.previousPRD
        );
      }
      if (context?.previousPRDs) {
        queryClient.setQueryData(queryKeys.prdsList(projectId), context.previousPRDs);
      }
    },
    onSuccess: (updatedPRD, { prdId }) => {
      queryClient.setQueryData(queryKeys.prdDetail(projectId, prdId), updatedPRD);

      queryClient.setQueryData<PRD[]>(queryKeys.prdsList(projectId), (old = []) =>
        old.map(prd => (prd.id === prdId ? updatedPRD : prd))
      );
    },
    onSettled: (_updatedPRD, _error, { prdId }) => {
      invalidatePRD(projectId, prdId);
    },
  });
}

export function useDeletePRD(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (prdId: string) => prdsService.deletePRD(projectId, prdId),
    onMutate: async (prdId) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.prdsList(projectId) });

      const previousPRDs = queryClient.getQueryData<PRD[]>(queryKeys.prdsList(projectId));

      queryClient.setQueryData<PRD[]>(queryKeys.prdsList(projectId), (old = []) =>
        old.filter(prd => prd.id !== prdId)
      );

      return { previousPRDs };
    },
    onError: (_err, _prdId, context) => {
      if (context?.previousPRDs) {
        queryClient.setQueryData(queryKeys.prdsList(projectId), context.previousPRDs);
      }
    },
    onSuccess: (_, prdId) => {
      queryClient.removeQueries({ queryKey: queryKeys.prdDetail(projectId, prdId) });
      queryClient.removeQueries({ queryKey: queryKeys.prdAnalysis(projectId, prdId) });
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.prdsList(projectId) });
    },
  });
}

export function useAnalyzePRD(projectId: string, prdId: string) {
  return useQuery({
    queryKey: queryKeys.prdAnalysis(projectId, prdId),
    queryFn: () => prdsService.analyzePRD(projectId, prdId),
    enabled: false,
    staleTime: 10 * 60 * 1000,
    gcTime: 30 * 60 * 1000,
  });
}

export function useTriggerPRDAnalysis(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (prdId: string) => prdsService.analyzePRD(projectId, prdId),
    onSuccess: (analysisResult, prdId) => {
      queryClient.setQueryData(
        queryKeys.prdAnalysis(projectId, prdId),
        analysisResult
      );
    },
  });
}

export function useSyncSpecsToPRD(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (prdId: string) => prdsService.syncSpecsToPRD(projectId, prdId),
    onSuccess: (updatedPRD, prdId) => {
      queryClient.setQueryData(queryKeys.prdDetail(projectId, prdId), updatedPRD);

      queryClient.setQueryData<PRD[]>(queryKeys.prdsList(projectId), (old = []) =>
        old ? old.map(prd => (prd.id === prdId ? updatedPRD : prd)) : [updatedPRD]
      );
    },
    onSettled: (_updatedPRD, _error, prdId) => {
      invalidatePRD(projectId, prdId);
    },
  });
}

export function useInvalidatePRDs(projectId: string) {
  return () => invalidatePRDQueries(projectId);
}

export function useInvalidatePRD(projectId: string, prdId: string) {
  return () => invalidatePRD(projectId, prdId);
}
