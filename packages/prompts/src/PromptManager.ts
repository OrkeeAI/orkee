// ABOUTME: Core prompt management service with loading, caching, and parameter substitution
// ABOUTME: Provides unified interface for accessing JSON-based and database-backed prompts

import * as fs from 'fs/promises';
import * as path from 'path';
import type { Prompt, PromptParameters, PromptCategory } from './types';
import { PromptNotFoundError, PromptParameterError } from './types';

export class PromptManager {
  private promptCache: Map<string, Prompt> = new Map();
  private promptsDir: string;

  constructor(promptsDir?: string) {
    // Default to the prompts directory relative to this file
    this.promptsDir = promptsDir || path.join(__dirname, '..');
  }

  /**
   * Load a prompt by ID from the appropriate category directory
   */
  async getPrompt(promptId: string, parameters?: PromptParameters): Promise<string> {
    const prompt = await this.loadPrompt(promptId);

    if (parameters) {
      return this.substituteParameters(prompt.template, parameters, prompt.parameters);
    }

    return prompt.template;
  }

  /**
   * Get a system prompt by category
   */
  async getSystemPrompt(category: 'prd' | 'research'): Promise<string> {
    const prompt = await this.loadPromptFromPath(
      path.join(this.promptsDir, 'system', `${category}.json`)
    );
    return prompt.template;
  }

  /**
   * Load prompt metadata without substitution
   */
  async getPromptMetadata(promptId: string): Promise<Prompt> {
    return this.loadPrompt(promptId);
  }

  /**
   * Substitute parameters in a template
   */
  private substituteParameters(
    template: string,
    parameters: PromptParameters,
    requiredParams: string[]
  ): string {
    // Validate all required parameters are provided
    const missing = requiredParams.filter(param => !(param in parameters));
    if (missing.length > 0) {
      throw new PromptParameterError(
        `Missing required parameters: ${missing.join(', ')}`
      );
    }

    // Replace {{parameter}} with values
    let result = template;
    for (const [key, value] of Object.entries(parameters)) {
      const placeholder = `{{${key}}}`;
      result = result.replace(new RegExp(placeholder.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g'), value);
    }

    return result;
  }

  /**
   * Load a prompt from disk with caching
   */
  private async loadPrompt(promptId: string): Promise<Prompt> {
    // Check cache first
    if (this.promptCache.has(promptId)) {
      return this.promptCache.get(promptId)!;
    }

    // Try to find the prompt in standard locations
    const possiblePaths = [
      path.join(this.promptsDir, 'prd', `${promptId}.json`),
      path.join(this.promptsDir, 'research', `${promptId}.json`),
      path.join(this.promptsDir, 'system', `${promptId}.json`),
    ];

    for (const promptPath of possiblePaths) {
      try {
        const prompt = await this.loadPromptFromPath(promptPath);
        this.promptCache.set(promptId, prompt);
        return prompt;
      } catch (error) {
        // Continue to next path
        continue;
      }
    }

    throw new PromptNotFoundError(promptId);
  }

  /**
   * Load a prompt from a specific file path
   */
  private async loadPromptFromPath(filePath: string): Promise<Prompt> {
    try {
      const content = await fs.readFile(filePath, 'utf-8');
      const prompt = JSON.parse(content) as Prompt;

      // Basic validation
      if (!prompt.id || !prompt.template || !prompt.category) {
        throw new Error(`Invalid prompt format in ${filePath}`);
      }

      return prompt;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
        throw new PromptNotFoundError(path.basename(filePath, '.json'));
      }
      throw error;
    }
  }

  /**
   * Clear the prompt cache (useful for testing)
   */
  clearCache(): void {
    this.promptCache.clear();
  }

  /**
   * List all available prompts in a category
   */
  async listPrompts(category: PromptCategory): Promise<string[]> {
    const categoryDir = path.join(this.promptsDir, category);
    try {
      const files = await fs.readdir(categoryDir);
      return files
        .filter(file => file.endsWith('.json'))
        .map(file => path.basename(file, '.json'));
    } catch (error) {
      return [];
    }
  }
}
