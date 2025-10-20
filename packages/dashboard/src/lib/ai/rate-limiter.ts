// ABOUTME: Rate limiting for AI API calls to prevent runaway costs
// ABOUTME: Tracks call frequency and enforces configurable limits per operation type

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

/**
 * Rate limiter for AI API calls
 */
export class AIRateLimiter {
  private config: RateLimitConfig;
  private callHistory: CallRecord[] = [];

  constructor(config: Partial<RateLimitConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Check if a call can be made
   */
  canMakeCall(operation: string): { allowed: boolean; reason?: string } {
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
  }

  /**
   * Remove records older than 24 hours to prevent memory leaks
   */
  private cleanupOldRecords(now: number): void {
    const cutoff = now - 24 * 60 * 60 * 1000;
    this.callHistory = this.callHistory.filter((r) => r.timestamp > cutoff);
  }
}

/**
 * Global rate limiter instance
 */
export const aiRateLimiter = new AIRateLimiter();

/**
 * Custom error for rate limit exceeded
 */
export class RateLimitError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'RateLimitError';
  }
}
