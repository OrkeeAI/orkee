use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::env::parse_env_or_default_with_validation;
use crate::types::DevServerStatus;

/// Central server registry that tracks ALL dev servers across all Orkee instances
/// This is stored in ~/.orkee/server-registry.json and is the single source of truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistryEntry {
    pub id: String,
    pub project_id: String,
    pub project_name: Option<String>,
    pub project_root: PathBuf,
    pub port: u16,
    pub pid: Option<u32>,
    pub status: DevServerStatus,
    pub preview_url: Option<String>,
    pub framework_name: Option<String>,
    pub actual_command: Option<String>,
    pub started_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub api_port: u16, // Which Orkee API instance manages this server
}

/// The central server registry
pub struct ServerRegistry {
    registry_path: PathBuf,
    entries: Arc<RwLock<HashMap<String, ServerRegistryEntry>>>,
    /// Timeout in minutes before considering an entry stale (default: 5)
    stale_timeout_minutes: i64,
}

impl ServerRegistry {
    /// Create a new server registry instance
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| {
            // Fallback to current directory if home can't be determined
            // Log warning and use a temp location
            warn!("Could not determine home directory, using current directory");
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });
        let registry_path = home.join(".orkee").join("server-registry.json");

        // Read stale timeout from environment variable, default to 5 minutes
        // Validate the timeout value (must be positive, max 1440 minutes = 24 hours)
        let stale_timeout_minutes =
            parse_env_or_default_with_validation("ORKEE_STALE_TIMEOUT_MINUTES", 5, |v| {
                v > 0 && v <= 1440
            });

        debug!(
            "Server registry stale timeout set to {} minutes",
            stale_timeout_minutes
        );

        let registry = Self {
            registry_path,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stale_timeout_minutes,
        };

        // Load existing registry on creation
        let _ = futures::executor::block_on(registry.load_registry());
        registry
    }

    /// Load the registry from disk
    pub async fn load_registry(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.registry_path.exists() {
            debug!(
                "Server registry does not exist yet at {:?}",
                self.registry_path
            );
            return Ok(());
        }

        let content = fs::read_to_string(&self.registry_path)?;
        let entries: HashMap<String, ServerRegistryEntry> = serde_json::from_str(&content)?;

        let mut registry = self.entries.write().await;
        *registry = entries;

        info!("Loaded {} servers from registry", registry.len());
        Ok(())
    }

    /// Save the registry to disk
    pub async fn save_registry(&self) -> Result<(), Box<dyn std::error::Error>> {
        let registry = self.entries.read().await;
        self.save_entries_to_disk(&*registry).await
    }

    /// Helper to save entries to disk (used for transactional updates)
    async fn save_entries_to_disk(
        &self,
        entries: &HashMap<String, ServerRegistryEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure the .orkee directory exists
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(entries)?;

        // Write to temporary file first for atomic operation
        let temp_path = self.registry_path.with_extension("tmp");
        fs::write(&temp_path, &json)?;

        // Atomic rename to actual file
        fs::rename(&temp_path, &self.registry_path)?;

        debug!("Saved {} servers to registry", entries.len());
        Ok(())
    }

    /// Register a new server or update an existing one
    /// Uses transactional update: save to disk first, then update memory
    pub async fn register_server(
        &self,
        entry: ServerRegistryEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry_id = entry.id.clone();

        // Create new state with the entry added
        let new_entries = {
            let mut registry = self.entries.read().await.clone();
            registry.insert(entry_id.clone(), entry);
            registry
        };

        // Save to disk first (transactional boundary)
        self.save_entries_to_disk(&new_entries).await?;

        // Only update memory if save succeeded
        {
            let mut registry = self.entries.write().await;
            *registry = new_entries;
        }

        Ok(())
    }

    /// Remove a server from the registry
    /// Uses transactional update: save to disk first, then update memory
    pub async fn unregister_server(
        &self,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create new state with the entry removed
        let new_entries = {
            let mut registry = self.entries.read().await.clone();
            registry.remove(server_id);
            registry
        };

        // Save to disk first (transactional boundary)
        self.save_entries_to_disk(&new_entries).await?;

        // Only update memory if save succeeded
        {
            let mut registry = self.entries.write().await;
            *registry = new_entries;
        }

        Ok(())
    }

    /// Get all servers from the registry
    pub async fn get_all_servers(&self) -> Vec<ServerRegistryEntry> {
        let registry = self.entries.read().await;
        registry.values().cloned().collect()
    }

    /// Get a specific server by ID
    pub async fn get_server(&self, server_id: &str) -> Option<ServerRegistryEntry> {
        let registry = self.entries.read().await;
        registry.get(server_id).cloned()
    }

    /// Update server status
    /// Uses transactional update: save to disk first, then update memory
    pub async fn update_server_status(
        &self,
        server_id: &str,
        status: DevServerStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create new state with the status updated
        let new_entries = {
            let mut registry = self.entries.read().await.clone();
            if let Some(entry) = registry.get_mut(server_id) {
                entry.status = status;
                entry.last_seen = Utc::now();
            }
            registry
        };

        // Save to disk first (transactional boundary)
        self.save_entries_to_disk(&new_entries).await?;

        // Only update memory if save succeeded
        {
            let mut registry = self.entries.write().await;
            *registry = new_entries;
        }

        Ok(())
    }

    /// Get the configured stale timeout in minutes
    pub fn get_stale_timeout_minutes(&self) -> i64 {
        self.stale_timeout_minutes
    }

    /// Clean up stale entries based on configured timeout
    /// Removes servers that haven't been seen recently and whose process is no longer running
    /// Timeout can be configured via ORKEE_STALE_TIMEOUT_MINUTES environment variable (default: 5)
    pub async fn cleanup_stale_entries(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff = Utc::now() - chrono::Duration::minutes(self.stale_timeout_minutes);
        let mut to_remove = Vec::new();

        {
            let registry = self.entries.read().await;
            for (id, entry) in registry.iter() {
                if entry.last_seen < cutoff {
                    // Check if the process is still running with validation
                    if let Some(pid) = entry.pid {
                        if !is_process_running_validated(
                            pid,
                            Some(entry.started_at),
                            &["node", "python", "npm", "yarn", "bun", "pnpm", "deno"],
                        ) {
                            to_remove.push(id.clone());
                        }
                    } else {
                        to_remove.push(id.clone());
                    }
                }
            }
        }

        if !to_remove.is_empty() {
            let mut registry = self.entries.write().await;
            for id in to_remove {
                warn!("Removing stale server entry: {}", id);
                registry.remove(&id);
            }
            drop(registry);
            self.save_registry().await?;
        }

        Ok(())
    }

    /// Sync from preview-locks directory (for backwards compatibility)
    pub async fn sync_from_preview_locks(
        &self,
        api_port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let home = dirs::home_dir()
            .ok_or_else(|| "Could not determine home directory for preview locks sync")?;
        let locks_dir = home.join(".orkee").join("preview-locks");

        if !locks_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&locks_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        // Parse the lock file
                        if let Ok(lock_data) = serde_json::from_str::<serde_json::Value>(&content) {
                            // Convert lock file to registry entry
                            if let Some(project_id) = lock_data["project_id"].as_str() {
                                let server_id = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(project_id)
                                    .to_string();

                                let entry = ServerRegistryEntry {
                                    id: server_id.clone(),
                                    project_id: project_id.to_string(),
                                    project_name: None,
                                    project_root: PathBuf::from(
                                        lock_data["project_root"].as_str().unwrap_or("/"),
                                    ),
                                    port: lock_data["port"].as_u64().unwrap_or(0) as u16,
                                    pid: lock_data["pid"].as_u64().map(|p| p as u32),
                                    status: DevServerStatus::Running,
                                    preview_url: lock_data["preview_url"]
                                        .as_str()
                                        .map(|s| s.to_string()),
                                    framework_name: None,
                                    actual_command: None,
                                    started_at: lock_data["started_at"]
                                        .as_str()
                                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                        .map(|dt| dt.with_timezone(&Utc))
                                        .unwrap_or_else(Utc::now),
                                    last_seen: Utc::now(),
                                    api_port,
                                };

                                // Check if process is still running with validation
                                if let Some(pid) = entry.pid {
                                    if is_process_running_validated(
                                        pid,
                                        Some(entry.started_at),
                                        &["node", "python", "npm", "yarn", "bun", "pnpm", "deno"],
                                    ) {
                                        let mut registry = self.entries.write().await;
                                        registry.insert(server_id, entry);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read lock file {:?}: {}", path, e);
                    }
                }
            }
        }

        self.save_registry().await?;
        Ok(())
    }
}

