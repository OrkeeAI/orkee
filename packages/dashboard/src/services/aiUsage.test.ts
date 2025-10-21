// ABOUTME: Tests for AI usage cost formatting and calculations
// ABOUTME: Validates cost, token, and duration formatting functions

import { describe, it, expect } from 'vitest';
import { formatCost, formatTokens, formatDuration } from './aiUsage';

describe('AI Usage Formatting', () => {
  describe('formatCost', () => {
    it('should format zero cost', () => {
      expect(formatCost(0)).toBe('$0.00');
    });

    it('should format costs less than 1 cent in cents', () => {
      expect(formatCost(0.001)).toBe('0.100¢');
      expect(formatCost(0.005)).toBe('0.500¢');
      expect(formatCost(0.009)).toBe('0.900¢');
    });

    it('should format costs 1 cent and above in dollars', () => {
      expect(formatCost(0.01)).toBe('$0.01');
      expect(formatCost(0.10)).toBe('$0.10');
      expect(formatCost(1.00)).toBe('$1.00');
      expect(formatCost(10.50)).toBe('$10.50');
    });

    it('should round to 2 decimal places for dollars', () => {
      expect(formatCost(1.234)).toBe('$1.23');
      expect(formatCost(1.235)).toBe('$1.24');  // Rounds up
      expect(formatCost(1.999)).toBe('$2.00');
    });

    it('should handle large costs', () => {
      expect(formatCost(100)).toBe('$100.00');
      expect(formatCost(1000.99)).toBe('$1000.99');
      expect(formatCost(10000.123)).toBe('$10000.12');
    });

    it('should handle very small costs', () => {
      expect(formatCost(0.0001)).toBe('0.010¢');
      expect(formatCost(0.00001)).toBe('0.001¢');
    });

    it('should format typical API costs correctly', () => {
      // GPT-4 typical cost per 1K tokens: ~$0.03
      expect(formatCost(0.03)).toBe('$0.03');

      // Claude-3 Sonnet typical cost per 1K tokens: ~$0.003
      expect(formatCost(0.003)).toBe('0.300¢');

      // Very small request
      expect(formatCost(0.0001)).toBe('0.010¢');
    });
  });

  describe('formatTokens', () => {
    it('should format small token counts without commas', () => {
      expect(formatTokens(0)).toBe('0');
      expect(formatTokens(1)).toBe('1');
      expect(formatTokens(999)).toBe('999');
    });

    it('should format token counts with commas for thousands', () => {
      expect(formatTokens(1000)).toBe('1,000');
      expect(formatTokens(10000)).toBe('10,000');
      expect(formatTokens(100000)).toBe('100,000');
    });

    it('should format large token counts', () => {
      expect(formatTokens(1000000)).toBe('1,000,000');
      expect(formatTokens(1234567)).toBe('1,234,567');
    });

    it('should format typical API token usage', () => {
      // Small request (e.g., simple question)
      expect(formatTokens(150)).toBe('150');

      // Medium request (e.g., code generation)
      expect(formatTokens(2500)).toBe('2,500');

      // Large request (e.g., document analysis)
      expect(formatTokens(15000)).toBe('15,000');
    });
  });

  describe('formatDuration', () => {
    it('should format durations under 1 second in milliseconds', () => {
      expect(formatDuration(0)).toBe('0ms');
      expect(formatDuration(100)).toBe('100ms');
      expect(formatDuration(500)).toBe('500ms');
      expect(formatDuration(999)).toBe('999ms');
    });

    it('should format durations 1 second and above in seconds', () => {
      expect(formatDuration(1000)).toBe('1.00s');
      expect(formatDuration(2000)).toBe('2.00s');
      expect(formatDuration(5000)).toBe('5.00s');
    });

    it('should format durations with decimal precision', () => {
      expect(formatDuration(1234)).toBe('1.23s');
      expect(formatDuration(1567)).toBe('1.57s');
      expect(formatDuration(9999)).toBe('10.00s');
    });

    it('should format typical API latencies', () => {
      // Fast response (100ms)
      expect(formatDuration(100)).toBe('100ms');

      // Normal response (500ms)
      expect(formatDuration(500)).toBe('500ms');

      // Slow response (2s)
      expect(formatDuration(2000)).toBe('2.00s');

      // Very slow response (10s)
      expect(formatDuration(10000)).toBe('10.00s');
    });

    it('should handle edge cases', () => {
      expect(formatDuration(999.9)).toBe('1000ms');
      expect(formatDuration(1000.1)).toBe('1.00s');
    });
  });

  describe('Integration: Cost Dashboard Calculations', () => {
    it('should calculate total cost correctly', () => {
      const requests = [
        { cost: 0.001, tokens: 100 },
        { cost: 0.05, tokens: 2000 },
        { cost: 0.10, tokens: 5000 },
      ];

      const totalCost = requests.reduce((sum, req) => sum + req.cost, 0);
      expect(formatCost(totalCost)).toBe('$0.15');
    });

    it('should calculate average cost per request', () => {
      const requests = [
        { cost: 0.01 },
        { cost: 0.02 },
        { cost: 0.03 },
      ];

      const avgCost = requests.reduce((sum, req) => sum + req.cost, 0) / requests.length;
      expect(formatCost(avgCost)).toBe('$0.02');
    });

    it('should calculate cost per token', () => {
      const totalCost = 1.00;
      const totalTokens = 100000;
      const costPerToken = totalCost / totalTokens;

      expect(formatCost(costPerToken)).toBe('0.001¢');
    });

    it('should format aggregated statistics correctly', () => {
      const stats = {
        totalRequests: 1000,
        totalTokens: 500000,
        totalCost: 15.50,
        averageDurationMs: 750,
      };

      expect(formatTokens(stats.totalTokens)).toBe('500,000');
      expect(formatCost(stats.totalCost)).toBe('$15.50');
      expect(formatDuration(stats.averageDurationMs)).toBe('750ms');

      const costPerRequest = stats.totalCost / stats.totalRequests;
      expect(formatCost(costPerRequest)).toBe('$0.02');
    });

    it('should handle zero values in calculations', () => {
      const stats = {
        totalRequests: 0,
        totalTokens: 0,
        totalCost: 0,
      };

      expect(formatTokens(stats.totalTokens)).toBe('0');
      expect(formatCost(stats.totalCost)).toBe('$0.00');

      // Avoid division by zero
      const avgCost = stats.totalRequests > 0 ? stats.totalCost / stats.totalRequests : 0;
      expect(formatCost(avgCost)).toBe('$0.00');
    });
  });
});
