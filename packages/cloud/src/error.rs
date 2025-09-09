//! Cloud error types

use thiserror::Error;

/// Result type for cloud operations
pub type CloudResult<T> = Result<T, CloudError>;

/// Cloud error types
#[derive(Debug, Error)]
pub enum CloudError {
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Token expired or invalid")]
    TokenExpired,
    
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
}

impl CloudError {
    /// Create an authentication error
    pub fn auth(msg: impl Into<String>) -> Self {
        CloudError::Authentication(msg.into())
    }
    
    /// Create an API error
    pub fn api(msg: impl Into<String>) -> Self {
        CloudError::Api(msg.into())
    }
    
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        CloudError::Configuration(msg.into())
    }
    
    /// Check if this is a network-related error
    pub fn is_network_error(&self) -> bool {
        matches!(self, CloudError::Network(_))
    }
    
    /// Check if this is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, CloudError::Authentication(_) | CloudError::TokenExpired)
    }
}