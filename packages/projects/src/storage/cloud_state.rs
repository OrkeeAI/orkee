use crate::storage::{StorageError, StorageResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Cloud sync state record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncState {
    pub id: i64,
    pub provider_name: String,
    pub provider_type: String,
    pub enabled: bool,
    
    // Last sync information
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_successful_sync_at: Option<DateTime<Utc>>,
    pub last_snapshot_id: Option<String>,
    
    // Current state
    pub sync_in_progress: bool,
    pub current_operation: Option<String>,
    
    // Error tracking
    pub error_count: i64,
    pub last_error_message: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
    
    // Configuration
    pub auto_sync_enabled: bool,
    pub sync_interval_minutes: i64,
    pub max_snapshots: i64,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cloud snapshot record (local cache)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSnapshot {
    pub id: String,
    pub provider_name: String,
    pub snapshot_id: String,
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub size_bytes: i64,
    pub compressed_size_bytes: i64,
    pub project_count: i64,
    pub version: i64,
    pub checksum: Option<String>,
    pub encrypted: bool,
    
    // Storage information
    pub storage_path: Option<String>,
    pub etag: Option<String>,
    pub last_accessed_at: Option<DateTime<Utc>>,
    
    // Sync tracking
    pub uploaded_at: Option<DateTime<Utc>>,
    pub download_count: i64,
    pub last_downloaded_at: Option<DateTime<Utc>>,
    
    // Local state
    pub locally_deleted: bool,
    pub deletion_scheduled_at: Option<DateTime<Utc>>,
    
    // JSON metadata
    pub metadata_json: Option<String>,
    pub tags_json: Option<String>,
}

/// Sync conflict record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub id: i64,
    pub provider_name: String,
    pub snapshot_id: Option<String>,
    pub project_id: String,
    
    // Conflict details
    pub detected_at: DateTime<Utc>,
    pub conflict_type: String,
    
    // Conflict data
    pub local_value: Option<String>,
    pub remote_value: Option<String>,
    pub local_version: Option<i64>,
    pub remote_version: Option<i64>,
    
    // Resolution
    pub resolution_status: String,
    pub resolution_strategy: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub resolution_notes: Option<String>,
}

/// Sync operation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperationLog {
    pub id: i64,
    pub provider_name: String,
    pub operation_type: String,
    pub operation_id: Option<String>,
    
    // Operation details
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    
    // Data tracking
    pub snapshot_id: Option<String>,
    pub projects_affected: i64,
    pub bytes_transferred: i64,
    
    // Results
    pub error_message: Option<String>,
    pub warning_messages: Option<String>,
    
    // Performance metrics
    pub duration_seconds: Option<i64>,
    pub network_time_seconds: Option<i64>,
    pub processing_time_seconds: Option<i64>,
    
    // Context information
    pub initiated_by: String,
    pub user_agent: Option<String>,
    pub client_version: Option<String>,
    
    // Additional metadata
    pub metadata_json: Option<String>,
}

/// Database operations for cloud sync state management
pub struct CloudSyncStateManager {
    pool: SqlitePool,
}

