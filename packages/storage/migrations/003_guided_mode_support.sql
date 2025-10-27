-- ABOUTME: Schema updates for Guided Mode support
-- ABOUTME: Adds navigation tracking and AI-generation markers

PRAGMA foreign_keys = ON;

-- ============================================================================
-- SESSION NAVIGATION TRACKING
-- ============================================================================

-- Add current_section field to track guided mode progress
ALTER TABLE ideate_sessions ADD COLUMN current_section TEXT;

-- Create index for quick lookups
CREATE INDEX idx_ideate_sessions_current_section ON ideate_sessions(current_section);

-- ============================================================================
-- AI-GENERATED CONTENT MARKERS
-- ============================================================================

-- Mark AI-generated content in each section table
ALTER TABLE ideate_overview ADD COLUMN ai_generated INTEGER DEFAULT 0;
ALTER TABLE ideate_ux ADD COLUMN ai_generated INTEGER DEFAULT 0;
ALTER TABLE ideate_technical ADD COLUMN ai_generated INTEGER DEFAULT 0;
ALTER TABLE ideate_roadmap ADD COLUMN ai_generated INTEGER DEFAULT 0;
ALTER TABLE ideate_dependencies ADD COLUMN ai_generated INTEGER DEFAULT 0;
ALTER TABLE ideate_risks ADD COLUMN ai_generated INTEGER DEFAULT 0;
ALTER TABLE ideate_research ADD COLUMN ai_generated INTEGER DEFAULT 0;

-- Note: ideate_features table doesn't need ai_generated since features are
-- created individually (not as a whole section)
