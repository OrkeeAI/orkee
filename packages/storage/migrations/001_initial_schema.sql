-- ABOUTME: Initial Orkee database schema with project management, task tracking, OpenSpec, and telemetry
-- ABOUTME: Includes security, context management, API tokens, and comprehensive indexing for performance

-- ============================================================================
-- PRE-1.0 SCHEMA CONSOLIDATION
-- ============================================================================
-- This migration represents the complete initial schema for Orkee, including
-- the ideate feature. During pre-release development, this schema was built
-- incrementally across multiple development iterations, but has been
-- consolidated into a single initial migration since no production instances
-- exist yet.
--
-- This consolidation approach is valid because:
-- - No users are running Orkee in production
-- - No existing databases need migration
-- - Simpler initial setup for new installations
-- - Easier to test as a single atomic schema
--
-- Post-1.0: All schema changes will be separate, incremental migrations.
-- ============================================================================

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- ID GENERATION STRATEGY
-- ============================================================================
-- ALL entities use application-generated IDs (no DEFAULT clause on id column).
--
-- Implementation: packages/projects/src/storage/mod.rs::generate_project_id()
-- - Generates 8-character random IDs using alphanumeric charset [0-9A-Za-z]
-- - Format: Similar to nanoid for collision resistance
-- - Examples: 'xY9kL2m7', 'A4bC8dEf', 'test-user' (tests use custom IDs)
--
-- Why application-generated?
-- - Enables predictable IDs in tests ('test-project-123', 'default-user')
-- - Allows custom ID formats (e.g., change IDs with verb prefixes)
-- - Provides consistent ID format across all entities
-- - Works well with RETURNING clause for immediate ID access
-- - Application has full control over ID collision handling

-- ============================================================================
-- CORE PROJECT MANAGEMENT
-- ============================================================================

-- Projects table - main entity
CREATE TABLE projects (
    -- Core fields
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
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

    -- GitHub integration
    github_owner TEXT,
    github_repo TEXT,
    github_sync_enabled BOOLEAN DEFAULT FALSE,
    github_token_encrypted TEXT CHECK(github_token_encrypted IS NULL OR length(github_token_encrypted) >= 38),
    github_labels_config TEXT,
    github_default_assignee TEXT,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    -- Constraints
    CHECK (json_valid(tags) OR tags IS NULL),
    CHECK (json_valid(manual_tasks) OR manual_tasks IS NULL),
    CHECK (json_valid(mcp_servers) OR mcp_servers IS NULL),
    CHECK (json_valid(git_repository) OR git_repository IS NULL),
    CHECK (json_valid(github_labels_config) OR github_labels_config IS NULL)
);

-- Project indexes
-- Query patterns:
--   - Filter by status alone (many queries)
--   - Filter by status + priority (API supports via filter, rarely used)
--   - Filter by status then ORDER BY rank (active_projects view)
--   - Sort by rank, priority, or updated_at independently
CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_priority ON projects(priority);
CREATE INDEX idx_projects_rank ON projects(rank);
CREATE INDEX idx_projects_updated_at ON projects(updated_at);
CREATE INDEX idx_projects_created_at ON projects(created_at);
CREATE INDEX idx_projects_task_source ON projects(task_source);
CREATE INDEX idx_projects_status_priority ON projects(status, priority);  -- Supports filtering by status+priority together
CREATE INDEX idx_projects_status_rank ON projects(status, rank);  -- Supports active_projects view (WHERE status IN (...) ORDER BY rank)

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
    UPDATE projects SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
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
-- API KEY ENCRYPTION (SECURITY-CRITICAL):
-- All API key fields MUST contain encrypted data, never plaintext
-- Encryption: ChaCha20-Poly1305 AEAD with base64 encoding
-- Format: base64(nonce || ciphertext || tag) - minimum 38 characters
-- Empty keys: Stored as NULL (prefer NULL for "not set")
-- Implementation: packages/projects/src/security/encryption.rs
-- Management: Use `orkee security` commands to configure encryption mode
CREATE TABLE users (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    default_agent_id TEXT,  -- References packages/agents/config/agents.json agent.id (no FK enforcement)
    theme TEXT DEFAULT 'system',

    -- API keys stored as encrypted base64 strings
    -- CHECK constraint enforces minimum encrypted length or NULL (use NULL for "not set")
    -- Minimum 38 chars = base64(12-byte nonce + 16-byte tag + 0-byte plaintext)
    openai_api_key TEXT CHECK (openai_api_key IS NULL OR length(openai_api_key) >= 38),
    anthropic_api_key TEXT CHECK (anthropic_api_key IS NULL OR length(anthropic_api_key) >= 38),
    google_api_key TEXT CHECK (google_api_key IS NULL OR length(google_api_key) >= 38),
    xai_api_key TEXT CHECK (xai_api_key IS NULL OR length(xai_api_key) >= 38),

    ai_gateway_enabled BOOLEAN DEFAULT FALSE,
    ai_gateway_url TEXT,
    ai_gateway_key TEXT CHECK (ai_gateway_key IS NULL OR length(ai_gateway_key) >= 38),
    avatar_url TEXT,
    preferences TEXT,
    last_login_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TRIGGER users_updated_at AFTER UPDATE ON users
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE users SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- Agents table
-- Agents table removed: Now loaded from packages/agents/config/agents.json
-- Models are defined in packages/models/config/models.json
-- See src/models/mod.rs for ModelRegistry (in-memory registry)

-- User-Agent association table
-- Stores which agents a user has enabled and their preferences
CREATE TABLE user_agents (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,  -- References packages/agents/config/agents.json agent.id (no FK enforcement)
    preferred_model_id TEXT,  -- References packages/models/config/models.json model.id (no FK enforcement)
    is_active INTEGER NOT NULL DEFAULT 1,

    -- User customization
    custom_name TEXT,
    custom_system_prompt TEXT,
    custom_temperature REAL,
    custom_max_tokens INTEGER,

    -- Usage stats
    tasks_assigned INTEGER DEFAULT 0,
    tasks_completed INTEGER DEFAULT 0,
    total_tokens_used INTEGER DEFAULT 0,
    total_cost_cents INTEGER DEFAULT 0,
    last_used_at TEXT,

    -- Metadata
    custom_settings TEXT,  -- JSON for additional preferences
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(user_id, agent_id)
);

CREATE INDEX idx_user_agents_agent_id ON user_agents(agent_id);

CREATE TRIGGER user_agents_updated_at AFTER UPDATE ON user_agents
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE user_agents SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- ============================================================================
-- TASK MANAGEMENT
-- ============================================================================

-- Tags table
CREATE TABLE tags (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
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
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,
    ideate_session_id TEXT,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'approved', 'superseded')),
    source TEXT DEFAULT 'manual' CHECK(source IN ('manual', 'generated', 'synced')),
    conversation_id TEXT,
    github_epic_url TEXT,
    discovery_status TEXT DEFAULT 'draft' CHECK(discovery_status IN ('draft', 'brainstorming', 'refining', 'validating', 'finalized')),
    discovery_completed_at TEXT,
    quality_score INTEGER CHECK(quality_score >= 0 AND quality_score <= 100),
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    created_by TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (ideate_session_id) REFERENCES ideate_sessions(id) ON DELETE SET NULL
);

CREATE INDEX idx_prds_project ON prds(project_id);
CREATE INDEX idx_prds_ideate_session ON prds(ideate_session_id);
CREATE INDEX idx_prds_status ON prds(status);
CREATE INDEX idx_prds_not_deleted ON prds(id) WHERE deleted_at IS NULL;
CREATE INDEX idx_prds_project_not_deleted ON prds(project_id, status) WHERE deleted_at IS NULL;

CREATE TRIGGER prds_updated_at AFTER UPDATE ON prds
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE prds SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- ============================================================================
-- TASK MANAGEMENT (CONTINUED - AFTER OPENSPEC)
-- ============================================================================

-- Tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    priority TEXT NOT NULL DEFAULT 'medium',
    created_by_user_id TEXT NOT NULL DEFAULT 'default-user' REFERENCES users(id) ON DELETE RESTRICT,
    assigned_agent_id TEXT,  -- References packages/agents/config/agents.json agent.id (no FK enforcement)
    reviewed_by_agent_id TEXT,  -- References packages/agents/config/agents.json agent.id (no FK enforcement)
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
    from_prd_id TEXT REFERENCES prds(id) ON DELETE SET NULL,
    epic_id TEXT,
    github_issue_number INTEGER,
    github_issue_url TEXT,
    parallel_group TEXT,
    depends_on TEXT,
    conflicts_with TEXT,
    task_type TEXT DEFAULT 'task' CHECK(task_type IN ('task', 'subtask')),
    size_estimate TEXT CHECK(size_estimate IN ('XS', 'S', 'M', 'L', 'XL')),
    technical_details TEXT,
    effort_hours INTEGER CHECK(effort_hours > 0),
    can_parallel BOOLEAN DEFAULT FALSE,
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
CREATE INDEX idx_tasks_from_prd_id ON tasks(from_prd_id);
CREATE INDEX idx_tasks_user_status ON tasks(created_by_user_id, status);
CREATE INDEX idx_tasks_project_status ON tasks(project_id, status);
CREATE INDEX idx_tasks_project_priority ON tasks(project_id, priority);

