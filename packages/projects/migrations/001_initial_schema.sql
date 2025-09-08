-- Initial SQLite schema for Orkee projects  
-- Migration: 001_initial_schema.sql

-- Enable foreign keys (other PRAGMA statements handled in code)
PRAGMA foreign_keys = ON;

-- Projects table - main entity
CREATE TABLE projects (
    -- Core fields
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    project_root TEXT NOT NULL UNIQUE,
    description TEXT,
    
    -- Status and priority
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
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

-- Indexes for performance
CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_priority ON projects(priority);
CREATE INDEX idx_projects_rank ON projects(rank);
CREATE INDEX idx_projects_updated_at ON projects(updated_at);
CREATE INDEX idx_projects_created_at ON projects(created_at);
CREATE INDEX idx_projects_task_source ON projects(task_source);

-- Composite indexes for common queries
CREATE INDEX idx_projects_status_priority ON projects(status, priority);
CREATE INDEX idx_projects_status_rank ON projects(status, rank);

-- Full-text search table using FTS5
CREATE VIRTUAL TABLE projects_fts USING fts5(
    id UNINDEXED,
    name,
    description,
    project_root,
    tags,
    content='projects',
    content_rowid='rowid'
);

-- Triggers to keep FTS5 in sync with projects table
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

-- Trigger to automatically update updated_at timestamp
CREATE TRIGGER projects_updated_at AFTER UPDATE ON projects
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE projects SET updated_at = datetime('now', 'utc') WHERE id = NEW.id;
END;

-- Storage metadata table for versioning and configuration
CREATE TABLE storage_metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

-- Insert initial metadata
INSERT INTO storage_metadata (key, value) VALUES 
    ('schema_version', '1'),
    ('created_at', datetime('now', 'utc')),
    ('storage_type', 'sqlite');

-- Cloud sync tables for future use
CREATE TABLE sync_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    compressed_data BLOB NOT NULL,
    checksum TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    project_count INTEGER NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'pending' CHECK (sync_status IN ('pending', 'uploading', 'uploaded', 'failed'))
);

CREATE INDEX idx_sync_snapshots_created_at ON sync_snapshots(created_at);
CREATE INDEX idx_sync_snapshots_status ON sync_snapshots(sync_status);

-- Table for tracking last sync state per user/device
CREATE TABLE sync_state (
    id TEXT PRIMARY KEY NOT NULL,
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

-- Views for common queries
CREATE VIEW active_projects AS
SELECT * FROM projects 
WHERE status = 'active' 
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

-- View for project search with scoring
CREATE VIEW project_search AS
SELECT 
    p.*,
    fts.rank as search_rank
FROM projects p
JOIN projects_fts fts ON p.rowid = fts.rowid;

-- Note: ANALYZE and VACUUM are run separately by the storage initialization code