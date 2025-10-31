// ABOUTME: Unit tests for PromptManager
// ABOUTME: Verifies prompt loading, caching, parameter substitution, and error handling

import { describe, it, expect, beforeEach } from 'vitest';
import { PromptManager } from './PromptManager';
import { PromptNotFoundError, PromptParameterError, PathTraversalError } from './types';
import * as path from 'path';

describe('PromptManager', () => {
  let manager: PromptManager;

  beforeEach(() => {
    // Point to the root prompts directory
    manager = new PromptManager(path.join(__dirname, '..'));
  });

  describe('getSystemPrompt', () => {
    it('should load PRD system prompt', async () => {
      const prompt = await manager.getSystemPrompt('prd');
      expect(prompt).toContain('expert product manager');
      expect(prompt).toContain('Product Requirement Documents');
    });

    it('should load research system prompt', async () => {
      const prompt = await manager.getSystemPrompt('research');
      expect(prompt).toContain('product researcher');
      expect(prompt).toContain('competitive analyst');
    });
  });

  describe('getPrompt', () => {
    it('should load overview prompt', async () => {
      const prompt = await manager.getPrompt('overview', {
        description: 'A mobile app for tracking habits'
      });
      expect(prompt).toContain('A mobile app for tracking habits');
      expect(prompt).toContain('problemStatement');
      expect(prompt).toContain('targetAudience');
    });

    it('should load features prompt', async () => {
      const prompt = await manager.getPrompt('features', {
        description: 'A task management tool'
      });
      expect(prompt).toContain('A task management tool');
      expect(prompt).toContain('5-8 core features');
    });

    it('should throw error for missing prompt', async () => {
      await expect(
        manager.getPrompt('nonexistent')
      ).rejects.toThrow(PromptNotFoundError);
    });

    it('should throw error for missing parameters', async () => {
      await expect(
        manager.getPrompt('overview', {})
      ).rejects.toThrow(PromptParameterError);
    });
  });

  describe('parameter substitution', () => {
    it('should substitute multiple parameters', async () => {
      const prompt = await manager.getPrompt('roadmap', {
        description: 'My project',
        features: 'Feature 1, Feature 2'
      });
      expect(prompt).toContain('My project');
      expect(prompt).toContain('Feature 1, Feature 2');
    });

    it('should handle parameters with special characters', async () => {
      const description = 'A project with $pecial ch@racters & symbols';
      const prompt = await manager.getPrompt('overview', { description });
      expect(prompt).toContain(description);
    });
  });

  describe('caching', () => {
    it('should cache loaded prompts', async () => {
      await manager.getPrompt('overview', { description: 'Test' });
      const metadata = await manager.getPromptMetadata('overview');
      expect(metadata.id).toBe('overview');
    });

    it('should clear cache', async () => {
      await manager.getPrompt('overview', { description: 'Test' });
      manager.clearCache();
      // Should still work after cache clear
      const prompt = await manager.getPrompt('overview', { description: 'Test 2' });
      expect(prompt).toContain('Test 2');
    });
  });

  describe('listPrompts', () => {
    it('should list PRD prompts', async () => {
      const prompts = await manager.listPrompts('prd');
      expect(prompts).toContain('overview');
      expect(prompts).toContain('features');
      expect(prompts).toContain('ux');
      expect(prompts).toContain('technical');
    });

    it('should list research prompts', async () => {
      const prompts = await manager.listPrompts('research');
      expect(prompts).toContain('competitor-analysis');
      expect(prompts).toContain('feature-extraction');
    });
  });

  describe('getPromptMetadata', () => {
    it('should return prompt metadata', async () => {
      const metadata = await manager.getPromptMetadata('overview');
      expect(metadata.id).toBe('overview');
      expect(metadata.name).toBe('Project Overview');
      expect(metadata.category).toBe('prd');
      expect(metadata.parameters).toContain('description');
    });
  });

  describe('security - path traversal prevention', () => {
    it('should reject prompt IDs with .. traversal', async () => {
      await expect(
        manager.getPrompt('../../../etc/passwd')
      ).rejects.toThrow(PathTraversalError);
    });

    it('should reject prompt IDs with forward slashes', async () => {
      await expect(
        manager.getPrompt('system/../../etc/passwd')
      ).rejects.toThrow(PathTraversalError);
    });

    it('should reject prompt IDs with backslashes', async () => {
      await expect(
        manager.getPrompt('..\\..\\..\\etc\\passwd')
      ).rejects.toThrow(PathTraversalError);
    });

    it('should reject prompt IDs with mixed path separators', async () => {
      await expect(
        manager.getPrompt('../prd/../system/../../etc/passwd')
      ).rejects.toThrow(PathTraversalError);
    });

    it('should allow valid prompt IDs with hyphens and underscores', async () => {
      // This test verifies that legitimate prompt IDs still work
      const prompt = await manager.getPrompt('competitor-analysis', {
        projectDescription: 'Test Project',
        htmlContent: '<html>Test</html>',
        url: 'https://example.com'
      });
      expect(prompt).toBeTruthy();
    });

    it('should reject symlink-based traversal attempts', async () => {
      // Even if a symlink exists, validation should prevent escaping prompts dir
      await expect(
        manager.getPrompt('../../symlink-to-etc')
      ).rejects.toThrow(PathTraversalError);
    });
  });
});
