// ABOUTME: Tests for custom error types
// ABOUTME: Verifies error creation, properties, and utility functions

import { describe, it, expect } from 'vitest';
import {
  TimeoutError,
  RateLimitError,
  NetworkError,
  ValidationError,
  AIServiceError,
  isRetryableError,
  getErrorMessage,
} from './errors';

describe('Error Types', () => {
  describe('TimeoutError', () => {
    it('should create timeout error with correct properties', () => {
      const error = new TimeoutError('Operation timed out', 5000);
      expect(error.name).toBe('TimeoutError');
      expect(error.message).toBe('Operation timed out');
      expect(error.timeoutMs).toBe(5000);
    });
  });

  describe('RateLimitError', () => {
    it('should create rate limit error with correct properties', () => {
      const error = new RateLimitError('Rate limit exceeded');
      expect(error.name).toBe('RateLimitError');
      expect(error.message).toBe('Rate limit exceeded');
    });
  });

  describe('NetworkError', () => {
    it('should create network error with default retryable true', () => {
      const error = new NetworkError('Connection failed');
      expect(error.name).toBe('NetworkError');
      expect(error.message).toBe('Connection failed');
      expect(error.retryable).toBe(true);
    });

    it('should create network error with custom retryable flag', () => {
      const cause = new Error('DNS failure');
      const error = new NetworkError('Connection failed', cause, false);
      expect(error.retryable).toBe(false);
      expect(error.cause).toBe(cause);
    });
  });

  describe('ValidationError', () => {
    it('should create validation error with validation details', () => {
      const validationErrors = { field: 'invalid value' };
      const error = new ValidationError('Invalid input', validationErrors);
      expect(error.name).toBe('ValidationError');
      expect(error.message).toBe('Invalid input');
      expect(error.validationErrors).toEqual(validationErrors);
    });
  });

  describe('AIServiceError', () => {
    it('should create service error with operation name', () => {
      const error = new AIServiceError('Analysis failed', 'analyzePRD');
      expect(error.name).toBe('AIServiceError');
      expect(error.message).toBe('Analysis failed');
      expect(error.operation).toBe('analyzePRD');
      expect(error.retryable).toBe(false);
    });

    it('should create retryable service error', () => {
      const cause = new Error('Temporary failure');
      const error = new AIServiceError('Analysis failed', 'analyzePRD', cause, true);
      expect(error.retryable).toBe(true);
      expect(error.cause).toBe(cause);
    });
  });

  describe('isRetryableError', () => {
    it('should identify retryable network errors', () => {
      const error = new NetworkError('Connection failed', undefined, true);
      expect(isRetryableError(error)).toBe(true);
    });

    it('should identify non-retryable network errors', () => {
      const error = new NetworkError('Connection failed', undefined, false);
      expect(isRetryableError(error)).toBe(false);
    });

    it('should identify retryable service errors', () => {
      const error = new AIServiceError('Failed', 'test', undefined, true);
      expect(isRetryableError(error)).toBe(true);
    });

    it('should identify non-retryable service errors', () => {
      const error = new AIServiceError('Failed', 'test', undefined, false);
      expect(isRetryableError(error)).toBe(false);
    });

    it('should identify timeout errors as retryable', () => {
      const error = new TimeoutError('Timed out', 5000);
      expect(isRetryableError(error)).toBe(true);
    });

    it('should return false for unknown errors', () => {
      const error = new Error('Unknown');
      expect(isRetryableError(error)).toBe(false);
    });
  });

  describe('getErrorMessage', () => {
    it('should format AIServiceError message', () => {
      const error = new AIServiceError('Analysis failed', 'analyzePRD');
      expect(getErrorMessage(error)).toBe('analyzePRD failed: Analysis failed');
    });

    it('should format TimeoutError message', () => {
      const error = new TimeoutError('Operation timed out', 5000);
      expect(getErrorMessage(error)).toBe('Operation timed out after 5000ms: Operation timed out');
    });

    it('should format RateLimitError message', () => {
      const error = new RateLimitError('Too many requests');
      expect(getErrorMessage(error)).toBe('Rate limit exceeded: Too many requests');
    });

    it('should format NetworkError message', () => {
      const error = new NetworkError('Connection failed');
      expect(getErrorMessage(error)).toBe('Network error: Connection failed');
    });

    it('should format ValidationError message', () => {
      const error = new ValidationError('Invalid input');
      expect(getErrorMessage(error)).toBe('Validation error: Invalid input');
    });

    it('should handle generic Error', () => {
      const error = new Error('Something went wrong');
      expect(getErrorMessage(error)).toBe('Something went wrong');
    });

    it('should handle unknown error types', () => {
      const error = 'string error';
      expect(getErrorMessage(error)).toBe('An unknown error occurred');
    });
  });
});
