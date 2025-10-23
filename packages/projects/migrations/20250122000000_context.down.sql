-- Rollback Context Tab Feature - Phase 1

DROP INDEX IF EXISTS idx_context_patterns_last_used;
DROP INDEX IF EXISTS idx_context_patterns_project;
DROP INDEX IF EXISTS idx_context_snapshots_config;
DROP INDEX IF EXISTS idx_context_snapshots_project;
DROP INDEX IF EXISTS idx_context_configs_project;

DROP TABLE IF EXISTS context_usage_patterns;
DROP TABLE IF EXISTS context_snapshots;
DROP TABLE IF EXISTS context_configurations;
