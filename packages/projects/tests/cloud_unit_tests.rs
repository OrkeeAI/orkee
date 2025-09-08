#[cfg(test)]
mod cloud_unit_tests {
    use orkee_projects::storage::cloud::{
        auth::{CredentialProvider, EnvironmentCredentialProvider, PasswordStrengthChecker, StrengthLevel},
        config::{CloudConfigManager, ProviderConfig, SecurityConfig},
        encryption::{
            EncryptionConfig, EncryptionAlgorithm, KeyDerivationConfig, 
            SnapshotEncryptor, EncryptedSnapshotManager, KeySource
        },
        types::{
            SyncConfig, SyncStatus, SyncResult, SyncOperation, ProgressInfo, 
            ValidationResult, RetryConfig, ProviderCapabilities
        },
        CloudCredentials, AuthToken, SnapshotId, SnapshotMetadata,
        generate_snapshot_id, calculate_checksum,
    };
    use chrono::{Utc, Duration};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_snapshot_id_generation_and_validation() {
        let id1 = generate_snapshot_id();
        let id2 = generate_snapshot_id();
        
        // Should be unique
        assert_ne!(id1.0, id2.0);
        
        // Should follow expected format
        assert!(id1.0.starts_with("snap_"));
        assert!(id1.0.len() > 15);
        
        // Test string conversion
        let id_str = id1.to_string();
        let id_from_str = SnapshotId::from(id_str.clone());
        assert_eq!(id_from_str.0, id_str);
        
        // Test from &str
        let id_from_ref = SnapshotId::from("test_id");
        assert_eq!(id_from_ref.0, "test_id");
    }

