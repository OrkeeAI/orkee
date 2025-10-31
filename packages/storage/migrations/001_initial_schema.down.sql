-- ABOUTME: Down migration for consolidated initial schema
-- ABOUTME: Drops all tables, views, triggers, and indexes in reverse dependency order

-- ============================================================================
-- DROP TRIGGERS FIRST (before any tables)
-- ============================================================================
DROP TRIGGER IF EXISTS projects_updated_at;
DROP TRIGGER IF EXISTS projects_fts_insert;
DROP TRIGGER IF EXISTS projects_fts_delete;
DROP TRIGGER IF EXISTS projects_fts_update;
DROP TRIGGER IF EXISTS users_updated_at;
DROP TRIGGER IF EXISTS user_agents_updated_at;
DROP TRIGGER IF EXISTS tags_updated_at;
DROP TRIGGER IF EXISTS prds_updated_at;
DROP TRIGGER IF EXISTS tasks_updated_at;
DROP TRIGGER IF EXISTS tasks_fts_insert;
DROP TRIGGER IF EXISTS tasks_fts_delete;
DROP TRIGGER IF EXISTS tasks_fts_update;
DROP TRIGGER IF EXISTS agent_executions_updated_at;
DROP TRIGGER IF EXISTS pr_reviews_updated_at;
DROP TRIGGER IF EXISTS context_configurations_updated_at;
DROP TRIGGER IF EXISTS context_snapshots_updated_at;
DROP TRIGGER IF EXISTS context_usage_patterns_updated_at;
DROP TRIGGER IF EXISTS context_templates_updated_at;
DROP TRIGGER IF EXISTS api_tokens_updated_at;
DROP TRIGGER IF EXISTS update_task_completion_stats;
DROP TRIGGER IF EXISTS update_task_completion_stats_insert;
DROP TRIGGER IF EXISTS update_task_completion_stats_delete;
DROP TRIGGER IF EXISTS update_task_actual_hours;
DROP TRIGGER IF EXISTS update_task_on_pr_merge;
DROP TRIGGER IF EXISTS ideate_sessions_updated_at;
DROP TRIGGER IF EXISTS prd_output_templates_updated_at;
DROP TRIGGER IF EXISTS epics_updated_at;
DROP TRIGGER IF EXISTS github_sync_updated_at;

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
DROP TABLE IF EXISTS context_usage_patterns;
DROP TABLE IF EXISTS context_snapshots;
DROP TABLE IF EXISTS context_configurations;

-- ============================================================================
-- DROP AI USAGE TABLES
-- ============================================================================
DROP TABLE IF EXISTS ai_usage_logs;

-- ============================================================================

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

-- Conversational Mode (CCPM) tables
DROP TABLE IF EXISTS conversation_insights;
DROP TABLE IF EXISTS discovery_questions;
DROP TABLE IF EXISTS work_analysis;
DROP TABLE IF EXISTS github_sync;
DROP TABLE IF EXISTS epics;
DROP TABLE IF EXISTS prd_conversations;

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
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS users;

-- ============================================================================
-- DROP PROJECT TABLES
-- ============================================================================
DROP TABLE IF EXISTS projects_fts;
DROP TABLE IF EXISTS projects;
