use chrono::Utc;
use orkee_projects::storage::{
    cloud::{
        encryption::{EncryptionConfig, SnapshotEncryptor, KeySource, EncryptedSnapshotManager},
        auth::{CredentialProvider, CredentialStore, CloudCredentials},
        config::{CloudConfigManager, ProviderConfig},
        s3::S3Provider,
        sync::{SyncEngine, SyncEngineConfig, SyncEngineFactory},
        types::SyncConfig,
        CloudProvider, AuthToken, SnapshotMetadata, UploadOptions, DownloadOptions, 
        ListOptions, generate_snapshot_id, calculate_checksum,
    },
    cloud_state::{CloudSyncStateManager, CloudSyncState, CloudSnapshot},
    sqlite::{SqliteStorage, SqliteConfig},
    StorageConfig, ProjectStorage,
};
use orkee_projects::types::{ProjectCreateInput, ProjectStatus, Priority};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio_test;

/// Integration tests for cloud storage functionality
#[cfg(test)]
mod cloud_integration_tests {
    use super::*;

    async fn create_test_storage() -> (Arc<SqliteStorage>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = SqliteConfig {
            database_url: format!("sqlite://{}?mode=rwc", db_path.display()),
            max_connections: 5,
        };
        
        let storage = SqliteStorage::new(config).await.unwrap();
        (Arc::new(storage), temp_dir)
    }

    async fn create_test_projects(storage: &Arc<dyn ProjectStorage>) -> Vec<String> {
        let mut project_ids = Vec::new();
        
        for i in 1..=3 {
            let input = ProjectCreateInput {
                name: format!("Test Project {}", i),
                project_root: format!("/tmp/project{}", i),
                description: Some(format!("Test project {} description", i)),
                status: Some(ProjectStatus::Active),
                priority: Some(Priority::Medium),
                setup_script: None,
                dev_script: None,
                cleanup_script: None,
                tags: vec![format!("test{}", i), "integration".to_string()],
            };
            
            let project = storage.create_project(input).await.unwrap();
            project_ids.push(project.id);
        }
        
        project_ids
    }

    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let config = EncryptionConfig::default();
        let encryptor = SnapshotEncryptor::new(config);
        
        let test_data = b"This is test snapshot data for encryption testing";
        let passphrase = "secure_test_passphrase_123";
        
        // Encrypt
        let encrypted = encryptor.encrypt_with_passphrase(test_data, passphrase).unwrap();
        assert_ne!(encrypted.encrypted_data, test_data);
        assert_eq!(encrypted.metadata.original_size, test_data.len());
        assert!(encrypted.metadata.checksum.is_some());
        
        // Decrypt
        let decrypted = encryptor.decrypt_with_passphrase(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, test_data);
        
