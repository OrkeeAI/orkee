use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export common types from providers
pub use crate::providers::{SnapshotId, SnapshotMetadata, SnapshotInfo, StorageUsage};

/// Credentials for authenticating with cloud providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudCredentials {
    /// AWS-style access key and secret
    AwsCredentials {
        access_key_id: String,
        secret_access_key: String,
        session_token: Option<String>,
        region: String,
    },
    /// OAuth2 token
    OAuth2 {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    },
    /// API key based authentication
    ApiKey {
        key: String,
        secret: Option<String>,
    },
    /// Service account key (JSON)
    ServiceAccount {
        key_data: String,
    },
}

/// Progress information for upload/download operations
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub elapsed_seconds: u64,
    pub estimated_remaining_seconds: Option<u64>,
}

impl ProgressInfo {
    pub fn new(bytes_transferred: u64, total_bytes: u64, elapsed_seconds: u64) -> Self {
        let percentage = if total_bytes > 0 {
            (bytes_transferred as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        let estimated_remaining_seconds = if bytes_transferred > 0 && elapsed_seconds > 0 {
            let rate = bytes_transferred as f64 / elapsed_seconds as f64;
            let remaining_bytes = total_bytes.saturating_sub(bytes_transferred);
            Some((remaining_bytes as f64 / rate) as u64)
        } else {
            None
        };

        Self {
            bytes_transferred,
            total_bytes,
            percentage,
            elapsed_seconds,
            estimated_remaining_seconds,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.bytes_transferred >= self.total_bytes
    }
}

/// Callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(ProgressInfo) + Send + Sync>;

/// Sync operation status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Preparing,
    InProgress,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

/// Result of a sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub status: SyncStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub snapshot_id: Option<SnapshotId>,
    pub bytes_transferred: u64,
    pub projects_affected: usize,
    pub error_message: Option<String>,
}

impl SyncResult {
    pub fn starting(started_at: DateTime<Utc>) -> Self {
        Self {
            status: SyncStatus::Preparing,
            started_at,
            completed_at: None,
            snapshot_id: None,
            bytes_transferred: 0,
            projects_affected: 0,
            error_message: None,
        }
    }

    pub fn in_progress(
        started_at: DateTime<Utc>,
        bytes_transferred: u64,
        projects_affected: usize,
    ) -> Self {
        Self {
            status: SyncStatus::InProgress,
            started_at,
            completed_at: None,
            snapshot_id: None,
            bytes_transferred,
            projects_affected,
            error_message: None,
        }
    }

    pub fn completed(
        started_at: DateTime<Utc>,
        snapshot_id: SnapshotId,
        bytes_transferred: u64,
        projects_affected: usize,
    ) -> Self {
        Self {
            status: SyncStatus::Completed,
            started_at,
            completed_at: Some(Utc::now()),
            snapshot_id: Some(snapshot_id),
            bytes_transferred,
            projects_affected,
            error_message: None,
        }
    }

    pub fn failed(started_at: DateTime<Utc>, error: String) -> Self {
        Self {
            status: SyncStatus::Failed,
            started_at,
            completed_at: Some(Utc::now()),
            snapshot_id: None,
            bytes_transferred: 0,
            projects_affected: 0,
            error_message: Some(error),
        }
    }
}

/// Sync operation type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncOperation {
    Backup,
    Restore,
    List,
    Test,
}

/// Sync configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub auto_sync_enabled: bool,
    pub sync_interval_hours: u32,
    pub max_snapshots: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub cleanup_old_snapshots: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync_enabled: false,
            sync_interval_hours: 24,
            max_snapshots: 30,
            compression_enabled: true,
            encryption_enabled: true,
            cleanup_old_snapshots: true,
        }
    }
}

/// Cloud snapshot data structure for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSnapshot {
    pub id: String,
    pub provider_name: String,
    pub snapshot_id: String,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub project_count: usize,
    pub version: u32,
    pub checksum: String,
    pub encrypted: bool,
    pub storage_path: String,
    pub etag: Option<String>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub download_count: i64,
    pub last_downloaded_at: Option<DateTime<Utc>>,
    pub locally_deleted: bool,
    pub deletion_scheduled_at: Option<DateTime<Utc>>,
    pub metadata_json: String,
    pub tags_json: String,
}

/// Cloud sync state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncState {
    pub id: i64,
    pub provider_name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_successful_sync_at: Option<DateTime<Utc>>,
    pub last_snapshot_id: Option<String>,
    pub sync_interval_minutes: Option<i64>,
    pub auto_sync_enabled: bool,
    pub max_snapshots: Option<i64>,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config_json: Option<String>,
}

/// Sync conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub id: i64,
    pub provider_name: String,
    pub snapshot_id: Option<String>,
    pub project_id: String,
    pub detected_at: DateTime<Utc>,
    pub conflict_type: String,
    pub local_value: Option<String>,
    pub remote_value: Option<String>,
    pub local_version: Option<i64>,
    pub remote_version: Option<i64>,
    pub resolution_status: String,
    pub resolution_strategy: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub resolution_notes: Option<String>,
}

/// Sync health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHealthSummary {
    pub provider_name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub auto_sync_enabled: bool,
    pub last_successful_sync_at: Option<DateTime<Utc>>,
    pub error_count: i64,
    pub snapshot_count: i64,
    pub pending_conflicts: i64,
    pub latest_snapshot_at: Option<DateTime<Utc>>,
}

/// Snapshot validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub checksum_matches: bool,
    pub size_matches: bool,
    pub metadata_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            checksum_matches: true,
            size_matches: true,
            metadata_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            checksum_matches: false,
            size_matches: false,
            metadata_valid: false,
            errors,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        // For now, warnings are just added to errors - could be separate in the future
        self.errors.push(format!("Warning: {}", warning));
    }
}