CREATE TRIGGER tasks_updated_at AFTER UPDATE ON tasks
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE tasks SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- Task full-text search
CREATE VIRTUAL TABLE tasks_fts USING fts5(
    id UNINDEXED,
    title,
    description,
    details,
    tags,
    content='tasks',
    content_rowid='rowid'
);

-- Task FTS triggers
CREATE TRIGGER tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, id, title, description, details, tags)
    VALUES (new.rowid, new.id, new.title, new.description, new.details, new.tags);
END;

CREATE TRIGGER tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER tasks_fts_update AFTER UPDATE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.rowid;
    INSERT INTO tasks_fts(rowid, id, title, description, details, tags)
    VALUES (new.rowid, new.id, new.title, new.description, new.details, new.tags);
END;

-- ============================================================================
-- AGENT EXECUTION TRACKING
-- ============================================================================

-- Agent executions table
CREATE TABLE agent_executions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    agent_id TEXT,  -- References packages/agents/config/agents.json agent.id (no FK enforcement)
    model TEXT,  -- References packages/models/config/models.json model.id (no FK enforcement)
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
    UPDATE agent_executions SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
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
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    execution_id TEXT NOT NULL REFERENCES agent_executions(id) ON DELETE CASCADE,
    reviewer_id TEXT,  -- References packages/agents/config/agents.json agent.id (no FK enforcement)
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
    UPDATE pr_reviews SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
    WHERE id = NEW.id;
END;

-- Task-Spec Links


-- PRD-Spec Sync History


-- AI Usage Tracking
CREATE TABLE ai_usage_logs (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
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
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_ai_usage_logs_project ON ai_usage_logs(project_id);
CREATE INDEX idx_ai_usage_logs_created ON ai_usage_logs(created_at);
CREATE INDEX idx_ai_usage_logs_operation ON ai_usage_logs(operation);
CREATE INDEX idx_ai_usage_logs_provider_model ON ai_usage_logs(provider, model);
CREATE INDEX idx_ai_usage_logs_provider_model_created ON ai_usage_logs(provider, model, created_at);
CREATE INDEX idx_ai_usage_logs_context ON ai_usage_logs(context_snapshot_id);

-- Note: AI usage logs should be cleaned up via external scheduled job or cron task
-- Recommended retention: 30-90 days depending on usage patterns
-- Cleanup logic is implemented in application code, not in this migration

-- ============================================================================
-- SECURITY & AUTHENTICATION
-- ============================================================================

-- Encryption Settings
CREATE TABLE encryption_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    encryption_mode TEXT NOT NULL DEFAULT 'machine' CHECK (encryption_mode IN ('machine', 'password')),
    password_salt BLOB,
    password_hash BLOB,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_encryption_settings_mode ON encryption_settings(encryption_mode);

-- Password Attempts
CREATE TABLE password_attempts (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    last_attempt_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_password_attempts_lockout
ON password_attempts(locked_until) WHERE locked_until IS NOT NULL;

-- API Tokens
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    token_hash TEXT NOT NULL,  -- SHA-256 hash of token (see api_tokens/storage.rs)
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_used_at TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
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
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- System Settings
CREATE TABLE system_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    data_type TEXT NOT NULL DEFAULT 'string',
    is_secret BOOLEAN DEFAULT FALSE,
    requires_restart BOOLEAN DEFAULT FALSE,
    is_env_only BOOLEAN DEFAULT FALSE,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_by TEXT DEFAULT 'system'
);

CREATE INDEX idx_system_settings_category ON system_settings(category);

CREATE TRIGGER system_settings_updated_at AFTER UPDATE ON system_settings
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE system_settings SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE key = NEW.key;
END;

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
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TRIGGER update_telemetry_settings_timestamp
AFTER UPDATE ON telemetry_settings
BEGIN
    UPDATE telemetry_settings
    SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
    WHERE id = NEW.id;
END;

-- Telemetry Events
CREATE TABLE telemetry_events (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    event_type TEXT NOT NULL,
    event_name TEXT NOT NULL,
    event_data TEXT,
    anonymous BOOLEAN NOT NULL DEFAULT TRUE,
    session_id TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
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
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    stat_date TEXT NOT NULL,
    total_events INTEGER DEFAULT 0,
    error_events INTEGER DEFAULT 0,
    usage_events INTEGER DEFAULT 0,
    performance_events INTEGER DEFAULT 0,
    events_sent INTEGER DEFAULT 0,
    events_pending INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(stat_date)
);

-- ============================================================================
-- CLOUD SYNC (FUTURE USE)
-- ============================================================================

-- Sync Snapshots
CREATE TABLE sync_snapshots (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
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
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    user_id TEXT,
    device_id TEXT,
    last_sync_at TEXT,
    last_snapshot_id TEXT,
    conflict_resolution TEXT DEFAULT 'manual' CHECK (conflict_resolution IN ('manual', 'local_wins', 'remote_wins', 'merge')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (last_snapshot_id) REFERENCES sync_snapshots(id)
);

CREATE INDEX idx_sync_state_user_device ON sync_state(user_id, device_id);
CREATE INDEX idx_sync_state_last_sync ON sync_state(last_sync_at);

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Storage metadata
-- Note: Schema version is managed by SQLx via _sqlx_migrations table
-- Do not add schema_version here - it creates two sources of truth
INSERT OR IGNORE INTO storage_metadata (key, value) VALUES
    ('created_at', strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('storage_type', 'sqlite');

-- Default user (required for FK dependency)
INSERT OR IGNORE INTO users (id, email, name, created_at, updated_at)
VALUES ('default-user', 'user@localhost', 'Default User', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));

-- Default tag (required for task FK dependency)
INSERT OR IGNORE INTO tags (id, name, color, description, created_at)
VALUES ('tag-main', 'main', '#3b82f6', 'Default main tag for tasks', strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));

-- Note: Agents and models are now loaded from packages/agents/config/agents.json and packages/models/config/models.json
-- No seed data needed here. See src/models/mod.rs for ModelRegistry

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
    ('telemetry_enabled', 'false', 'telemetry', 'Enable telemetry data collection', 'boolean', 0, 0),

    -- AI & Ideation Configuration
    ('ideate.max_tokens', '8000', 'ai', 'Maximum tokens per PRD section generation', 'integer', 0, 0),
    ('ideate.temperature', '0.7', 'ai', 'AI temperature for creativity (0-1)', 'number', 0, 0),
    ('ideate.model', 'claude-3-opus-20240229', 'ai', 'AI model for PRD generation', 'string', 0, 0),
    ('ideate.timeout_seconds', '120', 'ai', 'Timeout for PRD generation requests', 'integer', 0, 0),
    ('ideate.retry_attempts', '3', 'ai', 'Number of retry attempts on AI API failure', 'integer', 0, 0);

-- ============================================================================
-- IDEATE SCHEMA
-- ============================================================================

CREATE TABLE ideate_sessions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,

    -- Minimal required info
    initial_description TEXT NOT NULL,

    -- Session metadata
    mode TEXT NOT NULL CHECK(mode IN ('quick', 'guided', 'comprehensive', 'conversational')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'in_progress', 'ready_for_prd', 'completed')),
    current_section TEXT,

    -- Track what user chose to skip (JSON array of section names)
    skipped_sections TEXT,

    -- Link to template and generated PRD
    template_id TEXT,
    generated_prd_id TEXT,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (template_id) REFERENCES prd_output_templates(id) ON DELETE SET NULL,
    FOREIGN KEY (generated_prd_id) REFERENCES prds(id) ON DELETE SET NULL,
    CHECK (json_valid(skipped_sections) OR skipped_sections IS NULL)
);

CREATE INDEX idx_ideate_sessions_project ON ideate_sessions(project_id);
CREATE INDEX idx_ideate_sessions_status ON ideate_sessions(status);
CREATE INDEX idx_ideate_sessions_mode ON ideate_sessions(mode);
CREATE INDEX idx_ideate_sessions_current_section ON ideate_sessions(current_section);
CREATE INDEX idx_ideate_sessions_template ON ideate_sessions(template_id);

CREATE TRIGGER ideate_sessions_updated_at AFTER UPDATE ON ideate_sessions
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE ideate_sessions SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- ============================================================================
-- PRD SECTIONS (All Optional)
-- ============================================================================

-- Overview Section
CREATE TABLE ideate_overview (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    problem_statement TEXT,
    target_audience TEXT,
    value_proposition TEXT,
    one_line_pitch TEXT,
    ai_generated INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_ideate_overview_session ON ideate_overview(session_id);

-- Core Features
CREATE TABLE ideate_features (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    feature_name TEXT NOT NULL,
    what_it_does TEXT,
    why_important TEXT,
    how_it_works TEXT,

    -- Dependency chain fields
    depends_on TEXT, -- JSON array of feature IDs this depends on
    enables TEXT, -- JSON array of feature IDs this unlocks
    build_phase INTEGER DEFAULT 1, -- 1=foundation, 2=visible, 3=enhancement
    is_visible INTEGER DEFAULT 0, -- Boolean: does this give user something to see/use?

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(depends_on) OR depends_on IS NULL),
    CHECK (json_valid(enables) OR enables IS NULL),
    CHECK (build_phase IN (1, 2, 3)),
    CHECK (depends_on IS NULL OR json_array_length(depends_on) <= 100),
    CHECK (enables IS NULL OR json_array_length(enables) <= 100)
);

