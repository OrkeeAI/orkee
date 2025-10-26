-- ABOUTME: Rollback migration for change ID unique constraint
-- ABOUTME: Removes the unique index on (project_id, verb_prefix, change_number)

DROP INDEX IF EXISTS idx_spec_changes_unique_change_number;
