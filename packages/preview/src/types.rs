use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Project type detected from the codebase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Nextjs,
    React,
    Vue,
    Node,
    Python,
    Static,
    Unknown,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Nextjs => "nextjs",
            ProjectType::React => "react",
            ProjectType::Vue => "vue",
            ProjectType::Node => "node",
            ProjectType::Python => "python",
            ProjectType::Static => "static",
            ProjectType::Unknown => "unknown",
        }
    }
}

/// Package manager detected for the project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
        }
    }
}

/// Status of a development server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DevServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl DevServerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DevServerStatus::Stopped => "stopped",
            DevServerStatus::Starting => "starting",
            DevServerStatus::Running => "running",
            DevServerStatus::Stopping => "stopping",
            DevServerStatus::Error => "error",
        }
    }
}

/// Framework information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Framework {
    pub name: String,
    pub version: Option<String>,
}

/// Development server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerConfig {
    pub project_type: ProjectType,
    pub dev_command: String,
    pub port: u16,
    pub package_manager: PackageManager,
    pub framework: Option<Framework>,
}

/// Development server instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerInstance {
    pub id: Uuid,
    pub project_id: String,
    pub config: DevServerConfig,
    pub status: DevServerStatus,
    pub preview_url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub pid: Option<u32>,
}

/// Log entry type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogType {
    Stdout,
    Stderr,
    System,
}

/// Development server log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevServerLog {
    pub timestamp: DateTime<Utc>,
    pub log_type: LogType,
    pub message: String,
}

/// Project detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetectionResult {
    pub project_type: ProjectType,
    pub framework: Option<Framework>,
    pub package_manager: PackageManager,
    pub has_lock_file: bool,
    pub dev_command: String,
    pub port: u16,
    pub scripts: Option<HashMap<String, String>>,
}

/// Server lock file data for persistence across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLockData {
    pub project_id: String,
    pub pid: u32,
    pub port: u16,
    pub started_at: DateTime<Utc>,
    pub preview_url: String,
    pub project_root: String,
}

/// Preview options for customizing the preview display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub device: Option<PreviewDevice>,
    pub theme: Option<PreviewTheme>,
}

/// Preview device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewDevice {
    Desktop,
    Tablet,
    Mobile,
}

/// Preview theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewTheme {
    Light,
    Dark,
    System,
}

/// Error types for preview operations
#[derive(Debug, thiserror::Error)]
pub enum PreviewError {
    #[error("Project not found: {project_id}")]
    ProjectNotFound { project_id: String },

    #[error("Server already running for project: {project_id}")]
    ServerAlreadyRunning { project_id: String },

    #[error("Server not running for project: {project_id}")]
    ServerNotRunning { project_id: String },

    #[error("Port {port} is already in use")]
    PortInUse { port: u16 },

    #[error("Failed to start server process: {reason}")]
    ProcessStartFailed { reason: String },

    #[error("Failed to stop server process: {reason}")]
    ProcessStopFailed { reason: String },

    #[error("Project detection failed: {reason}")]
    DetectionFailed { reason: String },
    #[error("Failed to spawn process '{command}': {error}")]
    ProcessSpawnError { command: String, error: String },
    #[error("Failed to kill process with PID {pid}: {error}")]
    ProcessKillError { pid: u32, error: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

/// Result type for preview operations
pub type PreviewResult<T> = Result<T, PreviewError>;

/// Request/Response types for API endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct StartServerRequest {
    pub custom_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartServerResponse {
    pub instance: DevServerInstance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub instance: Option<DevServerInstance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerLogsRequest {
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerLogsResponse {
    pub logs: Vec<DevServerLog>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error<E: ToString>(error: E) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.to_string()),
        }
    }
}