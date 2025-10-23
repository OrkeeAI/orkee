-- Add system_settings table for runtime configuration
-- Migration: 20250125000000_system_settings.sql

CREATE TABLE IF NOT EXISTS system_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    data_type TEXT NOT NULL DEFAULT 'string', -- string, boolean, integer, json
    is_secret INTEGER DEFAULT 0,
    requires_restart INTEGER DEFAULT 0,
    is_env_only INTEGER DEFAULT 0, -- If 1, must be set via .env file (read-only in UI)
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_by TEXT DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_system_settings_category ON system_settings(category);

-- Insert default settings
INSERT OR IGNORE INTO system_settings (key, value, category, description, data_type, requires_restart, is_env_only) VALUES
    -- Cloud Configuration (database-driven)
    ('cloud_enabled', 'false', 'cloud', 'Enable cloud sync features', 'boolean', 0, 0),
    ('cloud_api_url', 'https://api.orkee.ai', 'cloud', 'Orkee Cloud API URL', 'string', 0, 0),
    
    -- Server Configuration (MUST be in .env - read-only reference in UI)
    ('api_port', '4001', 'server', 'API server port (set via ORKEE_API_PORT in .env)', 'integer', 1, 1),
    ('ui_port', '5173', 'server', 'Dashboard UI port (set via ORKEE_UI_PORT in .env)', 'integer', 1, 1),
    ('dev_mode', 'false', 'server', 'Development mode (set via ORKEE_DEV_MODE in .env)', 'boolean', 1, 1),
    
    -- Security Configuration (database-driven, runtime configurable)
    ('cors_allow_any_localhost', 'true', 'security', 'Allow any localhost origin in development', 'boolean', 1, 0),
    ('allowed_browse_paths', '~/Documents,~/Projects,~/Code,~/Desktop', 'security', 'Comma-separated list of allowed directory paths', 'string', 0, 0),
    ('browse_sandbox_mode', 'relaxed', 'security', 'Directory browsing security mode: strict/relaxed/disabled', 'string', 0, 0),
    
    -- TLS/HTTPS Configuration (database-driven)
    ('tls_enabled', 'false', 'tls', 'Enable HTTPS/TLS support', 'boolean', 1, 0),
    ('tls_cert_path', '~/.orkee/certs/cert.pem', 'tls', 'Path to TLS certificate file', 'string', 1, 0),
    ('tls_key_path', '~/.orkee/certs/key.pem', 'tls', 'Path to TLS private key file', 'string', 1, 0),
    ('auto_generate_cert', 'true', 'tls', 'Auto-generate self-signed certificates for development', 'boolean', 1, 0),
    
    -- Rate Limiting (database-driven, requires restart)
    ('rate_limit_enabled', 'true', 'rate_limiting', 'Enable rate limiting middleware', 'boolean', 1, 0),
    ('rate_limit_health_rpm', '60', 'rate_limiting', 'Health endpoint requests per minute', 'integer', 1, 0),
    ('rate_limit_browse_rpm', '20', 'rate_limiting', 'Directory browsing requests per minute', 'integer', 1, 0),
    ('rate_limit_projects_rpm', '30', 'rate_limiting', 'Project operations requests per minute', 'integer', 1, 0),
    ('rate_limit_preview_rpm', '10', 'rate_limiting', 'Preview operations requests per minute', 'integer', 1, 0),
    ('rate_limit_ai_rpm', '10', 'rate_limiting', 'AI proxy endpoint requests per minute', 'integer', 1, 0),
    ('rate_limit_global_rpm', '30', 'rate_limiting', 'Global requests per minute for other endpoints', 'integer', 1, 0),
    ('rate_limit_burst_size', '5', 'rate_limiting', 'Burst size multiplier', 'integer', 1, 0),
    
    -- Security Headers (database-driven)
    ('security_headers_enabled', 'true', 'security', 'Enable security headers middleware', 'boolean', 1, 0),
    ('enable_hsts', 'false', 'security', 'Enable HSTS (only for HTTPS)', 'boolean', 1, 0),
    ('enable_request_id', 'true', 'security', 'Enable request ID generation for audit logging', 'boolean', 1, 0),
    
    -- Telemetry (database-driven, no restart needed)
    ('telemetry_enabled', 'false', 'telemetry', 'Enable telemetry data collection', 'boolean', 0, 0);
