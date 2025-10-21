// ABOUTME: User storage layer using SQLite
// ABOUTME: Handles CRUD operations for users and their settings

use sqlx::{QueryBuilder, Row, SqlitePool};
use std::env;
use tracing::debug;

use super::types::{User, UserUpdateInput};
use crate::security::ApiKeyEncryption;
use crate::storage::StorageError;

pub struct UserStorage {
    pool: SqlitePool,
    encryption: ApiKeyEncryption,
}

impl UserStorage {
    pub fn new(pool: SqlitePool) -> Result<Self, StorageError> {
        let encryption = ApiKeyEncryption::new().map_err(|e| {
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

        // Using QueryBuilder with push_bind() for all values to prevent SQL injection.
        // Column names use push() (not parameterizable), values use push_bind() (parameterized).
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
