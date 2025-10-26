-- ABOUTME: Initial Orkee database schema with project management, task tracking, OpenSpec, and telemetry
-- ABOUTME: Includes security, context management, API tokens, and comprehensive indexing for performance

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- CORE PROJECT MANAGEMENT
-- ============================================================================

-- Projects table - main entity
CREATE TABLE projects (
    -- Core fields
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    project_root TEXT NOT NULL UNIQUE,
    description TEXT,

    -- Status and priority
    status TEXT NOT NULL DEFAULT 'planning' CHECK (status IN ('planning', 'building', 'review', 'launched', 'on-hold', 'archived')),
    priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
    rank INTEGER,

    -- Scripts (nullable)
    setup_script TEXT,
    dev_script TEXT,
    cleanup_script TEXT,

    -- Task configuration
    task_source TEXT CHECK (task_source IN ('manual', 'taskmaster')),

    -- Complex data as JSON (nullable)
    tags TEXT, -- JSON array of strings
    manual_tasks TEXT, -- JSON array of task objects
    mcp_servers TEXT, -- JSON array of MCP server configs
    git_repository TEXT, -- JSON object with repo info

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    -- Constraints
    CHECK (json_valid(tags) OR tags IS NULL),
    CHECK (json_valid(manual_tasks) OR manual_tasks IS NULL),
    CHECK (json_valid(mcp_servers) OR mcp_servers IS NULL),
    CHECK (json_valid(git_repository) OR git_repository IS NULL)
);

-- Project indexes
CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_priority ON projects(priority);
CREATE INDEX idx_projects_rank ON projects(rank);
CREATE INDEX idx_projects_updated_at ON projects(updated_at);
CREATE INDEX idx_projects_created_at ON projects(created_at);
CREATE INDEX idx_projects_task_source ON projects(task_source);
CREATE INDEX idx_projects_status_priority ON projects(status, priority);
CREATE INDEX idx_projects_status_rank ON projects(status, rank);

-- Project full-text search
CREATE VIRTUAL TABLE projects_fts USING fts5(
    id UNINDEXED,
    name,
    description,
    project_root,
    tags,
    content='projects',
    content_rowid='rowid'
);

-- Project FTS triggers
CREATE TRIGGER projects_fts_insert AFTER INSERT ON projects BEGIN
    INSERT INTO projects_fts(rowid, id, name, description, project_root, tags)
    VALUES (new.rowid, new.id, new.name, new.description, new.project_root, new.tags);
END;

CREATE TRIGGER projects_fts_update AFTER UPDATE ON projects BEGIN
    INSERT INTO projects_fts(projects_fts, rowid, id, name, description, project_root, tags)
    VALUES ('delete', old.rowid, old.id, old.name, old.description, old.project_root, old.tags);
    INSERT INTO projects_fts(rowid, id, name, description, project_root, tags)
    VALUES (new.rowid, new.id, new.name, new.description, new.project_root, new.tags);
END;

CREATE TRIGGER projects_fts_delete AFTER DELETE ON projects BEGIN
    INSERT INTO projects_fts(projects_fts, rowid, id, name, description, project_root, tags)
    VALUES ('delete', old.rowid, old.id, old.name, old.description, old.project_root, old.tags);
END;

-- Project updated_at trigger
CREATE TRIGGER projects_updated_at AFTER UPDATE ON projects
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE projects SET updated_at = datetime('now', 'utc') WHERE id = NEW.id;
END;

-- Project views
CREATE VIEW active_projects AS
SELECT * FROM projects
WHERE status IN ('planning', 'building', 'review')
ORDER BY
    CASE WHEN rank IS NULL THEN 1 ELSE 0 END,
    rank ASC,
    name ASC;

CREATE VIEW projects_with_git AS
SELECT
    *,
    json_extract(git_repository, '$.owner') as git_owner,
    json_extract(git_repository, '$.repo') as git_repo,
    json_extract(git_repository, '$.branch') as git_branch
FROM projects
WHERE git_repository IS NOT NULL;

CREATE VIEW project_search AS
SELECT
    p.*,
    fts.rank as search_rank
FROM projects p
JOIN projects_fts fts ON p.rowid = fts.rowid;

-- ============================================================================
-- USER & AGENT MANAGEMENT
-- ============================================================================

