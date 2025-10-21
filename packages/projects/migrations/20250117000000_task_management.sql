-- ABOUTME: Task management system with users, agents, tasks, tags, and execution tracking
-- ABOUTME: Includes agent execution history, PR reviews, and AI gateway configuration

-- Users table
CREATE TABLE IF NOT EXISTS users (
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
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Seed default user
INSERT OR IGNORE INTO users (id, email, name, created_at, updated_at)
VALUES ('default-user', 'user@localhost', 'Default User', datetime('now', 'utc'), datetime('now', 'utc'));

-- Agents table
CREATE TABLE IF NOT EXISTS agents (
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

-- Seed AI agents
INSERT OR IGNORE INTO agents (id, name, type, provider, model, display_name, description, cost_per_1k_input_tokens, cost_per_1k_output_tokens, max_context_tokens, supports_tools, supports_vision, supports_web_search, created_at, updated_at)
VALUES
    ('claude-code', 'claude-code', 'ai', 'anthropic', 'claude-3-5-sonnet-20241022', 'Claude Code', 'Optimized for software development with MCP tools', 0.003, 0.015, 200000, 1, 1, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('claude-sonnet', 'claude-sonnet', 'ai', 'anthropic', 'claude-3-5-sonnet-20241022', 'Claude Sonnet 3.5', 'Balanced performance and intelligence', 0.003, 0.015, 200000, 1, 1, 0, datetime('now', 'utc'), datetime('now', 'utc')),
    ('claude-opus', 'claude-opus', 'ai', 'anthropic', 'claude-3-opus-20240229', 'Claude Opus 3', 'Most capable Claude model', 0.015, 0.075, 200000, 1, 1, 0, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gpt-4o', 'gpt-4o', 'ai', 'openai', 'gpt-4o', 'GPT-4o', 'Multimodal GPT-4 optimized for speed', 0.0025, 0.01, 128000, 1, 1, 0, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gpt-4-turbo', 'gpt-4-turbo', 'ai', 'openai', 'gpt-4-turbo-preview', 'GPT-4 Turbo', 'Latest GPT-4 with improved performance', 0.01, 0.03, 128000, 1, 1, 0, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gemini-pro', 'gemini-pro', 'ai', 'google', 'gemini-1.5-pro-latest', 'Gemini 1.5 Pro', 'Google''s most capable multimodal AI', 0.00125, 0.005, 2000000, 1, 1, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('gemini-flash', 'gemini-flash', 'ai', 'google', 'gemini-1.5-flash-latest', 'Gemini 1.5 Flash', 'Fast and efficient for high-volume tasks', 0.000075, 0.0003, 1000000, 1, 1, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('grok-2', 'grok-2', 'ai', 'xai', 'grok-2-latest', 'Grok 2', 'xAI''s flagship conversational AI', 0.002, 0.01, 131072, 1, 0, 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('grok-beta', 'grok-beta', 'ai', 'xai', 'grok-beta', 'Grok Beta', 'Experimental Grok model', 0.005, 0.015, 131072, 1, 0, 1, datetime('now', 'utc'), datetime('now', 'utc'));

-- User-Agent association table
CREATE TABLE IF NOT EXISTS user_agents (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    is_active INTEGER NOT NULL DEFAULT 1,
    custom_settings TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(user_id, agent_id)
);

-- Seed default user's active agents
INSERT OR IGNORE INTO user_agents (id, user_id, agent_id, is_active, created_at, updated_at)
VALUES
    ('ua-1', 'default-user', 'claude-code', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-2', 'default-user', 'claude-sonnet', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-3', 'default-user', 'gpt-4o', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-4', 'default-user', 'gemini-pro', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-5', 'default-user', 'gemini-flash', 1, datetime('now', 'utc'), datetime('now', 'utc')),
    ('ua-6', 'default-user', 'grok-2', 1, datetime('now', 'utc'), datetime('now', 'utc'));

-- Update default user to use Claude Code as default agent
UPDATE users SET default_agent_id = 'claude-code' WHERE id = 'default-user';

-- Tags table for organizing tasks
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    description TEXT,
    created_at TEXT NOT NULL,
    archived_at TEXT
);

-- Create index for active tags
CREATE INDEX IF NOT EXISTS idx_tags_archived_at ON tags(archived_at);

-- Insert default "main" tag
INSERT OR IGNORE INTO tags (id, name, color, description, created_at)
VALUES ('tag-main', 'main', '#3b82f6', 'Default tag for general tasks', datetime('now', 'utc'));

-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    priority TEXT NOT NULL DEFAULT 'medium',
    created_by_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE SET NULL DEFAULT 'default-user',
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
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Index for task queries
CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_parent_id ON tasks(parent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned_agent_id ON tasks(assigned_agent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_created_by_user_id ON tasks(created_by_user_id);
CREATE INDEX IF NOT EXISTS idx_tasks_tag_id ON tasks(tag_id);

-- Full-text search for tasks
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
    task_id UNINDEXED,
    title,
    description,
    details,
    tags,
    content='tasks',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts(rowid, task_id, title, description, details, tags)
    VALUES (new.rowid, new.id, new.title, new.description, new.details, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER IF NOT EXISTS tasks_fts_update AFTER UPDATE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE rowid = old.rowid;
    INSERT INTO tasks_fts(rowid, task_id, title, description, details, tags)
    VALUES (new.rowid, new.id, new.title, new.description, new.details, new.tags);
END;

-- Agent executions table for tracking multiple AI attempts per task
CREATE TABLE IF NOT EXISTS agent_executions (
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

-- Indexes for agent executions
CREATE INDEX IF NOT EXISTS idx_agent_executions_task_id ON agent_executions(task_id);
CREATE INDEX IF NOT EXISTS idx_agent_executions_agent_id ON agent_executions(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_executions_status ON agent_executions(status);
CREATE INDEX IF NOT EXISTS idx_agent_executions_pr_number ON agent_executions(pr_number);

-- PR reviews table for detailed review tracking
CREATE TABLE IF NOT EXISTS pr_reviews (
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

-- Indexes for PR reviews
CREATE INDEX IF NOT EXISTS idx_pr_reviews_execution_id ON pr_reviews(execution_id);
CREATE INDEX IF NOT EXISTS idx_pr_reviews_reviewer_id ON pr_reviews(reviewer_id);
CREATE INDEX IF NOT EXISTS idx_pr_reviews_status ON pr_reviews(review_status);

-- Migrate existing tasks with tags to use the new tag system
WITH parsed_tags AS (
    SELECT
        id as task_id,
        TRIM(REPLACE(REPLACE(
            SUBSTR(tags, 3, INSTR(tags || '","', '","') - 3),
            '[', ''), '"', '')) as tag_name
    FROM tasks
    WHERE tags IS NOT NULL
      AND tags != '[]'
      AND tags != ''
      AND tag_id IS NULL
)
INSERT OR IGNORE INTO tags (id, name, color, created_at)
SELECT
    'tag-' || LOWER(REPLACE(tag_name, ' ', '-')),
    tag_name,
    CASE
        WHEN tag_name LIKE '%feature%' THEN '#10b981'
        WHEN tag_name LIKE '%bug%' THEN '#ef4444'
        WHEN tag_name LIKE '%fix%' THEN '#ef4444'
        WHEN tag_name LIKE '%refactor%' THEN '#f59e0b'
        WHEN tag_name LIKE '%test%' THEN '#8b5cf6'
        WHEN tag_name LIKE '%docs%' THEN '#6b7280'
        ELSE '#3b82f6'
    END,
    datetime('now', 'utc')
FROM parsed_tags
WHERE tag_name IS NOT NULL AND tag_name != '';

-- Update tasks to reference the new tag_id
WITH parsed_tags AS (
    SELECT
        id as task_id,
        'tag-' || LOWER(REPLACE(
            TRIM(REPLACE(REPLACE(
                SUBSTR(tags, 3, INSTR(tags || '","', '","') - 3),
                '[', ''), '"', '')), ' ', '-')) as tag_id
    FROM tasks
    WHERE tags IS NOT NULL
      AND tags != '[]'
      AND tags != ''
      AND tag_id IS NULL
)
UPDATE tasks
SET tag_id = pt.tag_id
FROM parsed_tags pt
WHERE tasks.id = pt.task_id
  AND tasks.tag_id IS NULL;

-- Set default tag for tasks without tags
UPDATE tasks
SET tag_id = 'tag-main'
WHERE tag_id IS NULL;

-- Create triggers for timestamp management
CREATE TRIGGER IF NOT EXISTS agent_executions_updated_at
AFTER UPDATE ON agent_executions
BEGIN
    UPDATE agent_executions SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS pr_reviews_updated_at
AFTER UPDATE ON pr_reviews
BEGIN
    UPDATE pr_reviews SET updated_at = datetime('now', 'utc')
    WHERE id = NEW.id;
END;

-- Add trigger to update task actual_hours when execution completes
CREATE TRIGGER IF NOT EXISTS update_task_actual_hours
AFTER UPDATE ON agent_executions
WHEN NEW.status = 'completed' AND NEW.execution_time_seconds IS NOT NULL
BEGIN
    UPDATE tasks
    SET actual_hours = COALESCE(actual_hours, 0) + (NEW.execution_time_seconds / 3600.0)
    WHERE id = NEW.task_id;
END;

-- Add trigger to update task status when PR is merged
CREATE TRIGGER IF NOT EXISTS update_task_on_pr_merge
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
