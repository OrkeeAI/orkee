-- ABOUTME: Rollback for soft delete support migration
-- ABOUTME: Removes deleted_at column from prds table

-- Note: SQLite does not support DROP COLUMN without recreating the table
-- The deleted_at column will remain in the prds table
-- This is documented for manual intervention if full rollback is required
