// ABOUTME: Rate limiting for AI API calls to prevent runaway costs
// ABOUTME: Tracks call frequency and enforces configurable limits per operation type

import { RateLimitError } from './errors';

interface RateLimitConfig {
  maxCallsPerMinute: number;
  maxCostPerHour: number; // in USD
  maxCostPerDay: number; // in USD
}

interface CallRecord {
  timestamp: number;
  cost: number;
  operation: string;
}

const DEFAULT_CONFIG: RateLimitConfig = {
  maxCallsPerMinute: 60,
  maxCostPerHour: 5.0, // $5/hour
  maxCostPerDay: 50.0, // $50/day
};

const STORAGE_KEY = 'orkee_ai_rate_limiter_history';

/**
 * Rate limiter for AI API calls
 */
export class AIRateLimiter {
  private config: RateLimitConfig;
  private callHistory: CallRecord[] = [];

  constructor(config: Partial<RateLimitConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.loadFromStorage();
  }

  /**
   * Load call history from localStorage
   */
  private loadFromStorage(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        this.callHistory = JSON.parse(stored);
      }
    } catch (error) {
      console.warn('Failed to load rate limiter history from localStorage:', error);
      this.callHistory = [];
    }
  }

  /**
   * Save call history to localStorage
   */
  private saveToStorage(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.callHistory));
    } catch (error) {
      console.warn('Failed to save rate limiter history to localStorage:', error);
    }
  }

  /**
   * Check if a call can be made
   */
  canMakeCall(): { allowed: boolean; reason?: string } {
    const now = Date.now();
    this.cleanupOldRecords(now);

    // Check calls per minute
    const oneMinuteAgo = now - 60 * 1000;
    const recentCalls = this.callHistory.filter((r) => r.timestamp > oneMinuteAgo);

    if (recentCalls.length >= this.config.maxCallsPerMinute) {
      return {
        allowed: false,
        reason: `Rate limit exceeded: ${recentCalls.length} calls in the last minute (max: ${this.config.maxCallsPerMinute})`,
      };
    }

    // Check cost per hour
    const oneHourAgo = now - 60 * 60 * 1000;
    const hourCost = this.callHistory
      .filter((r) => r.timestamp > oneHourAgo)
      .reduce((sum, r) => sum + r.cost, 0);

    if (hourCost >= this.config.maxCostPerHour) {
      return {
        allowed: false,
        reason: `Hourly cost limit exceeded: $${hourCost.toFixed(4)} in the last hour (max: $${this.config.maxCostPerHour})`,
      };
    }

    // Check cost per day
    const oneDayAgo = now - 24 * 60 * 60 * 1000;
    const dayCost = this.callHistory
      .filter((r) => r.timestamp > oneDayAgo)
      .reduce((sum, r) => sum + r.cost, 0);

    if (dayCost >= this.config.maxCostPerDay) {
      return {
        allowed: false,
        reason: `Daily cost limit exceeded: $${dayCost.toFixed(4)} in the last 24 hours (max: $${this.config.maxCostPerDay})`,
      };
    }

    return { allowed: true };
  }

  /**
   * Record a completed call
   */
  recordCall(operation: string, cost: number): void {
    this.callHistory.push({
      timestamp: Date.now(),
      cost,
      operation,
    });
    this.saveToStorage();
  }

  /**
   * Get current usage statistics
   */
  getUsageStats(): {
    callsLastMinute: number;
    costLastHour: number;
    costLastDay: number;
    limits: RateLimitConfig;
  } {
    const now = Date.now();
    this.cleanupOldRecords(now);

    const oneMinuteAgo = now - 60 * 1000;
    const oneHourAgo = now - 60 * 60 * 1000;
    const oneDayAgo = now - 24 * 60 * 60 * 1000;

    const callsLastMinute = this.callHistory.filter((r) => r.timestamp > oneMinuteAgo).length;
    const costLastHour = this.callHistory
      .filter((r) => r.timestamp > oneHourAgo)
      .reduce((sum, r) => sum + r.cost, 0);
    const costLastDay = this.callHistory
      .filter((r) => r.timestamp > oneDayAgo)
      .reduce((sum, r) => sum + r.cost, 0);

    return {
      callsLastMinute,
      costLastHour,
      costLastDay,
      limits: this.config,
    };
  }

  /**
   * Update rate limit configuration
   */
  updateConfig(config: Partial<RateLimitConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Reset all tracked calls (useful for testing)
   */
  reset(): void {
    this.callHistory = [];
    this.saveToStorage();
  }

  /**
   * Remove records older than 24 hours to prevent memory leaks
   */
  private cleanupOldRecords(now: number): void {
    const cutoff = now - 24 * 60 * 60 * 1000;
    const oldLength = this.callHistory.length;
    this.callHistory = this.callHistory.filter((r) => r.timestamp > cutoff);
    if (this.callHistory.length !== oldLength) {
      this.saveToStorage();
    }
  }
}

/**
 * Global rate limiter instance
 */
export const aiRateLimiter = new AIRateLimiter();

/**
 * Re-export RateLimitError for backward compatibility
 */
export { RateLimitError };
