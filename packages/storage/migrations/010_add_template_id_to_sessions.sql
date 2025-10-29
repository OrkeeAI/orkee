-- ABOUTME: Add template_id to ideate_sessions for PRD template tracking
-- ABOUTME: Enables storing which template was used for generation to allow re-generation

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Add template_id column to ideate_sessions
ALTER TABLE ideate_sessions ADD COLUMN template_id TEXT;

-- Add foreign key constraint (SQLite doesn't allow adding constraints to existing tables,
-- but we can still document it and enforce it in the application layer)
-- FOREIGN KEY (template_id) REFERENCES prd_output_templates(id) ON DELETE SET NULL

-- Add index for performance
CREATE INDEX idx_ideate_sessions_template ON ideate_sessions(template_id);
