-- ABOUTME: Rollback for encryption settings table migration
-- ABOUTME: Drops encryption_settings table and its index

-- Drop index
DROP INDEX IF EXISTS idx_encryption_settings_mode;

-- Drop table
DROP TABLE IF EXISTS encryption_settings;
