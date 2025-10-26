// ABOUTME: User storage layer using SQLite
// ABOUTME: Handles CRUD operations for users and their settings

use std::env;

use sqlx::{QueryBuilder, Row, SqlitePool};
use tracing::debug;

use super::types::{User, UserUpdateInput};
use crate::encryption::ApiKeyEncryption;
use storage::StorageError;

pub struct UserStorage {
    pool: SqlitePool,
    encryption: ApiKeyEncryption,
}

impl UserStorage {
    pub fn new(pool: SqlitePool) -> Result<Self, StorageError> {
        let encryption = ApiKeyEncryption::new().map_err(|e| {
            tracing::error!("Failed to initialize API key encryption: {}", e);
            tracing::error!("This is likely due to:");
            tracing::error!("  - Unable to get machine ID");
            tracing::error!("  - Unable to get hostname");
            tracing::error!("  - Cryptographic library initialization failure");
            StorageError::Encryption(format!("Failed to initialize encryption: {}", e))
        })?;
        Ok(Self { pool, encryption })
    }

    pub async fn get_user(&self, user_id: &str) -> Result<User, StorageError> {
        debug!("Fetching user: {}", user_id);

        let row = sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_user(&row)
    }

    pub async fn get_current_user(&self) -> Result<User, StorageError> {
        // For now, always return the default user
        // In future, this could be based on auth context
        self.get_user("default-user").await
    }

