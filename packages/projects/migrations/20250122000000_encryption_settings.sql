-- Encryption Settings Table
-- Stores configuration for API key encryption modes

CREATE TABLE IF NOT EXISTS encryption_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1), -- Singleton table
    encryption_mode TEXT NOT NULL DEFAULT 'machine' CHECK (encryption_mode IN ('machine', 'password')),
    password_salt BLOB, -- Random salt for password-based key derivation (32 bytes)
    password_hash BLOB, -- Argon2id hash for password verification (not the encryption key)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default row (machine-based encryption)
INSERT OR IGNORE INTO encryption_settings (id, encryption_mode)
VALUES (1, 'machine');

-- Index for quick lookup (though singleton, for consistency)
CREATE INDEX IF NOT EXISTS idx_encryption_settings_mode ON encryption_settings(encryption_mode);
