// ABOUTME: Execution tracking for Epics and Tasks with checkpoint system
// ABOUTME: Provides append-only progress tracking and logical checkpoint management

use chrono::{DateTime, Utc};
use orkee_storage::StorageError as StoreError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Checkpoint type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CheckpointType {
    /// User reviews completed work
    Review,
    /// Run test suite
    Test,
    /// Verify integration points
    Integration,
    /// Stakeholder approval needed
    Approval,
}

/// Execution checkpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub id: String,
    pub epic_id: String,
    pub after_task_id: String,
    pub checkpoint_type: CheckpointType,
    pub message: String,
    pub required_validation: Vec<String>,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCheckpointInput {
    pub epic_id: String,
    pub after_task_id: String,
    pub checkpoint_type: CheckpointType,
    pub message: String,
    pub required_validation: Vec<String>,
}

/// Validation entry type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ValidationEntryType {
    /// Work completed
    Progress,
    /// Problem encountered
    Issue,
    /// Technical decision made
    Decision,
    /// Checkpoint reached
    Checkpoint,
}

/// Validation entry for append-only progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationEntry {
    pub id: String,
    pub task_id: String,
    pub timestamp: DateTime<Utc>,
    pub entry_type: ValidationEntryType,
    pub content: String,
    pub author: String,
}

/// Input for appending progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendProgressInput {
    pub task_id: String,
    pub entry_type: ValidationEntryType,
    pub content: String,
    pub author: String,
}

/// Execution tracker service
pub struct ExecutionTracker {
    pool: SqlitePool,
}

impl ExecutionTracker {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Generate logical checkpoints for an epic based on task structure
    pub async fn generate_checkpoints(
        &self,
        epic_id: &str,
    ) -> Result<Vec<ExecutionCheckpoint>, StoreError> {
        // Get all tasks for the epic
        let tasks = self.get_epic_tasks(epic_id).await?;

        let mut checkpoints = Vec::new();
        let total_tasks = tasks.len();

        // Create checkpoints at logical boundaries
        // - After every 3 tasks
        // - After each category boundary
        // - At the end

        let mut category_boundaries = std::collections::HashMap::new();
        let mut last_category: Option<String> = None;

        for (idx, task) in tasks.iter().enumerate() {
            let task_num = idx + 1;

            // Check for category change (logical boundary)
            if let Some(ref category) = task.category {
                if last_category.as_ref() != Some(category) {
                    if let Some(ref last_cat) = last_category {
                        // Category boundary checkpoint
                        if let Some(prev_task) = tasks.get(idx.saturating_sub(1)) {
                            checkpoints.push(self.create_checkpoint_for_boundary(
                                epic_id,
                                &prev_task.id,
                                last_cat,
                                CheckpointType::Review,
                            )?);
                        }
                    }
                    last_category = Some(category.clone());
                    category_boundaries.insert(category.clone(), idx);
                }
            }

            // Every 3 tasks checkpoint
            if task_num % 3 == 0 && task_num < total_tasks {
                checkpoints.push(ExecutionCheckpoint {
                    id: nanoid::nanoid!(),
                    epic_id: epic_id.to_string(),
                    after_task_id: task.id.clone(),
                    checkpoint_type: CheckpointType::Test,
                    message: format!(
                        "Completed {} of {} tasks. Run tests before continuing?",
                        task_num, total_tasks
                    ),
                    required_validation: vec![
                        "All tests pass".to_string(),
                        "No regressions detected".to_string(),
                    ],
                    completed: false,
                    completed_at: None,
                    created_at: Utc::now(),
                });
            }
        }

        // Final checkpoint
        if let Some(last_task) = tasks.last() {
            checkpoints.push(ExecutionCheckpoint {
                id: nanoid::nanoid!(),
                epic_id: epic_id.to_string(),
                after_task_id: last_task.id.clone(),
                checkpoint_type: CheckpointType::Integration,
                message: "All tasks complete. Ready for final integration testing?".to_string(),
                required_validation: vec![
                    "Integration tests pass".to_string(),
                    "Documentation updated".to_string(),
                    "Success criteria met".to_string(),
                ],
                completed: false,
                completed_at: None,
                created_at: Utc::now(),
            });
        }

        // Save checkpoints to database
        for checkpoint in &checkpoints {
            self.save_checkpoint(checkpoint).await?;
        }

        Ok(checkpoints)
    }

