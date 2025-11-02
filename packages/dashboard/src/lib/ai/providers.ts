// ABOUTME: AI provider initialization and configuration
// ABOUTME: Sets up OpenAI and Anthropic clients with browser-side API calls

import { createOpenAI } from '@ai-sdk/openai';
import { createAnthropic } from '@ai-sdk/anthropic';
import { AI_CONFIG } from './config';

/**
 * Initialize OpenAI provider with API key from environment
 * API key is read from VITE_OPENAI_API_KEY environment variable
 */
export function getOpenAIProvider() {
  const apiKey = import.meta.env.VITE_OPENAI_API_KEY;

  if (!apiKey) {
    throw new Error('VITE_OPENAI_API_KEY environment variable is not set');
  }

  return createOpenAI({
    apiKey,
  });
}

/**
 * Initialize Anthropic provider with API key from environment
 * API key is read from VITE_ANTHROPIC_API_KEY environment variable
 */
export function getAnthropicProvider() {
  const apiKey = import.meta.env.VITE_ANTHROPIC_API_KEY;

  if (!apiKey) {
    throw new Error('VITE_ANTHROPIC_API_KEY environment variable is not set');
  }

  return createAnthropic({
    apiKey,
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
