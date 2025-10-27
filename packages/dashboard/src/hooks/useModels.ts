// ABOUTME: React Query hooks for model operations
// ABOUTME: Provides hooks for fetching available AI models with pricing
import { useQuery } from '@tanstack/react-query';
import { modelsService } from '@/services/models';
import { queryKeys } from '@/lib/queryClient';

/**
 * Get all available AI models
 */
export function useModels() {
  return useQuery({
    queryKey: queryKeys.models,
    queryFn: async () => {
      const response = await modelsService.listModels();
      return response.data;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes - models don't change often
  });
}

/**
 * Get a specific model by ID
 */
export function useModel(modelId: string) {
  return useQuery({
    queryKey: [...queryKeys.models, modelId],
    queryFn: () => modelsService.getModel(modelId),
    enabled: !!modelId,
    staleTime: 5 * 60 * 1000,
  });
}

/**
 * Get models for a specific provider
 */
export function useModelsByProvider(provider: string) {
  return useQuery({
    queryKey: [...queryKeys.models, 'provider', provider],
    queryFn: async () => {
      const response = await modelsService.listModelsByProvider(provider);
      return response.data;
    },
    enabled: !!provider,
    staleTime: 5 * 60 * 1000,
  });
}