    #[test]
    fn test_checksum_calculation() {
        let data1 = b"test data for checksum";
        let data2 = b"test data for checksum";
        let data3 = b"different test data";
        
        let checksum1 = calculate_checksum(data1);
        let checksum2 = calculate_checksum(data2);
        let checksum3 = calculate_checksum(data3);
        
        // Same data should produce same checksum
        assert_eq!(checksum1, checksum2);
        
        // Different data should produce different checksums
        assert_ne!(checksum1, checksum3);
        
        // Should be hex string
        assert!(checksum1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(checksum1.len() > 0);
    }

    #[test]
    fn test_auth_token_expiry() {
        let expired_token = AuthToken {
            token: "expired".to_string(),
            expires_at: Some(Utc::now() - Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(expired_token.is_expired());

        let valid_token = AuthToken {
            token: "valid".to_string(),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(!valid_token.is_expired());

        let no_expiry_token = AuthToken {
            token: "no_expiry".to_string(),
            expires_at: None,
            token_type: "Bearer".to_string(),
            scope: None,
        };
        assert!(!no_expiry_token.is_expired());
    }

    #[test]
    fn test_cloud_credentials_variants() {
        // Test AWS credentials
        let aws_creds = CloudCredentials::AwsCredentials {
            access_key_id: "ACCESS_KEY".to_string(),
            secret_access_key: "SECRET_KEY".to_string(),
            session_token: Some("SESSION_TOKEN".to_string()),
            region: "us-east-1".to_string(),
        };
        
        match aws_creds {
            CloudCredentials::AwsCredentials { access_key_id, .. } => {
                assert_eq!(access_key_id, "ACCESS_KEY");
            }
            _ => panic!("Expected AWS credentials"),
        }

        // Test OAuth2 credentials
        let oauth_creds = CloudCredentials::OAuth2 {
            access_token: "ACCESS_TOKEN".to_string(),
            refresh_token: Some("REFRESH_TOKEN".to_string()),
            expires_at: Some(Utc::now() + Duration::hours(1)),
        };
        
        match oauth_creds {
            CloudCredentials::OAuth2 { access_token, .. } => {
                assert_eq!(access_token, "ACCESS_TOKEN");
            }
            _ => panic!("Expected OAuth2 credentials"),
        }

        // Test API key credentials
        let api_key_creds = CloudCredentials::ApiKey {
            key: "API_KEY".to_string(),
            secret: Some("API_SECRET".to_string()),
        };
        
        match api_key_creds {
            CloudCredentials::ApiKey { key, .. } => {
                assert_eq!(key, "API_KEY");
            }
            _ => panic!("Expected API key credentials"),
        }
    }

    #[test]
    fn test_encryption_config_defaults() {
        let config = EncryptionConfig::default();
        
        assert!(matches!(config.algorithm, EncryptionAlgorithm::Aes256Gcm));
        assert!(config.compression_before_encryption);
        assert!(config.include_integrity_check);
        
        let kd_config = &config.key_derivation;
        assert!(kd_config.iterations > 0);
        assert!(kd_config.memory_cost > 0);
        assert!(kd_config.parallelism > 0);
        assert!(kd_config.salt_length > 0);
    }

    #[test]
    fn test_key_derivation_config() {
        let config = KeyDerivationConfig::default();
        
        assert_eq!(config.iterations, 3);
        assert_eq!(config.memory_cost, 65536);
        assert_eq!(config.parallelism, 4);
        assert_eq!(config.salt_length, 32);
        
        // Test custom config
        let custom_config = KeyDerivationConfig {
            iterations: 5,
            memory_cost: 131072,
            parallelism: 8,
            salt_length: 16,
        };
        
        assert_eq!(custom_config.iterations, 5);
        assert_eq!(custom_config.memory_cost, 131072);
    }

    #[test]
    fn test_password_strength_checker() {
        // Test very weak password
        let weak = PasswordStrengthChecker::check_strength("123");
        assert_eq!(weak.level, StrengthLevel::VeryWeak);
        assert!(!weak.issues.is_empty());
        assert!(!weak.recommendations.is_empty());

        // Test medium password
        let medium = PasswordStrengthChecker::check_strength("Password123");
        assert!(matches!(medium.level, StrengthLevel::Fair | StrengthLevel::Good));

        // Test strong password
        let strong = PasswordStrengthChecker::check_strength("MyVeryStr0ng&SecureP@ssw0rd!");
        assert!(matches!(strong.level, StrengthLevel::Good | StrengthLevel::Excellent));
        assert!(strong.score >= 70);

        // Test password with common patterns
        let common = PasswordStrengthChecker::check_strength("password123");
        assert!(common.issues.iter().any(|issue| issue.contains("common")));

        // Test empty password
        let empty = PasswordStrengthChecker::check_strength("");
        assert_eq!(empty.level, StrengthLevel::VeryWeak);
    }

    #[tokio::test]
    async fn test_provider_config_creation() {
        // Test S3 provider config
        let s3_config = ProviderConfig::new_s3(
            "my-s3".to_string(),
            "my-bucket".to_string(),
            "us-west-2".to_string(),
            Some("https://custom-endpoint".to_string()),
        );
        
        assert_eq!(s3_config.provider_type, "s3");
        assert_eq!(s3_config.name, "my-s3");
        assert_eq!(s3_config.get_setting("bucket").unwrap(), "my-bucket");
        assert_eq!(s3_config.get_setting("region").unwrap(), "us-west-2");
        assert_eq!(s3_config.get_setting("endpoint").unwrap(), "https://custom-endpoint");
        assert!(s3_config.enabled);

        // Test R2 provider config
        let r2_config = ProviderConfig::new_r2(
            "my-r2".to_string(),
            "my-r2-bucket".to_string(),
            "account123".to_string(),
        );
        
        assert_eq!(r2_config.provider_type, "r2");
        assert_eq!(r2_config.name, "my-r2");
        assert_eq!(r2_config.get_setting("bucket").unwrap(), "my-r2-bucket");
        assert_eq!(r2_config.get_setting("account_id").unwrap(), "account123");
        assert_eq!(r2_config.get_setting("region").unwrap(), "auto");
        assert!(r2_config.get_setting("endpoint").unwrap().contains("account123"));
    }

    #[tokio::test]
    async fn test_cloud_config_manager_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CloudConfigManager::new(temp_dir.path().to_path_buf());
        
        // Test initial load
        manager.load().await.unwrap();
        assert!(!manager.get_config().enabled);
        
        // Test provider management
        let s3_config = ProviderConfig::new_s3(
            "test-s3".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            None,
        );
        
        manager.add_provider(s3_config).await.unwrap();
        assert_eq!(manager.list_providers().len(), 1);
        assert!(manager.get_provider("test-s3").is_some());
        
        // Test default provider setting
        manager.set_default_provider("test-s3").await.unwrap();
        assert_eq!(
            manager.get_default_provider().unwrap().name, 
            "test-s3"
        );
        
        // Test enabling/disabling
        manager.enable_cloud().await.unwrap();
        assert!(manager.get_config().enabled);
        
        manager.disable_cloud().await.unwrap();
        assert!(!manager.get_config().enabled);
        
        // Test configuration validation
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_sync_types_and_operations() {
        // Test SyncStatus display
        assert_eq!(SyncStatus::Idle.to_string(), "Idle");
        assert_eq!(SyncStatus::Uploading.to_string(), "Uploading");
        assert_eq!(SyncStatus::Completed.to_string(), "Completed");
        assert_eq!(SyncStatus::Failed.to_string(), "Failed");

        // Test SyncOperation display
        assert_eq!(SyncOperation::Backup.to_string(), "Backup");
        assert_eq!(SyncOperation::Restore.to_string(), "Restore");
        assert_eq!(SyncOperation::FullSync.to_string(), "Full Sync");

        // Test SyncResult creation
        let started = SyncResult::started();
        assert_eq!(started.status, SyncStatus::Preparing);
        assert_eq!(started.bytes_transferred, 0);
        
        let now = Utc::now();
        let snapshot_id = generate_snapshot_id();
        let completed = SyncResult::completed(now, snapshot_id.clone(), 1024, 5);
        assert_eq!(completed.status, SyncStatus::Completed);
        assert_eq!(completed.bytes_transferred, 1024);
        assert_eq!(completed.projects_synced, 5);
        assert!(completed.duration_seconds.unwrap() >= 0);
        
        let failed = SyncResult::failed(now, "Test error".to_string());
        assert_eq!(failed.status, SyncStatus::Failed);
        assert_eq!(failed.error_message.unwrap(), "Test error");
    }

    #[test]
    fn test_progress_info() {
        let progress = ProgressInfo::new(50, 100, 10);
        assert_eq!(progress.bytes_transferred, 50);
        assert_eq!(progress.total_bytes, 100);
        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.elapsed_seconds, 10);
        assert!(!progress.is_complete());
        
        let complete = ProgressInfo::new(100, 100, 20);
        assert_eq!(complete.percentage, 100.0);
        assert!(complete.is_complete());
        
        // Test with zero total bytes
        let zero_total = ProgressInfo::new(0, 0, 5);
        assert_eq!(zero_total.percentage, 0.0);
        assert!(zero_total.is_complete());
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.checksum_matches);
        assert!(result.size_matches);
        assert!(result.metadata_valid);
        assert!(result.errors.is_empty());
        
        result.add_error("Test error".to_string());
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        
        result.add_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);
        assert!(!result.is_valid); // Still invalid due to error
        
        let invalid = ValidationResult::invalid(vec![
            "Error 1".to_string(),
            "Error 2".to_string(),
        ]);
        assert!(!invalid.is_valid);
        assert_eq!(invalid.errors.len(), 2);
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.jitter);
        
        // Test delay calculation without jitter
        let no_jitter_config = RetryConfig {
            jitter: false,
            ..Default::default()
        };
        
        assert_eq!(no_jitter_config.calculate_delay(0), 1000);
        assert_eq!(no_jitter_config.calculate_delay(1), 2000);
        assert_eq!(no_jitter_config.calculate_delay(2), 4000);
        
        // Should cap at max delay
        assert_eq!(no_jitter_config.calculate_delay(10), 30000);
        
        // Test with jitter
        let jitter_config = RetryConfig::default();
        let delay1 = jitter_config.calculate_delay(1);
        let delay2 = jitter_config.calculate_delay(1);
        
        // Should be within expected range (with jitter variation)
        assert!(delay1 >= 1000 && delay1 <= 3000);
        assert!(delay2 >= 1000 && delay2 <= 3000);
    }

