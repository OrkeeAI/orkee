-- ABOUTME: Rollback for AI gateway configuration migration
-- ABOUTME: Removes AI gateway columns from users table

-- Note: SQLite does not support DROP COLUMN without recreating the table
-- The columns ai_gateway_enabled, ai_gateway_url, ai_gateway_key will remain
-- This is documented for manual intervention if full rollback is required
--
-- For full rollback, the users table would need to be recreated without these columns
