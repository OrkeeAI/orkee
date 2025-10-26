-- ABOUTME: Task completion tracking for OpenSpec changes
-- ABOUTME: Enables individual task tracking, progress monitoring, and sequential completion enforcement

-- Create table for tracking individual tasks within change proposals
CREATE TABLE IF NOT EXISTS spec_change_tasks (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    change_id TEXT NOT NULL,
    task_number TEXT NOT NULL,        -- e.g., "1.1", "2.3", "1"
    task_text TEXT NOT NULL,          -- Original task description from markdown
    is_completed BOOLEAN DEFAULT FALSE NOT NULL,
    completed_by TEXT,
    completed_at TIMESTAMP,
    display_order INTEGER NOT NULL,   -- Ordering within the change (0-indexed)
    parent_number TEXT,                -- For hierarchical tasks (e.g., "1" is parent of "1.1")
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (change_id) REFERENCES spec_changes(id) ON DELETE CASCADE
);

-- Add task completion metadata to spec_changes table
ALTER TABLE spec_changes ADD COLUMN tasks_completion_percentage INTEGER DEFAULT 0 CHECK(tasks_completion_percentage >= 0 AND tasks_completion_percentage <= 100);
ALTER TABLE spec_changes ADD COLUMN tasks_parsed_at TIMESTAMP;
ALTER TABLE spec_changes ADD COLUMN tasks_total_count INTEGER DEFAULT 0;
ALTER TABLE spec_changes ADD COLUMN tasks_completed_count INTEGER DEFAULT 0;

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_spec_change_tasks_change ON spec_change_tasks(change_id, display_order);
CREATE INDEX IF NOT EXISTS idx_spec_change_tasks_completion ON spec_change_tasks(change_id, is_completed);
CREATE INDEX IF NOT EXISTS idx_spec_change_tasks_parent ON spec_change_tasks(change_id, parent_number);

-- Create trigger to update task completion statistics on spec_changes
CREATE TRIGGER IF NOT EXISTS update_task_completion_stats
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
        updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.change_id;
END;

-- Create trigger to update task completion stats when tasks are inserted
CREATE TRIGGER IF NOT EXISTS update_task_completion_stats_insert
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
        updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.change_id;
END;

-- Create trigger to update task completion stats when tasks are deleted
CREATE TRIGGER IF NOT EXISTS update_task_completion_stats_delete
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
        updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.change_id;
END;
