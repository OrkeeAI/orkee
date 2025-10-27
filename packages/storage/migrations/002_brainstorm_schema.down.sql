-- ABOUTME: Rollback migration for brainstorming schema
-- ABOUTME: Drops all brainstorming-related tables in reverse dependency order

-- Drop tables in reverse order (child tables first)
DROP TABLE IF EXISTS prd_quickstart_templates;
DROP TABLE IF EXISTS roundtable_sessions;
DROP TABLE IF EXISTS brainstorm_research;
DROP TABLE IF EXISTS brainstorm_risks;
DROP TABLE IF EXISTS brainstorm_dependencies;
DROP TABLE IF EXISTS brainstorm_roadmap;
DROP TABLE IF EXISTS brainstorm_technical;
DROP TABLE IF EXISTS brainstorm_ux;
DROP TABLE IF EXISTS brainstorm_features;
DROP TABLE IF EXISTS brainstorm_overview;
DROP TABLE IF EXISTS brainstorm_sessions;
