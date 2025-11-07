// ABOUTME: SQLite storage implementation for preview servers registry
// ABOUTME: Replaces JSON file storage with database-backed persistence

use crate::types::{DevServerStatus, ServerSource};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use orkee_storage::sqlite::SqliteStorage;
use sqlx::{sqlite::SqliteRow, Pool, Row, Sqlite};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Preview server entry stored in the database
#[derive(Debug, Clone)]
pub struct PreviewServerEntry {
    pub id: String,
    pub project_id: String,
    pub project_name: Option<String>,
    pub port: u16,
    pub preview_url: Option<String>,
    pub pid: Option<u32>,
    pub status: DevServerStatus,
    pub source: ServerSource,
    pub project_root: PathBuf,
    pub matched_project_id: Option<String>,
    pub framework_name: Option<String>,
    pub actual_command: Option<String>,
    pub started_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub api_port: u16,
}

/// Storage interface for preview servers
#[derive(Clone)]
pub struct PreviewServerStorage {
    pool: Pool<Sqlite>,
}

impl PreviewServerStorage {
    /// Create a new storage instance from the shared SqliteStorage
    pub async fn new(storage: &SqliteStorage) -> Result<Self> {
        Ok(Self {
            pool: storage.pool().clone(),
        })
    }

    /// Insert a new preview server entry
    pub async fn insert(&self, entry: &PreviewServerEntry) -> Result<()> {
        let status_str = self.status_to_string(&entry.status);
        let source_str = self.source_to_string(&entry.source);
        let project_root_str = entry.project_root.to_string_lossy();

        sqlx::query(
            r#"
            INSERT INTO preview_servers (
                id, project_id, project_name, port, preview_url, pid, status, source,
                project_root, matched_project_id, framework_name, actual_command,
                started_at, last_seen, api_port, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.project_id)
        .bind(&entry.project_name)
        .bind(entry.port as i64)
        .bind(&entry.preview_url)
        .bind(entry.pid.map(|p| p as i64))
        .bind(status_str)
        .bind(source_str)
        .bind(project_root_str.as_ref())
        .bind(&entry.matched_project_id)
        .bind(&entry.framework_name)
        .bind(&entry.actual_command)
        .bind(entry.started_at)
        .bind(entry.last_seen)
        .bind(entry.api_port as i64)
        .bind(None::<String>) // TODO: error_message - Reserved for future server crash/error tracking
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("FOREIGN KEY constraint failed") {
                anyhow::anyhow!(
                    "Failed to insert preview server '{}': project '{}' does not exist",
                    entry.id,
                    entry.project_id
                )
            } else {
                anyhow::anyhow!("Failed to insert preview server '{}': {}", entry.id, e)
            }
        })?;

