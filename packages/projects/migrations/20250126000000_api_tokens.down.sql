-- Rollback api_tokens table
-- Down migration: 20250126000000_api_tokens.down.sql

DROP INDEX IF EXISTS idx_api_tokens_hash;
DROP INDEX IF EXISTS idx_api_tokens_active;
DROP TABLE IF EXISTS api_tokens;
