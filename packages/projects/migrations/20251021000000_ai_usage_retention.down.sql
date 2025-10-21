-- ABOUTME: Rollback for AI usage logs retention trigger migration
-- ABOUTME: Drops the automatic cleanup trigger for old AI logs

-- Drop trigger
DROP TRIGGER IF EXISTS cleanup_old_ai_logs;
