-- ABOUTME: Rollback template_id column addition
-- ABOUTME: Removes template_id tracking from ideate_sessions

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Drop the index
DROP INDEX IF EXISTS idx_ideate_sessions_template;

-- Remove template_id column from ideate_sessions
-- Note: SQLite doesn't support DROP COLUMN directly before version 3.35.0
-- This requires recreating the table without the column

-- Create temporary table without template_id
CREATE TABLE ideate_sessions_temp (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,
    initial_description TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('quick', 'guided', 'comprehensive')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'in_progress', 'ready_for_prd', 'completed')),
    skipped_sections TEXT,
    generated_prd_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (generated_prd_id) REFERENCES prds(id) ON DELETE SET NULL,
    CHECK (json_valid(skipped_sections) OR skipped_sections IS NULL)
);

-- Copy data from old table
INSERT INTO ideate_sessions_temp
SELECT id, project_id, initial_description, mode, status, skipped_sections, generated_prd_id, created_at, updated_at
FROM ideate_sessions;

-- Drop old table
DROP TABLE ideate_sessions;

-- Rename temp table
ALTER TABLE ideate_sessions_temp RENAME TO ideate_sessions;

-- Recreate indexes
CREATE INDEX idx_ideate_sessions_project ON ideate_sessions(project_id);
CREATE INDEX idx_ideate_sessions_status ON ideate_sessions(status);
CREATE INDEX idx_ideate_sessions_mode ON ideate_sessions(mode);

-- Recreate trigger
CREATE TRIGGER ideate_sessions_updated_at AFTER UPDATE ON ideate_sessions
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE ideate_sessions SET updated_at = datetime('now', 'utc') WHERE id = NEW.id;
END;