CREATE INDEX idx_ideate_features_session ON ideate_features(session_id);
CREATE INDEX idx_ideate_features_phase ON ideate_features(build_phase);
CREATE INDEX idx_ideate_features_visible ON ideate_features(is_visible);

-- User Experience
CREATE TABLE ideate_ux (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    personas TEXT, -- JSON array of personas
    user_flows TEXT, -- JSON array of user flows
    ui_considerations TEXT,
    ux_principles TEXT,
    ai_generated INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(personas) OR personas IS NULL),
    CHECK (json_valid(user_flows) OR user_flows IS NULL)
);

CREATE INDEX idx_ideate_ux_session ON ideate_ux(session_id);

-- Technical Architecture
CREATE TABLE ideate_technical (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    components TEXT, -- JSON array
    data_models TEXT, -- JSON array
    apis TEXT, -- JSON array
    infrastructure TEXT, -- JSON object
    tech_stack_quick TEXT, -- For quick mode: "React + Node + PostgreSQL"
    ai_generated INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(components) OR components IS NULL),
    CHECK (json_valid(data_models) OR data_models IS NULL),
    CHECK (json_valid(apis) OR apis IS NULL),
    CHECK (json_valid(infrastructure) OR infrastructure IS NULL)
);

CREATE INDEX idx_ideate_technical_session ON ideate_technical(session_id);

-- Development Roadmap (NO timelines, just scope)
CREATE TABLE ideate_roadmap (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    mvp_scope TEXT, -- JSON array of features in MVP
    future_phases TEXT, -- JSON array of post-MVP phases
    ai_generated INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(mvp_scope) OR mvp_scope IS NULL),
    CHECK (json_valid(future_phases) OR future_phases IS NULL)
);

CREATE INDEX idx_ideate_roadmap_session ON ideate_roadmap(session_id);

-- Logical Dependency Chain
CREATE TABLE ideate_dependencies (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    foundation_features TEXT, -- JSON array of feature IDs: must build first
    visible_features TEXT, -- JSON array of feature IDs: get something usable quickly
    enhancement_features TEXT, -- JSON array of feature IDs: build upon foundation
    dependency_graph TEXT, -- JSON object for visual representation
    ai_generated INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(foundation_features) OR foundation_features IS NULL),
    CHECK (json_valid(visible_features) OR visible_features IS NULL),
    CHECK (json_valid(enhancement_features) OR enhancement_features IS NULL),
    CHECK (json_valid(dependency_graph) OR dependency_graph IS NULL)
);

CREATE INDEX idx_ideate_dependencies_session ON ideate_dependencies(session_id);

-- Risks and Mitigations
CREATE TABLE ideate_risks (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    technical_risks TEXT, -- JSON array
    mvp_scoping_risks TEXT, -- JSON array
    resource_risks TEXT, -- JSON array
    mitigations TEXT, -- JSON array
    ai_generated INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(technical_risks) OR technical_risks IS NULL),
    CHECK (json_valid(mvp_scoping_risks) OR mvp_scoping_risks IS NULL),
    CHECK (json_valid(resource_risks) OR resource_risks IS NULL),
    CHECK (json_valid(mitigations) OR mitigations IS NULL)
);

CREATE INDEX idx_ideate_risks_session ON ideate_risks(session_id);

-- Research & Appendix
CREATE TABLE ideate_research (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    competitors TEXT, -- JSON array
    similar_projects TEXT, -- JSON array
    research_findings TEXT,
    technical_specs TEXT,
    reference_links TEXT, -- JSON array (renamed from 'references' to avoid SQL keyword)
    ai_generated INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(competitors) OR competitors IS NULL),
    CHECK (json_valid(similar_projects) OR similar_projects IS NULL),
    CHECK (json_valid(reference_links) OR reference_links IS NULL)
);

CREATE INDEX idx_ideate_research_session ON ideate_research(session_id);

-- ============================================================================
-- COMPREHENSIVE MODE FEATURES
-- ============================================================================

-- ============================================================================
-- GENERATION TRACKING (Quick Mode)
-- ============================================================================

-- Track PRD generation progress for Quick Mode
CREATE TABLE ideate_generations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('generating', 'completed', 'failed')),
    generated_sections TEXT, -- JSON object: {"overview": "...", "features": "..."}
    current_section TEXT, -- Which section is currently being generated
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(generated_sections) OR generated_sections IS NULL)
);

CREATE INDEX idx_ideate_generations_session ON ideate_generations(session_id);
CREATE INDEX idx_ideate_generations_status ON ideate_generations(status);

-- ============================================================================
-- QUICKSTART TEMPLATES
-- ============================================================================

-- Templates for Quick Start
CREATE TABLE prd_quickstart_templates (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    name TEXT NOT NULL,
    description TEXT, -- Template description for UI display
    project_type TEXT, -- 'saas', 'mobile', 'api', 'marketplace', etc.
    one_liner_prompts TEXT, -- JSON array of prompts to expand one-liner
    default_features TEXT, -- JSON array of common features for this type
    default_dependencies TEXT, -- JSON object with typical dependency chains
    default_problem_statement TEXT,
    default_target_audience TEXT,
    default_value_proposition TEXT,
    default_ui_considerations TEXT,
    default_ux_principles TEXT,
    default_tech_stack_quick TEXT,
    default_mvp_scope TEXT,
    default_research_findings TEXT,
    default_technical_specs TEXT,
    default_competitors TEXT,
    default_similar_projects TEXT,
    is_system INTEGER DEFAULT 0, -- Boolean: system template vs user-created
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    CHECK (json_valid(one_liner_prompts) OR one_liner_prompts IS NULL),
    CHECK (json_valid(default_features) OR default_features IS NULL),
    CHECK (json_valid(default_dependencies) OR default_dependencies IS NULL),
    CHECK (json_valid(default_mvp_scope) OR default_mvp_scope IS NULL),
    CHECK (json_valid(default_competitors) OR default_competitors IS NULL),
    CHECK (json_valid(default_similar_projects) OR default_similar_projects IS NULL)
);

CREATE INDEX idx_prd_quickstart_templates_type ON prd_quickstart_templates(project_type);
CREATE INDEX idx_prd_quickstart_templates_system ON prd_quickstart_templates(is_system);

-- ============================================================================
-- DEPENDENCY INTELLIGENCE
-- ============================================================================

CREATE TABLE feature_dependencies (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    from_feature_id TEXT NOT NULL, -- Feature that depends on another
    to_feature_id TEXT NOT NULL, -- Feature that is depended upon
    dependency_type TEXT NOT NULL CHECK(dependency_type IN ('technical', 'logical', 'business')),
    strength TEXT DEFAULT 'required' CHECK(strength IN ('required', 'recommended', 'optional')),
    reason TEXT, -- Why this dependency exists
    auto_detected INTEGER DEFAULT 0, -- Boolean: was this detected by AI or manually added?
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (from_feature_id) REFERENCES ideate_features(id) ON DELETE CASCADE,
    FOREIGN KEY (to_feature_id) REFERENCES ideate_features(id) ON DELETE CASCADE,

    -- Prevent self-dependencies
    CHECK (from_feature_id != to_feature_id),

    -- Prevent duplicate dependencies
    UNIQUE(from_feature_id, to_feature_id)
);

CREATE INDEX idx_feature_dependencies_session ON feature_dependencies(session_id);
CREATE INDEX idx_feature_dependencies_from ON feature_dependencies(from_feature_id);
CREATE INDEX idx_feature_dependencies_to ON feature_dependencies(to_feature_id);
CREATE INDEX idx_feature_dependencies_type ON feature_dependencies(dependency_type);
CREATE INDEX idx_feature_dependencies_auto ON feature_dependencies(auto_detected);

-- ============================================================================
-- AI DEPENDENCY ANALYSIS CACHE
-- ============================================================================

-- Cache AI analysis results for performance
CREATE TABLE dependency_analysis_cache (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    features_hash TEXT NOT NULL, -- Hash of feature descriptions to detect changes
    analysis_type TEXT NOT NULL CHECK(analysis_type IN ('dependencies', 'build_order', 'visibility')),
    analysis_result TEXT NOT NULL, -- JSON object with analysis results
    confidence_score REAL, -- 0.0-1.0 confidence in analysis
    model_version TEXT, -- Which AI model was used
    analyzed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    expires_at TEXT, -- Optional cache expiration
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(analysis_result)),
    CHECK (confidence_score IS NULL OR (confidence_score >= 0.0 AND confidence_score <= 1.0))
);

