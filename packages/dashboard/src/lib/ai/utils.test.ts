// ABOUTME: Tests for AI utility functions including token estimation and text chunking
// ABOUTME: Validates size limits, chunking logic, and cost calculations

import { describe, it, expect } from 'vitest';
import {
  estimateTokens,
  validateContentSize,
  chunkText,
  createChunkPrompt,
  estimateProcessingCost,
} from './utils';

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
