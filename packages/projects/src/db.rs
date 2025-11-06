// ABOUTME: Database connection management and storage initialization
// ABOUTME: Provides shared access to SQLite pool and storage layers

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;
use tracing::{debug, info};

use orkee_agents::UserAgentStorage;
use orkee_ai::AiUsageLogStorage;
use orkee_executions::ExecutionStorage;
use orkee_sandbox::SettingsManager as SandboxSettingsManager;
use orkee_security::api_tokens::TokenStorage;
use orkee_security::UserStorage;
use orkee_settings::SettingsStorage;
use orkee_storage::model_preferences::ModelPreferencesStorage;
use orkee_storage::StorageError;
use orkee_tags::TagStorage;
use orkee_tasks::storage::TaskStorage;

/// Shared database state for API handlers
#[derive(Clone)]
pub struct DbState {
    pub pool: SqlitePool,
    pub task_storage: Arc<TaskStorage>,
    pub agent_storage: Arc<UserAgentStorage>,
    pub user_storage: Arc<UserStorage>,
    pub tag_storage: Arc<TagStorage>,
    pub execution_storage: Arc<ExecutionStorage>,
    pub ai_usage_log_storage: Arc<AiUsageLogStorage>,
    pub settings_storage: Arc<SettingsStorage>,
    pub token_storage: Arc<TokenStorage>,
    pub model_preferences_storage: Arc<ModelPreferencesStorage>,
    pub sandbox_settings: Arc<SandboxSettingsManager>,
    pub sandbox_manager: Arc<orkee_sandbox::SandboxManager>,
}

