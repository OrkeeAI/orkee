// ABOUTME: React Query hooks for OpenSpec change operations (list, get, validate, archive)
// ABOUTME: Includes cache invalidation and optimistic updates for responsive UI
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { changesService } from '@/services/changes';
import { queryKeys } from '@/lib/queryClient';
import type { PaginationParams } from '@/types/pagination';
import type {
  ChangeListItem,
  ChangeWithDeltas,
  ChangeStatus,
} from '@/services/changes';

interface ApiError {
  message?: string;
  status?: number;
}

export function useChanges(
  projectId: string,
  status?: ChangeStatus,
  pagination?: PaginationParams
) {
  return useQuery({
    queryKey: [...queryKeys.changesList(projectId), { status, pagination }],
    queryFn: async () => {
      const response = await changesService.listChanges(projectId, status, pagination);
      return response.data;
    },
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}

export function useChangesPaginated(
  projectId: string,
  status?: ChangeStatus,
  pagination?: PaginationParams
) {
  return useQuery({
    queryKey: [...queryKeys.changesList(projectId), 'paginated', { status, pagination }],
    queryFn: () => changesService.listChanges(projectId, status, pagination),
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}

export function useChange(projectId: string, changeId: string) {
  return useQuery({
    queryKey: queryKeys.changeDetail(projectId, changeId),
    queryFn: () => changesService.getChange(projectId, changeId),
    enabled: !!projectId && !!changeId,
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

export function useValidateChange(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ changeId, strict }: { changeId: string; strict?: boolean }) =>
      changesService.validateChange(projectId, changeId, strict),
    onSuccess: (validationResult, { changeId }) => {
      const change = queryClient.getQueryData<ChangeWithDeltas>(
        queryKeys.changeDetail(projectId, changeId)
      );

      if (change) {
        const updatedChange: ChangeWithDeltas = {
          ...change,
          validationStatus: validationResult.isValid ? 'valid' : 'invalid',
          validationErrors: validationResult.errors,
        };
        queryClient.setQueryData(
          queryKeys.changeDetail(projectId, changeId),
          updatedChange
        );
      }

      queryClient.setQueryData<ChangeListItem[]>(
        queryKeys.changesList(projectId),
        (old = []) =>
          old.map(c =>
            c.id === changeId
              ? {
                  ...c,
                  validationStatus: validationResult.isValid ? 'valid' : 'invalid',
                }
              : c
          )
      );
    },
  });
}

export function useArchiveChange(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ changeId, applySpecs }: { changeId: string; applySpecs?: boolean }) =>
      changesService.archiveChange(projectId, changeId, applySpecs),
    onMutate: async ({ changeId }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.changeDetail(projectId, changeId) });
      await queryClient.cancelQueries({ queryKey: queryKeys.changesList(projectId) });

      const previousChange = queryClient.getQueryData<ChangeWithDeltas>(
        queryKeys.changeDetail(projectId, changeId)
      );
      const previousChanges = queryClient.getQueryData<ChangeListItem[]>(
        queryKeys.changesList(projectId)
      );

      if (previousChange) {
        const updatedChange: ChangeWithDeltas = {
          ...previousChange,
          status: 'archived',
          archivedAt: new Date().toISOString(),
        };
        queryClient.setQueryData(
          queryKeys.changeDetail(projectId, changeId),
          updatedChange
        );
      }

      if (previousChanges) {
        queryClient.setQueryData<ChangeListItem[]>(
          queryKeys.changesList(projectId),
          (old = []) =>
            old.map(c =>
              c.id === changeId ? { ...c, status: 'archived' as ChangeStatus } : c
            )
        );
      }

      return { previousChange, previousChanges };
    },
    onError: (_err, { changeId }, context) => {
      if (context?.previousChange) {
        queryClient.setQueryData(
          queryKeys.changeDetail(projectId, changeId),
          context.previousChange
        );
      }
      if (context?.previousChanges) {
        queryClient.setQueryData(
          queryKeys.changesList(projectId),
          context.previousChanges
        );
      }
    },
    onSuccess: (_archiveResult, { changeId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.changeDetail(projectId, changeId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.changesList(projectId) });
      queryClient.invalidateQueries({ queryKey: ['capabilities', projectId] });
    },
  });
}

export function useDeltas(projectId: string, changeId: string) {
  return useQuery({
    queryKey: [...queryKeys.changeDetail(projectId, changeId), 'deltas'],
    queryFn: () => changesService.getDeltas(projectId, changeId),
    enabled: !!projectId && !!changeId,
    staleTime: 5 * 60 * 1000,
  });
}

export function useUpdateChangeStatus(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      changeId,
      status,
      metadata,
    }: {
      changeId: string;
      status: ChangeStatus;
      metadata?: { approvedBy?: string; notes?: string };
    }) => changesService.updateChangeStatus(projectId, changeId, status, metadata),
    onMutate: async ({ changeId, status, metadata }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.changeDetail(projectId, changeId) });
      await queryClient.cancelQueries({ queryKey: queryKeys.changesList(projectId) });

      const previousChange = queryClient.getQueryData<ChangeWithDeltas>(
        queryKeys.changeDetail(projectId, changeId)
      );
      const previousChanges = queryClient.getQueryData<ChangeListItem[]>(
        queryKeys.changesList(projectId)
      );

      if (previousChange) {
        const updatedChange: ChangeWithDeltas = {
          ...previousChange,
          status,
          approvedBy: metadata?.approvedBy || previousChange.approvedBy,
          approvedAt: status === 'approved' ? new Date().toISOString() : previousChange.approvedAt,
        };
        queryClient.setQueryData(queryKeys.changeDetail(projectId, changeId), updatedChange);
      }

      if (previousChanges) {
        queryClient.setQueryData<ChangeListItem[]>(
          queryKeys.changesList(projectId),
          (old = []) =>
            old.map(c => (c.id === changeId ? { ...c, status } : c))
        );
      }

      return { previousChange, previousChanges };
    },
    onError: (_err, { changeId }, context) => {
      if (context?.previousChange) {
        queryClient.setQueryData(
          queryKeys.changeDetail(projectId, changeId),
          context.previousChange
        );
      }
      if (context?.previousChanges) {
        queryClient.setQueryData(queryKeys.changesList(projectId), context.previousChanges);
      }
    },
    onSuccess: (_result, { changeId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.changeDetail(projectId, changeId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.changesList(projectId) });
    },
  });
}
