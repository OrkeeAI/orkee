-- ABOUTME: Rollback for OpenSpec soft delete partial indexes migration
-- ABOUTME: Drops all partial indexes created for deleted_at IS NULL optimization

-- Drop composite partial indexes
DROP INDEX IF EXISTS idx_prds_project_not_deleted;
DROP INDEX IF EXISTS idx_spec_changes_project_not_deleted;
DROP INDEX IF EXISTS idx_spec_capabilities_prd_not_deleted;
DROP INDEX IF EXISTS idx_spec_capabilities_project_not_deleted;

-- Drop simple partial indexes
DROP INDEX IF EXISTS idx_prds_not_deleted;
DROP INDEX IF EXISTS idx_spec_changes_not_deleted;
DROP INDEX IF EXISTS idx_spec_capabilities_not_deleted;
