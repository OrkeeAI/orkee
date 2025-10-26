// ABOUTME: Task storage layer using SQLite
// ABOUTME: Handles CRUD operations for tasks with agent assignment support

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use tracing::debug;

use super::types::{Task, TaskCreateInput, TaskPriority, TaskStatus, TaskUpdateInput};
use crate::models::REGISTRY;
use storage::StorageError;

pub struct TaskStorage {
    pool: SqlitePool,
}

impl TaskStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_tasks(&self, project_id: &str) -> Result<Vec<Task>, StorageError> {
        let (tasks, _) = self.list_tasks_paginated(project_id, None, None).await?;
        Ok(tasks)
    }

    pub async fn list_tasks_paginated(
        &self,
        project_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<(Vec<Task>, i64), StorageError> {
        debug!(
            "Fetching tasks for project: {} (limit: {:?}, offset: {:?})",
            project_id, limit, offset
        );

        // Get total count
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM tasks t
            WHERE t.project_id = ?
            AND t.parent_id IS NULL
            "#,
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        // Build query with optional pagination
        let mut query_str = String::from(
            r#"
            SELECT
                t.*,
                a.id as agent_id,
                a.name as agent_name,
                a.type as agent_type,
                a.provider as agent_provider,
                a.display_name as agent_display_name,
                a.description as agent_description
            FROM tasks t
            LEFT JOIN agents a ON t.assigned_agent_id = a.id
            WHERE t.project_id = ?
            AND t.parent_id IS NULL
            ORDER BY t.position, t.created_at
            "#,
        );

        if let Some(lim) = limit {
            query_str.push_str(&format!(" LIMIT {}", lim));
        }
        if let Some(off) = offset {
            query_str.push_str(&format!(" OFFSET {}", off));
        }

        let rows = sqlx::query(&query_str)
            .bind(project_id)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let mut tasks = Vec::new();
        for row in rows {
            let mut task = self.row_to_task_sync(&row)?;
            // Fetch subtasks for parent tasks
            if task.parent_id.is_none() {
                task.subtasks = Some(self.get_subtasks(&task.id).await?);
            }
            tasks.push(task);
        }

        Ok((tasks, count))
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Task, StorageError> {
        debug!("Fetching task: {}", task_id);

        let row = sqlx::query(
            r#"
            SELECT
                t.*,
                a.id as agent_id,
                a.name as agent_name,
                a.type as agent_type,
                a.provider as agent_provider,
                a.display_name as agent_display_name,
                a.description as agent_description
            FROM tasks t
            LEFT JOIN agents a ON t.assigned_agent_id = a.id
            WHERE t.id = ?
            "#,
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        let mut task = self.row_to_task_sync(&row)?;
        // Fetch subtasks if this is a parent task
        if task.parent_id.is_none() {
            task.subtasks = Some(self.get_subtasks(&task.id).await?);
        }
        Ok(task)
    }

    pub async fn create_task(
        &self,
        project_id: &str,
        user_id: &str,
        input: TaskCreateInput,
    ) -> Result<Task, StorageError> {
        let task_id = nanoid::nanoid!();
        let now = Utc::now();
        let status = input.status.unwrap_or(TaskStatus::Pending);
        let priority = input.priority.unwrap_or(TaskPriority::Medium);

        debug!("Creating task: {} for project: {}", task_id, project_id);

        // Validate agent exists in registry if provided (replaces DB foreign key constraint)
        if let Some(agent_id) = &input.assigned_agent_id {
            if !crate::models::REGISTRY.agent_exists(agent_id) {
                return Err(StorageError::InvalidAgent(agent_id.to_string()));
            }
        }

        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, project_id, title, description, status, priority,
                created_by_user_id, assigned_agent_id, parent_id, position,
                dependencies, due_date, estimated_hours, complexity_score,
                details, test_strategy, acceptance_criteria,
                prompt, context, tags, category,
                retry_count, created_at, updated_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?,
                ?, ?, ?, ?,
                0, ?, ?
            )
            "#,
        )
        .bind(&task_id)
        .bind(project_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&status)
        .bind(&priority)
        .bind(user_id)
        .bind(&input.assigned_agent_id)
        .bind(&input.parent_id)
        .bind(input.position.unwrap_or(0))
        .bind(
            input
                .dependencies
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap()),
        )
        .bind(input.due_date)
        .bind(input.estimated_hours)
        .bind(input.complexity_score)
        .bind(&input.details)
        .bind(&input.test_strategy)
        .bind(&input.acceptance_criteria)
        .bind(&input.prompt)
        .bind(&input.context)
        .bind(
            input
                .tags
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap()),
        )
        .bind(&input.category)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        self.get_task(&task_id).await
    }

    pub async fn update_task(
        &self,
        task_id: &str,
        input: TaskUpdateInput,
    ) -> Result<Task, StorageError> {
        debug!("Updating task: {}", task_id);

        // Validate agent exists in registry if provided (replaces DB foreign key constraint)
        if let Some(agent_id) = &input.assigned_agent_id {
            if !crate::models::REGISTRY.agent_exists(agent_id) {
                return Err(StorageError::InvalidAgent(agent_id.to_string()));
            }
        }

        // Build dynamic UPDATE query based on provided fields
        let mut query = String::from("UPDATE tasks SET updated_at = ?");
        let mut has_updates = false;

        if input.title.is_some() {
            query.push_str(", title = ?");
            has_updates = true;
        }
        if input.description.is_some() {
            query.push_str(", description = ?");
            has_updates = true;
        }
        if input.status.is_some() {
            query.push_str(", status = ?");
            has_updates = true;
        }
        if input.priority.is_some() {
            query.push_str(", priority = ?");
            has_updates = true;
        }
        if input.assigned_agent_id.is_some() {
            query.push_str(", assigned_agent_id = ?");
            has_updates = true;
        }
        if input.position.is_some() {
            query.push_str(", position = ?");
            has_updates = true;
        }
        if input.dependencies.is_some() {
            query.push_str(", dependencies = ?");
            has_updates = true;
        }
        if input.due_date.is_some() {
            query.push_str(", due_date = ?");
            has_updates = true;
        }
        if input.estimated_hours.is_some() {
            query.push_str(", estimated_hours = ?");
            has_updates = true;
        }
        if input.actual_hours.is_some() {
            query.push_str(", actual_hours = ?");
            has_updates = true;
        }
        if input.complexity_score.is_some() {
            query.push_str(", complexity_score = ?");
            has_updates = true;
        }
        if input.details.is_some() {
            query.push_str(", details = ?");
            has_updates = true;
        }
        if input.test_strategy.is_some() {
            query.push_str(", test_strategy = ?");
            has_updates = true;
        }
        if input.acceptance_criteria.is_some() {
            query.push_str(", acceptance_criteria = ?");
            has_updates = true;
        }
        if input.tags.is_some() {
            query.push_str(", tags = ?");
            has_updates = true;
        }
        if input.category.is_some() {
            query.push_str(", category = ?");
            has_updates = true;
        }

        query.push_str(" WHERE id = ?");

        if !has_updates {
            return self.get_task(task_id).await;
        }

        let now = Utc::now();
        let mut q = sqlx::query(&query).bind(now);

        if let Some(title) = &input.title {
            q = q.bind(title);
        }
        if let Some(description) = &input.description {
            q = q.bind(description);
        }
        if let Some(status) = &input.status {
            q = q.bind(status);
        }
        if let Some(priority) = &input.priority {
            q = q.bind(priority);
        }
        if let Some(agent_id) = &input.assigned_agent_id {
            q = q.bind(agent_id);
        }
        if let Some(position) = input.position {
            q = q.bind(position);
        }
        if let Some(deps) = &input.dependencies {
            q = q.bind(serde_json::to_string(deps).unwrap());
        }
        if let Some(due_date) = &input.due_date {
            q = q.bind(due_date);
        }
        if let Some(hours) = input.estimated_hours {
            q = q.bind(hours);
        }
        if let Some(hours) = input.actual_hours {
            q = q.bind(hours);
        }
        if let Some(score) = input.complexity_score {
            q = q.bind(score);
        }
        if let Some(details) = &input.details {
            q = q.bind(details);
        }
        if let Some(strategy) = &input.test_strategy {
            q = q.bind(strategy);
        }
        if let Some(criteria) = &input.acceptance_criteria {
            q = q.bind(criteria);
        }
        if let Some(tags) = &input.tags {
            q = q.bind(serde_json::to_string(tags).unwrap());
        }
        if let Some(category) = &input.category {
            q = q.bind(category);
        }

        q = q.bind(task_id);

        q.execute(&self.pool).await.map_err(StorageError::Sqlx)?;

        self.get_task(task_id).await
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), StorageError> {
        debug!("Deleting task: {}", task_id);

        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    pub async fn get_subtasks(&self, parent_id: &str) -> Result<Vec<Task>, StorageError> {
        debug!("Fetching subtasks for parent: {}", parent_id);

        let rows = sqlx::query(
            r#"
            SELECT
                t.*,
                a.id as agent_id,
                a.name as agent_name,
                a.type as agent_type,
                a.provider as agent_provider,
                a.display_name as agent_display_name,
                a.description as agent_description
            FROM tasks t
            LEFT JOIN agents a ON t.assigned_agent_id = a.id
            WHERE t.parent_id = ?
            ORDER BY t.position, t.created_at
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        let mut tasks = Vec::new();
        for row in rows {
            // Subtasks don't have subtasks, so we don't need to fetch recursively
            let task = self.row_to_task_sync(&row)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    fn row_to_task_sync(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Task, StorageError> {
        let task_id: String = row.try_get("id")?;
        let subtasks = None; // Will be populated by the caller if needed

        // Load agent from JSON registry if present
        let assigned_agent =
            if let Ok(Some(agent_id)) = row.try_get::<Option<String>, _>("assigned_agent_id") {
                REGISTRY.get_agent(&agent_id).cloned()
            } else {
                None
            };

        Ok(Task {
            id: task_id,
            project_id: row.try_get("project_id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            status: row.try_get("status")?,
            priority: row.try_get("priority")?,
            created_by_user_id: row.try_get("created_by_user_id")?,
            assigned_agent_id: row.try_get("assigned_agent_id")?,
            assigned_agent,
            reviewed_by_agent_id: row.try_get("reviewed_by_agent_id")?,
            parent_id: row.try_get("parent_id")?,
            position: row.try_get("position")?,
            subtasks,
            dependencies: row
                .try_get::<Option<String>, _>("dependencies")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            blockers: row
                .try_get::<Option<String>, _>("blockers")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            due_date: row.try_get("due_date")?,
            estimated_hours: row.try_get("estimated_hours")?,
            actual_hours: row.try_get("actual_hours")?,
            complexity_score: row.try_get("complexity_score")?,
            details: row.try_get("details")?,
            test_strategy: row.try_get("test_strategy")?,
            acceptance_criteria: row.try_get("acceptance_criteria")?,
            prompt: row.try_get("prompt")?,
            context: row.try_get("context")?,
            output_format: row.try_get("output_format")?,
            validation_rules: row
                .try_get::<Option<String>, _>("validation_rules")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            execution_log: row
                .try_get::<Option<String>, _>("execution_log")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            error_log: row
                .try_get::<Option<String>, _>("error_log")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            retry_count: row.try_get("retry_count")?,
            tags: row
                .try_get::<Option<String>, _>("tags")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            category: row.try_get("category")?,
            metadata: row
                .try_get::<Option<String>, _>("metadata")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
