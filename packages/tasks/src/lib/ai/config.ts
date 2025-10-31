// ABOUTME: AI Gateway and provider configuration
// ABOUTME: Manages Vercel AI SDK settings, model selection, and cost tracking

export const AI_CONFIG = {
  gateway: {
    baseURL: import.meta.env.VITE_VERCEL_AI_GATEWAY_URL || 'https://gateway.vercel.sh',
    apiKey: import.meta.env.VITE_VERCEL_AI_GATEWAY_KEY,
  },

  providers: {
    openai: {
      apiKey: import.meta.env.VITE_OPENAI_API_KEY,
      defaultModel: 'gpt-4-turbo-preview',
      models: {
        fast: 'gpt-3.5-turbo',
        balanced: 'gpt-4-turbo-preview',
        powerful: 'gpt-4',
      },
      maxTokens: {
        fast: 4096,
        balanced: 8192,
        powerful: 8192,
      },
      costPer1kTokens: {
        'gpt-3.5-turbo': { input: 0.0005, output: 0.0015 },
        'gpt-4-turbo-preview': { input: 0.01, output: 0.03 },
        'gpt-4': { input: 0.03, output: 0.06 },
      },
    },

    anthropic: {
      apiKey: import.meta.env.VITE_ANTHROPIC_API_KEY,
      defaultModel: 'claude-3-5-sonnet-20241022',
      models: {
        fast: 'claude-3-haiku-20240307',
        balanced: 'claude-3-5-sonnet-20241022',
        powerful: 'claude-3-opus-20240229',
      },
      maxTokens: {
        fast: 4096,
        balanced: 8192,
        powerful: 8192,
      },
      costPer1kTokens: {
        'claude-3-haiku-20240307': { input: 0.00025, output: 0.00125 },
        'claude-3-5-sonnet-20241022': { input: 0.003, output: 0.015 },
        'claude-3-opus-20240229': { input: 0.015, output: 0.075 },
      },
    },
  },

  operations: {
    analyzePRD: {
      provider: 'anthropic',
      model: 'balanced',
      temperature: 0.3,
      maxRetries: 2,
    },
    generateSpec: {
      provider: 'anthropic',
      model: 'balanced',
      temperature: 0.2,
      maxRetries: 2,
    },
    suggestTasks: {
      provider: 'openai',
      model: 'balanced',
      temperature: 0.4,
      maxRetries: 1,
    },
    validateScenario: {
      provider: 'openai',
      model: 'fast',
      temperature: 0.1,
      maxRetries: 1,
    },
    generateChange: {
      provider: 'anthropic',
      model: 'powerful',
      temperature: 0.3,
      maxRetries: 2,
    },
  },

  streaming: {
    enabled: true,
    chunkSize: 512,
  },

  caching: {
    enabled: true,
    ttl: 3600, // 1 hour in seconds
  },

  rateLimit: {
    requestsPerMinute: 60,
    tokensPerMinute: 90000,
  },
} as const;

export type AIProvider = keyof typeof AI_CONFIG.providers;
export type AIModel = 'fast' | 'balanced' | 'powerful';
export type AIOperation = keyof typeof AI_CONFIG.operations;

// Get provider configuration
export function getProviderConfig(provider: AIProvider) {
  return AI_CONFIG.providers[provider];
}

// Get operation configuration
export function getOperationConfig(operation: AIOperation) {
  return AI_CONFIG.operations[operation];
}

// Calculate estimated cost
export function calculateCost(
  provider: AIProvider,
  model: string,
  inputTokens: number,
  outputTokens: number
): number {
  const costs = AI_CONFIG.providers[provider].costPer1kTokens[model as keyof typeof AI_CONFIG.providers[typeof provider]['costPer1kTokens']];
  if (!costs) return 0;

  const inputCost = (inputTokens / 1000) * costs.input;
  const outputCost = (outputTokens / 1000) * costs.output;

  return inputCost + outputCost;
}

// Validate configuration
export function validateAIConfig(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  // Check if at least one provider is configured
  const hasOpenAI = !!AI_CONFIG.providers.openai.apiKey;
  const hasAnthropic = !!AI_CONFIG.providers.anthropic.apiKey;
  const hasGateway = !!AI_CONFIG.gateway.apiKey;

  if (!hasOpenAI && !hasAnthropic && !hasGateway) {
    errors.push('No AI provider configured. Set VITE_OPENAI_API_KEY, VITE_ANTHROPIC_API_KEY, or VITE_VERCEL_AI_GATEWAY_KEY');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
