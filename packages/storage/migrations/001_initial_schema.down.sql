-- ABOUTME: Down migration for consolidated initial schema
-- ABOUTME: Drops all tables, views, triggers, and indexes in reverse dependency order

-- Temporarily disable foreign key constraints for clean down migration
PRAGMA foreign_keys = OFF;

-- ============================================================================
-- DROP TRIGGERS FIRST (before any tables)
-- ============================================================================
DROP TRIGGER IF EXISTS projects_updated_at;
DROP TRIGGER IF EXISTS projects_fts_insert;
DROP TRIGGER IF EXISTS projects_fts_delete;
DROP TRIGGER IF EXISTS projects_fts_update;
DROP TRIGGER IF EXISTS users_updated_at;
DROP TRIGGER IF EXISTS user_agents_updated_at;
DROP TRIGGER IF EXISTS prds_updated_at;
DROP TRIGGER IF EXISTS tasks_updated_at;
DROP TRIGGER IF EXISTS tasks_fts_insert;
DROP TRIGGER IF EXISTS tasks_fts_delete;
DROP TRIGGER IF EXISTS tasks_fts_update;
DROP TRIGGER IF EXISTS agent_executions_updated_at;
DROP TRIGGER IF EXISTS pr_reviews_updated_at;
DROP TRIGGER IF EXISTS update_task_actual_hours;
DROP TRIGGER IF EXISTS update_task_on_pr_merge;
DROP TRIGGER IF EXISTS ideate_sessions_updated_at;
DROP TRIGGER IF EXISTS prd_output_templates_updated_at;
DROP TRIGGER IF EXISTS epics_updated_at;
DROP TRIGGER IF EXISTS github_sync_updated_at;
DROP TRIGGER IF EXISTS encryption_settings_updated_at;
DROP TRIGGER IF EXISTS password_attempts_updated_at;
DROP TRIGGER IF EXISTS storage_metadata_updated_at;
DROP TRIGGER IF EXISTS sync_state_updated_at;
DROP TRIGGER IF EXISTS system_settings_updated_at;
DROP TRIGGER IF EXISTS update_telemetry_settings_timestamp;
DROP TRIGGER IF EXISTS ideate_prd_generations_updated_at;
DROP TRIGGER IF EXISTS model_preferences_updated_at;
DROP TRIGGER IF EXISTS sandbox_settings_updated_at;
DROP TRIGGER IF EXISTS sandbox_provider_settings_updated_at;
DROP TRIGGER IF EXISTS preview_servers_updated_at;

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
-- DROP PREVIEW SERVER TABLES
-- ============================================================================
DROP TABLE IF EXISTS preview_servers;

-- ============================================================================
-- DROP SANDBOX TABLES
-- ============================================================================
-- Drop execution-related tables (in reverse order due to foreign keys)
DROP TABLE IF EXISTS sandbox_volumes;
DROP TABLE IF EXISTS sandbox_env_vars;
DROP TABLE IF EXISTS sandbox_executions;
DROP TABLE IF EXISTS sandboxes;

-- Drop configuration tables
DROP TABLE IF EXISTS sandbox_provider_settings;
DROP TABLE IF EXISTS sandbox_settings;

-- ============================================================================
-- DROP SYSTEM & SECURITY TABLES
-- ============================================================================
DROP TABLE IF EXISTS system_settings;
DROP TABLE IF EXISTS storage_metadata;
DROP TABLE IF EXISTS api_tokens;
DROP TABLE IF EXISTS password_attempts;
DROP TABLE IF EXISTS encryption_settings;

-- Drop OAuth tables
DROP TABLE IF EXISTS oauth_tokens;

-- ============================================================================
-- DROP AI USAGE TABLES
-- ============================================================================
DROP TABLE IF EXISTS ai_usage_logs;

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
-- DROP IDEATE TABLES
-- ============================================================================
-- PRD Generation tables
DROP TABLE IF EXISTS ideate_generation_stats;
DROP TABLE IF EXISTS ideate_generations;
DROP TABLE IF EXISTS ideate_validation_rules;
DROP TABLE IF EXISTS ideate_section_generations;
DROP TABLE IF EXISTS ideate_exports;
DROP TABLE IF EXISTS ideate_prd_generations;

-- PRD Output Templates
DROP TABLE IF EXISTS prd_output_templates;

-- Dependency Intelligence tables
DROP TABLE IF EXISTS circular_dependencies;
DROP TABLE IF EXISTS quick_win_features;
DROP TABLE IF EXISTS build_order_optimization;
DROP TABLE IF EXISTS dependency_analysis_cache;
DROP TABLE IF EXISTS feature_dependencies;

-- Expert Roundtable tables
DROP TABLE IF EXISTS roundtable_insights;
DROP TABLE IF EXISTS expert_suggestions;
DROP TABLE IF EXISTS roundtable_messages;
DROP TABLE IF EXISTS roundtable_participants;
DROP TABLE IF EXISTS roundtable_sessions;
DROP TABLE IF EXISTS expert_personas;

-- Research Analysis Cache
DROP TABLE IF EXISTS competitor_analysis_cache;
DROP TABLE IF EXISTS complexity_analysis_cache;
DROP TABLE IF EXISTS codebase_context_cache;
DROP TABLE IF EXISTS validation_score_cache;

-- Chat Mode (CCPM) tables
DROP TABLE IF EXISTS chat_insights;

-- Phase 1 Enhancement tables
DROP TABLE IF EXISTS prd_validation_history;
DROP TABLE IF EXISTS discovery_sessions;
DROP TABLE IF EXISTS task_complexity_reports;

-- Phase 5 Enhancement tables
DROP TABLE IF EXISTS validation_entries;
DROP TABLE IF EXISTS execution_checkpoints;

DROP TABLE IF EXISTS discovery_questions;
DROP TABLE IF EXISTS work_analysis;
DROP TABLE IF EXISTS github_sync;
DROP TABLE IF EXISTS epics;
DROP TABLE IF EXISTS prd_chats;

-- Ideate core tables
DROP TABLE IF EXISTS prd_quickstart_templates;
DROP TABLE IF EXISTS ideate_research;
DROP TABLE IF EXISTS ideate_risks;
DROP TABLE IF EXISTS ideate_dependencies;
DROP TABLE IF EXISTS ideate_roadmap;
DROP TABLE IF EXISTS ideate_technical;
DROP TABLE IF EXISTS ideate_ux;
DROP TABLE IF EXISTS ideate_features;
DROP TABLE IF EXISTS ideate_overview;
DROP TABLE IF EXISTS ideate_sessions;

-- PRDs table (must be dropped after ideate_sessions due to FK)
DROP TABLE IF EXISTS prds;

-- ============================================================================
-- DROP USER & AGENT TABLES
-- ============================================================================
DROP TABLE IF EXISTS user_agents;
DROP TABLE IF EXISTS model_preferences;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS users;

-- ============================================================================
-- DROP PROJECT TABLES
-- ============================================================================
DROP TABLE IF EXISTS projects_fts;
DROP TABLE IF EXISTS projects;


-- Re-enable foreign key constraints after down migration
PRAGMA foreign_keys = ON;
