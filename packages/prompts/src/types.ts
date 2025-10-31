// ABOUTME: TypeScript type definitions for AI prompt templates
// ABOUTME: Provides type safety for prompt loading, validation, and parameter substitution

export type PromptCategory = 'prd' | 'research' | 'expert' | 'system';

export interface PromptMetadata {
  version: string;
  lastModified: string;
  description: string;
}

export interface Prompt {
  id: string;
  name: string;
  category: PromptCategory;
  template: string;
  parameters: string[];
  outputSchema?: Record<string, any>;
  metadata?: PromptMetadata;
}

export interface PromptParameters {
  [key: string]: string;
}

export class PromptNotFoundError extends Error {
  constructor(promptId: string) {
    super(`Prompt not found: ${promptId}`);
    this.name = 'PromptNotFoundError';
  }
}

export class PromptParameterError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'PromptParameterError';
  }
}

export class PathTraversalError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'PathTraversalError';
  }
}
