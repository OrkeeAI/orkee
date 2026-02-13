-- Rollback: Remove agent_runs table and agent_executions columns

DROP TRIGGER IF EXISTS update_agent_runs_updated_at;
DROP INDEX IF EXISTS idx_agent_executions_run;
DROP INDEX IF EXISTS idx_agent_runs_status;
DROP INDEX IF EXISTS idx_agent_runs_project;

-- SQLite doesn't support DROP COLUMN before 3.35.0
-- For development resets, recreate agent_executions without the new columns
-- or simply drop and recreate the whole database

DROP TABLE IF EXISTS agent_runs;
