//! API request and response models for Orkee Cloud

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Authentication request
#[derive(Debug, Serialize)]
pub struct AuthRequest {
    pub auth_code: String,
}

/// Authentication response
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: User,
}

/// Token refresh request
#[derive(Debug, Serialize)]
pub struct RefreshRequest {
    pub token: String,
}

/// Token refresh response
#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// User information
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
}

/// Project sync request
#[derive(Debug, Serialize)]
pub struct ProjectSyncRequest {
    pub project: CloudProject,
    pub snapshot_data: String, // Base64 encoded project data
}

/// Project sync response
#[derive(Debug, Deserialize)]
pub struct ProjectSyncResponse {
    pub snapshot_id: String,
    pub synced_at: DateTime<Utc>,
}

/// Cloud project representation with full OSS compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProject {
    pub id: String,  // 8-character format
    pub name: String,
    pub path: String,  // project_root
    pub description: Option<String>,
    pub setup_script: Option<String>,
    pub dev_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub tags: Vec<String>,
    pub status: String,  // "active" or "archived"
    pub priority: String,  // "high", "medium", "low"
    pub rank: Option<u32>,
    pub task_source: Option<String>,  // "taskmaster" or "manual"
    pub mcp_servers: std::collections::HashMap<String, bool>,
    pub git_repository: Option<GitRepositoryInfo>,
    pub manual_tasks: Option<Vec<serde_json::Value>>,  // Serialized tasks
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepositoryInfo {
    pub owner: String,
    pub repo: String,
    pub url: String,
    pub branch: Option<String>,
}

/// List projects response
#[derive(Debug, Deserialize)]
pub struct ListProjectsResponse {
    pub projects: Vec<CloudProject>,
}

/// Project restore response
#[derive(Debug, Deserialize)]
pub struct RestoreResponse {
    pub project: CloudProject,
    pub snapshot_data: String, // Base64 encoded project data
    pub snapshot_id: String,
    pub created_at: DateTime<Utc>,
}

/// Usage statistics
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub projects_count: usize,
    pub storage_used_bytes: u64,
    pub storage_limit_bytes: u64,
    pub api_calls_this_month: u64,
    pub api_calls_limit: u64,
    pub subscription_tier: String,
}

/// Standard API error response
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(default)]
    pub details: Option<String>,
}

/// Conflict report for sync operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictReport {
    pub has_conflicts: bool,
    pub conflicts: Vec<FieldConflict>,
    pub local_updated_at: chrono::DateTime<chrono::Utc>,
    pub cloud_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldConflict {
    pub field: String,
    pub local_value: serde_json::Value,
    pub cloud_value: serde_json::Value,
}

/// Conflict resolution strategy
#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub strategy: ConflictStrategy,
    pub field_resolutions: Option<Vec<FieldResolution>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    LocalWins,
    CloudWins,
    Merge,
    Manual,
    Newest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldResolution {
    pub field: String,
    pub use_value: String,  // "local" or "cloud"
}

/// Project diff for incremental sync
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDiff {
    pub changed_fields: Vec<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_root: Option<String>,
    pub setup_script: Option<String>,
    pub dev_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub rank: Option<u32>,
    pub task_source: Option<String>,
    pub mcp_servers: Option<std::collections::HashMap<String, bool>>,
    pub git_repository: Option<GitRepositoryInfo>,
}

/// Generic API response wrapper
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> ApiResponse<T> {
    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the data, returning an error if the response was unsuccessful
    pub fn into_result(self) -> Result<T, crate::error::CloudError> {
        if self.success {
            self.data.ok_or_else(|| {
                crate::error::CloudError::api("Response indicated success but contained no data")
            })
        } else {
            let error_msg = self
                .error
                .map(|e| format!("{}: {}", e.error, e.message))
                .unwrap_or_else(|| "Unknown API error".to_string());
            Err(crate::error::CloudError::api(error_msg))
        }
    }
}

// Note: Conversion functions are handled by the CLI layer to avoid circular dependencies
