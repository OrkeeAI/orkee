use super::super::{ProjectStorage, StorageResult, StorageError, ImportResult};
use crate::storage::cloud::{
    CloudProvider, CloudResult, CloudError, SnapshotId, SnapshotMetadata, UploadOptions, 
    DownloadOptions, ListOptions, AuthToken, types::{SyncResult, SyncStatus, SyncOperation}
};
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error, debug};

/// Configuration for the sync engine
#[derive(Debug, Clone)]
pub struct SyncEngineConfig {
    pub auto_sync_enabled: bool,
    pub sync_interval_minutes: u64,
    pub max_concurrent_uploads: usize,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub max_snapshot_age_days: u32,
    pub cleanup_old_snapshots: bool,
}

impl Default for SyncEngineConfig {
    fn default() -> Self {
        Self {
            auto_sync_enabled: false,
            sync_interval_minutes: 60,
            max_concurrent_uploads: 3,
            retry_attempts: 3,
            retry_delay_seconds: 5,
            compression_enabled: true,
            encryption_enabled: true,
            max_snapshot_age_days: 30,
            cleanup_old_snapshots: true,
        }
    }
}

/// Sync state tracking
#[derive(Debug, Clone)]
pub struct SyncState {
    pub last_sync: Option<DateTime<Utc>>,
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub last_snapshot_id: Option<SnapshotId>,
    pub sync_in_progress: bool,
    pub current_operation: Option<SyncOperation>,
    pub error_count: u32,
    pub last_error: Option<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync: None,
            last_successful_sync: None,
            last_snapshot_id: None,
            sync_in_progress: false,
            current_operation: None,
            error_count: 0,
            last_error: None,
        }
    }
}

