// ABOUTME: React Query hooks for spec capability operations (fetch, create, update, delete, validate)
// ABOUTME: Includes optimistic updates and cache invalidation for responsive UI
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { specsService } from '@/services/specs';
import { queryKeys, invalidateSpecQueries, invalidateSpec } from '@/lib/queryClient';
import type {
  SpecCapability,
  SpecCapabilityCreateInput,
  SpecCapabilityUpdateInput,
} from '@/services/specs';

interface ApiError {
  message?: string;
  status?: number;
}

export function useSpecs(projectId: string) {
  return useQuery({
    queryKey: queryKeys.specsList(projectId),
    queryFn: () => specsService.listSpecs(projectId),
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}

export function useSpec(projectId: string, specId: string) {
  return useQuery({
    queryKey: queryKeys.specDetail(projectId, specId),
    queryFn: () => specsService.getSpec(projectId, specId),
    enabled: !!projectId && !!specId,
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

export function useSpecRequirements(projectId: string, specId: string) {
  return useQuery({
    queryKey: queryKeys.specRequirements(projectId, specId),
    queryFn: () => specsService.getSpecRequirements(projectId, specId),
    enabled: !!projectId && !!specId,
    staleTime: 5 * 60 * 1000,
  });
}

export function useCreateSpec(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: SpecCapabilityCreateInput) =>
      specsService.createSpec(projectId, input),
    onMutate: async (newSpec) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.specsList(projectId) });

      const previousSpecs = queryClient.getQueryData<SpecCapability[]>(
        queryKeys.specsList(projectId)
      );

      queryClient.setQueryData<SpecCapability[]>(
        queryKeys.specsList(projectId),
        (old = []) => {
          const optimisticSpec: SpecCapability = {
            id: `temp-${Date.now()}`,
            projectId,
            prdId: newSpec.prdId,
            name: newSpec.name,
            purpose: newSpec.purpose,
            specMarkdown: '',
            designMarkdown: newSpec.designMarkdown,
            requirements: newSpec.requirements,
            requirementCount: newSpec.requirements.length,
            version: 1,
            status: newSpec.status || 'active',
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
          };
          return [optimisticSpec, ...old];
        }
      );

      return { previousSpecs };
    },
    onError: (_err, _newSpec, context) => {
      if (context?.previousSpecs) {
        queryClient.setQueryData(queryKeys.specsList(projectId), context.previousSpecs);
      }
    },
    onSuccess: (createdSpec) => {
      queryClient.setQueryData<SpecCapability[]>(
        queryKeys.specsList(projectId),
        (old = []) => {
          const filtered = old.filter((s) => !s.id.startsWith('temp-'));
          return [createdSpec, ...filtered];
        }
      );

      queryClient.setQueryData(
        queryKeys.specDetail(projectId, createdSpec.id),
        createdSpec
      );
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.specsList(projectId) });
    },
  });
}

export function useUpdateSpec(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ specId, input }: { specId: string; input: SpecCapabilityUpdateInput }) =>
      specsService.updateSpec(projectId, specId, input),
    onMutate: async ({ specId, input }) => {
      await queryClient.cancelQueries({
        queryKey: queryKeys.specDetail(projectId, specId),
      });
      await queryClient.cancelQueries({ queryKey: queryKeys.specsList(projectId) });

      const previousSpec = queryClient.getQueryData<SpecCapability>(
        queryKeys.specDetail(projectId, specId)
      );
      const previousSpecs = queryClient.getQueryData<SpecCapability[]>(
        queryKeys.specsList(projectId)
      );

      if (previousSpec) {
        const updatedSpec = {
          ...previousSpec,
          ...input,
          updatedAt: new Date().toISOString(),
        };
        queryClient.setQueryData(
          queryKeys.specDetail(projectId, specId),
          updatedSpec
        );
      }

      if (previousSpecs) {
        queryClient.setQueryData<SpecCapability[]>(
          queryKeys.specsList(projectId),
          (old = []) =>
            old.map((spec) =>
              spec.id === specId
                ? { ...spec, ...input, updatedAt: new Date().toISOString() }
                : spec
            )
        );
      }

      return { previousSpec, previousSpecs };
    },
    onError: (_err, { specId }, context) => {
      if (context?.previousSpec) {
        queryClient.setQueryData(
          queryKeys.specDetail(projectId, specId),
          context.previousSpec
        );
      }
      if (context?.previousSpecs) {
        queryClient.setQueryData(queryKeys.specsList(projectId), context.previousSpecs);
      }
    },
    onSuccess: (updatedSpec, { specId }) => {
      queryClient.setQueryData(
        queryKeys.specDetail(projectId, specId),
        updatedSpec
      );

      queryClient.setQueryData<SpecCapability[]>(
        queryKeys.specsList(projectId),
        (old = []) => old.map((spec) => (spec.id === specId ? updatedSpec : spec))
      );
    },
    onSettled: (_updatedSpec, _error, { specId }) => {
      invalidateSpec(projectId, specId);
    },
  });
}

export function useDeleteSpec(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (specId: string) => specsService.deleteSpec(projectId, specId),
    onMutate: async (specId) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.specsList(projectId) });

      const previousSpecs = queryClient.getQueryData<SpecCapability[]>(
        queryKeys.specsList(projectId)
      );

      queryClient.setQueryData<SpecCapability[]>(
        queryKeys.specsList(projectId),
        (old = []) => old.filter((spec) => spec.id !== specId)
      );

      return { previousSpecs };
    },
    onError: (_err, _specId, context) => {
      if (context?.previousSpecs) {
        queryClient.setQueryData(queryKeys.specsList(projectId), context.previousSpecs);
      }
    },
    onSuccess: (_, specId) => {
      queryClient.removeQueries({
        queryKey: queryKeys.specDetail(projectId, specId),
      });
      queryClient.removeQueries({
        queryKey: queryKeys.specRequirements(projectId, specId),
      });
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.specsList(projectId) });
    },
  });
}

export function useValidateSpec(projectId: string) {
  return useMutation({
    mutationFn: (specData: SpecCapabilityCreateInput) =>
      specsService.validateSpec(projectId, specData),
  });
}

export function useInvalidateSpecs(projectId: string) {
  return () => invalidateSpecQueries(projectId);
}

export function useInvalidateSpec(projectId: string, specId: string) {
  return () => invalidateSpec(projectId, specId);
}
