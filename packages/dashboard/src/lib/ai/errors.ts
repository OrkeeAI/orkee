// ABOUTME: Custom error types for AI service operations
// ABOUTME: Provides specialized error classes for timeout, rate limiting, and service failures

/**
 * Timeout error for AI operations that exceed time limits
 */
export class TimeoutError extends Error {
  constructor(message: string, public readonly timeoutMs: number) {
    super(message);
    this.name = 'TimeoutError';
  }
}

/**
 * Rate limit error when API call limits are exceeded
 */
export class RateLimitError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'RateLimitError';
  }
}

/**
 * Network error for connection and request failures
 */
export class NetworkError extends Error {
  constructor(
    message: string,
    public readonly cause?: unknown,
    public readonly retryable: boolean = true
  ) {
    super(message);
    this.name = 'NetworkError';
  }
}

/**
 * Validation error for invalid inputs or responses
 */
export class ValidationError extends Error {
  constructor(
    message: string,
    public readonly validationErrors?: unknown
  ) {
    super(message);
    this.name = 'ValidationError';
  }
}

/**
 * Service error for general AI service failures
 */
export class AIServiceError extends Error {
  constructor(
    message: string,
    public readonly operation: string,
    public readonly cause?: unknown,
    public readonly retryable: boolean = false
  ) {
    super(message);
    this.name = 'AIServiceError';
  }
}

/**
 * Check if an error is retryable
 */
export function isRetryableError(error: unknown): boolean {
  if (error instanceof NetworkError) {
    return error.retryable;
  }
  if (error instanceof AIServiceError) {
    return error.retryable;
  }
  if (error instanceof TimeoutError) {
    return true;
  }
  return false;
}

/**
 * Extract user-friendly error message from any error
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof AIServiceError) {
    return `${error.operation} failed: ${error.message}`;
  }
  if (error instanceof TimeoutError) {
    return `Operation timed out after ${error.timeoutMs}ms: ${error.message}`;
  }
  if (error instanceof RateLimitError) {
    return `Rate limit exceeded: ${error.message}`;
  }
  if (error instanceof NetworkError) {
    return `Network error: ${error.message}`;
  }
  if (error instanceof ValidationError) {
    return `Validation error: ${error.message}`;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unknown error occurred';
}
