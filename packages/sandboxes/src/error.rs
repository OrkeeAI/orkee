// ABOUTME: Error types for sandbox execution
// ABOUTME: Comprehensive error handling for Docker, Vibekit, Node.js bridge, and execution failures

use thiserror::Error;

/// Main error type for sandbox operations
#[derive(Error, Debug)]
pub enum SandboxError {
    /// Docker/container-related errors
    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),

    /// Docker container not found
    #[error("Container not found: {0}")]
    ContainerNotFound(String),

    /// Docker image not found or failed to pull
    #[error("Docker image error: {0}")]
    ImageError(String),

    /// Container failed to start
    #[error("Container failed to start: {0}")]
    ContainerStartFailed(String),

    /// Container exceeded resource limits
    #[error("Container exceeded {resource} limit: {details}")]
    ResourceLimitExceeded { resource: String, details: String },

    /// Vibekit SDK errors
    #[error("Vibekit SDK error: {0}")]
    Vibekit(String),

    /// Vibekit session not found
    #[error("Vibekit session not found: {0}")]
    VibikitSessionNotFound(String),

    /// Node.js bridge errors
    #[error("Node.js bridge error: {0}")]
    NodeBridge(String),

    /// Node.js bridge script not found
    #[error("Bridge script not found: {0}")]
    BridgeNotFound(String),

    /// Node.js bridge failed to start
    #[error("Bridge failed to start: {0}")]
    BridgeStartFailed(String),

    /// Node.js bridge is not running
    #[error("Bridge is not running")]
    BridgeNotRunning,

    /// Node.js bridge communication error
    #[error("Bridge communication error: {0}")]
    BridgeCommunicationError(String),

    /// Node.js process failed to start
    #[error("Node.js process failed to start: {0}")]
    NodeProcessStartFailed(String),

    /// Node.js process crashed
    #[error("Node.js process crashed: {0}")]
    NodeProcessCrashed(String),

    /// IPC communication error
    #[error("IPC communication error: {0}")]
    IpcError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Execution timeout
    #[error("Execution timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// Execution cancelled by user
    #[error("Execution cancelled: {0}")]
    Cancelled(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Invalid execution request
    #[error("Invalid execution request: {0}")]
    InvalidRequest(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Artifact not found
    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    /// Artifact upload/download error
    #[error("Artifact transfer error: {0}")]
    ArtifactTransferError(String),

    /// Workspace error (file access, permissions, etc.)
    #[error("Workspace error: {0}")]
    WorkspaceError(String),

    /// Unknown or unhandled error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Type alias for Results that return SandboxError
pub type Result<T> = std::result::Result<T, SandboxError>;