    pub async fn set_default_agent(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<(), StorageError> {
        debug!("Setting default agent: {} for user: {}", agent_id, user_id);

        // NOTE: Agent validation should be performed at the API/handler level
        // before calling this method. The security package doesn't have access
        // to the models registry to avoid circular dependencies.

        sqlx::query(
            r#"
            UPDATE users
            SET default_agent_id = ?, updated_at = datetime('now', 'utc')
            WHERE id = ?
            "#,
        )
        .bind(agent_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    pub async fn update_theme(&self, user_id: &str, theme: &str) -> Result<(), StorageError> {
        debug!("Updating theme for user: {}", user_id);

        sqlx::query(
            r#"
            UPDATE users
            SET theme = ?, updated_at = datetime('now', 'utc')
            WHERE id = ?
            "#,
        )
        .bind(theme)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    pub async fn update_credentials(
        &self,
        user_id: &str,
        input: UserUpdateInput,
    ) -> Result<User, StorageError> {
        debug!("Updating credentials for user: {}", user_id);

        // SQL injection safety: Column names are hardcoded string literals (safe),
        // user values use push_bind() for parameterization (protected from injection).
        let mut query_builder =
            QueryBuilder::new("UPDATE users SET updated_at = datetime('now', 'utc')");
        let mut has_updates = false;

        if let Some(key) = &input.openai_api_key {
            let encrypted = self.encryption.encrypt(key).map_err(|e| {
                StorageError::Encryption(format!("Failed to encrypt OpenAI API key: {}", e))
            })?;
            query_builder.push(", openai_api_key = ");
            query_builder.push_bind(encrypted); // Parameterized - safe from SQL injection
            has_updates = true;
        }
        if let Some(key) = &input.anthropic_api_key {
            let encrypted = self.encryption.encrypt(key).map_err(|e| {
                StorageError::Encryption(format!("Failed to encrypt Anthropic API key: {}", e))
            })?;
            query_builder.push(", anthropic_api_key = ");
            query_builder.push_bind(encrypted); // Parameterized - safe from SQL injection
            has_updates = true;
        }
        if let Some(key) = &input.google_api_key {
            let encrypted = self.encryption.encrypt(key).map_err(|e| {
                StorageError::Encryption(format!("Failed to encrypt Google API key: {}", e))
            })?;
            query_builder.push(", google_api_key = ");
            query_builder.push_bind(encrypted); // Parameterized - safe from SQL injection
            has_updates = true;
        }
        if let Some(key) = &input.xai_api_key {
            let encrypted = self.encryption.encrypt(key).map_err(|e| {
                StorageError::Encryption(format!("Failed to encrypt xAI API key: {}", e))
            })?;
            query_builder.push(", xai_api_key = ");
            query_builder.push_bind(encrypted); // Parameterized - safe from SQL injection
            has_updates = true;
        }
        if let Some(enabled) = input.ai_gateway_enabled {
            query_builder.push(", ai_gateway_enabled = ");
            query_builder.push_bind(enabled); // Parameterized - safe from SQL injection
            has_updates = true;
        }
        if let Some(url) = &input.ai_gateway_url {
            query_builder.push(", ai_gateway_url = ");
            query_builder.push_bind(url); // Parameterized - safe from SQL injection
            has_updates = true;
        }
        if let Some(key) = &input.ai_gateway_key {
            let encrypted = self.encryption.encrypt(key).map_err(|e| {
                StorageError::Encryption(format!("Failed to encrypt AI gateway key: {}", e))
            })?;
            query_builder.push(", ai_gateway_key = ");
            query_builder.push_bind(encrypted); // Parameterized - safe from SQL injection
            has_updates = true;
        }

        if !has_updates {
            return self.get_user(user_id).await;
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(user_id); // Parameterized - safe from SQL injection

        let mut tx = self.pool.begin().await.map_err(StorageError::Sqlx)?;

        query_builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(StorageError::Sqlx)?;

        let row = sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(StorageError::Sqlx)?;

        let user = self.row_to_user(&row)?;

        tx.commit().await.map_err(StorageError::Sqlx)?;

        Ok(user)
    }

    pub async fn get_api_key(
        &self,
        user_id: &str,
        provider: &str,
    ) -> Result<Option<String>, StorageError> {
        debug!(
            "Getting API key for provider: {} user: {}",
            provider, user_id
        );

        let user = self.get_user(user_id).await?;

        let db_key = match provider {
            "openai" => user.openai_api_key,
            "anthropic" => user.anthropic_api_key,
            "google" => user.google_api_key,
            "xai" => user.xai_api_key,
            _ => None,
        };

        if db_key.is_some() {
            return Ok(db_key);
        }

        let env_var_name = match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "google" => "GOOGLE_API_KEY",
            "xai" => "XAI_API_KEY",
            _ => return Ok(None),
        };

        Ok(env::var(env_var_name).ok().filter(|k| !k.is_empty()))
    }

    /// Rotate encryption keys from old password to new password
    /// This decrypts all API keys with the old encryption, then re-encrypts with new encryption
    pub async fn rotate_encryption_keys(
        &self,
        user_id: &str,
        old_encryption: &ApiKeyEncryption,
        new_encryption: &ApiKeyEncryption,
    ) -> Result<(), StorageError> {
        debug!("Rotating encryption keys for user: {}", user_id);

        // Start transaction
        let mut tx = self.pool.begin().await.map_err(StorageError::Sqlx)?;

        // Fetch encrypted keys from database
        let row = sqlx::query("SELECT openai_api_key, anthropic_api_key, google_api_key, xai_api_key, ai_gateway_key FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(StorageError::Sqlx)?;

        // Helper to decrypt with old key and re-encrypt with new key
        let rotate_key = |encrypted_key: Option<String>| -> Result<Option<String>, StorageError> {
            match encrypted_key {
                Some(value) if !value.is_empty() && ApiKeyEncryption::is_encrypted(&value) => {
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

        // Commit transaction
        tx.commit().await.map_err(StorageError::Sqlx)?;

        debug!("Successfully rotated encryption keys for user: {}", user_id);
        Ok(())
    }

    /// Check if there are environment variable API keys that should be migrated to the database
    pub async fn check_env_key_migration(
        &self,
        user_id: &str,
    ) -> Result<Vec<String>, StorageError> {
        let user = self.get_user(user_id).await?;
        let mut env_keys_to_migrate = Vec::new();

        // Check each provider
        let providers = [
            ("anthropic", "ANTHROPIC_API_KEY", &user.anthropic_api_key),
            ("openai", "OPENAI_API_KEY", &user.openai_api_key),
            ("google", "GOOGLE_API_KEY", &user.google_api_key),
            ("xai", "XAI_API_KEY", &user.xai_api_key),
        ];

        for (provider_name, env_var_name, db_key) in providers {
            // Check if env var exists and has a non-empty value
            if let Ok(env_value) = env::var(env_var_name) {
                if !env_value.is_empty() && db_key.is_none() {
                    env_keys_to_migrate.push(provider_name.to_string());
                }
            }
        }

        Ok(env_keys_to_migrate)
    }

    fn row_to_user(&self, row: &sqlx::sqlite::SqliteRow) -> Result<User, StorageError> {
        // Helper to decrypt API keys with migration support
        let decrypt_key = |key: Option<String>| -> Result<Option<String>, StorageError> {
            match key {
                Some(value) if !value.is_empty() => {
                    if ApiKeyEncryption::is_encrypted(&value) {
                        // Encrypted key - decrypt it
                        self.encryption.decrypt(&value).map(Some).map_err(|e| {
                            StorageError::Encryption(format!("Failed to decrypt API key: {}", e))
                        })
                    } else {
                        // Plaintext key - return as-is for backward compatibility
                        Ok(Some(value))
                    }
                }
                _ => Ok(None),
            }
        };

        Ok(User {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            avatar_url: row.try_get("avatar_url")?,
            default_agent_id: row.try_get("default_agent_id")?,
            theme: row.try_get("theme")?,
            openai_api_key: decrypt_key(row.try_get("openai_api_key")?)?,
            anthropic_api_key: decrypt_key(row.try_get("anthropic_api_key")?)?,
            google_api_key: decrypt_key(row.try_get("google_api_key")?)?,
            xai_api_key: decrypt_key(row.try_get("xai_api_key")?)?,
            ai_gateway_enabled: row
                .try_get::<Option<i32>, _>("ai_gateway_enabled")?
                .map(|v| v != 0)
                .unwrap_or(false),
            ai_gateway_url: row.try_get("ai_gateway_url")?,
            ai_gateway_key: decrypt_key(row.try_get("ai_gateway_key")?)?,
            preferences: row
                .try_get::<Option<String>, _>("preferences")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            last_login_at: row.try_get("last_login_at")?,
        })
    }
}
