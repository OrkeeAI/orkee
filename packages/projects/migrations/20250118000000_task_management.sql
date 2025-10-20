-- Task Management Schema
-- Adds users, agents, user_agents, and tasks tables for manual task tracking

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
