-- ABOUTME: Rollback ideate_session_id from PRDs table
-- ABOUTME: Removes the foreign key link to ideate_sessions from prds table

-- Drop the new index on ideate_session_id
DROP INDEX IF EXISTS idx_prds_ideate_session;

-- Create a new prds table without ideate_session_id column
CREATE TABLE prds_new (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
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

-- Migrate existing data (excluding ideate_session_id)
INSERT INTO prds_new (id, project_id, title, content_markdown, version, status, source, deleted_at, created_at, updated_at, created_by)
SELECT id, project_id, title, content_markdown, version, status, source, deleted_at, created_at, updated_at, created_by FROM prds;

-- Drop old table and rename new one
DROP TABLE prds;
ALTER TABLE prds_new RENAME TO prds;

-- Recreate the index
CREATE INDEX idx_prds_project ON prds(project_id);
