-- ABOUTME: Rollback for initial schema migration
-- ABOUTME: Drops all tables, indexes, triggers, and views created in 001_initial_schema.sql

-- Drop views first (they depend on tables)
DROP VIEW IF EXISTS project_search;
DROP VIEW IF EXISTS projects_with_git;
DROP VIEW IF EXISTS active_projects;

-- Drop triggers
DROP TRIGGER IF EXISTS projects_updated_at;
DROP TRIGGER IF EXISTS projects_fts_delete;
DROP TRIGGER IF EXISTS projects_fts_update;
DROP TRIGGER IF EXISTS projects_fts_insert;

-- Drop FTS table
DROP TABLE IF EXISTS projects_fts;

-- Drop indexes
DROP INDEX IF EXISTS idx_projects_status_rank;
DROP INDEX IF EXISTS idx_projects_status_priority;
DROP INDEX IF EXISTS idx_projects_task_source;
DROP INDEX IF EXISTS idx_projects_created_at;
DROP INDEX IF EXISTS idx_projects_updated_at;
DROP INDEX IF EXISTS idx_projects_rank;
DROP INDEX IF EXISTS idx_projects_priority;
DROP INDEX IF EXISTS idx_projects_status;
DROP INDEX IF EXISTS idx_projects_name;

-- Drop sync tables
DROP INDEX IF EXISTS idx_sync_state_last_sync;
DROP INDEX IF EXISTS idx_sync_state_user_device;
DROP TABLE IF EXISTS sync_state;

DROP INDEX IF EXISTS idx_sync_snapshots_status;
DROP INDEX IF EXISTS idx_sync_snapshots_created_at;
DROP TABLE IF EXISTS sync_snapshots;

-- Drop metadata table
DROP TABLE IF EXISTS storage_metadata;

-- Drop main projects table
DROP TABLE IF EXISTS projects;