        debug!(
            "Inserted preview server {} for project {}",
            entry.id, entry.project_id
        );
        Ok(())
    }

    /// Update an existing preview server entry
    pub async fn update(&self, entry: &PreviewServerEntry) -> Result<()> {
        let status_str = self.status_to_string(&entry.status);
        let source_str = self.source_to_string(&entry.source);
        let project_root_str = entry.project_root.to_string_lossy();

        sqlx::query(
            r#"
            UPDATE preview_servers SET
                project_id = ?, project_name = ?, port = ?, preview_url = ?, pid = ?,
                status = ?, source = ?, project_root = ?, matched_project_id = ?,
                framework_name = ?, actual_command = ?, started_at = ?, last_seen = ?,
                api_port = ?, error_message = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(&entry.project_id)
        .bind(&entry.project_name)
        .bind(entry.port as i64)
        .bind(&entry.preview_url)
        .bind(entry.pid.map(|p| p as i64))
        .bind(status_str)
        .bind(source_str)
        .bind(project_root_str.as_ref())
        .bind(&entry.matched_project_id)
        .bind(&entry.framework_name)
        .bind(&entry.actual_command)
        .bind(entry.started_at)
        .bind(entry.last_seen)
        .bind(entry.api_port as i64)
        .bind(None::<String>) // TODO: error_message - Reserved for future server crash/error tracking
        .bind(&entry.id)
        .execute(&self.pool)
        .await
        .context("Failed to update preview server")?;

        debug!(
            "Updated preview server {} for project {}",
            entry.id, entry.project_id
        );
        Ok(())
    }

    /// Delete a preview server entry by ID
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM preview_servers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete preview server")?;

        debug!("Deleted preview server {}", id);
        Ok(())
    }

    /// Get a preview server by ID
    pub async fn get(&self, id: &str) -> Result<Option<PreviewServerEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, project_name, port, preview_url, pid, status, source,
                   project_root, matched_project_id, framework_name, actual_command,
                   started_at, last_seen, api_port, error_message
            FROM preview_servers
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get preview server")?;

        Ok(row.as_ref().map(|r| self.row_to_entry(r)))
    }

    /// Get all preview servers
    pub async fn get_all(&self) -> Result<Vec<PreviewServerEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, project_name, port, preview_url, pid, status, source,
                   project_root, matched_project_id, framework_name, actual_command,
                   started_at, last_seen, api_port, error_message
            FROM preview_servers
            ORDER BY last_seen DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get all preview servers")?;

        Ok(rows.into_iter().map(|r| self.row_to_entry(&r)).collect())
    }

    /// Get preview servers by project ID
    pub async fn get_by_project(&self, project_id: &str) -> Result<Vec<PreviewServerEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, project_name, port, preview_url, pid, status, source,
                   project_root, matched_project_id, framework_name, actual_command,
                   started_at, last_seen, api_port, error_message
            FROM preview_servers
            WHERE project_id = ? OR matched_project_id = ?
            ORDER BY last_seen DESC
            "#,
        )
        .bind(project_id)
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get preview servers by project")?;

        Ok(rows.into_iter().map(|r| self.row_to_entry(&r)).collect())
    }

    /// Get preview server by port
    pub async fn get_by_port(&self, port: u16) -> Result<Option<PreviewServerEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, project_name, port, preview_url, pid, status, source,
                   project_root, matched_project_id, framework_name, actual_command,
                   started_at, last_seen, api_port, error_message
            FROM preview_servers
            WHERE port = ?
            LIMIT 1
            "#,
        )
        .bind(port as i64)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get preview server by port")?;

        Ok(row.as_ref().map(|r| self.row_to_entry(r)))
    }

    /// Update last_seen timestamp for a server
    pub async fn update_last_seen(&self, id: &str) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE preview_servers SET last_seen = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to update last_seen")?;

        Ok(())
    }

    /// Update server status and PID
    pub async fn update_status(
        &self,
        id: &str,
        status: crate::types::DevServerStatus,
        pid: Option<u32>,
    ) -> Result<()> {
        let status_str = match status {
            crate::types::DevServerStatus::Running => "running",
            crate::types::DevServerStatus::Stopped => "stopped",
            crate::types::DevServerStatus::Error => "error",
            crate::types::DevServerStatus::Starting => "starting",
            crate::types::DevServerStatus::Stopping => "stopping",
        };

        sqlx::query(
            "UPDATE preview_servers SET status = ?, pid = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(status_str)
        .bind(pid)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to update server status")?;

        Ok(())
    }

    /// Delete stale servers (not seen recently)
    pub async fn delete_stale(&self, older_than: DateTime<Utc>) -> Result<usize> {
        let result = sqlx::query("DELETE FROM preview_servers WHERE last_seen < ?")
            .bind(older_than)
            .execute(&self.pool)
            .await
            .context("Failed to delete stale servers")?;

        let deleted = result.rows_affected() as usize;
        if deleted > 0 {
            info!("Deleted {} stale preview server entries", deleted);
        }
        Ok(deleted)
    }

    /// Migrate from JSON file if it exists
    ///
    /// # Migration Strategy
    ///
    /// This uses a **partial migration** approach rather than all-or-nothing transactions:
    /// - Valid entries are migrated successfully
    /// - Failed entries are logged but don't block valid migrations
    /// - Original JSON file is preserved on any failure for recovery
    ///
    /// This design prioritizes data preservation over transaction atomicity because:
    /// 1. User data loss is worse than partial migration state
    /// 2. Failed entries often indicate data corruption or missing dependencies (FK violations)
    /// 3. Users can investigate failures and retry with corrected data
    /// 4. Valid server state is preserved immediately
    ///
    /// Alternative (all-or-nothing) would reject all migrations if one entry fails,
    /// losing valid server state unnecessarily.
    pub async fn migrate_from_json(&self, json_path: &PathBuf) -> Result<()> {
        use crate::registry::ServerRegistryEntry as JsonEntry;
        use std::fs;

        if !json_path.exists() {
            debug!("No JSON registry to migrate");
            return Ok(());
        }

        info!("Migrating preview server registry from {:?}", json_path);

        // Read and parse JSON file
        let json_content = fs::read_to_string(json_path).context("Failed to read JSON registry")?;

        let entries: std::collections::HashMap<String, JsonEntry> =
            serde_json::from_str(&json_content).context("Failed to parse JSON registry")?;

        let total_count = entries.len();
        let mut success_count = 0;
        let mut failed_entries = Vec::new();

        for (_, json_entry) in entries {
            let db_entry = PreviewServerEntry {
                id: json_entry.id.clone(),
                project_id: json_entry.project_id.clone(),
                project_name: json_entry.project_name.clone(),
                port: json_entry.port,
                preview_url: json_entry.preview_url.clone(),
                pid: json_entry.pid,
                status: json_entry.status,
                source: json_entry.source,
                project_root: json_entry.project_root.clone(),
                matched_project_id: json_entry.matched_project_id.clone(),
                framework_name: json_entry.framework_name.clone(),
                actual_command: json_entry.actual_command.clone(),
                started_at: json_entry.started_at,
                last_seen: json_entry.last_seen,
                api_port: json_entry.api_port,
            };

            match self.insert(&db_entry).await {
                Ok(_) => {
                    debug!("Migrated server {}", db_entry.id);
                    success_count += 1;
                }
                Err(e) => {
                    warn!("Failed to migrate server {}: {}", db_entry.id, e);
                    failed_entries.push((db_entry.id, e.to_string()));
                }
            }
        }

        // Only rename the JSON file if all migrations succeeded
        if failed_entries.is_empty() {
            let backup_path = json_path.with_extension("json.migrated");
            fs::rename(json_path, &backup_path).context("Failed to rename migrated JSON file")?;

            info!(
                "Successfully migrated {} servers to database. Original file backed up to {:?}",
                total_count, backup_path
            );
            Ok(())
        } else {
            // Don't rename - leave original file intact so user can retry or investigate
            let error_msg = format!(
                "Migration partially failed: {}/{} servers migrated successfully. Failed entries: {}. Original JSON file preserved for recovery.",
                success_count,
                total_count,
                failed_entries
                    .iter()
                    .map(|(id, err)| format!("{} ({})", id, err))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            warn!("{}", error_msg);
            anyhow::bail!(error_msg)
        }
    }

    // Helper to convert database row to entry struct
    fn row_to_entry(&self, row: &SqliteRow) -> PreviewServerEntry {
        let status_str: String = row.get("status");
        let source_str: String = row.get("source");

        let status = self.string_to_status(&status_str);
        let source = self.string_to_source(&source_str);

        let project_root_str: String = row.get("project_root");

        PreviewServerEntry {
            id: row.get("id"),
            project_id: row.get("project_id"),
            project_name: row.get("project_name"),
            port: row.get::<i64, _>("port") as u16,
            preview_url: row.get("preview_url"),
            pid: row.get::<Option<i64>, _>("pid").map(|p| p as u32),
            status,
            source,
            project_root: PathBuf::from(project_root_str),
            matched_project_id: row.get("matched_project_id"),
            framework_name: row.get("framework_name"),
            actual_command: row.get("actual_command"),
            started_at: row.get("started_at"),
            last_seen: row.get("last_seen"),
            api_port: row.get::<i64, _>("api_port") as u16,
        }
    }

    // Helper methods for enum conversion
    fn status_to_string(&self, status: &DevServerStatus) -> &'static str {
        match status {
            DevServerStatus::Stopped => "stopped",
            DevServerStatus::Starting => "starting",
            DevServerStatus::Running => "running",
            DevServerStatus::Stopping => "stopping",
            DevServerStatus::Error => "error",
        }
    }

    fn string_to_status(&self, status: &str) -> DevServerStatus {
        match status {
            "stopped" => DevServerStatus::Stopped,
            "starting" => DevServerStatus::Starting,
            "running" => DevServerStatus::Running,
            "stopping" => DevServerStatus::Stopping,
            "error" => DevServerStatus::Error,
            _ => DevServerStatus::Stopped,
        }
    }

    fn source_to_string(&self, source: &ServerSource) -> &'static str {
        match source {
            ServerSource::Orkee => "orkee",
            ServerSource::External => "external",
            ServerSource::Discovered => "discovered",
        }
    }

    fn string_to_source(&self, source: &str) -> ServerSource {
        match source {
            "orkee" => ServerSource::Orkee,
            "external" => ServerSource::External,
            "discovered" => ServerSource::Discovered,
            _ => ServerSource::Orkee,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orkee_storage::ProjectStorage;
    use tempfile::TempDir;
    use uuid::Uuid;

    async fn setup_test_db() -> (PreviewServerStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create storage with test database using StorageConfig
        let config = orkee_storage::StorageConfig {
            provider: orkee_storage::StorageProvider::Sqlite {
                path: db_path.clone(),
            },
            max_connections: 5,
            busy_timeout_seconds: 30,
            enable_wal: false, // WAL doesn't work well with temporary files
            enable_fts: true,
        };
        let storage = SqliteStorage::new(config).await.unwrap();
        // Run migrations to create tables
        storage.initialize().await.unwrap();

        // Insert a test project to satisfy foreign key constraint
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("test-project")
        .bind("Test Project")
        .bind("/home/test/project")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(storage.pool())
        .await
        .unwrap();

        let preview_storage = PreviewServerStorage::new(&storage).await.unwrap();

        (preview_storage, temp_dir)
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let (storage, _temp_dir) = setup_test_db().await;

        let entry = PreviewServerEntry {
            id: Uuid::new_v4().to_string(),
            project_id: "test-project".to_string(),
            project_name: Some("Test Project".to_string()),
            port: 3000,
            preview_url: Some("http://localhost:3000".to_string()),
            pid: Some(12345),
            status: DevServerStatus::Running,
            source: ServerSource::Orkee,
            project_root: PathBuf::from("/home/test/project"),
            matched_project_id: None,
            framework_name: Some("Next.js".to_string()),
            actual_command: Some("npm run dev".to_string()),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: 4001,
        };

        // Insert
        storage.insert(&entry).await.unwrap();

        // Get by ID
        let retrieved = storage.get(&entry.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.project_id, entry.project_id);
        assert_eq!(retrieved.port, entry.port);
    }

    #[tokio::test]
    async fn test_update() {
        let (storage, _temp_dir) = setup_test_db().await;

        let mut entry = PreviewServerEntry {
            id: Uuid::new_v4().to_string(),
            project_id: "test-project".to_string(),
            project_name: Some("Test Project".to_string()),
            port: 3000,
            preview_url: Some("http://localhost:3000".to_string()),
            pid: Some(12345),
            status: DevServerStatus::Running,
            source: ServerSource::Orkee,
            project_root: PathBuf::from("/home/test/project"),
            matched_project_id: None,
            framework_name: Some("Next.js".to_string()),
            actual_command: Some("npm run dev".to_string()),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: 4001,
        };

        // Insert
        storage.insert(&entry).await.unwrap();

        // Update
        entry.port = 3001;
        entry.status = DevServerStatus::Stopped;
        storage.update(&entry).await.unwrap();

        // Verify update
        let retrieved = storage.get(&entry.id).await.unwrap().unwrap();
        assert_eq!(retrieved.port, 3001);
        matches!(retrieved.status, DevServerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_delete() {
        let (storage, _temp_dir) = setup_test_db().await;

        let entry = PreviewServerEntry {
            id: Uuid::new_v4().to_string(),
            project_id: "test-project".to_string(),
            project_name: Some("Test Project".to_string()),
            port: 3000,
            preview_url: Some("http://localhost:3000".to_string()),
            pid: Some(12345),
            status: DevServerStatus::Running,
            source: ServerSource::Orkee,
            project_root: PathBuf::from("/home/test/project"),
            matched_project_id: None,
            framework_name: Some("Next.js".to_string()),
            actual_command: Some("npm run dev".to_string()),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: 4001,
        };

        // Insert
        storage.insert(&entry).await.unwrap();

        // Delete
        storage.delete(&entry.id).await.unwrap();

        // Verify deletion
        let retrieved = storage.get(&entry.id).await.unwrap();
        assert!(retrieved.is_none());
    }
}
