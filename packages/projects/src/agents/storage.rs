// ABOUTME: User-agent storage layer using SQLite
// ABOUTME: Handles CRUD operations for user-specific agent configurations

use sqlx::{Row, SqlitePool};
use tracing::debug;

use super::types::UserAgent;
use crate::models::REGISTRY;
use crate::storage::StorageError;

pub struct AgentStorage {
    pool: SqlitePool,
}

impl AgentStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_user_agents(&self, user_id: &str) -> Result<Vec<UserAgent>, StorageError> {
        let (user_agents, _) = self.list_user_agents_paginated(user_id, None, None).await?;
        Ok(user_agents)
    }

    pub async fn list_user_agents_paginated(
        &self,
        user_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<(Vec<UserAgent>, i64), StorageError> {
        debug!(
            "Fetching user agents for user: {} (limit: {:?}, offset: {:?})",
            user_id, limit, offset
        );

        // Get total count
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_agents WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        // Build query with optional pagination - no join needed, agents loaded from JSON
        let mut query = String::from(
            r#"
            SELECT * FROM user_agents
            WHERE user_id = ?
            ORDER BY is_active DESC, created_at DESC
            "#,
        );

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }
        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET {}", off));
        }

        let rows = sqlx::query(&query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let mut user_agents = Vec::new();
        for row in rows {
            user_agents.push(self.row_to_user_agent(&row)?);
        }

        Ok((user_agents, count))
    }

    pub async fn get_user_agent(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<UserAgent, StorageError> {
        debug!("Fetching user agent: {} for user: {}", agent_id, user_id);

        let row = sqlx::query(
            r#"
            SELECT * FROM user_agents
            WHERE user_id = ? AND agent_id = ?
            "#,
        )
        .bind(user_id)
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        self.row_to_user_agent(&row)
    }

    pub async fn activate_agent(&self, user_id: &str, agent_id: &str) -> Result<(), StorageError> {
        debug!("Activating agent: {} for user: {}", agent_id, user_id);

        // Validate agent exists in registry (replaces DB foreign key constraint)
        if !REGISTRY.agent_exists(agent_id) {
            return Err(StorageError::InvalidAgent(agent_id.to_string()));
        }

        sqlx::query(
            r#"
            UPDATE user_agents
            SET is_active = 1, updated_at = datetime('now', 'utc')
            WHERE user_id = ? AND agent_id = ?
            "#,
        )
        .bind(user_id)
        .bind(agent_id)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    pub async fn deactivate_agent(
        &self,
        user_id: &str,
        agent_id: &str,
    ) -> Result<(), StorageError> {
        debug!("Deactivating agent: {} for user: {}", agent_id, user_id);

        // Validate agent exists in registry (replaces DB foreign key constraint)
        if !REGISTRY.agent_exists(agent_id) {
            return Err(StorageError::InvalidAgent(agent_id.to_string()));
        }

        sqlx::query(
            r#"
            UPDATE user_agents
            SET is_active = 0, updated_at = datetime('now', 'utc')
            WHERE user_id = ? AND agent_id = ?
            "#,
        )
        .bind(user_id)
        .bind(agent_id)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    fn row_to_user_agent(&self, row: &sqlx::sqlite::SqliteRow) -> Result<UserAgent, StorageError> {
        let agent_id: String = row.try_get("agent_id")?;

        // Load agent from JSON registry
        let agent = REGISTRY.get_agent(&agent_id).cloned();

        Ok(UserAgent {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            agent_id,
            agent,
            preferred_model_id: row.try_get("preferred_model_id")?,
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            custom_name: row.try_get("custom_name")?,
            custom_system_prompt: row.try_get("custom_system_prompt")?,
            custom_temperature: row.try_get("custom_temperature")?,
            custom_max_tokens: row.try_get("custom_max_tokens")?,
            tasks_assigned: row.try_get("tasks_assigned")?,
            tasks_completed: row.try_get("tasks_completed")?,
            total_tokens_used: row.try_get("total_tokens_used")?,
            total_cost_cents: row.try_get("total_cost_cents")?,
            last_used_at: row.try_get("last_used_at")?,
            custom_settings: row
                .try_get::<Option<String>, _>("custom_settings")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
