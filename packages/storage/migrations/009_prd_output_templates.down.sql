-- ABOUTME: Rollback migration for PRD output templates
-- ABOUTME: Drops template table, indexes, and trigger

-- Drop trigger first
DROP TRIGGER IF EXISTS prd_output_templates_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_prd_output_templates_name;
DROP INDEX IF EXISTS idx_prd_output_templates_default;
DROP INDEX IF EXISTS idx_prd_output_templates_created;

-- Drop table
DROP TABLE IF EXISTS prd_output_templates;
