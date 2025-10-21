-- ABOUTME: Add soft delete support to PRDs and ensure consistency
-- ABOUTME: Adds deleted_at column to prds table for soft delete functionality

-- Add deleted_at column to prds table
ALTER TABLE prds ADD COLUMN deleted_at TIMESTAMP;

-- Note: spec_capabilities and spec_changes already have deleted_at columns from 20250120000000_openspec.sql
-- This migration adds consistency by ensuring all OpenSpec tables support soft deletion
