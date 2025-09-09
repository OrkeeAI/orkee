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

/// Cloud project representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProject {
    pub id: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_sync: Option<DateTime<Utc>>,
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
            let error_msg = self.error
                .map(|e| format!("{}: {}", e.error, e.message))
                .unwrap_or_else(|| "Unknown API error".to_string());
            Err(crate::error::CloudError::api(error_msg))
        }
    }
}

// Note: Conversion functions are handled by the CLI layer to avoid circular dependencies