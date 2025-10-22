-- ABOUTME: Security infrastructure for API key encryption and brute-force protection
-- ABOUTME: Supports machine-based and password-based encryption modes with account lockout

-- Encryption Settings Table
CREATE TABLE IF NOT EXISTS encryption_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    encryption_mode TEXT NOT NULL DEFAULT 'machine' CHECK (encryption_mode IN ('machine', 'password')),
    password_salt BLOB,
    password_hash BLOB,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default row (machine-based encryption)
INSERT OR IGNORE INTO encryption_settings (id, encryption_mode)
VALUES (1, 'machine');

-- Index for quick lookup
CREATE INDEX IF NOT EXISTS idx_encryption_settings_mode ON encryption_settings(encryption_mode);

-- Password Attempts Table
CREATE TABLE IF NOT EXISTS password_attempts (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    last_attempt_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

-- Insert initial row
INSERT OR IGNORE INTO password_attempts (id, attempt_count) VALUES (1, 0);

-- Index for lockout checks (partial index for non-null locked_until)
CREATE INDEX IF NOT EXISTS idx_password_attempts_lockout
ON password_attempts(locked_until) WHERE locked_until IS NOT NULL;
