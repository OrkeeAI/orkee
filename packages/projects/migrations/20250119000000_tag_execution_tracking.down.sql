-- ABOUTME: Rollback for tag execution tracking migration
-- ABOUTME: Drops tags, agent_executions, pr_reviews tables and removes tag_id from tasks

-- Drop triggers
DROP TRIGGER IF EXISTS update_task_on_pr_merge;
DROP TRIGGER IF EXISTS update_task_actual_hours;
DROP TRIGGER IF EXISTS pr_reviews_updated_at;
DROP TRIGGER IF EXISTS agent_executions_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_pr_reviews_status;
DROP INDEX IF EXISTS idx_pr_reviews_reviewer_id;
DROP INDEX IF EXISTS idx_pr_reviews_execution_id;
DROP INDEX IF EXISTS idx_agent_executions_pr_number;
DROP INDEX IF EXISTS idx_agent_executions_status;
DROP INDEX IF EXISTS idx_agent_executions_agent_id;
DROP INDEX IF EXISTS idx_agent_executions_task_id;
DROP INDEX IF EXISTS idx_tasks_tag_id;
DROP INDEX IF EXISTS idx_tags_archived_at;

-- Drop tables
DROP TABLE IF EXISTS pr_reviews;
DROP TABLE IF EXISTS agent_executions;
DROP TABLE IF EXISTS tags;

-- Note: Cannot use ALTER TABLE DROP COLUMN in SQLite without recreating table
-- The tag_id column will remain in tasks table as SQLite doesn't support DROP COLUMN
-- If full rollback is needed, the tasks table would need to be recreated without tag_id
-- This is documented for manual intervention if required
