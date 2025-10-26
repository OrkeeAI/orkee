-- ABOUTME: Rollback migration for OpenSpec alignment changes
-- ABOUTME: Removes compliance tracking, validation status, and materialization tables

-- Drop indexes
DROP INDEX IF EXISTS idx_spec_materializations_path;
DROP INDEX IF EXISTS idx_spec_materializations_project;
DROP INDEX IF EXISTS idx_spec_capabilities_change;
DROP INDEX IF EXISTS idx_spec_changes_project_verb;

-- Drop materialization table
DROP TABLE IF EXISTS spec_materializations;

-- Remove columns from spec_changes (SQLite doesn't support DROP COLUMN directly for old versions)
-- For SQLite 3.35.0+ (2021-03-12), we can use DROP COLUMN
-- For older versions, a table recreation would be needed
-- Since this is a new migration, we assume modern SQLite
ALTER TABLE spec_changes DROP COLUMN validation_errors;
ALTER TABLE spec_changes DROP COLUMN validation_status;
ALTER TABLE spec_changes DROP COLUMN change_number;
ALTER TABLE spec_changes DROP COLUMN verb_prefix;

-- Remove columns from spec_capabilities
ALTER TABLE spec_capabilities DROP COLUMN is_openspec_compliant;
ALTER TABLE spec_capabilities DROP COLUMN change_id;
