// ABOUTME: AI provider initialization and configuration
// ABOUTME: Sets up OpenAI and Anthropic clients with optional Vercel AI Gateway

import { createOpenAI } from '@ai-sdk/openai';
import { createAnthropic } from '@ai-sdk/anthropic';
import { AI_CONFIG, isProviderConfigured } from './config';

/**
 * Initialize OpenAI provider
 */
export function getOpenAIProvider() {
  if (!isProviderConfigured('openai')) {
    throw new Error('OpenAI API key not configured. Please set VITE_OPENAI_API_KEY.');
  }

  const config = AI_CONFIG.providers.openai;

  // Use Vercel AI Gateway if configured
  if (AI_CONFIG.gateway.enabled && AI_CONFIG.gateway.baseURL && AI_CONFIG.gateway.apiKey) {
    return createOpenAI({
      apiKey: config.apiKey,
      baseURL: `${AI_CONFIG.gateway.baseURL}/openai/v1`,
      headers: {
        'Helicone-Auth': `Bearer ${AI_CONFIG.gateway.apiKey}`,
      },
    });
  }

  // Direct OpenAI connection
  return createOpenAI({
    apiKey: config.apiKey,
  });
}

/**
 * Initialize Anthropic provider
 */
export function getAnthropicProvider() {
  // Use local proxy endpoint to avoid CORS issues
  // The proxy will forward requests to Anthropic with the API key from server environment
  const apiBaseUrl = window.location.origin.includes('localhost')
    ? 'http://localhost:4001'
    : window.location.origin;

  return createAnthropic({
    apiKey: 'proxy', // Dummy key - actual key is on server
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
 * Check if any provider is configured
 */
export function isAnyProviderConfigured(): boolean {
  return isProviderConfigured('openai') || isProviderConfigured('anthropic');
}

/**
 * Get provider display name
 */
export function getProviderDisplayName(provider: 'openai' | 'anthropic'): string {
  return provider === 'openai' ? 'OpenAI' : 'Anthropic';
}
