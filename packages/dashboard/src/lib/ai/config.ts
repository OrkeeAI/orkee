// ABOUTME: AI configuration for OpenAI, Anthropic, and Vercel AI Gateway
// ABOUTME: Provides centralized settings for models, tokens, and API endpoints

export interface AIProviderConfig {
  apiKey?: string;
  defaultModel: string;
  models: {
    [key: string]: {
      maxTokens: number;
      contextWindow: number;
      costPer1kInput: number;
      costPer1kOutput: number;
    };
  };
}

export interface AIGatewayConfig {
  enabled: boolean;
  baseURL?: string;
  apiKey?: string;
}

export interface AIConfig {
  gateway: AIGatewayConfig;
  providers: {
    openai: AIProviderConfig;
    anthropic: AIProviderConfig;
  };
  defaults: {
    maxTokens: number;
    temperature: number;
    topP: number;
  };
  features: {
    streaming: boolean;
    caching: boolean;
    rateLimiting: boolean;
  };
  rateLimits: {
    requestsPerMinute: number;
    tokensPerMinute: number;
  };
  sizeLimits: {
    maxPRDTokens: number;
    chunkSize: number;
    promptOverhead: number;
    timeoutMs: number;
  };
}

export const AI_CONFIG: AIConfig = {
  gateway: {
    enabled: !!import.meta.env.VITE_VERCEL_AI_GATEWAY_URL,
    baseURL: import.meta.env.VITE_VERCEL_AI_GATEWAY_URL || 'https://gateway.vercel.sh',
    apiKey: import.meta.env.VITE_VERCEL_AI_GATEWAY_KEY,
  },
  providers: {
    openai: {
      apiKey: import.meta.env.VITE_OPENAI_API_KEY,
      defaultModel: 'gpt-4-turbo',
      models: {
        'gpt-4-turbo': {
          maxTokens: 4096,
          contextWindow: 128000,
          costPer1kInput: 0.01,
          costPer1kOutput: 0.03,
        },
        'gpt-4': {
          maxTokens: 4096,
          contextWindow: 8192,
          costPer1kInput: 0.03,
          costPer1kOutput: 0.06,
        },
        'gpt-3.5-turbo': {
          maxTokens: 4096,
          contextWindow: 16385,
          costPer1kInput: 0.0005,
          costPer1kOutput: 0.0015,
        },
      },
    },
    anthropic: {
      apiKey: import.meta.env.VITE_ANTHROPIC_API_KEY,
      defaultModel: 'claude-3-5-sonnet-20241022',
      models: {
        'claude-3-5-sonnet-20241022': {
          maxTokens: 8192,
          contextWindow: 200000,
          costPer1kInput: 0.003,
          costPer1kOutput: 0.015,
        },
        'claude-3-opus-20240229': {
          maxTokens: 4096,
          contextWindow: 200000,
          costPer1kInput: 0.015,
          costPer1kOutput: 0.075,
        },
        'claude-3-sonnet-20240229': {
          maxTokens: 4096,
          contextWindow: 200000,
          costPer1kInput: 0.003,
          costPer1kOutput: 0.015,
        },
        'claude-3-haiku-20240307': {
          maxTokens: 4096,
          contextWindow: 200000,
          costPer1kInput: 0.00025,
          costPer1kOutput: 0.00125,
        },
      },
    },
  },
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
    maxPRDTokens: 100000, // ~400KB text - safe for both providers with output buffer
    chunkSize: 30000, // ~120KB per chunk
    promptOverhead: 500, // Estimated prompt template tokens
    timeoutMs: 120000, // 2 minutes timeout for AI calls
  },
};

/**
 * Get the preferred provider based on available API keys
 */
export function getPreferredProvider(): 'openai' | 'anthropic' | null {
  if (import.meta.env.VITE_ANTHROPIC_API_KEY) {
    return 'anthropic';
  }
  if (import.meta.env.VITE_OPENAI_API_KEY) {
    return 'openai';
  }
  return null;
}

/**
 * Get the configuration for a specific provider
 */
export function getProviderConfig(
  provider: 'openai' | 'anthropic'
): AIProviderConfig {
  return AI_CONFIG.providers[provider];
}

/**
 * Calculate estimated cost for a given token usage
 */
export function calculateCost(
  provider: 'openai' | 'anthropic',
  model: string,
  inputTokens: number,
  outputTokens: number
): number {
  const config = AI_CONFIG.providers[provider];
  const modelConfig = config.models[model];

  if (!modelConfig) {
    console.warn(`Model ${model} not found in configuration for ${provider}`);
    return 0;
  }

  const inputCost = (inputTokens / 1000) * modelConfig.costPer1kInput;
  const outputCost = (outputTokens / 1000) * modelConfig.costPer1kOutput;

  return inputCost + outputCost;
}

/**
 * Check if a provider is configured
 */
export function isProviderConfigured(provider: 'openai' | 'anthropic'): boolean {
  const config = AI_CONFIG.providers[provider];
  return !!config.apiKey;
}

/**
 * Get all configured providers
 */
export function getConfiguredProviders(): Array<'openai' | 'anthropic'> {
  const providers: Array<'openai' | 'anthropic'> = [];

  if (isProviderConfigured('openai')) {
    providers.push('openai');
  }
  if (isProviderConfigured('anthropic')) {
    providers.push('anthropic');
  }

  return providers;
}
