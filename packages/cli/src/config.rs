use std::env;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;
use crate::middleware::RateLimitConfig;
use crate::tls::TlsConfig;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid port number: {0}")]
    InvalidPort(#[from] ParseIntError),
    #[error("Port {0} is out of valid range (1-65535)")]
    PortOutOfRange(u16),
    #[error("Invalid sandbox mode: {0}")]
    InvalidSandboxMode(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SandboxMode {
    Strict,   // Only explicitly allowed paths
    Relaxed,  // Home + allowed paths, block dangerous
    Disabled, // No restrictions (NOT recommended)
}

impl FromStr for SandboxMode {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "strict" => Ok(SandboxMode::Strict),
            "relaxed" => Ok(SandboxMode::Relaxed),
            "disabled" => Ok(SandboxMode::Disabled),
            _ => Err(ConfigError::InvalidSandboxMode(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub cors_origin: String,
    pub cors_allow_any_localhost: bool,
    pub allowed_browse_paths: Vec<String>,
    pub browse_sandbox_mode: SandboxMode,
    
    // Middleware configuration
    pub rate_limit: RateLimitConfig,
    pub security_headers_enabled: bool,
    pub enable_hsts: bool,
    pub enable_request_id: bool,
    
    // TLS configuration
    pub tls: TlsConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port_str = env::var("PORT").unwrap_or_else(|_| "4001".to_string());

        let port = port_str.parse::<u16>()?;

        // Validate port is in valid range
        if port == 0 {
            return Err(ConfigError::PortOutOfRange(port));
        }

        let cors_origin =
            env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string());

        let cors_allow_any_localhost = env::var("CORS_ALLOW_ANY_LOCALHOST")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        // Parse allowed browse paths from environment
        let allowed_browse_paths = env::var("ALLOWED_BROWSE_PATHS")
            .unwrap_or_else(|_| "~/Documents,~/Projects,~/Desktop,~/Downloads".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let browse_sandbox_mode = env::var("BROWSE_SANDBOX_MODE")
            .unwrap_or_else(|_| "relaxed".to_string())
            .parse::<SandboxMode>()?;

        // Parse rate limiting configuration
        let rate_limit = RateLimitConfig {
            enabled: env::var("RATE_LIMIT_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse::<bool>()
                .unwrap_or(true),
            health_rpm: env::var("RATE_LIMIT_HEALTH_RPM")
                .unwrap_or_else(|_| "60".to_string())
                .parse::<u32>()
                .unwrap_or(60),
            browse_rpm: env::var("RATE_LIMIT_BROWSE_RPM")
                .unwrap_or_else(|_| "20".to_string())
                .parse::<u32>()
                .unwrap_or(20),
            projects_rpm: env::var("RATE_LIMIT_PROJECTS_RPM")
                .unwrap_or_else(|_| "30".to_string())
                .parse::<u32>()
                .unwrap_or(30),
            preview_rpm: env::var("RATE_LIMIT_PREVIEW_RPM")
                .unwrap_or_else(|_| "10".to_string())
                .parse::<u32>()
                .unwrap_or(10),
            global_rpm: env::var("RATE_LIMIT_GLOBAL_RPM")
                .unwrap_or_else(|_| "30".to_string())
                .parse::<u32>()
                .unwrap_or(30),
            burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                .unwrap_or_else(|_| "5".to_string())
                .parse::<u32>()
                .unwrap_or(5),
        };

        // Parse security headers configuration
        let security_headers_enabled = env::var("SECURITY_HEADERS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let enable_hsts = env::var("ENABLE_HSTS")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let enable_request_id = env::var("ENABLE_REQUEST_ID")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        // Parse TLS configuration
        let tls_enabled = env::var("TLS_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
            
        let default_cert_dir = crate::tls::TlsManager::default_cert_dir();
        
        let tls_cert_path = env::var("TLS_CERT_PATH")
            .unwrap_or_else(|_| default_cert_dir.join("cert.pem").to_string_lossy().to_string())
            .into();
            
        let tls_key_path = env::var("TLS_KEY_PATH")
            .unwrap_or_else(|_| default_cert_dir.join("key.pem").to_string_lossy().to_string())
            .into();
            
        let auto_generate_cert = env::var("AUTO_GENERATE_CERT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let tls = TlsConfig {
            enabled: tls_enabled,
            cert_path: tls_cert_path,
            key_path: tls_key_path,
            auto_generate: auto_generate_cert,
        };

        Ok(Config { 
            port, 
            cors_origin,
            cors_allow_any_localhost,
            allowed_browse_paths,
            browse_sandbox_mode,
            rate_limit,
            security_headers_enabled,
            enable_hsts,
            enable_request_id,
            tls,
        })
    }
}