CREATE INDEX idx_dependency_analysis_session ON dependency_analysis_cache(session_id);
CREATE INDEX idx_dependency_analysis_type ON dependency_analysis_cache(analysis_type);
CREATE INDEX idx_dependency_analysis_hash ON dependency_analysis_cache(features_hash);
CREATE INDEX idx_dependency_analysis_expires ON dependency_analysis_cache(expires_at);

-- ============================================================================
-- BUILD ORDER OPTIMIZATION
-- ============================================================================

-- Store computed build sequences
CREATE TABLE build_order_optimization (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    build_sequence TEXT NOT NULL, -- JSON array of feature IDs in build order
    parallel_groups TEXT, -- JSON array of arrays: features that can be built in parallel
    critical_path TEXT, -- JSON array of feature IDs on critical path
    estimated_phases INTEGER, -- Number of sequential phases needed
    optimization_strategy TEXT CHECK(optimization_strategy IN ('fastest', 'balanced', 'safest')),
    computed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    is_valid INTEGER DEFAULT 1, -- Boolean: becomes invalid when features/dependencies change
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(build_sequence)),
    CHECK (json_valid(parallel_groups) OR parallel_groups IS NULL),
    CHECK (json_valid(critical_path) OR critical_path IS NULL)
);

CREATE INDEX idx_build_order_session ON build_order_optimization(session_id);
CREATE INDEX idx_build_order_valid ON build_order_optimization(is_valid);
CREATE INDEX idx_build_order_strategy ON build_order_optimization(optimization_strategy);

-- ============================================================================
-- QUICK-WIN FEATURES ANALYSIS
-- ============================================================================

-- Track features identified as quick wins (high value, low dependency)
CREATE TABLE quick_win_features (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    feature_id TEXT NOT NULL,
    visibility_score REAL, -- How quickly users see value (0.0-1.0)
    dependency_count INTEGER DEFAULT 0, -- Number of dependencies
    complexity_score REAL, -- Estimated complexity (0.0-1.0, lower is simpler)
    value_score REAL, -- User value score (0.0-1.0, higher is better)
    overall_score REAL, -- Combined quick-win score
    reasoning TEXT, -- Why this is a quick win
    analyzed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (feature_id) REFERENCES ideate_features(id) ON DELETE CASCADE,
    CHECK (visibility_score IS NULL OR (visibility_score >= 0.0 AND visibility_score <= 1.0)),
    CHECK (complexity_score IS NULL OR (complexity_score >= 0.0 AND complexity_score <= 1.0)),
    CHECK (value_score IS NULL OR (value_score >= 0.0 AND value_score <= 1.0)),
    CHECK (overall_score IS NULL OR (overall_score >= 0.0 AND overall_score <= 1.0)),

    UNIQUE(session_id, feature_id)
);

CREATE INDEX idx_quick_win_session ON quick_win_features(session_id);
CREATE INDEX idx_quick_win_feature ON quick_win_features(feature_id);
CREATE INDEX idx_quick_win_score ON quick_win_features(overall_score DESC);
CREATE INDEX idx_quick_win_visibility ON quick_win_features(visibility_score DESC);

-- ============================================================================
-- CIRCULAR DEPENDENCY DETECTION
-- ============================================================================

-- Track detected circular dependencies for warnings
CREATE TABLE circular_dependencies (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    cycle_path TEXT NOT NULL, -- JSON array of feature IDs forming the cycle
    severity TEXT DEFAULT 'error' CHECK(severity IN ('warning', 'error', 'critical')),
    detected_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    resolved INTEGER DEFAULT 0, -- Boolean: has this cycle been resolved?
    resolution_note TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(cycle_path))
);

CREATE INDEX idx_circular_dependencies_session ON circular_dependencies(session_id);
CREATE INDEX idx_circular_dependencies_resolved ON circular_dependencies(resolved);
CREATE INDEX idx_circular_dependencies_severity ON circular_dependencies(severity);

-- ============================================================================
-- RESEARCH ANALYSIS CACHE
-- ============================================================================

CREATE TABLE IF NOT EXISTS competitor_analysis_cache (
    session_id TEXT NOT NULL,
    url TEXT NOT NULL,
    name TEXT NOT NULL,
    strengths TEXT NOT NULL,  -- JSON array: ["strength1", "strength2", ...]
    gaps TEXT NOT NULL,       -- JSON array: ["gap1", "gap2", ...]
    features TEXT NOT NULL,   -- JSON array: ["feature1", "feature2", ...]
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    PRIMARY KEY (session_id, url),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

-- Index for cache expiration queries
CREATE INDEX IF NOT EXISTS idx_competitor_cache_created
ON competitor_analysis_cache(created_at);

-- Index for session lookups
CREATE INDEX IF NOT EXISTS idx_competitor_cache_session
ON competitor_analysis_cache(session_id);

-- ============================================================================
-- EXPERT ROUNDTABLE SYSTEM
-- ============================================================================

CREATE TABLE IF NOT EXISTS expert_personas (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    role TEXT NOT NULL,              -- e.g., "Product Manager", "Senior Engineer"
    expertise TEXT NOT NULL,         -- JSON array: ["area1", "area2", ...]
    system_prompt TEXT NOT NULL,     -- AI system prompt defining expert behavior
    bio TEXT,                        -- Short bio/description of expert
    is_default BOOLEAN NOT NULL DEFAULT 0,  -- System default vs user-created
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(name, role)
);

-- ============================================================================
-- ROUNDTABLE SESSIONS
-- ============================================================================
-- Purpose: Track roundtable discussion sessions for each ideate session
-- Retention: Tied to ideate_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    session_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'setup',  -- setup, discussing, completed, cancelled
    topic TEXT NOT NULL,                   -- Discussion topic/focus
    num_experts INTEGER NOT NULL DEFAULT 3,
    moderator_persona TEXT,                -- Optional custom moderator
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (status IN ('setup', 'discussing', 'completed', 'cancelled')),
    CHECK (num_experts >= 2 AND num_experts <= 5)
);

-- ============================================================================
-- ROUNDTABLE PARTICIPANTS
-- ============================================================================
-- Purpose: Link experts to roundtable sessions (many-to-many relationship)
-- Retention: Tied to roundtable_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_participants (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    roundtable_id TEXT NOT NULL,
    expert_id TEXT NOT NULL,
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (roundtable_id) REFERENCES roundtable_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (expert_id) REFERENCES expert_personas(id) ON DELETE CASCADE,
    UNIQUE(roundtable_id, expert_id)
);

-- ============================================================================
-- ROUNDTABLE MESSAGES
-- ============================================================================
-- Purpose: Store chronological stream of discussion messages
-- Retention: Tied to roundtable_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_messages (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    roundtable_id TEXT NOT NULL,
    message_order INTEGER NOT NULL,        -- Sequence number for ordering
    role TEXT NOT NULL,                    -- expert, user, moderator, system
    expert_id TEXT,                        -- NULL for user/moderator/system messages
    expert_name TEXT,                      -- Denormalized for display
    content TEXT NOT NULL,
    metadata TEXT,                         -- JSON for additional data
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (roundtable_id) REFERENCES roundtable_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (expert_id) REFERENCES expert_personas(id) ON DELETE SET NULL,
    CHECK (role IN ('expert', 'user', 'moderator', 'system')),
    UNIQUE(roundtable_id, message_order)
);

-- ============================================================================
-- EXPERT SUGGESTIONS
-- ============================================================================
-- Purpose: Store AI-generated expert recommendations for each session
-- Retention: Tied to ideate_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS expert_suggestions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    session_id TEXT NOT NULL,
    expert_name TEXT NOT NULL,
    role TEXT NOT NULL,
    expertise_area TEXT NOT NULL,         -- Primary expertise relevant to project
    reason TEXT NOT NULL,                 -- Why this expert is recommended
    relevance_score REAL,                 -- 0.0-1.0 relevance score
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

