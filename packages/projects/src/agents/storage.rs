// ABOUTME: Agent storage layer using SQLite
// ABOUTME: Handles CRUD operations for agents and user-agent configurations

use sqlx::{Row, SqlitePool};
use tracing::debug;

use super::types::{Agent, UserAgent};
use crate::storage::StorageError;

pub struct AgentStorage {
    pool: SqlitePool,
}

impl AgentStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_agents(&self) -> Result<Vec<Agent>, StorageError> {
        let (agents, _) = self.list_agents_paginated(None, None).await?;
        Ok(agents)
    }

    pub async fn list_agents_paginated(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<(Vec<Agent>, i64), StorageError> {
        debug!("Fetching all agents (limit: {:?}, offset: {:?})", limit, offset);

        // Get total count
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        // Build query with optional pagination
        let mut query = String::from(
            r#"
            SELECT * FROM agents
            ORDER BY
                CASE type
                    WHEN 'system' THEN 0
                    WHEN 'ai' THEN 1
                    WHEN 'human' THEN 2
                END,
                display_name
            "#
        );

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }
        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET {}", off));
        }

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let mut agents = Vec::new();
        for row in rows {
            agents.push(self.row_to_agent(&row)?);
        }

        Ok((agents, count))
    }

    pub async fn get_agent(&self, agent_id: &str) -> Result<Agent, StorageError> {
        debug!("Fetching agent: {}", agent_id);

        let row = sqlx::query("SELECT * FROM agents WHERE id = ?")
            .bind(agent_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_agent(&row)
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
        debug!("Fetching user agents for user: {} (limit: {:?}, offset: {:?})", user_id, limit, offset);

        // Get total count
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_agents WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        // Build query with optional pagination
        let mut query = String::from(
            r#"
            SELECT
                ua.*,
                a.*
            FROM user_agents ua
            JOIN agents a ON ua.agent_id = a.id
            WHERE ua.user_id = ?
            ORDER BY ua.is_favorite DESC, a.display_name
            "#
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
            SELECT
                ua.*,
                a.*
            FROM user_agents ua
            JOIN agents a ON ua.agent_id = a.id
            WHERE ua.user_id = ? AND ua.agent_id = ?
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

    fn row_to_agent(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Agent, StorageError> {
        Ok(Agent {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            agent_type: row.try_get("type")?,
            provider: row.try_get("provider")?,
            model: row.try_get("model")?,
            display_name: row.try_get("display_name")?,
            avatar_url: row.try_get("avatar_url")?,
            description: row.try_get("description")?,
            capabilities: row
                .try_get::<Option<String>, _>("capabilities")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            languages: row
                .try_get::<Option<String>, _>("languages")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            frameworks: row.try_get("frameworks")?,
            max_context_tokens: row.try_get("max_context_tokens")?,
            supports_tools: row.try_get::<i64, _>("supports_tools")? != 0,
            supports_vision: row.try_get::<i64, _>("supports_vision")? != 0,
            supports_web_search: row.try_get::<i64, _>("supports_web_search")? != 0,
            api_endpoint: row.try_get("api_endpoint")?,
            temperature: row.try_get("temperature")?,
            max_tokens: row.try_get("max_tokens")?,
            system_prompt: row.try_get("system_prompt")?,
            cost_per_1k_input_tokens: row.try_get("cost_per_1k_input_tokens")?,
            cost_per_1k_output_tokens: row.try_get("cost_per_1k_output_tokens")?,
            is_available: row.try_get::<i64, _>("is_available")? != 0,
            requires_api_key: row.try_get::<i64, _>("requires_api_key")? != 0,
            metadata: row
                .try_get::<Option<String>, _>("metadata")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    fn row_to_user_agent(&self, row: &sqlx::sqlite::SqliteRow) -> Result<UserAgent, StorageError> {
        // Agent fields are prefixed with "a." in the query, but SQLite returns them without prefix
        let agent = Agent {
            id: row.try_get("agent_id")?,
            name: row.try_get("name")?,
            agent_type: row.try_get("type")?,
            provider: row.try_get("provider")?,
            model: row.try_get("model")?,
            display_name: row.try_get("display_name")?,
            avatar_url: row.try_get("avatar_url")?,
            description: row.try_get("description")?,
            capabilities: row
                .try_get::<Option<String>, _>("capabilities")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            languages: row
                .try_get::<Option<String>, _>("languages")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            frameworks: row.try_get("frameworks")?,
            max_context_tokens: row.try_get("max_context_tokens")?,
            supports_tools: row.try_get::<i64, _>("supports_tools")? != 0,
            supports_vision: row.try_get::<i64, _>("supports_vision")? != 0,
            supports_web_search: row.try_get::<i64, _>("supports_web_search")? != 0,
            api_endpoint: row.try_get("api_endpoint")?,
            temperature: row.try_get("temperature")?,
            max_tokens: row.try_get("max_tokens")?,
            system_prompt: row.try_get("system_prompt")?,
            cost_per_1k_input_tokens: row.try_get("cost_per_1k_input_tokens")?,
            cost_per_1k_output_tokens: row.try_get("cost_per_1k_output_tokens")?,
            is_available: row.try_get::<i64, _>("is_available")? != 0,
            requires_api_key: row.try_get::<i64, _>("requires_api_key")? != 0,
            metadata: row
                .try_get::<Option<String>, _>("metadata")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };

        Ok(UserAgent {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            agent_id: row.try_get("agent_id")?,
            agent: Some(agent),
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            is_favorite: row.try_get::<i64, _>("is_favorite")? != 0,
            custom_name: row.try_get("custom_name")?,
            custom_system_prompt: row.try_get("custom_system_prompt")?,
            custom_temperature: row.try_get("custom_temperature")?,
            custom_max_tokens: row.try_get("custom_max_tokens")?,
            tasks_assigned: row.try_get("tasks_assigned")?,
            tasks_completed: row.try_get("tasks_completed")?,
            total_tokens_used: row.try_get("total_tokens_used")?,
            total_cost_cents: row.try_get("total_cost_cents")?,
            last_used_at: row.try_get("last_used_at")?,
            preferences: row
                .try_get::<Option<String>, _>("preferences")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
