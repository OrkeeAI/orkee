-- ABOUTME: Add ideate_session_id to PRDs table for tracking regenerated PRDs
-- ABOUTME: Allows linking multiple PRDs back to their originating ideate session

-- Add ideate_session_id column to prds table to link regenerated PRDs back to sessions
ALTER TABLE prds ADD COLUMN ideate_session_id TEXT;

-- Add foreign key constraint
CREATE TABLE prds_new (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,
    ideate_session_id TEXT,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'approved', 'superseded')),
    source TEXT DEFAULT 'manual' CHECK(source IN ('manual', 'generated', 'synced')),
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    created_by TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (ideate_session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

-- Migrate existing data
INSERT INTO prds_new SELECT * FROM prds;

-- Drop old table and rename new one
DROP TABLE prds;
ALTER TABLE prds_new RENAME TO prds;

-- Recreate the index
CREATE INDEX idx_prds_project ON prds(project_id);

-- Add index on ideate_session_id for faster lookups
CREATE INDEX idx_prds_ideate_session ON prds(ideate_session_id);
