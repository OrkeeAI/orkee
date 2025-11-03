// ABOUTME: Integration tests for AI service functions with model preferences
// ABOUTME: Validates correct model usage, provider support, fallbacks, and error handling

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ModelPreferences } from '@/types/models';

// Mock localStorage for rate limiter
global.localStorage = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
  length: 0,
  key: vi.fn(),
};

// Mock the AI SDK before importing services
const mockGenerateObject = vi.fn();
const mockGenerateText = vi.fn();
const mockStreamObject = vi.fn();
const mockStreamText = vi.fn();

vi.mock('ai', () => ({
  generateObject: (...args: any[]) => mockGenerateObject(...args),
  generateText: (...args: any[]) => mockGenerateText(...args),
  streamObject: (...args: any[]) => mockStreamObject(...args),
  streamText: (...args: any[]) => mockStreamText(...args),
}));

// Mock the providers
const mockAnthropicModel = { id: 'anthropic-mock', provider: 'anthropic' };
const mockOpenAIModel = { id: 'openai-mock', provider: 'openai' };
const mockGoogleModel = { id: 'google-mock', provider: 'google' };
const mockXAIModel = { id: 'xai-mock', provider: 'xai' };

vi.mock('./providers', () => ({
  anthropic: vi.fn(() => mockAnthropicModel),
  openai: vi.fn(() => mockOpenAIModel),
  getPreferredModel: vi.fn(() => ({ model: mockAnthropicModel, modelName: 'claude-sonnet-4-5-20250929' })),
}));

vi.mock('./config', () => ({
  getModelInstance: vi.fn((provider: string) => {
    switch (provider) {
      case 'anthropic': return mockAnthropicModel;
      case 'openai': return mockOpenAIModel;
      case 'google': return mockGoogleModel;
      case 'xai': return mockXAIModel;
      default: return mockAnthropicModel;
    }
  }),
  calculateCost: vi.fn(() => ({
    inputCost: 1.5,
    outputCost: 7.5,
    totalCost: 9.0,
  })),
  AI_CONFIG: {
    defaults: {
      maxTokens: 4096,
      temperature: 0.7,
      topP: 1,
    },
    features: {
      streaming: true,
      caching: true,
      rateLimiting: true,
    },
    rateLimits: {
      requestsPerMinute: 60,
      tokensPerMinute: 100000,
    },
    sizeLimits: {
      maxPRDTokens: 100000,
      chunkSize: 30000,
      promptOverhead: 500,
      timeoutMs: 120000,
    },
  },
}));

// Mock rate limiter to allow all calls
vi.mock('./rate-limiter', () => ({
  aiRateLimiter: {
    canMakeCall: vi.fn(() => ({ allowed: true })),
    recordCall: vi.fn(),
  },
}));

// Mock cache to return null (no cached results)
vi.mock('./cache', () => ({
  aiCache: {
    get: vi.fn(() => null),
    set: vi.fn(),
  },
}));

