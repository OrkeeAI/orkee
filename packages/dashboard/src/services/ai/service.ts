// ABOUTME: AI service for generating structured PRD content using Vercel AI SDK
// ABOUTME: Supports both non-streaming (generateObject) and streaming (streamObject) generation with Anthropic Claude

import { createAnthropic } from '@ai-sdk/anthropic';
import { generateObject, streamObject } from 'ai';
import type { ZodSchema } from 'zod';
import { PromptManager } from '@orkee/prompts';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import { sendAIResultTelemetry, detectProvider } from '@/lib/ai/telemetry';
import type { ModelConfig } from '@/types/models';
import {
  CompletePRDSchema,
  IdeateOverviewSchema,
  FeaturesResponseSchema,
  IdeateUXSchema,
  IdeateTechnicalSchema,
  IdeateRoadmapSchema,
  IdeateDependenciesSchema,
  IdeateRisksSchema,
  IdeateResearchSchema,
  type CompletePRD,
  type IdeateOverview,
  type IdeateFeature,
  type IdeateUX,
  type IdeateTechnical,
  type IdeateRoadmap,
  type IdeateDependencies,
  type IdeateRisks,
  type IdeateResearch,
} from './schemas';

/**
 * Configuration for AI generation
 */
export interface AIGenerationConfig {
  apiKey: string;
  model?: string;
  maxTokens?: number;
  temperature?: number;
}

/**
 * Result of AI generation with usage stats
 */
export interface AIGenerationResult<T> {
  data: T;
  usage: {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
  };
  model: string;
  provider: string;
  estimatedCost: number;
}

/**
 * Default model settings
 */
const DEFAULT_MODEL = 'claude-sonnet-4-20250514'; // Claude Sonnet 4
const DEFAULT_MAX_TOKENS = 64000;
const DEFAULT_TEMPERATURE = 0.7;

/**
 * Error types for AI generation
 */
export class AIGenerationError extends Error {
  constructor(
    message: string,
    public readonly code: 'API_ERROR' | 'PARSE_ERROR' | 'VALIDATION_ERROR' | 'NO_API_KEY',
    public readonly cause?: unknown
  ) {
    super(message);
    this.name = 'AIGenerationError';
  }
}

/**
 * Result of streaming AI generation
 */
export interface AIStreamingResult<T> {
  partialObjectStream: AsyncIterable<DeepPartial<T>>;
  object: Promise<T>;
  usage: Promise<{
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
  }>;
  model: string;
  provider: string;
  estimatedCost: Promise<number>;
}

/**
 * Deep partial type for streaming results
 */
type DeepPartial<T> = T extends object
  ? {
      [P in keyof T]?: DeepPartial<T[P]>;
    }
  : T;

/**
 * Core generation function using Vercel AI SDK with direct Anthropic API calls (non-streaming)
 */
async function generateStructured<T>(
  operation: string,
  config: AIGenerationConfig,
  prompt: string,
  schema: ZodSchema<T>,
  systemPrompt?: string,
  modelPreferences?: ModelConfig
): Promise<AIGenerationResult<T>> {
  const startTime = performance.now();
  // Load system prompt if not provided
  if (!systemPrompt) {
    const promptManager = new PromptManager();
    systemPrompt = await promptManager.getSystemPrompt('prd');
  }
  if (!config.apiKey) {
    throw new AIGenerationError(
      'API key is required. Please add your Anthropic API key in Settings.',
      'NO_API_KEY'
    );
  }

  try {
    // Determine model to use: preferences > config > default
    let modelInstance;
    let modelName: string;
    let providerName: string;

    if (modelPreferences) {
      console.log(`[AI Service] Using model preferences: ${modelPreferences.provider}/${modelPreferences.model}`);
      modelInstance = getModelInstance(modelPreferences.provider, modelPreferences.model);
      modelName = modelPreferences.model;
      providerName = modelPreferences.provider;
    } else {
      // Create Anthropic client for direct API calls (no proxy)
      const anthropic = createAnthropic({
        apiKey: config.apiKey,
        headers: {
          'anthropic-dangerous-direct-browser-access': 'true',
        },
      });

      console.log(`[AI Service] Calling Anthropic API directly`);

      modelName = config.model || DEFAULT_MODEL;
      modelInstance = anthropic(modelName);
      providerName = detectProvider(modelName);
    }

    const maxTokens = config.maxTokens || DEFAULT_MAX_TOKENS;
    const temperature = config.temperature ?? DEFAULT_TEMPERATURE;

    console.log(`[AI Service] Generating with model: ${modelName}`);
    console.log(`[AI Service] Max tokens: ${maxTokens}, Temperature: ${temperature}`);

    const result = await generateObject({
      model: modelInstance,
      schema,
      system: systemPrompt,
      prompt,
      maxTokens,
      temperature,
    });

    console.log(`[AI Service] Generation complete`);
    console.log(`[AI Service] Input tokens: ${result.usage.promptTokens}`);
    console.log(`[AI Service] Output tokens: ${result.usage.completionTokens}`);
    console.log(`[AI Service] Total tokens: ${result.usage.totalTokens}`);

    const usage = {
      inputTokens: result.usage.promptTokens,
      outputTokens: result.usage.completionTokens,
      totalTokens: result.usage.totalTokens,
    };

    const estimatedCost = calculateCost(providerName, modelName, usage.inputTokens, usage.outputTokens);
    const durationMs = Math.round(performance.now() - startTime);

    // Send telemetry
    await sendAIResultTelemetry(
      operation,
      null, // PRD generation doesn't have project ID
      modelName,
      providerName,
      usage,
      estimatedCost,
      durationMs
    );

    return {
      data: result.object,
      usage,
      model: modelName,
      provider: providerName,
      estimatedCost,
    };
  } catch (error) {
    console.error('[AI Service] Generation failed:', error);

    if (error instanceof Error) {
      if (error.message.includes('API key')) {
        throw new AIGenerationError(
          'Invalid API key. Please check your Anthropic API key in Settings.',
          'API_ERROR',
          error
        );
      }
      if (error.message.includes('timeout')) {
        throw new AIGenerationError(
          'Generation timed out. Please try again or use a faster model.',
          'API_ERROR',
          error
        );
      }
      throw new AIGenerationError(
        `Generation failed: ${error.message}`,
        'API_ERROR',
        error
      );
    }

    throw new AIGenerationError(
      'An unexpected error occurred during generation',
      'API_ERROR',
      error
    );
  }
}

