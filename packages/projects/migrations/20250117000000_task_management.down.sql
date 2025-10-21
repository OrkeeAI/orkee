-- ABOUTME: Rollback for task management system migration
-- ABOUTME: Drops all task management tables, indexes, and triggers

-- Drop triggers
DROP TRIGGER IF EXISTS update_task_on_pr_merge;
DROP TRIGGER IF EXISTS update_task_actual_hours;
DROP TRIGGER IF EXISTS pr_reviews_updated_at;
DROP TRIGGER IF EXISTS agent_executions_updated_at;
DROP TRIGGER IF EXISTS tasks_fts_update;
DROP TRIGGER IF EXISTS tasks_fts_delete;
DROP TRIGGER IF EXISTS tasks_fts_insert;

-- Drop FTS table for tasks
DROP TABLE IF EXISTS tasks_fts;

-- Drop indexes
DROP INDEX IF EXISTS idx_pr_reviews_status;
DROP INDEX IF EXISTS idx_pr_reviews_reviewer_id;
DROP INDEX IF EXISTS idx_pr_reviews_execution_id;
DROP INDEX IF EXISTS idx_agent_executions_pr_number;
DROP INDEX IF EXISTS idx_agent_executions_status;
DROP INDEX IF EXISTS idx_agent_executions_agent_id;
DROP INDEX IF EXISTS idx_agent_executions_task_id;
DROP INDEX IF EXISTS idx_tasks_tag_id;
DROP INDEX IF EXISTS idx_tasks_created_by_user_id;
DROP INDEX IF EXISTS idx_tasks_assigned_agent_id;
DROP INDEX IF EXISTS idx_tasks_status;
DROP INDEX IF EXISTS idx_tasks_parent_id;
DROP INDEX IF EXISTS idx_tasks_project_id;
DROP INDEX IF EXISTS idx_tags_archived_at;

-- Drop tables
DROP TABLE IF EXISTS pr_reviews;
DROP TABLE IF EXISTS agent_executions;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS user_agents;
DROP TABLE IF EXISTS agents;
DROP TABLE IF EXISTS users;
