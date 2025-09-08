use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::SnapshotId;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Preparing,
    Uploading,
    Downloading,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Idle => write!(f, "Idle"),
            SyncStatus::Preparing => write!(f, "Preparing"),
            SyncStatus::Uploading => write!(f, "Uploading"),
            SyncStatus::Downloading => write!(f, "Downloading"),
            SyncStatus::Processing => write!(f, "Processing"),
            SyncStatus::Completed => write!(f, "Completed"),
            SyncStatus::Failed => write!(f, "Failed"),
            SyncStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Sync operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub status: SyncStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub snapshot_id: Option<super::SnapshotId>,
    pub bytes_transferred: u64,
    pub projects_synced: usize,
    pub error_message: Option<String>,
    pub duration_seconds: Option<u64>,
}

impl SyncResult {
    pub fn started() -> Self {
        Self {
            status: SyncStatus::Preparing,
            started_at: Utc::now(),
            completed_at: None,
            snapshot_id: None,
            bytes_transferred: 0,
            projects_synced: 0,
            error_message: None,
            duration_seconds: None,
        }
    }

    pub fn completed(
        started_at: DateTime<Utc>,
        snapshot_id: super::SnapshotId,
        bytes_transferred: u64,
        projects_synced: usize,
    ) -> Self {
        let now = Utc::now();
        let duration = (now - started_at).num_seconds() as u64;

        Self {
            status: SyncStatus::Completed,
            started_at,
            completed_at: Some(now),
            snapshot_id: Some(snapshot_id),
            bytes_transferred,
            projects_synced,
            error_message: None,
            duration_seconds: Some(duration),
        }
    }

    pub fn failed(started_at: DateTime<Utc>, error: String) -> Self {
        let now = Utc::now();
        let duration = (now - started_at).num_seconds() as u64;

        Self {
            status: SyncStatus::Failed,
            started_at,
            completed_at: Some(now),
            snapshot_id: None,
            bytes_transferred: 0,
            projects_synced: 0,
            error_message: Some(error),
            duration_seconds: Some(duration),
        }
    }
}

/// Configuration for sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub auto_sync_enabled: bool,
    pub sync_interval_hours: u32,
    pub max_snapshots: usize,
    pub compression_level: u32,
    pub encrypt_snapshots: bool,
    pub include_deleted: bool,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u32,
    pub timeout_seconds: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync_enabled: false,
            sync_interval_hours: 24,
            max_snapshots: 30,
            compression_level: 6,
            encrypt_snapshots: true,
            include_deleted: true,
            retry_attempts: 3,
            retry_delay_seconds: 5,
            timeout_seconds: 300,
        }
    }
}

/// Sync operation type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncOperation {
    Backup,
    Restore,
    FullSync,
    IncrementalSync,
}

impl std::fmt::Display for SyncOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncOperation::Backup => write!(f, "Backup"),
            SyncOperation::Restore => write!(f, "Restore"),
            SyncOperation::FullSync => write!(f, "Full Sync"),
            SyncOperation::IncrementalSync => write!(f, "Incremental Sync"),
        }
    }
}

/// Snapshot validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub checksum_matches: bool,
    pub size_matches: bool,
    pub metadata_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            checksum_matches: true,
            size_matches: true,
            metadata_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            checksum_matches: false,
            size_matches: false,
            metadata_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Cloud provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supports_multipart_upload: bool,
    pub supports_resume: bool,
    pub supports_encryption: bool,
    pub supports_versioning: bool,
    pub supports_lifecycle: bool,
    pub supports_tags: bool,
    pub max_file_size_bytes: Option<u64>,
    pub max_objects: Option<usize>,
    pub regions: Vec<String>,
}

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let base_delay = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let delay = base_delay.min(self.max_delay_ms as f64) as u64;

        if self.jitter {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.5..=1.5);
            (delay as f64 * jitter_factor) as u64
        } else {
            delay
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_info() {
        let progress = ProgressInfo::new(50, 100, 10);
        assert_eq!(progress.percentage, 50.0);
        assert!(!progress.is_complete());

        let complete = ProgressInfo::new(100, 100, 20);
        assert_eq!(complete.percentage, 100.0);
        assert!(complete.is_complete());
    }

    #[test]
    fn test_sync_result_creation() {
        let result = SyncResult::started();
        assert_eq!(result.status, SyncStatus::Preparing);
        assert_eq!(result.bytes_transferred, 0);

        let completed = SyncResult::completed(
            Utc::now() - chrono::Duration::seconds(30),
            SnapshotId("test".to_string()),
            1024,
            5,
        );
        assert_eq!(completed.status, SyncStatus::Completed);
        assert_eq!(completed.bytes_transferred, 1024);
        assert_eq!(completed.projects_synced, 5);
        assert!(completed.duration_seconds.unwrap() >= 30);
    }

    #[test]
    fn test_retry_config_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            jitter: false,
        };

        assert_eq!(config.calculate_delay(0), 1000);
        assert_eq!(config.calculate_delay(1), 2000);
        assert_eq!(config.calculate_delay(2), 4000);
        assert_eq!(config.calculate_delay(10), 10000); // Should cap at max_delay_ms
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        result.add_error("Test error".to_string());
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        result.add_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);
        assert!(!result.is_valid); // Still invalid due to error
    }
}