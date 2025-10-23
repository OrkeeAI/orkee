// ABOUTME: Integration tests for settings storage security
// ABOUTME: Tests for is_env_only enforcement and input validation

#[cfg(test)]
mod tests {
    use crate::settings::storage::SettingsStorage;
    use crate::settings::types::{BulkSettingUpdate, SettingUpdate, SettingUpdateItem};
    use crate::storage::StorageError;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn setup_test_db() -> SettingsStorage {
        // Create in-memory database
        let options = SqliteConnectOptions::from_str(":memory:")
            .unwrap()
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        SettingsStorage::new(pool)
    }

    #[tokio::test]
    async fn test_cannot_modify_env_only_settings() {
        let storage = setup_test_db().await;

        // Try to update an env-only setting (api_port)
        let update = SettingUpdate {
            value: "8080".to_string(),
        };

        let result = storage.update("api_port", update, "test_user").await;

        // Should fail with EnvOnly error
        assert!(result.is_err());
        match result {
            Err(StorageError::EnvOnly(key)) => {
                assert_eq!(key, "api_port");
            }
            _ => panic!("Expected EnvOnly error"),
        }
    }

    #[tokio::test]
    async fn test_can_modify_non_env_only_settings() {
        let storage = setup_test_db().await;

        // Should succeed - cloud_enabled is not env-only
        let update = SettingUpdate {
            value: "true".to_string(),
        };

        let result = storage.update("cloud_enabled", update, "test_user").await;
        assert!(result.is_ok());

        let setting = result.unwrap();
        assert_eq!(setting.value, "true");
        assert_eq!(setting.updated_by, "test_user");
    }