/// Check if a process with the given PID is running and matches expected criteria
/// This prevents PID reuse attacks where a new process reuses an old PID
fn is_process_running_validated(
    pid: u32,
    expected_start_time: Option<DateTime<Utc>>,
    expected_name_patterns: &[&str], // e.g., ["node", "python", "npm"]
) -> bool {
    use sysinfo::{Pid, System};
    let mut system = System::new();
    system.refresh_processes();

    let pid_obj = Pid::from_u32(pid);

    if let Some(process) = system.process(pid_obj) {
        // Validate process name matches expected patterns (node/python/npm/etc.)
        let process_name = process.name().to_string().to_lowercase();
        let name_matches = expected_name_patterns.is_empty()
            || expected_name_patterns
                .iter()
                .any(|pattern| process_name.contains(pattern));

        if !name_matches {
            warn!(
                "PID {} exists but process name '{}' doesn't match expected patterns {:?} - likely PID reuse",
                pid, process_name, expected_name_patterns
            );
            return false;
        }

        // Validate start time if available (within 5 second tolerance for clock skew)
        if let Some(expected_time) = expected_start_time {
            let process_start_secs = process.start_time();
            let expected_unix = expected_time.timestamp() as u64;

            // Allow 5 second tolerance for clock skew
            if process_start_secs.abs_diff(expected_unix) > 5 {
                warn!(
                    "PID {} exists but start time mismatch (process: {}, expected: {}) - likely PID reuse",
                    pid,
                    process_start_secs,
                    expected_unix
                );
                return false;
            }
        }

        true
    } else {
        false
    }
}

