// ABOUTME: AI usage log storage layer using SQLite
// ABOUTME: Handles querying AI usage logs with filtering and aggregation

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use tracing::debug;

use super::types::{
    AiUsageLog, AiUsageQuery, AiUsageStats, ModelStats, OperationStats, ProviderStats,
};
use storage::StorageError;

pub struct AiUsageLogStorage {
    pool: SqlitePool,
}

impl AiUsageLogStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new AI usage log entry
    pub async fn create_log(&self, log: &AiUsageLog) -> Result<AiUsageLog, StorageError> {
        let created_at_str = log.created_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO ai_usage_logs (
                id, project_id, request_id, operation, model, provider,
                input_tokens, output_tokens, total_tokens, estimated_cost,
                duration_ms, error, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&log.id)
        .bind(&log.project_id)
        .bind(&log.request_id)
        .bind(&log.operation)
        .bind(&log.model)
        .bind(&log.provider)
        .bind(log.input_tokens)
        .bind(log.output_tokens)
        .bind(log.total_tokens)
        .bind(log.estimated_cost)
        .bind(log.duration_ms)
        .bind(&log.error)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(log.clone())
    }

    /// List AI usage logs with optional filtering
    pub async fn list_logs(&self, query: AiUsageQuery) -> Result<Vec<AiUsageLog>, StorageError> {
        let mut sql = String::from("SELECT * FROM ai_usage_logs WHERE 1=1");
        let mut conditions = Vec::new();

        if query.project_id.is_some() {
            conditions.push("project_id = ?");
        }
        if query.start_date.is_some() {
            conditions.push("created_at >= ?");
        }
        if query.end_date.is_some() {
            conditions.push("created_at <= ?");
        }
        if query.operation.is_some() {
            conditions.push("operation = ?");
        }
        if query.model.is_some() {
            conditions.push("model = ?");
        }
        if query.provider.is_some() {
            conditions.push("provider = ?");
        }

        for condition in conditions {
            sql.push_str(&format!(" AND {}", condition));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        } else {
            sql.push_str(" LIMIT 100"); // Default limit
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        debug!("Fetching AI usage logs with query: {}", sql);

        let mut db_query = sqlx::query(&sql);

        // Bind parameters in the same order as conditions
        if let Some(project_id) = &query.project_id {
            db_query = db_query.bind(project_id);
        }
        if let Some(start_date) = &query.start_date {
            db_query = db_query.bind(start_date);
        }
        if let Some(end_date) = &query.end_date {
            db_query = db_query.bind(end_date);
        }
        if let Some(operation) = &query.operation {
            db_query = db_query.bind(operation);
        }
        if let Some(model) = &query.model {
            db_query = db_query.bind(model);
        }
        if let Some(provider) = &query.provider {
            db_query = db_query.bind(provider);
        }

        let rows = db_query
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let logs = rows
            .iter()
            .map(|row| self.row_to_log(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Get aggregate statistics for AI usage
    pub async fn get_stats(&self, query: AiUsageQuery) -> Result<AiUsageStats, StorageError> {
        let mut where_conditions = Vec::new();
        let mut bind_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> = Vec::new();

        if let Some(project_id) = &query.project_id {
            where_conditions.push("project_id = ?");
            bind_values.push(Box::new(project_id.clone()));
        }
        if let Some(start_date) = &query.start_date {
            where_conditions.push("created_at >= ?");
            bind_values.push(Box::new(*start_date));
        }
        if let Some(end_date) = &query.end_date {
            where_conditions.push("created_at <= ?");
            bind_values.push(Box::new(*end_date));
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Get overall statistics
        let overall_sql = format!(
            r#"
            SELECT
                COUNT(*) as total_requests,
                SUM(CASE WHEN error IS NULL THEN 1 ELSE 0 END) as successful_requests,
                SUM(CASE WHEN error IS NOT NULL THEN 1 ELSE 0 END) as failed_requests,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COALESCE(SUM(estimated_cost), 0.0) as total_cost,
                COALESCE(AVG(duration_ms), 0.0) as average_duration_ms
            FROM ai_usage_logs
            {}
            "#,
            where_clause
        );

        debug!("Fetching overall stats with query: {}", overall_sql);

        let mut overall_query = sqlx::query(&overall_sql);
        if let Some(project_id) = &query.project_id {
            overall_query = overall_query.bind(project_id);
        }
        if let Some(start_date) = &query.start_date {
            overall_query = overall_query.bind(start_date);
        }
        if let Some(end_date) = &query.end_date {
            overall_query = overall_query.bind(end_date);
        }

        let row = overall_query
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let total_requests: i64 = row.try_get("total_requests").unwrap_or(0);
        let successful_requests: i64 = row.try_get("successful_requests").unwrap_or(0);
        let failed_requests: i64 = row.try_get("failed_requests").unwrap_or(0);
        let total_input_tokens: i64 = row.try_get("total_input_tokens").unwrap_or(0);
        let total_output_tokens: i64 = row.try_get("total_output_tokens").unwrap_or(0);
        let total_tokens: i64 = row.try_get("total_tokens").unwrap_or(0);
        let total_cost: f64 = row.try_get("total_cost").unwrap_or(0.0);
        let average_duration_ms: f64 = row.try_get("average_duration_ms").unwrap_or(0.0);

        // Get stats by operation
        let by_operation = self.get_operation_stats(&query).await?;

        // Get stats by model
        let by_model = self.get_model_stats(&query).await?;

        // Get stats by provider
        let by_provider = self.get_provider_stats(&query).await?;

        Ok(AiUsageStats {
            total_requests,
            successful_requests,
            failed_requests,
            total_input_tokens,
            total_output_tokens,
            total_tokens,
            total_cost,
            average_duration_ms,
            by_operation,
            by_model,
            by_provider,
        })
    }

    /// Get statistics grouped by operation
    async fn get_operation_stats(
        &self,
        query: &AiUsageQuery,
    ) -> Result<Vec<OperationStats>, StorageError> {
        let mut where_conditions = Vec::new();

        if query.project_id.is_some() {
            where_conditions.push("project_id = ?");
        }
        if query.start_date.is_some() {
            where_conditions.push("created_at >= ?");
        }
        if query.end_date.is_some() {
            where_conditions.push("created_at <= ?");
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT
                operation,
                COUNT(*) as count,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COALESCE(SUM(estimated_cost), 0.0) as total_cost
            FROM ai_usage_logs
            {}
            GROUP BY operation
            ORDER BY total_cost DESC
            "#,
            where_clause
        );

        let mut db_query = sqlx::query(&sql);
        if let Some(project_id) = &query.project_id {
            db_query = db_query.bind(project_id);
        }
        if let Some(start_date) = &query.start_date {
            db_query = db_query.bind(start_date);
        }
        if let Some(end_date) = &query.end_date {
            db_query = db_query.bind(end_date);
        }

        let rows = db_query
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let stats = rows
            .iter()
            .map(|row| OperationStats {
                operation: row.try_get("operation").unwrap_or_default(),
                count: row.try_get("count").unwrap_or(0),
                total_tokens: row.try_get("total_tokens").unwrap_or(0),
                total_cost: row.try_get("total_cost").unwrap_or(0.0),
            })
            .collect();

        Ok(stats)
    }

    /// Get statistics grouped by model
    async fn get_model_stats(&self, query: &AiUsageQuery) -> Result<Vec<ModelStats>, StorageError> {
        let mut where_conditions = Vec::new();

        if query.project_id.is_some() {
            where_conditions.push("project_id = ?");
        }
        if query.start_date.is_some() {
            where_conditions.push("created_at >= ?");
        }
        if query.end_date.is_some() {
            where_conditions.push("created_at <= ?");
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT
                model,
                COUNT(*) as count,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COALESCE(SUM(estimated_cost), 0.0) as total_cost
            FROM ai_usage_logs
            {}
            GROUP BY model
            ORDER BY total_cost DESC
            "#,
            where_clause
        );

        let mut db_query = sqlx::query(&sql);
        if let Some(project_id) = &query.project_id {
            db_query = db_query.bind(project_id);
        }
        if let Some(start_date) = &query.start_date {
            db_query = db_query.bind(start_date);
        }
        if let Some(end_date) = &query.end_date {
            db_query = db_query.bind(end_date);
        }

        let rows = db_query
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let stats = rows
            .iter()
            .map(|row| ModelStats {
                model: row.try_get("model").unwrap_or_default(),
                count: row.try_get("count").unwrap_or(0),
                total_tokens: row.try_get("total_tokens").unwrap_or(0),
                total_cost: row.try_get("total_cost").unwrap_or(0.0),
            })
            .collect();

        Ok(stats)
    }

    /// Get statistics grouped by provider
    async fn get_provider_stats(
        &self,
        query: &AiUsageQuery,
    ) -> Result<Vec<ProviderStats>, StorageError> {
        let mut where_conditions = Vec::new();

        if query.project_id.is_some() {
            where_conditions.push("project_id = ?");
        }
        if query.start_date.is_some() {
            where_conditions.push("created_at >= ?");
        }
        if query.end_date.is_some() {
            where_conditions.push("created_at <= ?");
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT
                provider,
                COUNT(*) as count,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COALESCE(SUM(estimated_cost), 0.0) as total_cost
            FROM ai_usage_logs
            {}
            GROUP BY provider
            ORDER BY total_cost DESC
            "#,
            where_clause
        );

        let mut db_query = sqlx::query(&sql);
        if let Some(project_id) = &query.project_id {
            db_query = db_query.bind(project_id);
        }
        if let Some(start_date) = &query.start_date {
            db_query = db_query.bind(start_date);
        }
        if let Some(end_date) = &query.end_date {
            db_query = db_query.bind(end_date);
        }

        let rows = db_query
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let stats = rows
            .iter()
            .map(|row| ProviderStats {
                provider: row.try_get("provider").unwrap_or_default(),
                count: row.try_get("count").unwrap_or(0),
                total_tokens: row.try_get("total_tokens").unwrap_or(0),
                total_cost: row.try_get("total_cost").unwrap_or(0.0),
            })
            .collect();

        Ok(stats)
    }

    /// Convert a database row to an AiUsageLog
    fn row_to_log(&self, row: &sqlx::sqlite::SqliteRow) -> Result<AiUsageLog, StorageError> {
        let created_at_str: String = row.try_get("created_at").map_err(StorageError::Sqlx)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| {
                StorageError::Database(format!("Failed to parse created_at timestamp: {}", e))
            })?
            .with_timezone(&Utc);

        Ok(AiUsageLog {
            id: row.try_get("id").map_err(StorageError::Sqlx)?,
            project_id: row.try_get("project_id").map_err(StorageError::Sqlx)?,
            request_id: row.try_get("request_id").map_err(StorageError::Sqlx)?,
            operation: row.try_get("operation").map_err(StorageError::Sqlx)?,
            model: row.try_get("model").map_err(StorageError::Sqlx)?,
            provider: row.try_get("provider").map_err(StorageError::Sqlx)?,
            input_tokens: row.try_get("input_tokens").map_err(StorageError::Sqlx)?,
            output_tokens: row.try_get("output_tokens").map_err(StorageError::Sqlx)?,
            total_tokens: row.try_get("total_tokens").map_err(StorageError::Sqlx)?,
            estimated_cost: row.try_get("estimated_cost").map_err(StorageError::Sqlx)?,
            duration_ms: row.try_get("duration_ms").map_err(StorageError::Sqlx)?,
            error: row.try_get("error").map_err(StorageError::Sqlx)?,
            created_at,
        })
    }
}
