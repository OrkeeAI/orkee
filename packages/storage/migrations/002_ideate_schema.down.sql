-- ABOUTME: Rollback migration for ideateing schema
-- ABOUTME: Drops all ideateing-related tables in reverse dependency order

-- Drop tables in reverse order (child tables first)
DROP TABLE IF EXISTS prd_quickstart_templates;
DROP TABLE IF EXISTS roundtable_sessions;
DROP TABLE IF EXISTS ideate_research;
DROP TABLE IF EXISTS ideate_risks;
DROP TABLE IF EXISTS ideate_dependencies;
DROP TABLE IF EXISTS ideate_roadmap;
DROP TABLE IF EXISTS ideate_technical;
DROP TABLE IF EXISTS ideate_ux;
DROP TABLE IF EXISTS ideate_features;
DROP TABLE IF EXISTS ideate_overview;
DROP TABLE IF EXISTS ideate_sessions;
