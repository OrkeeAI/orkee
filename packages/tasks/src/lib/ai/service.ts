// ABOUTME: AI service layer using Vercel AI SDK for structured OpenSpec generation
// ABOUTME: Handles AI provider calls, streaming, cost tracking, and error handling

import { generateObject, streamObject } from 'ai';
import { openai } from '@ai-sdk/openai';
import { anthropic } from '@ai-sdk/anthropic';
import type { z } from 'zod';
import { AI_CONFIG, type AIProvider, type AIOperation, getOperationConfig, calculateCost } from './config';
import type { AISchemas } from './schemas';

// AI Usage Log Entry
export interface AIUsageLog {
  operation: AIOperation;
  provider: AIProvider;
  model: string;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  cost: number;
  duration: number;
  timestamp: Date;
  error?: string;
}

// AI Service Options
export interface AIServiceOptions {
  provider?: AIProvider;
  model?: 'fast' | 'balanced' | 'powerful';
  temperature?: number;
  maxRetries?: number;
  streaming?: boolean;
  onUsage?: (log: AIUsageLog) => void | Promise<void>;
}

// Get model instance for provider
function getModel(provider: AIProvider, modelTier: 'fast' | 'balanced' | 'powerful') {
  const config = AI_CONFIG.providers[provider];
  const modelName = config.models[modelTier];

  if (provider === 'openai') {
    return openai(modelName, {
      apiKey: config.apiKey,
    });
  } else {
    return anthropic(modelName, {
      apiKey: config.apiKey,
    });
  }
}

// Generate structured object using AI
export async function generateStructured<T extends z.ZodType>(
  operation: AIOperation,
  schema: T,
  prompt: string,
  options: AIServiceOptions = {}
): Promise<{ data: z.infer<T>; usage: AIUsageLog }> {
  const config = getOperationConfig(operation);
  const provider = (options.provider || config.provider) as AIProvider;
  const modelTier = options.model || 'balanced';
  const temperature = options.temperature ?? config.temperature;
  const maxRetries = options.maxRetries ?? config.maxRetries;

  const model = getModel(provider, modelTier);
  const startTime = Date.now();

  try {
    const result = await generateObject({
      model,
      schema,
      prompt,
      temperature,
      maxRetries,
    });

    const duration = Date.now() - startTime;
    const modelName = AI_CONFIG.providers[provider].models[modelTier];
    const inputTokens = result.usage?.promptTokens || 0;
    const outputTokens = result.usage?.completionTokens || 0;
    const totalTokens = inputTokens + outputTokens;
    const cost = calculateCost(provider, modelName, inputTokens, outputTokens);

    const usageLog: AIUsageLog = {
      operation,
      provider,
      model: modelName,
      inputTokens,
      outputTokens,
      totalTokens,
      cost,
      duration,
      timestamp: new Date(),
    };

    // Call usage callback if provided
    if (options.onUsage) {
      await options.onUsage(usageLog);
    }

    return {
      data: result.object,
      usage: usageLog,
    };
  } catch (error) {
    const duration = Date.now() - startTime;
    const modelName = AI_CONFIG.providers[provider].models[modelTier];

    const usageLog: AIUsageLog = {
      operation,
      provider,
      model: modelName,
      inputTokens: 0,
      outputTokens: 0,
      totalTokens: 0,
      cost: 0,
      duration,
      timestamp: new Date(),
      error: error instanceof Error ? error.message : String(error),
    };

    if (options.onUsage) {
      await options.onUsage(usageLog);
    }

    throw error;
  }
}

// Stream structured object using AI
export async function streamStructured<T extends z.ZodType>(
  operation: AIOperation,
  schema: T,
  prompt: string,
  options: AIServiceOptions = {}
) {
  const config = getOperationConfig(operation);
  const provider = (options.provider || config.provider) as AIProvider;
  const modelTier = options.model || 'balanced';
  const temperature = options.temperature ?? config.temperature;
  const maxRetries = options.maxRetries ?? config.maxRetries;

  const model = getModel(provider, modelTier);
  const startTime = Date.now();

  try {
    const result = await streamObject({
      model,
      schema,
      prompt,
      temperature,
      maxRetries,
    });

    return {
      partialObjectStream: result.partialObjectStream,
      object: result.object,
      usage: async () => {
        const finalObject = await result.object;
        const duration = Date.now() - startTime;
        const modelName = AI_CONFIG.providers[provider].models[modelTier];
        const inputTokens = result.usage?.promptTokens || 0;
        const outputTokens = result.usage?.completionTokens || 0;
        const totalTokens = inputTokens + outputTokens;
        const cost = calculateCost(provider, modelName, inputTokens, outputTokens);

        const usageLog: AIUsageLog = {
          operation,
          provider,
          model: modelName,
          inputTokens,
          outputTokens,
          totalTokens,
          cost,
          duration,
          timestamp: new Date(),
        };

        if (options.onUsage) {
          await options.onUsage(usageLog);
        }

        return usageLog;
      },
    };
  } catch (error) {
    const duration = Date.now() - startTime;
    const modelName = AI_CONFIG.providers[provider].models[modelTier];

    const usageLog: AIUsageLog = {
      operation,
      provider,
      model: modelName,
      inputTokens: 0,
      outputTokens: 0,
      totalTokens: 0,
      cost: 0,
      duration,
      timestamp: new Date(),
      error: error instanceof Error ? error.message : String(error),
    };

    if (options.onUsage) {
      await options.onUsage(usageLog);
    }

    throw error;
  }
}

