-- Rollback Migration: Research Analysis Cache
-- Description: Remove competitor analysis cache table

-- Drop indexes first
DROP INDEX IF EXISTS idx_competitor_cache_created;
DROP INDEX IF EXISTS idx_competitor_cache_session;

-- Drop the cache table
DROP TABLE IF EXISTS competitor_analysis_cache;
