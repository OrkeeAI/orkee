// ABOUTME: Public API exports for the prompts package
// ABOUTME: Exposes PromptManager and type definitions for consumers

export { PromptManager } from './PromptManager';
export type { Prompt, PromptParameters, PromptCategory, PromptMetadata } from './types';
export { PromptNotFoundError, PromptParameterError } from './types';
