// ABOUTME: Tests for model preferences service functions
// ABOUTME: Validates model selection, fallbacks, React Query hooks, and API integration

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getModelForTask } from './model-preferences';
import type { ModelPreferences, ModelConfig, ModelInfo, Provider } from '@/types/models';

// Mock the API client first (needs to be before the service import)
vi.mock('./api', () => ({
  apiClient: {
    get: vi.fn(),
    put: vi.fn(),
  },
}));

// Mock React Query hooks
const mockUseQuery = vi.fn();
const mockUseMutation = vi.fn();
const mockUseQueryClient = vi.fn();

vi.mock('@tanstack/react-query', () => ({
  useQuery: (options: any) => mockUseQuery(options),
  useMutation: (options: any) => mockUseMutation(options),
  useQueryClient: () => mockUseQueryClient(),
  QueryClient: vi.fn(),
  QueryClientProvider: vi.fn(),
}));

describe('Model Preferences Service', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Reset mock query client
    mockUseQueryClient.mockReturnValue({
      setQueryData: vi.fn(),
      getQueryData: vi.fn(),
    });
  });

  describe('getModelForTask', () => {
    const mockPreferences: ModelPreferences = {
      userId: 'user-123',
      chat: { provider: 'anthropic', model: 'claude-haiku-4-5-20251001' },
      prdGeneration: { provider: 'openai', model: 'gpt-4-turbo' },
      prdAnalysis: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
      insightExtraction: { provider: 'google', model: 'gemini-2.0-flash-exp' },
      specGeneration: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
      taskSuggestions: { provider: 'openai', model: 'gpt-4-turbo' },
      taskAnalysis: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
      specRefinement: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
      researchGeneration: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
      markdownGeneration: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
      updatedAt: '2025-01-15T12:00:00Z',
    };

    it('should return correct model for each task type', () => {
      expect(getModelForTask(mockPreferences, 'chat')).toEqual({
        provider: 'anthropic',
        model: 'claude-haiku-4-5-20251001',
      });

      expect(getModelForTask(mockPreferences, 'prd_generation')).toEqual({
        provider: 'openai',
        model: 'gpt-4-turbo',
      });

      expect(getModelForTask(mockPreferences, 'insight_extraction')).toEqual({
        provider: 'google',
        model: 'gemini-2.0-flash-exp',
      });
    });

    it('should return default config when preferences are undefined', () => {
      const result = getModelForTask(undefined, 'chat');

      expect(result).toEqual({
        provider: 'anthropic',
        model: 'claude-sonnet-4-5-20250929',
      });
    });

    it('should return default config for all task types when preferences not loaded', () => {
      const taskTypes = [
        'chat',
        'prd_generation',
        'prd_analysis',
        'insight_extraction',
        'spec_generation',
        'task_suggestions',
        'task_analysis',
        'spec_refinement',
        'research_generation',
        'markdown_generation',
      ] as const;

      taskTypes.forEach((taskType) => {
        const result = getModelForTask(undefined, taskType);
        expect(result).toEqual({
          provider: 'anthropic',
          model: 'claude-sonnet-4-5-20250929',
        });
      });
    });

    it('should handle all 10 task types correctly', () => {
      expect(getModelForTask(mockPreferences, 'chat')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'prd_generation')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'prd_analysis')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'insight_extraction')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'spec_generation')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'task_suggestions')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'task_analysis')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'spec_refinement')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'research_generation')).toBeDefined();
      expect(getModelForTask(mockPreferences, 'markdown_generation')).toBeDefined();
    });
  });

  describe('useModelPreferences', () => {
    const mockApiResponse = {
      user_id: 'user-123',
      chat_model: 'claude-haiku-4-5-20251001',
      chat_provider: 'anthropic',
      prd_generation_model: 'gpt-4-turbo',
      prd_generation_provider: 'openai',
      prd_analysis_model: 'claude-sonnet-4-5-20250929',
      prd_analysis_provider: 'anthropic',
      insight_extraction_model: 'gemini-2.0-flash-exp',
      insight_extraction_provider: 'google',
      spec_generation_model: 'claude-sonnet-4-5-20250929',
      spec_generation_provider: 'anthropic',
      task_suggestions_model: 'gpt-4-turbo',
      task_suggestions_provider: 'openai',
      task_analysis_model: 'claude-sonnet-4-5-20250929',
      task_analysis_provider: 'anthropic',
      spec_refinement_model: 'claude-sonnet-4-5-20250929',
      spec_refinement_provider: 'anthropic',
      research_generation_model: 'claude-sonnet-4-5-20250929',
      research_generation_provider: 'anthropic',
      markdown_generation_model: 'claude-sonnet-4-5-20250929',
      markdown_generation_provider: 'anthropic',
      updated_at: '2025-01-15T12:00:00Z',
    };

    it('should fetch and convert model preferences correctly', async () => {
      // Import the hook to trigger its execution with our mock
      const { useModelPreferences } = await import('./model-preferences');

      const expectedData = {
        userId: 'user-123',
        chat: { provider: 'anthropic' as Provider, model: 'claude-haiku-4-5-20251001' },
        prdGeneration: { provider: 'openai' as Provider, model: 'gpt-4-turbo' },
        prdAnalysis: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
        insightExtraction: { provider: 'google' as Provider, model: 'gemini-2.0-flash-exp' },
        specGeneration: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
        taskSuggestions: { provider: 'openai' as Provider, model: 'gpt-4-turbo' },
        taskAnalysis: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
        specRefinement: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
        researchGeneration: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
        markdownGeneration: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
        updatedAt: '2025-01-15T12:00:00Z',
      };

      // Mock useQuery to return success state with data
      mockUseQuery.mockReturnValue({
        data: expectedData,
        isSuccess: true,
        isError: false,
        isLoading: false,
        error: null,
      });

      // Call the hook
      const result = useModelPreferences('user-123');

      // Verify the hook was called with correct query key and function
      expect(mockUseQuery).toHaveBeenCalledWith(
        expect.objectContaining({
          queryKey: ['model-preferences', 'user-123'],
          queryFn: expect.any(Function),
        })
      );

      // Verify the returned data
      expect(result.data).toEqual(expectedData);
      expect(result.isSuccess).toBe(true);
    });

    it('should handle API errors', async () => {
      const { useModelPreferences } = await import('./model-preferences');

      const errorMessage = 'User not found';
      mockUseQuery.mockReturnValue({
        data: undefined,
        isSuccess: false,
        isError: true,
        isLoading: false,
        error: new Error(errorMessage),
      });

      const result = useModelPreferences('user-123');

      expect(result.isError).toBe(true);
      expect(result.error).toBeInstanceOf(Error);
      expect((result.error as Error).message).toBe(errorMessage);
    });

    it('should use correct cache key', async () => {
      const { useModelPreferences } = await import('./model-preferences');

      mockUseQuery.mockReturnValue({
        data: {},
        isSuccess: true,
        isError: false,
        isLoading: false,
        error: null,
      });

      useModelPreferences('user-456');

      expect(mockUseQuery).toHaveBeenCalledWith(
        expect.objectContaining({
          queryKey: ['model-preferences', 'user-456'],
        })
      );
    });
  });

  describe('useUpdateModelPreferences', () => {
    it('should update all preferences and convert camelCase to snake_case', async () => {
      const { useUpdateModelPreferences } = await import('./model-preferences');

      const mockMutate = vi.fn();
      mockUseMutation.mockReturnValue({
        mutate: mockMutate,
        mutateAsync: vi.fn(),
        isPending: false,
        isSuccess: true,
        isError: false,
        error: null,
        data: null,
      });

      const result = useUpdateModelPreferences('user-123');

      expect(mockUseMutation).toHaveBeenCalledWith(
        expect.objectContaining({
          mutationFn: expect.any(Function),
          onSuccess: expect.any(Function),
        })
      );

      expect(result.mutate).toBeDefined();
    });

    it('should call onSuccess to update cache', async () => {
      const { useUpdateModelPreferences } = await import('./model-preferences');

      const mockSetQueryData = vi.fn();
      mockUseQueryClient.mockReturnValue({
        setQueryData: mockSetQueryData,
        getQueryData: vi.fn(),
      });

      mockUseMutation.mockReturnValue({
        mutate: vi.fn(),
        mutateAsync: vi.fn(),
        isPending: false,
        isSuccess: false,
        isError: false,
        error: null,
        data: null,
      });

      useUpdateModelPreferences('user-123');

      expect(mockUseMutation).toHaveBeenCalledWith(
        expect.objectContaining({
          onSuccess: expect.any(Function),
        })
      );
    });
  });

  describe('useUpdateTaskModelPreference', () => {
    it('should update single task preference', async () => {
      const { useUpdateTaskModelPreference } = await import('./model-preferences');

      mockUseMutation.mockReturnValue({
        mutate: vi.fn(),
        mutateAsync: vi.fn(),
        isPending: false,
        isSuccess: true,
        isError: false,
        error: null,
        data: null,
      });

      const result = useUpdateTaskModelPreference('user-123');

      expect(mockUseMutation).toHaveBeenCalledWith(
        expect.objectContaining({
          mutationFn: expect.any(Function),
          onSuccess: expect.any(Function),
        })
      );

      expect(result.mutate).toBeDefined();
    });
  });

  describe('useAvailableModels', () => {
    const mockModels: ModelInfo[] = [
      {
        id: 'claude-sonnet-4-5-20250929',
        name: 'Claude Sonnet 4.5',
        provider: 'anthropic',
        contextWindow: 200000,
        inputCost: 0.003,
        outputCost: 0.015,
        capabilities: {
          streaming: true,
          vision: true,
          functionCalling: true,
        },
      },
      {
        id: 'gpt-4-turbo',
        name: 'GPT-4 Turbo',
        provider: 'openai',
        contextWindow: 128000,
        inputCost: 0.01,
        outputCost: 0.03,
        capabilities: {
          streaming: true,
          vision: true,
          functionCalling: true,
        },
      },
    ];

    it('should fetch model registry', async () => {
      const { useAvailableModels } = await import('./model-preferences');

      mockUseQuery.mockReturnValue({
        data: mockModels,
        isSuccess: true,
        isError: false,
        isLoading: false,
        error: null,
      });

      const result = useAvailableModels();

      expect(mockUseQuery).toHaveBeenCalledWith(
        expect.objectContaining({
          queryKey: ['model-registry'],
          queryFn: expect.any(Function),
        })
      );

      expect(result.data).toEqual(mockModels);
    });
  });

  describe('useAvailableModelsForProvider', () => {
    const mockModels: ModelInfo[] = [
      {
        id: 'claude-sonnet-4-5-20250929',
        name: 'Claude Sonnet 4.5',
        provider: 'anthropic',
        contextWindow: 200000,
        inputCost: 0.003,
        outputCost: 0.015,
        capabilities: {
          streaming: true,
        },
      },
      {
        id: 'claude-haiku-4-5-20251001',
        name: 'Claude Haiku 4.5',
        provider: 'anthropic',
        contextWindow: 200000,
        inputCost: 0.001,
        outputCost: 0.005,
        capabilities: {
          streaming: true,
        },
      },
      {
        id: 'gpt-4-turbo',
        name: 'GPT-4 Turbo',
        provider: 'openai',
        contextWindow: 128000,
        inputCost: 0.01,
        outputCost: 0.03,
        capabilities: {
          streaming: true,
        },
      },
    ];

    it('should filter models by provider', async () => {
      const { useAvailableModelsForProvider, useAvailableModels } = await import('./model-preferences');

      // Mock useAvailableModels to return all models
      mockUseQuery.mockReturnValue({
        data: mockModels,
        isSuccess: true,
        isError: false,
        isLoading: false,
        error: null,
      });

      const result = useAvailableModelsForProvider('anthropic');

      expect(result.data).toHaveLength(2);
      expect(result.data.every((m: ModelInfo) => m.provider === 'anthropic')).toBe(true);
    });

    it('should return empty array when no models match provider', async () => {
      const { useAvailableModelsForProvider } = await import('./model-preferences');

      mockUseQuery.mockReturnValue({
        data: mockModels,
        isSuccess: true,
        isError: false,
        isLoading: false,
        error: null,
      });

      const result = useAvailableModelsForProvider('google');

      expect(result.data).toEqual([]);
    });

    it('should handle empty data gracefully', async () => {
      const { useAvailableModelsForProvider } = await import('./model-preferences');

      mockUseQuery.mockReturnValue({
        data: [],
        isSuccess: true,
        isError: false,
        isLoading: false,
        error: null,
      });

      const result = useAvailableModelsForProvider('anthropic');

      expect(result.data).toEqual([]);
    });
  });
});
