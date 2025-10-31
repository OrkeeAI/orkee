// ABOUTME: Epic management operations and database interactions
// ABOUTME: Handles CRUD operations, progress tracking, and work analysis for Epics

use crate::epic::{
    ArchitectureDecision, ConflictAnalysis, CreateEpicInput, DependencyGraph, Epic, EpicStatus,
    ExternalDependency, GraphEdge, GraphNode, SuccessCriterion, UpdateEpicInput, WorkAnalysis,
    WorkStream,
};
use crate::error::{IdeateError, Result};
use sqlx::SqlitePool;

pub struct EpicManager {
    pool: SqlitePool,
}

impl EpicManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new Epic
    pub async fn create_epic(
        &self,
        project_id: &str,
        input: CreateEpicInput,
    ) -> Result<Epic> {
        let id = nanoid::nanoid!(12);

        // Serialize JSON fields
        let architecture_decisions_json = input
            .architecture_decisions
            .as_ref()
            .map(|d| serde_json::to_string(d))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        let dependencies_json = input
            .dependencies
            .as_ref()
            .map(|d| serde_json::to_string(d))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        let success_criteria_json = input
            .success_criteria
            .as_ref()
            .map(|s| serde_json::to_string(s))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        let task_categories_json = input
            .task_categories
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO epics (
                id, project_id, prd_id, name, overview_markdown,
                architecture_decisions, technical_approach, implementation_strategy,
                dependencies, success_criteria, task_categories,
                estimated_effort, complexity, status, progress_percentage
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'draft', 0)
            "#,
        )
        .bind(&id)
        .bind(project_id)
        .bind(&input.prd_id)
        .bind(&input.name)
        .bind(&input.overview_markdown)
        .bind(architecture_decisions_json)
        .bind(&input.technical_approach)
        .bind(&input.implementation_strategy)
        .bind(dependencies_json)
        .bind(success_criteria_json)
        .bind(task_categories_json)
        .bind(&input.estimated_effort)
        .bind(&input.complexity)
        .execute(&self.pool)
        .await
        .map_err(|e| IdeateError::DatabaseError(e.to_string()))?;

        self.get_epic(project_id, &id)
            .await?
            .ok_or_else(|| IdeateError::NotFound(format!("Epic {} not found after creation", id)))
    }

    /// Get an Epic by ID
    pub async fn get_epic(&self, project_id: &str, epic_id: &str) -> Result<Option<Epic>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, project_id, prd_id, name, overview_markdown,
                architecture_decisions, technical_approach, implementation_strategy,
                dependencies, success_criteria, task_categories,
                estimated_effort, complexity, status, progress_percentage,
                github_issue_number, github_issue_url, github_synced_at,
                created_at, updated_at, started_at, completed_at
            FROM epics
            WHERE id = ? AND project_id = ?
            "#,
        )
        .bind(epic_id)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IdeateError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let epic = self.row_to_epic(row)?;
                Ok(Some(epic))
            }
            None => Ok(None),
        }
    }

    /// List all Epics for a project
    pub async fn list_epics(&self, project_id: &str) -> Result<Vec<Epic>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, project_id, prd_id, name, overview_markdown,
                architecture_decisions, technical_approach, implementation_strategy,
                dependencies, success_criteria, task_categories,
                estimated_effort, complexity, status, progress_percentage,
                github_issue_number, github_issue_url, github_synced_at,
                created_at, updated_at, started_at, completed_at
            FROM epics
            WHERE project_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IdeateError::DatabaseError(e.to_string()))?;

        rows.into_iter()
            .map(|row| self.row_to_epic(row))
            .collect()
    }

    /// List Epics for a specific PRD
    pub async fn list_epics_by_prd(&self, project_id: &str, prd_id: &str) -> Result<Vec<Epic>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, project_id, prd_id, name, overview_markdown,
                architecture_decisions, technical_approach, implementation_strategy,
                dependencies, success_criteria, task_categories,
                estimated_effort, complexity, status, progress_percentage,
                github_issue_number, github_issue_url, github_synced_at,
                created_at, updated_at, started_at, completed_at
            FROM epics
            WHERE project_id = ? AND prd_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(project_id)
        .bind(prd_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IdeateError::DatabaseError(e.to_string()))?;

        rows.into_iter()
            .map(|row| self.row_to_epic(row))
            .collect()
    }

    /// Update an Epic
    pub async fn update_epic(
        &self,
        project_id: &str,
        epic_id: &str,
        input: UpdateEpicInput,
    ) -> Result<Epic> {
        // Build dynamic UPDATE query
        let mut query = String::from("UPDATE epics SET ");
        let mut updates = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(name) = &input.name {
            updates.push("name = ?");
            params.push(name.clone());
        }
        if let Some(overview) = &input.overview_markdown {
            updates.push("overview_markdown = ?");
            params.push(overview.clone());
        }
        if let Some(arch_decisions) = &input.architecture_decisions {
            updates.push("architecture_decisions = ?");
            let json = serde_json::to_string(arch_decisions)
                .map_err(|e| IdeateError::SerializationError(e.to_string()))?;
            params.push(json);
        }
        if let Some(tech_approach) = &input.technical_approach {
            updates.push("technical_approach = ?");
            params.push(tech_approach.clone());
        }
        if let Some(impl_strategy) = &input.implementation_strategy {
            updates.push("implementation_strategy = ?");
            params.push(impl_strategy.clone());
        }
        if let Some(deps) = &input.dependencies {
            updates.push("dependencies = ?");
            let json = serde_json::to_string(deps)
                .map_err(|e| IdeateError::SerializationError(e.to_string()))?;
            params.push(json);
        }
        if let Some(criteria) = &input.success_criteria {
            updates.push("success_criteria = ?");
            let json = serde_json::to_string(criteria)
                .map_err(|e| IdeateError::SerializationError(e.to_string()))?;
            params.push(json);
        }
        if let Some(categories) = &input.task_categories {
            updates.push("task_categories = ?");
            let json = serde_json::to_string(categories)
                .map_err(|e| IdeateError::SerializationError(e.to_string()))?;
            params.push(json);
        }
        if let Some(effort) = &input.estimated_effort {
            updates.push("estimated_effort = ?");
            let effort_str = match effort {
                crate::epic::EstimatedEffort::Days => "days",
                crate::epic::EstimatedEffort::Weeks => "weeks",
                crate::epic::EstimatedEffort::Months => "months",
            };
            params.push(effort_str.to_string());
        }
        if let Some(complexity) = &input.complexity {
            updates.push("complexity = ?");
            let complexity_str = match complexity {
                crate::epic::EpicComplexity::Low => "low",
                crate::epic::EpicComplexity::Medium => "medium",
                crate::epic::EpicComplexity::High => "high",
                crate::epic::EpicComplexity::VeryHigh => "very_high",
            };
            params.push(complexity_str.to_string());
        }
        if let Some(status) = &input.status {
            updates.push("status = ?");
            let status_str = match status {
                EpicStatus::Draft => "draft",
                EpicStatus::Ready => "ready",
                EpicStatus::InProgress => "in_progress",
                EpicStatus::Blocked => "blocked",
                EpicStatus::Completed => "completed",
                EpicStatus::Cancelled => "cancelled",
            };
            params.push(status_str.to_string());
        }
        if let Some(progress) = &input.progress_percentage {
            updates.push("progress_percentage = ?");
            params.push(progress.to_string());
        }

        if updates.is_empty() {
            return self
                .get_epic(project_id, epic_id)
                .await?
                .ok_or_else(|| IdeateError::NotFound(format!("Epic {} not found", epic_id)));
        }

        query.push_str(&updates.join(", "));
        query.push_str(" WHERE id = ? AND project_id = ?");

        let mut q = sqlx::query(&query);
        for param in params {
            q = q.bind(param);
        }
        q = q.bind(epic_id).bind(project_id);

        q.execute(&self.pool)
            .await
            .map_err(|e| IdeateError::DatabaseError(e.to_string()))?;

        self.get_epic(project_id, epic_id)
            .await?
            .ok_or_else(|| IdeateError::NotFound(format!("Epic {} not found", epic_id)))
    }

    /// Delete an Epic
    pub async fn delete_epic(&self, project_id: &str, epic_id: &str) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM epics
            WHERE id = ? AND project_id = ?
            "#,
        )
        .bind(epic_id)
        .bind(project_id)
        .execute(&self.pool)
        .await
        .map_err(|e| IdeateError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IdeateError::NotFound(format!(
                "Epic {} not found",
                epic_id
            )));
        }

        Ok(())
    }

    /// Calculate Epic progress based on task completion
    pub async fn calculate_progress(&self, _project_id: &str, epic_id: &str) -> Result<i32> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(CASE WHEN status = 'done' THEN 1 END) as completed
            FROM tasks
            WHERE epic_id = ?
            "#,
        )
        .bind(epic_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| IdeateError::DatabaseError(e.to_string()))?;

        let total: i64 = row.get("total");
        let completed: i64 = row.get("completed");

        if total == 0 {
            return Ok(0);
        }

        let progress = ((completed as f64 / total as f64) * 100.0).round() as i32;
        Ok(progress)
    }

    /// Helper to convert SQLite row to Epic
    fn row_to_epic(&self, row: sqlx::sqlite::SqliteRow) -> Result<Epic> {
        use sqlx::Row;

        let architecture_decisions: Option<Vec<ArchitectureDecision>> = row
            .get::<Option<String>, _>("architecture_decisions")
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        let dependencies: Option<Vec<ExternalDependency>> = row
            .get::<Option<String>, _>("dependencies")
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        let success_criteria: Option<Vec<SuccessCriterion>> = row
            .get::<Option<String>, _>("success_criteria")
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        let task_categories: Option<Vec<String>> = row
            .get::<Option<String>, _>("task_categories")
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| IdeateError::SerializationError(e.to_string()))?;

        Ok(Epic {
            id: row.get("id"),
            project_id: row.get("project_id"),
            prd_id: row.get("prd_id"),
            name: row.get("name"),
            overview_markdown: row.get("overview_markdown"),
            architecture_decisions,
            technical_approach: row.get("technical_approach"),
            implementation_strategy: row.get("implementation_strategy"),
            dependencies,
            success_criteria,
            task_categories,
            estimated_effort: row.get("estimated_effort"),
            complexity: row.get("complexity"),
            status: row.get("status"),
            progress_percentage: row.get("progress_percentage"),
            github_issue_number: row.get("github_issue_number"),
            github_issue_url: row.get("github_issue_url"),
            github_synced_at: row.get("github_synced_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
        })
    }
}
