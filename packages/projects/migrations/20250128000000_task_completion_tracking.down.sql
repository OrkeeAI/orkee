-- ABOUTME: Rollback migration for task completion tracking
-- ABOUTME: Removes task tracking tables, columns, and triggers

-- Drop triggers
DROP TRIGGER IF EXISTS update_task_completion_stats;
DROP TRIGGER IF EXISTS update_task_completion_stats_insert;
DROP TRIGGER IF EXISTS update_task_completion_stats_delete;

-- Drop indexes
DROP INDEX IF EXISTS idx_spec_change_tasks_change;
DROP INDEX IF EXISTS idx_spec_change_tasks_completion;
DROP INDEX IF EXISTS idx_spec_change_tasks_parent;

-- Drop table
DROP TABLE IF EXISTS spec_change_tasks;

-- Remove columns from spec_changes (SQLite doesn't support DROP COLUMN easily, but documenting here)
-- ALTER TABLE spec_changes DROP COLUMN tasks_completion_percentage;
-- ALTER TABLE spec_changes DROP COLUMN tasks_parsed_at;
-- ALTER TABLE spec_changes DROP COLUMN tasks_total_count;
-- ALTER TABLE spec_changes DROP COLUMN tasks_completed_count;

-- Note: In SQLite, dropping columns requires recreating the table.
-- For production, consider using a tool like sqlx or implementing a full table recreation.
