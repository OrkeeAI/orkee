-- Rollback Migration: Expert Roundtable System
-- Description: Remove all expert roundtable tables and indexes

-- Drop indexes first
DROP INDEX IF EXISTS idx_expert_personas_default;
DROP INDEX IF EXISTS idx_roundtable_sessions_session;
DROP INDEX IF EXISTS idx_roundtable_sessions_status;
DROP INDEX IF EXISTS idx_roundtable_participants_roundtable;
DROP INDEX IF EXISTS idx_roundtable_participants_expert;
DROP INDEX IF EXISTS idx_roundtable_messages_roundtable;
DROP INDEX IF EXISTS idx_roundtable_messages_created;
DROP INDEX IF EXISTS idx_expert_suggestions_session;
DROP INDEX IF EXISTS idx_expert_suggestions_relevance;
DROP INDEX IF EXISTS idx_roundtable_insights_roundtable;
DROP INDEX IF EXISTS idx_roundtable_insights_priority;
DROP INDEX IF EXISTS idx_roundtable_insights_category;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS roundtable_insights;
DROP TABLE IF EXISTS expert_suggestions;
DROP TABLE IF EXISTS roundtable_messages;
DROP TABLE IF EXISTS roundtable_participants;
DROP TABLE IF EXISTS roundtable_sessions;
DROP TABLE IF EXISTS expert_personas;
