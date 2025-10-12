use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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
}

impl ServerRegistry {
    /// Create a new server registry instance
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Could not determine home directory");
        let registry_path = home.join(".orkee").join("server-registry.json");

        let registry = Self {
            registry_path,
            entries: Arc::new(RwLock::new(HashMap::new())),
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
        // Ensure the .orkee directory exists
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let registry = self.entries.read().await;
        let json = serde_json::to_string_pretty(&*registry)?;
        fs::write(&self.registry_path, json)?;

        debug!("Saved {} servers to registry", registry.len());
        Ok(())
    }

    /// Register a new server or update an existing one
    pub async fn register_server(
        &self,
        entry: ServerRegistryEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut registry = self.entries.write().await;
            registry.insert(entry.id.clone(), entry);
        }
        self.save_registry().await?;
        Ok(())
    }

    /// Remove a server from the registry
    pub async fn unregister_server(
        &self,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut registry = self.entries.write().await;
            registry.remove(server_id);
        }
        self.save_registry().await?;
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
    pub async fn update_server_status(
        &self,
        server_id: &str,
        status: DevServerStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut registry = self.entries.write().await;
            if let Some(entry) = registry.get_mut(server_id) {
                entry.status = status;
                entry.last_seen = Utc::now();
            }
        }
        self.save_registry().await?;
        Ok(())
    }

    /// Clean up stale entries (servers that haven't been seen in 5 minutes)
    pub async fn cleanup_stale_entries(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff = Utc::now() - chrono::Duration::minutes(5);
        let mut to_remove = Vec::new();

        {
            let registry = self.entries.read().await;
            for (id, entry) in registry.iter() {
                if entry.last_seen < cutoff {
                    // Check if the process is still running
                    if let Some(pid) = entry.pid {
                        if !is_process_running(pid) {
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
    pub async fn sync_from_preview_locks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let home = dirs::home_dir().expect("Could not determine home directory");
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
                                    api_port: 4001, // Default API port
                                };

                                // Check if process is still running
                                if let Some(pid) = entry.pid {
                                    if is_process_running(pid) {
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

/// Check if a process with the given PID is running
fn is_process_running(pid: u32) -> bool {
    // Use sysinfo to check process
    use sysinfo::{Pid, System};
    let mut system = System::new();
    system.refresh_processes();

    let pid = Pid::from_u32(pid);
    system.process(pid).is_some()
}

// Singleton instance for global access
use once_cell::sync::Lazy;
pub static GLOBAL_REGISTRY: Lazy<ServerRegistry> = Lazy::new(ServerRegistry::new);
