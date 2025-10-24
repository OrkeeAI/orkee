-- ABOUTME: OpenSpec alignment migration - adds change management and materialization tracking
-- ABOUTME: Enhances existing OpenSpec schema with compliance tracking, validation, and file export support

-- Add OpenSpec compliance tracking to capabilities
ALTER TABLE spec_capabilities ADD COLUMN change_id TEXT REFERENCES spec_changes(id);
ALTER TABLE spec_capabilities ADD COLUMN is_openspec_compliant BOOLEAN DEFAULT FALSE;

-- Add change ID generation support to spec_changes
ALTER TABLE spec_changes ADD COLUMN verb_prefix TEXT;
ALTER TABLE spec_changes ADD COLUMN change_number INTEGER;

-- Add validation status to spec_changes
ALTER TABLE spec_changes ADD COLUMN validation_status TEXT DEFAULT 'pending' CHECK(validation_status IN ('pending', 'valid', 'invalid'));
ALTER TABLE spec_changes ADD COLUMN validation_errors TEXT;

-- Create materialization tracking table
CREATE TABLE IF NOT EXISTS spec_materializations (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL,
    path TEXT NOT NULL,
    materialized_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sha256_hash TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_spec_changes_project_verb ON spec_changes(project_id, verb_prefix);
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_change ON spec_capabilities(change_id);
CREATE INDEX IF NOT EXISTS idx_spec_materializations_project ON spec_materializations(project_id);
CREATE INDEX IF NOT EXISTS idx_spec_materializations_path ON spec_materializations(project_id, path);
