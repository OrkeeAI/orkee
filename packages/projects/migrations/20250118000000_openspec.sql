-- ABOUTME: OpenSpec integration for spec-driven development with PRDs, capabilities, requirements
-- ABOUTME: Includes AI usage tracking, soft delete support, performance indexes, and data retention

-- Product Requirements Documents
CREATE TABLE IF NOT EXISTS prds (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'approved', 'superseded')),
    source TEXT DEFAULT 'manual' CHECK(source IN ('manual', 'generated', 'synced')),
    deleted_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Spec Capabilities (equivalent to openspec/specs/[capability]/)
CREATE TABLE IF NOT EXISTS spec_capabilities (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL,
    prd_id TEXT,
    name TEXT NOT NULL,
    purpose_markdown TEXT,
    spec_markdown TEXT NOT NULL,
    design_markdown TEXT,
    requirement_count INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    status TEXT DEFAULT 'active' CHECK(status IN ('active', 'deprecated', 'archived')),
    deleted_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE SET NULL
);

-- Spec Capabilities History (version control for full change tracking)
CREATE TABLE IF NOT EXISTS spec_capabilities_history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    capability_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    spec_markdown TEXT NOT NULL,
    design_markdown TEXT,
    purpose_markdown TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (capability_id) REFERENCES spec_capabilities(id) ON DELETE CASCADE,
    UNIQUE (capability_id, version)
);

-- Individual Requirements within Capabilities
CREATE TABLE IF NOT EXISTS spec_requirements (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    capability_id TEXT NOT NULL,
    name TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    position INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (capability_id) REFERENCES spec_capabilities(id) ON DELETE CASCADE
);

-- Scenarios for Requirements (WHEN/THEN/AND)
CREATE TABLE IF NOT EXISTS spec_scenarios (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    requirement_id TEXT NOT NULL,
    name TEXT NOT NULL,
    when_clause TEXT NOT NULL,
    then_clause TEXT NOT NULL,
    and_clauses TEXT,
    position INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (requirement_id) REFERENCES spec_requirements(id) ON DELETE CASCADE
);

-- Change Proposals (equivalent to openspec/changes/[change-id]/)
CREATE TABLE IF NOT EXISTS spec_changes (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    prd_id TEXT,
    proposal_markdown TEXT NOT NULL,
    tasks_markdown TEXT NOT NULL,
    design_markdown TEXT,
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'review', 'approved', 'implementing', 'completed', 'archived')),
    created_by TEXT NOT NULL,
    approved_by TEXT,
    approved_at TIMESTAMP,
    archived_at TIMESTAMP,
    deleted_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE SET NULL
);

-- Spec Deltas (changes to capabilities)
CREATE TABLE IF NOT EXISTS spec_deltas (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    change_id TEXT NOT NULL,
    capability_id TEXT,
    capability_name TEXT NOT NULL,
    delta_type TEXT NOT NULL CHECK(delta_type IN ('added', 'modified', 'removed')),
    delta_markdown TEXT NOT NULL,
    position INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (change_id) REFERENCES spec_changes(id) ON DELETE CASCADE,
    FOREIGN KEY (capability_id) REFERENCES spec_capabilities(id) ON DELETE SET NULL
);

-- Task-Spec-Requirement Links
CREATE TABLE IF NOT EXISTS task_spec_links (
    task_id TEXT NOT NULL,
    requirement_id TEXT NOT NULL,
    scenario_id TEXT,
    validation_status TEXT DEFAULT 'pending' CHECK(validation_status IN ('pending', 'passed', 'failed')),
    validation_result TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (task_id, requirement_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (requirement_id) REFERENCES spec_requirements(id) ON DELETE CASCADE,
    FOREIGN KEY (scenario_id) REFERENCES spec_scenarios(id) ON DELETE SET NULL
);

-- PRD-Spec Sync History
CREATE TABLE IF NOT EXISTS prd_spec_sync_history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    prd_id TEXT NOT NULL,
    direction TEXT NOT NULL CHECK(direction IN ('prd_to_spec', 'spec_to_prd', 'task_to_spec')),
    changes_json TEXT NOT NULL,
    performed_by TEXT,
    performed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE CASCADE
);

