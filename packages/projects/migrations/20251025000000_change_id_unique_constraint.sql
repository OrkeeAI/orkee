-- ABOUTME: Add unique constraint to prevent race conditions in change ID generation
-- ABOUTME: Ensures atomicity of change number assignment per project and verb prefix

-- Add unique constraint to enforce atomic change ID generation
-- This prevents two concurrent transactions from creating changes with the same number
CREATE UNIQUE INDEX IF NOT EXISTS idx_spec_changes_unique_change_number
  ON spec_changes(project_id, verb_prefix, change_number)
  WHERE deleted_at IS NULL;
