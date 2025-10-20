// ABOUTME: React Query hooks for AI operations
// ABOUTME: Provides type-safe hooks for PRD analysis, spec generation, and task suggestions

import { useMutation, useQuery } from '@tanstack/react-query';
import { aiSpecService, createSpecWorkflow } from '@/lib/ai';
import type {
  PRDAnalysis,
  SpecCapability,
  TaskSuggestion,
  OrphanTaskAnalysis,
  TaskValidation,
  SpecRefinement,
} from '@/lib/ai/schemas';
import type { AIResult } from '@/lib/ai/services';

/**
 * Hook for analyzing a PRD document
 */
export function useAnalyzePRD() {
  return useMutation({
    mutationFn: async (prdContent: string): Promise<AIResult<PRDAnalysis>> => {
      return await aiSpecService.analyzePRD(prdContent);
    },
  });
}

/**
 * Hook for generating a spec from requirements
 */
export function useGenerateSpec() {
  return useMutation({
    mutationFn: async ({
      capabilityName,
      purpose,
      requirements,
    }: {
      capabilityName: string;
      purpose: string;
      requirements?: string[];
    }): Promise<AIResult<SpecCapability>> => {
      return await aiSpecService.generateSpec(capabilityName, purpose, requirements);
    },
  });
}

/**
 * Hook for suggesting tasks from a capability
 */
export function useSuggestTasks() {
  return useMutation({
    mutationFn: async ({
      capability,
      existingTasks,
    }: {
      capability: SpecCapability;
      existingTasks?: string[];
    }): Promise<AIResult<TaskSuggestion[]>> => {
      return await aiSpecService.suggestTasks(capability, existingTasks);
    },
  });
}

/**
 * Hook for analyzing an orphan task
 */
export function useAnalyzeOrphanTask() {
  return useMutation({
    mutationFn: async ({
      task,
      existingCapabilities,
    }: {
      task: { title: string; description: string };
      existingCapabilities: Array<{ id: string; name: string; purpose: string }>;
    }): Promise<AIResult<OrphanTaskAnalysis>> => {
      return await aiSpecService.analyzeOrphanTask(task, existingCapabilities);
    },
  });
}

/**
 * Hook for validating task completion
 */
export function useValidateTask() {
  return useMutation({
    mutationFn: async ({
      task,
      scenarios,
    }: {
      task: { title: string; description: string; implementation?: string };
      scenarios: Array<{ name: string; when: string; then: string; and?: string[] }>;
    }): Promise<AIResult<TaskValidation>> => {
      return await aiSpecService.validateTaskCompletion(task, scenarios);
    },
  });
}

/**
 * Hook for refining a spec
 */
export function useRefineSpec() {
  return useMutation({
    mutationFn: async ({
      capability,
      feedback,
    }: {
      capability: SpecCapability;
      feedback: string;
    }): Promise<AIResult<SpecRefinement>> => {
      return await aiSpecService.refineSpec(capability, feedback);
    },
  });
}

/**
 * Hook for generating spec markdown
 */
export function useGenerateSpecMarkdown() {
  return useMutation({
    mutationFn: async (capability: SpecCapability): Promise<AIResult<string>> => {
      return await aiSpecService.generateSpecMarkdown(capability);
    },
  });
}

/**
 * Hook for complete PRD workflow with progress tracking
 */
export function usePRDWorkflow() {
  return useMutation({
    mutationFn: async ({
      prdId,
      prdContent,
      projectId,
      onProgress,
    }: {
      prdId: string;
      prdContent: string;
      projectId: string;
      onProgress?: (step: string, progress: number) => void;
    }) => {
      const workflow = createSpecWorkflow(onProgress);
      return await workflow.processNewPRD(prdId, prdContent, projectId);
    },
  });
}

/**
 * Hook for orphan task workflow
 */
export function useOrphanTaskWorkflow() {
  return useMutation({
    mutationFn: async ({
      task,
      existingCapabilities,
      onProgress,
    }: {
      task: { id: string; title: string; description: string };
      existingCapabilities: Array<{ id: string; name: string; purpose: string }>;
      onProgress?: (step: string, progress: number) => void;
    }) => {
      const workflow = createSpecWorkflow(onProgress);
      return await workflow.syncOrphanTask(task, existingCapabilities);
    },
  });
}

/**
 * Hook to check if AI is configured
 */
export function useAIConfiguration() {
  return useQuery({
    queryKey: ['ai-configuration'],
    queryFn: async () => {
      const { isProviderConfigured, getPreferredProvider } = await import('@/lib/ai/config');

      const openaiConfigured = isProviderConfigured('openai');
      const anthropicConfigured = isProviderConfigured('anthropic');
      const preferredProvider = getPreferredProvider();

      return {
        isConfigured: openaiConfigured || anthropicConfigured,
        openaiConfigured,
        anthropicConfigured,
        preferredProvider,
      };
    },
    staleTime: Infinity, // Configuration doesn't change during runtime
  });
}

/**
 * Hook to track cumulative AI costs
 */
export function useAICostTracker() {
  const { data: costs = [], addCost } = useCostStorage();

  const totalCost = costs.reduce((sum, cost) => sum + cost.estimatedCost, 0);
  const totalTokens = costs.reduce((sum, cost) => sum + cost.totalTokens, 0);

  const costsByProvider = costs.reduce(
    (acc, cost) => {
      acc[cost.provider] = (acc[cost.provider] || 0) + cost.estimatedCost;
      return acc;
    },
    {} as Record<string, number>
  );

  const costsByModel = costs.reduce(
    (acc, cost) => {
      acc[cost.model] = (acc[cost.model] || 0) + cost.estimatedCost;
      return acc;
    },
    {} as Record<string, number>
  );

  return {
    costs,
    totalCost,
    totalTokens,
    costsByProvider,
    costsByModel,
    addCost,
  };
}

/**
 * Simple in-memory cost storage (replace with proper persistence later)
 */
function useCostStorage() {
  // This would be replaced with proper state management or localStorage
  const costs: Array<{
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
    estimatedCost: number;
    model: string;
    provider: 'openai' | 'anthropic';
    timestamp: Date;
  }> = [];

  const addCost = (cost: {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
    estimatedCost: number;
    model: string;
    provider: 'openai' | 'anthropic';
  }) => {
    costs.push({
      ...cost,
      timestamp: new Date(),
    });
  };

  return { data: costs, addCost };
}
