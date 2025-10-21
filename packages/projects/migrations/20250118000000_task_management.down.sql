-- ABOUTME: Rollback for task management schema migration
-- ABOUTME: Drops tasks, agents, users, and user_agents tables with their dependencies

-- Drop triggers for tasks FTS
DROP TRIGGER IF EXISTS tasks_fts_update;
DROP TRIGGER IF EXISTS tasks_fts_delete;
DROP TRIGGER IF EXISTS tasks_fts_insert;

-- Drop FTS table for tasks
DROP TABLE IF EXISTS tasks_fts;

-- Drop indexes for tasks
DROP INDEX IF EXISTS idx_tasks_created_by_user_id;
DROP INDEX IF EXISTS idx_tasks_assigned_agent_id;
DROP INDEX IF EXISTS idx_tasks_status;
DROP INDEX IF EXISTS idx_tasks_parent_id;
DROP INDEX IF EXISTS idx_tasks_project_id;

-- Drop tasks table (cascade will remove foreign key dependencies)
DROP TABLE IF EXISTS tasks;

-- Drop user_agents table
DROP TABLE IF EXISTS user_agents;

-- Drop agents table
DROP TABLE IF EXISTS agents;

-- Drop users table
DROP TABLE IF EXISTS users;
