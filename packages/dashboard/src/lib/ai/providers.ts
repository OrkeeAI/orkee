// ABOUTME: AI provider initialization and configuration
// ABOUTME: Sets up OpenAI and Anthropic clients using secure proxy endpoints

import { createOpenAI } from '@ai-sdk/openai';
import { createAnthropic } from '@ai-sdk/anthropic';
import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { createXai } from '@ai-sdk/xai';
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
 * Initialize Google provider via secure proxy
 * API keys are stored in database and retrieved server-side
 */
export function getGoogleProvider() {
  const apiBaseUrl = getApiBaseUrl();

  return createGoogleGenerativeAI({
    apiKey: 'proxy', // Dummy key - actual key is retrieved from database on server
    baseURL: `${apiBaseUrl}/api/ai/google/v1`,
  });
}

/**
 * Initialize xAI provider via secure proxy
 * API keys are stored in database and retrieved server-side
 */
export function getXAIProvider() {
  const apiBaseUrl = getApiBaseUrl();

  return createXai({
    apiKey: 'proxy', // Dummy key - actual key is retrieved from database on server
    baseURL: `${apiBaseUrl}/api/ai/xai/v1`,
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
export function getModel(provider: 'openai' | 'anthropic' | 'google' | 'xai', modelName?: string) {
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

  if (provider === 'google') {
    const google = getGoogleProvider();
    const selectedModel = modelName || 'gemini-2.0-flash-exp';
    console.log(`[providers] Creating Google model instance:`, selectedModel);
    return google(selectedModel);
  }

  if (provider === 'xai') {
    const xai = getXAIProvider();
    const selectedModel = modelName || 'grok-beta';
    console.log(`[providers] Creating xAI model instance:`, selectedModel);
    return xai(selectedModel);
  }

  throw new Error(`Unknown provider: ${provider}`);
}

/**
 * Get model with full info (provider, model instance, model name)
 */
export function getModelWithInfo(provider: 'openai' | 'anthropic' | 'google' | 'xai', modelName?: string) {
  let selectedModel: string;

  if (provider === 'google') {
    selectedModel = modelName || 'gemini-2.0-flash-exp';
  } else if (provider === 'xai') {
    selectedModel = modelName || 'grok-beta';
  } else {
    const config = AI_CONFIG.providers[provider];
    selectedModel = modelName || config.defaultModel;
  }

  return {
    provider,
    model: getModel(provider, selectedModel),
    modelName: selectedModel,
  };
}

/**
 * Get available models for a provider
 */
export function getAvailableModels(provider: 'openai' | 'anthropic' | 'google' | 'xai'): string[] {
  if (provider === 'google' || provider === 'xai') {
    // Google and xAI models will be populated from the model registry
    return [];
  }
  const config = AI_CONFIG.providers[provider];
  return Object.keys(config.models);
}

/**
 * Get available models with display names for a provider
 */
export function getAvailableModelsWithNames(provider: 'openai' | 'anthropic' | 'google' | 'xai'): Array<{ id: string; name: string }> {
  if (provider === 'google' || provider === 'xai') {
    // Google and xAI models will be populated from the model registry
    return [];
  }
  const config = AI_CONFIG.providers[provider];
  return Object.entries(config.models).map(([id, model]) => ({
    id,
    name: model.displayName,
  }));
}

/**
 * Get all available providers
 */
export function getAvailableProviders(): Array<{ id: 'openai' | 'anthropic' | 'google' | 'xai'; name: string }> {
  return [
    { id: 'openai', name: AI_CONFIG.providers.openai.displayName },
    { id: 'anthropic', name: AI_CONFIG.providers.anthropic.displayName },
    { id: 'google', name: 'Google' },
    { id: 'xai', name: 'xAI' },
  ];
}

/**
 * Get provider display name
 */
export function getProviderDisplayName(provider: 'openai' | 'anthropic' | 'google' | 'xai'): string {
  if (provider === 'google') return 'Google';
  if (provider === 'xai') return 'xAI';
  return AI_CONFIG.providers[provider].displayName;
}

/**
 * Get model display name
 */
export function getModelDisplayName(provider: 'openai' | 'anthropic' | 'google' | 'xai', modelId: string): string {
  if (provider === 'google' || provider === 'xai') {
    // Display name will come from model registry
    return modelId;
  }
  const config = AI_CONFIG.providers[provider];
  return config.models[modelId]?.displayName || modelId;
}
