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
    baseURL: `${apiBaseUrl}/api/ai/anthropic`,
  });
}

/**
 * Get the preferred model instance
 */
export function getPreferredModel() {
  // Use Anthropic via proxy (always available)
  const provider = getAnthropicProvider();
  const config = AI_CONFIG.providers.anthropic;
  return {
    provider: 'anthropic' as const,
    model: provider(config.defaultModel),
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
    return openai(modelName || config.defaultModel);
  }

  if (provider === 'anthropic') {
    const anthropic = getAnthropicProvider();
    const config = AI_CONFIG.providers.anthropic;
    return anthropic(modelName || config.defaultModel);
  }

  throw new Error(`Unknown provider: ${provider}`);
}

/**
 * Get available models for a provider
 */
export function getAvailableModels(provider: 'openai' | 'anthropic'): string[] {
  const config = AI_CONFIG.providers[provider];
  return Object.keys(config.models);
}

/**
 * Get provider display name
 */
export function getProviderDisplayName(provider: 'openai' | 'anthropic'): string {
  return provider === 'openai' ? 'OpenAI' : 'Anthropic';
}
