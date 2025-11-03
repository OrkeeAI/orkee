-- Phase 1: Chat Mode Revolution - Database Schema Changes
-- This migration adds fields for one-question-at-a-time system, chunking, and smart answer formatting

-- Add Phase 1 fields to discovery_sessions table
ALTER TABLE discovery_sessions ADD COLUMN answer_format TEXT DEFAULT 'open'
    CHECK(answer_format IN ('open', 'letter', 'number', 'scale'));

ALTER TABLE discovery_sessions ADD COLUMN question_sequence INTEGER;

ALTER TABLE discovery_sessions ADD COLUMN is_critical BOOLEAN DEFAULT FALSE;

ALTER TABLE discovery_sessions ADD COLUMN branching_logic TEXT
    CHECK(json_valid(branching_logic) OR branching_logic IS NULL);

ALTER TABLE discovery_sessions ADD COLUMN options_presented TEXT
    CHECK(json_valid(options_presented) OR options_presented IS NULL);

ALTER TABLE discovery_sessions ADD COLUMN response_time INTEGER;

ALTER TABLE discovery_sessions ADD COLUMN category TEXT;

-- Add Phase 1 fields to prd_validation_history table for chunking support
ALTER TABLE prd_validation_history ADD COLUMN chunk_number INTEGER;

ALTER TABLE prd_validation_history ADD COLUMN chunk_word_count INTEGER;

ALTER TABLE prd_validation_history ADD COLUMN chunk_content TEXT;

ALTER TABLE prd_validation_history ADD COLUMN edited_content TEXT;

-- Create index for efficient question sequence queries
CREATE INDEX IF NOT EXISTS idx_discovery_sessions_sequence
    ON discovery_sessions(session_id, question_sequence);

-- Create index for answer format queries
CREATE INDEX IF NOT EXISTS idx_discovery_sessions_format
    ON discovery_sessions(session_id, answer_format);

-- Create index for chunk validation queries
CREATE INDEX IF NOT EXISTS idx_prd_validation_chunks
    ON prd_validation_history(session_id, chunk_number);
