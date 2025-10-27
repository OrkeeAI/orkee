-- Migration: Research Analysis Cache for Comprehensive Mode
-- Description: Add caching table for competitor analysis to reduce redundant API calls and web scraping
--
-- This migration adds the competitor_analysis_cache table to store:
-- - Scraped competitor data (name, URL, features, strengths, gaps)
-- - Timestamp for cache expiration (24-hour TTL)
-- - Session association for context-aware caching

-- ============================================================================
-- COMPETITOR ANALYSIS CACHE
-- ============================================================================
-- Purpose: Cache competitor analysis results to avoid re-scraping URLs
-- Retention: 24 hours (enforced by application logic in research_analyzer.rs)

CREATE TABLE IF NOT EXISTS competitor_analysis_cache (
    session_id TEXT NOT NULL,
    url TEXT NOT NULL,
    name TEXT NOT NULL,
    strengths TEXT NOT NULL,  -- JSON array: ["strength1", "strength2", ...]
    gaps TEXT NOT NULL,       -- JSON array: ["gap1", "gap2", ...]
    features TEXT NOT NULL,   -- JSON array: ["feature1", "feature2", ...]
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    PRIMARY KEY (session_id, url),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

-- Index for cache expiration queries
CREATE INDEX IF NOT EXISTS idx_competitor_cache_created
ON competitor_analysis_cache(created_at);

-- Index for session lookups
CREATE INDEX IF NOT EXISTS idx_competitor_cache_session
ON competitor_analysis_cache(session_id);
