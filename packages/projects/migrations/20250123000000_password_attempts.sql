-- ABOUTME: Tracks failed password verification attempts for brute-force protection
-- ABOUTME: Implements account lockout after repeated failed attempts

CREATE TABLE IF NOT EXISTS password_attempts (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Single row table
    attempt_count INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,  -- ISO 8601 timestamp when lockout expires
    last_attempt_at TEXT,  -- ISO 8601 timestamp of last failed attempt
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

-- Insert initial row
INSERT INTO password_attempts (id, attempt_count) VALUES (1, 0);
