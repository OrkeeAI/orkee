// ABOUTME: Tests for AI utility functions including token estimation and text chunking
// ABOUTME: Validates size limits, chunking logic, and cost calculations

import { describe, it, expect } from 'vitest';
import {
  estimateTokens,
  validateContentSize,
  chunkText,
  createChunkPrompt,
  estimateProcessingCost,
  mergePRDAnalyses,
} from './utils';
import type { PRDAnalysis } from './schemas';

describe('estimateTokens', () => {
  it('should estimate tokens for short text', () => {
    const text = 'Hello world';
    const tokens = estimateTokens(text);
    expect(tokens).toBeGreaterThan(0);
    expect(tokens).toBeLessThan(10);
  });

  it('should estimate tokens for longer text', () => {
    const text = 'A'.repeat(1000);
    const tokens = estimateTokens(text);
    expect(tokens).toBeGreaterThan(200);
    expect(tokens).toBeLessThan(400);
  });

  it('should handle empty string', () => {
    expect(estimateTokens('')).toBe(0);
  });
});

describe('validateContentSize', () => {
  it('should accept content within limits', () => {
    const text = 'Small content';
    const result = validateContentSize(text, 1000, 100);
    expect(result.valid).toBe(true);
    expect(result.estimatedTokens).toBeGreaterThan(0);
    expect(result.reason).toBeUndefined();
  });

  it('should reject content exceeding limits', () => {
    const text = 'A'.repeat(100000); // ~28k tokens
    const result = validateContentSize(text, 1000, 100);
    expect(result.valid).toBe(false);
    expect(result.estimatedTokens).toBeGreaterThan(1000);
    expect(result.reason).toContain('exceeds limit');
  });

  it('should account for prompt overhead', () => {
    const text = 'A'.repeat(3400); // ~1000 tokens
    const result = validateContentSize(text, 1000, 100);
    expect(result.valid).toBe(false); // Should fail because 1000 + 100 overhead > 1000 limit
  });
});

describe('chunkText', () => {
  it('should not chunk small text', () => {
    const text = 'Small text that fits in one chunk';
    const chunks = chunkText(text, 1000);
    expect(chunks).toHaveLength(1);
    expect(chunks[0]).toBe(text);
  });

  it('should chunk large text at paragraph boundaries', () => {
    const paragraphs = Array(10)
      .fill(0)
      .map((_, i) => `Paragraph ${i}: ${'A'.repeat(500)}`)
      .join('\n\n');

    const chunks = chunkText(paragraphs, 1000); // ~140 chars per chunk
    expect(chunks.length).toBeGreaterThan(1);
    // Each chunk should be reasonably sized
    chunks.forEach((chunk) => {
      expect(chunk.length).toBeGreaterThan(0);
      expect(chunk.length).toBeLessThan(4000);
    });
  });

  it('should handle text with no paragraphs', () => {
    const text = 'A'.repeat(10000);
    const chunks = chunkText(text, 1000);
    expect(chunks.length).toBeGreaterThan(1);
  });

  it('should preserve content across chunks', () => {
    const text = Array(5)
      .fill(0)
      .map((_, i) => `Section ${i}`)
      .join('\n\n');

    const chunks = chunkText(text, 100);
    const reconstructed = chunks.join('\n\n');

    // All sections should be present
    for (let i = 0; i < 5; i++) {
      expect(reconstructed).toContain(`Section ${i}`);
    }
  });
});

describe('createChunkPrompt', () => {
  it('should create single chunk prompt', () => {
    const chunk = 'Content here';
    const basePrompt = 'Analyze this:';
    const prompt = createChunkPrompt(chunk, 0, 1, basePrompt);

    expect(prompt).toContain('Analyze this:');
    expect(prompt).toContain('Content here');
    expect(prompt).not.toContain('part');
  });

  it('should create multi-chunk prompt with context', () => {
    const chunk = 'Chunk content';
    const basePrompt = 'Analyze this:';
    const prompt = createChunkPrompt(chunk, 1, 5, basePrompt);

    expect(prompt).toContain('Analyze this:');
    expect(prompt).toContain('Chunk content');
    expect(prompt).toContain('part 2 of 5');
  });
});

