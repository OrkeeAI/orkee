// ABOUTME: SQLite-backed central registry for tracking development servers
// ABOUTME: Replaces JSON file storage with database persistence for better reliability

use chrono::{DateTime, Utc};
use orkee_config::env::parse_env_or_default_with_validation;
use orkee_storage::sqlite::SqliteStorage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::storage::{PreviewServerEntry, PreviewServerStorage};
use crate::types::{DevServerStatus, ServerSource};

/// Entry in the central server registry.
///
/// Represents a development server tracked across all Orkee instances on the system.
/// This is now stored in the SQLite database and serves as the single source
/// of truth for all development servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistryEntry {
    /// Unique identifier for this server instance
    pub id: String,
    /// Project identifier that this server is associated with
    pub project_id: String,
    /// Optional human-readable project name
    pub project_name: Option<String>,
    /// Absolute path to the project's root directory
    pub project_root: PathBuf,
    /// Port number the server is listening on
    pub port: u16,
    /// Process ID of the running server (if available)
    pub pid: Option<u32>,
    /// Current status of the server (Running, Stopped, Error, etc.)
    pub status: DevServerStatus,
    /// Full preview URL for accessing the server (e.g., "http://localhost:3000")
    pub preview_url: Option<String>,
    /// Detected or configured framework name (e.g., "Vite", "Next.js")
    pub framework_name: Option<String>,
    /// The actual command that was executed to start the server
    pub actual_command: Option<String>,
    /// Timestamp when the server was originally started
    pub started_at: DateTime<Utc>,
    /// Timestamp when the server was last seen active (for stale detection)
    pub last_seen: DateTime<Utc>,
    /// Port of the Orkee API instance that manages this server
    pub api_port: u16,
    /// Source of the server (Orkee, External, or Discovered)
    #[serde(default = "default_server_source")]
    pub source: ServerSource,
    /// ID of the matched project (for external/discovered servers)
    pub matched_project_id: Option<String>,
}

fn default_server_source() -> ServerSource {
    ServerSource::Orkee
}

impl From<PreviewServerEntry> for ServerRegistryEntry {
    fn from(entry: PreviewServerEntry) -> Self {
        Self {
            id: entry.id,
            project_id: entry.project_id,
            project_name: entry.project_name,
            project_root: entry.project_root,
            port: entry.port,
            pid: entry.pid,
            status: entry.status,
            preview_url: entry.preview_url,
            framework_name: entry.framework_name,
            actual_command: entry.actual_command,
            started_at: entry.started_at,
            last_seen: entry.last_seen,
            api_port: entry.api_port,
            source: entry.source,
            matched_project_id: entry.matched_project_id,
        }
    }
}

impl From<ServerRegistryEntry> for PreviewServerEntry {
    fn from(entry: ServerRegistryEntry) -> Self {
        Self {
            id: entry.id,
            project_id: entry.project_id,
            project_name: entry.project_name,
            project_root: entry.project_root,
            port: entry.port,
            pid: entry.pid,
            status: entry.status,
            preview_url: entry.preview_url,
            framework_name: entry.framework_name,
            actual_command: entry.actual_command,
            started_at: entry.started_at,
            last_seen: entry.last_seen,
            api_port: entry.api_port,
            source: entry.source,
            matched_project_id: entry.matched_project_id,
        }
    }
}

/// Central registry for tracking all development servers across Orkee instances.
///
/// This registry provides a global view of all development servers running on the system,
/// regardless of which Orkee instance started them. It persists to SQLite database at
/// `~/.orkee/orkee.db` and uses database transactions to ensure consistency.
#[derive(Clone)]
pub struct ServerRegistry {
    storage: PreviewServerStorage,
    /// Timeout in minutes before considering an entry stale (default: 5, configurable via ORKEE_STALE_TIMEOUT_MINUTES)
    stale_timeout_minutes: i64,
}

impl ServerRegistry {
    /// Create a new server registry instance.
    ///
    /// Initializes a new registry that persists to the SQLite database.
    /// The stale timeout can be configured via the `ORKEE_STALE_TIMEOUT_MINUTES`
    /// environment variable (default: 5 minutes, max: 240 minutes/4 hours).
    ///
    /// # Arguments
    ///
    /// * `storage` - Reference to the shared SqliteStorage instance for database access
    ///
    /// # Returns
    ///
    /// Returns a new `ServerRegistry` instance connected to the database.
    pub async fn new(storage: &SqliteStorage) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize preview server storage (will run migrations and JSON import if needed)
        let storage = PreviewServerStorage::new(storage).await?;