        // Test wrong passphrase
        let wrong_result = encryptor.decrypt_with_passphrase(&encrypted, "wrong_passphrase");
        assert!(wrong_result.is_err());
    }

    #[tokio::test]
    async fn test_encrypted_snapshot_manager() {
        let config = EncryptionConfig::default();
        let mut manager = EncryptedSnapshotManager::new(config);
        
        let test_data = b"Snapshot data for manager testing";
        let passphrase = "manager_test_passphrase";
        
        // Test with passphrase
        let encrypted = manager.encrypt_snapshot(
            test_data, 
            KeySource::Passphrase(passphrase.to_string())
        ).unwrap();
        
        let decrypted = manager.decrypt_snapshot(
            &encrypted, 
            KeySource::Passphrase(passphrase.to_string())
        ).unwrap();
        assert_eq!(decrypted, test_data);
        
        // Test encryption info
        let info = manager.get_encryption_info(&encrypted);
        assert_eq!(info.original_size, test_data.len());
        assert!(info.encrypted_size > test_data.len());
        assert!(info.has_integrity_check);
    }

    #[tokio::test]
    async fn test_cloud_config_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut config_manager = CloudConfigManager::new(temp_dir.path().to_path_buf());
        
        // Test initial state
        config_manager.load().await.unwrap();
        assert!(!config_manager.get_config().enabled);
        assert!(config_manager.list_providers().is_empty());
        
        // Test adding provider
        let s3_config = ProviderConfig::new_s3(
            "test-s3".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            None,
        );
        config_manager.add_provider(s3_config).await.unwrap();
        
        assert_eq!(config_manager.list_providers().len(), 1);
        assert!(config_manager.get_provider("test-s3").is_some());
        assert_eq!(config_manager.get_config().default_provider, Some("test-s3".to_string()));
        
        // Test enabling cloud
        config_manager.enable_cloud().await.unwrap();
        assert!(config_manager.get_config().enabled);
        
        // Test removing provider
        config_manager.remove_provider("test-s3").await.unwrap();
        assert!(config_manager.list_providers().is_empty());
        assert_eq!(config_manager.get_config().default_provider, None);
    }

    #[tokio::test]
    async fn test_cloud_sync_state_manager() {
        let (storage, _temp_dir) = create_test_storage().await;
        let state_manager = CloudSyncStateManager::new(storage.get_pool().clone());
        
        // Test upserting provider state
        let state = state_manager.upsert_provider_state("test-provider", "s3").await.unwrap();
        assert_eq!(state.provider_name, "test-provider");
        assert_eq!(state.provider_type, "s3");
        assert!(state.enabled);
        
        // Test updating sync state
        let snapshot_id = "test-snapshot-123";
        state_manager.update_sync_state(
            "test-provider",
            Some(Utc::now()),
            Some(Utc::now()),
            Some(snapshot_id),
            false,
            None,
            None,
        ).await.unwrap();
        
        let updated_state = state_manager.get_provider_state("test-provider").await.unwrap().unwrap();
        assert_eq!(updated_state.last_snapshot_id, Some(snapshot_id.to_string()));
        assert!(!updated_state.sync_in_progress);
        
        // Test storing snapshot metadata
        let snapshot = CloudSnapshot {
            id: "snapshot-1".to_string(),
            provider_name: "test-provider".to_string(),
            snapshot_id: snapshot_id.to_string(),
            created_at: Utc::now(),
            size_bytes: 1024,
            compressed_size_bytes: 512,
            project_count: 3,
            version: 1,
            checksum: Some("abc123".to_string()),
            encrypted: true,
            storage_path: Some("/snapshots/test-snapshot-123".to_string()),
            etag: Some("etag123".to_string()),
            last_accessed_at: None,
            uploaded_at: Some(Utc::now()),
            download_count: 0,
            last_downloaded_at: None,
            locally_deleted: false,
            deletion_scheduled_at: None,
            metadata_json: None,
            tags_json: None,
        };
        
        state_manager.store_snapshot_metadata(&snapshot).await.unwrap();
        
        // Test listing snapshots
        let snapshots = state_manager.list_snapshots("test-provider", Some(10)).await.unwrap();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].snapshot_id, snapshot_id);
        
        // Test logging sync operation
        let log_id = state_manager.log_sync_operation(
            "test-provider",
            "backup",
            Some("op-123"),
            Some(snapshot_id),
            3,
            1024,
            "completed",
            None,
            Some(30),
        ).await.unwrap();
        assert!(log_id > 0);
        
        // Test recording conflict
        let conflict_id = state_manager.record_sync_conflict(
            "test-provider",
            Some(snapshot_id),
            "project-1",
            "duplicate_name",
            Some("local_name"),
            Some("remote_name"),
            Some(1),
            Some(2),
        ).await.unwrap();
        assert!(conflict_id > 0);
        
        let conflicts = state_manager.get_pending_conflicts("test-provider").await.unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].project_id, "project-1");
        
        // Test health summary
        let summaries = state_manager.get_sync_health_summary().await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].provider_name, "test-provider");
        assert_eq!(summaries[0].snapshot_count, 1);
        assert_eq!(summaries[0].pending_conflicts, 1);
    }

    #[tokio::test]
    async fn test_full_backup_restore_cycle() {
        let (storage, _temp_dir) = create_test_storage().await;
        let project_ids = create_test_projects(&storage).await;
        
        // Export snapshot
        let snapshot_data = storage.export_snapshot().await.unwrap();
        assert!(!snapshot_data.is_empty());
        
        // Test encryption of snapshot
        let config = EncryptionConfig::default();
        let mut manager = EncryptedSnapshotManager::new(config);
        
        let encrypted = manager.encrypt_snapshot(
            &snapshot_data,
            KeySource::Passphrase("backup_test_passphrase".to_string()),
        ).unwrap();
        
        // Decrypt and verify
        let decrypted = manager.decrypt_snapshot(
            &encrypted,
            KeySource::Passphrase("backup_test_passphrase".to_string()),
        ).unwrap();
        assert_eq!(decrypted, snapshot_data);
        
        // Test import (restore)
        let import_result = storage.import_snapshot(&decrypted).await.unwrap();
        assert_eq!(import_result.projects_imported, 3); // Should skip existing projects
        assert_eq!(import_result.conflicts.len(), 3); // Duplicate names
        
        // Verify projects still exist
        let projects = storage.list_projects().await.unwrap();
        assert_eq!(projects.len(), 3);
    }

    #[tokio::test]
    async fn test_snapshot_metadata_operations() {
        let snapshot_id = generate_snapshot_id();
        assert!(snapshot_id.0.starts_with("snap_"));
        assert!(snapshot_id.0.len() > 15);
        
        let test_data = b"test data for checksum";
        let checksum1 = calculate_checksum(test_data);
        let checksum2 = calculate_checksum(test_data);
        assert_eq!(checksum1, checksum2);
        
        let different_data = b"different test data";
        let checksum3 = calculate_checksum(different_data);
        assert_ne!(checksum1, checksum3);
        
        // Test metadata creation
        let metadata = SnapshotMetadata {
            id: snapshot_id,
            created_at: Utc::now(),
            size_bytes: test_data.len() as u64,
            compressed_size_bytes: test_data.len() as u64,
            project_count: 3,
            version: 1,
            checksum: checksum1,
            encrypted: false,
            tags: HashMap::new(),
        };
        
        assert_eq!(metadata.project_count, 3);
        assert!(!metadata.encrypted);
    }

    #[tokio::test]
    async fn test_sync_engine_creation_and_state() {
        let (storage, _temp_dir) = create_test_storage().await;
        
        let sync_config = SyncEngineConfig {
            auto_sync_enabled: false,
            sync_interval_minutes: 60,
            max_concurrent_uploads: 2,
            retry_attempts: 3,
            retry_delay_seconds: 5,
            compression_enabled: true,
            encryption_enabled: true,
            max_snapshot_age_days: 30,
            cleanup_old_snapshots: true,
        };
        
        // Note: We can't fully test the sync engine without a real cloud provider
        // In a real integration test environment, you would use LocalStack or similar
        
        let factory_result = SyncEngineFactory::create_s3_sync_engine(
            storage.clone(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            sync_config.clone(),
        ).await;
        
        // This should succeed even without credentials (creation doesn't require auth)
        assert!(factory_result.is_ok());
        let sync_engine = factory_result.unwrap();
        
        // Test state management
        let state = sync_engine.get_sync_state().await;
        assert!(!state.sync_in_progress);
        assert!(state.last_sync.is_none());
        assert_eq!(state.error_count, 0);
        
        // Test auth token setting
        let token = AuthToken {
            token: "test-token".to_string(),
            expires_at: None,
            token_type: "Bearer".to_string(),
            scope: None,
        };
        sync_engine.set_auth_token(token).await;
        
        // Verify sync should not run without proper configuration
        assert!(!sync_engine.should_sync().await);
    }

    #[tokio::test]
    async fn test_credential_provider_functionality() {
        // Test environment credential provider
        std::env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
        std::env::set_var("AWS_REGION", "us-west-2");
        
        let provider = CredentialProvider::new("s3".to_string());
        let credentials = provider.get_credentials().unwrap();
        
        match credentials {
            CloudCredentials::AwsCredentials { access_key_id, secret_access_key, region, .. } => {
                assert_eq!(access_key_id, "test_access_key");
                assert_eq!(secret_access_key, "test_secret_key");
                assert_eq!(region, "us-west-2");
            }
            _ => panic!("Expected AWS credentials"),
        }
        
        // Clean up environment
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_REGION");
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config_manager = CloudConfigManager::new(temp_dir.path().to_path_buf());
        config_manager.load().await.unwrap();
        
        // Should be valid when disabled
        assert!(config_manager.validate().is_ok());
        
        // Should fail when enabled with no providers
        config_manager.enable_cloud().await.unwrap();
        assert!(config_manager.validate().is_err());
        
        // Should be valid after adding a provider
        let s3_config = ProviderConfig::new_s3(
            "test-s3".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            None,
        );
        config_manager.add_provider(s3_config).await.unwrap();
        assert!(config_manager.validate().is_ok());
        
        // Test invalid sync configuration
        let mut sync_config = SyncConfig::default();
        sync_config.sync_interval_hours = 0; // Invalid
        config_manager.update_sync_config(sync_config).await.unwrap();
        assert!(config_manager.validate().is_err());
    }

    #[tokio::test]
    async fn test_error_handling_and_edge_cases() {
        let (storage, _temp_dir) = create_test_storage().await;
        
        // Test export of empty storage
        let empty_snapshot = storage.export_snapshot().await.unwrap();
        assert!(!empty_snapshot.is_empty()); // Should contain metadata even if no projects
        
        // Test import of invalid data
        let invalid_data = b"invalid snapshot data";
        let import_result = storage.import_snapshot(invalid_data).await;
        assert!(import_result.is_err());
        
        // Test encryption with empty data
        let config = EncryptionConfig::default();
        let encryptor = SnapshotEncryptor::new(config);
        let empty_data = b"";
        let encrypted = encryptor.encrypt_with_passphrase(empty_data, "test").unwrap();
        let decrypted = encryptor.decrypt_with_passphrase(&encrypted, "test").unwrap();
        assert_eq!(decrypted, empty_data);
        
        // Test cloud state manager with non-existent provider
        let state_manager = CloudSyncStateManager::new(storage.get_pool().clone());
        let state = state_manager.get_provider_state("non-existent").await.unwrap();
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_performance_and_large_data() {
        let config = EncryptionConfig::default();
        let encryptor = SnapshotEncryptor::new(config);
        
        // Test with larger data (1MB)
        let large_data = vec![42u8; 1024 * 1024];
        let passphrase = "performance_test";
        
        let start = std::time::Instant::now();
        let encrypted = encryptor.encrypt_with_passphrase(&large_data, passphrase).unwrap();
        let encrypt_duration = start.elapsed();
        
        let start = std::time::Instant::now();
        let decrypted = encryptor.decrypt_with_passphrase(&encrypted, passphrase).unwrap();
        let decrypt_duration = start.elapsed();
        
        assert_eq!(decrypted, large_data);
        
        // Ensure encryption/decryption completes in reasonable time (< 1 second each)
        assert!(encrypt_duration.as_millis() < 1000);
        assert!(decrypt_duration.as_millis() < 1000);
        
        // Check compression effectiveness
        let compression_ratio = encrypted.encrypted_data.len() as f64 / large_data.len() as f64;
        // Should be well compressed since it's all the same byte
        assert!(compression_ratio < 0.1);
    }
}

/// Additional helper functions for testing
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn create_test_auth_token() -> AuthToken {
        AuthToken {
            token: "test-token-12345".to_string(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scope: Some(vec!["read".to_string(), "write".to_string()]),
        }
    }

    pub fn create_test_snapshot_metadata() -> SnapshotMetadata {
        SnapshotMetadata {
            id: generate_snapshot_id(),
            created_at: Utc::now(),
            size_bytes: 1024,
            compressed_size_bytes: 512,
            project_count: 3,
            version: 1,
            checksum: "test-checksum".to_string(),
            encrypted: true,
            tags: HashMap::new(),
        }
    }

    pub async fn wait_for_condition<F>(mut condition: F, timeout_ms: u64) -> bool 
    where 
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();
        
        while start.elapsed().as_millis() < timeout_ms as u128 {
            if condition() {
                return true;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        false
    }
}

// Mock implementations for testing without actual cloud providers
#[cfg(test)]
pub mod mocks {
    use super::*;
    use async_trait::async_trait;
    
    pub struct MockCloudProvider {
        pub provider_name: String,
        pub should_fail: bool,
        pub snapshots: std::sync::Mutex<Vec<(String, Vec<u8>)>>,
    }
    
    impl MockCloudProvider {
        pub fn new(name: String) -> Self {
            Self {
                provider_name: name,
                should_fail: false,
                snapshots: std::sync::Mutex::new(Vec::new()),
            }
        }
        
        pub fn with_failure(mut self, should_fail: bool) -> Self {
            self.should_fail = should_fail;
            self
        }
    }
    
    #[async_trait]
    impl CloudProvider for MockCloudProvider {
        fn provider_name(&self) -> &'static str {
            "mock"
        }
        
        async fn authenticate(&self, _credentials: &CloudCredentials) -> orkee_projects::storage::cloud::CloudResult<AuthToken> {
            if self.should_fail {
                return Err(orkee_projects::storage::cloud::CloudError::Authentication("Mock failure".to_string()));
            }
            
            Ok(AuthToken {
                token: "mock-token".to_string(),
                expires_at: None,
                token_type: "Mock".to_string(),
                scope: None,
            })
        }
        
        async fn test_connection(&self, _token: &AuthToken) -> orkee_projects::storage::cloud::CloudResult<bool> {
            Ok(!self.should_fail)
        }
        
        async fn upload_snapshot(
            &self,
            _token: &AuthToken,
            data: &[u8],
            metadata: SnapshotMetadata,
            _options: UploadOptions,
        ) -> orkee_projects::storage::cloud::CloudResult<orkee_projects::storage::cloud::SnapshotId> {
            if self.should_fail {
                return Err(orkee_projects::storage::cloud::CloudError::Provider("Mock upload failure".to_string()));
            }
            
            let mut snapshots = self.snapshots.lock().unwrap();
            snapshots.push((metadata.id.0.clone(), data.to_vec()));
            Ok(metadata.id)
        }
        
        async fn download_snapshot(
            &self,
            _token: &AuthToken,
            id: &orkee_projects::storage::cloud::SnapshotId,
            _options: DownloadOptions,
        ) -> orkee_projects::storage::cloud::CloudResult<Vec<u8>> {
            if self.should_fail {
                return Err(orkee_projects::storage::cloud::CloudError::Provider("Mock download failure".to_string()));
            }
            
            let snapshots = self.snapshots.lock().unwrap();
            for (snapshot_id, data) in snapshots.iter() {
                if snapshot_id == &id.0 {
                    return Ok(data.clone());
                }
            }
            
            Err(orkee_projects::storage::cloud::CloudError::SnapshotNotFound(id.0.clone()))
        }
        
        async fn list_snapshots(
            &self,
            _token: &AuthToken,
            _options: ListOptions,
        ) -> orkee_projects::storage::cloud::CloudResult<Vec<orkee_projects::storage::cloud::SnapshotInfo>> {
            if self.should_fail {
                return Err(orkee_projects::storage::cloud::CloudError::Provider("Mock list failure".to_string()));
            }
            
            Ok(Vec::new()) // Simplified for mock
        }
        
        async fn get_snapshot_info(
            &self,
            _token: &AuthToken,
            _id: &orkee_projects::storage::cloud::SnapshotId,
        ) -> orkee_projects::storage::cloud::CloudResult<orkee_projects::storage::cloud::SnapshotInfo> {
            Err(orkee_projects::storage::cloud::CloudError::Provider("Not implemented in mock".to_string()))
        }
        
        async fn delete_snapshot(
            &self,
            _token: &AuthToken,
            id: &orkee_projects::storage::cloud::SnapshotId,
        ) -> orkee_projects::storage::cloud::CloudResult<()> {
            if self.should_fail {
                return Err(orkee_projects::storage::cloud::CloudError::Provider("Mock delete failure".to_string()));
            }
            
            let mut snapshots = self.snapshots.lock().unwrap();
            snapshots.retain(|(snapshot_id, _)| snapshot_id != &id.0);
            Ok(())
        }
        
        async fn get_storage_usage(
            &self,
            _token: &AuthToken,
        ) -> orkee_projects::storage::cloud::CloudResult<orkee_projects::storage::cloud::StorageUsage> {
            Ok(orkee_projects::storage::cloud::StorageUsage {
                total_size_bytes: 0,
                snapshot_count: 0,
                oldest_snapshot: None,
                newest_snapshot: None,
                quota_bytes: None,
                available_bytes: None,
            })
        }
        
        async fn snapshot_exists(
            &self,
            _token: &AuthToken,
            id: &orkee_projects::storage::cloud::SnapshotId,
        ) -> orkee_projects::storage::cloud::CloudResult<bool> {
            let snapshots = self.snapshots.lock().unwrap();
            Ok(snapshots.iter().any(|(snapshot_id, _)| snapshot_id == &id.0))
        }
    }
}