// High-level AI operations using pre-defined schemas

export const AIOperations = {
  async analyzePRD(prdContent: string, options?: AIServiceOptions) {
    const { AISchemas } = await import('./schemas');
    return generateStructured(
      'analyzePRD',
      AISchemas.PRDAnalysis,
      `Analyze the following Product Requirements Document and extract key information:\n\n${prdContent}`,
      options
    );
  },

  async generateSpec(prdAnalysis: string, options?: AIServiceOptions) {
    const { AISchemas } = await import('./schemas');
    return generateStructured(
      'generateSpec',
      AISchemas.SpecGeneration,
      `Based on this PRD analysis, generate a detailed specification with capabilities, requirements, and WHEN/THEN/AND scenarios:\n\n${prdAnalysis}`,
      options
    );
  },

  async suggestTasks(capabilityName: string, requirements: string, options?: AIServiceOptions) {
    const { AISchemas } = await import('./schemas');
    return generateStructured(
      'suggestTasks',
      AISchemas.TaskSuggestion,
      `For the capability "${capabilityName}" with these requirements:\n\n${requirements}\n\nGenerate a list of implementation tasks with estimates and priorities.`,
      options
    );
  },

  async validateScenarios(capability: string, requirements: string, scenarios: string, options?: AIServiceOptions) {
    const { AISchemas } = await import('./schemas');
    return generateStructured(
      'validateScenario',
      AISchemas.ScenarioValidation,
      `Validate the test scenario coverage for capability "${capability}":\n\nRequirements:\n${requirements}\n\nScenarios:\n${scenarios}\n\nIdentify gaps and suggest additional scenarios.`,
      options
    );
  },

  async generateChange(changeDescription: string, currentSpecs: string, options?: AIServiceOptions) {
    const { AISchemas } = await import('./schemas');
    return generateStructured(
      'generateChange',
      AISchemas.ChangeProposal,
      `Generate a change proposal for:\n\n${changeDescription}\n\nCurrent specifications:\n${currentSpecs}\n\nInclude deltas (ADDED/MODIFIED/REMOVED) and implementation tasks.`,
      options
    );
  },

  async extractFromPRD(prdContent: string, options?: AIServiceOptions) {
    const { AISchemas } = await import('./schemas');
    return generateStructured(
      'analyzePRD',
      AISchemas.PRDExtraction,
      `Extract capabilities, requirements, assumptions, and constraints from this PRD:\n\n${prdContent}`,
      options
    );
  },

  async analyzeSyncNeeds(direction: 'prd-to-spec' | 'spec-to-prd' | 'task-to-spec', sourceContent: string, targetContent: string, options?: AIServiceOptions) {
    const { AISchemas } = await import('./schemas');
    return generateStructured(
      'analyzePRD',
      AISchemas.SyncAnalysis,
      `Analyze synchronization needs from ${direction}:\n\nSource:\n${sourceContent}\n\nTarget:\n${targetContent}\n\nIdentify changes needed and potential conflicts.`,
      options
    );
  },
};

// Usage example export for documentation
export const USAGE_EXAMPLE = `
// Example: Analyze a PRD
const { data, usage } = await AIOperations.analyzePRD(prdContent, {
  onUsage: async (log) => {
    // Save to database
    await saveUsageLog(log);
  }
});

// Example: Stream spec generation
const stream = await streamStructured(
  'generateSpec',
  AISchemas.SpecGeneration,
  prompt,
  {
    streaming: true,
    onUsage: async (log) => console.log('Cost:', log.cost)
  }
);

for await (const partial of stream.partialObjectStream) {
  console.log('Partial:', partial);
}
`;
