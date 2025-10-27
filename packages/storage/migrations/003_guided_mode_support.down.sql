-- ABOUTME: Rollback migration for Guided Mode support
-- ABOUTME: Removes navigation tracking and AI-generation markers

PRAGMA foreign_keys = ON;

-- ============================================================================
-- REMOVE AI-GENERATED CONTENT MARKERS
-- ============================================================================

-- Note: SQLite doesn't support DROP COLUMN directly, so we would need to recreate tables
-- For development, we can document the rollback approach:
-- In production, these columns can simply be ignored if the migration is rolled back

-- Drop index for current_section
DROP INDEX IF EXISTS idx_ideate_sessions_current_section;

-- Note: To fully rollback, you would need to:
-- 1. CREATE new tables without the new columns
-- 2. COPY data from old tables to new tables
-- 3. DROP old tables
-- 4. RENAME new tables to original names

-- For development purposes, the presence of unused columns is harmless
-- and allows for easier migration testing.