        // Parse stale timeout with validation (max 240 minutes = 4 hours)
        let stale_timeout_minutes =
            parse_env_or_default_with_validation("ORKEE_STALE_TIMEOUT_MINUTES", 5i64, |v| {
                (1..=240).contains(&v)
            });

        Ok(Self {
            storage,
            stale_timeout_minutes,
        })
    }

    /// Register a new server in the global registry.
    ///
    /// # Arguments
    ///
    /// * `entry` - The server entry to register
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    pub async fn register_server(
        &self,
        entry: ServerRegistryEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Registering server {} ({}) on port {} in global registry",
            entry.id, entry.project_id, entry.port
        );

        let server_id = entry.id.clone();
        self.storage
            .insert(&entry.into())
            .await
            .map_err(|e| format!("{}", e))?;

        debug!("Server {} successfully registered in database", server_id);
        Ok(())
    }

    /// Unregister a server from the global registry.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The unique identifier of the server to unregister
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    pub async fn unregister_server(
        &self,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Unregistering server {} from global registry", server_id);

        self.storage
            .delete(server_id)
            .await
            .map_err(|e| format!("{}", e))?;

        debug!(
            "Server {} successfully unregistered from database",
            server_id
        );
        Ok(())
    }

    /// Get all servers from the registry.
    ///
    /// # Returns
    ///
    /// Returns a vector of all server entries in the registry.
    pub async fn get_all_servers(&self) -> Vec<ServerRegistryEntry> {
        match self.storage.get_all().await {
            Ok(entries) => entries.into_iter().map(|e| e.into()).collect(),
            Err(e) => {
                error!("Failed to get all servers from database: {}", e);
                vec![]
            }
        }
    }

    /// Get a specific server by ID.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The unique identifier of the server to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(ServerRegistryEntry)` if found, `None` otherwise.
    pub async fn get_server(&self, server_id: &str) -> Option<ServerRegistryEntry> {
        match self.storage.get(server_id).await {
            Ok(Some(entry)) => Some(entry.into()),
            Ok(None) => None,
            Err(e) => {
                error!("Failed to get server {} from database: {}", server_id, e);
                None
            }
        }
    }

    /// Get a server by port number.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to search for
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(ServerRegistryEntry))` if a server is found on that port,
    /// `Ok(None)` if no server is running on that port, or `Err` on database error.
    pub async fn get_by_port(&self, port: u16) -> anyhow::Result<Option<ServerRegistryEntry>> {
        match self.storage.get_by_port(port).await {
            Ok(Some(entry)) => Ok(Some(entry.into())),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to get server by port {} from database: {}", port, e);
                Err(e)
            }
        }
    }

    /// Update the status of a server in the registry.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The unique identifier of the server to update
    /// * `status` - The new status to set
    /// * `pid` - Optional new process ID
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    pub async fn update_server_status(
        &self,
        server_id: &str,
        status: DevServerStatus,
        pid: Option<u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Updating server {} status to {:?}", server_id, status);

        self.storage
            .update_status(server_id, status, pid)
            .await
            .map_err(|e| format!("{}", e))?;

        Ok(())
    }

    /// Get the stale timeout in minutes.
    ///
    /// This is the amount of time after which a server entry is considered stale
    /// if it hasn't been seen. The timeout can be configured via the
    /// `ORKEE_STALE_TIMEOUT_MINUTES` environment variable.
    ///
    /// # Returns
    ///
    /// Returns the stale timeout in minutes.
    pub fn get_stale_timeout_minutes(&self) -> i64 {
        self.stale_timeout_minutes
    }

    /// Clean up stale entries from the registry.
    ///
    /// Removes entries that haven't been seen within the stale timeout period
    /// and validates that processes are still running.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation fails.
    pub async fn cleanup_stale_entries(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let stale_threshold = now - chrono::Duration::minutes(self.stale_timeout_minutes);

        debug!(
            "Running cleanup: removing servers not seen since {}",
            stale_threshold
        );

        let all_servers = self.get_all_servers().await;
        let mut removed_count = 0;

        for entry in all_servers {
            let mut should_remove = false;

            // Check if entry is stale (hasn't been seen recently)
            if entry.last_seen < stale_threshold {
                debug!(
                    "Server {} is stale (last seen: {})",
                    entry.id, entry.last_seen
                );
                should_remove = true;
            }

            // Additionally check if process is still running (if we have a PID)
            if let Some(pid) = entry.pid {
                if !is_process_running_validated(pid, &entry.project_root, entry.started_at) {
                    debug!("Server {} process {} is no longer running", entry.id, pid);
                    should_remove = true;
                }
            }

            if should_remove {
                if let Err(e) = self.unregister_server(&entry.id).await {
                    error!("Failed to remove stale server {}: {}", entry.id, e);
                } else {
                    removed_count += 1;
                    info!(
                        "Removed stale server {} (port {}, last seen: {})",
                        entry.id, entry.port, entry.last_seen
                    );
                }
            }
        }

        if removed_count > 0 {
            info!("Cleanup complete: removed {} stale entries", removed_count);
        } else {
            debug!("Cleanup complete: no stale entries found");
        }

        Ok(())
    }
}

/// Validate that a process is running and matches expected criteria.
///
/// This function performs multiple validation checks to ensure the process is legitimate:
/// 1. Process with given PID exists
/// 2. Process working directory matches expected path (prevents PID reuse false positives)
/// 3. Process start time is close to expected start time (prevents PID reuse on fast systems)
///
/// # Arguments
///
/// * `pid` - Process ID to validate
/// * `expected_cwd` - Expected working directory of the process
/// * `expected_start_time` - Expected start time of the process
///
/// # Returns
///
/// Returns `true` if the process is running and valid, `false` otherwise.
pub fn is_process_running_validated(
    pid: u32,
    expected_cwd: &std::path::Path,
    expected_start_time: DateTime<Utc>,
) -> bool {
    use sysinfo::{Pid, System};

    let mut sys = System::new();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        // Validate working directory matches
        if let Some(cwd) = process.cwd() {
            if cwd != expected_cwd {
                debug!(
                    "Process {} CWD mismatch: expected {:?}, got {:?}",
                    pid, expected_cwd, cwd
                );
                return false;
            }
        }

        // Validate start time is close to expected (within tolerance)
        let process_start = process.start_time();
        let expected_start = expected_start_time.timestamp() as u64;

        let tolerance_secs: u64 = parse_env_or_default_with_validation(
            "ORKEE_PROCESS_START_TIME_TOLERANCE_SECS",
            5u64,
            |v| (1..=60).contains(&v),
        );

        let time_diff = process_start.abs_diff(expected_start);

        if time_diff > tolerance_secs {
            debug!(
                "Process {} start time mismatch: expected {}, got {}, diff: {}s",
                pid, expected_start, process_start, time_diff
            );
            return false;
        }

        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orkee_storage::ProjectStorage;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_registry_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create storage and registry
        let config = orkee_storage::StorageConfig {
            provider: orkee_storage::StorageProvider::Sqlite { path: db_path },
            max_connections: 5,
            busy_timeout_seconds: 30,
            enable_wal: false,
            enable_fts: true,
        };
        let storage = SqliteStorage::new(config).await.unwrap();
        // Run migrations to create tables
        storage.initialize().await.unwrap();

        // Insert test project to satisfy foreign key constraint
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("project-1")
        .bind("Test Project")
        .bind("/tmp/test")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(storage.pool())
        .await
        .unwrap();

        let registry = ServerRegistry::new(&storage).await.unwrap();

        // Create a test entry
        let entry = ServerRegistryEntry {
            id: "test-123".to_string(),
            project_id: "project-1".to_string(),
            project_name: Some("Test Project".to_string()),
            project_root: PathBuf::from("/tmp/test"),
            port: 3000,
            pid: Some(12345),
            status: DevServerStatus::Running,
            preview_url: Some("http://localhost:3000".to_string()),
            framework_name: Some("Vite".to_string()),
            actual_command: Some("npm run dev".to_string()),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: 4001,
            source: ServerSource::Orkee,
            matched_project_id: None,
        };

        // Register server
        registry.register_server(entry.clone()).await.unwrap();

        // Get server
        let retrieved = registry.get_server("test-123").await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "test-123");
        assert_eq!(retrieved.port, 3000);

        // Get all servers
        let all = registry.get_all_servers().await;
        assert_eq!(all.len(), 1);

        // Update status
        registry
            .update_server_status("test-123", DevServerStatus::Stopped, None)
            .await
            .unwrap();

        let updated = registry.get_server("test-123").await.unwrap();
        assert_eq!(updated.status, DevServerStatus::Stopped);

        // Unregister server
        registry.unregister_server("test-123").await.unwrap();

        let after_delete = registry.get_server("test-123").await;
        assert!(after_delete.is_none());
    }
}