/**
 * Core streaming generation function using Vercel AI SDK with direct Anthropic API calls
 */
async function generateStreamedStructured<T>(
  operation: string,
  config: AIGenerationConfig,
  prompt: string,
  schema: ZodSchema<T>,
  systemPrompt?: string,
  modelPreferences?: ModelConfig
): Promise<AIStreamingResult<T>> {
  const startTime = performance.now();
  // Load system prompt if not provided
  if (!systemPrompt) {
    const promptManager = new PromptManager();
    systemPrompt = await promptManager.getSystemPrompt('prd');
  }
  if (!config.apiKey) {
    throw new AIGenerationError(
      'API key is required. Please add your Anthropic API key in Settings.',
      'NO_API_KEY'
    );
  }

  try {
    // Determine model to use: preferences > config > default
    let modelInstance;
    let modelName: string;
    let providerName: string;

    if (modelPreferences) {
      console.log(`[AI Service] Using model preferences (streaming): ${modelPreferences.provider}/${modelPreferences.model}`);
      modelInstance = getModelInstance(modelPreferences.provider, modelPreferences.model);
      modelName = modelPreferences.model;
      providerName = modelPreferences.provider;
    } else {
      // Create Anthropic client for direct API calls (no proxy)
      const anthropic = createAnthropic({
        apiKey: config.apiKey,
        headers: {
          'anthropic-dangerous-direct-browser-access': 'true',
        },
      });

      console.log(`[AI Service] Calling Anthropic API directly (streaming)`);

      modelName = config.model || DEFAULT_MODEL;
      modelInstance = anthropic(modelName);
      providerName = detectProvider(modelName);
    }

    const maxTokens = config.maxTokens || DEFAULT_MAX_TOKENS;
    const temperature = config.temperature ?? DEFAULT_TEMPERATURE;

    console.log(`[AI Service] Generating with model: ${modelName} (streaming)`);
    console.log(`[AI Service] Max tokens: ${maxTokens}, Temperature: ${temperature}`);

    const result = streamObject({
      model: modelInstance,
      schema,
      system: systemPrompt,
      prompt,
      maxTokens,
      temperature,
    });

    console.log(`[AI Service] Stream started`);

    // Transform the result to match our interface with telemetry
    const usage = result.usage.then((usage) => ({
      inputTokens: usage.promptTokens,
      outputTokens: usage.completionTokens,
      totalTokens: usage.totalTokens,
    }));

    // Calculate cost and send telemetry when stream completes
    const estimatedCost = usage.then(async (usageData) => {
      const cost = calculateCost(providerName, modelName, usageData.inputTokens, usageData.outputTokens);
      const durationMs = Math.round(performance.now() - startTime);

      // Send telemetry
      await sendAIResultTelemetry(
        operation,
        null, // PRD generation doesn't have project ID
        modelName,
        providerName,
        usageData,
        cost,
        durationMs
      );

      return cost;
    });

    return {
      partialObjectStream: result.partialObjectStream,
      object: result.object,
      usage,
      model: modelName,
      provider: providerName,
      estimatedCost,
    };
  } catch (error) {
    console.error('[AI Service] Streaming generation failed:', error);

    if (error instanceof Error) {
      if (error.message.includes('API key')) {
        throw new AIGenerationError(
          'Invalid API key. Please check your Anthropic API key in Settings.',
          'API_ERROR',
          error
        );
      }
      if (error.message.includes('timeout')) {
        throw new AIGenerationError(
          'Generation timed out. Please try again or use a faster model.',
          'API_ERROR',
          error
        );
      }
      throw new AIGenerationError(
        `Streaming generation failed: ${error.message}`,
        'API_ERROR',
        error
      );
    }

    throw new AIGenerationError(
      'An unexpected error occurred during streaming generation',
      'API_ERROR',
      error
    );
  }
}

