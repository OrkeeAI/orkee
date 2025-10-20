// ABOUTME: User storage layer using SQLite
// ABOUTME: Handles CRUD operations for users and their settings

use sqlx::{Row, SqlitePool};
use std::env;
use tracing::debug;

use super::types::{User, UserUpdateInput};
use crate::storage::StorageError;

pub struct UserStorage {
    pool: SqlitePool,
}

impl UserStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
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

        let mut query = String::from("UPDATE users SET updated_at = datetime('now', 'utc')");
        let mut has_updates = false;

        if input.openai_api_key.is_some() {
            query.push_str(", openai_api_key = ?");
            has_updates = true;
        }
        if input.anthropic_api_key.is_some() {
            query.push_str(", anthropic_api_key = ?");
            has_updates = true;
        }
        if input.google_api_key.is_some() {
            query.push_str(", google_api_key = ?");
            has_updates = true;
        }
        if input.xai_api_key.is_some() {
            query.push_str(", xai_api_key = ?");
            has_updates = true;
        }
        if input.ai_gateway_enabled.is_some() {
            query.push_str(", ai_gateway_enabled = ?");
            has_updates = true;
        }
        if input.ai_gateway_url.is_some() {
            query.push_str(", ai_gateway_url = ?");
            has_updates = true;
        }
        if input.ai_gateway_key.is_some() {
            query.push_str(", ai_gateway_key = ?");
            has_updates = true;
        }

        query.push_str(" WHERE id = ?");

        if !has_updates {
            return self.get_user(user_id).await;
        }

        let mut q = sqlx::query(&query);

        if let Some(key) = &input.openai_api_key {
            q = q.bind(key);
        }
        if let Some(key) = &input.anthropic_api_key {
            q = q.bind(key);
        }
        if let Some(key) = &input.google_api_key {
            q = q.bind(key);
        }
        if let Some(key) = &input.xai_api_key {
            q = q.bind(key);
        }
        if let Some(enabled) = input.ai_gateway_enabled {
            q = q.bind(enabled);
        }
        if let Some(url) = &input.ai_gateway_url {
            q = q.bind(url);
        }
        if let Some(key) = &input.ai_gateway_key {
            q = q.bind(key);
        }

        q = q.bind(user_id);

        q.execute(&self.pool).await.map_err(StorageError::Sqlx)?;

        self.get_user(user_id).await
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
        Ok(User {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            avatar_url: row.try_get("avatar_url")?,
            default_agent_id: row.try_get("default_agent_id")?,
            theme: row.try_get("theme")?,
            openai_api_key: row.try_get("openai_api_key")?,
            anthropic_api_key: row.try_get("anthropic_api_key")?,
            google_api_key: row.try_get("google_api_key")?,
            xai_api_key: row.try_get("xai_api_key")?,
            ai_gateway_enabled: row
                .try_get::<Option<i32>, _>("ai_gateway_enabled")?
                .map(|v| v != 0)
                .unwrap_or(false),
            ai_gateway_url: row.try_get("ai_gateway_url")?,
            ai_gateway_key: row.try_get("ai_gateway_key")?,
            preferences: row
                .try_get::<Option<String>, _>("preferences")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            last_login_at: row.try_get("last_login_at")?,
        })
    }
}