impl CloudSyncStateManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create or update cloud sync state for a provider
    pub async fn upsert_provider_state(
        &self,
        provider_name: &str,
        provider_type: &str,
    ) -> StorageResult<CloudSyncState> {
        let now = Utc::now();
        
        let result = sqlx::query!(
            r#"
            INSERT INTO cloud_sync_state (
                provider_name, provider_type, enabled, 
                error_count, auto_sync_enabled, sync_interval_minutes, 
                max_snapshots, encryption_enabled, compression_enabled,
                created_at, updated_at
            ) VALUES (?, ?, 1, 0, 0, 1440, 30, 1, 1, ?, ?)
            ON CONFLICT(provider_name) DO UPDATE SET
                provider_type = excluded.provider_type,
                updated_at = excluded.updated_at
            "#,
            provider_name, provider_type,
            now.to_rfc3339(), now.to_rfc3339()
        )
        .execute(&self.pool)
        .await?;

        self.get_provider_state(provider_name).await?
            .ok_or(StorageError::NotFound)
    }

    /// Get cloud sync state for a provider
    pub async fn get_provider_state(&self, provider_name: &str) -> StorageResult<Option<CloudSyncState>> {
        let record = sqlx::query!(
            "SELECT * FROM cloud_sync_state WHERE provider_name = ?",
            provider_name
        )
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(row) => Ok(Some(CloudSyncState {
                id: row.id,
                provider_name: row.provider_name,
                provider_type: row.provider_type,
                enabled: row.enabled.unwrap_or(false),
                last_sync_at: row.last_sync_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                last_successful_sync_at: row.last_successful_sync_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                last_snapshot_id: row.last_snapshot_id,
                sync_in_progress: row.sync_in_progress.unwrap_or(false),
                current_operation: row.current_operation,
                error_count: row.error_count.unwrap_or(0),
                last_error_message: row.last_error_message,
                last_error_at: row.last_error_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                auto_sync_enabled: row.auto_sync_enabled.unwrap_or(false),
                sync_interval_minutes: row.sync_interval_minutes.unwrap_or(1440),
                max_snapshots: row.max_snapshots.unwrap_or(30),
                encryption_enabled: row.encryption_enabled.unwrap_or(true),
                compression_enabled: row.compression_enabled.unwrap_or(true),
                created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at).unwrap().with_timezone(&Utc),
            })),
            None => Ok(None),
        }
    }

    /// Update sync state after an operation
    pub async fn update_sync_state(
        &self,
        provider_name: &str,
        last_sync_at: Option<DateTime<Utc>>,
        last_successful_sync_at: Option<DateTime<Utc>>,
        last_snapshot_id: Option<&str>,
        sync_in_progress: bool,
        current_operation: Option<&str>,
        error_message: Option<&str>,
    ) -> StorageResult<()> {
        let now = Utc::now();
        
        sqlx::query!(
            r#"
            UPDATE cloud_sync_state SET
                last_sync_at = COALESCE(?, last_sync_at),
                last_successful_sync_at = COALESCE(?, last_successful_sync_at),
                last_snapshot_id = COALESCE(?, last_snapshot_id),
                sync_in_progress = ?,
                current_operation = ?,
                last_error_message = ?,
                last_error_at = CASE WHEN ? IS NOT NULL THEN ? ELSE last_error_at END,
                error_count = CASE WHEN ? IS NOT NULL THEN error_count + 1 ELSE error_count END,
                updated_at = ?
            WHERE provider_name = ?
            "#,
            last_sync_at.map(|dt| dt.to_rfc3339()),
            last_successful_sync_at.map(|dt| dt.to_rfc3339()),
            last_snapshot_id,
            sync_in_progress,
            current_operation,
            error_message,
            error_message,
            now.to_rfc3339(),
            error_message,
            now.to_rfc3339(),
            provider_name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store snapshot metadata
    pub async fn store_snapshot_metadata(
        &self,
        snapshot: &CloudSnapshot,
    ) -> StorageResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO cloud_snapshots (
                id, provider_name, snapshot_id, created_at, size_bytes, compressed_size_bytes,
                project_count, version, checksum, encrypted, storage_path, etag,
                last_accessed_at, uploaded_at, download_count, last_downloaded_at,
                locally_deleted, deletion_scheduled_at, metadata_json, tags_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(provider_name, snapshot_id) DO UPDATE SET
                size_bytes = excluded.size_bytes,
                compressed_size_bytes = excluded.compressed_size_bytes,
                project_count = excluded.project_count,
                checksum = excluded.checksum,
                etag = excluded.etag,
                last_accessed_at = excluded.last_accessed_at,
                metadata_json = excluded.metadata_json,
                tags_json = excluded.tags_json
            "#,
            snapshot.id, snapshot.provider_name, snapshot.snapshot_id,
            snapshot.created_at.to_rfc3339(), snapshot.size_bytes, snapshot.compressed_size_bytes,
            snapshot.project_count, snapshot.version, snapshot.checksum, snapshot.encrypted,
            snapshot.storage_path, snapshot.etag,
            snapshot.last_accessed_at.map(|dt| dt.to_rfc3339()),
            snapshot.uploaded_at.map(|dt| dt.to_rfc3339()),
            snapshot.download_count,
            snapshot.last_downloaded_at.map(|dt| dt.to_rfc3339()),
            snapshot.locally_deleted,
            snapshot.deletion_scheduled_at.map(|dt| dt.to_rfc3339()),
            snapshot.metadata_json, snapshot.tags_json
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List snapshots for a provider
    pub async fn list_snapshots(
        &self,
        provider_name: &str,
        limit: Option<u32>,
    ) -> StorageResult<Vec<CloudSnapshot>> {
        let limit = limit.unwrap_or(50) as i64;
        
        let records = sqlx::query!(
            r#"
            SELECT * FROM cloud_snapshots 
            WHERE provider_name = ? AND locally_deleted = 0 
            ORDER BY created_at DESC 
            LIMIT ?
            "#,
            provider_name, limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut snapshots = Vec::new();
        for row in records {
            snapshots.push(CloudSnapshot {
                id: row.id,
                provider_name: row.provider_name,
                snapshot_id: row.snapshot_id,
                created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
                size_bytes: row.size_bytes,
                compressed_size_bytes: row.compressed_size_bytes,
                project_count: row.project_count,
                version: row.version,
                checksum: row.checksum,
                encrypted: row.encrypted.unwrap_or(false),
                storage_path: row.storage_path,
                etag: row.etag,
                last_accessed_at: row.last_accessed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                uploaded_at: row.uploaded_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                download_count: row.download_count.unwrap_or(0),
                last_downloaded_at: row.last_downloaded_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                locally_deleted: row.locally_deleted.unwrap_or(false),
                deletion_scheduled_at: row.deletion_scheduled_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                metadata_json: row.metadata_json,
                tags_json: row.tags_json,
            });
        }

        Ok(snapshots)
    }

    /// Log a sync operation
    pub async fn log_sync_operation(
        &self,
        provider_name: &str,
        operation_type: &str,
        operation_id: Option<&str>,
        snapshot_id: Option<&str>,
        projects_affected: i64,
        bytes_transferred: i64,
        status: &str,
        error_message: Option<&str>,
        duration_seconds: Option<i64>,
    ) -> StorageResult<i64> {
        let now = Utc::now();
        
        let result = sqlx::query!(
            r#"
            INSERT INTO sync_operations_log (
                provider_name, operation_type, operation_id, started_at,
                snapshot_id, projects_affected, bytes_transferred, status,
                error_message, duration_seconds, initiated_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'system')
            "#,
            provider_name, operation_type, operation_id, now.to_rfc3339(),
            snapshot_id, projects_affected, bytes_transferred, status,
            error_message, duration_seconds
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Record a sync conflict
    pub async fn record_sync_conflict(
        &self,
        provider_name: &str,
        snapshot_id: Option<&str>,
        project_id: &str,
        conflict_type: &str,
        local_value: Option<&str>,
        remote_value: Option<&str>,
        local_version: Option<i64>,
        remote_version: Option<i64>,
    ) -> StorageResult<i64> {
        let now = Utc::now();
        
        let result = sqlx::query!(
            r#"
            INSERT INTO sync_conflicts (
                provider_name, snapshot_id, project_id, detected_at, conflict_type,
                local_value, remote_value, local_version, remote_version, resolution_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending')
            "#,
            provider_name, snapshot_id, project_id, now.to_rfc3339(), conflict_type,
            local_value, remote_value, local_version, remote_version
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get pending conflicts for a provider
    pub async fn get_pending_conflicts(&self, provider_name: &str) -> StorageResult<Vec<SyncConflict>> {
        let records = sqlx::query!(
            r#"
            SELECT * FROM sync_conflicts 
            WHERE provider_name = ? AND resolution_status = 'pending'
            ORDER BY detected_at DESC
            "#,
            provider_name
        )
        .fetch_all(&self.pool)
        .await?;

        let mut conflicts = Vec::new();
        for row in records {
            conflicts.push(SyncConflict {
                id: row.id,
                provider_name: row.provider_name,
                snapshot_id: row.snapshot_id,
                project_id: row.project_id,
                detected_at: DateTime::parse_from_rfc3339(&row.detected_at).unwrap().with_timezone(&Utc),
                conflict_type: row.conflict_type,
                local_value: row.local_value,
                remote_value: row.remote_value,
                local_version: row.local_version,
                remote_version: row.remote_version,
                resolution_status: row.resolution_status,
                resolution_strategy: row.resolution_strategy,
                resolved_at: row.resolved_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                resolved_by: row.resolved_by,
                resolution_notes: row.resolution_notes,
            });
        }

        Ok(conflicts)
    }

    /// Get sync health summary
    pub async fn get_sync_health_summary(&self) -> StorageResult<Vec<SyncHealthSummary>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                css.provider_name,
                css.provider_type,
                css.enabled,
                css.auto_sync_enabled,
                css.last_successful_sync_at,
                css.error_count,
                COUNT(cs.id) as snapshot_count,
                COUNT(CASE WHEN sc.resolution_status = 'pending' THEN 1 END) as pending_conflicts,
                MAX(cs.created_at) as latest_snapshot_at
            FROM cloud_sync_state css
            LEFT JOIN cloud_snapshots cs ON css.provider_name = cs.provider_name AND cs.locally_deleted = 0
            LEFT JOIN sync_conflicts sc ON css.provider_name = sc.provider_name AND sc.resolution_status = 'pending'
            GROUP BY css.provider_name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut summaries = Vec::new();
        for row in records {
            summaries.push(SyncHealthSummary {
                provider_name: row.provider_name,
                provider_type: row.provider_type,
                enabled: row.enabled.unwrap_or(false),
                auto_sync_enabled: row.auto_sync_enabled.unwrap_or(false),
                last_successful_sync_at: row.last_successful_sync_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                error_count: row.error_count.unwrap_or(0),
                snapshot_count: row.snapshot_count.unwrap_or(0),
                pending_conflicts: row.pending_conflicts.unwrap_or(0),
                latest_snapshot_at: row.latest_snapshot_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            });
        }

        Ok(summaries)
    }

    /// Clean up old snapshots based on retention policy
    pub async fn cleanup_old_snapshots(
        &self,
        provider_name: &str,
        max_snapshots: i64,
    ) -> StorageResult<i64> {
        // Mark snapshots for deletion beyond the retention limit
        let result = sqlx::query!(
            r#"
            UPDATE cloud_snapshots 
            SET locally_deleted = 1, deletion_scheduled_at = datetime('now')
            WHERE provider_name = ? AND locally_deleted = 0
            AND id NOT IN (
                SELECT id FROM cloud_snapshots 
                WHERE provider_name = ? AND locally_deleted = 0 
                ORDER BY created_at DESC 
                LIMIT ?
            )
            "#,
            provider_name, provider_name, max_snapshots
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

/// Sync health summary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHealthSummary {
    pub provider_name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub auto_sync_enabled: bool,
    pub last_successful_sync_at: Option<DateTime<Utc>>,
    pub error_count: i64,
    pub snapshot_count: i64,
    pub pending_conflicts: i64,
    pub latest_snapshot_at: Option<DateTime<Utc>>,
}

impl std::fmt::Display for SyncHealthSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Provider: {} ({})", self.provider_name, self.provider_type)?;
        writeln!(f, "  Status: {}", if self.enabled { "✅ Enabled" } else { "❌ Disabled" })?;
        writeln!(f, "  Auto Sync: {}", if self.auto_sync_enabled { "On" } else { "Off" })?;
        writeln!(f, "  Snapshots: {}", self.snapshot_count)?;
        writeln!(f, "  Errors: {}", self.error_count)?;
        
        if self.pending_conflicts > 0 {
            writeln!(f, "  ⚠️  Pending Conflicts: {}", self.pending_conflicts)?;
        }
        
        match self.last_successful_sync_at {
            Some(last_sync) => {
                let ago = Utc::now().signed_duration_since(last_sync);
                if ago.num_days() > 0 {
                    writeln!(f, "  Last Sync: {} days ago", ago.num_days())?;
                } else if ago.num_hours() > 0 {
                    writeln!(f, "  Last Sync: {} hours ago", ago.num_hours())?;
                } else {
                    writeln!(f, "  Last Sync: {} minutes ago", ago.num_minutes())?;
                }
            }
            None => writeln!(f, "  Last Sync: Never")?,
        }
        
        Ok(())
    }
}