// Singleton instance for global access
use once_cell::sync::Lazy;
pub static GLOBAL_REGISTRY: Lazy<ServerRegistry> = Lazy::new(ServerRegistry::new);

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test registry with a temporary directory
    fn create_test_registry() -> (ServerRegistry, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("server-registry.json");

        let registry = ServerRegistry {
            registry_path,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stale_timeout_minutes: 5,
        };

        (registry, temp_dir)
    }

    /// Helper to create a test server entry
    fn create_test_entry(id: &str, project_id: &str, port: u16) -> ServerRegistryEntry {
        ServerRegistryEntry {
            id: id.to_string(),
            project_id: project_id.to_string(),
            project_name: Some(format!("Test Project {}", project_id)),
            project_root: PathBuf::from("/test/path"),
            port,
            pid: Some(std::process::id()), // Use current process PID for testing
            status: DevServerStatus::Running,
            preview_url: Some(format!("http://localhost:{}", port)),
            framework_name: Some("vite".to_string()),
            actual_command: Some("npm run dev".to_string()),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: 4001,
        }
    }

    #[tokio::test]
    async fn test_register_and_get_server() {
        let (registry, _temp_dir) = create_test_registry();
        let entry = create_test_entry("server1", "proj1", 3000);

        // Register server
        registry.register_server(entry.clone()).await.unwrap();

        // Get server back
        let retrieved = registry.get_server("server1").await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "server1");
        assert_eq!(retrieved.project_id, "proj1");
        assert_eq!(retrieved.port, 3000);
    }

    #[tokio::test]
    async fn test_register_multiple_servers() {
        let (registry, _temp_dir) = create_test_registry();

        let entry1 = create_test_entry("server1", "proj1", 3000);
        let entry2 = create_test_entry("server2", "proj2", 3001);
        let entry3 = create_test_entry("server3", "proj3", 3002);

        registry.register_server(entry1).await.unwrap();
        registry.register_server(entry2).await.unwrap();
        registry.register_server(entry3).await.unwrap();

        let servers = registry.get_all_servers().await;
        assert_eq!(servers.len(), 3);
    }

    #[tokio::test]
    async fn test_unregister_server() {
        let (registry, _temp_dir) = create_test_registry();
        let entry = create_test_entry("server1", "proj1", 3000);

        // Register then unregister
        registry.register_server(entry).await.unwrap();
        assert!(registry.get_server("server1").await.is_some());

        registry.unregister_server("server1").await.unwrap();
        assert!(registry.get_server("server1").await.is_none());
    }

    #[tokio::test]
    async fn test_update_server_status() {
        let (registry, _temp_dir) = create_test_registry();
        let entry = create_test_entry("server1", "proj1", 3000);

        registry.register_server(entry).await.unwrap();

        // Update status
        registry
            .update_server_status("server1", DevServerStatus::Stopped)
            .await
            .unwrap();

        let retrieved = registry.get_server("server1").await.unwrap();
        assert_eq!(retrieved.status, DevServerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_transactional_register_persists_to_disk() {
        let (registry, temp_dir) = create_test_registry();
        let entry = create_test_entry("server1", "proj1", 3000);

        // Register server
        registry.register_server(entry).await.unwrap();

        // Verify file was created
        assert!(registry.registry_path.exists());

        // Create new registry instance with same path
        let registry2 = ServerRegistry {
            registry_path: temp_dir.path().join("server-registry.json"),
            entries: Arc::new(RwLock::new(HashMap::new())),
            stale_timeout_minutes: 5,
        };

        // Load from disk
        registry2.load_registry().await.unwrap();

        // Verify server was loaded
        let retrieved = registry2.get_server("server1").await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_transactional_unregister_persists_to_disk() {
        let (registry, temp_dir) = create_test_registry();
        let entry = create_test_entry("server1", "proj1", 3000);

        // Register then unregister
        registry.register_server(entry).await.unwrap();
        registry.unregister_server("server1").await.unwrap();

        // Create new registry instance
        let registry2 = ServerRegistry {
            registry_path: temp_dir.path().join("server-registry.json"),
            entries: Arc::new(RwLock::new(HashMap::new())),
            stale_timeout_minutes: 5,
        };

        registry2.load_registry().await.unwrap();

        // Verify server was removed
        assert!(registry2.get_server("server1").await.is_none());
    }

    #[tokio::test]
    async fn test_atomic_file_write() {
        let (registry, _temp_dir) = create_test_registry();
        let entry = create_test_entry("server1", "proj1", 3000);

        // Register server (uses atomic write)
        registry.register_server(entry).await.unwrap();

        // Verify main file exists
        assert!(registry.registry_path.exists());

        // Verify temp file was cleaned up
        let temp_path = registry.registry_path.with_extension("tmp");
        assert!(!temp_path.exists());
    }

    #[tokio::test]
    async fn test_update_existing_server() {
        let (registry, _temp_dir) = create_test_registry();
        let entry1 = create_test_entry("server1", "proj1", 3000);

        registry.register_server(entry1).await.unwrap();

        // Update with new port
        let mut entry2 = create_test_entry("server1", "proj1", 4000);
        entry2.status = DevServerStatus::Stopped;
        registry.register_server(entry2).await.unwrap();

        // Verify update
        let retrieved = registry.get_server("server1").await.unwrap();
        assert_eq!(retrieved.port, 4000);
        assert_eq!(retrieved.status, DevServerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_get_all_servers_empty() {
        let (registry, _temp_dir) = create_test_registry();
        let servers = registry.get_all_servers().await;
        assert_eq!(servers.len(), 0);
    }

    #[tokio::test]
    async fn test_get_nonexistent_server() {
        let (registry, _temp_dir) = create_test_registry();
        let result = registry.get_server("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_server() {
        let (registry, _temp_dir) = create_test_registry();
        // Should not panic or error
        let result = registry.unregister_server("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_status_nonexistent_server() {
        let (registry, _temp_dir) = create_test_registry();
        // Should not panic or error, just no-op
        let result = registry
            .update_server_status("nonexistent", DevServerStatus::Stopped)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stale_timeout_getter() {
        let (registry, _temp_dir) = create_test_registry();
        assert_eq!(registry.get_stale_timeout_minutes(), 5);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let (registry, _temp_dir) = create_test_registry();
        let registry = Arc::new(registry);

        // Spawn multiple concurrent tasks
        let mut handles = vec![];
        for i in 0..10 {
            let reg = registry.clone();
            let handle = tokio::spawn(async move {
                let entry =
                    create_test_entry(&format!("server{}", i), &format!("proj{}", i), 3000 + i);
                reg.register_server(entry).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all servers were registered
        let servers = registry.get_all_servers().await;
        assert_eq!(servers.len(), 10);
    }
}
