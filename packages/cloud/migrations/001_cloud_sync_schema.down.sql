-- ABOUTME: Rollback for cloud sync schema migration
-- ABOUTME: Drops all cloud sync tables, views, triggers, and indexes

-- Drop views
DROP VIEW IF EXISTS sync_health_summary;
DROP VIEW IF EXISTS recent_snapshots;
DROP VIEW IF EXISTS active_providers;

-- Drop triggers
DROP TRIGGER IF EXISTS update_sync_statistics_updated_at;
DROP TRIGGER IF EXISTS update_cloud_sync_state_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_sync_statistics_provider_date;
DROP INDEX IF EXISTS idx_sync_operations_log_status;
DROP INDEX IF EXISTS idx_sync_operations_log_provider_date;
DROP INDEX IF EXISTS idx_sync_conflicts_project;
DROP INDEX IF EXISTS idx_sync_conflicts_status;
DROP INDEX IF EXISTS idx_cloud_snapshots_project_count;
DROP INDEX IF EXISTS idx_cloud_snapshots_size;
DROP INDEX IF EXISTS idx_cloud_snapshots_provider_created;

-- Drop tables (in reverse order of dependencies)
DROP TABLE IF EXISTS sync_statistics;
DROP TABLE IF EXISTS sync_operations_log;
DROP TABLE IF EXISTS sync_conflicts;
DROP TABLE IF EXISTS cloud_snapshots;
DROP TABLE IF EXISTS cloud_sync_state;