/// Main sync engine that coordinates between local storage and cloud providers
pub struct SyncEngine {
    local_storage: Arc<dyn ProjectStorage>,
    cloud_provider: Arc<dyn CloudProvider>,
    auth_token: Arc<RwLock<Option<AuthToken>>>,
    config: Arc<RwLock<SyncEngineConfig>>,
    state: Arc<RwLock<SyncState>>,
    sync_mutex: Arc<Mutex<()>>, // Ensures only one sync operation at a time
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(
        local_storage: Arc<dyn ProjectStorage>,
        cloud_provider: Arc<dyn CloudProvider>,
        config: SyncEngineConfig,
    ) -> Self {
        Self {
            local_storage,
            cloud_provider,
            auth_token: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(SyncState::default())),
            sync_mutex: Arc::new(Mutex::new(())),
        }
    }

    /// Set authentication token
    pub async fn set_auth_token(&self, token: AuthToken) {
        let mut auth_guard = self.auth_token.write().await;
        *auth_guard = Some(token);
        info!("Authentication token set for sync engine");
    }

    /// Get current sync state
    pub async fn get_sync_state(&self) -> SyncState {
        self.state.read().await.clone()
    }

    /// Update sync configuration
    pub async fn update_config(&self, config: SyncEngineConfig) {
        let mut config_guard = self.config.write().await;
        *config_guard = config;
        info!("Sync engine configuration updated");
    }

    /// Perform a full backup to cloud storage
    pub async fn backup(&self) -> CloudResult<SyncResult> {
        let _lock = self.sync_mutex.lock().await;
        let started_at = Utc::now();

        info!("Starting cloud backup");
        self.update_sync_state(|state| {
            state.sync_in_progress = true;
            state.current_operation = Some(SyncOperation::Backup);
            state.last_sync = Some(started_at);
        }).await;

        match self.perform_backup_internal().await {
            Ok(result) => {
                info!(
                    "Backup completed successfully: {} projects, {} bytes", 
                    result.projects_synced, 
                    result.bytes_transferred
                );

                self.update_sync_state(|state| {
                    state.sync_in_progress = false;
                    state.current_operation = None;
                    state.last_successful_sync = result.completed_at;
                    state.last_snapshot_id = result.snapshot_id.clone();
                    state.error_count = 0;
                    state.last_error = None;
                }).await;

                Ok(result)
            }
            Err(error) => {
                error!("Backup failed: {}", error);
                
                let failed_result = SyncResult::failed(started_at, error.to_string());
                
                self.update_sync_state(|state| {
                    state.sync_in_progress = false;
                    state.current_operation = None;
                    state.error_count += 1;
                    state.last_error = Some(error.to_string());
                }).await;

                Err(error)
            }
        }
    }

    /// Restore from a cloud snapshot
    pub async fn restore(&self, snapshot_id: &SnapshotId) -> CloudResult<SyncResult> {
        let _lock = self.sync_mutex.lock().await;
        let started_at = Utc::now();

        info!("Starting restore from snapshot: {}", snapshot_id);
        self.update_sync_state(|state| {
            state.sync_in_progress = true;
            state.current_operation = Some(SyncOperation::Restore);
            state.last_sync = Some(started_at);
        }).await;

        match self.perform_restore_internal(snapshot_id).await {
            Ok(result) => {
                info!(
                    "Restore completed successfully: {} projects", 
                    result.projects_synced
                );

                self.update_sync_state(|state| {
                    state.sync_in_progress = false;
                    state.current_operation = None;
                    state.last_successful_sync = result.completed_at;
                    state.error_count = 0;
                    state.last_error = None;
                }).await;

                Ok(result)
            }
            Err(error) => {
                error!("Restore failed: {}", error);
                
                let failed_result = SyncResult::failed(started_at, error.to_string());
                
                self.update_sync_state(|state| {
                    state.sync_in_progress = false;
                    state.current_operation = None;
                    state.error_count += 1;
                    state.last_error = Some(error.to_string());
                }).await;

                Err(error)
            }
        }
    }

    /// List available snapshots in cloud storage
    pub async fn list_snapshots(&self) -> CloudResult<Vec<crate::storage::cloud::SnapshotInfo>> {
        let auth_token = self.get_valid_auth_token().await?;
        let options = ListOptions::default();
        
        self.cloud_provider.list_snapshots(&auth_token, options).await
    }

    /// Check if sync is needed based on configuration and last sync time
    pub async fn should_sync(&self) -> bool {
        let config = self.config.read().await;
        let state = self.state.read().await;

        if !config.auto_sync_enabled || state.sync_in_progress {
            return false;
        }

        match state.last_successful_sync {
            Some(last_sync) => {
                let interval = Duration::minutes(config.sync_interval_minutes as i64);
                Utc::now() > last_sync + interval
            }
            None => true, // Never synced before
        }
    }

    /// Start automatic sync loop (runs in background)
    pub async fn start_auto_sync(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(60) // Check every minute
            );

            loop {
                interval.tick().await;

                if self.should_sync().await {
                    debug!("Auto-sync triggered");
                    
                    if let Err(error) = self.backup().await {
                        warn!("Auto-sync backup failed: {}", error);
                    }
                }
            }
        })
    }

    /// Clean up old snapshots based on configuration
    pub async fn cleanup_old_snapshots(&self) -> CloudResult<u32> {
        let config = self.config.read().await;
        
        if !config.cleanup_old_snapshots {
            return Ok(0);
        }

        let auth_token = self.get_valid_auth_token().await?;
        let cutoff_date = Utc::now() - Duration::days(config.max_snapshot_age_days as i64);
        
        let list_options = ListOptions {
            created_before: Some(cutoff_date),
            ..Default::default()
        };

        let old_snapshots = self.cloud_provider.list_snapshots(&auth_token, list_options).await?;
        let mut deleted_count = 0u32;

        for snapshot in old_snapshots {
            match self.cloud_provider.delete_snapshot(&auth_token, &snapshot.id).await {
                Ok(_) => {
                    info!("Deleted old snapshot: {}", snapshot.id);
                    deleted_count += 1;
                }
                Err(e) => {
                    warn!("Failed to delete snapshot {}: {}", snapshot.id, e);
                }
            }
        }

        info!("Cleaned up {} old snapshots", deleted_count);
        Ok(deleted_count)
    }

    /// Get storage usage from cloud provider
    pub async fn get_cloud_usage(&self) -> CloudResult<crate::storage::cloud::StorageUsage> {
        let auth_token = self.get_valid_auth_token().await?;
        self.cloud_provider.get_storage_usage(&auth_token).await
    }

    // Internal implementation methods

    async fn perform_backup_internal(&self) -> CloudResult<SyncResult> {
        let started_at = Utc::now();

        // Export snapshot from local storage
        debug!("Exporting local snapshot");
        let snapshot_data = self.local_storage
            .export_snapshot()
            .await
            .map_err(|e| CloudError::Provider(format!("Failed to export local snapshot: {}", e)))?;

        // Get project count for metadata
        let projects = self.local_storage
            .list_projects()
            .await
            .map_err(|e| CloudError::Provider(format!("Failed to list projects: {}", e)))?;

        // Create snapshot metadata
        let snapshot_id = crate::storage::cloud::generate_snapshot_id();
        let checksum = crate::storage::cloud::calculate_checksum(&snapshot_data);
        
        let metadata = SnapshotMetadata {
            id: snapshot_id.clone(),
            created_at: Utc::now(),
            size_bytes: snapshot_data.len() as u64,
            compressed_size_bytes: snapshot_data.len() as u64, // Already compressed by export
            project_count: projects.len(),
            version: 1,
            checksum,
            encrypted: false, // TODO: Add encryption support
            tags: std::collections::HashMap::new(),
        };

        // Upload to cloud
        debug!("Uploading snapshot to cloud: {}", snapshot_id);
        let auth_token = self.get_valid_auth_token().await?;
        let upload_options = UploadOptions::default();

        let uploaded_id = self.cloud_provider
            .upload_snapshot(&auth_token, &snapshot_data, metadata.clone(), upload_options)
            .await?;

        if uploaded_id != snapshot_id {
            warn!("Uploaded snapshot ID differs from generated ID: {} vs {}", uploaded_id, snapshot_id);
        }

        Ok(SyncResult::completed(
            started_at,
            snapshot_id,
            snapshot_data.len() as u64,
            projects.len(),
        ))
    }

    async fn perform_restore_internal(&self, snapshot_id: &SnapshotId) -> CloudResult<SyncResult> {
        let started_at = Utc::now();

        // Download snapshot from cloud
        debug!("Downloading snapshot from cloud: {}", snapshot_id);
        let auth_token = self.get_valid_auth_token().await?;
        let download_options = DownloadOptions::default();

        let snapshot_data = self.cloud_provider
            .download_snapshot(&auth_token, snapshot_id, download_options)
            .await?;

        // Import snapshot into local storage
        debug!("Importing snapshot into local storage");
        let import_result = self.local_storage
            .import_snapshot(&snapshot_data)
            .await
            .map_err(|e| CloudError::Provider(format!("Failed to import snapshot: {}", e)))?;

        info!(
            "Import completed: {} projects imported, {} skipped, {} conflicts",
            import_result.projects_imported,
            import_result.projects_skipped,
            import_result.conflicts.len()
        );

        if !import_result.conflicts.is_empty() {
            warn!("Import had conflicts that may need manual resolution");
            for conflict in &import_result.conflicts {
                warn!("Conflict: {} - {:?}", conflict.project_name, conflict.conflict_type);
            }
        }

        Ok(SyncResult::completed(
            started_at,
            snapshot_id.clone(),
            snapshot_data.len() as u64,
            import_result.projects_imported,
        ))
    }

    async fn get_valid_auth_token(&self) -> CloudResult<AuthToken> {
        let auth_guard = self.auth_token.read().await;
        
        match &*auth_guard {
            Some(token) => {
                if token.is_expired() {
                    Err(CloudError::Authentication("Auth token has expired".to_string()))
                } else {
                    Ok(token.clone())
                }
            }
            None => Err(CloudError::Authentication("No auth token available".to_string())),
        }
    }

    async fn update_sync_state<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut SyncState),
    {
        let mut state_guard = self.state.write().await;
        update_fn(&mut *state_guard);
    }

    /// Validate snapshot integrity
    pub async fn validate_snapshot(&self, snapshot_id: &SnapshotId) -> CloudResult<crate::storage::cloud::types::ValidationResult> {
        let auth_token = self.get_valid_auth_token().await?;
        
        // Get snapshot info
        let snapshot_info = self.cloud_provider.get_snapshot_info(&auth_token, snapshot_id).await?;
        
        // Download snapshot data for validation
        let download_options = DownloadOptions::default();
        let snapshot_data = self.cloud_provider
            .download_snapshot(&auth_token, snapshot_id, download_options)
            .await?;

        let mut validation_result = crate::storage::cloud::types::ValidationResult::valid();

        // Validate size
        if snapshot_data.len() as u64 != snapshot_info.metadata.compressed_size_bytes {
            validation_result.size_matches = false;
            validation_result.add_error(format!(
                "Size mismatch: expected {}, got {}",
                snapshot_info.metadata.compressed_size_bytes,
                snapshot_data.len()
            ));
        }

        // Validate checksum
        let actual_checksum = crate::storage::cloud::calculate_checksum(&snapshot_data);
        if actual_checksum != snapshot_info.metadata.checksum {
            validation_result.checksum_matches = false;
            validation_result.add_error(format!(
                "Checksum mismatch: expected {}, got {}",
                snapshot_info.metadata.checksum,
                actual_checksum
            ));
        }

        // Try to parse the snapshot data
        match self.local_storage.import_snapshot(&snapshot_data).await {
            Ok(_) => {
                validation_result.add_warning("Snapshot data is valid and parseable".to_string());
            }
            Err(e) => {
                validation_result.metadata_valid = false;
                validation_result.add_error(format!("Failed to parse snapshot data: {}", e));
            }
        }

        Ok(validation_result)
    }
}

