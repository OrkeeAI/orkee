// ABOUTME: Caching layer for AI responses to reduce duplicate API calls
// ABOUTME: In-memory LRU cache with TTL support and content-based hashing

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  hits: number;
}

interface CacheConfig {
  maxEntries: number;
  defaultTTL: number; // in milliseconds
}

const DEFAULT_CONFIG: CacheConfig = {
  maxEntries: 100,
  defaultTTL: 60 * 60 * 1000, // 1 hour
};

/**
 * LRU cache for AI responses
 */
export class AICache {
  private cache: Map<string, CacheEntry<unknown>>;
  private config: CacheConfig;
  private hits: number = 0;
  private misses: number = 0;

  constructor(config: Partial<CacheConfig> = {}) {
    this.cache = new Map();
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Generate cache key from operation and parameters
   */
  private generateKey(operation: string, params: unknown): string {
    // Simple hash function for cache key
    const paramsStr = JSON.stringify(params);
    return `${operation}:${this.simpleHash(paramsStr)}`;
  }

  /**
   * Simple hash function for string content
   */
  private simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString(36);
  }

  /**
   * Get cached value if exists and not expired
   */
  get<T>(operation: string, params: unknown): T | null {
    const key = this.generateKey(operation, params);
    const entry = this.cache.get(key);

    if (!entry) {
      this.misses++;
      return null;
    }

    // Check if expired
    const now = Date.now();
    if (now - entry.timestamp > this.config.defaultTTL) {
      this.cache.delete(key);
      this.misses++;
      return null;
    }

    // Update hit count and move to end (LRU)
    entry.hits++;
    this.cache.delete(key);
    this.cache.set(key, entry);
    this.hits++;

    return entry.data as T;
  }

  /**
   * Set cache value
   */
  set<T>(operation: string, params: unknown, data: T): void {
    const key = this.generateKey(operation, params);

    // Check if we need to evict
    if (this.cache.size >= this.config.maxEntries) {
      // Remove oldest entry (first in Map)
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    }

    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      hits: 0,
    });
  }

  /**
   * Check if key exists in cache
   */
  has(operation: string, params: unknown): boolean {
    const key = this.generateKey(operation, params);
    const entry = this.cache.get(key);

    if (!entry) {
      return false;
    }

    // Check if expired
    const now = Date.now();
    if (now - entry.timestamp > this.config.defaultTTL) {
      this.cache.delete(key);
      return false;
    }

    return true;
  }

  /**
   * Clear all cached entries
   */
  clear(): void {
    this.cache.clear();
    this.hits = 0;
    this.misses = 0;
  }

  /**
   * Clear entries for a specific operation
   */
  clearOperation(operation: string): void {
    const keys = Array.from(this.cache.keys());
    for (const key of keys) {
      if (key.startsWith(`${operation}:`)) {
        this.cache.delete(key);
      }
    }
  }

  /**
   * Get cache statistics
   */
  getStats(): {
    size: number;
    hits: number;
    misses: number;
    hitRate: number;
    maxEntries: number;
  } {
    const total = this.hits + this.misses;
    return {
      size: this.cache.size,
      hits: this.hits,
      misses: this.misses,
      hitRate: total > 0 ? this.hits / total : 0,
      maxEntries: this.config.maxEntries,
    };
  }

  /**
   * Manually expire entries older than specified age
   */
  prune(maxAge?: number): number {
    const cutoff = Date.now() - (maxAge || this.config.defaultTTL);
    let pruned = 0;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.timestamp < cutoff) {
        this.cache.delete(key);
        pruned++;
      }
    }

    return pruned;
  }

  /**
   * Update cache configuration
   */
  updateConfig(config: Partial<CacheConfig>): void {
    this.config = { ...this.config, ...config };

    // If maxEntries reduced, trim cache
    while (this.cache.size > this.config.maxEntries) {
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    }
  }
}

/**
 * Global AI cache instance
 */
export const aiCache = new AICache();
