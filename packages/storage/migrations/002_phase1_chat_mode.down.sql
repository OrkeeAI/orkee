-- Rollback Phase 1: Chat Mode Revolution
-- This migration removes Phase 1 enhancements for one-question-at-a-time system, chunking, and smart answer formatting

-- Drop indexes first
DROP INDEX IF EXISTS idx_prd_validation_chunks;
DROP INDEX IF EXISTS idx_discovery_sessions_format;
DROP INDEX IF EXISTS idx_discovery_sessions_sequence;

-- Remove Phase 1 fields from prd_validation_history
-- Note: SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
-- For now, we'll leave the columns in place as they don't break functionality
-- In production, you'd want to recreate the table without these columns

-- SQLite limitation: Cannot drop columns directly
-- ALTER TABLE prd_validation_history DROP COLUMN edited_content;
-- ALTER TABLE prd_validation_history DROP COLUMN chunk_content;
-- ALTER TABLE prd_validation_history DROP COLUMN chunk_word_count;
-- ALTER TABLE prd_validation_history DROP COLUMN chunk_number;

-- Remove Phase 1 fields from discovery_sessions
-- SQLite limitation: Cannot drop columns directly
-- ALTER TABLE discovery_sessions DROP COLUMN category;
-- ALTER TABLE discovery_sessions DROP COLUMN response_time;
-- ALTER TABLE discovery_sessions DROP COLUMN options_presented;
-- ALTER TABLE discovery_sessions DROP COLUMN branching_logic;
-- ALTER TABLE discovery_sessions DROP COLUMN is_critical;
-- ALTER TABLE discovery_sessions DROP COLUMN question_sequence;
-- ALTER TABLE discovery_sessions DROP COLUMN answer_format;

-- Note: To fully rollback on SQLite, you would need to:
-- 1. Create new tables with old schema
-- 2. Copy data from existing tables
-- 3. Drop old tables
-- 4. Rename new tables
-- This is left as a manual operation if needed
