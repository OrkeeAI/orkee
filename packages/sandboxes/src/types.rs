// ABOUTME: Core type definitions for sandbox execution
// ABOUTME: Defines requests, responses, and execution state for containerized agent runs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sandbox provider for container execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxProvider {
    /// Local Docker containers
    Local,
    /// E2B cloud sandboxes
    E2B,
    /// Modal cloud containers
    Modal,
}

/// Container lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    /// Container is being created
    Creating,
    /// Container is running
    Running,
    /// Container has stopped
    Stopped,
    /// Container encountered an error
    Error,
}

/// Overall execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    /// Execution is queued/pending
    Pending,
    /// Execution is in progress
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed with error
    Failed,
    /// Execution was cancelled
    Cancelled,
}

/// Resource limits for container execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Memory limit in megabytes
    pub memory_mb: u64,
    /// CPU cores (can be fractional, e.g., 0.5 for half a core)
    pub cpu_cores: f64,
    /// Maximum execution time in seconds
    pub timeout_seconds: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_mb: 2048,
            cpu_cores: 2.0,
            timeout_seconds: 3600,
        }
    }
}

/// Request to start a new execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique execution ID
    pub execution_id: String,
    /// Task ID this execution belongs to
    pub task_id: String,
    /// Agent ID to use for execution
    pub agent_id: String,
    /// Model ID to use
    pub model: String,
    /// Prompt/instructions for the agent
    pub prompt: String,
    /// Sandbox provider to use
    pub provider: SandboxProvider,
    /// Container image to use (e.g., "ubuntu:22.04")
    pub container_image: String,
    /// Resource limits for the execution
    pub resource_limits: ResourceLimits,
    /// Working directory path inside container
    pub workspace_path: Option<String>,
    /// Environment variables to set
    pub environment_variables: HashMap<String, String>,
}

/// Response from execution start
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    /// Execution ID
    pub execution_id: String,
    /// Container ID assigned by Docker/provider
    pub container_id: String,
    /// Current execution status
    pub status: ExecutionStatus,
    /// Container status
    pub container_status: ContainerStatus,
    /// Vibekit session ID (if applicable)
    pub vibekit_session_id: Option<String>,
    /// Error message if status is Failed
    pub error_message: Option<String>,
}

/// Log entry from execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique log entry ID
    pub id: String,
    /// Execution ID this log belongs to
    pub execution_id: String,
    /// Timestamp of log entry (ISO 8601)
    pub timestamp: String,
    /// Log level (debug, info, warn, error, fatal)
    pub log_level: String,
    /// Log message content
    pub message: String,
    /// Source of the log (vibekit, agent, container, system)
    pub source: Option<String>,
    /// Additional structured metadata (JSON)
    pub metadata: Option<serde_json::Value>,
    /// Stack trace for errors
    pub stack_trace: Option<String>,
    /// Sequence number for ordering
    pub sequence_number: i64,
}

/// Artifact produced by execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique artifact ID
    pub id: String,
    /// Execution ID this artifact belongs to
    pub execution_id: String,
    /// Type of artifact (file, screenshot, test_report, coverage, output)
    pub artifact_type: String,
    /// Original file path in container
    pub file_path: String,
    /// File name
    pub file_name: String,
    /// File size in bytes
    pub file_size_bytes: Option<i64>,
    /// MIME type
    pub mime_type: Option<String>,
    /// Path where artifact is stored on host
    pub stored_path: Option<String>,
    /// Storage backend used (local, s3, gcs)
    pub storage_backend: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Additional metadata (JSON)
    pub metadata: Option<serde_json::Value>,
    /// Checksum for integrity verification
    pub checksum: Option<String>,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory used in megabytes
    pub memory_used_mb: u64,
    /// CPU usage percentage (0-100 per core, can exceed 100 for multi-core)
    pub cpu_usage_percent: f64,
}