/**
 * AI Service for PRD generation
 */
export class AIService {
  private config: AIGenerationConfig;
  private promptManager: PromptManager;

  constructor(config: AIGenerationConfig) {
    this.config = config;
    this.promptManager = new PromptManager();
  }

  /**
   * Generate complete PRD from description
   */
  async generateCompletePRD(description: string, modelPreferences?: ModelConfig): Promise<AIGenerationResult<CompletePRD>> {
    const prompt = await this.promptManager.getPrompt('complete', { description });
    return generateStructured('prd_generate_complete', this.config, prompt, CompletePRDSchema, undefined, modelPreferences);
  }

  /**
   * Generate overview section
   */
  async generateOverview(description: string, modelPreferences?: ModelConfig): Promise<AIGenerationResult<IdeateOverview>> {
    const prompt = await this.promptManager.getPrompt('overview', { description });
    return generateStructured('prd_generate_overview', this.config, prompt, IdeateOverviewSchema, undefined, modelPreferences);
  }

  /**
   * Generate features section
   */
  async generateFeatures(description: string, modelPreferences?: ModelConfig): Promise<AIGenerationResult<IdeateFeature[]>> {
    const prompt = await this.promptManager.getPrompt('features', { description });
    const result = await generateStructured('prd_generate_features', this.config, prompt, FeaturesResponseSchema, undefined, modelPreferences);
    return {
      data: result.data.features,
      usage: result.usage,
      model: result.model,
      provider: result.provider,
      estimatedCost: result.estimatedCost,
    };
  }

  /**
   * Generate UX section
   */
  async generateUX(description: string, modelPreferences?: ModelConfig): Promise<AIGenerationResult<IdeateUX>> {
    const prompt = await this.promptManager.getPrompt('ux', { description });
    return generateStructured('prd_generate_ux', this.config, prompt, IdeateUXSchema, undefined, modelPreferences);
  }

  /**
   * Generate technical architecture section
   */
  async generateTechnical(description: string, modelPreferences?: ModelConfig): Promise<AIGenerationResult<IdeateTechnical>> {
    const prompt = await this.promptManager.getPrompt('technical', { description });
    return generateStructured('prd_generate_technical', this.config, prompt, IdeateTechnicalSchema, undefined, modelPreferences);
  }

  /**
   * Generate roadmap section
   */
  async generateRoadmap(
    description: string,
    features: string,
    modelPreferences?: ModelConfig
  ): Promise<AIGenerationResult<IdeateRoadmap>> {
    const prompt = await this.promptManager.getPrompt('roadmap', { description, features });
    return generateStructured('prd_generate_roadmap', this.config, prompt, IdeateRoadmapSchema, undefined, modelPreferences);
  }

  /**
   * Generate dependencies section
   */
  async generateDependencies(
    description: string,
    features: string,
    modelPreferences?: ModelConfig
  ): Promise<AIGenerationResult<IdeateDependencies>> {
    const prompt = await this.promptManager.getPrompt('dependencies', { description, features });
    return generateStructured('prd_generate_dependencies', this.config, prompt, IdeateDependenciesSchema, undefined, modelPreferences);
  }

  /**
   * Generate risks section
   */
  async generateRisks(description: string, modelPreferences?: ModelConfig): Promise<AIGenerationResult<IdeateRisks>> {
    const prompt = await this.promptManager.getPrompt('risks', { description });
    return generateStructured('prd_generate_risks', this.config, prompt, IdeateRisksSchema, undefined, modelPreferences);
  }

  /**
   * Generate research section
   */
  async generateResearch(description: string, modelPreferences?: ModelConfig): Promise<AIGenerationResult<IdeateResearch>> {
    const prompt = await this.promptManager.getPrompt('research', { description });
    return generateStructured('prd_generate_research', this.config, prompt, IdeateResearchSchema, undefined, modelPreferences);
  }

