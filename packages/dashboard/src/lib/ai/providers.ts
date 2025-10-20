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
  if (!isProviderConfigured('anthropic')) {
    throw new Error('Anthropic API key not configured. Please set VITE_ANTHROPIC_API_KEY.');
  }

  const config = AI_CONFIG.providers.anthropic;

  // Use Vercel AI Gateway if configured
  if (AI_CONFIG.gateway.enabled && AI_CONFIG.gateway.baseURL && AI_CONFIG.gateway.apiKey) {
    return createAnthropic({
      apiKey: config.apiKey,
      baseURL: `${AI_CONFIG.gateway.baseURL}/anthropic/v1`,
      headers: {
        'Helicone-Auth': `Bearer ${AI_CONFIG.gateway.apiKey}`,
      },
    });
  }

  // Direct Anthropic connection
  return createAnthropic({
    apiKey: config.apiKey,
  });
}

/**
 * Get the preferred model instance
 */
export function getPreferredModel() {
  // Try Anthropic first (Claude is generally better for code analysis)
  if (isProviderConfigured('anthropic')) {
    const provider = getAnthropicProvider();
    const config = AI_CONFIG.providers.anthropic;
    return {
      provider: 'anthropic' as const,
      model: provider(config.defaultModel),
      modelName: config.defaultModel,
    };
  }

  // Fall back to OpenAI
  if (isProviderConfigured('openai')) {
    const provider = getOpenAIProvider();
    const config = AI_CONFIG.providers.openai;
    return {
      provider: 'openai' as const,
      model: provider(config.defaultModel),
      modelName: config.defaultModel,
    };
  }

  throw new Error(
    'No AI provider configured. Please set VITE_OPENAI_API_KEY or VITE_ANTHROPIC_API_KEY.'
  );
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
