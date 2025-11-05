// ABOUTME: Provider trait and implementations for sandbox execution backends
// ABOUTME: Defines abstract interface for container/VM lifecycle management

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

pub mod docker;

pub use docker::DockerProvider;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Container error: {0}")]
    ContainerError(String),

    #[error("Image error: {0}")]
    ImageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Provider not available: {0}")]
    NotAvailable(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

type Result<T> = std::result::Result<T, ProviderError>;

/// Container configuration for creating sandboxes
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    pub image: String,
    pub name: String,
    pub env_vars: HashMap<String, String>,
    pub volumes: Vec<VolumeMount>,
    pub ports: Vec<PortMapping>,
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub storage_gb: u64,
    pub command: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String, // tcp or udp
}

/// Container runtime information
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: ContainerStatus,
    pub ip_address: Option<String>,
    pub ports: HashMap<u16, u16>, // container_port -> host_port
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metrics: Option<ContainerMetrics>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStatus {
    Creating,
    Created,
    Running,
    Paused,
    Stopping,
    Stopped,
    Removing,
    Dead,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ContainerMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub memory_limit_mb: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// Execution result from running a command in a container
#[derive(Debug)]
pub struct ExecResult {
    pub exit_code: i64,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Stream output from container logs or exec
pub struct OutputStream {
    pub receiver: tokio::sync::mpsc::UnboundedReceiver<OutputChunk>,
}

#[derive(Debug, Clone)]
pub struct OutputChunk {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub stream: StreamType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Stdout,
    Stderr,
}

/// Provider trait for sandbox container/VM backends
#[async_trait]
pub trait Provider: Send + Sync {
    /// Check if the provider is available and configured correctly
    async fn is_available(&self) -> Result<bool>;

    /// Get provider information and capabilities
    async fn get_info(&self) -> Result<ProviderInfo>;

    /// Create and start a new container
    async fn create_container(&self, config: &ContainerConfig) -> Result<String>;

    /// Start a stopped container
    async fn start_container(&self, container_id: &str) -> Result<()>;

    /// Stop a running container
    async fn stop_container(&self, container_id: &str, timeout_secs: u64) -> Result<()>;

    /// Remove a container
    async fn remove_container(&self, container_id: &str, force: bool) -> Result<()>;

    /// Get container information
    async fn get_container_info(&self, container_id: &str) -> Result<ContainerInfo>;

    /// List all containers managed by this provider
    async fn list_containers(&self, include_stopped: bool) -> Result<Vec<ContainerInfo>>;

    /// Execute a command in a running container
    async fn exec_command(
        &self,
        container_id: &str,
        command: Vec<String>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<ExecResult>;

    /// Stream container logs
    async fn stream_logs(
        &self,
        container_id: &str,
        follow: bool,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<OutputStream>;

    /// Copy files into a container
    async fn copy_to_container(
        &self,
        container_id: &str,
        source_path: &str,
        dest_path: &str,
    ) -> Result<()>;

    /// Copy files from a container
    async fn copy_from_container(
        &self,
        container_id: &str,
        source_path: &str,
        dest_path: &str,
    ) -> Result<()>;

    /// Get container resource metrics
    async fn get_metrics(&self, container_id: &str) -> Result<ContainerMetrics>;

    /// Pull an image if it doesn't exist locally
    async fn pull_image(&self, image: &str, force: bool) -> Result<()>;

    /// Check if an image exists locally
    async fn image_exists(&self, image: &str) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub version: String,
    pub provider_type: String,
    pub capabilities: ProviderCapabilities,
    pub status: ProviderStatus,
}

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub gpu_support: bool,
    pub persistent_storage: bool,
    pub network_isolation: bool,
    pub resource_limits: bool,
    pub exec_support: bool,
    pub file_transfer: bool,
    pub metrics: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderStatus {
    Ready,
    NotAvailable(String),
    Degraded(String),
}