/// Factory for creating sync engines with different configurations
pub struct SyncEngineFactory;

impl SyncEngineFactory {
    /// Create a sync engine for S3
    pub async fn create_s3_sync_engine(
        local_storage: Arc<dyn ProjectStorage>,
        bucket: String,
        region: String,
        config: SyncEngineConfig,
    ) -> CloudResult<SyncEngine> {
        let s3_provider = crate::storage::cloud::s3::S3Provider::new(bucket, region);
        
        Ok(SyncEngine::new(
            local_storage,
            Arc::new(s3_provider),
            config,
        ))
    }

    /// Create a sync engine for Cloudflare R2
    pub async fn create_r2_sync_engine(
        local_storage: Arc<dyn ProjectStorage>,
        bucket: String,
        account_id: String,
        config: SyncEngineConfig,
    ) -> CloudResult<SyncEngine> {
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);
        let r2_provider = crate::storage::cloud::s3::S3Provider::new_with_endpoint(
            bucket,
            "auto".to_string(),
            endpoint,
        );
        
        Ok(SyncEngine::new(
            local_storage,
            Arc::new(r2_provider),
            config,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::SqliteStorage;
    use crate::storage::{StorageConfig, StorageProvider};
    use mockall::{mock, predicate::*};
    use std::collections::HashMap;
    use tempfile::TempDir;

    // Mock implementations for testing
    mock! {
        CloudProvider {}
        
        #[async_trait::async_trait]
        impl CloudProvider for CloudProvider {
            fn provider_name(&self) -> &'static str;
            async fn authenticate(&self, credentials: &crate::storage::cloud::CloudCredentials) -> CloudResult<AuthToken>;
            async fn test_connection(&self, token: &AuthToken) -> CloudResult<bool>;
            async fn upload_snapshot(&self, token: &AuthToken, data: &[u8], metadata: SnapshotMetadata, options: UploadOptions) -> CloudResult<SnapshotId>;
            async fn download_snapshot(&self, token: &AuthToken, id: &SnapshotId, options: DownloadOptions) -> CloudResult<Vec<u8>>;
            async fn list_snapshots(&self, token: &AuthToken, options: ListOptions) -> CloudResult<Vec<crate::storage::cloud::SnapshotInfo>>;
            async fn get_snapshot_info(&self, token: &AuthToken, id: &SnapshotId) -> CloudResult<crate::storage::cloud::SnapshotInfo>;
            async fn delete_snapshot(&self, token: &AuthToken, id: &SnapshotId) -> CloudResult<()>;
            async fn get_storage_usage(&self, token: &AuthToken) -> CloudResult<crate::storage::cloud::StorageUsage>;
            async fn snapshot_exists(&self, token: &AuthToken, id: &SnapshotId) -> CloudResult<bool>;
        }
    }

    #[tokio::test]
    async fn test_sync_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = StorageConfig {
            provider: StorageProvider::Sqlite { path: db_path },
            enable_wal: true,
            enable_fts: false,
            max_connections: 1,
            busy_timeout_seconds: 30,
        };
        
        let storage = Arc::new(SqliteStorage::new(config).await.unwrap());
        let mut mock_provider = MockCloudProvider::new();
        
        mock_provider.expect_provider_name().return_const("test");
        
        let sync_config = SyncEngineConfig::default();
        let engine = SyncEngine::new(
            storage,
            Arc::new(mock_provider),
            sync_config,
        );

        let state = engine.get_sync_state().await;
        assert!(!state.sync_in_progress);
        assert!(state.last_sync.is_none());
    }

    #[tokio::test]
    async fn test_sync_state_updates() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = StorageConfig {
            provider: StorageProvider::Sqlite { path: db_path },
            enable_wal: true,
            enable_fts: false,
            max_connections: 1,
            busy_timeout_seconds: 30,
        };
        
        let storage = Arc::new(SqliteStorage::new(config).await.unwrap());
        let mut mock_provider = MockCloudProvider::new();
        
        mock_provider.expect_provider_name().return_const("test");
        
        let sync_config = SyncEngineConfig::default();
        let engine = SyncEngine::new(
            storage,
            Arc::new(mock_provider),
            sync_config,
        );

        // Test auth token setting
        let token = AuthToken {
            token: "test-token".to_string(),
            expires_at: None,
            token_type: "Bearer".to_string(),
            scope: None,
        };

        engine.set_auth_token(token.clone()).await;
        
        let auth_guard = engine.auth_token.read().await;
        assert!(auth_guard.is_some());
        assert_eq!(auth_guard.as_ref().unwrap().token, "test-token");
    }

    #[test]
    fn test_sync_engine_config() {
        let config = SyncEngineConfig::default();
        
        assert!(!config.auto_sync_enabled);
        assert_eq!(config.sync_interval_minutes, 60);
        assert_eq!(config.max_concurrent_uploads, 3);
        assert_eq!(config.retry_attempts, 3);
        assert!(config.compression_enabled);
        assert!(config.encryption_enabled);
    }
}