-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    default_agent_id TEXT,
    theme TEXT DEFAULT 'system',
    openai_api_key TEXT,
    anthropic_api_key TEXT,
    google_api_key TEXT,
    xai_api_key TEXT,
    ai_gateway_enabled INTEGER DEFAULT 0,
    ai_gateway_url TEXT,
    ai_gateway_key TEXT,
    avatar_url TEXT,
    preferences TEXT,
    last_login_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Agents table
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT,
    display_name TEXT NOT NULL,
    avatar_url TEXT,
    description TEXT,
    capabilities TEXT,
    languages TEXT,
    frameworks TEXT,
    max_context_tokens INTEGER,
    supports_tools INTEGER NOT NULL DEFAULT 0,
    supports_vision INTEGER NOT NULL DEFAULT 0,
    supports_web_search INTEGER NOT NULL DEFAULT 0,
    api_endpoint TEXT,
    temperature REAL,
    max_tokens INTEGER,
    system_prompt TEXT,
    cost_per_1k_input_tokens REAL,
    cost_per_1k_output_tokens REAL,
    is_available INTEGER NOT NULL DEFAULT 1,
    requires_api_key INTEGER NOT NULL DEFAULT 1,
    metadata TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- User-Agent association table
CREATE TABLE user_agents (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    is_active INTEGER NOT NULL DEFAULT 1,
    custom_settings TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(user_id, agent_id)
);

-- ============================================================================
-- TASK MANAGEMENT
-- ============================================================================

-- Tags table
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    description TEXT,
    created_at TEXT NOT NULL,
    archived_at TEXT
);

CREATE INDEX idx_tags_archived_at ON tags(archived_at);

-- ============================================================================
-- OPENSPEC INTEGRATION
-- ============================================================================

-- Product Requirements Documents
CREATE TABLE prds (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'approved', 'superseded')),
    source TEXT DEFAULT 'manual' CHECK(source IN ('manual', 'generated', 'synced')),
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    created_by TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_prds_project ON prds(project_id);
CREATE INDEX idx_prds_status ON prds(status);
CREATE INDEX idx_prds_not_deleted ON prds(id) WHERE deleted_at IS NULL;
CREATE INDEX idx_prds_project_not_deleted ON prds(project_id, status) WHERE deleted_at IS NULL;

-- Spec Changes
CREATE TABLE spec_changes (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    prd_id TEXT,
    proposal_markdown TEXT NOT NULL,
    tasks_markdown TEXT NOT NULL,
    design_markdown TEXT,
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'review', 'approved', 'implementing', 'completed', 'archived')),
    created_by TEXT NOT NULL,
    approved_by TEXT,
    approved_at TEXT,
    archived_at TEXT,
    deleted_at TEXT,
    verb_prefix TEXT,
    change_number INTEGER,
    validation_status TEXT DEFAULT 'pending' CHECK(validation_status IN ('pending', 'valid', 'invalid')),
    validation_errors TEXT,
    tasks_completion_percentage INTEGER DEFAULT 0 CHECK(tasks_completion_percentage >= 0 AND tasks_completion_percentage <= 100),
    tasks_parsed_at TEXT,
    tasks_total_count INTEGER DEFAULT 0,
    tasks_completed_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE SET NULL
);

CREATE INDEX idx_spec_changes_project ON spec_changes(project_id);
CREATE INDEX idx_spec_changes_status ON spec_changes(status);
CREATE INDEX idx_spec_changes_project_verb ON spec_changes(project_id, verb_prefix);
CREATE INDEX idx_spec_changes_not_deleted ON spec_changes(id) WHERE deleted_at IS NULL;
CREATE INDEX idx_spec_changes_project_not_deleted ON spec_changes(project_id, status) WHERE deleted_at IS NULL;

-- Unique constraint for change ID generation
CREATE UNIQUE INDEX idx_spec_changes_unique_change_number
  ON spec_changes(project_id, verb_prefix, change_number)
  WHERE deleted_at IS NULL;

-- Spec Capabilities
CREATE TABLE spec_capabilities (
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
    change_id TEXT REFERENCES spec_changes(id),
    is_openspec_compliant BOOLEAN DEFAULT FALSE,
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE SET NULL
);

CREATE INDEX idx_spec_capabilities_project ON spec_capabilities(project_id);
CREATE INDEX idx_spec_capabilities_prd ON spec_capabilities(prd_id);
CREATE INDEX idx_spec_capabilities_status ON spec_capabilities(status);
CREATE INDEX idx_spec_capabilities_project_status ON spec_capabilities(project_id, status);
CREATE INDEX idx_spec_capabilities_change ON spec_capabilities(change_id);
CREATE INDEX idx_spec_capabilities_not_deleted ON spec_capabilities(id) WHERE deleted_at IS NULL;
CREATE INDEX idx_spec_capabilities_project_not_deleted ON spec_capabilities(project_id, status) WHERE deleted_at IS NULL;
CREATE INDEX idx_spec_capabilities_prd_not_deleted ON spec_capabilities(prd_id) WHERE deleted_at IS NULL;