-- ============================================================================
-- ROUNDTABLE INSIGHTS
-- ============================================================================
-- Purpose: Store extracted insights from roundtable discussions
-- Retention: Tied to roundtable_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_insights (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    roundtable_id TEXT NOT NULL,
    insight_text TEXT NOT NULL,
    category TEXT NOT NULL,              -- e.g., "Technical", "UX", "Business"
    priority TEXT NOT NULL DEFAULT 'medium',  -- low, medium, high, critical
    source_experts TEXT NOT NULL,        -- JSON array: ["expert1", "expert2"]
    source_message_ids TEXT,             -- JSON array: message IDs that support this
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (roundtable_id) REFERENCES roundtable_sessions(id) ON DELETE CASCADE,
    CHECK (priority IN ('low', 'medium', 'high', 'critical'))
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Expert persona lookups
CREATE INDEX IF NOT EXISTS idx_expert_personas_default
ON expert_personas(is_default);

-- Roundtable session lookups
CREATE INDEX IF NOT EXISTS idx_roundtable_sessions_session
ON roundtable_sessions(session_id);

CREATE INDEX IF NOT EXISTS idx_roundtable_sessions_status
ON roundtable_sessions(status);

-- Participant lookups
CREATE INDEX IF NOT EXISTS idx_roundtable_participants_roundtable
ON roundtable_participants(roundtable_id);

CREATE INDEX IF NOT EXISTS idx_roundtable_participants_expert
ON roundtable_participants(expert_id);

-- Message ordering and retrieval
CREATE INDEX IF NOT EXISTS idx_roundtable_messages_roundtable
ON roundtable_messages(roundtable_id, message_order);

CREATE INDEX IF NOT EXISTS idx_roundtable_messages_created
ON roundtable_messages(created_at);

-- Expert suggestion lookups
CREATE INDEX IF NOT EXISTS idx_expert_suggestions_session
ON expert_suggestions(session_id);

CREATE INDEX IF NOT EXISTS idx_expert_suggestions_relevance
ON expert_suggestions(relevance_score DESC);

-- Insight lookups
CREATE INDEX IF NOT EXISTS idx_roundtable_insights_roundtable
ON roundtable_insights(roundtable_id);

CREATE INDEX IF NOT EXISTS idx_roundtable_insights_priority
ON roundtable_insights(priority);

CREATE INDEX IF NOT EXISTS idx_roundtable_insights_category
ON roundtable_insights(category);

-- ============================================================================
-- SEED DATA: DEFAULT EXPERT PERSONAS
-- ============================================================================
-- Predefined expert personas for common use cases
-- These can be selected by users when creating roundtables

INSERT OR IGNORE INTO expert_personas (id, name, role, expertise, system_prompt, bio, is_default) VALUES
(
    'expert_product_manager',
    'Alex Chen',
    'Product Manager',
    '["product strategy", "user research", "roadmap planning", "stakeholder management"]',
    'You are Alex Chen, an experienced Product Manager with 10+ years in tech. You focus on user value, market fit, and business viability. Ask probing questions about user needs, priorities, and success metrics. Challenge assumptions and ensure features align with product vision.',
    'Seasoned PM who ensures features deliver real user value and business impact.',
    1
),
(
    'expert_senior_engineer',
    'Jordan Smith',
    'Senior Software Engineer',
    '["system design", "architecture", "performance", "scalability", "technical debt"]',
    'You are Jordan Smith, a Senior Software Engineer with deep expertise in system architecture. You analyze technical feasibility, scalability concerns, and implementation complexity. Point out potential technical risks, suggest architectural patterns, and estimate engineering effort realistically.',
    'Pragmatic engineer focused on building scalable, maintainable systems.',
    1
),
(
    'expert_ux_designer',
    'Maya Patel',
    'UX Designer',
    '["user experience", "interaction design", "accessibility", "usability", "design systems"]',
    'You are Maya Patel, a UX Designer passionate about intuitive, accessible interfaces. You advocate for user-centered design, question confusing flows, and ensure features are discoverable and delightful. Raise accessibility concerns and suggest design patterns that improve usability.',
    'User advocate who ensures products are intuitive and accessible to all.',
    1
),
(
    'expert_security',
    'Chris Johnson',
    'Security Engineer',
    '["application security", "data privacy", "threat modeling", "compliance", "authentication"]',
    'You are Chris Johnson, a Security Engineer focused on protecting user data and preventing vulnerabilities. You identify security risks, suggest secure implementation patterns, and ensure compliance with privacy regulations. Challenge features that introduce security concerns.',
    'Security-first engineer who proactively identifies and mitigates risks.',
    1
),
(
    'expert_data_scientist',
    'Dr. Sarah Lee',
    'Data Scientist',
    '["data analysis", "machine learning", "metrics", "experimentation", "insights"]',
    'You are Dr. Sarah Lee, a Data Scientist who brings analytical rigor to product decisions. You suggest metrics to track, design experiments to validate assumptions, and identify opportunities for data-driven features. Question unmeasurable goals and propose concrete success criteria.',
    'Data-driven thinker who turns insights into actionable product improvements.',
    1
),
(
    'expert_devops',
    'Taylor Martinez',
    'DevOps Engineer',
    '["infrastructure", "deployment", "monitoring", "reliability", "automation", "CI/CD"]',
    'You are Taylor Martinez, a DevOps Engineer who ensures systems are reliable, observable, and easy to deploy. You raise concerns about operational complexity, suggest monitoring strategies, and ensure features are deployable and maintainable in production.',
    'Operations expert focused on reliability, observability, and smooth deployments.',
    1
),
(
    'expert_qa',
    'Sam Kim',
    'QA Engineer',
    '["testing strategy", "edge cases", "quality assurance", "test automation", "bug prevention"]',
    'You are Sam Kim, a QA Engineer with a keen eye for edge cases and potential failures. You identify testing challenges, suggest test scenarios, and ensure features are thoroughly testable. Point out ambiguous requirements and potential user confusion.',
    'Quality guardian who finds issues before users do.',
    1
),
(
    'expert_researcher',
    'Dr. Jamie Wong',
    'User Researcher',
    '["user research", "behavioral psychology", "user interviews", "persona development", "journey mapping"]',
    'You are Dr. Jamie Wong, a User Researcher who deeply understands user behavior and motivations. You bring research insights, identify unmet user needs, and challenge assumptions about user behavior. Suggest research methods to validate hypotheses.',
    'Research expert who brings real user voices into product decisions.',
    1
),
(
    'expert_legal',
    'Avery Brown',
    'Legal Counsel',
    '["compliance", "privacy law", "terms of service", "intellectual property", "regulatory requirements"]',
    'You are Avery Brown, Legal Counsel specializing in tech products. You identify legal and compliance risks, ensure features meet regulatory requirements (GDPR, CCPA, etc.), and suggest terms of service implications. Raise concerns about liability and data handling.',
    'Legal expert who ensures products comply with laws and protect the company.',
    1
),
(
    'expert_performance',
    'Riley Thompson',
    'Performance Engineer',
    '["optimization", "profiling", "load testing", "caching", "database performance"]',
    'You are Riley Thompson, a Performance Engineer obsessed with speed and efficiency. You identify performance bottlenecks, suggest optimization strategies, and ensure features scale under load. Question resource-intensive features and propose efficient alternatives.',
    'Speed expert who ensures products are fast and responsive at scale.',
    1
);

-- ============================================================================
-- PRD GENERATION
-- ============================================================================

CREATE TABLE ideate_prd_generations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,

    -- Generated content
    generated_content TEXT, -- Full PRD JSON (GeneratedPRD structure)
    markdown_content TEXT,  -- Formatted markdown

    -- Generation metadata
    generation_method TEXT NOT NULL CHECK(generation_method IN ('full', 'sections', 'merged', 'ai_filled')),
    filled_sections TEXT, -- JSON array of sections that were AI-filled

    -- Validation status
    validation_status TEXT NOT NULL DEFAULT 'pending' CHECK(validation_status IN ('pending', 'valid', 'warnings', 'errors')),
    validation_details TEXT, -- JSON validation results

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(generated_content) OR generated_content IS NULL),
    CHECK (json_valid(filled_sections) OR filled_sections IS NULL),
    CHECK (json_valid(validation_details) OR validation_details IS NULL)
);

CREATE INDEX idx_prd_generations_session ON ideate_prd_generations(session_id);
CREATE INDEX idx_prd_generations_version ON ideate_prd_generations(session_id, version DESC);
CREATE INDEX idx_prd_generations_status ON ideate_prd_generations(validation_status);
CREATE INDEX idx_prd_generations_method ON ideate_prd_generations(generation_method);

CREATE TRIGGER ideate_prd_generations_updated_at AFTER UPDATE ON ideate_prd_generations
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE ideate_prd_generations SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- ============================================================================
-- EXPORT HISTORY
-- ============================================================================

-- Track PRD exports in various formats
CREATE TABLE ideate_exports (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    generation_id TEXT, -- Optional: links to specific generation version

    -- Export metadata
    format TEXT NOT NULL CHECK(format IN ('markdown', 'html', 'pdf', 'docx')),
    file_path TEXT, -- Relative or absolute path to exported file
    file_size_bytes INTEGER,

    -- Export options (JSON)
    export_options TEXT, -- JSON of ExportOptions

    -- Timestamps
    exported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (generation_id) REFERENCES ideate_prd_generations(id) ON DELETE SET NULL,
    CHECK (json_valid(export_options) OR export_options IS NULL)
);

CREATE INDEX idx_exports_session ON ideate_exports(session_id);
CREATE INDEX idx_exports_generation ON ideate_exports(generation_id);
CREATE INDEX idx_exports_format ON ideate_exports(format);
CREATE INDEX idx_exports_date ON ideate_exports(exported_at DESC);

-- ============================================================================
-- SECTION GENERATION HISTORY
-- ============================================================================

-- Track individual section generation/regeneration events
CREATE TABLE ideate_section_generations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    generation_id TEXT, -- Links to main generation record

    -- Section info
    section_name TEXT NOT NULL CHECK(section_name IN ('overview', 'features', 'ux', 'technical', 'roadmap', 'dependencies', 'risks', 'research')),
    section_content TEXT, -- JSON content for the section

    -- Generation context
    was_skipped INTEGER NOT NULL DEFAULT 0, -- Boolean: was this section originally skipped?
    was_ai_filled INTEGER NOT NULL DEFAULT 0, -- Boolean: was this filled by AI?
    context_used TEXT, -- Context string used for generation

    -- AI usage
    tokens_used INTEGER, -- Number of tokens used for generation
    model TEXT, -- AI model used

    -- Timestamps
    generated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (generation_id) REFERENCES ideate_prd_generations(id) ON DELETE SET NULL,
    CHECK (json_valid(section_content) OR section_content IS NULL)
);

