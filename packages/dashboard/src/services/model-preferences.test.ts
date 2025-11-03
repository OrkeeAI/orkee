// ABOUTME: Tests for model preferences service functions
// ABOUTME: Validates model selection, fallbacks, React Query hooks, and API integration

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import {
  useModelPreferences,
  useUpdateModelPreferences,
  useUpdateTaskModelPreference,
  getModelForTask,
  useAvailableModels,
  useAvailableModelsForProvider,
} from './model-preferences';
import { apiClient } from './api';
import type { ModelPreferences, ModelConfig, ModelInfo, Provider } from '@/types/models';

// Mock the API client
vi.mock('./api', () => ({
  apiClient: {
    get: vi.fn(),
    put: vi.fn(),
  },
}));

describe('Model Preferences Service', () => {
  let queryClient: QueryClient;

  const createWrapper = () => {
    return ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
        mutations: {
          retry: false,
        },
      },
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
      (apiClient.get as any).mockResolvedValue({
        success: true,
        data: mockApiResponse,
        error: undefined,
      });

      const { result } = renderHook(() => useModelPreferences('user-123'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      expect(result.current.data).toEqual({
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
      });

      expect(apiClient.get).toHaveBeenCalledWith('/api/users/user-123/model-preferences');
    });

    it('should handle API errors', async () => {
      (apiClient.get as any).mockResolvedValue({
        success: false,
        data: null,
        error: 'User not found',
      });

      const { result } = renderHook(() => useModelPreferences('user-123'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isError).toBe(true));

      expect(result.current.error).toBeInstanceOf(Error);
      expect((result.current.error as Error).message).toBe('User not found');
    });

    it('should use correct cache key', async () => {
      (apiClient.get as any).mockResolvedValue({
        success: true,
        data: mockApiResponse,
        error: undefined,
      });

      const { result } = renderHook(() => useModelPreferences('user-123'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      const cachedData = queryClient.getQueryData(['model-preferences', 'user-123']);
      expect(cachedData).toBeDefined();
    });
  });

  describe('useUpdateModelPreferences', () => {
    it('should update all preferences and convert camelCase to snake_case', async () => {
      const mockResponse = {
        user_id: 'user-123',
        chat_model: 'claude-opus-4-1-20250805',
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
        updated_at: '2025-01-15T13:00:00Z',
      };

      (apiClient.put as any).mockResolvedValue({
        success: true,
        data: mockResponse,
        error: undefined,
      });

      const { result } = renderHook(() => useUpdateModelPreferences('user-123'), {
        wrapper: createWrapper(),
      });

      const updateData = {
        chat: { provider: 'anthropic' as Provider, model: 'claude-opus-4-1-20250805' },
      };

      result.current.mutate(updateData);

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      expect(apiClient.put).toHaveBeenCalledWith('/api/users/user-123/model-preferences', {
        chat_provider: 'anthropic',
        chat_model: 'claude-opus-4-1-20250805',
      });
    });

    it('should update cache on successful mutation', async () => {
      const mockResponse = {
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
        updated_at: '2025-01-15T13:00:00Z',
      };

      (apiClient.put as any).mockResolvedValue({
        success: true,
        data: mockResponse,
        error: undefined,
      });

      const { result } = renderHook(() => useUpdateModelPreferences('user-123'), {
        wrapper: createWrapper(),
      });

      result.current.mutate({
        chat: { provider: 'anthropic', model: 'claude-haiku-4-5-20251001' },
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      const cachedData = queryClient.getQueryData<ModelPreferences>([
        'model-preferences',
        'user-123',
      ]);
      expect(cachedData?.chat.model).toBe('claude-haiku-4-5-20251001');
    });
  });

  describe('useUpdateTaskModelPreference', () => {
    it('should update single task preference', async () => {
      const mockResponse = {
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
        updated_at: '2025-01-15T13:00:00Z',
      };

      (apiClient.put as any).mockResolvedValue({
        success: true,
        data: mockResponse,
        error: undefined,
      });

      const { result } = renderHook(() => useUpdateTaskModelPreference('user-123'), {
        wrapper: createWrapper(),
      });

      result.current.mutate({
        taskType: 'chat',
        config: { provider: 'anthropic', model: 'claude-haiku-4-5-20251001' },
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      expect(apiClient.put).toHaveBeenCalledWith('/api/users/user-123/model-preferences/chat', {
        provider: 'anthropic',
        model: 'claude-haiku-4-5-20251001',
      });
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
      (apiClient.get as any).mockResolvedValue({
        success: true,
        data: mockModels,
        error: undefined,
      });

      const { result } = renderHook(() => useAvailableModels(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      expect(result.current.data).toEqual(mockModels);
      expect(apiClient.get).toHaveBeenCalledWith('/api/models/registry');
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
      (apiClient.get as any).mockResolvedValue({
        success: true,
        data: mockModels,
        error: undefined,
      });

      const { result } = renderHook(() => useAvailableModelsForProvider('anthropic'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      expect(result.current.data).toHaveLength(2);
      expect(result.current.data?.every((m) => m.provider === 'anthropic')).toBe(true);
    });

    it('should return empty array when no models match provider', async () => {
      (apiClient.get as any).mockResolvedValue({
        success: true,
        data: mockModels,
        error: undefined,
      });

      const { result } = renderHook(() => useAvailableModelsForProvider('google'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      expect(result.current.data).toEqual([]);
    });

    it('should handle undefined data gracefully', async () => {
      (apiClient.get as any).mockResolvedValue({
        success: true,
        data: undefined,
        error: undefined,
      });

      const { result } = renderHook(() => useAvailableModelsForProvider('anthropic'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));

      expect(result.current.data).toEqual([]);
    });
  });
});
