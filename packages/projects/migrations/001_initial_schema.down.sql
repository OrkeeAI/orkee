-- ABOUTME: Down migration for initial schema - drops all tables in reverse dependency order
-- ABOUTME: Used for development resets and integration test cleanup

-- ============================================================================
-- DROP VIEWS FIRST
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