    #[test]
    fn test_provider_capabilities() {
        let capabilities = ProviderCapabilities {
            supports_multipart_upload: true,
            supports_resume: false,
            supports_encryption: true,
            supports_versioning: true,
            supports_lifecycle: true,
            supports_tags: true,
            max_file_size_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
            max_objects: None,
            regions: vec![
                "us-east-1".to_string(),
                "us-west-2".to_string(),
                "eu-west-1".to_string(),
            ],
        };
        
        assert!(capabilities.supports_multipart_upload);
        assert!(!capabilities.supports_resume);
        assert!(capabilities.supports_encryption);
        assert_eq!(capabilities.regions.len(), 3);
        assert!(capabilities.max_objects.is_none());
    }

    #[test]
    fn test_sync_config_defaults() {
        let config = SyncConfig::default();
        
        assert!(!config.auto_sync_enabled);
        assert_eq!(config.sync_interval_hours, 24);
        assert_eq!(config.max_snapshots, 30);
        assert_eq!(config.compression_level, 6);
        assert!(config.encrypt_snapshots);
        assert!(config.include_deleted);
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.retry_delay_seconds, 5);
        assert_eq!(config.timeout_seconds, 300);
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        
        assert!(config.encrypt_snapshots);
        assert_eq!(config.encryption_algorithm, "AES-256-GCM");
        assert_eq!(config.key_derivation_iterations, 100000);
        assert!(!config.require_mfa);
        assert_eq!(config.max_credential_age_days, 90);
        assert!(config.audit_logging);
    }

    #[test]
    fn test_environment_credential_provider() {
        // Test AWS credentials from environment
        std::env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
        std::env::set_var("AWS_REGION", "us-east-1");
        
        let creds = EnvironmentCredentialProvider::get_aws_credentials(None).unwrap();
        match creds {
            CloudCredentials::AwsCredentials { access_key_id, secret_access_key, region, .. } => {
                assert_eq!(access_key_id, "test_access_key");
                assert_eq!(secret_access_key, "test_secret_key");
                assert_eq!(region, "us-east-1");
            }
            _ => panic!("Expected AWS credentials"),
        }
        
        // Test with custom region override
        let creds_with_region = EnvironmentCredentialProvider::get_aws_credentials(
            Some("us-west-2".to_string())
        ).unwrap();
        match creds_with_region {
            CloudCredentials::AwsCredentials { region, .. } => {
                assert_eq!(region, "us-west-2");
            }
            _ => panic!("Expected AWS credentials"),
        }
        
        // Clean up
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_REGION");
        
        // Test API key credentials
        std::env::set_var("API_KEY", "test_api_key");
        std::env::set_var("API_SECRET", "test_api_secret");
        
        let api_creds = EnvironmentCredentialProvider::get_api_key_credentials(
            "API_KEY", 
            Some("API_SECRET")
        ).unwrap();
        match api_creds {
            CloudCredentials::ApiKey { key, secret } => {
                assert_eq!(key, "test_api_key");
                assert_eq!(secret.unwrap(), "test_api_secret");
            }
            _ => panic!("Expected API key credentials"),
        }
        
        std::env::remove_var("API_KEY");
        std::env::remove_var("API_SECRET");
    }

    #[test]
    fn test_credential_provider_creation() {
        let provider = CredentialProvider::new("s3".to_string());
        assert_eq!(provider.provider_name, "s3");
        
        let r2_provider = CredentialProvider::new("r2".to_string());
        assert_eq!(r2_provider.provider_name, "r2");
    }

    #[test]
    fn test_snapshot_metadata_creation() {
        let snapshot_id = generate_snapshot_id();
        let now = Utc::now();
        let mut tags = HashMap::new();
        tags.insert("environment".to_string(), "test".to_string());
        tags.insert("backup_type".to_string(), "manual".to_string());
        
        let metadata = SnapshotMetadata {
            id: snapshot_id.clone(),
            created_at: now,
            size_bytes: 1024 * 1024, // 1MB
            compressed_size_bytes: 512 * 1024, // 512KB
            project_count: 10,
            version: 1,
            checksum: "abcdef123456".to_string(),
            encrypted: true,
            tags: tags.clone(),
        };
        
        assert_eq!(metadata.id, snapshot_id);
        assert_eq!(metadata.size_bytes, 1024 * 1024);
        assert_eq!(metadata.compressed_size_bytes, 512 * 1024);
        assert_eq!(metadata.project_count, 10);
        assert!(metadata.encrypted);
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.tags.get("environment").unwrap(), "test");
    }

    #[test]
    fn test_encryption_config_serialization() {
        let config = EncryptionConfig::default();
        
        // Test serialization
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("Aes256Gcm"));
        assert!(serialized.contains("compression_before_encryption"));
        
        // Test deserialization
        let deserialized: EncryptionConfig = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized.algorithm, EncryptionAlgorithm::Aes256Gcm));
        assert_eq!(deserialized.compression_before_encryption, config.compression_before_encryption);
        assert_eq!(deserialized.include_integrity_check, config.include_integrity_check);
    }
}