// ABOUTME: Brainstorming session management with CRUD operations
// ABOUTME: Handles session lifecycle, section management, and status tracking

use crate::error::{IdeateError, Result};
use crate::types::*;
use chrono::Utc;
use sqlx::{Row, SqlitePool};

/// Manager for brainstorming sessions
pub struct BrainstormManager {
    db: SqlitePool,
}

impl BrainstormManager {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create a new brainstorming session
    pub async fn create_session(&self, input: CreateBrainstormSessionInput) -> Result<BrainstormSession> {
        let id = nanoid::nanoid!(8);
        let now = Utc::now();

        let session = sqlx::query(
            "INSERT INTO brainstorm_sessions (id, project_id, initial_description, mode, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, project_id, initial_description, mode, status, skipped_sections, created_at, updated_at"
        )
        .bind(&id)
        .bind(&input.project_id)
        .bind(&input.initial_description)
        .bind(&input.mode)
        .bind(BrainstormStatus::Draft)
        .bind(now)
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        Ok(BrainstormSession {
            id: session.get("id"),
            project_id: session.get("project_id"),
            initial_description: session.get("initial_description"),
            mode: session.get("mode"),
            status: session.get("status"),
            skipped_sections: session.get::<Option<String>, _>("skipped_sections")
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: session.get("created_at"),
            updated_at: session.get("updated_at"),
        })
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<BrainstormSession> {
        let session = sqlx::query(
            "SELECT id, project_id, initial_description, mode, status, skipped_sections, created_at, updated_at
             FROM brainstorm_sessions
             WHERE id = $1"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| IdeateError::SessionNotFound(session_id.to_string()))?;

        Ok(BrainstormSession {
            id: session.get("id"),
            project_id: session.get("project_id"),
            initial_description: session.get("initial_description"),
            mode: session.get("mode"),
            status: session.get("status"),
            skipped_sections: session.get::<Option<String>, _>("skipped_sections")
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: session.get("created_at"),
            updated_at: session.get("updated_at"),
        })
    }

    /// List sessions for a project
    pub async fn list_sessions(&self, project_id: &str) -> Result<Vec<BrainstormSession>> {
        let sessions = sqlx::query(
            "SELECT id, project_id, initial_description, mode, status, skipped_sections, created_at, updated_at
             FROM brainstorm_sessions
             WHERE project_id = $1
             ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&self.db)
        .await?;

        sessions.into_iter()
            .map(|row| {
                Ok(BrainstormSession {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    initial_description: row.get("initial_description"),
                    mode: row.get("mode"),
                    status: row.get("status"),
                    skipped_sections: row.get::<Option<String>, _>("skipped_sections")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
            })
            .collect()
    }

    /// Update a session
    pub async fn update_session(
        &self,
        session_id: &str,
        input: UpdateBrainstormSessionInput,
    ) -> Result<BrainstormSession> {
        let mut updates = Vec::new();
        let mut bind_count = 1;

        if input.initial_description.is_some() {
            updates.push(format!("initial_description = ${}", bind_count));
            bind_count += 1;
        }
        if input.mode.is_some() {
            updates.push(format!("mode = ${}", bind_count));
            bind_count += 1;
        }
        if input.status.is_some() {
            updates.push(format!("status = ${}", bind_count));
            bind_count += 1;
        }
        if input.skipped_sections.is_some() {
            updates.push(format!("skipped_sections = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            return self.get_session(session_id).await;
        }

        updates.push("updated_at = datetime('now', 'utc')".to_string());

        let query = format!(
            "UPDATE brainstorm_sessions SET {} WHERE id = ${}
             RETURNING id, project_id, initial_description, mode, status, skipped_sections, created_at, updated_at",
            updates.join(", "),
            bind_count
        );

        let mut q = sqlx::query(&query);

        if let Some(desc) = input.initial_description {
            q = q.bind(desc);
        }
        if let Some(mode) = input.mode {
            q = q.bind(mode);
        }
        if let Some(status) = input.status {
            q = q.bind(status);
        }
        if let Some(sections) = input.skipped_sections {
            let json = serde_json::to_string(&sections)?;
            q = q.bind(json);
        }

        q = q.bind(session_id);

        let session = q.fetch_one(&self.db).await?;

        Ok(BrainstormSession {
            id: session.get("id"),
            project_id: session.get("project_id"),
            initial_description: session.get("initial_description"),
            mode: session.get("mode"),
            status: session.get("status"),
            skipped_sections: session.get::<Option<String>, _>("skipped_sections")
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: session.get("created_at"),
            updated_at: session.get("updated_at"),
        })
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM brainstorm_sessions WHERE id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(IdeateError::SessionNotFound(session_id.to_string()));
        }

        Ok(())
    }

    /// Mark a section as skipped
    pub async fn skip_section(&self, session_id: &str, request: SkipSectionRequest) -> Result<()> {
        let session = self.get_session(session_id).await?;
        let mut skipped = session.skipped_sections.unwrap_or_default();

        if !skipped.contains(&request.section) {
            skipped.push(request.section.clone());
        }

        let json = serde_json::to_string(&skipped)?;
        sqlx::query("UPDATE brainstorm_sessions SET skipped_sections = $1, updated_at = datetime('now', 'utc') WHERE id = $2")
            .bind(json)
            .bind(session_id)
            .execute(&self.db)
            .await?;

        // TODO: If ai_fill is true, trigger AI to fill the section
        if request.ai_fill {
            tracing::info!("AI fill requested for section: {}", request.section);
            // This will be implemented in Phase 2-3
        }

        Ok(())
    }

    /// Get session completion status
    pub async fn get_completion_status(&self, session_id: &str) -> Result<SessionCompletionStatus> {
        let session = self.get_session(session_id).await?;

        // Check which sections have data
        let mut completed_count = 0;
        let total_sections = 8;

        // Check overview
        if self.has_overview(session_id).await? {
            completed_count += 1;
        }

        // Check features
        if self.has_features(session_id).await? {
            completed_count += 1;
        }

        // Check UX
        if self.has_ux(session_id).await? {
            completed_count += 1;
        }

        // Check technical
        if self.has_technical(session_id).await? {
            completed_count += 1;
        }

        // Check roadmap
        if self.has_roadmap(session_id).await? {
            completed_count += 1;
        }

        // Check dependencies
        if self.has_dependencies(session_id).await? {
            completed_count += 1;
        }

        // Check risks
        if self.has_risks(session_id).await? {
            completed_count += 1;
        }

        // Check research
        if self.has_research(session_id).await? {
            completed_count += 1;
        }

        let is_ready = match session.mode {
            BrainstormMode::Quick => true, // Quick mode is always ready
            BrainstormMode::Guided => completed_count >= 2, // At least 2 sections
            BrainstormMode::Comprehensive => completed_count >= 5, // At least 5 sections
        };

        Ok(SessionCompletionStatus {
            session_id: session_id.to_string(),
            total_sections,
            completed_sections: completed_count,
            skipped_sections: session.skipped_sections.unwrap_or_default(),
            is_ready_for_prd: is_ready,
            missing_required_sections: Vec::new(), // For now, no sections are strictly required
        })
    }

    // Helper methods to check if sections have data
    async fn has_overview(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_overview WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_features(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_features WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_ux(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_ux WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_technical(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_technical WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_roadmap(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_roadmap WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_dependencies(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_dependencies WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_risks(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_risks WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_research(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brainstorm_research WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }
}