-- Spec Capabilities History
CREATE TABLE spec_capabilities_history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    capability_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    spec_markdown TEXT NOT NULL,
    design_markdown TEXT,
    purpose_markdown TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (capability_id) REFERENCES spec_capabilities(id) ON DELETE CASCADE,
    UNIQUE (capability_id, version)
);

CREATE INDEX idx_spec_capabilities_history_capability_version ON spec_capabilities_history(capability_id, version);

-- Spec Requirements
CREATE TABLE spec_requirements (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    capability_id TEXT NOT NULL,
    name TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    position INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (capability_id) REFERENCES spec_capabilities(id) ON DELETE CASCADE
);

CREATE INDEX idx_spec_requirements_capability ON spec_requirements(capability_id);

-- Spec Scenarios
CREATE TABLE spec_scenarios (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    requirement_id TEXT NOT NULL,
    name TEXT NOT NULL,
    when_clause TEXT NOT NULL,
    then_clause TEXT NOT NULL,
    and_clauses TEXT,
    position INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (requirement_id) REFERENCES spec_requirements(id) ON DELETE CASCADE
);

CREATE INDEX idx_spec_scenarios_requirement ON spec_scenarios(requirement_id);

-- Spec Change Tasks
CREATE TABLE spec_change_tasks (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    change_id TEXT NOT NULL,
    task_number TEXT NOT NULL,
    task_text TEXT NOT NULL,
    is_completed BOOLEAN DEFAULT FALSE NOT NULL,
    completed_by TEXT,
    completed_at TEXT,
    display_order INTEGER NOT NULL,
    parent_number TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (change_id) REFERENCES spec_changes(id) ON DELETE CASCADE
);

CREATE INDEX idx_spec_change_tasks_change ON spec_change_tasks(change_id, display_order);
CREATE INDEX idx_spec_change_tasks_completion ON spec_change_tasks(change_id, is_completed);
CREATE INDEX idx_spec_change_tasks_parent ON spec_change_tasks(change_id, parent_number);

-- Spec change task triggers
CREATE TRIGGER update_task_completion_stats
AFTER UPDATE OF is_completed ON spec_change_tasks
BEGIN
    UPDATE spec_changes
    SET
        tasks_completed_count = (
            SELECT COUNT(*) FROM spec_change_tasks
            WHERE change_id = NEW.change_id AND is_completed = TRUE
        ),
        tasks_total_count = (
            SELECT COUNT(*) FROM spec_change_tasks
            WHERE change_id = NEW.change_id
        ),
        tasks_completion_percentage = (
            SELECT CAST(ROUND((COUNT(CASE WHEN is_completed THEN 1 END) * 100.0) / NULLIF(COUNT(*), 0)) AS INTEGER)
            FROM spec_change_tasks
            WHERE change_id = NEW.change_id
        ),
        updated_at = datetime('now', 'utc')
    WHERE id = NEW.change_id;
END;

CREATE TRIGGER update_task_completion_stats_insert
AFTER INSERT ON spec_change_tasks
BEGIN
    UPDATE spec_changes
    SET
        tasks_total_count = (
            SELECT COUNT(*) FROM spec_change_tasks
            WHERE change_id = NEW.change_id
        ),
        tasks_completion_percentage = (
            SELECT CAST(ROUND((COUNT(CASE WHEN is_completed THEN 1 END) * 100.0) / NULLIF(COUNT(*), 0)) AS INTEGER)
            FROM spec_change_tasks
            WHERE change_id = NEW.change_id
        ),
        updated_at = datetime('now', 'utc')
    WHERE id = NEW.change_id;
END;

CREATE TRIGGER update_task_completion_stats_delete
AFTER DELETE ON spec_change_tasks
BEGIN
    UPDATE spec_changes
    SET
        tasks_total_count = (
            SELECT COUNT(*) FROM spec_change_tasks
            WHERE change_id = OLD.change_id
        ),
        tasks_completed_count = (
            SELECT COUNT(*) FROM spec_change_tasks
            WHERE change_id = OLD.change_id AND is_completed = TRUE
        ),
        tasks_completion_percentage = (
            SELECT CAST(ROUND((COUNT(CASE WHEN is_completed THEN 1 END) * 100.0) / NULLIF(COUNT(*), 0)) AS INTEGER)
            FROM spec_change_tasks
            WHERE change_id = OLD.change_id
        ),
        updated_at = datetime('now', 'utc')
    WHERE id = OLD.change_id;
