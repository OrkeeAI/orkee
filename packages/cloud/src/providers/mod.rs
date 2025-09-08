use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// Re-export submodules
pub mod s3;

/// Cloud-specific errors
#[derive(Error, Debug)]
pub enum CloudError {
    #[error("Authentication failed: {0}")]
    Authentication(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Cloud provider error: {0}")]
    Provider(String),
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("AWS SDK build error: {0}")]
    AwsBuildError(String),
    #[error("Invalid metadata")]
    InvalidMetadata,
    #[error("Quota exceeded")]
    QuotaExceeded,
    #[error("Access denied")]
    AccessDenied,
}

pub type CloudResult<T> = Result<T, CloudError>;

// Implement From for AWS SDK BuildError
impl From<aws_sdk_s3::error::BuildError> for CloudError {
    fn from(error: aws_sdk_s3::error::BuildError) -> Self {
        CloudError::AwsBuildError(error.to_string())
    }
}

/// Authentication token for cloud operations
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: String,
    pub scope: Option<Vec<String>>,
}

impl AuthToken {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() > expires,
            None => false,
        }
    }
}

/// Unique identifier for a snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

impl From<String> for SnapshotId {
    fn from(id: String) -> Self {
        SnapshotId(id)
    }
}

impl From<&str> for SnapshotId {
    fn from(id: &str) -> Self {
        SnapshotId(id.to_string())
    }
}

impl std::fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata for a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub id: SnapshotId,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub project_count: usize,
    pub version: u32,
    pub checksum: String,
    pub encrypted: bool,
    pub tags: HashMap<String, String>,
}

/// Information about a snapshot stored in the cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: SnapshotId,
    pub metadata: SnapshotMetadata,
    pub storage_path: String,
    pub last_accessed: Option<DateTime<Utc>>,
    pub etag: Option<String>,
}

/// Upload options for snapshots
#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    pub storage_class: Option<String>,
    pub metadata: HashMap<String, String>,
    pub encryption: Option<EncryptionOptions>,
    pub tags: HashMap<String, String>,
}

/// Encryption options for uploads
#[derive(Debug, Clone)]
pub struct EncryptionOptions {
    pub algorithm: String,
    pub key_id: Option<String>,
}

/// Download options for snapshots
#[derive(Debug, Clone, Default)]
pub struct DownloadOptions {
    pub byte_range: Option<(u64, u64)>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

/// List options for querying snapshots
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub max_results: Option<usize>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub prefix: Option<String>,
    pub tags: HashMap<String, String>,
}

/// Main trait for cloud storage providers
#[async_trait]
pub trait CloudProvider: Send + Sync {
    /// Get the provider name
    fn provider_name(&self) -> &'static str;

    /// Authenticate with the cloud provider
    async fn authenticate(&self, credentials: &crate::auth::CloudCredentials) -> CloudResult<AuthToken>;

    /// Test the connection and credentials
    async fn test_connection(&self, token: &AuthToken) -> CloudResult<bool>;

    /// Upload a snapshot to cloud storage
    async fn upload_snapshot(
        &self,
        token: &AuthToken,
        data: &[u8],
        metadata: SnapshotMetadata,
        options: UploadOptions,
    ) -> CloudResult<SnapshotId>;

    /// Download a snapshot from cloud storage
    async fn download_snapshot(
        &self,
        token: &AuthToken,
        id: &SnapshotId,
        options: DownloadOptions,
    ) -> CloudResult<Vec<u8>>;

    /// List snapshots in cloud storage
    async fn list_snapshots(
        &self,
        token: &AuthToken,
        options: ListOptions,
    ) -> CloudResult<Vec<SnapshotInfo>>;

    /// Get detailed information about a specific snapshot
    async fn get_snapshot_info(
        &self,
        token: &AuthToken,
        id: &SnapshotId,
    ) -> CloudResult<SnapshotInfo>;

    /// Delete a snapshot from cloud storage
    async fn delete_snapshot(&self, token: &AuthToken, id: &SnapshotId) -> CloudResult<()>;

    /// Get storage usage information
    async fn get_storage_usage(&self, token: &AuthToken) -> CloudResult<StorageUsage>;

    /// Check if a snapshot exists
    async fn snapshot_exists(&self, token: &AuthToken, id: &SnapshotId) -> CloudResult<bool>;
}

/// Storage usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub total_size_bytes: u64,
    pub snapshot_count: usize,
    pub oldest_snapshot: Option<DateTime<Utc>>,
    pub newest_snapshot: Option<DateTime<Utc>>,
    pub quota_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
}

/// Factory for creating cloud providers
pub struct CloudProviderFactory;

impl CloudProviderFactory {
    /// Create a cloud provider instance for S3
    pub fn create_s3_provider(bucket: String, region: String) -> Box<dyn CloudProvider> {
        Box::new(s3::S3Provider::new(bucket, region))
    }

    /// Create a cloud provider instance for Cloudflare R2
    pub fn create_r2_provider(bucket: String, account_id: String) -> Box<dyn CloudProvider> {
        Box::new(s3::S3Provider::new_r2(bucket, account_id))
    }
}

// Utility functions
pub fn generate_snapshot_id() -> SnapshotId {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let random_suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    SnapshotId(format!("snap_{}_{}", timestamp, random_suffix))
}

pub fn calculate_checksum(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_expiry() {
        let expired_token = AuthToken {
            token: "test".to_string(),
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(expired_token.is_expired());

        let valid_token = AuthToken {
            token: "test".to_string(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(!valid_token.is_expired());

        let no_expiry_token = AuthToken {
            token: "test".to_string(),
            expires_at: None,
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(!no_expiry_token.is_expired());
    }

    #[test]
    fn test_snapshot_id_creation() {
        let id = generate_snapshot_id();
        assert!(id.0.starts_with("snap_"));
        assert!(id.0.len() > 15); // Should have timestamp + random suffix
    }

    #[test]
    fn test_checksum_calculation() {
        let data = b"test data";
        let checksum1 = calculate_checksum(data);
        let checksum2 = calculate_checksum(data);
        assert_eq!(checksum1, checksum2);

        let different_data = b"different test data";
        let checksum3 = calculate_checksum(different_data);
        assert_ne!(checksum1, checksum3);
    }
}