// Mock model preferences
vi.mock('@/services/model-preferences', () => ({
  getModelForTask: vi.fn((preferences: any, taskType: string) => {
    if (!preferences) {
      return { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' };
    }
    // Convert snake_case to camelCase for lookup
    const key = taskType.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
    return preferences[key as keyof typeof preferences];
  }),
}));

// Import services after mocks are set up
import { AISpecService } from './services';

describe('AI Services Integration Tests', () => {
  let aiService: AISpecService;

  beforeEach(() => {
    vi.clearAllMocks();
    aiService = new AISpecService();

    // Default mock response for analyzePRD
    mockGenerateObject.mockResolvedValue({
      object: {
        capabilities: [
          {
            id: 'test-capability',
            name: 'Test Capability',
            description: 'A test capability',
            priority: 'high' as const,
            complexity_score: 5,
            requirements: [],
          },
        ],
        suggested_tasks: [],
        dependencies: [],
        risks: [],
      },
      usage: {
        promptTokens: 100,
        completionTokens: 50,
        totalTokens: 150,
      },
    });
  });

  describe('Model Preference Selection', () => {
    it('should use anthropic model from preferences for PRD analysis', async () => {
      const mockPreferences: Partial<ModelPreferences> = {
        prdAnalysis: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
      };

      await aiService.analyzePRD('Test PRD content', mockPreferences.prdAnalysis);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
    });

    it('should use openai model from preferences for PRD analysis', async () => {
      const mockPreferences: Partial<ModelPreferences> = {
        prdAnalysis: { provider: 'openai', model: 'gpt-4-turbo' },
      };

      await aiService.analyzePRD('Test PRD content', mockPreferences.prdAnalysis);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockOpenAIModel);
    });

    it('should use google model from preferences for PRD analysis', async () => {
      const mockPreferences: Partial<ModelPreferences> = {
        prdAnalysis: { provider: 'google', model: 'gemini-2.0-flash-exp' },
      };

      await aiService.analyzePRD('Test PRD content', mockPreferences.prdAnalysis);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockGoogleModel);
    });

    it('should use xai model from preferences for PRD analysis', async () => {
      const mockPreferences: Partial<ModelPreferences> = {
        prdAnalysis: { provider: 'xai', model: 'grok-2' },
      };

      await aiService.analyzePRD('Test PRD content', mockPreferences.prdAnalysis);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockXAIModel);
    });
  });

  describe('Fallback to Defaults', () => {
    it('should use default model when preferences not provided', async () => {
      await aiService.analyzePRD('Test PRD content');

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
    });

    it('should use default anthropic provider when preferences undefined', async () => {
      await aiService.analyzePRD('Test PRD content', undefined);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
    });
  });

  describe('All Provider Support', () => {
    it('should support all 4 providers (anthropic, openai, google, xai)', async () => {
      const providers = [
        { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929', expectedModel: mockAnthropicModel },
        { provider: 'openai' as const, model: 'gpt-4-turbo', expectedModel: mockOpenAIModel },
        { provider: 'google' as const, model: 'gemini-2.0-flash-exp', expectedModel: mockGoogleModel },
        { provider: 'xai' as const, model: 'grok-2', expectedModel: mockXAIModel },
      ];

      for (const { provider, model, expectedModel } of providers) {
        vi.clearAllMocks();

        await aiService.analyzePRD('Test PRD', { provider, model });

        expect(mockGenerateObject).toHaveBeenCalledTimes(1);
        const call = mockGenerateObject.mock.calls[0][0];
        expect(call.model).toBe(expectedModel);
      }
    });
  });

  describe('Error Handling', () => {
    it('should throw AIServiceError when AI call fails', async () => {
      mockGenerateObject.mockRejectedValueOnce(new Error('API error'));

      await expect(
        aiService.analyzePRD('Test PRD')
      ).rejects.toThrow('Failed to analyze PRD');
    });

    it('should handle rate limit errors', async () => {
      const { aiRateLimiter } = await import('./rate-limiter');
      (aiRateLimiter.canMakeCall as any).mockReturnValueOnce({
        allowed: false,
        reason: 'Rate limit exceeded',
      });

      await expect(
        aiService.analyzePRD('Test PRD')
      ).rejects.toThrow('Rate limit exceeded');
    });

    it('should handle timeout errors', async () => {
      const timeoutError = new Error('Timeout');
      timeoutError.name = 'TimeoutError';
      mockGenerateObject.mockRejectedValueOnce(timeoutError);

      await expect(
        aiService.analyzePRD('Test PRD')
      ).rejects.toThrow();
    });

    it('should handle network errors', async () => {
      const networkError = new Error('Network error');
      networkError.name = 'NetworkError';
      mockGenerateObject.mockRejectedValueOnce(networkError);

      await expect(
        aiService.analyzePRD('Test PRD')
      ).rejects.toThrow();
    });

    it('should handle validation errors', async () => {
      const validationError = new Error('Invalid response');
      validationError.name = 'ValidationError';
      mockGenerateObject.mockRejectedValueOnce(validationError);

      await expect(
        aiService.analyzePRD('Test PRD')
      ).rejects.toThrow();
    });
  });

  describe('Response Structure Validation', () => {
    it('should return properly structured result with usage and cost', async () => {
      const result = await aiService.analyzePRD('Test PRD');

      expect(result).toHaveProperty('data');
      expect(result).toHaveProperty('usage');
      expect(result).toHaveProperty('cost');
      expect(result).toHaveProperty('model');
      expect(result).toHaveProperty('provider');

      expect(result.usage.inputTokens).toBeGreaterThan(0);
      expect(result.usage.outputTokens).toBeGreaterThan(0);
      expect(result.usage.totalTokens).toBeGreaterThan(0);
    });

    it('should include PRD analysis capabilities in response', async () => {
      const result = await aiService.analyzePRD('Test PRD');

      expect(result.data).toHaveProperty('capabilities');
      expect(Array.isArray(result.data.capabilities)).toBe(true);
    });

    it('should include suggested tasks in response', async () => {
      const result = await aiService.analyzePRD('Test PRD');

      expect(result.data).toHaveProperty('suggested_tasks');
      expect(Array.isArray(result.data.suggested_tasks)).toBe(true);
    });
  });

  describe('Model Selection Consistency', () => {
    it('should use same model across multiple calls when preferences are stable', async () => {
      const preferences = { provider: 'openai' as const, model: 'gpt-4-turbo' };

      await aiService.analyzePRD('Test PRD 1', preferences);
      await aiService.analyzePRD('Test PRD 2', preferences);

      expect(mockGenerateObject).toHaveBeenCalledTimes(2);
      const call1 = mockGenerateObject.mock.calls[0][0];
      const call2 = mockGenerateObject.mock.calls[1][0];
      expect(call1.model).toBe(call2.model);
      expect(call1.model).toBe(mockOpenAIModel);
    });

    it('should change model when preferences change', async () => {
      const preferences1 = { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
      const preferences2 = { provider: 'openai' as const, model: 'gpt-4-turbo' };

      await aiService.analyzePRD('Test PRD 1', preferences1);
      await aiService.analyzePRD('Test PRD 2', preferences2);

      expect(mockGenerateObject).toHaveBeenCalledTimes(2);
      const call1 = mockGenerateObject.mock.calls[0][0];
      const call2 = mockGenerateObject.mock.calls[1][0];
      expect(call1.model).toBe(mockAnthropicModel);
      expect(call2.model).toBe(mockOpenAIModel);
    });
  });

  describe('Caching Behavior', () => {
    it('should check cache before making AI call', async () => {
      const { aiCache } = await import('./cache');

      await aiService.analyzePRD('Test PRD');

      expect(aiCache.get).toHaveBeenCalledWith('analyzePRD', { prdContent: 'Test PRD' });
    });

    it('should return cached result when available', async () => {
      const { aiCache } = await import('./cache');
      const cachedResult = {
        data: { capabilities: [], suggested_tasks: [], dependencies: [], risks: [] },
        usage: { inputTokens: 100, outputTokens: 50, totalTokens: 150 },
        cost: { inputCost: 1, outputCost: 2, totalCost: 3 },
        model: 'cached-model',
        provider: 'anthropic' as const,
      };

      (aiCache.get as any).mockReturnValueOnce(cachedResult);

      const result = await aiService.analyzePRD('Test PRD');

      expect(result).toBe(cachedResult);
      expect(mockGenerateObject).not.toHaveBeenCalled();
    });
  });

  describe('Rate Limiting', () => {
    it('should check rate limit before making AI call', async () => {
      const { aiRateLimiter } = await import('./rate-limiter');

      await aiService.analyzePRD('Test PRD');

      expect(aiRateLimiter.canMakeCall).toHaveBeenCalledWith('analyzePRD');
    });
  });
});