    /// Create a checkpoint for a category boundary
    fn create_checkpoint_for_boundary(
        &self,
        epic_id: &str,
        after_task_id: &str,
        category: &str,
        checkpoint_type: CheckpointType,
    ) -> Result<ExecutionCheckpoint, StoreError> {
        Ok(ExecutionCheckpoint {
            id: nanoid::nanoid!(),
            epic_id: epic_id.to_string(),
            after_task_id: after_task_id.to_string(),
            checkpoint_type,
            message: format!("{} category complete. Review before continuing?", category),
            required_validation: vec![
                format!("{} functionality works as expected", category),
                "Tests pass for this category".to_string(),
            ],
            completed: false,
            completed_at: None,
            created_at: Utc::now(),
        })
    }

    /// Create a manual checkpoint
    pub async fn create_checkpoint(
        &self,
        input: CreateCheckpointInput,
    ) -> Result<ExecutionCheckpoint, StoreError> {
        let checkpoint = ExecutionCheckpoint {
            id: nanoid::nanoid!(),
            epic_id: input.epic_id,
            after_task_id: input.after_task_id,
            checkpoint_type: input.checkpoint_type,
            message: input.message,
            required_validation: input.required_validation,
            completed: false,
            completed_at: None,
            created_at: Utc::now(),
        };

        self.save_checkpoint(&checkpoint).await?;
        Ok(checkpoint)
    }