impl DbState {
    /// Create new database state from a SQLite pool
    pub fn new(pool: SqlitePool) -> Result<Self, StorageError> {
        let task_storage = Arc::new(TaskStorage::new(pool.clone()));
        let agent_storage = Arc::new(UserAgentStorage::new(pool.clone()));
        let user_storage = Arc::new(UserStorage::new(pool.clone())?);
        let tag_storage = Arc::new(TagStorage::new(pool.clone()));
        let execution_storage = Arc::new(ExecutionStorage::new(pool.clone()));
        let ai_usage_log_storage = Arc::new(AiUsageLogStorage::new(pool.clone()));
        let settings_storage = Arc::new(SettingsStorage::new(pool.clone()));
        let token_storage = Arc::new(TokenStorage::new(pool.clone()));
        let model_preferences_storage = Arc::new(ModelPreferencesStorage::new(pool.clone()));
        let sandbox_settings = Arc::new(SandboxSettingsManager::new(pool.clone()));

        // Initialize sandbox manager
        let sandbox_storage = Arc::new(orkee_sandbox::SandboxStorage::new(pool.clone()));
        let sandbox_manager = Arc::new(orkee_sandbox::SandboxManager::new(
            sandbox_storage,
            Arc::new(tokio::sync::RwLock::new(orkee_sandbox::SettingsManager::new(
                pool.clone(),
            ))),
        ));

        // Register Docker provider
        let docker_provider = Arc::new(
            orkee_sandbox::DockerProvider::new()
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to initialize Docker provider: {}", e);
                    // This is a temporary workaround - we create a Docker client that will fail
                    // gracefully when used. A better solution would be to have the provider
                    // initialization happen lazily or to have a null provider.
                    panic!("Docker provider initialization failed: {}", e)
                }),
        ) as Arc<dyn orkee_sandbox::SandboxProvider>;

        // Create a runtime handle for the async registration
        // Since we're in a sync context, we need to spawn this
        let manager_clone = sandbox_manager.clone();
        let provider_clone = docker_provider.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                manager_clone
                    .register_provider("local".to_string(), provider_clone)
                    .await;
            });
        });

        Ok(Self {
            pool,
            task_storage,
            agent_storage,
            user_storage,
            tag_storage,
            execution_storage,
            ai_usage_log_storage,
            settings_storage,
            token_storage,
            model_preferences_storage,
            sandbox_settings,
            sandbox_manager,
        })
    }

    /// Initialize database state with default configuration
    pub async fn init() -> Result<Self, StorageError> {
        Self::init_with_path(None).await
    }

    /// Initialize database state with optional custom database path
    pub async fn init_with_path(
        database_path: Option<std::path::PathBuf>,
    ) -> Result<Self, StorageError> {
        let database_path =
            database_path.unwrap_or_else(|| orkee_core::orkee_dir().join("orkee.db"));

        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).map_err(StorageError::Io)?;
        }

        let database_url = format!("sqlite:{}", database_path.display());

        debug!("Connecting to database: {}", database_url);

        // Configure connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&database_url)
            .await
            .map_err(StorageError::Sqlx)?;

        // Configure SQLite settings
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        info!("Database connection established");

        // Run migrations
        sqlx::migrate!("../storage/migrations")
            .run(&pool)
            .await
            .map_err(StorageError::Migration)?;

        debug!("Database migrations completed");

        Self::new(pool)
    }

    /// Atomically rotate encryption keys and update encryption settings
    /// This ensures both operations succeed or fail together
    pub async fn change_encryption_password_atomic(
        &self,
        user_id: &str,
        old_encryption: &orkee_security::encryption::ApiKeyEncryption,
        new_encryption: &orkee_security::encryption::ApiKeyEncryption,
        mode: orkee_security::encryption::EncryptionMode,
        new_salt: &[u8],
        new_hash: &[u8],
    ) -> Result<(), StorageError> {
        use sqlx::Row;
        use tracing::debug;

        debug!("Starting atomic password change for user: {}", user_id);

        // Start transaction
        let mut tx = self.pool.begin().await.map_err(StorageError::Sqlx)?;

        // Step 1: Rotate encryption keys (inline to share transaction)

        // Fetch encrypted keys from database
        let row = sqlx::query("SELECT openai_api_key, anthropic_api_key, google_api_key, xai_api_key, ai_gateway_key FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(StorageError::Sqlx)?;

        // Helper to decrypt with old key and re-encrypt with new key
        let rotate_key = |encrypted_key: Option<String>| -> Result<Option<String>, StorageError> {
            match encrypted_key {
                Some(value)
                    if !value.is_empty()
                        && orkee_security::encryption::ApiKeyEncryption::is_encrypted(&value) =>
                {
                    // Decrypt with old encryption
                    let plaintext = old_encryption.decrypt(&value).map_err(|e| {
                        StorageError::Encryption(format!(
                            "Failed to decrypt API key with old password: {}",
                            e
                        ))
                    })?;

                    // Re-encrypt with new encryption
                    let new_encrypted = new_encryption.encrypt(&plaintext).map_err(|e| {
                        StorageError::Encryption(format!(
                            "Failed to encrypt API key with new password: {}",
                            e
                        ))
                    })?;

                    Ok(Some(new_encrypted))
                }
                _ => Ok(None), // No key or plaintext key - skip rotation
            }
        };

        // Rotate all API keys
        let openai_key = rotate_key(row.try_get("openai_api_key")?)?;
        let anthropic_key = rotate_key(row.try_get("anthropic_api_key")?)?;
        let google_key = rotate_key(row.try_get("google_api_key")?)?;
        let xai_key = rotate_key(row.try_get("xai_api_key")?)?;
        let ai_gateway_key = rotate_key(row.try_get("ai_gateway_key")?)?;

        // Update database with re-encrypted keys
        sqlx::query(
            r#"
            UPDATE users
            SET openai_api_key = COALESCE(?, openai_api_key),
                anthropic_api_key = COALESCE(?, anthropic_api_key),
                google_api_key = COALESCE(?, google_api_key),
                xai_api_key = COALESCE(?, xai_api_key),
                ai_gateway_key = COALESCE(?, ai_gateway_key),
                updated_at = datetime('now', 'utc')
            WHERE id = ?
            "#,
        )
        .bind(openai_key)
        .bind(anthropic_key)
        .bind(google_key)
        .bind(xai_key)
        .bind(ai_gateway_key)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(StorageError::Sqlx)?;

        // Step 2: Update encryption settings (inline to share transaction)
        let mode_str = mode.to_string();

        sqlx::query(
            "INSERT OR REPLACE INTO encryption_settings (id, encryption_mode, password_salt, password_hash, updated_at)
             VALUES (1, ?, ?, ?, datetime('now'))"
        )
        .bind(&mode_str)
        .bind(new_salt)
        .bind(new_hash)
        .execute(&mut *tx)
        .await
        .map_err(StorageError::Sqlx)?;

        // Commit transaction - both operations succeed or fail together
        tx.commit().await.map_err(StorageError::Sqlx)?;

        debug!(
            "Successfully completed atomic password change for user: {}",
            user_id
        );
        Ok(())
    }
}
