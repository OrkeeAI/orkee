//! Cloud error types
use std::fmt;
use thiserror::Error;

/// Result type for cloud operations
pub type CloudResult<T> = Result<T, CloudError>;

/// Cloud-specific error types
#[derive(Debug, Error)]
pub enum CloudError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Token expired or invalid")]
    TokenExpired,

    #[error("Project not found: {0}")]
    ProjectNotFound(String),
}

impl CloudError {
    /// Create an authentication error
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }

    /// Create an API error
    pub fn api(msg: impl Into<String>) -> Self {
        Self::Api(msg.into())
    }

    /// Create a network error
    pub fn http(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }

    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Check if this is a network-related error
    pub fn is_network_error(&self) -> bool {
        matches!(self, CloudError::Network(_))
    }

    /// Check if this is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            CloudError::Authentication(_) | CloudError::TokenExpired
        )
    }
}

impl From<reqwest::Error> for CloudError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl From<serde_json::Error> for CloudError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for CloudError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
