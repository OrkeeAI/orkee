// ABOUTME: TypeScript type definitions for model preferences and AI provider configuration
// ABOUTME: Defines types for task-specific model selection, provider configuration, and model registry data

/**
 * Task types for per-task model configuration
 */
export type TaskType =
  | 'chat'
  | 'prd_generation'
  | 'prd_analysis'
  | 'insight_extraction'
  | 'spec_generation'
  | 'task_suggestions'
  | 'task_analysis'
  | 'spec_refinement'
  | 'research_generation'
  | 'markdown_generation';

/**
 * AI provider identifiers
 */
export type Provider = 'anthropic' | 'openai' | 'google' | 'xai';

/**
 * Model configuration for a specific task
 */
export interface ModelConfig {
  provider: Provider;
  model: string;
}

/**
 * Complete model preferences for all task types
 */
export interface ModelPreferences {
  userId: string;
  chat: ModelConfig;
  prdGeneration: ModelConfig;
  prdAnalysis: ModelConfig;
  insightExtraction: ModelConfig;
  specGeneration: ModelConfig;
  taskSuggestions: ModelConfig;
  taskAnalysis: ModelConfig;
  specRefinement: ModelConfig;
  researchGeneration: ModelConfig;
  markdownGeneration: ModelConfig;
  updatedAt: string;
}

/**
 * Model information from the model registry
 */
export interface ModelInfo {
  id: string;
  name: string;
  provider: Provider;
  contextWindow: number;
  inputCost: number;
  outputCost: number;
  capabilities: {
    streaming?: boolean;
    vision?: boolean;
    functionCalling?: boolean;
  };
}

/**
 * Map TaskType to ModelPreferences field names
 */
export const TASK_TYPE_TO_FIELD: Record<TaskType, keyof Omit<ModelPreferences, 'userId' | 'updatedAt'>> = {
  chat: 'chat',
  prd_generation: 'prdGeneration',
  prd_analysis: 'prdAnalysis',
  insight_extraction: 'insightExtraction',
  spec_generation: 'specGeneration',
  task_suggestions: 'taskSuggestions',
  task_analysis: 'taskAnalysis',
  spec_refinement: 'specRefinement',
  research_generation: 'researchGeneration',
  markdown_generation: 'markdownGeneration',
};

/**
 * Default model configuration (Anthropic Claude Sonnet 4.5)
 */
export const DEFAULT_MODEL_CONFIG: ModelConfig = {
  provider: 'anthropic',
  model: 'claude-sonnet-4-5-20250929',
};
