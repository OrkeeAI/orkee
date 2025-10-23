-- Rollback Context-OpenSpec Integration - Phase 3

-- Drop indexes
DROP INDEX IF EXISTS idx_ai_usage_logs_context;
DROP INDEX IF EXISTS idx_context_configs_spec;
DROP INDEX IF EXISTS idx_ast_spec_mappings_requirement;
DROP INDEX IF EXISTS idx_ast_spec_mappings_project;

-- Remove context_snapshot_id column from ai_usage_logs
-- Note: SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
-- This is commented out as it requires complex table recreation
-- ALTER TABLE ai_usage_logs DROP COLUMN context_snapshot_id;

-- Drop tables
DROP TABLE IF EXISTS context_templates;
DROP TABLE IF EXISTS ast_spec_mappings;

-- Remove spec_capability_id column from context_configurations
-- Note: SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
-- This is commented out as it requires complex table recreation
-- ALTER TABLE context_configurations DROP COLUMN spec_capability_id;
