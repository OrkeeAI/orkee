// ABOUTME: Agent execution and PR review storage layer using SQLite
// ABOUTME: Handles CRUD operations for execution tracking and code reviews

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use tracing::debug;

use super::types::{
    AgentExecution, AgentExecutionCreateInput, AgentExecutionUpdateInput, ExecutionStatus,
    PrReview, PrReviewCreateInput, PrReviewUpdateInput,
};
use models::REGISTRY;
use storage::StorageError;

pub struct ExecutionStorage {
    pool: SqlitePool,
}

impl ExecutionStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ==================== Agent Executions ====================

    /// List all executions for a task
    pub async fn list_executions(
        &self,
        task_id: &str,
    ) -> Result<Vec<AgentExecution>, StorageError> {
        let (executions, _) = self.list_executions_paginated(task_id, None, None).await?;
        Ok(executions)
    }

    /// List all executions for a task with pagination
    pub async fn list_executions_paginated(
        &self,
        task_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<(Vec<AgentExecution>, i64), StorageError> {
        debug!(
            "Fetching executions for task: {} (limit: {:?}, offset: {:?})",
            task_id, limit, offset
        );

        // Get total count
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM agent_executions WHERE task_id = ?")
                .bind(task_id)
                .fetch_one(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;

        // Build query with optional pagination
        let mut query = String::from(
            "SELECT * FROM agent_executions WHERE task_id = ? ORDER BY started_at DESC",
        );

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }
        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET {}", off));
        }

        let rows = sqlx::query(&query)
            .bind(task_id)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let executions = rows
            .iter()
            .map(|row| self.row_to_execution(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((executions, count))
    }

    /// Get a single execution by ID
    pub async fn get_execution(&self, execution_id: &str) -> Result<AgentExecution, StorageError> {
        debug!("Fetching execution: {}", execution_id);

        let row = sqlx::query("SELECT * FROM agent_executions WHERE id = ?")
            .bind(execution_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_execution(&row)
    }

    /// Create a new execution
    pub async fn create_execution(
        &self,
        input: AgentExecutionCreateInput,
    ) -> Result<AgentExecution, StorageError> {
        let execution_id = format!("exec-{}", nanoid::nanoid!());
        let now = Utc::now();

        debug!(
            "Creating execution: {} for task: {}",
            execution_id, input.task_id
        );

        // Validate agent exists in registry if provided (replaces DB foreign key constraint)
        if let Some(agent_id) = &input.agent_id {
            if !REGISTRY.agent_exists(agent_id) {
                return Err(StorageError::InvalidAgent(agent_id.to_string()));
            }
        }

        // Validate model exists in registry if provided (replaces DB foreign key constraint)
        if let Some(model_id) = &input.model {
            if !REGISTRY.model_exists(model_id) {
                return Err(StorageError::InvalidModel(model_id.to_string()));
            }
        }

        // Validate model is supported by agent if both are provided
        if let (Some(agent_id), Some(model_id)) = (&input.agent_id, &input.model) {
            if !REGISTRY.validate_agent_model(agent_id, model_id) {
                return Err(StorageError::InvalidAgentModel {
                    agent_id: agent_id.to_string(),
                    model_id: model_id.to_string(),
                });
            }
        }

        sqlx::query(
            r#"
            INSERT INTO agent_executions (
                id, task_id, agent_id, model, started_at, status,
                prompt, retry_attempt, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&execution_id)
        .bind(&input.task_id)
        .bind(&input.agent_id)
        .bind(&input.model)
        .bind(now)
        .bind(ExecutionStatus::Running)
        .bind(&input.prompt)
        .bind(input.retry_attempt.unwrap_or(0))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        self.get_execution(&execution_id).await
    }

    /// Update an execution
    pub async fn update_execution(
        &self,
        execution_id: &str,
        input: AgentExecutionUpdateInput,
    ) -> Result<AgentExecution, StorageError> {
        debug!("Updating execution: {}", execution_id);

        let now = Utc::now();
        let mut updates = vec!["updated_at = ?"];
        let mut query_str = String::from("UPDATE agent_executions SET ");

        // Build dynamic update query
        if input.status.is_some() {
            updates.push("status = ?");
        }
        if input.completed_at.is_some() {
            updates.push("completed_at = ?");
        }
        if input.execution_time_seconds.is_some() {
            updates.push("execution_time_seconds = ?");
        }
        if input.tokens_input.is_some() {
            updates.push("tokens_input = ?");
        }
        if input.tokens_output.is_some() {
            updates.push("tokens_output = ?");
        }
        if input.total_cost.is_some() {
            updates.push("total_cost = ?");
        }
        if input.response.is_some() {
            updates.push("response = ?");
        }
        if input.error_message.is_some() {
            updates.push("error_message = ?");
        }
        if input.files_changed.is_some() {
            updates.push("files_changed = ?");
        }
        if input.lines_added.is_some() {
            updates.push("lines_added = ?");
        }
        if input.lines_removed.is_some() {
            updates.push("lines_removed = ?");
        }
        if input.files_created.is_some() {
            updates.push("files_created = ?");
        }
        if input.files_modified.is_some() {
            updates.push("files_modified = ?");
        }
        if input.files_deleted.is_some() {
            updates.push("files_deleted = ?");
        }
        if input.branch_name.is_some() {
            updates.push("branch_name = ?");
        }
        if input.commit_hash.is_some() {
            updates.push("commit_hash = ?");
        }
        if input.commit_message.is_some() {
            updates.push("commit_message = ?");
        }
        if input.pr_number.is_some() {
            updates.push("pr_number = ?");
        }
        if input.pr_url.is_some() {
            updates.push("pr_url = ?");
        }
        if input.pr_title.is_some() {
            updates.push("pr_title = ?");
        }
        if input.pr_status.is_some() {
            updates.push("pr_status = ?");
        }
        if input.pr_created_at.is_some() {
            updates.push("pr_created_at = ?");
        }
        if input.pr_merged_at.is_some() {
            updates.push("pr_merged_at = ?");
        }
        if input.pr_merge_commit.is_some() {
            updates.push("pr_merge_commit = ?");
        }
        if input.review_status.is_some() {
            updates.push("review_status = ?");
        }
        if input.review_comments.is_some() {
            updates.push("review_comments = ?");
        }
        if input.test_results.is_some() {
            updates.push("test_results = ?");
        }
        if input.performance_metrics.is_some() {
            updates.push("performance_metrics = ?");
        }
        if input.metadata.is_some() {
            updates.push("metadata = ?");
        }

        query_str.push_str(&updates.join(", "));
        query_str.push_str(" WHERE id = ?");

        let mut query = sqlx::query(&query_str).bind(now);

        // Bind parameters in the same order
        if let Some(status) = input.status {
            query = query.bind(status);
        }
        if let Some(completed_at) = input.completed_at {
            query = query.bind(completed_at);
        }
        if let Some(execution_time) = input.execution_time_seconds {
            query = query.bind(execution_time);
        }
        if let Some(tokens_in) = input.tokens_input {
            query = query.bind(tokens_in);
        }
        if let Some(tokens_out) = input.tokens_output {
            query = query.bind(tokens_out);
        }
        if let Some(cost) = input.total_cost {
            query = query.bind(cost);
        }
        if let Some(response) = input.response {
            query = query.bind(response);
        }
        if let Some(error) = input.error_message {
            query = query.bind(error);
        }
        if let Some(files) = input.files_changed {
            query = query.bind(files);
        }
        if let Some(added) = input.lines_added {
            query = query.bind(added);
        }
        if let Some(removed) = input.lines_removed {
            query = query.bind(removed);
        }
        if let Some(created) = input.files_created {
            query = query.bind(serde_json::to_string(&created).unwrap());
        }
        if let Some(modified) = input.files_modified {
            query = query.bind(serde_json::to_string(&modified).unwrap());
        }
        if let Some(deleted) = input.files_deleted {
            query = query.bind(serde_json::to_string(&deleted).unwrap());
        }
        if let Some(branch) = input.branch_name {
            query = query.bind(branch);
        }
        if let Some(commit) = input.commit_hash {
            query = query.bind(commit);
        }
        if let Some(msg) = input.commit_message {
            query = query.bind(msg);
        }
        if let Some(pr_num) = input.pr_number {
            query = query.bind(pr_num);
        }
        if let Some(pr_url) = input.pr_url {
            query = query.bind(pr_url);
        }
        if let Some(pr_title) = input.pr_title {
            query = query.bind(pr_title);
        }
        if let Some(pr_status) = input.pr_status {
            query = query.bind(pr_status);
        }
        if let Some(pr_created) = input.pr_created_at {
            query = query.bind(pr_created);
        }
        if let Some(pr_merged) = input.pr_merged_at {
            query = query.bind(pr_merged);
        }
        if let Some(merge_commit) = input.pr_merge_commit {
            query = query.bind(merge_commit);
        }
        if let Some(review) = input.review_status {
            query = query.bind(review);
        }
        if let Some(comments) = input.review_comments {
            query = query.bind(comments);
        }
        if let Some(tests) = input.test_results {
            query = query.bind(serde_json::to_string(&tests).unwrap());
        }
        if let Some(metrics) = input.performance_metrics {
            query = query.bind(serde_json::to_string(&metrics).unwrap());
        }
        if let Some(meta) = input.metadata {
            query = query.bind(serde_json::to_string(&meta).unwrap());
        }

        query = query.bind(execution_id);

        query
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.get_execution(execution_id).await
    }

    /// Delete an execution
    pub async fn delete_execution(&self, execution_id: &str) -> Result<(), StorageError> {
        debug!("Deleting execution: {}", execution_id);

        sqlx::query("DELETE FROM agent_executions WHERE id = ?")
            .bind(execution_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    // ==================== PR Reviews ====================

    /// List all reviews for an execution
    pub async fn list_reviews(&self, execution_id: &str) -> Result<Vec<PrReview>, StorageError> {
        debug!("Fetching reviews for execution: {}", execution_id);

        let rows = sqlx::query(
            "SELECT * FROM pr_reviews WHERE execution_id = ? ORDER BY reviewed_at DESC",
        )
        .bind(execution_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        rows.iter()
            .map(|row| self.row_to_review(row))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Get a single review by ID
    pub async fn get_review(&self, review_id: &str) -> Result<PrReview, StorageError> {
        debug!("Fetching review: {}", review_id);

        let row = sqlx::query("SELECT * FROM pr_reviews WHERE id = ?")
            .bind(review_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_review(&row)
    }

    /// Create a new review
    pub async fn create_review(
        &self,
        input: PrReviewCreateInput,
    ) -> Result<PrReview, StorageError> {
        let review_id = format!("review-{}", nanoid::nanoid!());
        let now = Utc::now();

        debug!(
            "Creating review: {} for execution: {}",
            review_id, input.execution_id
        );

        sqlx::query(
            r#"
            INSERT INTO pr_reviews (
                id, execution_id, reviewer_id, reviewer_type, review_status,
                review_body, comments, suggested_changes, reviewed_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&review_id)
        .bind(&input.execution_id)
        .bind(&input.reviewer_id)
        .bind(&input.reviewer_type)
        .bind(&input.review_status)
        .bind(&input.review_body)
        .bind(
            input
                .comments
                .as_ref()
                .map(|c| serde_json::to_string(c).unwrap()),
        )
        .bind(
            input
                .suggested_changes
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap()),
        )
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        self.get_review(&review_id).await
    }

    /// Update a review
    pub async fn update_review(
        &self,
        review_id: &str,
        input: PrReviewUpdateInput,
    ) -> Result<PrReview, StorageError> {
        debug!("Updating review: {}", review_id);

        let now = Utc::now();
        let mut updates = vec!["updated_at = ?"];

        if input.review_status.is_some() {
            updates.push("review_status = ?");
        }
        if input.review_body.is_some() {
            updates.push("review_body = ?");
        }
        if input.comments.is_some() {
            updates.push("comments = ?");
        }
        if input.suggested_changes.is_some() {
            updates.push("suggested_changes = ?");
        }
        if input.approval_date.is_some() {
            updates.push("approval_date = ?");
        }
        if input.dismissal_reason.is_some() {
            updates.push("dismissal_reason = ?");
        }

        let query_str = format!("UPDATE pr_reviews SET {} WHERE id = ?", updates.join(", "));
        let mut query = sqlx::query(&query_str).bind(now);

        if let Some(status) = input.review_status {
            query = query.bind(status);
        }
        if let Some(body) = input.review_body {
            query = query.bind(body);
        }
        if let Some(comments) = input.comments {
            query = query.bind(serde_json::to_string(&comments).unwrap());
        }
        if let Some(changes) = input.suggested_changes {
            query = query.bind(serde_json::to_string(&changes).unwrap());
        }
        if let Some(approval) = input.approval_date {
            query = query.bind(approval);
        }
        if let Some(dismissal) = input.dismissal_reason {
            query = query.bind(dismissal);
        }

        query = query.bind(review_id);

        query
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.get_review(review_id).await
    }

    /// Delete a review
    pub async fn delete_review(&self, review_id: &str) -> Result<(), StorageError> {
        debug!("Deleting review: {}", review_id);

        sqlx::query("DELETE FROM pr_reviews WHERE id = ?")
            .bind(review_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    // ==================== Helper Methods ====================

    fn row_to_execution(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<AgentExecution, StorageError> {
        let files_created: Option<String> =
            row.try_get("files_created").map_err(StorageError::Sqlx)?;
        let files_modified: Option<String> =
            row.try_get("files_modified").map_err(StorageError::Sqlx)?;
        let files_deleted: Option<String> =
            row.try_get("files_deleted").map_err(StorageError::Sqlx)?;
        let test_results: Option<String> =
            row.try_get("test_results").map_err(StorageError::Sqlx)?;
        let performance_metrics: Option<String> = row
            .try_get("performance_metrics")
            .map_err(StorageError::Sqlx)?;
        let metadata: Option<String> = row.try_get("metadata").map_err(StorageError::Sqlx)?;

        Ok(AgentExecution {
            id: row.try_get("id").map_err(StorageError::Sqlx)?,
            task_id: row.try_get("task_id").map_err(StorageError::Sqlx)?,
            agent_id: row.try_get("agent_id").map_err(StorageError::Sqlx)?,
            model: row.try_get("model").map_err(StorageError::Sqlx)?,
            started_at: row.try_get("started_at").map_err(StorageError::Sqlx)?,
            completed_at: row.try_get("completed_at").map_err(StorageError::Sqlx)?,
            status: row.try_get("status").map_err(StorageError::Sqlx)?,
            execution_time_seconds: row
                .try_get("execution_time_seconds")
                .map_err(StorageError::Sqlx)?,
            tokens_input: row.try_get("tokens_input").map_err(StorageError::Sqlx)?,
            tokens_output: row.try_get("tokens_output").map_err(StorageError::Sqlx)?,
            total_cost: row.try_get("total_cost").map_err(StorageError::Sqlx)?,
            prompt: row.try_get("prompt").map_err(StorageError::Sqlx)?,
            response: row.try_get("response").map_err(StorageError::Sqlx)?,
            error_message: row.try_get("error_message").map_err(StorageError::Sqlx)?,
            retry_attempt: row.try_get("retry_attempt").map_err(StorageError::Sqlx)?,
            files_changed: row.try_get("files_changed").map_err(StorageError::Sqlx)?,
            lines_added: row.try_get("lines_added").map_err(StorageError::Sqlx)?,
            lines_removed: row.try_get("lines_removed").map_err(StorageError::Sqlx)?,
            files_created: files_created.and_then(|s| serde_json::from_str(&s).ok()),
            files_modified: files_modified.and_then(|s| serde_json::from_str(&s).ok()),
            files_deleted: files_deleted.and_then(|s| serde_json::from_str(&s).ok()),
            branch_name: row.try_get("branch_name").map_err(StorageError::Sqlx)?,
            commit_hash: row.try_get("commit_hash").map_err(StorageError::Sqlx)?,
            commit_message: row.try_get("commit_message").map_err(StorageError::Sqlx)?,
            pr_number: row.try_get("pr_number").map_err(StorageError::Sqlx)?,
            pr_url: row.try_get("pr_url").map_err(StorageError::Sqlx)?,
            pr_title: row.try_get("pr_title").map_err(StorageError::Sqlx)?,
            pr_status: row.try_get("pr_status").map_err(StorageError::Sqlx)?,
            pr_created_at: row.try_get("pr_created_at").map_err(StorageError::Sqlx)?,
            pr_merged_at: row.try_get("pr_merged_at").map_err(StorageError::Sqlx)?,
            pr_merge_commit: row.try_get("pr_merge_commit").map_err(StorageError::Sqlx)?,
            review_status: row.try_get("review_status").map_err(StorageError::Sqlx)?,
            review_comments: row.try_get("review_comments").map_err(StorageError::Sqlx)?,
            test_results: test_results.and_then(|s| serde_json::from_str(&s).ok()),
            performance_metrics: performance_metrics.and_then(|s| serde_json::from_str(&s).ok()),
            metadata: metadata.and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at").map_err(StorageError::Sqlx)?,
            updated_at: row.try_get("updated_at").map_err(StorageError::Sqlx)?,
        })
    }

    fn row_to_review(&self, row: &sqlx::sqlite::SqliteRow) -> Result<PrReview, StorageError> {
        let comments: Option<String> = row.try_get("comments").map_err(StorageError::Sqlx)?;
        let suggested_changes: Option<String> = row
            .try_get("suggested_changes")
            .map_err(StorageError::Sqlx)?;

        Ok(PrReview {
            id: row.try_get("id").map_err(StorageError::Sqlx)?,
            execution_id: row.try_get("execution_id").map_err(StorageError::Sqlx)?,
            reviewer_id: row.try_get("reviewer_id").map_err(StorageError::Sqlx)?,
            reviewer_type: row.try_get("reviewer_type").map_err(StorageError::Sqlx)?,
            review_status: row.try_get("review_status").map_err(StorageError::Sqlx)?,
            review_body: row.try_get("review_body").map_err(StorageError::Sqlx)?,
            comments: comments.and_then(|s| serde_json::from_str(&s).ok()),
            suggested_changes: suggested_changes.and_then(|s| serde_json::from_str(&s).ok()),
            approval_date: row.try_get("approval_date").map_err(StorageError::Sqlx)?,
            dismissal_reason: row
                .try_get("dismissal_reason")
                .map_err(StorageError::Sqlx)?,
            reviewed_at: row.try_get("reviewed_at").map_err(StorageError::Sqlx)?,
            created_at: row.try_get("created_at").map_err(StorageError::Sqlx)?,
            updated_at: row.try_get("updated_at").map_err(StorageError::Sqlx)?,
        })
    }
}
