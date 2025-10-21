-- ABOUTME: Privacy-first telemetry system for opt-in usage analytics
-- ABOUTME: Stores user preferences and telemetry events with local buffering

-- Telemetry settings table
CREATE TABLE IF NOT EXISTS telemetry_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    first_run BOOLEAN NOT NULL DEFAULT TRUE,
    onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE,
    error_reporting BOOLEAN NOT NULL DEFAULT FALSE,
    usage_metrics BOOLEAN NOT NULL DEFAULT FALSE,
    non_anonymous_metrics BOOLEAN NOT NULL DEFAULT FALSE,
    machine_id TEXT,
    user_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default settings row
INSERT OR IGNORE INTO telemetry_settings (id) VALUES (1);

-- Telemetry events table for local buffering
CREATE TABLE IF NOT EXISTS telemetry_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    event_name TEXT NOT NULL,
    event_data TEXT,
    anonymous BOOLEAN NOT NULL DEFAULT TRUE,
    session_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    sent_at TEXT,
    retry_count INTEGER DEFAULT 0
);

-- Index for efficient querying of unsent events
CREATE INDEX IF NOT EXISTS idx_telemetry_events_unsent
ON telemetry_events(sent_at, created_at)
WHERE sent_at IS NULL;

-- Index for event type queries
CREATE INDEX IF NOT EXISTS idx_telemetry_events_type
ON telemetry_events(event_type, created_at);

-- Telemetry statistics table
CREATE TABLE IF NOT EXISTS telemetry_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stat_date TEXT NOT NULL,
    total_events INTEGER DEFAULT 0,
    error_events INTEGER DEFAULT 0,
    usage_events INTEGER DEFAULT 0,
    performance_events INTEGER DEFAULT 0,
    events_sent INTEGER DEFAULT 0,
    events_pending INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(stat_date)
);

-- Trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_telemetry_settings_timestamp
AFTER UPDATE ON telemetry_settings
BEGIN
    UPDATE telemetry_settings
    SET updated_at = datetime('now')
    WHERE id = NEW.id;
END;
