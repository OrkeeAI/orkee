use crate::types::{
    Priority, Project, ProjectCreateInput, ProjectStatus, ProjectUpdateInput, TaskSource,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// Re-export modules
pub mod factory;
pub mod legacy;
pub mod sqlite;
pub mod cloud;
pub mod sync;
pub mod cloud_state;

/// Storage errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Project not found")]
    NotFound,
    #[error("Invalid configuration format")]
    InvalidFormat,
    #[error("Duplicate project name: {0}")]
    DuplicateName(String),
    #[error("Duplicate project path: {0}")]
    DuplicatePath(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub provider: StorageProvider,
    pub enable_wal: bool,
    pub enable_fts: bool,
    pub max_connections: u32,
    pub busy_timeout_seconds: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            provider: StorageProvider::Sqlite {
                path: crate::constants::orkee_dir().join("orkee.db"),
            },
            enable_wal: true,
            enable_fts: true,
            max_connections: 10,
            busy_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageProvider {
    Sqlite {
        path: PathBuf,
    },
    Cloud {
        provider: CloudProvider,
        local_cache: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    S3 { bucket: String, region: String },
    Convex { deployment_url: String },
}

/// Main storage trait that all storage implementations must implement
#[async_trait]
pub trait ProjectStorage: Send + Sync {
    // Initialization
    async fn initialize(&self) -> StorageResult<()>;

    // Core CRUD operations
    async fn create_project(&self, input: ProjectCreateInput) -> StorageResult<Project>;
    async fn get_project(&self, id: &str) -> StorageResult<Option<Project>>;
    async fn get_project_by_name(&self, name: &str) -> StorageResult<Option<Project>>;
    async fn get_project_by_path(&self, path: &str) -> StorageResult<Option<Project>>;
    async fn list_projects(&self) -> StorageResult<Vec<Project>>;
    async fn update_project(&self, id: &str, input: ProjectUpdateInput) -> StorageResult<Project>;
    async fn delete_project(&self, id: &str) -> StorageResult<()>;

    // Advanced queries
    async fn list_projects_with_filter(&self, filter: ProjectFilter)
        -> StorageResult<Vec<Project>>;
    async fn search_projects(&self, query: &str) -> StorageResult<Vec<Project>>;
    async fn bulk_update(
        &self,
        updates: Vec<(String, ProjectUpdateInput)>,
    ) -> StorageResult<Vec<Project>>;

    // Storage information
    async fn get_storage_info(&self) -> StorageResult<StorageInfo>;

    // Cloud sync operations (for future use)
    async fn export_snapshot(&self) -> StorageResult<Vec<u8>>;
    async fn import_snapshot(&self, data: &[u8]) -> StorageResult<ImportResult>;
}

/// Filter for querying projects
#[derive(Debug, Clone, Default)]
pub struct ProjectFilter {
    pub status: Option<ProjectStatus>,
    pub priority: Option<Priority>,
    pub task_source: Option<TaskSource>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
    pub updated_after: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Information about the storage system
#[derive(Debug)]
pub struct StorageInfo {
    pub provider: String,
    pub version: String,
    pub total_projects: usize,
    pub last_modified: DateTime<Utc>,
    pub size_bytes: u64,
    pub capabilities: StorageCapabilities,
}

/// Capabilities of the storage system
#[derive(Debug)]
pub struct StorageCapabilities {
    pub full_text_search: bool,
    pub real_time_sync: bool,
    pub offline_mode: bool,
    pub multi_user: bool,
    pub cloud_backup: bool,
}

/// Result of importing data
#[derive(Debug)]
pub struct ImportResult {
    pub projects_imported: usize,
    pub projects_skipped: usize,
    pub conflicts: Vec<ImportConflict>,
}

#[derive(Debug)]
pub struct ImportConflict {
    pub project_id: String,
    pub project_name: String,
    pub conflict_type: ConflictType,
}

#[derive(Debug)]
pub enum ConflictType {
    DuplicateName,
    DuplicatePath,
    VersionConflict,
}

/// Snapshot of database for export/import
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSnapshot {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub projects: Vec<Project>,
}

/// Utility functions for compression
pub fn compress_data(data: &[u8]) -> StorageResult<Vec<u8>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| StorageError::Compression(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| StorageError::Compression(e.to_string()))
}

pub fn decompress_data(data: &[u8]) -> StorageResult<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| StorageError::Compression(e.to_string()))?;
    Ok(decompressed)
}

/// Generate a unique project ID
pub fn generate_project_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

// Re-export legacy JSON storage functions for backward compatibility
pub use legacy::{ensure_projects_file, path_exists, read_projects_config, write_projects_config};
