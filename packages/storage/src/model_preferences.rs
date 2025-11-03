// ABOUTME: Model preferences type definitions and storage
// ABOUTME: Per-task AI model configuration for Ideate, PRD, and task features

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::StorageError;

/// Model configuration for a specific task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
}

/// User's model preferences for different task types
/// This is separate from agent conversations (user_agents.preferred_model_id)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelPreferences {
    pub user_id: String,

    // Chat (Ideate mode)
    pub chat_model: String,
    pub chat_provider: String,

    // PRD Generation
    pub prd_generation_model: String,
    pub prd_generation_provider: String,

    // PRD Analysis
    pub prd_analysis_model: String,
    pub prd_analysis_provider: String,

    // Insight Extraction
    pub insight_extraction_model: String,
    pub insight_extraction_provider: String,

    // Spec Generation
    pub spec_generation_model: String,
    pub spec_generation_provider: String,

    // Task Suggestions
    pub task_suggestions_model: String,
    pub task_suggestions_provider: String,

    // Task Analysis
    pub task_analysis_model: String,
    pub task_analysis_provider: String,

    // Spec Refinement
    pub spec_refinement_model: String,
    pub spec_refinement_provider: String,

    // Research Generation
    pub research_generation_model: String,
    pub research_generation_provider: String,

    // Markdown Generation
    pub markdown_generation_model: String,
    pub markdown_generation_provider: String,

    pub updated_at: String,
}

/// Request to update model preferences for a specific task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskModelRequest {
    pub model: String,
    pub provider: String,
}

/// Storage layer for model preferences
pub struct ModelPreferencesStorage {
    pool: SqlitePool,
}

impl ModelPreferencesStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get model preferences for a user
    /// Returns default preferences if not found
    pub async fn get_preferences(&self, user_id: &str) -> Result<ModelPreferences, StorageError> {
        let result = sqlx::query_as::<_, ModelPreferences>(
            "SELECT * FROM model_preferences WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        match result {
            Some(prefs) => Ok(prefs),
            None => {
                // Create default preferences if they don't exist
                self.create_default_preferences(user_id).await?;

                // Fetch again after creating (avoiding recursion)
                sqlx::query_as::<_, ModelPreferences>(
                    "SELECT * FROM model_preferences WHERE user_id = ?",
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(StorageError::Sqlx)
            }
        }
    }

    /// Create default preferences for a user
    async fn create_default_preferences(&self, user_id: &str) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO model_preferences (user_id)
            VALUES (?)
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    /// Update all model preferences for a user
    pub async fn update_preferences(&self, prefs: &ModelPreferences) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO model_preferences (
                user_id,
                chat_model, chat_provider,
                prd_generation_model, prd_generation_provider,
                prd_analysis_model, prd_analysis_provider,
                insight_extraction_model, insight_extraction_provider,
                spec_generation_model, spec_generation_provider,
                task_suggestions_model, task_suggestions_provider,
                task_analysis_model, task_analysis_provider,
                spec_refinement_model, spec_refinement_provider,
                research_generation_model, research_generation_provider,
                markdown_generation_model, markdown_generation_provider
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                chat_model = excluded.chat_model,
                chat_provider = excluded.chat_provider,
                prd_generation_model = excluded.prd_generation_model,
                prd_generation_provider = excluded.prd_generation_provider,
                prd_analysis_model = excluded.prd_analysis_model,
                prd_analysis_provider = excluded.prd_analysis_provider,
                insight_extraction_model = excluded.insight_extraction_model,
                insight_extraction_provider = excluded.insight_extraction_provider,
                spec_generation_model = excluded.spec_generation_model,
                spec_generation_provider = excluded.spec_generation_provider,
                task_suggestions_model = excluded.task_suggestions_model,
                task_suggestions_provider = excluded.task_suggestions_provider,
                task_analysis_model = excluded.task_analysis_model,
                task_analysis_provider = excluded.task_analysis_provider,
                spec_refinement_model = excluded.spec_refinement_model,
                spec_refinement_provider = excluded.spec_refinement_provider,
                research_generation_model = excluded.research_generation_model,
                research_generation_provider = excluded.research_generation_provider,
                markdown_generation_model = excluded.markdown_generation_model,
                markdown_generation_provider = excluded.markdown_generation_provider,
                updated_at = datetime('now', 'utc')
            "#,
        )
        .bind(&prefs.user_id)
        .bind(&prefs.chat_model)
        .bind(&prefs.chat_provider)
        .bind(&prefs.prd_generation_model)
        .bind(&prefs.prd_generation_provider)
        .bind(&prefs.prd_analysis_model)
        .bind(&prefs.prd_analysis_provider)
        .bind(&prefs.insight_extraction_model)
        .bind(&prefs.insight_extraction_provider)
        .bind(&prefs.spec_generation_model)
        .bind(&prefs.spec_generation_provider)
        .bind(&prefs.task_suggestions_model)
        .bind(&prefs.task_suggestions_provider)
        .bind(&prefs.task_analysis_model)
        .bind(&prefs.task_analysis_provider)
        .bind(&prefs.spec_refinement_model)
        .bind(&prefs.spec_refinement_provider)
        .bind(&prefs.research_generation_model)
        .bind(&prefs.research_generation_provider)
        .bind(&prefs.markdown_generation_model)
        .bind(&prefs.markdown_generation_provider)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    /// Update model preference for a specific task
    pub async fn update_task_model(
        &self,
        user_id: &str,
        task_type: &str,
        request: &UpdateTaskModelRequest,
    ) -> Result<(), StorageError> {
        // Ensure preferences exist
        self.get_preferences(user_id).await?;

        // Use match expression with pre-written UPDATE statements to prevent SQL injection
        match task_type {
            "chat" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET chat_model = ?, chat_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "prd_generation" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET prd_generation_model = ?, prd_generation_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "prd_analysis" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET prd_analysis_model = ?, prd_analysis_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "insight_extraction" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET insight_extraction_model = ?, insight_extraction_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "spec_generation" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET spec_generation_model = ?, spec_generation_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "task_suggestions" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET task_suggestions_model = ?, task_suggestions_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "task_analysis" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET task_analysis_model = ?, task_analysis_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "spec_refinement" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET spec_refinement_model = ?, spec_refinement_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "research_generation" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET research_generation_model = ?, research_generation_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            "markdown_generation" => {
                sqlx::query(
                    r#"
                    UPDATE model_preferences
                    SET markdown_generation_model = ?, markdown_generation_provider = ?, updated_at = datetime('now', 'utc')
                    WHERE user_id = ?
                    "#,
                )
                .bind(&request.model)
                .bind(&request.provider)
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
            }
            _ => {
                return Err(StorageError::InvalidInput(format!(
                    "Invalid task type: {}",
                    task_type
                )))
            }
        }

        Ok(())
    }
}
