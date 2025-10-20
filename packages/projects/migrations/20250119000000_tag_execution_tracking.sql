-- ABOUTME: Migration to add tags table, agent_executions tracking, and PR review management
-- ABOUTME: Replaces tasks.tags TEXT field with normalized tag_id reference and adds execution history

-- Tags table for organizing tasks
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    description TEXT,
    created_at TEXT NOT NULL,
    archived_at TEXT -- NULL = active, timestamp = archived
);

-- Create index for active tags
CREATE INDEX IF NOT EXISTS idx_tags_archived_at ON tags(archived_at);

-- Insert default "main" tag (like GitHub's default branch)
INSERT OR IGNORE INTO tags (id, name, color, description, created_at)
VALUES ('tag-main', 'main', '#3b82f6', 'Default tag for general tasks', datetime('now', 'utc'));

-- Agent executions table for tracking multiple AI attempts per task
CREATE TABLE IF NOT EXISTS agent_executions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    model TEXT, -- Specific model version like "claude-3-5-sonnet-20241022"
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL DEFAULT 'running', -- running, completed, failed, cancelled
    execution_time_seconds INTEGER,

    -- Token and cost tracking
    tokens_input INTEGER,
    tokens_output INTEGER,
    total_cost REAL,

    -- Execution details
    prompt TEXT,
    response TEXT,
    error_message TEXT,
    retry_attempt INTEGER NOT NULL DEFAULT 0,

    -- File change tracking
    files_changed INTEGER DEFAULT 0,
    lines_added INTEGER DEFAULT 0,
    lines_removed INTEGER DEFAULT 0,
    files_created TEXT, -- JSON array of file paths
    files_modified TEXT, -- JSON array of file paths
    files_deleted TEXT, -- JSON array of file paths

    -- Git/PR tracking
    branch_name TEXT,
    commit_hash TEXT,
    commit_message TEXT,
    pr_number INTEGER,
    pr_url TEXT,
    pr_title TEXT,
    pr_status TEXT, -- draft, open, closed, merged
    pr_created_at TEXT,
    pr_merged_at TEXT,
    pr_merge_commit TEXT,

    -- PR review status
    review_status TEXT, -- pending, changes_requested, approved, dismissed
    review_comments INTEGER DEFAULT 0,

    -- Performance metrics
    test_results TEXT, -- JSON with test pass/fail counts
    performance_metrics TEXT, -- JSON with custom metrics

    metadata TEXT, -- JSON for additional data
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
    reviewer_type TEXT NOT NULL DEFAULT 'ai', -- ai, human
    review_status TEXT NOT NULL, -- pending, approved, changes_requested, commented, dismissed
    review_body TEXT,
    comments TEXT, -- JSON array of inline comments
    suggested_changes TEXT, -- JSON array of suggested code changes
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

-- Migration: Add tag_id column to tasks table if it doesn't exist
-- SQLite doesn't support ALTER TABLE ADD COLUMN IF NOT EXISTS, so we need to check first
-- This will be handled by the migration system checking column existence

-- Add tag_id column to tasks (migration system will check if column exists)
ALTER TABLE tasks ADD COLUMN tag_id TEXT REFERENCES tags(id) ON DELETE SET NULL;

-- Create index for tag_id on tasks
CREATE INDEX IF NOT EXISTS idx_tasks_tag_id ON tasks(tag_id);

-- Migrate existing tasks with tags to use the new tag system
-- Parse the first tag from the JSON tags field and create/assign tags
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