CREATE INDEX idx_section_generations_session ON ideate_section_generations(session_id);
CREATE INDEX idx_section_generations_generation ON ideate_section_generations(generation_id);
CREATE INDEX idx_section_generations_section ON ideate_section_generations(section_name);
CREATE INDEX idx_section_generations_ai_filled ON ideate_section_generations(was_ai_filled);
CREATE INDEX idx_section_generations_date ON ideate_section_generations(generated_at DESC);

-- ============================================================================
-- VALIDATION RULES
-- ============================================================================

-- Store PRD validation rules and results
CREATE TABLE ideate_validation_rules (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),

    -- Rule definition
    rule_name TEXT NOT NULL UNIQUE,
    rule_type TEXT NOT NULL CHECK(rule_type IN ('required_field', 'min_length', 'max_length', 'format', 'consistency', 'completeness')),
    section TEXT, -- Which section this applies to (NULL = all sections)
    field_path TEXT, -- JSON path to field (e.g., "overview.problem_statement")

    -- Rule parameters (JSON)
    rule_params TEXT, -- e.g., {"min_length": 50, "max_length": 500}

    -- Rule metadata
    severity TEXT NOT NULL DEFAULT 'warning' CHECK(severity IN ('error', 'warning', 'info')),
    error_message TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1, -- Boolean: is this rule currently enforced?

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    CHECK (json_valid(rule_params) OR rule_params IS NULL)
);

CREATE INDEX idx_validation_rules_section ON ideate_validation_rules(section);
CREATE INDEX idx_validation_rules_type ON ideate_validation_rules(rule_type);
CREATE INDEX idx_validation_rules_severity ON ideate_validation_rules(severity);
CREATE INDEX idx_validation_rules_active ON ideate_validation_rules(is_active);

-- Seed default validation rules
INSERT OR IGNORE INTO ideate_validation_rules (id, rule_name, rule_type, section, field_path, rule_params, severity, error_message)
VALUES
    ('val_rule_1', 'overview_problem_required', 'required_field', 'overview', 'problem_statement', NULL, 'error', 'Problem statement is required'),
    ('val_rule_2', 'overview_audience_required', 'required_field', 'overview', 'target_audience', NULL, 'error', 'Target audience is required'),
    ('val_rule_3', 'overview_value_required', 'required_field', 'overview', 'value_proposition', NULL, 'error', 'Value proposition is required'),
    ('val_rule_4', 'problem_min_length', 'min_length', 'overview', 'problem_statement', '{"min_length": 50}', 'warning', 'Problem statement should be at least 50 characters'),
    ('val_rule_5', 'features_min_count', 'completeness', 'features', NULL, '{"min_count": 3}', 'warning', 'PRD should have at least 3 core features'),
    ('val_rule_6', 'roadmap_mvp_required', 'required_field', 'roadmap', 'mvp_scope', NULL, 'warning', 'MVP scope should be defined'),
    ('val_rule_7', 'dependencies_foundation_required', 'completeness', 'dependencies', 'foundation_features', '{"min_count": 1}', 'warning', 'At least one foundation feature should be identified');

-- ============================================================================
-- GENERATION STATISTICS
-- ============================================================================

-- Track generation statistics for analytics
CREATE TABLE ideate_generation_stats (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    generation_id TEXT NOT NULL,

    -- Token usage
    total_tokens_used INTEGER NOT NULL DEFAULT 0,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,

    -- Timing
    generation_duration_ms INTEGER, -- Time taken to generate in milliseconds

    -- Sections
    sections_generated INTEGER NOT NULL DEFAULT 0,
    sections_skipped INTEGER NOT NULL DEFAULT 0,
    sections_ai_filled INTEGER NOT NULL DEFAULT 0,

    -- Quality metrics
    completeness_score REAL, -- 0.0 to 1.0
    validation_score REAL, -- 0.0 to 1.0

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (generation_id) REFERENCES ideate_prd_generations(id) ON DELETE CASCADE
);

CREATE INDEX idx_generation_stats_session ON ideate_generation_stats(session_id);
CREATE INDEX idx_generation_stats_generation ON ideate_generation_stats(generation_id);
CREATE INDEX idx_generation_stats_date ON ideate_generation_stats(created_at DESC);
CREATE INDEX idx_generation_stats_completeness ON ideate_generation_stats(completeness_score DESC);

-- ============================================================================
-- IDEATE TEMPLATES
-- ============================================================================

INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_saas',
    'SaaS Application',
    'Perfect for building web-based software as a service applications with subscription models, user management, and team collaboration features.',
    'saas',
    '["What problem does this SaaS solve for users?", "What makes your SaaS unique in the market?", "Who is your primary target customer?", "What pricing model will you use (freemium, tiered, per-seat)?"]',
    '["User authentication and authorization", "Subscription billing and payments", "Team/workspace management", "Admin dashboard", "User settings and profiles", "Email notifications", "API access", "Activity logging and audit trails"]',
    '{"authentication": ["billing", "teams"], "billing": ["dashboard", "user-profiles"], "teams": ["workspace-management", "permissions"], "dashboard": ["analytics", "reporting"]}',
    1,
    strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
);

-- Template 2: Mobile App
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_mobile',
    'Mobile App (iOS/Android)',
    'Ideal for native or cross-platform mobile applications with offline capabilities, push notifications, and device integration.',
    'mobile',
    '["What core functionality do users need on mobile?", "Will this work offline or require constant connectivity?", "What device features will you use (camera, GPS, etc)?", "Is this iOS-only, Android-only, or cross-platform?"]',
    '["User onboarding flow", "Push notifications", "Offline mode and sync", "Device permissions handling", "In-app purchases (optional)", "Social sharing", "User profiles", "Settings and preferences", "Pull-to-refresh", "Deep linking"]',
    '{"onboarding": ["auth", "user-profile"], "auth": ["user-profile", "settings"], "offline-mode": ["sync-engine", "local-storage"], "push-notifications": ["user-settings", "notification-center"], "in-app-purchases": ["billing-system"]}',
    1,
    strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
);

-- Template 3: API/Backend Service
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_backend',
    'API / Backend Service',
    'Best for building RESTful or GraphQL APIs, microservices, and backend infrastructure with authentication, rate limiting, and comprehensive documentation.',
    'api',
    '["What data will your API manage and expose?", "Who are the API consumers (internal apps, third parties, both)?", "What authentication method will you use (API keys, OAuth, JWT)?", "What is your expected scale and performance requirements?"]',
    '["RESTful endpoints or GraphQL schema", "Authentication and authorization (JWT, OAuth)", "Rate limiting and throttling", "API documentation (Swagger/OpenAPI)", "Versioning strategy", "Error handling and logging", "Database integration", "Caching layer", "Webhooks", "Monitoring and health checks"]',
    '{"auth": ["endpoints", "middleware"], "rate-limiting": ["auth", "middleware"], "database": ["models", "migrations"], "caching": ["database"], "monitoring": ["logging", "health-checks"], "documentation": ["endpoints"]}',
    1,
    strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
);

-- Template 4: Marketplace/Platform
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_marketplace',
    'Marketplace / Two-Sided Platform',
    'For building platforms connecting buyers and sellers, service providers and customers, or any two-sided marketplace with transactions and reviews.',
    'marketplace',
    '["Who are the two sides of your marketplace (buyers/sellers, hosts/guests, etc)?", "What gets exchanged (products, services, bookings, etc)?", "How will you handle payments and transactions?", "What trust and safety features are needed?"]',
    '["User profiles (buyer and seller)", "Listing creation and management", "Search and filtering", "Messaging between users", "Payment processing and escrow", "Reviews and ratings system", "Dispute resolution workflow", "Commission/fee structure", "Trust and safety features", "Analytics dashboard for sellers"]',
    '{"user-profiles": ["authentication", "verification"], "listings": ["user-profiles", "search"], "search": ["listings", "filters"], "messaging": ["user-profiles"], "payments": ["listings", "escrow"], "reviews": ["payments", "user-profiles"], "dispute-resolution": ["messaging", "admin-tools"], "analytics": ["payments", "listings"]}',
    1,
    strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
);

-- Template 5: Internal Tool / Dashboard
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_internal',
    'Internal Tool / Dashboard',
    'Perfect for building internal enterprise tools, admin panels, data visualization dashboards, and business intelligence applications.',
    'internal-tool',
    '["What business process or workflow does this tool support?", "Who will use this tool (which departments or roles)?", "What data sources will it connect to?", "What actions can users perform through this tool?"]',
    '["Role-based access control (RBAC)", "Data visualization and charts", "Reporting and exports", "Bulk data operations", "Audit logs", "Search and filtering", "Data import/export", "Notifications and alerts", "Workflow automation", "Integration with existing systems"]',
    '{"rbac": ["authentication", "permissions"], "data-viz": ["data-access", "charting-library"], "reporting": ["data-access", "export-engine"], "bulk-operations": ["data-validation", "queuing"], "audit-logs": ["user-actions", "logging"], "automation": ["triggers", "actions"], "integrations": ["api-client", "webhooks"]}',
    1,
    strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
);

