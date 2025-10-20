// ABOUTME: User storage layer using SQLite
// ABOUTME: Handles CRUD operations for users and their settings

use sqlx::{Row, SqlitePool};
use tracing::debug;

use super::types::User;
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
            preferences: row
                .try_get::<Option<String>, _>("preferences")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            last_login_at: row.try_get("last_login_at")?,
        })
    }
}
