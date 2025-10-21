-- ABOUTME: Rollback for security infrastructure migration
-- ABOUTME: Drops encryption settings and password attempts tables

-- Drop index
DROP INDEX IF EXISTS idx_encryption_settings_mode;

-- Drop tables
DROP TABLE IF EXISTS password_attempts;
DROP TABLE IF EXISTS encryption_settings;
