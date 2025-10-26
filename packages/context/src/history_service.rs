use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub token_count: i32,
    pub file_count: i32,
    pub configuration_id: Option<String>,
    pub task_id: Option<String>,
    pub task_success: Option<bool>,
    pub files_included: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextStats {
    pub total_contexts_generated: i32,
    pub average_tokens: f64,
    pub success_rate: f64,
    pub most_used_files: Vec<FileUsage>,
    pub token_usage_over_time: Vec<TokenUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUsage {
    pub file: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub date: String,
    pub tokens: i32,
}

pub struct HistoryService {
    pool: SqlitePool,
}

impl HistoryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Save a context snapshot to the database
    pub async fn save_snapshot(&self, snapshot: &ContextSnapshot) -> Result<String, sqlx::Error> {
        let id = generate_id();
        let files_json =
            serde_json::to_string(&snapshot.files_included).unwrap_or_else(|_| "[]".to_string());

        sqlx::query!(
            r#"
            INSERT INTO context_snapshots
            (id, project_id, content, file_count, total_tokens, metadata, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
            "#,
            id,
            snapshot.project_id,
            snapshot.content,
            snapshot.file_count,
            snapshot.token_count,
            files_json
        )
        .execute(&self.pool)
        .await?;

        // Track file usage patterns
        for file in &snapshot.files_included {
            self.track_file_usage(&snapshot.project_id, file).await?;
        }

        Ok(id)
    }

    /// Track usage of a specific file in context generation
    async fn track_file_usage(&self, project_id: &str, file_path: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO context_usage_patterns (project_id, file_path, inclusion_count, last_used)
            VALUES (?1, ?2, 1, CURRENT_TIMESTAMP)
            ON CONFLICT(project_id, file_path)
            DO UPDATE SET
                inclusion_count = inclusion_count + 1,
                last_used = CURRENT_TIMESTAMP
            "#,
            project_id,
            file_path
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all snapshots for a project
    pub async fn get_snapshots(
        &self,
        project_id: &str,
        limit: i32,
    ) -> Result<Vec<ContextSnapshot>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, project_id, content, CAST(file_count AS INTEGER) as "file_count!: i32", 
                   CAST(total_tokens AS INTEGER) as "token_count!: i32", 
                   metadata, created_at
            FROM context_snapshots
            WHERE project_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
            project_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut snapshots = Vec::new();
        for row in rows {
            let files_included: Vec<String> =
                serde_json::from_str(&row.metadata).unwrap_or_default();

            let created_at = DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            snapshots.push(ContextSnapshot {
                id: row.id.unwrap_or_default(),
                project_id: row.project_id,
                content: row.content,
                token_count: row.token_count,
                file_count: row.file_count,
                configuration_id: None,
                task_id: None,
                task_success: None,
                files_included,
                created_at,
            });
        }

        Ok(snapshots)
    }

    /// Get a specific snapshot by ID
    pub async fn get_snapshot(
        &self,
        snapshot_id: &str,
    ) -> Result<Option<ContextSnapshot>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, project_id, content, CAST(file_count AS INTEGER) as "file_count!: i32", 
                   CAST(total_tokens AS INTEGER) as "token_count!: i32",
                   metadata, created_at
            FROM context_snapshots
            WHERE id = ?
            "#,
            snapshot_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let files_included: Vec<String> = serde_json::from_str(&r.metadata).unwrap_or_default();
            let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            ContextSnapshot {
                id: r.id.unwrap_or_default(),
                project_id: r.project_id,
                content: r.content,
                token_count: r.token_count,
                file_count: r.file_count,
                configuration_id: None,
                task_id: None,
                task_success: None,
                files_included,
                created_at,
            }
        }))
    }

    /// Get statistics for a project's context usage
    pub async fn get_stats(&self, project_id: &str) -> Result<ContextStats, sqlx::Error> {
        // Total contexts
        let total = sqlx::query_scalar!(
            "SELECT CAST(COUNT(*) AS INTEGER) FROM context_snapshots WHERE project_id = ?",
            project_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Average tokens
        let avg_tokens = sqlx::query_scalar::<_, f64>(
            "SELECT COALESCE(AVG(CAST(total_tokens AS REAL)), 0.0) FROM context_snapshots WHERE project_id = ?"
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        // Success rate (placeholder - would need task tracking)
        let success_rate = self.calculate_success_rate(project_id).await?;

        // Most used files
        let most_used = sqlx::query!(
            r#"
            SELECT file_path, CAST(inclusion_count AS INTEGER) as "inclusion_count!: i32"
            FROM context_usage_patterns
            WHERE project_id = ?
            ORDER BY inclusion_count DESC
            LIMIT 10
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| FileUsage {
            file: r.file_path,
            count: r.inclusion_count,
        })
        .collect();

        // Token usage over time (last 30 days)
        let token_timeline = self.get_token_timeline(project_id).await?;

        Ok(ContextStats {
            total_contexts_generated: total as i32,
            average_tokens: avg_tokens,
            success_rate,
            most_used_files: most_used,
            token_usage_over_time: token_timeline,
        })
    }

    async fn calculate_success_rate(&self, _project_id: &str) -> Result<f64, sqlx::Error> {
        // Placeholder - would integrate with task tracking
        Ok(85.0)
    }

    async fn get_token_timeline(&self, project_id: &str) -> Result<Vec<TokenUsage>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                DATE(created_at) as "date: String",
                CAST(SUM(total_tokens) AS INTEGER) as "tokens!: i32"
            FROM context_snapshots
            WHERE project_id = ?
            AND created_at > datetime('now', '-30 days')
            GROUP BY DATE(created_at)
            ORDER BY DATE(created_at)
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| TokenUsage {
                date: r.date.unwrap_or_default(),
                tokens: r.tokens,
            })
            .collect())
    }

    /// Delete old snapshots to manage storage
    pub async fn cleanup_old_snapshots(
        &self,
        project_id: &str,
        keep_recent: i32,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM context_snapshots
            WHERE id IN (
                SELECT id FROM context_snapshots
                WHERE project_id = ?
                ORDER BY created_at DESC
                LIMIT -1 OFFSET ?
            )
            "#,
            project_id,
            keep_recent
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

fn generate_id() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..12)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
