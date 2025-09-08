use std::env;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

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

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub cors_origin: String,
    pub cors_allow_any_localhost: bool,
    pub allowed_browse_paths: Vec<String>,
    pub browse_sandbox_mode: SandboxMode,
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

        Ok(Config { 
            port, 
            cors_origin,
            cors_allow_any_localhost,
            allowed_browse_paths,
            browse_sandbox_mode,
        })
    }
}
