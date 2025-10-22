-- Context Tab Feature - Phase 1
-- Store context configurations for projects

-- Store context configurations for projects
CREATE TABLE context_configurations (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    include_patterns TEXT DEFAULT '[]',  -- JSON array: ["src/**/*.ts", "lib/**/*.js"]
    exclude_patterns TEXT DEFAULT '[]',  -- JSON array: ["node_modules", "*.test.ts"]
    max_tokens INTEGER DEFAULT 100000,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Store generated context snapshots
CREATE TABLE context_snapshots (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    configuration_id TEXT REFERENCES context_configurations(id) ON DELETE SET NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    content TEXT NOT NULL,  -- The actual generated context
    file_count INTEGER,
    total_tokens INTEGER,
    metadata TEXT,  -- JSON object with file list, generation time, etc.
    created_at TEXT DEFAULT (datetime('now'))
);

-- Track which files/folders users commonly include
CREATE TABLE context_usage_patterns (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    inclusion_count INTEGER DEFAULT 0,
    last_used TEXT DEFAULT (datetime('now')),
    UNIQUE(project_id, file_path) ON CONFLICT REPLACE
);

-- Indexes for performance
CREATE INDEX idx_context_configs_project ON context_configurations(project_id);
CREATE INDEX idx_context_snapshots_project ON context_snapshots(project_id);
CREATE INDEX idx_context_snapshots_config ON context_snapshots(configuration_id);
CREATE INDEX idx_context_patterns_project ON context_usage_patterns(project_id);
CREATE INDEX idx_context_patterns_last_used ON context_usage_patterns(last_used);