-- ============================================================================
-- PRD OUTPUT TEMPLATES
-- ============================================================================

CREATE TABLE prd_output_templates (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),

    -- Template metadata
    name TEXT NOT NULL,
    description TEXT,

    -- Template content (markdown format)
    content TEXT NOT NULL,

    -- Template settings
    is_default INTEGER NOT NULL DEFAULT 0, -- Boolean: is this the default template?

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Indexes for performance
CREATE INDEX idx_prd_output_templates_name ON prd_output_templates(name);
CREATE INDEX idx_prd_output_templates_default ON prd_output_templates(is_default);
CREATE INDEX idx_prd_output_templates_created ON prd_output_templates(created_at DESC);

-- Auto-update updated_at timestamp
CREATE TRIGGER prd_output_templates_updated_at AFTER UPDATE ON prd_output_templates
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE prd_output_templates SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- ============================================================================
-- SEED DATA: Default Template
-- ============================================================================

-- Standard PRD template (idempotent insert)
INSERT OR IGNORE INTO prd_output_templates (id, name, description, content, is_default)
VALUES (
    'standard',
    'Standard PRD',
    'Default template for general product requirements',
    '# Product Requirements Document

## Overview

{{overview}}

## Core Features

{{features}}

## Technical Architecture

{{technical}}

## User Experience

{{ux}}

## Roadmap

{{roadmap}}

## Dependencies & Build Order

{{dependencies}}

## Risks & Mitigations

{{risks}}

## Research & Competitive Analysis

{{research}}
',
    1
);

-- ============================================================================
-- Populate defaults for existing system templates
-- ============================================================================

-- Template 1: SaaS Application
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the core problem or pain point your SaaS solves for users',
  default_target_audience = 'Define your primary users: business teams, enterprises, SMBs, or specific verticals',
  default_value_proposition = 'Explain the unique value and competitive advantage of your SaaS solution',
  default_ui_considerations = 'Clean, professional interface with intuitive navigation. Dashboard-centric design with real-time data updates. Mobile-responsive for on-the-go access.',
  default_ux_principles = 'Simplicity and efficiency. Minimize clicks to complete tasks. Provide contextual help and onboarding. Enable power users with keyboard shortcuts and advanced features.',
  default_tech_stack_quick = 'Frontend: React/Vue/Angular with TypeScript. Backend: Node.js/Python/Go. Database: PostgreSQL or MongoDB. Hosting: AWS/GCP/Azure. Authentication: OAuth 2.0 or SAML.',
  default_mvp_scope = '["User authentication and authorization", "Subscription billing and payments", "Team/workspace management", "Admin dashboard", "User settings and profiles"]',
  default_research_findings = 'Market analysis shows growing demand for specialized SaaS solutions. Key competitors: [list competitors]. Market size: [estimate]. Growth rate: [percentage].',
  default_technical_specs = 'API-first architecture. RESTful or GraphQL endpoints. Real-time sync with WebSockets. Horizontal scalability with microservices.',
  default_competitors = '["Competitor A - brief description", "Competitor B - brief description", "Competitor C - brief description"]',
  default_similar_projects = '["Similar project 1 - what to learn from it", "Similar project 2 - what to learn from it"]'
WHERE id = 'tpl_saas';

-- Template 2: Mobile App
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the core problem your mobile app solves and why mobile is the right platform',
  default_target_audience = 'Define your users: iOS only, Android only, or both. Age range, tech-savviness, use cases.',
  default_value_proposition = 'Explain the unique value of your mobile app compared to web alternatives',
  default_ui_considerations = 'Touch-friendly interface with large tap targets. Minimal text, icon-driven navigation. Bottom tab bar for main features. Native platform conventions (iOS vs Android).',
  default_ux_principles = 'Mobile-first thinking. Offline-first where possible. Fast load times. Minimal data usage. Intuitive gestures and animations.',
  default_tech_stack_quick = 'React Native or Flutter for cross-platform. Native Swift (iOS) or Kotlin (Android) for platform-specific. Firebase or similar for backend. SQLite for local storage.',
  default_mvp_scope = '["User onboarding flow", "Core feature 1", "Core feature 2", "Push notifications", "User profiles"]',
  default_research_findings = 'Mobile-first users expect fast, lightweight apps. Key competitors: [list]. App store trends: [insights]. User retention rates: [data].',
  default_technical_specs = 'Native APIs for camera, GPS, contacts. Offline sync with backend. Push notification service integration. App store deployment pipeline.',
  default_competitors = '["Competitor A - features and ratings", "Competitor B - features and ratings"]',
  default_similar_projects = '["Open source project 1 - architecture lessons", "Open source project 2 - design patterns"]'
WHERE id = 'tpl_mobile';

-- Template 3: API/Backend Service
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the data or service your API exposes and the problem it solves for consumers',
  default_target_audience = 'Define API consumers: internal apps, third-party developers, mobile clients, or all',
  default_value_proposition = 'Explain why developers should use your API over alternatives',
  default_ui_considerations = 'N/A - API only. Focus on developer experience: clear documentation, SDKs, interactive API explorer.',
  default_ux_principles = 'Developer experience first. Consistent API design. Clear error messages. Comprehensive documentation. Easy authentication and rate limiting.',
  default_tech_stack_quick = 'Node.js/Python/Go/Rust for API server. Express/FastAPI/Gin for framework. PostgreSQL/MongoDB for data. Redis for caching. Docker for deployment.',
  default_mvp_scope = '["Core endpoints", "Authentication (API keys or OAuth)", "Rate limiting", "API documentation", "Error handling"]',
  default_research_findings = 'API market trends: [insights]. Popular API patterns: REST vs GraphQL. Developer preferences: [data]. Monetization models: [options].',
  default_technical_specs = 'RESTful or GraphQL architecture. JWT or API key authentication. Versioning strategy (URL or header). Webhook support for events. OpenAPI/GraphQL schema.',
  default_competitors = '["Competitor API 1 - endpoints and features", "Competitor API 2 - pricing and limits"]',
  default_similar_projects = '["Open API 1 - design patterns", "Open API 2 - best practices"]'
WHERE id = 'tpl_backend';

-- Template 4: Marketplace/Platform
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the problem your marketplace solves by connecting two sides',
  default_target_audience = 'Define both sides: buyers/sellers, hosts/guests, service providers/customers. Include demographics and behaviors.',
  default_value_proposition = 'Explain the unique value for both sides of the marketplace',
  default_ui_considerations = 'Dual-sided interface design. Seller dashboard for listings and analytics. Buyer interface for discovery and checkout. Admin dashboard for moderation.',
  default_ux_principles = 'Trust and transparency. Easy listing creation for sellers. Intuitive search and filtering for buyers. Secure payment flow. Clear dispute resolution.',
  default_tech_stack_quick = 'Frontend: React for web, React Native for mobile. Backend: Node.js/Python. Database: PostgreSQL. Payment: Stripe/PayPal. Search: Elasticsearch.',
  default_mvp_scope = '["User profiles (buyer and seller)", "Listing creation and management", "Search and filtering", "Payment processing", "Reviews and ratings"]',
  default_research_findings = 'Marketplace trends: [insights]. Successful models: [examples]. Commission structures: [analysis]. User acquisition: [strategies].',
  default_technical_specs = 'Dual-sided authentication. Escrow payment system. Real-time notifications. Search and recommendation engine. Dispute resolution workflow.',
  default_competitors = '["Competitor 1 - commission, features, market share", "Competitor 2 - commission, features, market share"]',
  default_similar_projects = '["Successful marketplace 1 - what worked", "Successful marketplace 2 - lessons learned"]'
WHERE id = 'tpl_marketplace';

-- Template 5: Internal Tool / Dashboard
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the business process or workflow this internal tool streamlines',
  default_target_audience = 'Define users: which departments, roles, or teams. Their technical proficiency.',
  default_value_proposition = 'Explain time/cost savings and efficiency gains for the organization',
  default_ui_considerations = 'Enterprise-grade interface. Data-dense but organized. Keyboard shortcuts for power users. Customizable dashboards. Print-friendly reports.',
  default_ux_principles = 'Efficiency and power. Minimize clicks for common tasks. Bulk operations. Keyboard navigation. Audit trails for compliance.',
  default_tech_stack_quick = 'Frontend: React with TypeScript. Backend: Node.js or Python. Database: PostgreSQL. Authentication: LDAP/Active Directory integration. Hosting: On-premise or cloud.',
  default_mvp_scope = '["Role-based access control (RBAC)", "Core data views", "Key workflows", "Reporting and exports", "Audit logs"]',
  default_research_findings = 'Internal tool adoption: [factors]. ROI metrics: [examples]. Integration needs: [systems]. Compliance requirements: [standards].',
  default_technical_specs = 'Enterprise authentication (LDAP, SAML). Data export formats (CSV, Excel, PDF). API for third-party integrations. Audit logging for compliance.',
  default_competitors = '["Commercial tool 1 - features and cost", "Commercial tool 2 - features and cost"]',
  default_similar_projects = '["Internal tool 1 - architecture", "Internal tool 2 - lessons learned"]'
