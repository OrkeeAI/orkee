// ABOUTME: Ideation session management with CRUD operations
// ABOUTME: Handles session lifecycle, section management, and status tracking

use crate::error::{IdeateError, Result};
use crate::types::*;
use chrono::Utc;
use sqlx::{Row, SqlitePool};

/// Manager for brainstorming sessions
pub struct IdeateManager {
    db: SqlitePool,
}

impl IdeateManager {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create a new brainstorming session
    pub async fn create_session(&self, input: CreateIdeateSessionInput) -> Result<IdeateSession> {
        let id = nanoid::nanoid!(8);
        let now = Utc::now();

        let session = sqlx::query(
            "INSERT INTO ideate_sessions (id, project_id, initial_description, mode, status, research_tools_enabled, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, project_id, initial_description, mode, status, skipped_sections, current_section, research_tools_enabled, generated_prd_id, created_at, updated_at"
        )
        .bind(&id)
        .bind(&input.project_id)
        .bind(&input.initial_description)
        .bind(input.mode)
        .bind(IdeateStatus::Draft)
        .bind(input.research_tools_enabled)
        .bind(now)
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        Ok(IdeateSession {
            id: session.get("id"),
            project_id: session.get("project_id"),
            initial_description: session.get("initial_description"),
            mode: session.get("mode"),
            status: session.get("status"),
            skipped_sections: session
                .get::<Option<String>, _>("skipped_sections")
                .and_then(|s| serde_json::from_str(&s).ok()),
            current_section: session.get("current_section"),
            research_tools_enabled: session.get::<i32, _>("research_tools_enabled") != 0,
            generated_prd_id: session.get("generated_prd_id"),

            // Phase 1 enhancement fields
            non_goals: session.get("non_goals"),
            open_questions: session.get("open_questions"),
            constraints_assumptions: session.get("constraints_assumptions"),
            success_metrics: session.get("success_metrics"),
            alternative_approaches: session
                .get::<Option<String>, _>("alternative_approaches")
                .and_then(|s| serde_json::from_str(&s).ok()),
            validation_checkpoints: session
                .get::<Option<String>, _>("validation_checkpoints")
                .and_then(|s| serde_json::from_str(&s).ok()),
            codebase_context: session
                .get::<Option<String>, _>("codebase_context")
                .and_then(|s| serde_json::from_str(&s).ok()),

            created_at: session.get("created_at"),
            updated_at: session.get("updated_at"),
        })
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<IdeateSession> {
        let session = sqlx::query(
            "SELECT id, project_id, initial_description, mode, status, skipped_sections, current_section, research_tools_enabled, generated_prd_id, created_at, updated_at
             FROM ideate_sessions
             WHERE id = $1"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| IdeateError::SessionNotFound(session_id.to_string()))?;

        Ok(IdeateSession {
            id: session.get("id"),
            project_id: session.get("project_id"),
            initial_description: session.get("initial_description"),
            mode: session.get("mode"),
            status: session.get("status"),
            skipped_sections: session
                .get::<Option<String>, _>("skipped_sections")
                .and_then(|s| serde_json::from_str(&s).ok()),
            current_section: session.get("current_section"),
            research_tools_enabled: session.get::<i32, _>("research_tools_enabled") != 0,
            generated_prd_id: session.get("generated_prd_id"),

            // Phase 1 enhancement fields
            non_goals: session.get("non_goals"),
            open_questions: session.get("open_questions"),
            constraints_assumptions: session.get("constraints_assumptions"),
            success_metrics: session.get("success_metrics"),
            alternative_approaches: session
                .get::<Option<String>, _>("alternative_approaches")
                .and_then(|s| serde_json::from_str(&s).ok()),
            validation_checkpoints: session
                .get::<Option<String>, _>("validation_checkpoints")
                .and_then(|s| serde_json::from_str(&s).ok()),
            codebase_context: session
                .get::<Option<String>, _>("codebase_context")
                .and_then(|s| serde_json::from_str(&s).ok()),

            created_at: session.get("created_at"),
            updated_at: session.get("updated_at"),
        })
    }

    /// List sessions for a project
    pub async fn list_sessions(&self, project_id: &str) -> Result<Vec<IdeateSession>> {
        let sessions = sqlx::query(
            "SELECT id, project_id, initial_description, mode, status, skipped_sections, current_section, research_tools_enabled, generated_prd_id, created_at, updated_at
             FROM ideate_sessions
             WHERE project_id = $1
             ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&self.db)
        .await?;

        sessions
            .into_iter()
            .map(|row| {
                Ok(IdeateSession {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    initial_description: row.get("initial_description"),
                    mode: row.get("mode"),
                    status: row.get("status"),
                    skipped_sections: row
                        .get::<Option<String>, _>("skipped_sections")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    current_section: row.get("current_section"),
                    research_tools_enabled: row.get::<i32, _>("research_tools_enabled") != 0,
                    generated_prd_id: row.get("generated_prd_id"),

                    // Phase 1 enhancement fields
                    non_goals: row.get("non_goals"),
                    open_questions: row.get("open_questions"),
                    constraints_assumptions: row.get("constraints_assumptions"),
                    success_metrics: row.get("success_metrics"),
                    alternative_approaches: row
                        .get::<Option<String>, _>("alternative_approaches")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    validation_checkpoints: row
                        .get::<Option<String>, _>("validation_checkpoints")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    codebase_context: row
                        .get::<Option<String>, _>("codebase_context")
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
        input: UpdateIdeateSessionInput,
    ) -> Result<IdeateSession> {
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
        if input.current_section.is_some() {
            updates.push(format!("current_section = ${}", bind_count));
            bind_count += 1;
        }
        if input.research_tools_enabled.is_some() {
            updates.push(format!("research_tools_enabled = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            return self.get_session(session_id).await;
        }

        updates.push("updated_at = datetime('now', 'utc')".to_string());

        let query = format!(
            "UPDATE ideate_sessions SET {} WHERE id = ${}
             RETURNING id, project_id, initial_description, mode, status, skipped_sections, current_section, research_tools_enabled, generated_prd_id, created_at, updated_at",
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
        if let Some(current_section) = input.current_section {
            q = q.bind(current_section);
        }
        if let Some(research_tools_enabled) = input.research_tools_enabled {
            q = q.bind(research_tools_enabled);
        }

        q = q.bind(session_id);

        let session = q.fetch_one(&self.db).await?;

        Ok(IdeateSession {
            id: session.get("id"),
            project_id: session.get("project_id"),
            initial_description: session.get("initial_description"),
            mode: session.get("mode"),
            status: session.get("status"),
            skipped_sections: session
                .get::<Option<String>, _>("skipped_sections")
                .and_then(|s| serde_json::from_str(&s).ok()),
            current_section: session.get("current_section"),
            research_tools_enabled: session.get::<i32, _>("research_tools_enabled") != 0,
            generated_prd_id: session.get("generated_prd_id"),

            // Phase 1 enhancement fields
            non_goals: session.get("non_goals"),
            open_questions: session.get("open_questions"),
            constraints_assumptions: session.get("constraints_assumptions"),
            success_metrics: session.get("success_metrics"),
            alternative_approaches: session
                .get::<Option<String>, _>("alternative_approaches")
                .and_then(|s| serde_json::from_str(&s).ok()),
            validation_checkpoints: session
                .get::<Option<String>, _>("validation_checkpoints")
                .and_then(|s| serde_json::from_str(&s).ok()),
            codebase_context: session
                .get::<Option<String>, _>("codebase_context")
                .and_then(|s| serde_json::from_str(&s).ok()),

            created_at: session.get("created_at"),
            updated_at: session.get("updated_at"),
        })
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM ideate_sessions WHERE id = $1")
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
        sqlx::query("UPDATE ideate_sessions SET skipped_sections = $1, updated_at = datetime('now', 'utc') WHERE id = $2")
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

    // ========================================================================
    // SECTION CRUD OPERATIONS
    // ========================================================================

    /// Save or update overview section
    pub async fn save_overview(
        &self,
        session_id: &str,
        overview: IdeateOverview,
    ) -> Result<IdeateOverview> {
        // Check if session exists
        self.get_session(session_id).await?;

        // Check if overview already exists
        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM ideate_overview WHERE session_id = $1")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        if let Some(existing_id) = existing {
            // Update existing
            sqlx::query(
                "UPDATE ideate_overview
                 SET problem_statement = $1, target_audience = $2, value_proposition = $3,
                     one_line_pitch = $4, ai_generated = $5
                 WHERE id = $6",
            )
            .bind(&overview.problem_statement)
            .bind(&overview.target_audience)
            .bind(&overview.value_proposition)
            .bind(&overview.one_line_pitch)
            .bind(overview.ai_generated)
            .bind(&existing_id)
            .execute(&self.db)
            .await?;

            self.get_overview(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("overview".to_string()))
        } else {
            // Insert new
            let id = nanoid::nanoid!(8);
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO ideate_overview
                 (id, session_id, problem_statement, target_audience, value_proposition, one_line_pitch, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&id)
            .bind(session_id)
            .bind(&overview.problem_statement)
            .bind(&overview.target_audience)
            .bind(&overview.value_proposition)
            .bind(&overview.one_line_pitch)
            .bind(overview.ai_generated)
            .bind(now)
            .execute(&self.db)
            .await?;

            self.get_overview(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("overview".to_string()))
        }
    }

    /// Get overview section
    pub async fn get_overview(&self, session_id: &str) -> Result<Option<IdeateOverview>> {
        let row = sqlx::query(
            "SELECT id, session_id, problem_statement, target_audience, value_proposition,
                    one_line_pitch, ai_generated, created_at
             FROM ideate_overview WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| IdeateOverview {
            id: r.get("id"),
            session_id: r.get("session_id"),
            problem_statement: r.get("problem_statement"),
            target_audience: r.get("target_audience"),
            value_proposition: r.get("value_proposition"),
            one_line_pitch: r.get("one_line_pitch"),
            ai_generated: r.get("ai_generated"),
            created_at: r.get("created_at"),
        }))
    }

    /// Delete overview section
    pub async fn delete_overview(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM ideate_overview WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Save or update UX section
    pub async fn save_ux(&self, session_id: &str, ux: IdeateUX) -> Result<IdeateUX> {
        self.get_session(session_id).await?;

        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM ideate_ux WHERE session_id = $1")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        let personas_json = ux
            .personas
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let flows_json = ux
            .user_flows
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        if let Some(existing_id) = existing {
            sqlx::query(
                "UPDATE ideate_ux
                 SET personas = $1, user_flows = $2, ui_considerations = $3,
                     ux_principles = $4, ai_generated = $5
                 WHERE id = $6",
            )
            .bind(personas_json)
            .bind(flows_json)
            .bind(&ux.ui_considerations)
            .bind(&ux.ux_principles)
            .bind(ux.ai_generated)
            .bind(&existing_id)
            .execute(&self.db)
            .await?;

            self.get_ux(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("ux".to_string()))
        } else {
            let id = nanoid::nanoid!(8);
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO ideate_ux
                 (id, session_id, personas, user_flows, ui_considerations, ux_principles, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&id)
            .bind(session_id)
            .bind(personas_json)
            .bind(flows_json)
            .bind(&ux.ui_considerations)
            .bind(&ux.ux_principles)
            .bind(ux.ai_generated)
            .bind(now)
            .execute(&self.db)
            .await?;

            self.get_ux(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("ux".to_string()))
        }
    }

    /// Get UX section
    pub async fn get_ux(&self, session_id: &str) -> Result<Option<IdeateUX>> {
        let row = sqlx::query(
            "SELECT id, session_id, personas, user_flows, ui_considerations, ux_principles, ai_generated, created_at
             FROM ideate_ux WHERE session_id = $1"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| IdeateUX {
            id: r.get("id"),
            session_id: r.get("session_id"),
            personas: r
                .get::<Option<String>, _>("personas")
                .and_then(|s| serde_json::from_str(&s).ok()),
            user_flows: r
                .get::<Option<String>, _>("user_flows")
                .and_then(|s| serde_json::from_str(&s).ok()),
            ui_considerations: r.get("ui_considerations"),
            ux_principles: r.get("ux_principles"),
            ai_generated: r.get("ai_generated"),
            created_at: r.get("created_at"),
        }))
    }

    /// Delete UX section
    pub async fn delete_ux(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM ideate_ux WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Save or update technical section
    pub async fn save_technical(
        &self,
        session_id: &str,
        technical: IdeateTechnical,
    ) -> Result<IdeateTechnical> {
        self.get_session(session_id).await?;

        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM ideate_technical WHERE session_id = $1")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        let components_json = technical
            .components
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let models_json = technical
            .data_models
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let apis_json = technical
            .apis
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let infra_json = technical
            .infrastructure
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        if let Some(existing_id) = existing {
            sqlx::query(
                "UPDATE ideate_technical
                 SET components = $1, data_models = $2, apis = $3, infrastructure = $4,
                     tech_stack_quick = $5, ai_generated = $6
                 WHERE id = $7",
            )
            .bind(components_json)
            .bind(models_json)
            .bind(apis_json)
            .bind(infra_json)
            .bind(&technical.tech_stack_quick)
            .bind(technical.ai_generated)
            .bind(&existing_id)
            .execute(&self.db)
            .await?;

            self.get_technical(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("technical".to_string()))
        } else {
            let id = nanoid::nanoid!(8);
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO ideate_technical
                 (id, session_id, components, data_models, apis, infrastructure, tech_stack_quick, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
            )
            .bind(&id)
            .bind(session_id)
            .bind(components_json)
            .bind(models_json)
            .bind(apis_json)
            .bind(infra_json)
            .bind(&technical.tech_stack_quick)
            .bind(technical.ai_generated)
            .bind(now)
            .execute(&self.db)
            .await?;

            self.get_technical(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("technical".to_string()))
        }
    }

    /// Get technical section
    pub async fn get_technical(&self, session_id: &str) -> Result<Option<IdeateTechnical>> {
        let row = sqlx::query(
            "SELECT id, session_id, components, data_models, apis, infrastructure, tech_stack_quick, ai_generated, created_at
             FROM ideate_technical WHERE session_id = $1"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| IdeateTechnical {
            id: r.get("id"),
            session_id: r.get("session_id"),
            components: r
                .get::<Option<String>, _>("components")
                .and_then(|s| serde_json::from_str(&s).ok()),
            data_models: r
                .get::<Option<String>, _>("data_models")
                .and_then(|s| serde_json::from_str(&s).ok()),
            apis: r
                .get::<Option<String>, _>("apis")
                .and_then(|s| serde_json::from_str(&s).ok()),
            infrastructure: r
                .get::<Option<String>, _>("infrastructure")
                .and_then(|s| serde_json::from_str(&s).ok()),
            tech_stack_quick: r.get("tech_stack_quick"),
            ai_generated: r.get("ai_generated"),
            created_at: r.get("created_at"),
        }))
    }

    /// Delete technical section
    pub async fn delete_technical(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM ideate_technical WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Save or update roadmap section
    pub async fn save_roadmap(
        &self,
        session_id: &str,
        roadmap: IdeateRoadmap,
    ) -> Result<IdeateRoadmap> {
        self.get_session(session_id).await?;

        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM ideate_roadmap WHERE session_id = $1")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        let mvp_json = roadmap
            .mvp_scope
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let phases_json = roadmap
            .future_phases
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        if let Some(existing_id) = existing {
            sqlx::query(
                "UPDATE ideate_roadmap
                 SET mvp_scope = $1, future_phases = $2, ai_generated = $3
                 WHERE id = $4",
            )
            .bind(mvp_json)
            .bind(phases_json)
            .bind(roadmap.ai_generated)
            .bind(&existing_id)
            .execute(&self.db)
            .await?;

            self.get_roadmap(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("roadmap".to_string()))
        } else {
            let id = nanoid::nanoid!(8);
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO ideate_roadmap
                 (id, session_id, mvp_scope, future_phases, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&id)
            .bind(session_id)
            .bind(mvp_json)
            .bind(phases_json)
            .bind(roadmap.ai_generated)
            .bind(now)
            .execute(&self.db)
            .await?;

            self.get_roadmap(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("roadmap".to_string()))
        }
    }

    /// Get roadmap section
    pub async fn get_roadmap(&self, session_id: &str) -> Result<Option<IdeateRoadmap>> {
        let row = sqlx::query(
            "SELECT id, session_id, mvp_scope, future_phases, ai_generated, created_at
             FROM ideate_roadmap WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| IdeateRoadmap {
            id: r.get("id"),
            session_id: r.get("session_id"),
            mvp_scope: r
                .get::<Option<String>, _>("mvp_scope")
                .and_then(|s| serde_json::from_str(&s).ok()),
            future_phases: r
                .get::<Option<String>, _>("future_phases")
                .and_then(|s| serde_json::from_str(&s).ok()),
            ai_generated: r.get("ai_generated"),
            created_at: r.get("created_at"),
        }))
    }

    /// Delete roadmap section
    pub async fn delete_roadmap(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM ideate_roadmap WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Save or update dependencies section
    pub async fn save_dependencies(
        &self,
        session_id: &str,
        deps: IdeateDependencies,
    ) -> Result<IdeateDependencies> {
        self.get_session(session_id).await?;

        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM ideate_dependencies WHERE session_id = $1")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        let foundation_json = deps
            .foundation_features
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let visible_json = deps
            .visible_features
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let enhancement_json = deps
            .enhancement_features
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let graph_json = deps
            .dependency_graph
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        if let Some(existing_id) = existing {
            sqlx::query(
                "UPDATE ideate_dependencies
                 SET foundation_features = $1, visible_features = $2, enhancement_features = $3,
                     dependency_graph = $4, ai_generated = $5
                 WHERE id = $6",
            )
            .bind(foundation_json)
            .bind(visible_json)
            .bind(enhancement_json)
            .bind(graph_json)
            .bind(deps.ai_generated)
            .bind(&existing_id)
            .execute(&self.db)
            .await?;

            self.get_dependencies(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("dependencies".to_string()))
        } else {
            let id = nanoid::nanoid!(8);
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO ideate_dependencies
                 (id, session_id, foundation_features, visible_features, enhancement_features, dependency_graph, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&id)
            .bind(session_id)
            .bind(foundation_json)
            .bind(visible_json)
            .bind(enhancement_json)
            .bind(graph_json)
            .bind(deps.ai_generated)
            .bind(now)
            .execute(&self.db)
            .await?;

            self.get_dependencies(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("dependencies".to_string()))
        }
    }

    /// Get dependencies section
    pub async fn get_dependencies(&self, session_id: &str) -> Result<Option<IdeateDependencies>> {
        let row = sqlx::query(
            "SELECT id, session_id, foundation_features, visible_features, enhancement_features, dependency_graph, ai_generated, created_at
             FROM ideate_dependencies WHERE session_id = $1"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| IdeateDependencies {
            id: r.get("id"),
            session_id: r.get("session_id"),
            foundation_features: r
                .get::<Option<String>, _>("foundation_features")
                .and_then(|s| serde_json::from_str(&s).ok()),
            visible_features: r
                .get::<Option<String>, _>("visible_features")
                .and_then(|s| serde_json::from_str(&s).ok()),
            enhancement_features: r
                .get::<Option<String>, _>("enhancement_features")
                .and_then(|s| serde_json::from_str(&s).ok()),
            dependency_graph: r
                .get::<Option<String>, _>("dependency_graph")
                .and_then(|s| serde_json::from_str(&s).ok()),
            ai_generated: r.get("ai_generated"),
            created_at: r.get("created_at"),
        }))
    }

    /// Delete dependencies section
    pub async fn delete_dependencies(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM ideate_dependencies WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Save or update risks section
    pub async fn save_risks(&self, session_id: &str, risks: IdeateRisks) -> Result<IdeateRisks> {
        self.get_session(session_id).await?;

        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM ideate_risks WHERE session_id = $1")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        let tech_risks_json = risks
            .technical_risks
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let mvp_risks_json = risks
            .mvp_scoping_risks
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let resource_risks_json = risks
            .resource_risks
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let mitigations_json = risks
            .mitigations
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        if let Some(existing_id) = existing {
            sqlx::query(
                "UPDATE ideate_risks
                 SET technical_risks = $1, mvp_scoping_risks = $2, resource_risks = $3,
                     mitigations = $4, ai_generated = $5
                 WHERE id = $6",
            )
            .bind(tech_risks_json)
            .bind(mvp_risks_json)
            .bind(resource_risks_json)
            .bind(mitigations_json)
            .bind(risks.ai_generated)
            .bind(&existing_id)
            .execute(&self.db)
            .await?;

            self.get_risks(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("risks".to_string()))
        } else {
            let id = nanoid::nanoid!(8);
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO ideate_risks
                 (id, session_id, technical_risks, mvp_scoping_risks, resource_risks, mitigations, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&id)
            .bind(session_id)
            .bind(tech_risks_json)
            .bind(mvp_risks_json)
            .bind(resource_risks_json)
            .bind(mitigations_json)
            .bind(risks.ai_generated)
            .bind(now)
            .execute(&self.db)
            .await?;

            self.get_risks(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("risks".to_string()))
        }
    }

    /// Get risks section
    pub async fn get_risks(&self, session_id: &str) -> Result<Option<IdeateRisks>> {
        let row = sqlx::query(
            "SELECT id, session_id, technical_risks, mvp_scoping_risks, resource_risks, mitigations, ai_generated, created_at
             FROM ideate_risks WHERE session_id = $1"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| IdeateRisks {
            id: r.get("id"),
            session_id: r.get("session_id"),
            technical_risks: r
                .get::<Option<String>, _>("technical_risks")
                .and_then(|s| serde_json::from_str(&s).ok()),
            mvp_scoping_risks: r
                .get::<Option<String>, _>("mvp_scoping_risks")
                .and_then(|s| serde_json::from_str(&s).ok()),
            resource_risks: r
                .get::<Option<String>, _>("resource_risks")
                .and_then(|s| serde_json::from_str(&s).ok()),
            mitigations: r
                .get::<Option<String>, _>("mitigations")
                .and_then(|s| serde_json::from_str(&s).ok()),
            ai_generated: r.get("ai_generated"),
            created_at: r.get("created_at"),
        }))
    }

    /// Delete risks section
    pub async fn delete_risks(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM ideate_risks WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Save or update research section
    pub async fn save_research(
        &self,
        session_id: &str,
        research: IdeateResearch,
    ) -> Result<IdeateResearch> {
        self.get_session(session_id).await?;

        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM ideate_research WHERE session_id = $1")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        let competitors_json = research
            .competitors
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let projects_json = research
            .similar_projects
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let refs_json = research
            .reference_links
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        if let Some(existing_id) = existing {
            sqlx::query(
                "UPDATE ideate_research
                 SET competitors = $1, similar_projects = $2, research_findings = $3,
                     technical_specs = $4, reference_links = $5, ai_generated = $6
                 WHERE id = $7",
            )
            .bind(competitors_json)
            .bind(projects_json)
            .bind(&research.research_findings)
            .bind(&research.technical_specs)
            .bind(refs_json)
            .bind(research.ai_generated)
            .bind(&existing_id)
            .execute(&self.db)
            .await?;

            self.get_research(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("research".to_string()))
        } else {
            let id = nanoid::nanoid!(8);
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO ideate_research
                 (id, session_id, competitors, similar_projects, research_findings, technical_specs, reference_links, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
            )
            .bind(&id)
            .bind(session_id)
            .bind(competitors_json)
            .bind(projects_json)
            .bind(&research.research_findings)
            .bind(&research.technical_specs)
            .bind(refs_json)
            .bind(research.ai_generated)
            .bind(now)
            .execute(&self.db)
            .await?;

            self.get_research(session_id)
                .await?
                .ok_or_else(|| IdeateError::SectionNotFound("research".to_string()))
        }
    }

    /// Get research section
    pub async fn get_research(&self, session_id: &str) -> Result<Option<IdeateResearch>> {
        let row = sqlx::query(
            "SELECT id, session_id, competitors, similar_projects, research_findings, technical_specs, reference_links, ai_generated, created_at
             FROM ideate_research WHERE session_id = $1"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| IdeateResearch {
            id: r.get("id"),
            session_id: r.get("session_id"),
            competitors: r
                .get::<Option<String>, _>("competitors")
                .and_then(|s| serde_json::from_str(&s).ok()),
            similar_projects: r
                .get::<Option<String>, _>("similar_projects")
                .and_then(|s| serde_json::from_str(&s).ok()),
            research_findings: r.get("research_findings"),
            technical_specs: r.get("technical_specs"),
            reference_links: r
                .get::<Option<String>, _>("reference_links")
                .and_then(|s| serde_json::from_str(&s).ok()),
            ai_generated: r.get("ai_generated"),
            created_at: r.get("created_at"),
        }))
    }

    /// Delete research section
    pub async fn delete_research(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM ideate_research WHERE session_id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // ========================================================================
    // NAVIGATION HELPERS
    // ========================================================================

    /// Get the next incomplete section
    pub async fn get_next_section(&self, session_id: &str) -> Result<Option<String>> {
        let sections = [
            "overview",
            "features",
            "ux",
            "technical",
            "roadmap",
            "dependencies",
            "risks",
            "research",
        ];

        for section in sections {
            let has_data = match section {
                "overview" => self.has_overview(session_id).await?,
                "features" => self.has_features(session_id).await?,
                "ux" => self.has_ux(session_id).await?,
                "technical" => self.has_technical(session_id).await?,
                "roadmap" => self.has_roadmap(session_id).await?,
                "dependencies" => self.has_dependencies(session_id).await?,
                "risks" => self.has_risks(session_id).await?,
                "research" => self.has_research(session_id).await?,
                _ => false,
            };

            if !has_data {
                return Ok(Some(section.to_string()));
            }
        }

        Ok(None) // All sections complete
    }

    /// Navigate to a specific section (updates current_section field)
    pub async fn navigate_to(&self, session_id: &str, section: &str) -> Result<IdeateSession> {
        let valid_sections = [
            "overview",
            "features",
            "ux",
            "technical",
            "roadmap",
            "dependencies",
            "risks",
            "research",
        ];

        if !valid_sections.contains(&section) {
            return Err(IdeateError::InvalidInput(format!(
                "Invalid section: {}",
                section
            )));
        }

        self.update_session(
            session_id,
            UpdateIdeateSessionInput {
                current_section: Some(section.to_string()),
                initial_description: None,
                mode: None,
                status: None,
                skipped_sections: None,
                research_tools_enabled: None,
            },
        )
        .await
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
            IdeateMode::Quick => true,                  // Quick mode is always ready
            IdeateMode::Guided => completed_count >= 2, // At least 2 sections
            IdeateMode::Chat => true,                   // Chat uses quality score instead
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
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ideate_overview WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }

    async fn has_features(&self, session_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ideate_features WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }

    async fn has_ux(&self, session_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ideate_ux WHERE session_id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn has_technical(&self, session_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ideate_technical WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }

    async fn has_roadmap(&self, session_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ideate_roadmap WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }

    async fn has_dependencies(&self, session_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ideate_dependencies WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }

    async fn has_risks(&self, session_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ideate_risks WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }

    async fn has_research(&self, session_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ideate_research WHERE session_id = $1")
                .bind(session_id)
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }

    /// Apply template defaults to a session, creating sections with template data
    pub async fn apply_template_to_session(
        &self,
        session_id: &str,
        template: &PRDTemplate,
    ) -> Result<()> {
        let now = Utc::now();

        // Overview section
        if template.default_problem_statement.is_some()
            || template.default_target_audience.is_some()
            || template.default_value_proposition.is_some()
        {
            sqlx::query(
                "INSERT INTO ideate_overview (id, session_id, problem_statement, target_audience, value_proposition, one_line_pitch, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, NULL, 1, $6)"
            )
            .bind(nanoid::nanoid!(8))
            .bind(session_id)
            .bind(&template.default_problem_statement)
            .bind(&template.default_target_audience)
            .bind(&template.default_value_proposition)
            .bind(now)
            .execute(&self.db)
            .await?;
        }

        // UX section
        if template.default_ui_considerations.is_some() || template.default_ux_principles.is_some()
        {
            sqlx::query(
                "INSERT INTO ideate_ux (id, session_id, ui_considerations, ux_principles, personas, user_flows, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, NULL, NULL, 1, $5)"
            )
            .bind(nanoid::nanoid!(8))
            .bind(session_id)
            .bind(&template.default_ui_considerations)
            .bind(&template.default_ux_principles)
            .bind(now)
            .execute(&self.db)
            .await?;
        }

        // Technical section
        if template.default_tech_stack_quick.is_some() {
            sqlx::query(
                "INSERT INTO ideate_technical (id, session_id, tech_stack_quick, components, data_models, apis, infrastructure, ai_generated, created_at)
                 VALUES ($1, $2, $3, NULL, NULL, NULL, NULL, 1, $4)"
            )
            .bind(nanoid::nanoid!(8))
            .bind(session_id)
            .bind(&template.default_tech_stack_quick)
            .bind(now)
            .execute(&self.db)
            .await?;
        }

        // Roadmap section
        if let Some(mvp_scope) = &template.default_mvp_scope {
            let mvp_scope_json = serde_json::to_string(mvp_scope)?;
            sqlx::query(
                "INSERT INTO ideate_roadmap (id, session_id, mvp_scope, future_phases, ai_generated, created_at)
                 VALUES ($1, $2, $3, NULL, 1, $4)"
            )
            .bind(nanoid::nanoid!(8))
            .bind(session_id)
            .bind(&mvp_scope_json)
            .bind(now)
            .execute(&self.db)
            .await?;
        }

        // Research section
        if template.default_research_findings.is_some()
            || template.default_technical_specs.is_some()
            || template.default_competitors.is_some()
            || template.default_similar_projects.is_some()
        {
            let competitors_json = template
                .default_competitors
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;
            let similar_projects_json = template
                .default_similar_projects
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;

            sqlx::query(
                "INSERT INTO ideate_research (id, session_id, research_findings, technical_specs, competitors, similar_projects, reference_links, ai_generated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, NULL, 1, $7)"
            )
            .bind(nanoid::nanoid!(8))
            .bind(session_id)
            .bind(&template.default_research_findings)
            .bind(&template.default_technical_specs)
            .bind(&competitors_json)
            .bind(&similar_projects_json)
            .bind(now)
            .execute(&self.db)
            .await?;
        }

        // Dependencies section (existing logic for features)
        if let Some(features) = &template.default_features {
            let features_json = serde_json::to_string(features)?;
            let deps_json = template
                .default_dependencies
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;

            sqlx::query(
                "INSERT INTO ideate_dependencies (id, session_id, foundation_features, visible_features, enhancement_features, dependency_graph, ai_generated, created_at)
                 VALUES ($1, $2, $3, NULL, NULL, $4, 1, $5)"
            )
            .bind(nanoid::nanoid!(8))
            .bind(session_id)
            .bind(&features_json)
            .bind(&deps_json)
            .bind(now)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }
}
