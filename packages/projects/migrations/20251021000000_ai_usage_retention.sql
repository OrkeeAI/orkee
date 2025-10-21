-- Add retention policy for AI usage logs (90 days)
-- This trigger automatically deletes logs older than 90 days after each insert
CREATE TRIGGER IF NOT EXISTS cleanup_old_ai_logs
AFTER INSERT ON ai_usage_logs
BEGIN
  DELETE FROM ai_usage_logs
  WHERE created_at < datetime('now', '-90 days');
END;
