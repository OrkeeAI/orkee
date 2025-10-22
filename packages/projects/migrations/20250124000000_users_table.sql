-- Add missing columns to users table
-- Migration: 20250124000000_users_table.sql

-- Add avatar_url column
ALTER TABLE users ADD COLUMN avatar_url TEXT;

-- Add preferences column (JSON)
ALTER TABLE users ADD COLUMN preferences TEXT;

-- Add last_login_at column
ALTER TABLE users ADD COLUMN last_login_at TEXT;
