// ABOUTME: Roundtable manager for expert discussion sessions
// ABOUTME: CRUD operations for roundtables, experts, messages, participants, and insights

use crate::error::{IdeateError, Result};
use crate::roundtable::*;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use tracing::{debug, info, warn};

/// Manager for roundtable operations
pub struct RoundtableManager {
    db: SqlitePool,
}

impl RoundtableManager {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // ========================================================================
    // EXPERT PERSONA OPERATIONS
    // ========================================================================

    /// Get all expert personas (default + custom)
    pub async fn list_experts(&self, include_default: bool) -> Result<Vec<ExpertPersona>> {
        let query = if include_default {
            "SELECT id, name, role, expertise, system_prompt, bio, is_default, created_at
             FROM expert_personas
             ORDER BY is_default DESC, name ASC"
        } else {
            "SELECT id, name, role, expertise, system_prompt, bio, is_default, created_at
             FROM expert_personas
             WHERE is_default = 0
             ORDER BY name ASC"
        };

        let rows = sqlx::query(query)
            .fetch_all(&self.db)
            .await
            .map_err(|e| IdeateError::Database(e.to_string()))?;

        let experts = rows
            .into_iter()
            .map(|row| {
                let expertise_json: String = row.get("expertise");
                let expertise: Vec<String> = serde_json::from_str(&expertise_json)
                    .unwrap_or_default();

                ExpertPersona {
                    id: row.get("id"),
                    name: row.get("name"),
                    role: row.get("role"),
                    expertise,
                    system_prompt: row.get("system_prompt"),
                    bio: row.get("bio"),
                    is_default: row.get("is_default"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(experts)
    }

    /// Get a specific expert persona by ID
    pub async fn get_expert(&self, expert_id: &str) -> Result<ExpertPersona> {
        let row = sqlx::query(
            "SELECT id, name, role, expertise, system_prompt, bio, is_default, created_at
             FROM expert_personas
             WHERE id = ?",
        )
        .bind(expert_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?
        .ok_or_else(|| IdeateError::NotFound(format!("Expert not found: {}", expert_id)))?;

        let expertise_json: String = row.get("expertise");
        let expertise: Vec<String> = serde_json::from_str(&expertise_json)
            .unwrap_or_default();

        Ok(ExpertPersona {
            id: row.get("id"),
            name: row.get("name"),
            role: row.get("role"),
            expertise,
            system_prompt: row.get("system_prompt"),
            bio: row.get("bio"),
            is_default: row.get("is_default"),
            created_at: row.get("created_at"),
        })
    }

    /// Create a custom expert persona
    pub async fn create_expert(&self, input: CreateExpertPersonaInput) -> Result<ExpertPersona> {
        let id = format!("expert_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let expertise_json = serde_json::to_string(&input.expertise)
            .map_err(|e| IdeateError::Validation(e.to_string()))?;
        let created_at = Utc::now();

        sqlx::query(
            "INSERT INTO expert_personas (id, name, role, expertise, system_prompt, bio, is_default, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 0, ?)"
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.role)
        .bind(&expertise_json)
        .bind(&input.system_prompt)
        .bind(&input.bio)
        .bind(created_at)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        info!("Created custom expert persona: {}", id);

        Ok(ExpertPersona {
            id,
            name: input.name,
            role: input.role,
            expertise: input.expertise,
            system_prompt: input.system_prompt,
            bio: input.bio,
            is_default: false,
            created_at,
        })
    }

    /// Delete a custom expert persona (cannot delete defaults)
    pub async fn delete_expert(&self, expert_id: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM expert_personas WHERE id = ? AND is_default = 0"
        )
        .bind(expert_id)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IdeateError::NotFound(format!(
                "Expert not found or is a default expert: {}",
                expert_id
            )));
        }

        info!("Deleted custom expert: {}", expert_id);
        Ok(())
    }

    // ========================================================================
    // ROUNDTABLE SESSION OPERATIONS
    // ========================================================================

    /// Create a new roundtable session
    pub async fn create_roundtable(
        &self,
        session_id: &str,
        topic: String,
        num_experts: i32,
    ) -> Result<RoundtableSession> {
        if num_experts < 2 || num_experts > 5 {
            return Err(IdeateError::Validation(
                "Number of experts must be between 2 and 5".to_string(),
            ));
        }

        let id = format!("roundtable_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let created_at = Utc::now();

        sqlx::query(
            "INSERT INTO roundtable_sessions (id, session_id, status, topic, num_experts, created_at)
             VALUES (?, ?, 'setup', ?, ?, ?)"
        )
        .bind(&id)
        .bind(session_id)
        .bind(&topic)
        .bind(num_experts)
        .bind(created_at)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        info!("Created roundtable session: {} for ideate session: {}", id, session_id);

        Ok(RoundtableSession {
            id,
            session_id: session_id.to_string(),
            status: RoundtableStatus::Setup,
            topic,
            num_experts,
            moderator_persona: None,
            started_at: None,
            completed_at: None,
            created_at,
        })
    }

    /// Get roundtable session by ID
    pub async fn get_roundtable(&self, roundtable_id: &str) -> Result<RoundtableSession> {
        let row = sqlx::query(
            "SELECT id, session_id, status, topic, num_experts, moderator_persona,
                    started_at, completed_at, created_at
             FROM roundtable_sessions
             WHERE id = ?"
        )
        .bind(roundtable_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?
        .ok_or_else(|| IdeateError::NotFound(format!("Roundtable not found: {}", roundtable_id)))?;

        Ok(self.row_to_roundtable_session(row))
    }

    /// Get roundtable sessions for an ideate session
    pub async fn list_roundtables_for_session(&self, session_id: &str) -> Result<Vec<RoundtableSession>> {
        let rows = sqlx::query(
            "SELECT id, session_id, status, topic, num_experts, moderator_persona,
                    started_at, completed_at, created_at
             FROM roundtable_sessions
             WHERE session_id = ?
             ORDER BY created_at DESC"
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let roundtables = rows
            .into_iter()
            .map(|row| self.row_to_roundtable_session(row))
            .collect();

        Ok(roundtables)
    }

    /// Start a roundtable discussion (change status from setup to discussing)
    pub async fn start_roundtable(&self, roundtable_id: &str) -> Result<()> {
        let started_at = Utc::now();

        let result = sqlx::query(
            "UPDATE roundtable_sessions
             SET status = 'discussing', started_at = ?
             WHERE id = ? AND status = 'setup'"
        )
        .bind(started_at)
        .bind(roundtable_id)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IdeateError::Validation(format!(
                "Cannot start roundtable: {} (may not be in setup status)",
                roundtable_id
            )));
        }

        info!("Started roundtable discussion: {}", roundtable_id);
        Ok(())
    }

    /// Complete a roundtable discussion
    pub async fn complete_roundtable(&self, roundtable_id: &str) -> Result<()> {
        let completed_at = Utc::now();

        let result = sqlx::query(
            "UPDATE roundtable_sessions
             SET status = 'completed', completed_at = ?
             WHERE id = ? AND status = 'discussing'"
        )
        .bind(completed_at)
        .bind(roundtable_id)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IdeateError::Validation(format!(
                "Cannot complete roundtable: {} (may not be in discussing status)",
                roundtable_id
            )));
        }

        info!("Completed roundtable discussion: {}", roundtable_id);
        Ok(())
    }

    /// Cancel a roundtable discussion
    pub async fn cancel_roundtable(&self, roundtable_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE roundtable_sessions
             SET status = 'cancelled'
             WHERE id = ?"
        )
        .bind(roundtable_id)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        info!("Cancelled roundtable discussion: {}", roundtable_id);
        Ok(())
    }

    // ========================================================================
    // PARTICIPANT OPERATIONS
    // ========================================================================

    /// Add experts to a roundtable
    pub async fn add_participants(
        &self,
        roundtable_id: &str,
        expert_ids: Vec<String>,
    ) -> Result<Vec<RoundtableParticipant>> {
        let mut participants = Vec::new();

        for expert_id in expert_ids {
            let id = format!("participant_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
            let joined_at = Utc::now();

            sqlx::query(
                "INSERT INTO roundtable_participants (id, roundtable_id, expert_id, joined_at)
                 VALUES (?, ?, ?, ?)"
            )
            .bind(&id)
            .bind(roundtable_id)
            .bind(&expert_id)
            .bind(joined_at)
            .execute(&self.db)
            .await
            .map_err(|e| IdeateError::Database(e.to_string()))?;

            participants.push(RoundtableParticipant {
                id,
                roundtable_id: roundtable_id.to_string(),
                expert_id,
                joined_at,
            });
        }

        info!("Added {} participants to roundtable: {}", participants.len(), roundtable_id);
        Ok(participants)
    }

    /// Get participants for a roundtable with full expert details
    pub async fn get_participants(&self, roundtable_id: &str) -> Result<Vec<ExpertPersona>> {
        let rows = sqlx::query(
            "SELECT e.id, e.name, e.role, e.expertise, e.system_prompt, e.bio, e.is_default, e.created_at
             FROM expert_personas e
             JOIN roundtable_participants p ON e.id = p.expert_id
             WHERE p.roundtable_id = ?
             ORDER BY p.joined_at ASC"
        )
        .bind(roundtable_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let experts = rows
            .into_iter()
            .map(|row| {
                let expertise_json: String = row.get("expertise");
                let expertise: Vec<String> = serde_json::from_str(&expertise_json)
                    .unwrap_or_default();

                ExpertPersona {
                    id: row.get("id"),
                    name: row.get("name"),
                    role: row.get("role"),
                    expertise,
                    system_prompt: row.get("system_prompt"),
                    bio: row.get("bio"),
                    is_default: row.get("is_default"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(experts)
    }

    /// Get roundtable with full participant details
    pub async fn get_roundtable_with_participants(
        &self,
        roundtable_id: &str,
    ) -> Result<RoundtableWithParticipants> {
        let session = self.get_roundtable(roundtable_id).await?;
        let participants = self.get_participants(roundtable_id).await?;

        Ok(RoundtableWithParticipants {
            session,
            participants,
        })
    }

    // ========================================================================
    // MESSAGE OPERATIONS
    // ========================================================================

    /// Add a message to the roundtable
    pub async fn add_message(
        &self,
        roundtable_id: &str,
        role: MessageRole,
        expert_id: Option<String>,
        expert_name: Option<String>,
        content: String,
        metadata: Option<MessageMetadata>,
    ) -> Result<RoundtableMessage> {
        let id = format!("message_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let created_at = Utc::now();

        // Get next message order
        let message_order: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(message_order), 0) + 1
             FROM roundtable_messages
             WHERE roundtable_id = ?"
        )
        .bind(roundtable_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let metadata_json = metadata
            .map(|m| serde_json::to_string(&m).ok())
            .flatten();

        sqlx::query(
            "INSERT INTO roundtable_messages
             (id, roundtable_id, message_order, role, expert_id, expert_name, content, metadata, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(roundtable_id)
        .bind(message_order)
        .bind(role as i32) // sqlx should handle enum conversion
        .bind(&expert_id)
        .bind(&expert_name)
        .bind(&content)
        .bind(&metadata_json)
        .bind(created_at)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        debug!("Added message {} to roundtable: {}", id, roundtable_id);

        Ok(RoundtableMessage {
            id,
            roundtable_id: roundtable_id.to_string(),
            message_order,
            role,
            expert_id,
            expert_name,
            content,
            metadata: metadata_json,
            created_at,
        })
    }

    /// Get all messages for a roundtable in chronological order
    pub async fn get_messages(&self, roundtable_id: &str) -> Result<Vec<RoundtableMessage>> {
        let rows = sqlx::query(
            "SELECT id, roundtable_id, message_order, role, expert_id, expert_name, content, metadata, created_at
             FROM roundtable_messages
             WHERE roundtable_id = ?
             ORDER BY message_order ASC"
        )
        .bind(roundtable_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let messages = rows
            .into_iter()
            .map(|row| self.row_to_message(row))
            .collect();

        Ok(messages)
    }

    /// Get messages after a specific order number (for streaming)
    pub async fn get_messages_after(
        &self,
        roundtable_id: &str,
        after_order: i32,
    ) -> Result<Vec<RoundtableMessage>> {
        let rows = sqlx::query(
            "SELECT id, roundtable_id, message_order, role, expert_id, expert_name, content, metadata, created_at
             FROM roundtable_messages
             WHERE roundtable_id = ? AND message_order > ?
             ORDER BY message_order ASC"
        )
        .bind(roundtable_id)
        .bind(after_order)
        .fetch_all(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let messages = rows
            .into_iter()
            .map(|row| self.row_to_message(row))
            .collect();

        Ok(messages)
    }

    // ========================================================================
    // INSIGHT OPERATIONS
    // ========================================================================

    /// Store an extracted insight
    pub async fn add_insight(
        &self,
        roundtable_id: &str,
        insight_text: String,
        category: String,
        priority: InsightPriority,
        source_experts: Vec<String>,
        source_message_ids: Option<Vec<String>>,
    ) -> Result<RoundtableInsight> {
        let id = format!("insight_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let created_at = Utc::now();

        let source_experts_json = serde_json::to_string(&source_experts)
            .map_err(|e| IdeateError::Validation(e.to_string()))?;

        let source_message_ids_json = source_message_ids
            .map(|ids| serde_json::to_string(&ids).ok())
            .flatten();

        sqlx::query(
            "INSERT INTO roundtable_insights
             (id, roundtable_id, insight_text, category, priority, source_experts, source_message_ids, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(roundtable_id)
        .bind(&insight_text)
        .bind(&category)
        .bind(priority as i32)
        .bind(&source_experts_json)
        .bind(&source_message_ids_json)
        .bind(created_at)
        .execute(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        debug!("Added insight {} to roundtable: {}", id, roundtable_id);

        Ok(RoundtableInsight {
            id,
            roundtable_id: roundtable_id.to_string(),
            insight_text,
            category,
            priority,
            source_experts,
            source_message_ids,
            created_at,
        })
    }

    /// Get all insights for a roundtable
    pub async fn get_insights(&self, roundtable_id: &str) -> Result<Vec<RoundtableInsight>> {
        let rows = sqlx::query(
            "SELECT id, roundtable_id, insight_text, category, priority, source_experts, source_message_ids, created_at
             FROM roundtable_insights
             WHERE roundtable_id = ?
             ORDER BY priority DESC, created_at DESC"
        )
        .bind(roundtable_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let insights = rows
            .into_iter()
            .map(|row| self.row_to_insight(row))
            .collect();

        Ok(insights)
    }

    /// Get insights grouped by category
    pub async fn get_insights_by_category(
        &self,
        roundtable_id: &str,
    ) -> Result<Vec<InsightsByCategory>> {
        let insights = self.get_insights(roundtable_id).await?;

        let mut grouped: std::collections::HashMap<String, Vec<RoundtableInsight>> =
            std::collections::HashMap::new();

        for insight in insights {
            grouped
                .entry(insight.category.clone())
                .or_insert_with(Vec::new)
                .push(insight);
        }

        let result = grouped
            .into_iter()
            .map(|(category, insights)| InsightsByCategory { category, insights })
            .collect();

        Ok(result)
    }

    // ========================================================================
    // STATISTICS
    // ========================================================================

    /// Get statistics for a roundtable
    pub async fn get_statistics(&self, roundtable_id: &str) -> Result<RoundtableStatistics> {
        let session = self.get_roundtable(roundtable_id).await?;

        let message_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roundtable_messages WHERE roundtable_id = ?"
        )
        .bind(roundtable_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let expert_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roundtable_participants WHERE roundtable_id = ?"
        )
        .bind(roundtable_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let user_interjection_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roundtable_messages WHERE roundtable_id = ? AND role = 'user'"
        )
        .bind(roundtable_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        let insight_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roundtable_insights WHERE roundtable_id = ?"
        )
        .bind(roundtable_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| IdeateError::Database(e.to_string()))?;

        Ok(RoundtableStatistics {
            roundtable_id: roundtable_id.to_string(),
            message_count,
            expert_count,
            user_interjection_count,
            insight_count,
            duration_seconds: session.duration_seconds(),
            started_at: session.started_at,
            completed_at: session.completed_at,
        })
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    fn row_to_roundtable_session(&self, row: sqlx::sqlite::SqliteRow) -> RoundtableSession {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "setup" => RoundtableStatus::Setup,
            "discussing" => RoundtableStatus::Discussing,
            "completed" => RoundtableStatus::Completed,
            "cancelled" => RoundtableStatus::Cancelled,
            _ => RoundtableStatus::Setup,
        };

        RoundtableSession {
            id: row.get("id"),
            session_id: row.get("session_id"),
            status,
            topic: row.get("topic"),
            num_experts: row.get("num_experts"),
            moderator_persona: row.get("moderator_persona"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_message(&self, row: sqlx::sqlite::SqliteRow) -> RoundtableMessage {
        let role_str: String = row.get("role");
        let role = match role_str.as_str() {
            "expert" => MessageRole::Expert,
            "user" => MessageRole::User,
            "moderator" => MessageRole::Moderator,
            "system" => MessageRole::System,
            _ => MessageRole::System,
        };

        RoundtableMessage {
            id: row.get("id"),
            roundtable_id: row.get("roundtable_id"),
            message_order: row.get("message_order"),
            role,
            expert_id: row.get("expert_id"),
            expert_name: row.get("expert_name"),
            content: row.get("content"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_insight(&self, row: sqlx::sqlite::SqliteRow) -> RoundtableInsight {
        let priority_str: String = row.get("priority");
        let priority = match priority_str.as_str() {
            "low" => InsightPriority::Low,
            "medium" => InsightPriority::Medium,
            "high" => InsightPriority::High,
            "critical" => InsightPriority::Critical,
            _ => InsightPriority::Medium,
        };

        let source_experts_json: String = row.get("source_experts");
        let source_experts: Vec<String> = serde_json::from_str(&source_experts_json)
            .unwrap_or_default();

        let source_message_ids: Option<Vec<String>> = row
            .get::<Option<String>, _>("source_message_ids")
            .and_then(|json| serde_json::from_str(&json).ok());

        RoundtableInsight {
            id: row.get("id"),
            roundtable_id: row.get("roundtable_id"),
            insight_text: row.get("insight_text"),
            category: row.get("category"),
            priority,
            source_experts,
            source_message_ids,
            created_at: row.get("created_at"),
        }
    }
}
