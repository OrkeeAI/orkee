// ABOUTME: Integration tests for dependency analysis AI service
// ABOUTME: Validates dependency detection, build order, and quick wins generation without making real AI calls

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { IdeateFeature } from '@/types/ideate';

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

vi.mock('ai', () => ({
  generateObject: (...args: any[]) => mockGenerateObject(...args),
}));

// Mock the providers
const mockAnthropicModel = { id: 'anthropic-mock', provider: 'anthropic' };

vi.mock('@/lib/ai/config', () => ({
  getModelInstance: vi.fn(() => mockAnthropicModel),
  calculateCost: vi.fn(() => ({
    inputCost: 1.5,
    outputCost: 7.5,
    totalCost: 9.0,
  })),
  getModelForTask: vi.fn(() => ({
    provider: 'anthropic',
    model: 'claude-sonnet-4-5-20250929',
  })),
}));

// Mock telemetry tracking
const mockTrackAIOperationWithCost = vi.fn((operationName, projectId, model, provider, costFn, operation) => {
  return operation();
});

vi.mock('@/lib/ai/telemetry', () => ({
  trackAIOperationWithCost: (...args: any[]) => mockTrackAIOperationWithCost(...args),
}));

// Mock ideate service to prevent real HTTP calls
const mockCreateFeatureDependency = vi.fn().mockImplementation((sessionId, input) => {
  return Promise.resolve({
    id: 'dep-1',
    session_id: sessionId,
    from_feature_id: input.fromFeatureId,
    to_feature_id: input.toFeatureId,
    dependency_type: input.dependencyType,
    strength: input.strength,
    reason: input.reason,
    created_at: new Date().toISOString(),
  });
});

vi.mock('@/services/ideate', () => ({
  ideateService: {
    createFeatureDependency: (...args: any[]) => mockCreateFeatureDependency(...args),
  },
}));

// Import the service after mocks are set up
import {
  analyzeDependencies,
  suggestBuildOrder,
  suggestQuickWins,
} from './dependency-ai';

