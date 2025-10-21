-- ABOUTME: Rollback for OpenSpec integration migration
-- ABOUTME: Drops all OpenSpec tables, indexes, triggers, and partial indexes

-- Drop trigger
DROP TRIGGER IF EXISTS cleanup_old_ai_logs;

-- Drop composite partial indexes
DROP INDEX IF EXISTS idx_prds_project_not_deleted;
DROP INDEX IF EXISTS idx_spec_changes_project_not_deleted;
DROP INDEX IF EXISTS idx_spec_capabilities_prd_not_deleted;
DROP INDEX IF EXISTS idx_spec_capabilities_project_not_deleted;

-- Drop simple partial indexes
DROP INDEX IF EXISTS idx_prds_not_deleted;
DROP INDEX IF EXISTS idx_spec_changes_not_deleted;
DROP INDEX IF EXISTS idx_spec_capabilities_not_deleted;

-- Drop composite indexes
DROP INDEX IF EXISTS idx_spec_capabilities_history_capability_version;
DROP INDEX IF EXISTS idx_ai_usage_logs_provider_model_created;
DROP INDEX IF EXISTS idx_ai_usage_logs_provider_model;
DROP INDEX IF EXISTS idx_spec_capabilities_project_status;

-- Drop single-column indexes
DROP INDEX IF EXISTS idx_task_spec_links_scenario;
DROP INDEX IF EXISTS idx_task_spec_links_status;
DROP INDEX IF EXISTS idx_ai_usage_logs_operation;
DROP INDEX IF EXISTS idx_ai_usage_logs_created;
DROP INDEX IF EXISTS idx_ai_usage_logs_project;
DROP INDEX IF EXISTS idx_prd_spec_sync_history_prd;
DROP INDEX IF EXISTS idx_task_spec_links_requirement;
DROP INDEX IF EXISTS idx_task_spec_links_task;
DROP INDEX IF EXISTS idx_spec_deltas_capability;
DROP INDEX IF EXISTS idx_spec_deltas_change;
DROP INDEX IF EXISTS idx_spec_changes_status;
DROP INDEX IF EXISTS idx_spec_changes_project;
DROP INDEX IF EXISTS idx_spec_scenarios_requirement;
DROP INDEX IF EXISTS idx_spec_requirements_capability;
DROP INDEX IF EXISTS idx_spec_capabilities_status;
DROP INDEX IF EXISTS idx_spec_capabilities_prd;
DROP INDEX IF EXISTS idx_spec_capabilities_project;
DROP INDEX IF EXISTS idx_prds_status;
DROP INDEX IF EXISTS idx_prds_project;

-- Drop tables (in reverse order of dependencies)
DROP TABLE IF EXISTS prd_spec_sync_history;
DROP TABLE IF EXISTS task_spec_links;
DROP TABLE IF EXISTS spec_deltas;
DROP TABLE IF EXISTS spec_changes;
DROP TABLE IF EXISTS spec_scenarios;
DROP TABLE IF EXISTS spec_requirements;
DROP TABLE IF EXISTS spec_capabilities_history;
DROP TABLE IF EXISTS spec_capabilities;
DROP TABLE IF EXISTS ai_usage_logs;
DROP TABLE IF EXISTS prds;