-- AI Usage Tracking
CREATE TABLE IF NOT EXISTS ai_usage_logs (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL,
    request_id TEXT,
    operation TEXT NOT NULL,
    model TEXT NOT NULL,
    provider TEXT NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    estimated_cost REAL,
    duration_ms INTEGER,
    error TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Add spec-related columns to tasks table
ALTER TABLE tasks ADD COLUMN spec_driven BOOLEAN DEFAULT FALSE;
ALTER TABLE tasks ADD COLUMN change_id TEXT REFERENCES spec_changes(id);
ALTER TABLE tasks ADD COLUMN from_prd_id TEXT REFERENCES prds(id);
ALTER TABLE tasks ADD COLUMN spec_validation_status TEXT;
ALTER TABLE tasks ADD COLUMN spec_validation_result TEXT;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_prds_project ON prds(project_id);
CREATE INDEX IF NOT EXISTS idx_prds_status ON prds(status);
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_project ON spec_capabilities(project_id);
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_prd ON spec_capabilities(prd_id);
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_status ON spec_capabilities(status);
CREATE INDEX IF NOT EXISTS idx_spec_requirements_capability ON spec_requirements(capability_id);
CREATE INDEX IF NOT EXISTS idx_spec_scenarios_requirement ON spec_scenarios(requirement_id);
CREATE INDEX IF NOT EXISTS idx_spec_changes_project ON spec_changes(project_id);
CREATE INDEX IF NOT EXISTS idx_spec_changes_status ON spec_changes(status);
CREATE INDEX IF NOT EXISTS idx_spec_deltas_change ON spec_deltas(change_id);
CREATE INDEX IF NOT EXISTS idx_spec_deltas_capability ON spec_deltas(capability_id);
CREATE INDEX IF NOT EXISTS idx_task_spec_links_task ON task_spec_links(task_id);
CREATE INDEX IF NOT EXISTS idx_task_spec_links_requirement ON task_spec_links(requirement_id);
CREATE INDEX IF NOT EXISTS idx_prd_spec_sync_history_prd ON prd_spec_sync_history(prd_id);
CREATE INDEX IF NOT EXISTS idx_ai_usage_logs_project ON ai_usage_logs(project_id);
CREATE INDEX IF NOT EXISTS idx_ai_usage_logs_created ON ai_usage_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_ai_usage_logs_operation ON ai_usage_logs(operation);

-- Composite indexes for common multi-column queries
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_project_status ON spec_capabilities(project_id, status);
CREATE INDEX IF NOT EXISTS idx_ai_usage_logs_provider_model ON ai_usage_logs(provider, model);
CREATE INDEX IF NOT EXISTS idx_ai_usage_logs_provider_model_created ON ai_usage_logs(provider, model, created_at);
CREATE INDEX IF NOT EXISTS idx_spec_capabilities_history_capability_version ON spec_capabilities_history(capability_id, version);

-- Additional indexes for task_spec_links and ai_usage_logs
CREATE INDEX IF NOT EXISTS idx_task_spec_links_status ON task_spec_links(validation_status);
CREATE INDEX IF NOT EXISTS idx_task_spec_links_scenario ON task_spec_links(scenario_id);

-- Partial indexes only index rows where deleted_at IS NULL
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

-- Add retention policy for AI usage logs (90 days)
CREATE TRIGGER IF NOT EXISTS cleanup_old_ai_logs
AFTER INSERT ON ai_usage_logs
BEGIN
  DELETE FROM ai_usage_logs
  WHERE created_at < datetime('now', '-90 days');
END;