END;

-- Spec Deltas
CREATE TABLE spec_deltas (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    change_id TEXT NOT NULL,
    capability_id TEXT,
    capability_name TEXT NOT NULL,
    delta_type TEXT NOT NULL CHECK(delta_type IN ('added', 'modified', 'removed')),
    delta_markdown TEXT NOT NULL,
    position INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (change_id) REFERENCES spec_changes(id) ON DELETE CASCADE,
    FOREIGN KEY (capability_id) REFERENCES spec_capabilities(id) ON DELETE SET NULL
);

CREATE INDEX idx_spec_deltas_change ON spec_deltas(change_id);
CREATE INDEX idx_spec_deltas_capability ON spec_deltas(capability_id);

-- ============================================================================
-- TASK MANAGEMENT (CONTINUED - AFTER OPENSPEC)
-- ============================================================================

-- Tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    priority TEXT NOT NULL DEFAULT 'medium',
    created_by_user_id TEXT NOT NULL DEFAULT 'default-user',
    assigned_agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    reviewed_by_agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    parent_id TEXT REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id TEXT REFERENCES tags(id) ON DELETE SET NULL,
    position INTEGER NOT NULL DEFAULT 0,
    dependencies TEXT,
    blockers TEXT,
    due_date TEXT,
    estimated_hours REAL,
    actual_hours REAL,
    complexity_score INTEGER,
    details TEXT,
    test_strategy TEXT,
    acceptance_criteria TEXT,
    prompt TEXT,
    context TEXT,
    output_format TEXT,
    validation_rules TEXT,
    started_at TEXT,
    completed_at TEXT,
    execution_log TEXT,
    error_log TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    tags TEXT,
    category TEXT,
    metadata TEXT,
    spec_driven BOOLEAN DEFAULT FALSE,
    change_id TEXT REFERENCES spec_changes(id),
    from_prd_id TEXT REFERENCES prds(id),
    spec_validation_status TEXT,
    spec_validation_result TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Task indexes
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_parent_id ON tasks(parent_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assigned_agent_id ON tasks(assigned_agent_id);
CREATE INDEX idx_tasks_created_by_user_id ON tasks(created_by_user_id);
CREATE INDEX idx_tasks_tag_id ON tasks(tag_id);

-- Task full-text search
CREATE VIRTUAL TABLE tasks_fts USING fts5(
    task_id UNINDEXED,
    title,
    description,
    details,
    tags,
    content='tasks',
    content_rowid='rowid'
);

-- Task FTS triggers
CREATE TRIGGER tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, task_id, title, description, details, tags)
    VALUES (new.rowid, new.id, new.title, new.description, new.details, new.tags);
END;

CREATE TRIGGER tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER tasks_fts_update AFTER UPDATE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.rowid;
    INSERT INTO tasks_fts(rowid, task_id, title, description, details, tags)
    VALUES (new.rowid, new.id, new.title, new.description, new.details, new.tags);
END;

-- ============================================================================
-- AGENT EXECUTION TRACKING
-- ============================================================================

-- Agent executions table
CREATE TABLE agent_executions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    model TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    execution_time_seconds INTEGER,
    tokens_input INTEGER,
    tokens_output INTEGER,
    total_cost REAL,
    prompt TEXT,
    response TEXT,
    error_message TEXT,
    retry_attempt INTEGER NOT NULL DEFAULT 0,
    files_changed INTEGER DEFAULT 0,
    lines_added INTEGER DEFAULT 0,
    lines_removed INTEGER DEFAULT 0,
    files_created TEXT,
    files_modified TEXT,
    files_deleted TEXT,
    branch_name TEXT,
    commit_hash TEXT,
    commit_message TEXT,
    pr_number INTEGER,
    pr_url TEXT,
    pr_title TEXT,
    pr_status TEXT,
    pr_created_at TEXT,
    pr_merged_at TEXT,
    pr_merge_commit TEXT,
    review_status TEXT,
    review_comments INTEGER DEFAULT 0,
    test_results TEXT,
    performance_metrics TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Agent execution indexes
CREATE INDEX idx_agent_executions_task_id ON agent_executions(task_id);
CREATE INDEX idx_agent_executions_agent_id ON agent_executions(agent_id);
CREATE INDEX idx_agent_executions_status ON agent_executions(status);
CREATE INDEX idx_agent_executions_pr_number ON agent_executions(pr_number);

-- Agent execution triggers
CREATE TRIGGER agent_executions_updated_at
AFTER UPDATE ON agent_executions
BEGIN
    UPDATE agent_executions SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;

