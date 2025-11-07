// ABOUTME: Sandbox settings storage layer using SQLite
// ABOUTME: Handles CRUD operations for sandbox and provider-specific configurations

use orkee_security::ApiKeyEncryption;
use orkee_storage::StorageError;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tracing::{debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSettings {
    // General Settings
    pub enabled: bool,
    pub default_provider: String,
    pub default_image: String,
    pub docker_username: Option<String>,

    // Resource Limits
    pub max_concurrent_local: i64,
    pub max_concurrent_cloud: i64,
    pub max_cpu_cores_per_sandbox: i64,
    pub max_memory_gb_per_sandbox: i64,
    pub max_disk_gb_per_sandbox: i64,
    pub max_gpu_per_sandbox: i64,

    // Lifecycle Settings
    pub auto_stop_idle_minutes: i64,
    pub max_runtime_hours: i64,
    pub cleanup_interval_minutes: i64,
    pub preserve_stopped_sandboxes: bool,
    pub auto_restart_failed: bool,
    pub max_restart_attempts: i64,

    // Cost Management
    pub cost_tracking_enabled: bool,
    pub cost_alert_threshold: f64,
    pub max_cost_per_sandbox: f64,
    pub max_total_cost: f64,
    pub auto_stop_at_cost_limit: bool,

    // Network Settings
    pub default_network_mode: String,
    pub allow_public_endpoints: bool,
    pub require_auth_for_web: bool,

    // Security Settings
    /// SECURITY CRITICAL: Allows Docker privileged mode which grants extensive host access.
    /// Should ALWAYS be false in production. Only enable in trusted development environments.
    /// When true, containers can: access all devices, modify kernel parameters, mount filesystems.
    /// Default: false (secure)
    pub allow_privileged_containers: bool,
    pub require_non_root_user: bool,
    pub enable_security_scanning: bool,
    pub allowed_base_images: Option<serde_json::Value>,
    pub blocked_commands: Option<serde_json::Value>,

    // Monitoring
    pub resource_monitoring_interval_seconds: i64,
    pub health_check_interval_seconds: i64,
    pub log_retention_days: i64,
    pub metrics_retention_days: i64,

    // Templates
    pub allow_custom_templates: bool,
    pub require_template_approval: bool,
    pub share_templates_globally: bool,

    // Metadata
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub provider: String,

    // Status
    pub enabled: bool,
    pub configured: bool,
    pub validated_at: Option<String>,
    pub validation_error: Option<String>,

    // Credentials (encrypted)
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_endpoint: Option<String>,

    // Provider-specific IDs
    pub workspace_id: Option<String>,
    pub project_id: Option<String>,
    pub account_id: Option<String>,
    pub organization_id: Option<String>,
    pub app_name: Option<String>,
    pub namespace_id: Option<String>,

    // Defaults
    pub default_region: Option<String>,
    pub default_instance_type: Option<String>,
    pub default_image: Option<String>,

    // Resource defaults
    pub default_cpu_cores: Option<f64>,
    pub default_memory_mb: Option<i64>,
    pub default_disk_gb: Option<i64>,
    pub default_gpu_type: Option<String>,

    // Cost overrides
    pub cost_per_hour: Option<f64>,
    pub cost_per_gb_memory: Option<f64>,
    pub cost_per_vcpu: Option<f64>,
    pub cost_per_gpu_hour: Option<f64>,

    // Limits
    pub max_sandboxes: Option<i64>,
    pub max_runtime_hours: Option<i64>,
    pub max_total_cost: Option<f64>,

    // Additional configuration
    pub custom_config: Option<serde_json::Value>,

    // Metadata
    pub updated_at: String,
    pub updated_by: Option<String>,
}

pub struct SettingsManager {
    pool: SqlitePool,
    encryption: ApiKeyEncryption,
}

impl SettingsManager {
    pub fn new(pool: SqlitePool) -> Result<Self, StorageError> {
        let encryption = ApiKeyEncryption::new().map_err(|e| {
            StorageError::Encryption(format!("Failed to initialize encryption: {}", e))
        })?;
        Ok(Self { pool, encryption })
    }

    /// Get sandbox settings (singleton record)
    pub async fn get_sandbox_settings(&self) -> Result<SandboxSettings, StorageError> {
        debug!("Fetching sandbox settings");

        let row = sqlx::query("SELECT * FROM sandbox_settings WHERE id = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_sandbox_settings(&row)
    }

    /// Update sandbox settings with optimistic locking
    ///
    /// Uses the updated_at timestamp for optimistic locking to prevent
    /// concurrent modification conflicts. If the record has been modified
    /// since the settings object was loaded, the update will fail.
    pub async fn update_sandbox_settings(
        &self,
        settings: &SandboxSettings,
        updated_by: Option<&str>,
    ) -> Result<(), StorageError> {
        debug!("Updating sandbox settings with optimistic locking");

        let result = sqlx::query(
            r#"
            UPDATE sandbox_settings SET
                enabled = ?,
                default_provider = ?,
                default_image = ?,
                docker_username = ?,
                max_concurrent_local = ?,
                max_concurrent_cloud = ?,
                max_cpu_cores_per_sandbox = ?,
                max_memory_gb_per_sandbox = ?,
                max_disk_gb_per_sandbox = ?,
                max_gpu_per_sandbox = ?,
                auto_stop_idle_minutes = ?,
                max_runtime_hours = ?,
                cleanup_interval_minutes = ?,
                preserve_stopped_sandboxes = ?,
                auto_restart_failed = ?,
                max_restart_attempts = ?,
                cost_tracking_enabled = ?,
                cost_alert_threshold = ?,
                max_cost_per_sandbox = ?,
                max_total_cost = ?,
                auto_stop_at_cost_limit = ?,
                default_network_mode = ?,
                allow_public_endpoints = ?,
                require_auth_for_web = ?,
                allow_privileged_containers = ?,
                require_non_root_user = ?,
                enable_security_scanning = ?,
                allowed_base_images = ?,
                blocked_commands = ?,
                resource_monitoring_interval_seconds = ?,
                health_check_interval_seconds = ?,
                log_retention_days = ?,
                metrics_retention_days = ?,
                allow_custom_templates = ?,
                require_template_approval = ?,
                share_templates_globally = ?,
                updated_by = ?,
                updated_at = datetime('now', 'utc')
            WHERE id = 1 AND updated_at = ?
            "#,
        )
        .bind(settings.enabled as i64)
        .bind(&settings.default_provider)
        .bind(&settings.default_image)
        .bind(&settings.docker_username)
        .bind(settings.max_concurrent_local)
        .bind(settings.max_concurrent_cloud)
        .bind(settings.max_cpu_cores_per_sandbox)
        .bind(settings.max_memory_gb_per_sandbox)
        .bind(settings.max_disk_gb_per_sandbox)
        .bind(settings.max_gpu_per_sandbox)
        .bind(settings.auto_stop_idle_minutes)
        .bind(settings.max_runtime_hours)
        .bind(settings.cleanup_interval_minutes)
        .bind(settings.preserve_stopped_sandboxes as i64)
        .bind(settings.auto_restart_failed as i64)
        .bind(settings.max_restart_attempts)
        .bind(settings.cost_tracking_enabled as i64)
        .bind(settings.cost_alert_threshold)
        .bind(settings.max_cost_per_sandbox)
        .bind(settings.max_total_cost)
        .bind(settings.auto_stop_at_cost_limit as i64)
        .bind(&settings.default_network_mode)
        .bind(settings.allow_public_endpoints as i64)
        .bind(settings.require_auth_for_web as i64)
        .bind(settings.allow_privileged_containers as i64)
        .bind(settings.require_non_root_user as i64)
        .bind(settings.enable_security_scanning as i64)
        .bind(
            settings
                .allowed_base_images
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
        )
        .bind(
            settings
                .blocked_commands
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
        )
        .bind(settings.resource_monitoring_interval_seconds)
        .bind(settings.health_check_interval_seconds)
        .bind(settings.log_retention_days)
        .bind(settings.metrics_retention_days)
        .bind(settings.allow_custom_templates as i64)
        .bind(settings.require_template_approval as i64)
        .bind(settings.share_templates_globally as i64)
        .bind(updated_by)
        .bind(&settings.updated_at)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        // Check if the update affected any rows (optimistic locking check)
        if result.rows_affected() == 0 {
            return Err(StorageError::Sqlx(sqlx::Error::RowNotFound));
        }

        Ok(())
    }

    /// Get provider settings
    pub async fn get_provider_settings(
        &self,
        provider: &str,
    ) -> Result<ProviderSettings, StorageError> {
        debug!("Fetching provider settings for: {}", provider);

        let row = sqlx::query("SELECT * FROM sandbox_provider_settings WHERE provider = ?")
            .bind(provider)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_provider_settings(&row)
    }

    /// List all provider settings
    pub async fn list_provider_settings(&self) -> Result<Vec<ProviderSettings>, StorageError> {
        debug!("Fetching all provider settings");

        let rows = sqlx::query("SELECT * FROM sandbox_provider_settings ORDER BY provider")
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let mut settings = Vec::new();
        for row in rows {
            settings.push(self.row_to_provider_settings(&row)?);
        }

        Ok(settings)
    }

    /// Update provider settings
    pub async fn update_provider_settings(
        &self,
        settings: &ProviderSettings,
        updated_by: Option<&str>,
    ) -> Result<(), StorageError> {
        debug!("Updating provider settings for: {}", settings.provider);

        // Encrypt api_key if present
        let encrypted_api_key = match &settings.api_key {
            Some(key) => Some(self.encryption.encrypt(key).map_err(|e| {
                error!(
                    "Failed to encrypt api_key for provider {}: {}",
                    settings.provider, e
                );
                StorageError::Encryption(format!("API key encryption failed: {}", e))
            })?),
            None => None,
        };

        // Encrypt api_secret if present
        let encrypted_api_secret = match &settings.api_secret {
            Some(secret) => Some(self.encryption.encrypt(secret).map_err(|e| {
                error!(
                    "Failed to encrypt api_secret for provider {}: {}",
                    settings.provider, e
                );
                StorageError::Encryption(format!("API secret encryption failed: {}", e))
            })?),
            None => None,
        };

        sqlx::query(
            r#"
            INSERT INTO sandbox_provider_settings (
                provider, enabled, configured, validated_at, validation_error,
                api_key, api_secret, api_endpoint,
                workspace_id, project_id, account_id, organization_id, app_name, namespace_id,
                default_region, default_instance_type, default_image,
                default_cpu_cores, default_memory_mb, default_disk_gb, default_gpu_type,
                cost_per_hour, cost_per_gb_memory, cost_per_vcpu, cost_per_gpu_hour,
                max_sandboxes, max_runtime_hours, max_total_cost,
                custom_config, updated_by, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now', 'utc'))
            ON CONFLICT(provider) DO UPDATE SET
                enabled = excluded.enabled,
                configured = excluded.configured,
                validated_at = excluded.validated_at,
                validation_error = excluded.validation_error,
                api_key = excluded.api_key,
                api_secret = excluded.api_secret,
                api_endpoint = excluded.api_endpoint,
                workspace_id = excluded.workspace_id,
                project_id = excluded.project_id,
                account_id = excluded.account_id,
                organization_id = excluded.organization_id,
                app_name = excluded.app_name,
                namespace_id = excluded.namespace_id,
                default_region = excluded.default_region,
                default_instance_type = excluded.default_instance_type,
                default_image = excluded.default_image,
                default_cpu_cores = excluded.default_cpu_cores,
                default_memory_mb = excluded.default_memory_mb,
                default_disk_gb = excluded.default_disk_gb,
                default_gpu_type = excluded.default_gpu_type,
                cost_per_hour = excluded.cost_per_hour,
                cost_per_gb_memory = excluded.cost_per_gb_memory,
                cost_per_vcpu = excluded.cost_per_vcpu,
                cost_per_gpu_hour = excluded.cost_per_gpu_hour,
                max_sandboxes = excluded.max_sandboxes,
                max_runtime_hours = excluded.max_runtime_hours,
                max_total_cost = excluded.max_total_cost,
                custom_config = excluded.custom_config,
                updated_by = excluded.updated_by,
                updated_at = datetime('now', 'utc')
            "#
        )
        .bind(&settings.provider)
        .bind(settings.enabled as i64)
        .bind(settings.configured as i64)
        .bind(&settings.validated_at)
        .bind(&settings.validation_error)
        .bind(&encrypted_api_key)
        .bind(&encrypted_api_secret)
        .bind(&settings.api_endpoint)
        .bind(&settings.workspace_id)
        .bind(&settings.project_id)
        .bind(&settings.account_id)
        .bind(&settings.organization_id)
        .bind(&settings.app_name)
        .bind(&settings.namespace_id)
        .bind(&settings.default_region)
        .bind(&settings.default_instance_type)
        .bind(&settings.default_image)
        .bind(settings.default_cpu_cores)
        .bind(settings.default_memory_mb)
        .bind(settings.default_disk_gb)
        .bind(&settings.default_gpu_type)
        .bind(settings.cost_per_hour)
        .bind(settings.cost_per_gb_memory)
        .bind(settings.cost_per_vcpu)
        .bind(settings.cost_per_gpu_hour)
        .bind(settings.max_sandboxes)
        .bind(settings.max_runtime_hours)
        .bind(settings.max_total_cost)
        .bind(settings.custom_config.as_ref().and_then(|v| serde_json::to_string(v).ok()))
        .bind(updated_by)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    /// Delete provider settings
    pub async fn delete_provider_settings(&self, provider: &str) -> Result<(), StorageError> {
        debug!("Deleting provider settings for: {}", provider);

        sqlx::query("DELETE FROM sandbox_provider_settings WHERE provider = ?")
            .bind(provider)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    fn row_to_sandbox_settings(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<SandboxSettings, StorageError> {
        Ok(SandboxSettings {
            enabled: row.try_get::<i64, _>("enabled")? != 0,
            default_provider: row.try_get("default_provider")?,
            default_image: row.try_get("default_image")?,
            docker_username: row.try_get("docker_username")?,
            max_concurrent_local: row.try_get("max_concurrent_local")?,
            max_concurrent_cloud: row.try_get("max_concurrent_cloud")?,
            max_cpu_cores_per_sandbox: row.try_get("max_cpu_cores_per_sandbox")?,
            max_memory_gb_per_sandbox: row.try_get("max_memory_gb_per_sandbox")?,
            max_disk_gb_per_sandbox: row.try_get("max_disk_gb_per_sandbox")?,
            max_gpu_per_sandbox: row.try_get("max_gpu_per_sandbox")?,
            auto_stop_idle_minutes: row.try_get("auto_stop_idle_minutes")?,
            max_runtime_hours: row.try_get("max_runtime_hours")?,
            cleanup_interval_minutes: row.try_get("cleanup_interval_minutes")?,
            preserve_stopped_sandboxes: row.try_get::<i64, _>("preserve_stopped_sandboxes")? != 0,
            auto_restart_failed: row.try_get::<i64, _>("auto_restart_failed")? != 0,
            max_restart_attempts: row.try_get("max_restart_attempts")?,
            cost_tracking_enabled: row.try_get::<i64, _>("cost_tracking_enabled")? != 0,
            cost_alert_threshold: row.try_get("cost_alert_threshold")?,
            max_cost_per_sandbox: row.try_get("max_cost_per_sandbox")?,
            max_total_cost: row.try_get("max_total_cost")?,
            auto_stop_at_cost_limit: row.try_get::<i64, _>("auto_stop_at_cost_limit")? != 0,
            default_network_mode: row.try_get("default_network_mode")?,
            allow_public_endpoints: row.try_get::<i64, _>("allow_public_endpoints")? != 0,
            require_auth_for_web: row.try_get::<i64, _>("require_auth_for_web")? != 0,
            allow_privileged_containers: row.try_get::<i64, _>("allow_privileged_containers")? != 0,
            require_non_root_user: row.try_get::<i64, _>("require_non_root_user")? != 0,
            enable_security_scanning: row.try_get::<i64, _>("enable_security_scanning")? != 0,
            allowed_base_images: row
                .try_get::<Option<String>, _>("allowed_base_images")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            blocked_commands: row
                .try_get::<Option<String>, _>("blocked_commands")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            resource_monitoring_interval_seconds: row
                .try_get("resource_monitoring_interval_seconds")?,
            health_check_interval_seconds: row.try_get("health_check_interval_seconds")?,
            log_retention_days: row.try_get("log_retention_days")?,
            metrics_retention_days: row.try_get("metrics_retention_days")?,
            allow_custom_templates: row.try_get::<i64, _>("allow_custom_templates")? != 0,
            require_template_approval: row.try_get::<i64, _>("require_template_approval")? != 0,
            share_templates_globally: row.try_get::<i64, _>("share_templates_globally")? != 0,
            updated_at: row.try_get("updated_at")?,
            updated_by: row.try_get("updated_by")?,
        })
    }

    fn row_to_provider_settings(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<ProviderSettings, StorageError> {
        // Decrypt api_key if present
        let encrypted_api_key: Option<String> = row.try_get("api_key")?;
        let api_key = match encrypted_api_key {
            Some(encrypted) => Some(self.encryption.decrypt(&encrypted).map_err(|e| {
                error!("Failed to decrypt api_key: {}", e);
                StorageError::Encryption(format!("API key decryption failed: {}", e))
            })?),
            None => None,
        };

        // Decrypt api_secret if present
        let encrypted_api_secret: Option<String> = row.try_get("api_secret")?;
        let api_secret = match encrypted_api_secret {
            Some(encrypted) => Some(self.encryption.decrypt(&encrypted).map_err(|e| {
                error!("Failed to decrypt api_secret: {}", e);
                StorageError::Encryption(format!("API secret decryption failed: {}", e))
            })?),
            None => None,
        };

        Ok(ProviderSettings {
            provider: row.try_get("provider")?,
            enabled: row.try_get::<i64, _>("enabled")? != 0,
            configured: row.try_get::<i64, _>("configured")? != 0,
            validated_at: row.try_get("validated_at")?,
            validation_error: row.try_get("validation_error")?,
            api_key,
            api_secret,
            api_endpoint: row.try_get("api_endpoint")?,
            workspace_id: row.try_get("workspace_id")?,
            project_id: row.try_get("project_id")?,
            account_id: row.try_get("account_id")?,
            organization_id: row.try_get("organization_id")?,
            app_name: row.try_get("app_name")?,
            namespace_id: row.try_get("namespace_id")?,
            default_region: row.try_get("default_region")?,
            default_instance_type: row.try_get("default_instance_type")?,
            default_image: row.try_get("default_image")?,
            default_cpu_cores: row.try_get("default_cpu_cores")?,
            default_memory_mb: row.try_get("default_memory_mb")?,
            default_disk_gb: row.try_get("default_disk_gb")?,
            default_gpu_type: row.try_get("default_gpu_type")?,
            cost_per_hour: row.try_get("cost_per_hour")?,
            cost_per_gb_memory: row.try_get("cost_per_gb_memory")?,
            cost_per_vcpu: row.try_get("cost_per_vcpu")?,
            cost_per_gpu_hour: row.try_get("cost_per_gpu_hour")?,
            max_sandboxes: row.try_get("max_sandboxes")?,
            max_runtime_hours: row.try_get("max_runtime_hours")?,
            max_total_cost: row.try_get("max_total_cost")?,
            custom_config: row
                .try_get::<Option<String>, _>("custom_config")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            updated_at: row.try_get("updated_at")?,
            updated_by: row.try_get("updated_by")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("../storage/migrations")
            .run(&pool)
            .await
            .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_get_sandbox_settings() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool).unwrap();

        let settings = manager.get_sandbox_settings().await.unwrap();
        assert_eq!(settings.default_provider, "local");
        assert!(settings.enabled);
    }

    #[tokio::test]
    async fn test_update_sandbox_settings() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool).unwrap();

        let mut settings = manager.get_sandbox_settings().await.unwrap();
        settings.max_concurrent_local = 20;
        settings.default_provider = "beam".to_string();

        manager
            .update_sandbox_settings(&settings, Some("test-user"))
            .await
            .unwrap();

        let updated = manager.get_sandbox_settings().await.unwrap();
        assert_eq!(updated.max_concurrent_local, 20);
        assert_eq!(updated.default_provider, "beam");
    }

    #[tokio::test]
    async fn test_get_provider_settings() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool).unwrap();

        let settings = manager.get_provider_settings("local").await.unwrap();
        assert_eq!(settings.provider, "local");
        assert!(settings.enabled);
    }

    #[tokio::test]
    async fn test_list_provider_settings() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool).unwrap();

        let settings = manager.list_provider_settings().await.unwrap();
        assert_eq!(settings.len(), 8); // 8 providers in seed data
    }

    #[tokio::test]
    async fn test_update_provider_settings() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool).unwrap();

        let mut settings = manager.get_provider_settings("beam").await.unwrap();
        settings.enabled = true;
        settings.configured = true;
        settings.api_key =
            Some("test-api-key-1234567890123456789012345678901234567890".to_string());

        manager
            .update_provider_settings(&settings, Some("test-user"))
            .await
            .unwrap();

        let updated = manager.get_provider_settings("beam").await.unwrap();
        assert!(updated.enabled);
        assert!(updated.configured);
        assert_eq!(
            updated.api_key,
            Some("test-api-key-1234567890123456789012345678901234567890".to_string())
        );
    }

    #[tokio::test]
    async fn test_provider_credentials_encryption_roundtrip() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool.clone()).unwrap();

        // Create test credentials
        let test_api_key = "my-secret-api-key-12345";
        let test_api_secret = "my-secret-api-secret-67890";

        // Update provider with credentials
        let mut settings = manager.get_provider_settings("beam").await.unwrap();
        settings.api_key = Some(test_api_key.to_string());
        settings.api_secret = Some(test_api_secret.to_string());
        settings.enabled = true;
        settings.configured = true;

        manager
            .update_provider_settings(&settings, Some("test-user"))
            .await
            .unwrap();

        // Read back and verify values match original
        let retrieved = manager.get_provider_settings("beam").await.unwrap();
        assert_eq!(retrieved.api_key, Some(test_api_key.to_string()));
        assert_eq!(retrieved.api_secret, Some(test_api_secret.to_string()));

        // Verify database contains encrypted values (not plaintext)
        let row = sqlx::query(
            "SELECT api_key, api_secret FROM sandbox_provider_settings WHERE provider = 'beam'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let encrypted_api_key: String = row.try_get("api_key").unwrap();
        let encrypted_api_secret: String = row.try_get("api_secret").unwrap();

        // Encrypted values should not equal plaintext
        assert_ne!(encrypted_api_key, test_api_key);
        assert_ne!(encrypted_api_secret, test_api_secret);

        // Encrypted values should be base64-encoded (longer than original)
        assert!(encrypted_api_key.len() > test_api_key.len());
        assert!(encrypted_api_secret.len() > test_api_secret.len());
    }

    #[tokio::test]
    async fn test_provider_credentials_none_handling() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool).unwrap();

        // Update provider with None credentials
        let mut settings = manager.get_provider_settings("e2b").await.unwrap();
        settings.api_key = None;
        settings.api_secret = None;

        manager
            .update_provider_settings(&settings, Some("test-user"))
            .await
            .unwrap();

        // Verify None values stay None
        let retrieved = manager.get_provider_settings("e2b").await.unwrap();
        assert_eq!(retrieved.api_key, None);
        assert_eq!(retrieved.api_secret, None);
    }

    #[tokio::test]
    async fn test_provider_credentials_update() {
        let pool = create_test_db().await;
        let manager = SettingsManager::new(pool.clone()).unwrap();

        // Insert initial encrypted credentials
        let mut settings = manager.get_provider_settings("modal").await.unwrap();
        settings.api_key = Some("initial-key-123".to_string());
        settings.api_secret = Some("initial-secret-456".to_string());

        manager
            .update_provider_settings(&settings, Some("test-user"))
            .await
            .unwrap();

        // Update with new credentials
        let mut updated_settings = manager.get_provider_settings("modal").await.unwrap();
        updated_settings.api_key = Some("updated-key-789".to_string());
        updated_settings.api_secret = Some("updated-secret-abc".to_string());

        manager
            .update_provider_settings(&updated_settings, Some("test-user"))
            .await
            .unwrap();

        // Verify old values are overwritten with new encrypted values
        let final_settings = manager.get_provider_settings("modal").await.unwrap();
        assert_eq!(final_settings.api_key, Some("updated-key-789".to_string()));
        assert_eq!(
            final_settings.api_secret,
            Some("updated-secret-abc".to_string())
        );

        // Verify database contains new encrypted values (not plaintext)
        let row = sqlx::query(
            "SELECT api_key, api_secret FROM sandbox_provider_settings WHERE provider = 'modal'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let encrypted_api_key: String = row.try_get("api_key").unwrap();
        let encrypted_api_secret: String = row.try_get("api_secret").unwrap();

        assert_ne!(encrypted_api_key, "updated-key-789");
        assert_ne!(encrypted_api_secret, "updated-secret-abc");
    }
}
