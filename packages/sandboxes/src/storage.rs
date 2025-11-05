// ABOUTME: Database storage operations for execution logs and artifacts
// ABOUTME: Provides CRUD interface for execution tracking and artifact management

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;

use crate::error::{Result, SandboxError};
use crate::types::{Artifact, LogEntry};

/// Storage layer for execution logs and artifacts
#[derive(Clone)]
pub struct ExecutionStorage {
    pool: Arc<SqlitePool>,
}

impl ExecutionStorage {
    /// Create a new execution storage instance
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    // ==================== Log Operations ====================

    /// Insert a log entry into the database
    pub async fn insert_log(&self, log: LogEntry) -> Result<()> {
        let metadata_json = log
            .metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()
            .map_err(SandboxError::Json)?;

        sqlx::query(
            r#"
            INSERT INTO execution_logs (
                id, execution_id, timestamp, log_level, message,
                source, metadata, stack_trace, sequence_number
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&log.id)
        .bind(&log.execution_id)
        .bind(&log.timestamp)
        .bind(&log.log_level)
        .bind(&log.message)
        .bind(&log.source)
        .bind(metadata_json)
        .bind(&log.stack_trace)
        .bind(log.sequence_number)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// Insert multiple log entries in a batch
    pub async fn insert_logs_batch(&self, logs: Vec<LogEntry>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for log in logs {
            let metadata_json = log
                .metadata
                .map(|m| serde_json::to_string(&m))
                .transpose()?;

            sqlx::query(
                r#"
                INSERT INTO execution_logs (
                    id, execution_id, timestamp, log_level, message,
                    source, metadata, stack_trace, sequence_number
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&log.id)
            .bind(&log.execution_id)
            .bind(&log.timestamp)
            .bind(&log.log_level)
            .bind(&log.message)
            .bind(&log.source)
            .bind(metadata_json)
            .bind(&log.stack_trace)
            .bind(log.sequence_number)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Get logs for an execution with pagination
    pub async fn get_logs(
        &self,
        execution_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<(Vec<LogEntry>, i64)> {
        // Get total count
        let total: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM execution_logs WHERE execution_id = ?")
                .bind(execution_id)
                .fetch_one(&*self.pool)
                .await?;

        // Get logs with pagination
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"
            SELECT id, execution_id, timestamp, log_level, message,
                   source, metadata, stack_trace, sequence_number, created_at
            FROM execution_logs
            WHERE execution_id = ?
            ORDER BY sequence_number ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(execution_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|row| {
                let metadata_str: Option<String> = row.get("metadata");
                let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

                LogEntry {
                    id: row.get("id"),
                    execution_id: row.get("execution_id"),
                    timestamp: row.get("timestamp"),
                    log_level: row.get("log_level"),
                    message: row.get("message"),
                    source: row.get("source"),
                    metadata,
                    stack_trace: row.get("stack_trace"),
                    sequence_number: row.get("sequence_number"),
                }
            })
            .collect();

        Ok((logs, total))
    }

    /// Search logs with filters
    pub async fn search_logs(
        &self,
        execution_id: &str,
        log_level: Option<&str>,
        source: Option<&str>,
        search_text: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<(Vec<LogEntry>, i64)> {
        let mut conditions = vec!["execution_id = ?".to_string()];
        let mut params: Vec<&str> = vec![execution_id];

        if let Some(level) = log_level {
            conditions.push("log_level = ?".to_string());
            params.push(level);
        }

        if let Some(src) = source {
            conditions.push("source = ?".to_string());
            params.push(src);
        }

        if let Some(text) = search_text {
            conditions.push("message LIKE ?".to_string());
            params.push(text);
        }

        let where_clause = conditions.join(" AND ");
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        // Build count query
        let count_query = format!("SELECT COUNT(*) FROM execution_logs WHERE {}", where_clause);

        // Build select query
        let select_query = format!(
            r#"
            SELECT id, execution_id, timestamp, log_level, message,
                   source, metadata, stack_trace, sequence_number, created_at
            FROM execution_logs
            WHERE {}
            ORDER BY sequence_number ASC
            LIMIT ? OFFSET ?
            "#,
            where_clause
        );

        // Execute count query
        let mut count_q = sqlx::query_scalar(&count_query);
        for param in &params {
            count_q = count_q.bind(param);
        }
        let total: i64 = count_q.fetch_one(&*self.pool).await?;

        // Execute select query
        let mut select_q = sqlx::query(&select_query);
        for param in &params {
            select_q = select_q.bind(param);
        }
        select_q = select_q.bind(limit).bind(offset);

        let rows = select_q.fetch_all(&*self.pool).await?;

        let logs = rows
            .into_iter()
            .map(|row| {
                let metadata_str: Option<String> = row.get("metadata");
                let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

                LogEntry {
                    id: row.get("id"),
                    execution_id: row.get("execution_id"),
                    timestamp: row.get("timestamp"),
                    log_level: row.get("log_level"),
                    message: row.get("message"),
                    source: row.get("source"),
                    metadata,
                    stack_trace: row.get("stack_trace"),
                    sequence_number: row.get("sequence_number"),
                }
            })
            .collect();

        Ok((logs, total))
    }

    // ==================== Artifact Operations ====================

    /// Create a new artifact record
    pub async fn create_artifact(&self, artifact: Artifact) -> Result<Artifact> {
        let metadata_json = artifact
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO execution_artifacts (
                id, execution_id, artifact_type, file_path, file_name,
                file_size_bytes, mime_type, stored_path, storage_backend,
                description, metadata, checksum
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&artifact.id)
        .bind(&artifact.execution_id)
        .bind(&artifact.artifact_type)
        .bind(&artifact.file_path)
        .bind(&artifact.file_name)
        .bind(artifact.file_size_bytes)
        .bind(&artifact.mime_type)
        .bind(&artifact.stored_path)
        .bind(&artifact.storage_backend)
        .bind(&artifact.description)
        .bind(metadata_json)
        .bind(&artifact.checksum)
        .execute(&*self.pool)
        .await?;

        Ok(artifact)
    }

    /// List artifacts for an execution
    pub async fn list_artifacts(&self, execution_id: &str) -> Result<Vec<Artifact>> {
        let rows = sqlx::query(
            r#"
            SELECT id, execution_id, artifact_type, file_path, file_name,
                   file_size_bytes, mime_type, stored_path, storage_backend,
                   description, metadata, checksum, created_at
            FROM execution_artifacts
            WHERE execution_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(execution_id)
        .fetch_all(&*self.pool)
        .await?;

        let artifacts = rows
            .into_iter()
            .map(|row| {
                let metadata_str: Option<String> = row.get("metadata");
                let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

                Artifact {
                    id: row.get("id"),
                    execution_id: row.get("execution_id"),
                    artifact_type: row.get("artifact_type"),
                    file_path: row.get("file_path"),
                    file_name: row.get("file_name"),
                    file_size_bytes: row.get("file_size_bytes"),
                    mime_type: row.get("mime_type"),
                    stored_path: row.get("stored_path"),
                    storage_backend: row.get("storage_backend"),
                    description: row.get("description"),
                    metadata,
                    checksum: row.get("checksum"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(artifacts)
    }

    /// Get a single artifact by ID
    pub async fn get_artifact(&self, artifact_id: &str) -> Result<Artifact> {
        let row = sqlx::query(
            r#"
            SELECT id, execution_id, artifact_type, file_path, file_name,
                   file_size_bytes, mime_type, stored_path, storage_backend,
                   description, metadata, checksum, created_at
            FROM execution_artifacts
            WHERE id = ?
            "#,
        )
        .bind(artifact_id)
        .fetch_optional(&*self.pool)
        .await?
        .ok_or_else(|| SandboxError::ArtifactNotFound(artifact_id.to_string()))?;

        let metadata_str: Option<String> = row.get("metadata");
        let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Artifact {
            id: row.get("id"),
            execution_id: row.get("execution_id"),
            artifact_type: row.get("artifact_type"),
            file_path: row.get("file_path"),
            file_name: row.get("file_name"),
            file_size_bytes: row.get("file_size_bytes"),
            mime_type: row.get("mime_type"),
            stored_path: row.get("stored_path"),
            storage_backend: row.get("storage_backend"),
            description: row.get("description"),
            metadata,
            checksum: row.get("checksum"),
            created_at: row.get("created_at"),
        })
    }

    /// Delete an artifact
    pub async fn delete_artifact(&self, artifact_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM execution_artifacts WHERE id = ?")
            .bind(artifact_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    // ==================== Execution Status Updates ====================

    /// Update container status for an execution
    pub async fn update_container_status(
        &self,
        execution_id: &str,
        container_id: &str,
        container_status: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agent_executions
            SET container_id = ?, container_status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(container_id)
        .bind(container_status)
        .bind(Utc::now().to_rfc3339())
        .bind(execution_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// Update execution status
    pub async fn update_execution_status(
        &self,
        execution_id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agent_executions
            SET status = ?, error_message = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(error_message)
        .bind(Utc::now().to_rfc3339())
        .bind(execution_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// Update resource usage for an execution
    pub async fn update_resource_usage(
        &self,
        execution_id: &str,
        memory_used_mb: u64,
        cpu_usage_percent: f64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agent_executions
            SET memory_used_mb = ?, cpu_usage_percent = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(memory_used_mb as i64)
        .bind(cpu_usage_percent)
        .bind(Utc::now().to_rfc3339())
        .bind(execution_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// Store Vibekit session ID for an execution
    pub async fn update_vibekit_session(
        &self,
        execution_id: &str,
        vibekit_session_id: &str,
        vibekit_version: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agent_executions
            SET vibekit_session_id = ?, vibekit_version = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(vibekit_session_id)
        .bind(vibekit_version)
        .bind(Utc::now().to_rfc3339())
        .bind(execution_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_pool() -> SqlitePool {
        SqlitePool::connect(":memory:")
            .await
            .expect("Failed to create test database")
    }

    #[tokio::test]
    async fn test_log_operations() {
        let pool = Arc::new(create_test_pool().await);
        let storage = ExecutionStorage::new(pool);

        // Note: This test would need the full schema to run
        // For now, it's a placeholder showing the intended API
    }

    #[tokio::test]
    async fn test_artifact_operations() {
        let pool = Arc::new(create_test_pool().await);
        let storage = ExecutionStorage::new(pool);

        // Note: This test would need the full schema to run
        // For now, it's a placeholder showing the intended API
    }
}