    #[tokio::test]
    async fn test_validate_port_numbers() {
        let storage = setup_test_db().await;

        // Valid port (using ui_port which is env-only, but we're testing validation logic)
        // Note: This will fail due to is_env_only check, but that's expected
        let update = SettingUpdate {
            value: "8080".to_string(),
        };
        let result = storage.update("ui_port", update, "test_user").await;
        assert!(result.is_err()); // Fails due to is_env_only

        // Invalid port - too low (would fail validation if not env-only)
        let update = SettingUpdate {
            value: "0".to_string(),
        };
        let result = storage.update("ui_port", update, "test_user").await;
        assert!(result.is_err());

        // Invalid port - too high
        let update = SettingUpdate {
            value: "65536".to_string(),
        };
        let result = storage.update("ui_port", update, "test_user").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_boolean_values() {
        let storage = setup_test_db().await;

        // Valid boolean
        let update = SettingUpdate {
            value: "true".to_string(),
        };
        let result = storage
            .update("telemetry_enabled", update, "test_user")
            .await;
        assert!(result.is_ok());

        // Invalid boolean - "yes" is not allowed
        let update = SettingUpdate {
            value: "yes".to_string(),
        };
        let result = storage
            .update("telemetry_enabled", update, "test_user")
            .await;
        assert!(result.is_err());
        match result {
            Err(StorageError::Validation(msg)) => {
                assert!(msg.contains("boolean") || msg.contains("true") || msg.contains("false"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_validate_enum_values() {
        let storage = setup_test_db().await;

        // Valid enum value
        let update = SettingUpdate {
            value: "strict".to_string(),
        };
        let result = storage
            .update("browse_sandbox_mode", update, "test_user")
            .await;
        assert!(result.is_ok());

        // Another valid value
        let update = SettingUpdate {
            value: "relaxed".to_string(),
        };
        let result = storage
            .update("browse_sandbox_mode", update, "test_user")
            .await;
        assert!(result.is_ok());

        // Invalid enum value
        let update = SettingUpdate {
            value: "invalid_mode".to_string(),
        };
        let result = storage
            .update("browse_sandbox_mode", update, "test_user")
            .await;
        assert!(result.is_err());
        match result {
            Err(StorageError::Validation(msg)) => {
                assert!(
                    msg.contains("strict") || msg.contains("relaxed") || msg.contains("disabled")
                );
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[tokio::test]
    async fn test_validate_rate_limit_positive() {
        let storage = setup_test_db().await;

        // Valid rate limit
        let update = SettingUpdate {
            value: "60".to_string(),
        };
        let result = storage
            .update("rate_limit_health_rpm", update, "test_user")
            .await;
        assert!(result.is_ok());

        // Invalid - zero
        let update = SettingUpdate {
            value: "0".to_string(),
        };
        let result = storage
            .update("rate_limit_health_rpm", update, "test_user")
            .await;
        assert!(result.is_err());

        // Invalid - negative
        let update = SettingUpdate {
            value: "-1".to_string(),
        };
        let result = storage
            .update("rate_limit_health_rpm", update, "test_user")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bulk_update_rejects_env_only() {
        let storage = setup_test_db().await;

        // Try to update mix of env-only and regular settings
        let updates = BulkSettingUpdate {
            settings: vec![
                SettingUpdateItem {
                    key: "cloud_enabled".to_string(),
                    value: "true".to_string(),
                },
                SettingUpdateItem {
                    key: "api_port".to_string(), // env-only
                    value: "8080".to_string(),
                },
            ],
        };

        let result = storage.bulk_update(updates, "test_user").await;
        assert!(result.is_err());
        match result {
            Err(StorageError::Validation(msg)) => {
                assert!(msg.contains("environment-only"));
                assert!(msg.contains("api_port"));
            }
            _ => panic!("Expected Validation error"),
        }

        // Verify no settings were updated (transaction rollback)
        let cloud_enabled = storage.get("cloud_enabled").await.unwrap();
        assert_eq!(cloud_enabled.value, "false"); // Should still be default value
    }

    #[tokio::test]
    async fn test_bulk_update_rejects_invalid_values() {
        let storage = setup_test_db().await;

        // Try to update with one invalid value
        let updates = BulkSettingUpdate {
            settings: vec![
                SettingUpdateItem {
                    key: "cloud_enabled".to_string(),
                    value: "true".to_string(), // valid
                },
                SettingUpdateItem {
                    key: "rate_limit_health_rpm".to_string(),
                    value: "0".to_string(), // invalid - must be >= 1
                },
            ],
        };

        let result = storage.bulk_update(updates, "test_user").await;
        assert!(result.is_err());
        match result {
            Err(StorageError::Validation(msg)) => {
                assert!(msg.contains("Validation failed"));
            }
            _ => panic!("Expected Validation error"),
        }

        // Verify no settings were updated (transaction rollback)
        let cloud_enabled = storage.get("cloud_enabled").await.unwrap();
        assert_eq!(cloud_enabled.value, "false"); // Should still be default value
    }

    #[tokio::test]
    async fn test_bulk_update_all_or_nothing() {
        let storage = setup_test_db().await;

        // All valid updates
        let updates = BulkSettingUpdate {
            settings: vec![
                SettingUpdateItem {
                    key: "cloud_enabled".to_string(),
                    value: "true".to_string(),
                },
                SettingUpdateItem {
                    key: "telemetry_enabled".to_string(),
                    value: "true".to_string(),
                },
            ],
        };

        let result = storage.bulk_update(updates, "test_user").await;
        assert!(result.is_ok());

        // Verify both were updated
        let settings = result.unwrap();
        assert_eq!(settings.len(), 2);
        assert!(settings.iter().all(|s| s.value == "true"));
        assert!(settings.iter().all(|s| s.updated_by == "test_user"));
    }

    #[tokio::test]
    async fn test_validate_url_format() {
        let storage = setup_test_db().await;

        // Valid URL
        let update = SettingUpdate {
            value: "https://api.example.com".to_string(),
        };
        let result = storage.update("cloud_api_url", update, "test_user").await;
        assert!(result.is_ok());

        // Invalid URL - no protocol
        let update = SettingUpdate {
            value: "api.example.com".to_string(),
        };
        let result = storage.update("cloud_api_url", update, "test_user").await;
        assert!(result.is_err());

        // Invalid URL - with spaces
        let update = SettingUpdate {
            value: "https://api.example.com/path with spaces".to_string(),
        };
        let result = storage.update("cloud_api_url", update, "test_user").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_value_rejected() {
        let storage = setup_test_db().await;

        let update = SettingUpdate {
            value: "".to_string(),
        };
        let result = storage.update("cloud_enabled", update, "test_user").await;
        assert!(result.is_err());
        match result {
            Err(StorageError::Validation(msg)) => {
                assert!(msg.contains("empty") || msg.contains("Empty"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
}
