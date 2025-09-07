use std::env;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid port number: {0}")]
    InvalidPort(#[from] ParseIntError),
    #[error("Port {0} is out of valid range (1-65535)")]
    PortOutOfRange(u16),
}

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub cors_origin: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port_str = env::var("PORT")
            .unwrap_or_else(|_| "4001".to_string());
        
        let port = port_str.parse::<u16>()?;
        
        // Validate port is in valid range
        if port == 0 {
            return Err(ConfigError::PortOutOfRange(port));
        }
            
        let cors_origin = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        Ok(Config {
            port,
            cors_origin,
        })
    }
}