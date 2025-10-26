-- ABOUTME: Down migration for initial schema - drops all tables in reverse dependency order
-- ABOUTME: Used for development resets and integration test cleanup

-- ============================================================================
-- PRE-DROP VERIFICATION
-- ============================================================================
-- Check for user data that will be lost (development safety check)
-- Note: These queries don't prevent the drop, they're for logging/verification
-- In production, you should backup data before running down migrations

-- Count records that will be deleted
-- SELECT 'Projects to delete: ' || COUNT(*) FROM projects WHERE 1=1;
-- SELECT 'Users to delete: ' || COUNT(*) FROM users WHERE id != 'default-user';
-- SELECT 'Tasks to delete: ' || COUNT(*) FROM tasks WHERE 1=1;

-- Check for orphaned data (data without proper FK relationships)
-- This helps identify data integrity issues before cleanup
-- SELECT 'Orphaned tasks (no project): ' || COUNT(*) FROM tasks
--   WHERE project_id NOT IN (SELECT id FROM projects);
-- SELECT 'Orphaned user_agents (no user): ' || COUNT(*) FROM user_agents
--   WHERE user_id NOT IN (SELECT id FROM users);

-- ============================================================================
-- DROP TRIGGERS FIRST (before any tables)
-- ============================================================================
-- Must drop triggers before dropping tables to avoid errors from trigger execution
-- Triggers may reference other tables that will be dropped

-- Drop all triggers (comprehensive list from up migration)
DROP TRIGGER IF EXISTS projects_updated_at;
DROP TRIGGER IF EXISTS projects_fts_insert;
DROP TRIGGER IF EXISTS projects_fts_delete;
DROP TRIGGER IF EXISTS projects_fts_update;
DROP TRIGGER IF EXISTS users_updated_at;
DROP TRIGGER IF EXISTS user_agents_updated_at;
DROP TRIGGER IF EXISTS tags_updated_at;
DROP TRIGGER IF EXISTS prds_updated_at;
DROP TRIGGER IF EXISTS spec_changes_updated_at;
DROP TRIGGER IF EXISTS spec_capabilities_updated_at;
DROP TRIGGER IF EXISTS spec_capabilities_history_updated_at;
DROP TRIGGER IF EXISTS spec_requirements_updated_at;
DROP TRIGGER IF EXISTS spec_scenarios_updated_at;
DROP TRIGGER IF EXISTS spec_change_tasks_updated_at;
DROP TRIGGER IF EXISTS spec_deltas_updated_at;
DROP TRIGGER IF EXISTS task_spec_links_updated_at;
DROP TRIGGER IF EXISTS prd_spec_sync_history_updated_at;
DROP TRIGGER IF EXISTS spec_materializations_updated_at;
DROP TRIGGER IF EXISTS tasks_updated_at;
DROP TRIGGER IF EXISTS tasks_fts_insert;
DROP TRIGGER IF EXISTS tasks_fts_delete;
DROP TRIGGER IF EXISTS tasks_fts_update;
DROP TRIGGER IF EXISTS agent_executions_updated_at;
DROP TRIGGER IF EXISTS pr_reviews_updated_at;
DROP TRIGGER IF EXISTS context_configurations_updated_at;
DROP TRIGGER IF EXISTS context_snapshots_updated_at;
DROP TRIGGER IF EXISTS context_usage_patterns_updated_at;
DROP TRIGGER IF EXISTS ast_spec_mappings_updated_at;
DROP TRIGGER IF EXISTS context_templates_updated_at;
DROP TRIGGER IF EXISTS api_tokens_updated_at;

-- ============================================================================
-- DROP VIEWS (after triggers)
-- ============================================================================
DROP VIEW IF EXISTS project_search;
DROP VIEW IF EXISTS projects_with_git;
DROP VIEW IF EXISTS active_projects;

-- ============================================================================
-- DROP TELEMETRY TABLES
-- ============================================================================
DROP TABLE IF EXISTS telemetry_stats;
DROP TABLE IF EXISTS telemetry_events;
DROP TABLE IF EXISTS telemetry_settings;