WHERE id = 'tpl_internal';

-- ============================================================================
-- CONVERSATIONAL MODE (CCPM) TABLES
-- ============================================================================

-- PRD Conversation History
CREATE TABLE prd_conversations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    prd_id TEXT,
    message_order INTEGER NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    message_type TEXT CHECK(message_type IN ('discovery', 'refinement', 'validation', 'general')),
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE CASCADE,
    UNIQUE(session_id, message_order),
    CHECK (json_valid(metadata) OR metadata IS NULL)
);

CREATE INDEX idx_prd_conversations_session ON prd_conversations(session_id);
CREATE INDEX idx_prd_conversations_prd ON prd_conversations(prd_id);
CREATE INDEX idx_prd_conversations_order ON prd_conversations(session_id, message_order);
CREATE INDEX idx_prd_conversations_type ON prd_conversations(message_type);

-- Epic Management
CREATE TABLE epics (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,
    prd_id TEXT NOT NULL,
    name TEXT NOT NULL,

    -- Epic content (markdown stored in DB)
    overview_markdown TEXT NOT NULL,
    architecture_decisions TEXT,
    technical_approach TEXT NOT NULL,
    implementation_strategy TEXT,
    dependencies TEXT,
    success_criteria TEXT,

    -- Task breakdown metadata
    task_categories TEXT,
    estimated_effort TEXT CHECK(estimated_effort IN ('days', 'weeks', 'months')),
    complexity TEXT CHECK(complexity IN ('low', 'medium', 'high', 'very_high')),

    -- Status tracking
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'ready', 'in_progress', 'blocked', 'completed', 'cancelled')),
    progress_percentage INTEGER DEFAULT 0 CHECK(progress_percentage >= 0 AND progress_percentage <= 100),

    -- GitHub integration
    github_issue_number INTEGER,
    github_issue_url TEXT,
    github_synced_at TEXT,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at TEXT,
    completed_at TEXT,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE CASCADE,
    CHECK (json_valid(architecture_decisions) OR architecture_decisions IS NULL),
    CHECK (json_valid(dependencies) OR dependencies IS NULL),
    CHECK (json_valid(success_criteria) OR success_criteria IS NULL),
    CHECK (json_valid(task_categories) OR task_categories IS NULL)
);

CREATE INDEX idx_epics_project ON epics(project_id);
CREATE INDEX idx_epics_prd ON epics(prd_id);
CREATE INDEX idx_epics_status ON epics(status);
CREATE INDEX idx_epics_progress ON epics(progress_percentage);
CREATE INDEX idx_epics_github ON epics(github_issue_number);

-- Epic update trigger
CREATE TRIGGER epics_updated_at AFTER UPDATE ON epics
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE epics SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- GitHub Synchronization Tracking
CREATE TABLE github_sync (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK(entity_type IN ('epic', 'task', 'comment', 'status')),
    entity_id TEXT NOT NULL,
    github_issue_number INTEGER,
    github_issue_url TEXT,
    sync_status TEXT DEFAULT 'pending' CHECK(sync_status IN ('pending', 'syncing', 'synced', 'failed', 'conflict')),
    sync_direction TEXT CHECK(sync_direction IN ('local_to_github', 'github_to_local', 'bidirectional')),
    last_synced_at TEXT,
    last_sync_hash TEXT,
    last_sync_error TEXT,
    retry_count INTEGER DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(entity_type, entity_id)
);

CREATE INDEX idx_github_sync_project ON github_sync(project_id);
CREATE INDEX idx_github_sync_entity ON github_sync(entity_type, entity_id);
CREATE INDEX idx_github_sync_status ON github_sync(sync_status);
CREATE INDEX idx_github_sync_issue ON github_sync(github_issue_number);
CREATE INDEX idx_github_sync_pending ON github_sync(sync_status) WHERE sync_status = 'pending';

-- GitHub sync update trigger
CREATE TRIGGER github_sync_updated_at AFTER UPDATE ON github_sync
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE github_sync SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;

-- Work Stream Analysis for Parallel Execution
CREATE TABLE work_analysis (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    epic_id TEXT NOT NULL,

    -- Analysis results (all JSON)
    parallel_streams TEXT NOT NULL,
    file_patterns TEXT,
    dependency_graph TEXT NOT NULL,
    conflict_analysis TEXT,
    parallelization_strategy TEXT,

    -- Metadata
    analyzed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    is_current BOOLEAN DEFAULT TRUE,
    analysis_version INTEGER DEFAULT 1,
    confidence_score REAL CHECK(confidence_score >= 0.0 AND confidence_score <= 1.0),

    FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE,
    CHECK (json_valid(parallel_streams)),
    CHECK (json_valid(file_patterns) OR file_patterns IS NULL),
    CHECK (json_valid(dependency_graph)),
    CHECK (json_valid(conflict_analysis) OR conflict_analysis IS NULL),
    CHECK (json_valid(parallelization_strategy) OR parallelization_strategy IS NULL)
);

CREATE INDEX idx_work_analysis_epic ON work_analysis(epic_id);
CREATE INDEX idx_work_analysis_current ON work_analysis(epic_id, is_current);

-- Reusable Discovery Questions for Conversational Mode
CREATE TABLE discovery_questions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    category TEXT NOT NULL CHECK(category IN ('problem', 'users', 'features', 'technical', 'risks', 'constraints', 'success')),
    question_text TEXT NOT NULL,
    follow_up_prompts TEXT,
    context_keywords TEXT,
    priority INTEGER DEFAULT 5 CHECK(priority >= 1 AND priority <= 10),
    is_required BOOLEAN DEFAULT FALSE,
    display_order INTEGER,
    is_active BOOLEAN DEFAULT TRUE,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_discovery_questions_category ON discovery_questions(category, display_order);
CREATE INDEX idx_discovery_questions_active ON discovery_questions(is_active, priority DESC);
CREATE INDEX idx_discovery_questions_required ON discovery_questions(is_required) WHERE is_required = TRUE;

-- Extracted Insights from Conversations
CREATE TABLE conversation_insights (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    insight_type TEXT NOT NULL CHECK(insight_type IN ('requirement', 'constraint', 'risk', 'assumption', 'decision')),
    insight_text TEXT NOT NULL,
    confidence_score REAL CHECK(confidence_score >= 0.0 AND confidence_score <= 1.0),
    source_message_ids TEXT,
    applied_to_prd BOOLEAN DEFAULT FALSE,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(source_message_ids) OR source_message_ids IS NULL)
);

CREATE INDEX idx_conversation_insights_session ON conversation_insights(session_id);
CREATE INDEX idx_conversation_insights_type ON conversation_insights(insight_type);
CREATE INDEX idx_conversation_insights_applied ON conversation_insights(applied_to_prd);

-- ============================================================================
-- SEED DATA: Default Discovery Questions for Conversational Mode
-- ============================================================================

INSERT OR IGNORE INTO discovery_questions (id, category, question_text, priority, is_required, display_order) VALUES
('dq-prob-1', 'problem', 'What specific problem are you trying to solve?', 10, TRUE, 1),
('dq-prob-2', 'problem', 'Who experiences this problem most acutely?', 9, TRUE, 2),
('dq-prob-3', 'problem', 'What happens if this problem isn''t solved?', 7, FALSE, 3),
('dq-user-1', 'users', 'Who are your primary users or customers?', 10, TRUE, 1),
('dq-user-2', 'users', 'What are their main goals and pain points?', 9, TRUE, 2),
('dq-user-3', 'users', 'How do they currently solve this problem?', 8, FALSE, 3),
('dq-feat-1', 'features', 'What are the must-have features for MVP?', 10, TRUE, 1),
('dq-feat-2', 'features', 'What features would delight users but aren''t essential?', 6, FALSE, 2),
('dq-feat-3', 'features', 'Are there features you explicitly want to avoid?', 5, FALSE, 3),
('dq-tech-1', 'technical', 'Do you have any technical constraints or requirements?', 8, FALSE, 1),
('dq-tech-2', 'technical', 'What existing systems need to integrate with this?', 7, FALSE, 2),
('dq-tech-3', 'technical', 'Are there performance or scalability requirements?', 6, FALSE, 3),
('dq-risk-1', 'risks', 'What are the biggest risks to this project?', 8, FALSE, 1),
('dq-risk-2', 'risks', 'What would cause this project to fail?', 7, FALSE, 2),
('dq-cons-1', 'constraints', 'What is your timeline for this project?', 9, TRUE, 1),
('dq-cons-2', 'constraints', 'Do you have budget or resource constraints?', 7, FALSE, 2),
('dq-succ-1', 'success', 'How will you measure success?', 9, TRUE, 1),
('dq-succ-2', 'success', 'What does "done" look like for the MVP?', 8, TRUE, 2);
