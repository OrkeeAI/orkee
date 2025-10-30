// ABOUTME: Environment variable name constants
// ABOUTME: Centralized definitions of all environment variable names used across Orkee

// Port Configuration
pub const ORKEE_API_PORT: &str = "ORKEE_API_PORT";
pub const ORKEE_UI_PORT: &str = "ORKEE_UI_PORT";
pub const PORT: &str = "PORT"; // Legacy

// Development Configuration
pub const ORKEE_DEV_MODE: &str = "ORKEE_DEV_MODE";
pub const ORKEE_DASHBOARD_PATH: &str = "ORKEE_DASHBOARD_PATH";

// CORS Configuration
pub const ORKEE_CORS_ORIGIN: &str = "ORKEE_CORS_ORIGIN";
pub const CORS_ORIGIN: &str = "CORS_ORIGIN"; // Legacy
pub const CORS_ALLOW_ANY_LOCALHOST: &str = "CORS_ALLOW_ANY_LOCALHOST";

// Path Validation & Sandboxing
pub const BROWSE_SANDBOX_MODE: &str = "BROWSE_SANDBOX_MODE";
pub const ALLOWED_BROWSE_PATHS: &str = "ALLOWED_BROWSE_PATHS";

// TLS/HTTPS Configuration
pub const TLS_ENABLED: &str = "TLS_ENABLED";
pub const TLS_CERT_PATH: &str = "TLS_CERT_PATH";
pub const TLS_KEY_PATH: &str = "TLS_KEY_PATH";
pub const AUTO_GENERATE_CERT: &str = "AUTO_GENERATE_CERT";
pub const ENABLE_HSTS: &str = "ENABLE_HSTS";

// Rate Limiting
pub const RATE_LIMIT_ENABLED: &str = "RATE_LIMIT_ENABLED";
pub const RATE_LIMIT_HEALTH_RPM: &str = "RATE_LIMIT_HEALTH_RPM";
pub const RATE_LIMIT_BROWSE_RPM: &str = "RATE_LIMIT_BROWSE_RPM";
pub const RATE_LIMIT_PROJECTS_RPM: &str = "RATE_LIMIT_PROJECTS_RPM";
pub const RATE_LIMIT_PREVIEW_RPM: &str = "RATE_LIMIT_PREVIEW_RPM";
pub const RATE_LIMIT_TELEMETRY_RPM: &str = "RATE_LIMIT_TELEMETRY_RPM";
pub const RATE_LIMIT_AI_RPM: &str = "RATE_LIMIT_AI_RPM";
pub const RATE_LIMIT_USERS_RPM: &str = "RATE_LIMIT_USERS_RPM";
pub const RATE_LIMIT_SECURITY_RPM: &str = "RATE_LIMIT_SECURITY_RPM";
pub const RATE_LIMIT_GLOBAL_RPM: &str = "RATE_LIMIT_GLOBAL_RPM";
pub const RATE_LIMIT_BURST_SIZE: &str = "RATE_LIMIT_BURST_SIZE";

// Security Headers
pub const SECURITY_HEADERS_ENABLED: &str = "SECURITY_HEADERS_ENABLED";
pub const ENABLE_REQUEST_ID: &str = "ENABLE_REQUEST_ID";

// Cloud Sync Configuration
pub const ORKEE_CLOUD_TOKEN: &str = "ORKEE_CLOUD_TOKEN";
pub const ORKEE_CLOUD_API_URL: &str = "ORKEE_CLOUD_API_URL";
pub const ORKEE_CLOUD_ENABLED: &str = "ORKEE_CLOUD_ENABLED";

// Dashboard Tauri Configuration
pub const ORKEE_TRAY_POLL_INTERVAL_SECS: &str = "ORKEE_TRAY_POLL_INTERVAL_SECS";
pub const ORKEE_API_HOST: &str = "ORKEE_API_HOST";
pub const ORKEE_ALLOW_REMOTE_API: &str = "ORKEE_ALLOW_REMOTE_API";
pub const ORKEE_HTTP_REQUEST_TIMEOUT_SECS: &str = "ORKEE_HTTP_REQUEST_TIMEOUT_SECS";
pub const ORKEE_HTTP_CONNECT_TIMEOUT_SECS: &str = "ORKEE_HTTP_CONNECT_TIMEOUT_SECS";

// Preview Server Configuration
pub const ORKEE_STALE_TIMEOUT_MINUTES: &str = "ORKEE_STALE_TIMEOUT_MINUTES";
pub const ORKEE_PROCESS_START_TIME_TOLERANCE_SECS: &str = "ORKEE_PROCESS_START_TIME_TOLERANCE_SECS";
pub const ORKEE_CLEANUP_INTERVAL_MINUTES: &str = "ORKEE_CLEANUP_INTERVAL_MINUTES";

// External Server Discovery Configuration
pub const ORKEE_DISCOVERY_ENABLED: &str = "ORKEE_DISCOVERY_ENABLED";
pub const ORKEE_DISCOVERY_INTERVAL_SECS: &str = "ORKEE_DISCOVERY_INTERVAL_SECS";
pub const ORKEE_DISCOVERY_PORTS: &str = "ORKEE_DISCOVERY_PORTS";

// SSE Stream Configuration
pub const ORKEE_SSE_MAX_DURATION_MINUTES: &str = "ORKEE_SSE_MAX_DURATION_MINUTES";
pub const ORKEE_SSE_POLL_INTERVAL_SECS: &str = "ORKEE_SSE_POLL_INTERVAL_SECS";

// System Environment Variables
pub const HOME: &str = "HOME";
pub const USERPROFILE: &str = "USERPROFILE"; // Windows
