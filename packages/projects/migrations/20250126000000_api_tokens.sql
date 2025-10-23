-- Add api_tokens table for API authentication
-- Migration: 20250126000000_api_tokens.sql

CREATE TABLE IF NOT EXISTS api_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    token_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    last_used_at TEXT,
    is_active INTEGER DEFAULT 1 NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_api_tokens_active ON api_tokens(is_active);
CREATE INDEX IF NOT EXISTS idx_api_tokens_hash ON api_tokens(token_hash);