-- ============================================================================
-- DROP CLOUD SYNC TABLES
-- ============================================================================
DROP TABLE IF EXISTS sync_state;
DROP TABLE IF EXISTS sync_snapshots;

-- ============================================================================
-- DROP SYSTEM & SECURITY TABLES
-- ============================================================================
DROP TABLE IF EXISTS system_settings;
DROP TABLE IF EXISTS storage_metadata;
DROP TABLE IF EXISTS api_tokens;
DROP TABLE IF EXISTS password_attempts;
DROP TABLE IF EXISTS encryption_settings;

-- ============================================================================
-- DROP CONTEXT MANAGEMENT TABLES (reverse dependency order)
-- ============================================================================
DROP TABLE IF EXISTS context_templates;
DROP TABLE IF EXISTS ast_spec_mappings;
DROP TABLE IF EXISTS context_usage_patterns;
DROP TABLE IF EXISTS context_snapshots;
DROP TABLE IF EXISTS context_configurations;

-- ============================================================================
-- DROP AI USAGE TABLES
-- ============================================================================
DROP TABLE IF EXISTS ai_usage_logs;

-- ============================================================================
-- DROP OPENSPEC-TASK LINK TABLES
-- ============================================================================
DROP TABLE IF EXISTS spec_materializations;
DROP TABLE IF EXISTS prd_spec_sync_history;
DROP TABLE IF EXISTS task_spec_links;

-- ============================================================================
-- DROP AGENT EXECUTION TABLES
-- ============================================================================
DROP TABLE IF EXISTS pr_reviews;
DROP TABLE IF EXISTS agent_executions;

-- ============================================================================
-- DROP TASK TABLES
-- ============================================================================
DROP TABLE IF EXISTS tasks_fts;
DROP TABLE IF EXISTS tasks;

-- ============================================================================
-- DROP OPENSPEC TABLES (reverse dependency order)
-- ============================================================================
DROP TABLE IF EXISTS spec_deltas;
DROP TABLE IF EXISTS spec_change_tasks;
DROP TABLE IF EXISTS spec_scenarios;
DROP TABLE IF EXISTS spec_requirements;
DROP TABLE IF EXISTS spec_capabilities_history;
DROP TABLE IF EXISTS spec_capabilities;
DROP TABLE IF EXISTS spec_changes;
DROP TABLE IF EXISTS prds;

-- ============================================================================
-- DROP USER & AGENT TABLES
-- ============================================================================
DROP TABLE IF EXISTS user_agents;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS agents;
DROP TABLE IF EXISTS users;

-- ============================================================================
-- DROP PROJECT TABLES
-- ============================================================================
DROP TABLE IF EXISTS projects_fts;
DROP TABLE IF EXISTS projects;

-- ============================================================================
-- CLEAN STATE VERIFICATION
-- ============================================================================
-- Verify all tables have been dropped (except SQLx migration tracking)
-- Uncomment these queries to verify clean state during development:

-- List remaining tables (should only show _sqlx_migrations and sqlite_* tables)
-- SELECT name FROM sqlite_master
--   WHERE type='table'
--   AND name NOT LIKE 'sqlite_%'
--   AND name != '_sqlx_migrations'
--   ORDER BY name;

-- List remaining views (should be empty)
-- SELECT name FROM sqlite_master WHERE type='view' ORDER BY name;

-- List remaining triggers (should be empty)
-- SELECT name FROM sqlite_master WHERE type='trigger' ORDER BY name;

-- List remaining indexes (should only show indexes on _sqlx_migrations and sqlite_* tables)
-- SELECT name FROM sqlite_master
--   WHERE type='index'
--   AND name NOT LIKE 'sqlite_%'
--   AND tbl_name != '_sqlx_migrations'
--   ORDER BY name;

-- ============================================================================
-- MIGRATION METADATA CLEANUP (OPTIONAL)
-- ============================================================================
-- WARNING: Deleting from _sqlx_migrations will cause SQLx to re-run all migrations
-- Only uncomment this if you want to completely reset the migration state
-- This is useful for development but NEVER do this in production

-- DELETE FROM _sqlx_migrations WHERE version = 1;

-- To completely remove migration tracking (use with extreme caution):
-- DROP TABLE IF EXISTS _sqlx_migrations;
