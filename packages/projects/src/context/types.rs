use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Context configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContextConfiguration {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub include_patterns: Vec<String>,
    #[sqlx(default)]
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    pub max_tokens: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to generate context for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGenerationRequest {
    pub project_id: String,
    #[serde(default)]
    pub include_patterns: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    pub max_tokens: Option<i32>,
    #[serde(default)]
    pub save_configuration: bool,
    pub configuration_name: Option<String>,
}

/// Generated context result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedContext {
    pub content: String,
    pub file_count: usize,
    pub total_tokens: usize,
    pub files_included: Vec<String>,
    pub truncated: bool,
}

/// Context snapshot stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContextSnapshot {
    pub id: String,
    pub configuration_id: Option<String>,
    pub project_id: String,
    pub content: String,
    pub file_count: Option<i32>,
    pub total_tokens: Option<i32>,
    pub metadata: Option<String>, // JSON string
    pub created_at: DateTime<Utc>,
}

/// Metadata for a context snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub files_included: Vec<String>,
    pub generation_time_ms: u64,
    pub git_commit: Option<String>,
}

/// Usage pattern for tracking commonly included files
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContextUsagePattern {
    pub id: String,
    pub project_id: String,
    pub file_path: String,
    pub inclusion_count: i32,
    pub last_used: DateTime<Utc>,
}

/// File information for display in UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub extension: Option<String>,
    pub is_directory: bool,
}

/// Request to list files in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesRequest {
    pub project_id: String,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

/// Response with list of files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesResponse {
    pub files: Vec<FileInfo>,
    pub total_count: usize,
}
