// ABOUTME: AI provider initialization and configuration
// ABOUTME: Sets up OpenAI and Anthropic clients using secure proxy endpoints

import { createOpenAI } from '@ai-sdk/openai';
import { createAnthropic } from '@ai-sdk/anthropic';
import { AI_CONFIG } from './config';

/**
 * Get API base URL for proxy endpoints
 */
function getApiBaseUrl(): string {
  return window.location.origin.includes('localhost')
    ? 'http://localhost:4001'
    : window.location.origin;
}

/**
 * Initialize OpenAI provider via secure proxy
 * API keys are stored in database and retrieved server-side
 */
export function getOpenAIProvider() {
  const apiBaseUrl = getApiBaseUrl();

  return createOpenAI({
    apiKey: 'proxy', // Dummy key - actual key is retrieved from database on server
    baseURL: `${apiBaseUrl}/api/ai/openai/v1`,
  });
}

/**
 * Initialize Anthropic provider via secure proxy
 * API keys are stored in database and retrieved server-side
 */
export function getAnthropicProvider() {
  const apiBaseUrl = getApiBaseUrl();

  return createAnthropic({
    apiKey: 'proxy', // Dummy key - actual key is retrieved from database on server
    baseURL: `${apiBaseUrl}/api/ai/anthropic/v1`,
  });
}

/**
 * Get the preferred model instance (defaults to Anthropic Sonnet 4.5)
 */
export function getPreferredModel() {
  const config = AI_CONFIG.providers.anthropic;
  return {
    provider: 'anthropic' as const,
    model: getModel('anthropic', config.defaultModel),
    modelName: config.defaultModel,
  };
}

/**
 * Get a specific model instance by provider and model name
 */
export function getModel(provider: 'openai' | 'anthropic', modelName?: string) {
  if (provider === 'openai') {
    const openai = getOpenAIProvider();
    const config = AI_CONFIG.providers.openai;
    const selectedModel = modelName || config.defaultModel;
    console.log(`[providers] Creating OpenAI model instance:`, selectedModel);
    return openai(selectedModel);
  }

  if (provider === 'anthropic') {
    const anthropic = getAnthropicProvider();
    const config = AI_CONFIG.providers.anthropic;
    const selectedModel = modelName || config.defaultModel;
    console.log(`[providers] Creating Anthropic model instance:`, selectedModel);
    return anthropic(selectedModel);
  }

  throw new Error(`Unknown provider: ${provider}`);
}

/**
 * Get model with full info (provider, model instance, model name)
 */
export function getModelWithInfo(provider: 'openai' | 'anthropic', modelName?: string) {
  const config = AI_CONFIG.providers[provider];
  const selectedModel = modelName || config.defaultModel;

  return {
    provider,
    model: getModel(provider, selectedModel),
    modelName: selectedModel,
  };
}

/**
 * Get available models for a provider
 */
export function getAvailableModels(provider: 'openai' | 'anthropic'): string[] {
  const config = AI_CONFIG.providers[provider];
  return Object.keys(config.models);
}

/**
 * Get available models with display names for a provider
 */
export function getAvailableModelsWithNames(provider: 'openai' | 'anthropic'): Array<{ id: string; name: string }> {
  const config = AI_CONFIG.providers[provider];
  return Object.entries(config.models).map(([id, model]) => ({
    id,
    name: model.displayName,
  }));
}

/**
 * Get all available providers
 */
export function getAvailableProviders(): Array<{ id: 'openai' | 'anthropic'; name: string }> {
  return [
    { id: 'openai', name: AI_CONFIG.providers.openai.displayName },
    { id: 'anthropic', name: AI_CONFIG.providers.anthropic.displayName },
  ];
}

/**
 * Get provider display name
 */
export function getProviderDisplayName(provider: 'openai' | 'anthropic'): string {
  return AI_CONFIG.providers[provider].displayName;
}

/**
 * Get model display name
 */
export function getModelDisplayName(provider: 'openai' | 'anthropic', modelId: string): string {
  const config = AI_CONFIG.providers[provider];
  return config.models[modelId]?.displayName || modelId;
}