CREATE TRIGGER update_task_actual_hours
AFTER UPDATE ON agent_executions
WHEN NEW.status = 'completed' AND NEW.execution_time_seconds IS NOT NULL
BEGIN
    UPDATE tasks
    SET actual_hours = COALESCE(actual_hours, 0) + (NEW.execution_time_seconds / 3600.0)
    WHERE id = NEW.task_id;
END;

CREATE TRIGGER update_task_on_pr_merge
AFTER UPDATE ON agent_executions
WHEN NEW.pr_status = 'merged' AND OLD.pr_status != 'merged'
BEGIN
    UPDATE tasks
    SET
        status = 'completed',
        completed_at = NEW.pr_merged_at
    WHERE id = NEW.task_id
      AND status != 'completed';
END;

-- PR reviews table
CREATE TABLE pr_reviews (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL REFERENCES agent_executions(id) ON DELETE CASCADE,
    reviewer_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    reviewer_type TEXT NOT NULL DEFAULT 'ai',
    review_status TEXT NOT NULL,
    review_body TEXT,
    comments TEXT,
    suggested_changes TEXT,
    approval_date TEXT,
    dismissal_reason TEXT,
    reviewed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- PR review indexes
CREATE INDEX idx_pr_reviews_execution_id ON pr_reviews(execution_id);
CREATE INDEX idx_pr_reviews_reviewer_id ON pr_reviews(reviewer_id);
CREATE INDEX idx_pr_reviews_status ON pr_reviews(review_status);

CREATE TRIGGER pr_reviews_updated_at
AFTER UPDATE ON pr_reviews
BEGIN
    UPDATE pr_reviews SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;

-- Task-Spec Links
CREATE TABLE task_spec_links (
    task_id TEXT NOT NULL,
    requirement_id TEXT NOT NULL,
    scenario_id TEXT,
    validation_status TEXT DEFAULT 'pending' CHECK(validation_status IN ('pending', 'passed', 'failed')),
    validation_result TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    PRIMARY KEY (task_id, requirement_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (requirement_id) REFERENCES spec_requirements(id) ON DELETE CASCADE,
    FOREIGN KEY (scenario_id) REFERENCES spec_scenarios(id) ON DELETE SET NULL
);

CREATE INDEX idx_task_spec_links_task ON task_spec_links(task_id);
CREATE INDEX idx_task_spec_links_requirement ON task_spec_links(requirement_id);
CREATE INDEX idx_task_spec_links_status ON task_spec_links(validation_status);
CREATE INDEX idx_task_spec_links_scenario ON task_spec_links(scenario_id);

-- PRD-Spec Sync History
CREATE TABLE prd_spec_sync_history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    prd_id TEXT NOT NULL,
    direction TEXT NOT NULL CHECK(direction IN ('prd_to_spec', 'spec_to_prd', 'task_to_spec')),
    changes_json TEXT NOT NULL,
    performed_by TEXT,
    performed_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE CASCADE
);

CREATE INDEX idx_prd_spec_sync_history_prd ON prd_spec_sync_history(prd_id);

-- Spec Materializations
CREATE TABLE spec_materializations (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL,
    path TEXT NOT NULL,
    materialized_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    sha256_hash TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_spec_materializations_project ON spec_materializations(project_id);
CREATE INDEX idx_spec_materializations_path ON spec_materializations(project_id, path);

-- ============================================================================
-- CONTEXT MANAGEMENT
-- ============================================================================

-- Context Configurations
CREATE TABLE context_configurations (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    include_patterns TEXT NOT NULL DEFAULT '[]',
    exclude_patterns TEXT NOT NULL DEFAULT '[]',
    max_tokens INTEGER NOT NULL DEFAULT 100000,
    spec_capability_id TEXT REFERENCES spec_capabilities(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_context_configs_project ON context_configurations(project_id);

-- Context Snapshots
CREATE TABLE context_snapshots (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    configuration_id TEXT REFERENCES context_configurations(id) ON DELETE SET NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    file_count INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_context_snapshots_project ON context_snapshots(project_id);
CREATE INDEX idx_context_snapshots_config ON context_snapshots(configuration_id);

-- Context Usage Patterns
CREATE TABLE context_usage_patterns (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    inclusion_count INTEGER NOT NULL DEFAULT 0,
    last_used TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    UNIQUE(project_id, file_path) ON CONFLICT REPLACE
);

CREATE INDEX idx_context_patterns_project ON context_usage_patterns(project_id);
CREATE INDEX idx_context_patterns_last_used ON context_usage_patterns(last_used);

-- AST-Spec Mappings
CREATE TABLE ast_spec_mappings (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    project_id TEXT NOT NULL REFERENCES projects(id),
    file_path TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    symbol_type TEXT NOT NULL,
    line_number INTEGER,
    requirement_id TEXT REFERENCES spec_requirements(id),
    confidence REAL DEFAULT 0.0,
    verified INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_ast_spec_mappings_project ON ast_spec_mappings(project_id);
CREATE INDEX idx_ast_spec_mappings_requirement ON ast_spec_mappings(requirement_id);

-- Context Templates
CREATE TABLE context_templates (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    description TEXT,
    template_type TEXT NOT NULL,
    include_patterns TEXT DEFAULT '[]',
    exclude_patterns TEXT DEFAULT '[]',
    ast_filters TEXT,
    created_at TEXT DEFAULT (datetime('now', 'utc'))
);

-- AI Usage Tracking
CREATE TABLE ai_usage_logs (
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
    context_snapshot_id TEXT REFERENCES context_snapshots(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_ai_usage_logs_project ON ai_usage_logs(project_id);
CREATE INDEX idx_ai_usage_logs_created ON ai_usage_logs(created_at);
CREATE INDEX idx_ai_usage_logs_operation ON ai_usage_logs(operation);
CREATE INDEX idx_ai_usage_logs_provider_model ON ai_usage_logs(provider, model);
CREATE INDEX idx_ai_usage_logs_provider_model_created ON ai_usage_logs(provider, model, created_at);
CREATE INDEX idx_ai_usage_logs_context ON ai_usage_logs(context_snapshot_id);

-- Note: AI usage logs should be cleaned up via scheduled job (recommended: 90 days retention)
-- Running cleanup on every INSERT would cause performance issues at scale

-- ============================================================================
-- SECURITY & AUTHENTICATION
-- ============================================================================

-- Encryption Settings
CREATE TABLE encryption_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    encryption_mode TEXT NOT NULL DEFAULT 'machine' CHECK (encryption_mode IN ('machine', 'password')),
    password_salt BLOB,
    password_hash BLOB,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_encryption_settings_mode ON encryption_settings(encryption_mode);

-- Password Attempts
CREATE TABLE password_attempts (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    last_attempt_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_password_attempts_lockout
ON password_attempts(locked_until) WHERE locked_until IS NOT NULL;

-- API Tokens
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY,
    token_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    last_used_at TEXT,
    is_active INTEGER DEFAULT 1 NOT NULL
);

CREATE INDEX idx_api_tokens_active ON api_tokens(is_active);
CREATE INDEX idx_api_tokens_hash ON api_tokens(token_hash);

-- ============================================================================
-- SYSTEM CONFIGURATION
-- ============================================================================

-- Storage Metadata
CREATE TABLE storage_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

-- System Settings
CREATE TABLE system_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    data_type TEXT NOT NULL DEFAULT 'string',
    is_secret INTEGER DEFAULT 0,
    requires_restart INTEGER DEFAULT 0,
    is_env_only INTEGER DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_by TEXT DEFAULT 'system'
);

CREATE INDEX idx_system_settings_category ON system_settings(category);

-- ============================================================================
-- TELEMETRY
-- ============================================================================

-- Telemetry Settings
CREATE TABLE telemetry_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    first_run BOOLEAN NOT NULL DEFAULT TRUE,
    onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE,
    error_reporting BOOLEAN NOT NULL DEFAULT FALSE,
    usage_metrics BOOLEAN NOT NULL DEFAULT FALSE,
    non_anonymous_metrics BOOLEAN NOT NULL DEFAULT FALSE,
    machine_id TEXT,
    user_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE TRIGGER update_telemetry_settings_timestamp
AFTER UPDATE ON telemetry_settings
BEGIN
    UPDATE telemetry_settings
    SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;

-- Telemetry Events
CREATE TABLE telemetry_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    event_name TEXT NOT NULL,
    event_data TEXT,
    anonymous BOOLEAN NOT NULL DEFAULT TRUE,
    session_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    sent_at TEXT,
    retry_count INTEGER DEFAULT 0
);

CREATE INDEX idx_telemetry_events_unsent
ON telemetry_events(sent_at, created_at)
WHERE sent_at IS NULL;

CREATE INDEX idx_telemetry_events_type
ON telemetry_events(event_type, created_at);

-- Telemetry Statistics
CREATE TABLE telemetry_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stat_date TEXT NOT NULL,
    total_events INTEGER DEFAULT 0,
    error_events INTEGER DEFAULT 0,
    usage_events INTEGER DEFAULT 0,
    performance_events INTEGER DEFAULT 0,
    events_sent INTEGER DEFAULT 0,
    events_pending INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    UNIQUE(stat_date)
);

-- ============================================================================
-- CLOUD SYNC (FUTURE USE)
-- ============================================================================

-- Sync Snapshots
CREATE TABLE sync_snapshots (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    compressed_data BLOB NOT NULL,
    checksum TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    project_count INTEGER NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'pending' CHECK (sync_status IN ('pending', 'uploading', 'uploaded', 'failed'))
);

CREATE INDEX idx_sync_snapshots_created_at ON sync_snapshots(created_at);
CREATE INDEX idx_sync_snapshots_status ON sync_snapshots(sync_status);

-- Sync State
CREATE TABLE sync_state (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    device_id TEXT,
    last_sync_at TEXT,
    last_snapshot_id TEXT,
    conflict_resolution TEXT DEFAULT 'manual' CHECK (conflict_resolution IN ('manual', 'local_wins', 'remote_wins', 'merge')),
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (last_snapshot_id) REFERENCES sync_snapshots(id)
);

CREATE INDEX idx_sync_state_user_device ON sync_state(user_id, device_id);
CREATE INDEX idx_sync_state_last_sync ON sync_state(last_sync_at);

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Storage metadata
INSERT INTO storage_metadata (key, value) VALUES
    ('schema_version', '1'),
    ('created_at', datetime('now', 'utc')),
    ('storage_type', 'sqlite');

-- Default user
INSERT OR IGNORE INTO users (id, email, name, created_at, updated_at)
VALUES ('default-user', 'user@localhost', 'Default User', datetime('now', 'utc'), datetime('now', 'utc'));

-- AI agents (default configurations for supported models)
INSERT OR IGNORE INTO agents (id, name, type, provider, model, display_name, description, cost_per_1k_input_tokens, cost_per_1k_output_tokens, max_context_tokens, supports_tools, supports_vision, supports_web_search, created_at, updated_at)
VALUES
    ('claude-sonnet-4', 'claude-sonnet-4', 'ai', 'anthropic', 'claude-sonnet-4-20250514', 'Claude Sonnet 4', 'Best coding model in the world, strongest for building complex agents', 0.003, 0.015, 200000, 1, 1, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('claude-opus-4', 'claude-opus-4', 'ai', 'anthropic', 'claude-opus-4-20250514', 'Claude Opus 4', 'Most intelligent model, hybrid reasoning for complex tasks', 0.015, 0.075, 200000, 1, 1, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('claude-haiku-3-5', 'claude-haiku-3-5', 'ai', 'anthropic', 'claude-3-5-haiku-20241022', 'Claude 3.5 Haiku', 'Fast and efficient, near-frontier performance at low cost', 0.001, 0.005, 200000, 1, 1, 0, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gpt-4o', 'gpt-4o', 'ai', 'openai', 'gpt-4o', 'GPT-4o', 'Multimodal GPT-4 optimized for speed', 0.0025, 0.01, 128000, 1, 1, 0, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gpt-4-turbo', 'gpt-4-turbo', 'ai', 'openai', 'gpt-4-turbo-preview', 'GPT-4 Turbo', 'Latest GPT-4 with improved performance', 0.01, 0.03, 128000, 1, 1, 0, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gemini-pro', 'gemini-pro', 'ai', 'google', 'gemini-1.5-pro-latest', 'Gemini 1.5 Pro', 'Google''s most capable multimodal AI', 0.00125, 0.005, 2000000, 1, 1, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gemini-flash', 'gemini-flash', 'ai', 'google', 'gemini-1.5-flash-latest', 'Gemini 1.5 Flash', 'Fast and efficient for high-volume tasks', 0.000075, 0.0003, 1000000, 1, 1, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('grok-2', 'grok-2', 'ai', 'xai', 'grok-2-latest', 'Grok 2', 'xAI''s flagship conversational AI', 0.002, 0.01, 131072, 1, 0, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('grok-beta', 'grok-beta', 'ai', 'xai', 'grok-beta', 'Grok Beta', 'Experimental Grok model', 0.005, 0.015, 131072, 1, 0, 1, datetime('now', 'utc'), datetime('now', 'utc'));

-- Default user's active agents
INSERT OR IGNORE INTO user_agents (id, user_id, agent_id, is_active, created_at, updated_at)
VALUES
    ('ua-1', 'default-user', 'claude-sonnet-4', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-2', 'default-user', 'claude-haiku-3-5', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-3', 'default-user', 'gpt-4o', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-4', 'default-user', 'gemini-pro', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-5', 'default-user', 'gemini-flash', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-6', 'default-user', 'grok-2', 1, datetime('now', 'utc'), datetime('now', 'utc'));

-- Update default user's default agent
UPDATE users SET default_agent_id = 'claude-sonnet-4' WHERE id = 'default-user';

-- Default tag
INSERT OR IGNORE INTO tags (id, name, color, description, created_at)
VALUES ('tag-main', 'main', '#3b82f6', 'Default tag for general tasks', datetime('now', 'utc'));

-- Encryption settings (machine-based encryption by default)
INSERT OR IGNORE INTO encryption_settings (id, encryption_mode)
VALUES (1, 'machine');

-- Password attempts tracking
INSERT OR IGNORE INTO password_attempts (id, attempt_count) VALUES (1, 0);

-- Telemetry settings
INSERT OR IGNORE INTO telemetry_settings (id) VALUES (1);

-- System settings (default configuration)
INSERT OR IGNORE INTO system_settings (key, value, category, description, data_type, requires_restart, is_env_only) VALUES
    -- Cloud Configuration
    ('cloud_enabled', 'false', 'cloud', 'Enable cloud sync features', 'boolean', 0, 0),
    ('cloud_api_url', 'https://api.orkee.ai', 'cloud', 'Orkee Cloud API URL', 'string', 0, 0),

    -- Server Configuration
    ('api_port', '4001', 'server', 'API server port (set via ORKEE_API_PORT in .env)', 'integer', 1, 1),
    ('ui_port', '5173', 'server', 'Dashboard UI port (set via ORKEE_UI_PORT in .env)', 'integer', 1, 1),
    ('dev_mode', 'false', 'server', 'Development mode (set via ORKEE_DEV_MODE in .env)', 'boolean', 1, 1),

    -- Security Configuration
    ('cors_allow_any_localhost', 'true', 'security', 'Allow any localhost origin in development', 'boolean', 1, 0),
    ('allowed_browse_paths', '~/Documents,~/Projects,~/Code,~/Desktop', 'security', 'Comma-separated list of allowed directory paths', 'string', 0, 0),
    ('browse_sandbox_mode', 'relaxed', 'security', 'Directory browsing security mode: strict/relaxed/disabled', 'string', 0, 0),

    -- TLS/HTTPS Configuration
    ('tls_enabled', 'false', 'tls', 'Enable HTTPS/TLS support', 'boolean', 1, 0),
    ('tls_cert_path', '~/.orkee/certs/cert.pem', 'tls', 'Path to TLS certificate file', 'string', 1, 0),
    ('tls_key_path', '~/.orkee/certs/key.pem', 'tls', 'Path to TLS private key file', 'string', 1, 0),
    ('auto_generate_cert', 'true', 'tls', 'Auto-generate self-signed certificates for development', 'boolean', 1, 0),

    -- Rate Limiting
    ('rate_limit_enabled', 'true', 'rate_limiting', 'Enable rate limiting middleware', 'boolean', 1, 0),
    ('rate_limit_health_rpm', '60', 'rate_limiting', 'Health endpoint requests per minute', 'integer', 1, 0),
    ('rate_limit_browse_rpm', '20', 'rate_limiting', 'Directory browsing requests per minute', 'integer', 1, 0),
    ('rate_limit_projects_rpm', '30', 'rate_limiting', 'Project operations requests per minute', 'integer', 1, 0),
    ('rate_limit_preview_rpm', '10', 'rate_limiting', 'Preview operations requests per minute', 'integer', 1, 0),
    ('rate_limit_ai_rpm', '10', 'rate_limiting', 'AI proxy endpoint requests per minute', 'integer', 1, 0),
    ('rate_limit_global_rpm', '30', 'rate_limiting', 'Global requests per minute for other endpoints', 'integer', 1, 0),
    ('rate_limit_burst_size', '5', 'rate_limiting', 'Burst size multiplier', 'integer', 1, 0),

    -- Security Headers
    ('security_headers_enabled', 'true', 'security', 'Enable security headers middleware', 'boolean', 1, 0),
    ('enable_hsts', 'false', 'security', 'Enable HSTS (only for HTTPS)', 'boolean', 1, 0),
    ('enable_request_id', 'true', 'security', 'Enable request ID generation for audit logging', 'boolean', 1, 0),

    -- Telemetry
    ('telemetry_enabled', 'false', 'telemetry', 'Enable telemetry data collection', 'boolean', 0, 0);