describe('estimateProcessingCost', () => {
  it('should calculate cost correctly', () => {
    const text = 'A'.repeat(3500); // ~1000 tokens
    const result = estimateProcessingCost(text, 0.01, 0.03, 500);

    expect(result.inputTokens).toBeGreaterThan(900);
    expect(result.inputTokens).toBeLessThan(1100);
    expect(result.outputTokens).toBe(500);
    expect(result.totalCost).toBeGreaterThan(0);
    expect(result.totalCost).toBeLessThan(1); // Should be cents, not dollars
  });

  it('should handle zero cost', () => {
    const text = 'short';
    const result = estimateProcessingCost(text, 0, 0, 0);

    expect(result.totalCost).toBe(0);
  });
});

describe('mergePRDAnalyses', () => {
  // Helper to create a minimal valid PRDAnalysis
  const createMockAnalysis = (
    id: string,
    options: {
      capabilityId?: string;
      requirementName?: string;
      scenarioName?: string;
      taskTitle?: string;
    } = {}
  ): PRDAnalysis => ({
    summary: `Summary for ${id}`,
    capabilities: [
      {
        id: options.capabilityId || `capability-${id}`,
        name: `Capability ${id}`,
        purpose: `Purpose for ${id}`,
        requirements: [
          {
            name: options.requirementName || `Requirement ${id}`,
            content: `Content for ${id}`,
            scenarios: [
              {
                name: options.scenarioName || `Scenario ${id}`,
                when: `When ${id}`,
                then: `Then ${id}`,
              },
            ],
          },
        ],
      },
    ],
    suggestedTasks: [
      {
        title: options.taskTitle || `Task ${id}`,
        description: `Description for ${id}`,
        capabilityId: options.capabilityId || `capability-${id}`,
        requirementName: options.requirementName || `Requirement ${id}`,
        complexity: 5,
        priority: 'medium' as const,
      },
    ],
    dependencies: [`Dependency ${id}`],
    technicalConsiderations: [`Tech consideration ${id}`],
  });

  describe('edge cases', () => {
    it('should throw error for empty array', () => {
      expect(() => mergePRDAnalyses([])).toThrow('No analyses to merge');
    });

    it('should return single analysis unchanged', () => {
      const analysis = createMockAnalysis('A');
      const result = mergePRDAnalyses([analysis]);
      expect(result).toEqual(analysis);
    });
  });

  describe('basic merging', () => {
    it('should merge analyses with different capabilities', () => {
      const analysis1 = createMockAnalysis('1', { capabilityId: 'auth' });
      const analysis2 = createMockAnalysis('2', { capabilityId: 'payments' });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.capabilities).toHaveLength(2);
      expect(result.capabilities[0].id).toBe('auth');
      expect(result.capabilities[1].id).toBe('payments');
    });

    it('should combine summaries from all sections', () => {
      const analysis1 = createMockAnalysis('1');
      const analysis2 = createMockAnalysis('2');

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.summary).toContain('Combined analysis from 2 sections');
      expect(result.summary).toContain('Section 1: Summary for 1');
      expect(result.summary).toContain('Section 2: Summary for 2');
    });
  });

  describe('capability deduplication', () => {
    it('should merge capabilities with same ID', () => {
      const analysis1 = createMockAnalysis('1', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
      });
      const analysis2 = createMockAnalysis('2', {
        capabilityId: 'user-auth',
        requirementName: 'Signup',
      });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      // Should have only one capability
      expect(result.capabilities).toHaveLength(1);
      expect(result.capabilities[0].id).toBe('user-auth');

      // Should have both requirements
      expect(result.capabilities[0].requirements).toHaveLength(2);
      expect(result.capabilities[0].requirements[0].name).toBe('Login');
      expect(result.capabilities[0].requirements[1].name).toBe('Signup');
    });
  });

  describe('requirement deduplication', () => {
    it('should deduplicate requirements by name within same capability', () => {
      const analysis1 = createMockAnalysis('1', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
        scenarioName: 'Email Login',
      });
      const analysis2 = createMockAnalysis('2', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
        scenarioName: 'Social Login',
      });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      // Should have one capability
      expect(result.capabilities).toHaveLength(1);

      // Should have only one "Login" requirement (deduplicated)
      expect(result.capabilities[0].requirements).toHaveLength(1);
      expect(result.capabilities[0].requirements[0].name).toBe('Login');

      // Should have both scenarios merged into that requirement
      expect(result.capabilities[0].requirements[0].scenarios).toHaveLength(2);
      expect(result.capabilities[0].requirements[0].scenarios[0].name).toBe('Email Login');
      expect(result.capabilities[0].requirements[0].scenarios[1].name).toBe('Social Login');
    });

    it('should keep unique requirements from different chunks', () => {
      const analysis1 = createMockAnalysis('1', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
      });
      const analysis2 = createMockAnalysis('2', {
        capabilityId: 'user-auth',
        requirementName: 'Password Reset',
      });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.capabilities[0].requirements).toHaveLength(2);
      expect(result.capabilities[0].requirements[0].name).toBe('Login');
      expect(result.capabilities[0].requirements[1].name).toBe('Password Reset');
    });
  });

  describe('scenario deduplication', () => {
    it('should deduplicate scenarios by name within same requirement', () => {
      const analysis1 = createMockAnalysis('1', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
        scenarioName: 'Email Login',
      });
      const analysis2 = createMockAnalysis('2', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
        scenarioName: 'Email Login', // Same scenario name
      });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      // Should have only one scenario (deduplicated)
      expect(result.capabilities[0].requirements[0].scenarios).toHaveLength(1);
      expect(result.capabilities[0].requirements[0].scenarios[0].name).toBe('Email Login');
    });

    it('should keep unique scenarios within same requirement', () => {
      const analysis1 = createMockAnalysis('1', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
        scenarioName: 'Email Login',
      });
      const analysis2 = createMockAnalysis('2', {
        capabilityId: 'user-auth',
        requirementName: 'Login',
        scenarioName: 'Social Login',
      });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.capabilities[0].requirements[0].scenarios).toHaveLength(2);
      expect(result.capabilities[0].requirements[0].scenarios[0].name).toBe('Email Login');
      expect(result.capabilities[0].requirements[0].scenarios[1].name).toBe('Social Login');
    });
  });

  describe('task deduplication', () => {
    it('should deduplicate tasks by title', () => {
      const analysis1 = createMockAnalysis('1', {
        taskTitle: 'Implement Login',
      });
      const analysis2 = createMockAnalysis('2', {
        taskTitle: 'Implement Login', // Same task title
      });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.suggestedTasks).toHaveLength(1);
      expect(result.suggestedTasks[0].title).toBe('Implement Login');
    });

    it('should keep unique tasks', () => {
      const analysis1 = createMockAnalysis('1', {
        taskTitle: 'Implement Login',
      });
      const analysis2 = createMockAnalysis('2', {
        taskTitle: 'Implement Signup',
      });

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.suggestedTasks).toHaveLength(2);
      expect(result.suggestedTasks.map((t) => t.title)).toContain('Implement Login');
      expect(result.suggestedTasks.map((t) => t.title)).toContain('Implement Signup');
    });
  });

  describe('dependencies and technical considerations', () => {
    it('should deduplicate dependencies', () => {
      const analysis1 = createMockAnalysis('1');
      analysis1.dependencies = ['React', 'TypeScript'];

      const analysis2 = createMockAnalysis('2');
      analysis2.dependencies = ['TypeScript', 'Node.js']; // TypeScript duplicated

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.dependencies).toHaveLength(3);
      expect(result.dependencies).toContain('React');
      expect(result.dependencies).toContain('TypeScript');
      expect(result.dependencies).toContain('Node.js');
    });

    it('should deduplicate technical considerations', () => {
      const analysis1 = createMockAnalysis('1');
      analysis1.technicalConsiderations = ['Security', 'Performance'];

      const analysis2 = createMockAnalysis('2');
      analysis2.technicalConsiderations = ['Performance', 'Scalability']; // Performance duplicated

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.technicalConsiderations).toHaveLength(3);
      expect(result.technicalConsiderations).toContain('Security');
      expect(result.technicalConsiderations).toContain('Performance');
      expect(result.technicalConsiderations).toContain('Scalability');
    });

    it('should handle missing optional fields', () => {
      const analysis1 = createMockAnalysis('1');
      delete analysis1.dependencies;
      delete analysis1.technicalConsiderations;

      const analysis2 = createMockAnalysis('2');
      analysis2.dependencies = ['React'];
      analysis2.technicalConsiderations = ['Security'];

      const result = mergePRDAnalyses([analysis1, analysis2]);

      expect(result.dependencies).toEqual(['React']);
      expect(result.technicalConsiderations).toEqual(['Security']);
    });
  });

  describe('complex scenarios', () => {
    it('should handle three-level deduplication correctly', () => {
      // Chunk 1: auth capability with login requirement and email scenario
      const chunk1 = createMockAnalysis('1', {
        capabilityId: 'auth',
        requirementName: 'Login',
        scenarioName: 'Email',
      });

      // Chunk 2: Same auth capability, same login requirement, different scenario
      const chunk2 = createMockAnalysis('2', {
        capabilityId: 'auth',
        requirementName: 'Login',
        scenarioName: 'Social',
      });

      // Chunk 3: Same auth capability, different requirement
      const chunk3 = createMockAnalysis('3', {
        capabilityId: 'auth',
        requirementName: 'Signup',
        scenarioName: 'Email',
      });

      const result = mergePRDAnalyses([chunk1, chunk2, chunk3]);

      // Should have 1 capability
      expect(result.capabilities).toHaveLength(1);
      expect(result.capabilities[0].id).toBe('auth');

      // Should have 2 requirements (Login and Signup)
      expect(result.capabilities[0].requirements).toHaveLength(2);

      // Login requirement should have 2 scenarios
      const loginReq = result.capabilities[0].requirements.find((r) => r.name === 'Login');
      expect(loginReq?.scenarios).toHaveLength(2);
      expect(loginReq?.scenarios.map((s) => s.name)).toContain('Email');
      expect(loginReq?.scenarios.map((s) => s.name)).toContain('Social');

      // Signup requirement should have 1 scenario
      const signupReq = result.capabilities[0].requirements.find((r) => r.name === 'Signup');
      expect(signupReq?.scenarios).toHaveLength(1);
      expect(signupReq?.scenarios[0].name).toBe('Email');
    });

    it('should preserve all unique data across many chunks', () => {
      const chunks = [
        createMockAnalysis('1', { capabilityId: 'cap1' }),
        createMockAnalysis('2', { capabilityId: 'cap2' }),
        createMockAnalysis('3', { capabilityId: 'cap3' }),
        createMockAnalysis('4', { capabilityId: 'cap1' }), // Duplicate capability
      ];

      const result = mergePRDAnalyses(chunks);

      // Should have 3 unique capabilities
      expect(result.capabilities).toHaveLength(3);

      // cap1 should have merged requirements from chunks 1 and 4
      const cap1 = result.capabilities.find((c) => c.id === 'cap1');
      expect(cap1?.requirements).toHaveLength(2);
    });
  });
});