  // Streaming versions of generation methods

  /**
   * Generate complete PRD from description (streaming)
   */
  async generateCompletePRDStreaming(description: string, modelPreferences?: ModelConfig): Promise<AIStreamingResult<CompletePRD>> {
    const prompt = await this.promptManager.getPrompt('complete', { description });
    return generateStreamedStructured('prd_generate_complete_stream', this.config, prompt, CompletePRDSchema, undefined, modelPreferences);
  }

  /**
   * Generate overview section (streaming)
   */
  async generateOverviewStreaming(description: string, modelPreferences?: ModelConfig): Promise<AIStreamingResult<IdeateOverview>> {
    const prompt = await this.promptManager.getPrompt('overview', { description });
    return generateStreamedStructured('prd_generate_overview_stream', this.config, prompt, IdeateOverviewSchema, undefined, modelPreferences);
  }

  /**
   * Generate features section (streaming)
   */
  async generateFeaturesStreaming(description: string, modelPreferences?: ModelConfig): Promise<AIStreamingResult<IdeateFeature[]>> {
    const prompt = await this.promptManager.getPrompt('features', { description });
    const result = await generateStreamedStructured('prd_generate_features_stream', this.config, prompt, FeaturesResponseSchema, undefined, modelPreferences);

    // Transform the stream to extract just the features array
    return {
      partialObjectStream: (async function* () {
        for await (const partial of result.partialObjectStream) {
          if (partial && 'features' in partial && partial.features) {
            yield partial.features;
          }
        }
      })(),
      object: result.object.then((obj) => obj.features),
      usage: result.usage,
      model: result.model,
      provider: result.provider,
      estimatedCost: result.estimatedCost,
    };
  }

  /**
   * Generate UX section (streaming)
   */
  async generateUXStreaming(description: string, modelPreferences?: ModelConfig): Promise<AIStreamingResult<IdeateUX>> {
    const prompt = await this.promptManager.getPrompt('ux', { description });
    return generateStreamedStructured('prd_generate_ux_stream', this.config, prompt, IdeateUXSchema, undefined, modelPreferences);
  }

  /**
   * Generate technical architecture section (streaming)
   */
  async generateTechnicalStreaming(description: string, modelPreferences?: ModelConfig): Promise<AIStreamingResult<IdeateTechnical>> {
    const prompt = await this.promptManager.getPrompt('technical', { description });
    return generateStreamedStructured('prd_generate_technical_stream', this.config, prompt, IdeateTechnicalSchema, undefined, modelPreferences);
  }

  /**
   * Generate roadmap section (streaming)
   */
  async generateRoadmapStreaming(
    description: string,
    features: string,
    modelPreferences?: ModelConfig
  ): Promise<AIStreamingResult<IdeateRoadmap>> {
    const prompt = await this.promptManager.getPrompt('roadmap', { description, features });
    return generateStreamedStructured('prd_generate_roadmap_stream', this.config, prompt, IdeateRoadmapSchema, undefined, modelPreferences);
  }

  /**
   * Generate dependencies section (streaming)
   */
  async generateDependenciesStreaming(
    description: string,
    features: string,
    modelPreferences?: ModelConfig
  ): Promise<AIStreamingResult<IdeateDependencies>> {
    const prompt = await this.promptManager.getPrompt('dependencies', { description, features });
    return generateStreamedStructured('prd_generate_dependencies_stream', this.config, prompt, IdeateDependenciesSchema, undefined, modelPreferences);
  }

  /**
   * Generate risks section (streaming)
   */
  async generateRisksStreaming(description: string, modelPreferences?: ModelConfig): Promise<AIStreamingResult<IdeateRisks>> {
    const prompt = await this.promptManager.getPrompt('risks', { description });
    return generateStreamedStructured('prd_generate_risks_stream', this.config, prompt, IdeateRisksSchema, undefined, modelPreferences);
  }

  /**
   * Generate research section (streaming)
   */
  async generateResearchStreaming(description: string, modelPreferences?: ModelConfig): Promise<AIStreamingResult<IdeateResearch>> {
    const prompt = await this.promptManager.getPrompt('research', { description });
    return generateStreamedStructured('prd_generate_research_stream', this.config, prompt, IdeateResearchSchema, undefined, modelPreferences);
  }

  /**
   * Update configuration (useful for changing models mid-session)
   */
  updateConfig(updates: Partial<AIGenerationConfig>): void {
    this.config = { ...this.config, ...updates };
  }

  /**
   * Get current model being used
   */
  getModel(): string {
    return this.config.model || DEFAULT_MODEL;
  }
}

/**
 * Create a new AI service instance
 */
export function createAIService(config: AIGenerationConfig): AIService {
  return new AIService(config);
}