describe('Dependency Analysis AI Service', () => {
  const mockFeatures: IdeateFeature[] = [
    {
      id: 'feature-1',
      session_id: 'session-1',
      name: 'User Authentication',
      description: 'Implement user login and signup',
      category: 'authentication',
      priority: 'critical',
      complexity: 'high',
      technical_details: 'JWT-based authentication with refresh tokens',
      importance: 'critical',
      confidence_score: 0.9,
      status: 'pending',
      feature_type: 'feature',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
    {
      id: 'feature-2',
      session_id: 'session-1',
      name: 'User Profile',
      description: 'Display and edit user profile',
      category: 'user-management',
      priority: 'high',
      complexity: 'medium',
      technical_details: 'React components with form validation',
      importance: 'high',
      confidence_score: 0.8,
      status: 'pending',
      feature_type: 'feature',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  ] as any[];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('analyzeDependencies', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          dependencies: [
            {
              from_feature_id: 'feature-1',
              to_feature_id: 'feature-2',
              dependency_type: 'technical',
              strength: 'required',
              reason: 'Profile requires authentication',
              confidence_score: 0.9,
            },
          ],
          analysis_summary: 'Auth is required before profile',
          recommendations: ['Implement auth first', 'Consider API design'],
        },
        usage: {
          promptTokens: 200,
          completionTokens: 100,
          totalTokens: 300,
        },
      });
    });

    it('should analyze dependencies and return structured result', async () => {
      const result = await analyzeDependencies('session-1', mockFeatures);

      expect(result).toHaveProperty('detected_dependencies');
      expect(result).toHaveProperty('analysis_summary');
      expect(result).toHaveProperty('recommendations');
      expect(Array.isArray(result.detected_dependencies)).toBe(true);
      expect(Array.isArray(result.recommendations)).toBe(true);
    });

    it('should call generateObject with correct schema', async () => {
      await analyzeDependencies('session-1', mockFeatures);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
      expect(call.schema).toBeDefined();
      expect(call.prompt).toContain('User Authentication');
      expect(call.prompt).toContain('User Profile');
    });

    it('should return dependencies from AI analysis', async () => {
      const result = await analyzeDependencies('session-1', mockFeatures);

      // Verify the AI-generated dependencies are returned correctly
      expect(result.detected_dependencies).toHaveLength(1);
      expect(result.detected_dependencies[0].from_feature_id).toBe('feature-1');
      expect(result.detected_dependencies[0].to_feature_id).toBe('feature-2');
      expect(result.detected_dependencies[0].dependency_type).toBe('technical');
      expect(result.detected_dependencies[0].strength).toBe('required');
    });

    it('should track telemetry for AI operation', async () => {
      await analyzeDependencies('session-1', mockFeatures, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'analyze_dependencies',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should use custom model preferences when provided', async () => {
      const customPrefs = { provider: 'openai' as const, model: 'gpt-4-turbo' };

      await analyzeDependencies('session-1', mockFeatures, customPrefs);

      // Verify getModelInstance was called with the custom provider
      const { getModelInstance } = await import('@/lib/ai/config');
      expect(getModelInstance).toHaveBeenCalledWith('openai', 'gpt-4-turbo');
    });

    it('should handle empty features list', async () => {
      // Mock needs to return empty dependencies array
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          dependencies: [],
          analysis_summary: 'No features to analyze',
          recommendations: [],
        },
        usage: {
          promptTokens: 50,
          completionTokens: 20,
          totalTokens: 70,
        },
      });

      const result = await analyzeDependencies('session-1', []);

      expect(mockGenerateObject).toHaveBeenCalled();
      expect(result.detected_dependencies).toEqual([]);
      const call = mockGenerateObject.mock.calls[0][0];
      // Check that prompt was called with dependency analysis instructions
      expect(call.prompt).toContain('Analyze these feature descriptions');
      expect(call.prompt).toContain('identify dependencies');
    });

    it('should handle AI service errors gracefully', async () => {
      mockGenerateObject.mockRejectedValueOnce(new Error('API error'));

      await expect(
        analyzeDependencies('session-1', mockFeatures)
      ).rejects.toThrow('API error');
    });
  });

  describe('suggestBuildOrder', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          suggested_order: ['feature-1', 'feature-2'],
          rationale: 'Auth must come before profile',
          parallel_groups: [
            {
              features: ['feature-2'],
              description: 'Can be built after auth',
            },
          ],
          critical_path: ['feature-1', 'feature-2'],
          risks: ['Auth complexity might delay profile'],
        },
        usage: {
          promptTokens: 150,
          completionTokens: 80,
          totalTokens: 230,
        },
      });
    });

    it('should suggest build order with phases', async () => {
      const result = await suggestBuildOrder('session-1', mockFeatures, []);

      expect(result).toHaveProperty('suggested_order');
      expect(result).toHaveProperty('rationale');
      expect(result).toHaveProperty('parallel_groups');
      expect(result).toHaveProperty('critical_path');
      expect(result).toHaveProperty('risks');
      expect(Array.isArray(result.suggested_order)).toBe(true);
      expect(result.suggested_order.length).toBe(2);
    });

    it('should call generateObject with correct schema', async () => {
      await suggestBuildOrder('session-1', mockFeatures, []);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
      expect(call.schema).toBeDefined();
      expect(call.prompt).toContain('FEATURES');
    });

    it('should include dependency information in prompt when provided', async () => {
      const dependencies = [
        {
          from_feature_id: 'feature-1',
          to_feature_id: 'feature-2',
          dependency_type: 'technical' as const,
          strength: 'required' as const,
          reason: 'Profile requires authentication',
        },
      ] as any[];

      await suggestBuildOrder('session-1', mockFeatures, dependencies);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('DEPENDENCIES');
      expect(call.prompt).toContain('User Authentication');
    });

    it('should track telemetry for build order operation', async () => {
      await suggestBuildOrder('session-1', mockFeatures, [], undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'suggest_build_order',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });
  });

  describe('suggestQuickWins', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          quick_wins: [
            {
              feature_id: 'feature-2',
              name: 'User Profile',
              rationale: 'Simple UI work, no complex logic',
              estimated_effort: 4,
              expected_impact: 'High user engagement',
            },
          ],
          explanation: 'Focus on high-value, low-effort features',
        },
        usage: {
          promptTokens: 120,
          completionTokens: 60,
          totalTokens: 180,
        },
      });
    });

    it('should suggest quick wins with effort estimates', async () => {
      const result = await suggestQuickWins('session-1', mockFeatures, []);

      expect(result).toHaveProperty('quick_wins');
      expect(result).toHaveProperty('explanation');
      expect(Array.isArray(result.quick_wins)).toBe(true);
      expect(result.quick_wins.length).toBe(1);
      expect(result.quick_wins[0]).toHaveProperty('estimated_effort');
      expect(result.quick_wins[0]).toHaveProperty('expected_impact');
    });

    it('should call generateObject with correct schema', async () => {
      await suggestQuickWins('session-1', mockFeatures, []);

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
      expect(call.schema).toBeDefined();
      expect(call.prompt).toContain('quick win');
    });

    it('should return empty quick wins when all features have dependencies', async () => {
      const dependencies = [
        {
          from_feature_id: 'feature-1',
          to_feature_id: 'feature-2',
          dependency_type: 'technical' as const,
          strength: 'required' as const,
          reason: 'Auth required',
        },
        {
          from_feature_id: 'feature-2',
          to_feature_id: 'feature-1',
          dependency_type: 'technical' as const,
          strength: 'required' as const,
          reason: 'Profile depends on auth',
        },
      ] as any[];

      const result = await suggestQuickWins('session-1', mockFeatures, dependencies);

      expect(result.quick_wins).toEqual([]);
      expect(result.explanation).toContain('No independent features found');
    });

    it('should track telemetry for quick wins operation', async () => {
      await suggestQuickWins('session-1', mockFeatures, [], undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'suggest_quick_wins',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });
  });

  describe('Error Handling', () => {
    it('should handle network errors in analyzeDependencies', async () => {
      const networkError = new Error('Network error');
      mockGenerateObject.mockRejectedValueOnce(networkError);

      await expect(
        analyzeDependencies('session-1', mockFeatures)
      ).rejects.toThrow('Network error');
    });

    it('should handle validation errors in suggestBuildOrder', async () => {
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          suggested_order: [],
          rationale: 'No order determined',
          parallel_groups: [],
          critical_path: [],
          risks: [],
        },
        usage: { promptTokens: 10, completionTokens: 5, totalTokens: 15 },
      });

      // Should still return the result (Zod validation happens in the AI SDK)
      const result = await suggestBuildOrder('session-1', mockFeatures, []);
      expect(result.suggested_order).toEqual([]);
    });

    it('should provide recommendations based on analysis', async () => {
      // Set up mock for this test
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          dependencies: [
            {
              from_feature_id: 'feature-1',
              to_feature_id: 'feature-2',
              dependency_type: 'technical',
              strength: 'required',
              reason: 'Profile requires authentication',
              confidence_score: 0.9,
            },
          ],
          analysis_summary: 'Auth is required before profile',
          recommendations: ['Implement auth first', 'Consider API design'],
        },
        usage: { promptTokens: 200, completionTokens: 100, totalTokens: 300 },
      });

      const result = await analyzeDependencies('session-1', mockFeatures);

      expect(result.recommendations).toBeDefined();
      expect(Array.isArray(result.recommendations)).toBe(true);
      expect(result.recommendations[0]).toBe('Implement auth first');
    });
  });

  describe('Integration', () => {
    it('should work end-to-end: analyze -> build order -> quick wins', async () => {
      // First analyze dependencies
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          dependencies: [
            {
              from_feature_id: 'feature-1',
              to_feature_id: 'feature-2',
              dependency_type: 'technical',
              strength: 'required',
              reason: 'Profile requires authentication',
              confidence_score: 0.9,
            },
          ],
          analysis_summary: 'Auth is required first',
          recommendations: ['Implement auth first'],
        },
        usage: { promptTokens: 200, completionTokens: 100, totalTokens: 300 },
      });

      const depResult = await analyzeDependencies('session-1', mockFeatures);

      // Then suggest build order using those dependencies
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          suggested_order: ['feature-1', 'feature-2'],
          rationale: 'Auth must come first',
          parallel_groups: [],
          critical_path: ['feature-1', 'feature-2'],
          risks: ['Auth complexity'],
        },
        usage: { promptTokens: 150, completionTokens: 80, totalTokens: 230 },
      });

      const buildOrder = await suggestBuildOrder(
        'session-1',
        mockFeatures,
        depResult.detected_dependencies
      );

      // Finally suggest quick wins
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          quick_wins: [
            {
              feature_id: 'feature-2',
              name: 'User Profile',
              rationale: 'Simple after auth is done',
              estimated_effort: 4,
              expected_impact: 'High user engagement',
            },
          ],
          explanation: 'Build profile after auth',
        },
        usage: { promptTokens: 120, completionTokens: 60, totalTokens: 180 },
      });

      const quickWins = await suggestQuickWins(
        'session-1',
        mockFeatures,
        depResult.detected_dependencies
      );

      expect(depResult.detected_dependencies).toHaveLength(1);
      expect(buildOrder.suggested_order).toHaveLength(2);
      expect(quickWins.quick_wins).toHaveLength(1);
      expect(mockGenerateObject).toHaveBeenCalledTimes(3);
    });
  });
});