    /// Mark a checkpoint as completed
    pub async fn complete_checkpoint(
        &self,
        checkpoint_id: &str,
    ) -> Result<ExecutionCheckpoint, StoreError> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE execution_checkpoints SET completed = ?, completed_at = ? WHERE id = ?",
        )
        .bind(true)
        .bind(now)
        .bind(checkpoint_id)
        .execute(&self.pool)
        .await
        .map_err(StoreError::Sqlx)?;

        self.get_checkpoint(checkpoint_id).await
    }

    /// Get all checkpoints for an epic
    pub async fn get_epic_checkpoints(
        &self,
        epic_id: &str,
    ) -> Result<Vec<ExecutionCheckpoint>, StoreError> {
        let rows = sqlx::query(
            "SELECT * FROM execution_checkpoints WHERE epic_id = ? ORDER BY created_at",
        )
        .bind(epic_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::Sqlx)?;

        rows.iter()
            .map(|row| self.row_to_checkpoint(row))
            .collect()
    }

    /// Get checkpoint by ID
    async fn get_checkpoint(&self, checkpoint_id: &str) -> Result<ExecutionCheckpoint, StoreError> {
        let row = sqlx::query("SELECT * FROM execution_checkpoints WHERE id = ?")
            .bind(checkpoint_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StoreError::Sqlx)?;

        self.row_to_checkpoint(&row)
    }

    /// Append progress to a task (never overwrites, only appends)
    pub async fn append_progress(
        &self,
        input: AppendProgressInput,
    ) -> Result<ValidationEntry, StoreError> {
        let entry = ValidationEntry {
            id: nanoid::nanoid!(),
            task_id: input.task_id.clone(),
            timestamp: Utc::now(),
            entry_type: input.entry_type,
            content: input.content,
            author: input.author,
        };

        // Save validation entry to database
        sqlx::query(
            r#"
            INSERT INTO validation_entries (id, task_id, timestamp, entry_type, content, author)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.task_id)
        .bind(entry.timestamp)
        .bind(entry.entry_type)
        .bind(&entry.content)
        .bind(&entry.author)
        .execute(&self.pool)
        .await
        .map_err(StoreError::Sqlx)?;

        // Also update the task's validation_history JSON field
        self.update_task_validation_history(&entry).await?;

        Ok(entry)
    }

    /// Get validation history for a task
    pub async fn get_task_validation_history(
        &self,
        task_id: &str,
    ) -> Result<Vec<ValidationEntry>, StoreError> {
        let rows = sqlx::query(
            "SELECT * FROM validation_entries WHERE task_id = ? ORDER BY timestamp",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::Sqlx)?;

        rows.iter()
            .map(|row| self.row_to_validation_entry(row))
            .collect()
    }

    // Private helper methods

    async fn save_checkpoint(
        &self,
        checkpoint: &ExecutionCheckpoint,
    ) -> Result<(), StoreError> {
        let required_validation_json = serde_json::to_string(&checkpoint.required_validation)?;

        sqlx::query(
            r#"
            INSERT INTO execution_checkpoints (
                id, epic_id, after_task_id, checkpoint_type, message,
                required_validation, completed, completed_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&checkpoint.id)
        .bind(&checkpoint.epic_id)
        .bind(&checkpoint.after_task_id)
        .bind(checkpoint.checkpoint_type)
        .bind(&checkpoint.message)
        .bind(required_validation_json)
        .bind(checkpoint.completed)
        .bind(checkpoint.completed_at)
        .bind(checkpoint.created_at)
        .execute(&self.pool)
        .await
        .map_err(StoreError::Sqlx)?;

        Ok(())
    }

    async fn update_task_validation_history(
        &self,
        entry: &ValidationEntry,
    ) -> Result<(), StoreError> {
        // Get current validation history
        let current_history = self.get_task_validation_history(&entry.task_id).await?;

        // Append new entry (immutable - never overwrite)
        let mut all_entries = current_history;
        all_entries.push(entry.clone());

        let history_json = serde_json::to_string(&all_entries)?;

        sqlx::query("UPDATE tasks SET validation_history = ?, updated_at = ? WHERE id = ?")
            .bind(history_json)
            .bind(Utc::now())
            .bind(&entry.task_id)
            .execute(&self.pool)
            .await
            .map_err(StoreError::Sqlx)?;

        Ok(())
    }

    async fn get_epic_tasks(
        &self,
        epic_id: &str,
    ) -> Result<Vec<orkee_tasks::types::Task>, StoreError> {
        let rows =
            sqlx::query("SELECT * FROM tasks WHERE epic_id = ? ORDER BY position, created_at")
                .bind(epic_id)
                .fetch_all(&self.pool)
                .await
                .map_err(StoreError::Sqlx)?;

        use crate::task_decomposer::storage::row_to_task_result;
        rows.iter().map(row_to_task_result).collect()
    }

    fn row_to_checkpoint(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<ExecutionCheckpoint, StoreError> {
        use sqlx::Row;

        let required_validation_str: Option<String> = row.try_get("required_validation")?;
        let required_validation = required_validation_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Ok(ExecutionCheckpoint {
            id: row.try_get("id")?,
            epic_id: row.try_get("epic_id")?,
            after_task_id: row.try_get("after_task_id")?,
            checkpoint_type: row.try_get("checkpoint_type")?,
            message: row.try_get("message")?,
            required_validation,
            completed: row.try_get("completed")?,
            completed_at: row.try_get("completed_at")?,
            created_at: row.try_get("created_at")?,
        })
    }

    fn row_to_validation_entry(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<ValidationEntry, StoreError> {
        use sqlx::Row;

        Ok(ValidationEntry {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            timestamp: row.try_get("timestamp")?,
            entry_type: row.try_get("entry_type")?,
            content: row.try_get("content")?,
            author: row.try_get("author")?,
        })
    }
}
