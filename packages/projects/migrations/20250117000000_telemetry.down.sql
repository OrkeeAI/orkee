-- ABOUTME: Rollback for telemetry tables migration
-- ABOUTME: Drops telemetry_settings, telemetry_events, and telemetry_stats tables

-- Drop trigger
DROP TRIGGER IF EXISTS update_telemetry_settings_timestamp;

-- Drop indexes
DROP INDEX IF EXISTS idx_telemetry_events_type;
DROP INDEX IF EXISTS idx_telemetry_events_unsent;

-- Drop tables
DROP TABLE IF EXISTS telemetry_stats;
DROP TABLE IF EXISTS telemetry_events;
DROP TABLE IF EXISTS telemetry_settings;
