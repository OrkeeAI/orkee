// ABOUTME: AI service for generating structured PRD content using Vercel AI SDK
// ABOUTME: Supports both non-streaming (generateObject) and streaming (streamObject) generation with Anthropic Claude

import { createAnthropic } from '@ai-sdk/anthropic';
import { generateObject, streamObject } from 'ai';
import type { ZodSchema } from 'zod';
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
import {
  SYSTEM_PROMPT,
  completePRDPrompt,
  overviewPrompt,
  featuresPrompt,
  uxPrompt,
  technicalPrompt,
  roadmapPrompt,
  dependenciesPrompt,
  risksPrompt,
  researchPrompt,
} from './prompts';

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
  config: AIGenerationConfig,
  prompt: string,
  schema: ZodSchema<T>,
  systemPrompt: string = SYSTEM_PROMPT
): Promise<AIGenerationResult<T>> {
  if (!config.apiKey) {
    throw new AIGenerationError(
      'API key is required. Please add your Anthropic API key in Settings.',
      'NO_API_KEY'
    );
  }

  try {
    // Create Anthropic client for direct API calls (no proxy)
    const anthropic = createAnthropic({
      apiKey: config.apiKey,
      headers: {
        'anthropic-dangerous-direct-browser-access': 'true',
      },
    });

    console.log(`[AI Service] Calling Anthropic API directly`);

    const model = config.model || DEFAULT_MODEL;
    const maxTokens = config.maxTokens || DEFAULT_MAX_TOKENS;
    const temperature = config.temperature ?? DEFAULT_TEMPERATURE;

    console.log(`[AI Service] Generating with model: ${model}`);
    console.log(`[AI Service] Max tokens: ${maxTokens}, Temperature: ${temperature}`);

    const result = await generateObject({
      model: anthropic(model),
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

    return {
      data: result.object,
      usage: {
        inputTokens: result.usage.promptTokens,
        outputTokens: result.usage.completionTokens,
        totalTokens: result.usage.totalTokens,
      },
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
  config: AIGenerationConfig,
  prompt: string,
  schema: ZodSchema<T>,
  systemPrompt: string = SYSTEM_PROMPT
): Promise<AIStreamingResult<T>> {
  if (!config.apiKey) {
    throw new AIGenerationError(
      'API key is required. Please add your Anthropic API key in Settings.',
      'NO_API_KEY'
    );
  }

  try {
    // Create Anthropic client for direct API calls (no proxy)
    const anthropic = createAnthropic({
      apiKey: config.apiKey,
      headers: {
        'anthropic-dangerous-direct-browser-access': 'true',
      },
    });

    console.log(`[AI Service] Calling Anthropic API directly (streaming)`);

    const model = config.model || DEFAULT_MODEL;
    const maxTokens = config.maxTokens || DEFAULT_MAX_TOKENS;
    const temperature = config.temperature ?? DEFAULT_TEMPERATURE;

    console.log(`[AI Service] Generating with model: ${model} (streaming)`);
    console.log(`[AI Service] Max tokens: ${maxTokens}, Temperature: ${temperature}`);

    const result = streamObject({
      model: anthropic(model),
      schema,
      system: systemPrompt,
      prompt,
      maxTokens,
      temperature,
    });

    console.log(`[AI Service] Stream started`);

    // Transform the result to match our interface
    return {
      partialObjectStream: result.partialObjectStream,
      object: result.object,
      usage: result.usage.then((usage) => ({
        inputTokens: usage.promptTokens,
        outputTokens: usage.completionTokens,
        totalTokens: usage.totalTokens,
      })),
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

  constructor(config: AIGenerationConfig) {
    this.config = config;
  }

  /**
   * Generate complete PRD from description
   */
  async generateCompletePRD(description: string): Promise<AIGenerationResult<CompletePRD>> {
    const prompt = completePRDPrompt(description);
    return generateStructured(this.config, prompt, CompletePRDSchema);
  }

  /**
   * Generate overview section
   */
  async generateOverview(description: string): Promise<AIGenerationResult<IdeateOverview>> {
    const prompt = overviewPrompt(description);
    return generateStructured(this.config, prompt, IdeateOverviewSchema);
  }

  /**
   * Generate features section
   */
  async generateFeatures(description: string): Promise<AIGenerationResult<IdeateFeature[]>> {
    const prompt = featuresPrompt(description);
    const result = await generateStructured(this.config, prompt, FeaturesResponseSchema);
    return {
      data: result.data.features,
      usage: result.usage,
    };
  }

  /**
   * Generate UX section
   */
  async generateUX(description: string): Promise<AIGenerationResult<IdeateUX>> {
    const prompt = uxPrompt(description);
    return generateStructured(this.config, prompt, IdeateUXSchema);
  }

  /**
   * Generate technical architecture section
   */
  async generateTechnical(description: string): Promise<AIGenerationResult<IdeateTechnical>> {
    const prompt = technicalPrompt(description);
    return generateStructured(this.config, prompt, IdeateTechnicalSchema);
  }

  /**
   * Generate roadmap section
   */
  async generateRoadmap(
    description: string,
    features: string
  ): Promise<AIGenerationResult<IdeateRoadmap>> {
    const prompt = roadmapPrompt(description, features);
    return generateStructured(this.config, prompt, IdeateRoadmapSchema);
  }

  /**
   * Generate dependencies section
   */
  async generateDependencies(
    description: string,
    features: string
  ): Promise<AIGenerationResult<IdeateDependencies>> {
    const prompt = dependenciesPrompt(description, features);
    return generateStructured(this.config, prompt, IdeateDependenciesSchema);
  }

  /**
   * Generate risks section
   */
  async generateRisks(description: string): Promise<AIGenerationResult<IdeateRisks>> {
    const prompt = risksPrompt(description);
    return generateStructured(this.config, prompt, IdeateRisksSchema);
  }

  /**
   * Generate research section
   */
  async generateResearch(description: string): Promise<AIGenerationResult<IdeateResearch>> {
    const prompt = researchPrompt(description);
    return generateStructured(this.config, prompt, IdeateResearchSchema);
  }

  // Streaming versions of generation methods

  /**
   * Generate complete PRD from description (streaming)
   */
  async generateCompletePRDStreaming(description: string): Promise<AIStreamingResult<CompletePRD>> {
    const prompt = completePRDPrompt(description);
    return generateStreamedStructured(this.config, prompt, CompletePRDSchema);
  }

  /**
   * Generate overview section (streaming)
   */
  async generateOverviewStreaming(description: string): Promise<AIStreamingResult<IdeateOverview>> {
    const prompt = overviewPrompt(description);
    return generateStreamedStructured(this.config, prompt, IdeateOverviewSchema);
  }

  /**
   * Generate features section (streaming)
   */
  async generateFeaturesStreaming(description: string): Promise<AIStreamingResult<IdeateFeature[]>> {
    const prompt = featuresPrompt(description);
    const result = await generateStreamedStructured(this.config, prompt, FeaturesResponseSchema);

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
    };
  }

  /**
   * Generate UX section (streaming)
   */
  async generateUXStreaming(description: string): Promise<AIStreamingResult<IdeateUX>> {
    const prompt = uxPrompt(description);
    return generateStreamedStructured(this.config, prompt, IdeateUXSchema);
  }

  /**
   * Generate technical architecture section (streaming)
   */
  async generateTechnicalStreaming(description: string): Promise<AIStreamingResult<IdeateTechnical>> {
    const prompt = technicalPrompt(description);
    return generateStreamedStructured(this.config, prompt, IdeateTechnicalSchema);
  }

  /**
   * Generate roadmap section (streaming)
   */
  async generateRoadmapStreaming(
    description: string,
    features: string
  ): Promise<AIStreamingResult<IdeateRoadmap>> {
    const prompt = roadmapPrompt(description, features);
    return generateStreamedStructured(this.config, prompt, IdeateRoadmapSchema);
  }

  /**
   * Generate dependencies section (streaming)
   */
  async generateDependenciesStreaming(
    description: string,
    features: string
  ): Promise<AIStreamingResult<IdeateDependencies>> {
    const prompt = dependenciesPrompt(description, features);
    return generateStreamedStructured(this.config, prompt, IdeateDependenciesSchema);
  }

  /**
   * Generate risks section (streaming)
   */
  async generateRisksStreaming(description: string): Promise<AIStreamingResult<IdeateRisks>> {
    const prompt = risksPrompt(description);
    return generateStreamedStructured(this.config, prompt, IdeateRisksSchema);
  }

  /**
   * Generate research section (streaming)
   */
  async generateResearchStreaming(description: string): Promise<AIStreamingResult<IdeateResearch>> {
    const prompt = researchPrompt(description);
    return generateStreamedStructured(this.config, prompt, IdeateResearchSchema);
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
