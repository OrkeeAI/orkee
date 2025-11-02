// ABOUTME: Conversational mode manager for chat-based PRD discovery
// ABOUTME: Handles message storage, retrieval, insight extraction, and quality tracking

use crate::conversational::*;
use crate::error::{IdeateError, Result};
use nanoid::nanoid;
use sqlx::{Row, SqlitePool};
use tracing::{error, info};

pub struct ConversationalManager {
    pool: SqlitePool,
}

impl ConversationalManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get conversation history for a session
    pub async fn get_history(&self, session_id: &str) -> Result<Vec<ConversationMessage>> {
        info!("Getting conversation history for session: {}", session_id);

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                session_id,
                prd_id,
                message_order,
                role,
                content,
                message_type,
                metadata,
                created_at
            FROM prd_conversations
            WHERE session_id = ?
            ORDER BY message_order ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get conversation history: {}", e);
            IdeateError::Database(e)
        })?;

        let messages: Vec<ConversationMessage> = rows
            .into_iter()
            .map(|row| ConversationMessage {
                id: row.get("id"),
                session_id: row.get("session_id"),
                prd_id: row.get("prd_id"),
                message_order: row.get("message_order"),
                role: serde_json::from_str(&row.get::<String, _>("role")).unwrap(),
                content: row.get("content"),
                message_type: row
                    .get::<Option<String>, _>("message_type")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(messages)
    }

    /// Add a message to the conversation
    pub async fn add_message(
        &self,
        session_id: &str,
        role: MessageRole,
        content: String,
        message_type: Option<MessageType>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ConversationMessage> {
        info!(
            "Adding message to session: {} (role: {:?})",
            session_id, role
        );

        let id = nanoid!(12);

        // Get next message order
        let message_order: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(message_order), -1) + 1 FROM prd_conversations WHERE session_id = ?"
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get next message order: {}", e);
            IdeateError::Database(e)
        })?;

        // Insert message
        let created_at = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO prd_conversations (
                id, session_id, message_order, role, content, message_type, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(session_id)
        .bind(message_order)
        .bind(&role)
        .bind(&content)
        .bind(&message_type)
        .bind(&metadata)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to insert message: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(ConversationMessage {
            id,
            session_id: session_id.to_string(),
            prd_id: None,
            message_order,
            role,
            content,
            message_type,
            metadata,
            created_at,
        })
    }

    /// Get discovery questions, optionally filtered by category
    pub async fn get_discovery_questions(
        &self,
        category: Option<QuestionCategory>,
    ) -> Result<Vec<DiscoveryQuestion>> {
        info!("Getting discovery questions (category: {:?})", category);

        let rows = if let Some(cat) = category {
            let cat_str = serde_json::to_string(&cat)
                .unwrap()
                .trim_matches('"')
                .to_string();
            sqlx::query(
                r#"
                SELECT
                    id,
                    category,
                    question_text,
                    follow_up_prompts,
                    context_keywords,
                    priority,
                    is_required,
                    display_order,
                    is_active,
                    created_at
                FROM discovery_questions
                WHERE category = ? AND is_active = TRUE
                ORDER BY priority DESC, display_order ASC
                "#,
            )
            .bind(cat_str)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT
                    id,
                    category,
                    question_text,
                    follow_up_prompts,
                    context_keywords,
                    priority,
                    is_required,
                    display_order,
                    is_active,
                    created_at
                FROM discovery_questions
                WHERE is_active = TRUE
                ORDER BY priority DESC, display_order ASC
                "#,
            )
            .fetch_all(&self.pool)
            .await
        };

        let rows = rows.map_err(|e| {
            error!("Failed to get discovery questions: {}", e);
            IdeateError::Database(e)
        })?;

        let questions: Vec<DiscoveryQuestion> = rows
            .into_iter()
            .map(|row| DiscoveryQuestion {
                id: row.get("id"),
                category: row.get("category"),
                question_text: row.get("question_text"),
                follow_up_prompts: row
                    .get::<Option<String>, _>("follow_up_prompts")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                context_keywords: row
                    .get::<Option<String>, _>("context_keywords")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                priority: row.get("priority"),
                is_required: row.get("is_required"),
                display_order: row.get("display_order"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(questions)
    }

    /// Get insights for a session
    pub async fn get_insights(&self, session_id: &str) -> Result<Vec<ConversationInsight>> {
        info!("Getting insights for session: {}", session_id);

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                session_id,
                insight_type,
                insight_text,
                confidence_score,
                source_message_ids,
                applied_to_prd,
                created_at
            FROM conversation_insights
            WHERE session_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get insights: {}", e);
            IdeateError::Database(e)
        })?;

        let insights: Vec<ConversationInsight> = rows
            .into_iter()
            .map(|row| ConversationInsight {
                id: row.get("id"),
                session_id: row.get("session_id"),
                insight_type: serde_json::from_str(&row.get::<String, _>("insight_type")).unwrap(),
                insight_text: row.get("insight_text"),
                confidence_score: row.get("confidence_score"),
                source_message_ids: row
                    .get::<Option<String>, _>("source_message_ids")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                applied_to_prd: row.get("applied_to_prd"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(insights)
    }

    /// Create a new insight
    pub async fn create_insight(
        &self,
        session_id: &str,
        input: CreateInsightInput,
    ) -> Result<ConversationInsight> {
        info!(
            "Creating insight for session: {} (type: {:?})",
            session_id, input.insight_type
        );

        let id = nanoid!(12);
        let created_at = chrono::Utc::now().to_rfc3339();

        let source_message_ids_json = input
            .source_message_ids
            .as_ref()
            .map(|ids| serde_json::to_value(ids).unwrap());

        sqlx::query(
            r#"
            INSERT INTO conversation_insights (
                id, session_id, insight_type, insight_text, confidence_score,
                source_message_ids, applied_to_prd, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, FALSE, ?)
            "#,
        )
        .bind(&id)
        .bind(session_id)
        .bind(&input.insight_type)
        .bind(&input.insight_text)
        .bind(input.confidence_score)
        .bind(&source_message_ids_json)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create insight: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(ConversationInsight {
            id,
            session_id: session_id.to_string(),
            insight_type: input.insight_type,
            insight_text: input.insight_text,
            confidence_score: input.confidence_score,
            source_message_ids: input.source_message_ids,
            applied_to_prd: false,
            created_at,
        })
    }

    /// Update discovery status for a session
    pub async fn update_discovery_status(
        &self,
        session_id: &str,
        status: DiscoveryStatus,
    ) -> Result<()> {
        info!(
            "Updating discovery status for session: {} to {:?}",
            session_id, status
        );

        sqlx::query(
            r#"
            UPDATE ideate_sessions
            SET discovery_status = ?
            WHERE id = ?
            "#,
        )
        .bind(&status)
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to update discovery status: {}", e);
            IdeateError::Database(e)
        })?;

        Ok(())
    }
}
