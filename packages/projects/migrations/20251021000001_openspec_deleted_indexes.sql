-- ABOUTME: Partial indexes for soft delete queries on OpenSpec tables
-- ABOUTME: Optimizes WHERE deleted_at IS NULL filters to avoid full table scans

-- Partial indexes only index rows where deleted_at IS NULL
-- This is much more efficient for soft delete queries than a full column index
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_not_deleted
  ON spec_capabilities(id) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_spec_changes_not_deleted
  ON spec_changes(id) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_prds_not_deleted
  ON prds(id) WHERE deleted_at IS NULL;

-- Composite partial indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_project_not_deleted
  ON spec_capabilities(project_id, status) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_spec_capabilities_prd_not_deleted
  ON spec_capabilities(prd_id) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_spec_changes_project_not_deleted
  ON spec_changes(project_id, status) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_prds_project_not_deleted
  ON prds(project_id, status) WHERE deleted_at IS NULL;
