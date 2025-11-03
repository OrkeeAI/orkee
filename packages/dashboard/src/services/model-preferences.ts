// ABOUTME: Model preferences service with React Query hooks for fetching and updating user model preferences
// ABOUTME: Provides hooks for accessing model registry and managing per-task model configuration

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './api';
import type { ModelPreferences, ModelInfo, TaskType, ModelConfig, Provider } from '@/types/models';
import { DEFAULT_MODEL_CONFIG, TASK_TYPE_TO_FIELD } from '@/types/models';

/**
 * API response type for model preferences
 */
interface ModelPreferencesResponse {
  user_id: string;
  chat_model: string;
  chat_provider: string;
  prd_generation_model: string;
  prd_generation_provider: string;
  prd_analysis_model: string;
  prd_analysis_provider: string;
  insight_extraction_model: string;
  insight_extraction_provider: string;
  spec_generation_model: string;
  spec_generation_provider: string;
  task_suggestions_model: string;
  task_suggestions_provider: string;
  task_analysis_model: string;
  task_analysis_provider: string;
  spec_refinement_model: string;
  spec_refinement_provider: string;
  research_generation_model: string;
  research_generation_provider: string;
  markdown_generation_model: string;
  markdown_generation_provider: string;
  updated_at: string;
}

/**
 * Convert snake_case API response to camelCase TypeScript interface
 */
function convertToModelPreferences(response: ModelPreferencesResponse): ModelPreferences {
  return {
    userId: response.user_id,
    chat: {
      provider: response.chat_provider as Provider,
      model: response.chat_model,
    },
    prdGeneration: {
      provider: response.prd_generation_provider as Provider,
      model: response.prd_generation_model,
    },
    prdAnalysis: {
      provider: response.prd_analysis_provider as Provider,
      model: response.prd_analysis_model,
    },
    insightExtraction: {
      provider: response.insight_extraction_provider as Provider,
      model: response.insight_extraction_model,
    },
    specGeneration: {
      provider: response.spec_generation_provider as Provider,
      model: response.spec_generation_model,
    },
    taskSuggestions: {
      provider: response.task_suggestions_provider as Provider,
      model: response.task_suggestions_model,
    },
    taskAnalysis: {
      provider: response.task_analysis_provider as Provider,
      model: response.task_analysis_model,
    },
    specRefinement: {
      provider: response.spec_refinement_provider as Provider,
      model: response.spec_refinement_model,
    },
    researchGeneration: {
      provider: response.research_generation_provider as Provider,
      model: response.research_generation_model,
    },
    markdownGeneration: {
      provider: response.markdown_generation_provider as Provider,
      model: response.markdown_generation_model,
    },
    updatedAt: response.updated_at,
  };
}

/**
 * React Query hook to fetch user model preferences
 */
export function useModelPreferences(userId: string) {
  return useQuery({
    queryKey: ['model-preferences', userId],
    queryFn: async () => {
      const response = await apiClient.get<ModelPreferencesResponse>(
        `/api/users/${userId}/model-preferences`
      );

      if (response.error) {
        throw new Error(response.error);
      }

      return convertToModelPreferences(response.data);
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
    retry: 1,
  });
}

/**
 * React Query mutation hook to update all model preferences
 */
export function useUpdateModelPreferences(userId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (preferences: Partial<Omit<ModelPreferences, 'userId' | 'updatedAt'>>) => {
      // Convert camelCase to snake_case for API
      const apiPayload: Record<string, string> = {};

      for (const [key, value] of Object.entries(preferences)) {
        if (value && typeof value === 'object' && 'provider' in value && 'model' in value) {
          const snakeKey = key.replace(/([A-Z])/g, '_$1').toLowerCase();
          apiPayload[`${snakeKey}_provider`] = value.provider;
          apiPayload[`${snakeKey}_model`] = value.model;
        }
      }

      const response = await apiClient.put<ModelPreferencesResponse>(
        `/api/users/${userId}/model-preferences`,
        apiPayload
      );

      if (response.error) {
        throw new Error(response.error);
      }

      return convertToModelPreferences(response.data);
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['model-preferences', userId], data);
    },
  });
}

/**
 * React Query mutation hook to update a single task's model preference
 */
export function useUpdateTaskModelPreference(userId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ taskType, config }: { taskType: TaskType; config: ModelConfig }) => {
      const response = await apiClient.put<ModelPreferencesResponse>(
        `/api/users/${userId}/model-preferences/${taskType}`,
        {
          provider: config.provider,
          model: config.model,
        }
      );

      if (response.error) {
        throw new Error(response.error);
      }

      return convertToModelPreferences(response.data);
    },
    onSuccess: (data) => {
      queryClient.setQueryData(['model-preferences', userId], data);
    },
  });
}

/**
 * Get model configuration for a specific task type
 * Returns default config if preferences not loaded
 */
export function getModelForTask(
  preferences: ModelPreferences | undefined,
  taskType: TaskType
): ModelConfig {
  if (!preferences) {
    return DEFAULT_MODEL_CONFIG;
  }

  const field = TASK_TYPE_TO_FIELD[taskType];
  return preferences[field];
}

/**
 * React Query hook to fetch model registry
 */
export function useAvailableModels() {
  return useQuery({
    queryKey: ['model-registry'],
    queryFn: async () => {
      const response = await apiClient.get<ModelInfo[]>('/api/models/registry');

      if (response.error) {
        throw new Error(response.error);
      }

      return response.data;
    },
    staleTime: 10 * 60 * 1000, // 10 minutes - model registry rarely changes
    retry: 1,
  });
}

/**
 * React Query hook to fetch models for a specific provider
 */
export function useAvailableModelsForProvider(provider: Provider) {
  const { data: allModels, ...rest } = useAvailableModels();

  const filteredModels = allModels?.filter(model => model.provider === provider) ?? [];

  return {
    data: filteredModels,
    ...rest,
  };
}
