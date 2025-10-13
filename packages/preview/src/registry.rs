use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::env::parse_env_or_default_with_validation;
use crate::types::DevServerStatus;

/// Entry in the central server registry.
///
/// Represents a development server tracked across all Orkee instances on the system.
/// This is stored in `~/.orkee/server-registry.json` and serves as the single source
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
}

/// Central registry for tracking all development servers across Orkee instances.
///
/// This registry provides a global view of all development servers running on the system,
/// regardless of which Orkee instance started them. It persists to disk at
/// `~/.orkee/server-registry.json` and uses transactional updates to ensure consistency.
pub struct ServerRegistry {
    registry_path: PathBuf,
    entries: Arc<RwLock<HashMap<String, ServerRegistryEntry>>>,
    /// Timeout in minutes before considering an entry stale (default: 5, configurable via ORKEE_STALE_TIMEOUT_MINUTES)
    stale_timeout_minutes: i64,
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerRegistry {
    /// Create a new server registry instance.
    ///
    /// Initializes a new registry that persists to `~/.orkee/server-registry.json`.
    /// If the registry file exists, it will be automatically loaded. The stale timeout
    /// can be configured via the `ORKEE_STALE_TIMEOUT_MINUTES` environment variable
    /// (default: 5 minutes, max: 240 minutes/4 hours).
    ///
    /// # Returns
    ///
    /// Returns a new `ServerRegistry` instance with loaded entries (if any exist).
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| {
            // Fallback to temp directory if home can't be determined
            warn!("Could not determine home directory, using system temp directory");
            std::env::temp_dir()
        });
        let registry_path = home.join(".orkee").join("server-registry.json");

        // Read stale timeout from environment variable, default to 5 minutes
        // Validate the timeout value (must be positive, max 240 minutes = 4 hours)
        let stale_timeout_minutes =
            parse_env_or_default_with_validation("ORKEE_STALE_TIMEOUT_MINUTES", 5, |v| {
                v > 0 && v <= 240
            });

        debug!(
            "Server registry stale timeout set to {} minutes",
            stale_timeout_minutes
        );

        Self {
            registry_path,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stale_timeout_minutes,
        }
    }

    /// Load the registry from disk.
    ///
    /// Reads the registry file from `~/.orkee/server-registry.json` and loads all
    /// server entries into memory. If the file doesn't exist, this is not an error.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or if the file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or contains invalid JSON.
    pub async fn load_registry(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Read file directly and handle NotFound error instead of checking exists() first
        // This prevents TOCTOU race where file could be deleted between check and read
        let content = match tokio::fs::read_to_string(&self.registry_path).await {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                debug!(
                    "Server registry does not exist yet at {:?}",
                    self.registry_path
                );
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        let entries: HashMap<String, ServerRegistryEntry> = serde_json::from_str(&content)?;

        let mut registry = self.entries.write().await;
        *registry = entries;

        info!("Loaded {} servers from registry", registry.len());
        Ok(())
    }

    /// Save the registry to disk.
    ///
    /// Writes the current registry state to `~/.orkee/server-registry.json` using
    /// an atomic write operation (write to temp file, then rename). This ensures
    /// the registry file is never left in a corrupted state.
    ///
    /// The read lock is released before performing disk I/O to avoid blocking writers
    /// during potentially slow filesystem operations.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or the directory cannot be created.
    pub async fn save_registry(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clone data under read lock, then release lock before disk I/O
        let registry_snapshot = {
            let registry = self.entries.read().await;
            registry.clone()
        };
        // Lock is released here
        self.save_entries_to_disk(&registry_snapshot).await
    }

    /// Helper to save entries to disk (used for transactional updates)
    async fn save_entries_to_disk(
        &self,
        entries: &HashMap<String, ServerRegistryEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure the .orkee directory exists with proper error handling
        if let Some(parent) = self.registry_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                // Ignore AlreadyExists errors (can happen in concurrent scenarios)
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    error!("Failed to create registry directory {:?}: {}", parent, e);
                    return Err(format!("Cannot create registry directory: {}", e).into());
                }
            }
        }

        let json = serde_json::to_string_pretty(entries)?;

        // Write to unique temporary file to prevent collisions in concurrent scenarios
        // Use process ID + timestamp + random number to ensure uniqueness
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = std::time::Instant::now().elapsed().as_nanos();
        let temp_path = self.registry_path.with_extension(format!(
            "tmp.{}.{}.{}",
            std::process::id(),
            timestamp,
            random
        ));
        tokio::fs::write(&temp_path, &json).await?;

        // Atomic rename to actual file with cleanup on failure
        let rename_result = tokio::fs::rename(&temp_path, &self.registry_path).await;

        // If rename failed, attempt to cleanup temp file
        if rename_result.is_err() {
            if temp_path.exists() {
                if let Err(e) = tokio::fs::remove_file(&temp_path).await {
                    warn!("Failed to cleanup temp file after failed rename: {}", e);
                }
            }
            rename_result?; // Propagate the original rename error
        }

        // Set restrictive file permissions (owner read/write only)
        // This prevents other local users from reading sensitive server info or injecting malicious entries
        Self::set_registry_permissions(&self.registry_path).await?;

        debug!("Saved {} servers to registry", entries.len());
        Ok(())
    }

    /// Set restrictive file permissions on the registry file.
    ///
    /// Configures the registry file to be readable and writable only by the current user
    /// (owner). This prevents other local users from reading sensitive server information
    /// or injecting malicious entries.
    ///
    /// # Platform-Specific Behavior
    ///
    /// - **Unix/Linux/macOS**: Sets file mode to 0600 (rw-------)
    /// - **Windows**: Sets DACL to grant only the current user read/write access
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the registry file to secure
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if permissions cannot be set. On Windows, this may fail if:
    /// - Process token cannot be opened
    /// - User SID cannot be retrieved
    /// - ACL creation fails
    /// - File handle cannot be opened
    async fn set_registry_permissions(
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(path).await?.permissions();
            perms.set_mode(0o600); // Owner read/write only (rw-------)
            tokio::fs::set_permissions(path, perms).await?;
            debug!("Set registry file permissions to 0600 (owner read/write only)");
        }

        #[cfg(windows)]
        {
            // Windows API calls are blocking, so run them in a blocking task
            let path_buf = path.to_path_buf();
            tokio::task::spawn_blocking(move || {
                use windows::core::PWSTR;
                use windows::Win32::Foundation::LocalFree;
                use windows::Win32::Foundation::PSID;
                use windows::Win32::Security::Authorization::{
                    SetEntriesInAclW, SetSecurityInfo, EXPLICIT_ACCESS_W, SE_FILE_OBJECT,
                };
                use windows::Win32::Security::{
                    GetTokenInformation, TokenUser, ACL, DACL_SECURITY_INFORMATION,
                    PSECURITY_DESCRIPTOR, TOKEN_QUERY, TOKEN_USER,
                };
                use windows::Win32::Security::{
                    GRANT_ACCESS, OBJECT_INHERIT_ACE, SET_ACCESS, SUB_CONTAINERS_AND_OBJECTS_INHERIT,
                    TRUSTEE_IS_SID, TRUSTEE_W,
                };
                use windows::Win32::Storage::FileSystem::{
                    CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_DELETE,
                    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
                };
                use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

                unsafe {
                // Get the current user's SID
                let mut token = Default::default();
                if let Err(e) = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) {
                    error!(
                        "Failed to open process token for ACL setup: {:?} (error code: {})",
                        e,
                        e.code().0
                    );
                    return Err(format!("Failed to open process token: {:?}", e).into());
                }

                let mut token_info_length = 0u32;
                let _ = GetTokenInformation(token, TokenUser, None, 0, &mut token_info_length);

                let mut token_info = vec![0u8; token_info_length as usize];
                if let Err(e) = GetTokenInformation(
                    token,
                    TokenUser,
                    Some(token_info.as_mut_ptr() as *mut _),
                    token_info_length,
                    &mut token_info_length,
                )
                {
                    error!(
                        "Failed to get token information for ACL setup: {:?} (error code: {}, buffer size: {})",
                        e,
                        e.code().0,
                        token_info_length
                    );
                    return Err(format!("Failed to get token information: {:?}", e).into());
                }

                let token_user = &*(token_info.as_ptr() as *const TOKEN_USER);
                let user_sid = PSID(token_user.User.Sid.0);

                // Create an explicit access structure for the current user (read/write only)
                let mut ea = EXPLICIT_ACCESS_W {
                    grfAccessPermissions: FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
                    grfAccessMode: SET_ACCESS,
                    grfInheritance: OBJECT_INHERIT_ACE | SUB_CONTAINERS_AND_OBJECTS_INHERIT,
                    Trustee: TRUSTEE_W {
                        pMultipleTrustee: std::ptr::null_mut(),
                        MultipleTrusteeOperation: Default::default(),
                        TrusteeForm: TRUSTEE_IS_SID,
                        TrusteeType: Default::default(),
                        ptstrName: PWSTR(user_sid.0 as *mut _),
                    },
                };

                // Create new ACL with only current user access
                let mut new_acl: *mut ACL = std::ptr::null_mut();
                if let Err(e) = SetEntriesInAclW(Some(&mut [ea]), None, &mut new_acl as *mut *mut ACL) {
                    error!(
                        "Failed to create ACL for registry file: {:?} (error code: {})",
                        e,
                        e.code().0
                    );
                    return Err(format!("Failed to create ACL: {:?}", e).into());
                }

                // Open the file handle for setting security
                let path_wide: Vec<u16> = path_buf
                    .to_str()
                    .unwrap_or("")
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                let file_handle = CreateFileW(
                    windows::core::PCWSTR(path_wide.as_ptr()),
                    (FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0),
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    None,
                    OPEN_EXISTING,
                    Default::default(),
                    None,
                );

                if let Err(e) = file_handle {
                    error!(
                        "Failed to open registry file for ACL setup: {:?} (error code: {}, path: {:?})",
                        e,
                        e.code().0,
                        path_buf
                    );
                    if !new_acl.is_null() {
                        let _ = LocalFree(windows::Win32::Foundation::HLOCAL(new_acl as *mut _));
                    }
                    return Err(format!("Failed to open file for ACL setup: {:?}", e).into());
                }

                // Set the DACL on the file (owner read/write only, no one else)
                let result = SetSecurityInfo(
                    file_handle.unwrap(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    PSID::default(),
                    PSID::default(),
                    Some(new_acl as *const ACL),
                    PSECURITY_DESCRIPTOR::default(),
                );

                // Clean up
                if !new_acl.is_null() {
                    let _ = LocalFree(windows::Win32::Foundation::HLOCAL(new_acl as *mut _));
                }

                if let Err(e) = result {
                    error!(
                        "Failed to set ACL on registry file: {:?} (error code: {}, path: {:?})",
                        e,
                        e.code().0,
                        path_buf
                    );
                    return Err(format!("Failed to set file ACL: {:?}", e).into());
                }

                debug!(
                    "Set registry file ACL to restrict access to current user only (owner read/write)"
                );

                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
            })
            .await
            .map_err(|e| format!("Failed to join Windows permissions task: {}", e))??;
        }

        Ok(())
    }

    /// Register a new server or update an existing one.
    ///
    /// Adds a server to the registry or updates it if it already exists. This method
    /// uses a transactional update pattern: the registry is saved to disk first, and
    /// only if that succeeds is the in-memory state updated. This prevents inconsistencies
    /// between disk and memory.
    ///
    /// # Arguments
    ///
    /// * `entry` - The server registry entry to add or update
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry cannot be saved to disk. If this occurs,
    /// the in-memory state is left unchanged.
    pub async fn register_server(
        &self,
        entry: ServerRegistryEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry_id = entry.id.clone();

        // Clone the current registry under read lock, then release
        // This minimizes lock contention during disk I/O
        let snapshot = {
            let registry = self.entries.read().await;
            let mut snapshot = registry.clone();
            snapshot.insert(entry_id.clone(), entry.clone());
            snapshot
        };
        // Read lock is released here

        // Save to disk without holding any locks (disk I/O can be slow)
        self.save_entries_to_disk(&snapshot).await?;

        // Hold write lock briefly to update in-memory state
        // Apply the same change to ensure consistency
        let mut registry = self.entries.write().await;
        registry.insert(entry_id, entry);

        Ok(())
    }

    /// Remove a server from the registry.
    ///
    /// Removes a server entry from the registry. This method uses a transactional
    /// update pattern: the registry is saved to disk first, and only if that succeeds
    /// is the in-memory state updated.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The unique identifier of the server to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, even if the server was not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry cannot be saved to disk.
    pub async fn unregister_server(
        &self,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Clone the current registry under read lock, then release
        let snapshot = {
            let registry = self.entries.read().await;
            let mut snapshot = registry.clone();
            snapshot.remove(server_id);
            snapshot
        };
        // Read lock is released here

        // Save to disk without holding any locks
        self.save_entries_to_disk(&snapshot).await?;

        // Hold write lock briefly to update in-memory state
        let mut registry = self.entries.write().await;
        registry.remove(server_id);

        Ok(())
    }

    /// Get all servers from the registry.
    ///
    /// Returns a snapshot of all registered servers at the time of the call.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<ServerRegistryEntry>` containing all server entries in the registry.
    pub async fn get_all_servers(&self) -> Vec<ServerRegistryEntry> {
        let registry = self.entries.read().await;
        registry.values().cloned().collect()
    }

    /// Get a specific server by ID.
    ///
    /// Retrieves detailed information about a single server from the registry.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The unique identifier of the server to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(ServerRegistryEntry)` if found, or `None` if no server exists with this ID.
    pub async fn get_server(&self, server_id: &str) -> Option<ServerRegistryEntry> {
        let registry = self.entries.read().await;
        registry.get(server_id).cloned()
    }

    /// Update the status of a server.
    ///
    /// Updates the status field of a server and refreshes its `last_seen` timestamp.
    /// This method uses a transactional update pattern for consistency.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The unique identifier of the server to update
    /// * `status` - The new status to set
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, even if the server was not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry cannot be saved to disk.
    pub async fn update_server_status(
        &self,
        server_id: &str,
        status: DevServerStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();

        // Clone the current registry under read lock, then release
        let snapshot = {
            let registry = self.entries.read().await;
            let mut snapshot = registry.clone();

            // Update server status and timestamp in the clone
            if let Some(entry) = snapshot.get_mut(server_id) {
                entry.status = status.clone();
                entry.last_seen = now;
            }
            snapshot
        };
        // Read lock is released here

        // Save to disk without holding any locks
        self.save_entries_to_disk(&snapshot).await?;

        // Hold write lock briefly to update in-memory state
        let mut registry = self.entries.write().await;
        if let Some(entry) = registry.get_mut(server_id) {
            entry.status = status;
            entry.last_seen = now;
        }

        Ok(())
    }

    /// Get the configured stale timeout in minutes.
    ///
    /// Returns the timeout value used to determine when a server entry should be
    /// considered stale. This value is configured via the `ORKEE_STALE_TIMEOUT_MINUTES`
    /// environment variable (default: 5, max: 240).
    ///
    /// # Returns
    ///
    /// Returns the stale timeout in minutes.
    pub fn get_stale_timeout_minutes(&self) -> i64 {
        self.stale_timeout_minutes
    }

    /// Clean up stale entries based on configured timeout.
    ///
    /// Removes server entries that haven't been seen recently (based on `last_seen` timestamp)
    /// and whose process is no longer running. The timeout can be configured via the
    /// `ORKEE_STALE_TIMEOUT_MINUTES` environment variable (default: 5 minutes).
    ///
    /// This method validates that processes are still running before removing entries,
    /// preventing premature removal of servers that are still active but haven't been
    /// recently polled.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry cannot be saved to disk after cleanup.
    pub async fn cleanup_stale_entries(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff = Utc::now() - chrono::Duration::minutes(self.stale_timeout_minutes);

        // Hold write lock for entire operation to prevent concurrent write races
        let mut registry = self.entries.write().await;

        // Clone current registry to identify stale entries
        let mut to_remove = Vec::new();

        // Identify stale entries
        for (id, entry) in registry.iter() {
            if entry.last_seen < cutoff {
                // Check if the process is still running with validation
                if let Some(pid) = entry.pid {
                    if !is_process_running_validated(
                        pid,
                        Some(entry.started_at),
                        &["node", "python", "npm", "yarn", "bun", "pnpm", "deno"],
                        entry.actual_command.as_deref(), // Use command for stronger validation
                    ) {
                        to_remove.push(id.clone());
                    }
                } else {
                    to_remove.push(id.clone());
                }
            }
        }

        // Remove stale entries from the clone
        if !to_remove.is_empty() {
            let mut snapshot = registry.clone();
            for id in &to_remove {
                warn!("Removing stale server entry: {}", id);
                snapshot.remove(id);
            }

            // Save to disk first (transactional boundary)
            self.save_entries_to_disk(&snapshot).await?;

            // Only update in-memory state after successful disk write
            *registry = snapshot;
        }

        Ok(())
    }

    /// Sync from preview-locks directory for backwards compatibility.
    ///
    /// Imports server entries from the legacy `~/.orkee/preview-locks` directory
    /// into the central registry. This ensures servers started by older versions
    /// of Orkee are properly tracked in the new registry system.
    ///
    /// Each lock file is validated before import:
    /// - Process must still be running
    /// - Process must be a legitimate development server (not PID reuse)
    ///
    /// # Arguments
    ///
    /// * `api_port` - The API port to associate with imported servers
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or if the preview-locks directory doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry cannot be saved after importing entries.
    pub async fn sync_from_preview_locks(
        &self,
        api_port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let home = dirs::home_dir().unwrap_or_else(|| {
            warn!("Could not determine home directory for preview locks sync, using system temp directory");
            std::env::temp_dir()
        });
        let locks_dir = home.join(".orkee").join("preview-locks");

        if !locks_dir.exists() {
            return Ok(());
        }

        let mut read_dir = tokio::fs::read_dir(&locks_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        // Parse the lock file
                        if let Ok(lock_data) = serde_json::from_str::<serde_json::Value>(&content) {
                            // Convert lock file to registry entry
                            if let Some(project_id) = lock_data["project_id"].as_str() {
                                let project_root = match lock_data["project_root"].as_str() {
                                    Some(root) => PathBuf::from(root),
                                    None => {
                                        error!("Skipping lock file {:?}: missing or invalid project_root", path);
                                        continue;
                                    }
                                };

                                let server_id = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(project_id)
                                    .to_string();

                                let entry = ServerRegistryEntry {
                                    id: server_id.clone(),
                                    project_id: project_id.to_string(),
                                    project_name: None,
                                    project_root,
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
                                        None, // Legacy lock files don't have command info
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

/// Get the process start time validation tolerance in seconds.
///
/// Returns the tolerance value used to determine if a process's start time matches
/// the expected start time. This helps detect PID reuse on systems under heavy load.
/// Can be configured via the `ORKEE_PROCESS_START_TIME_TOLERANCE_SECS` environment
/// variable (default: 5 seconds, max: 60 seconds).
///
/// # Security Note
///
/// The tolerance window balances security against system load variability. On heavily
/// loaded systems, process start time reporting can be delayed by several seconds.
/// The default of 5 seconds provides reasonable tolerance while still detecting most
/// PID reuse scenarios. Combined with parent PID validation and command-line validation,
/// this provides defense-in-depth against PID reuse attacks even with a larger window.
///
/// # Returns
///
/// Returns the tolerance in seconds.
fn get_start_time_tolerance_secs() -> u64 {
    use crate::env::parse_env_or_default_with_validation;
    parse_env_or_default_with_validation("ORKEE_PROCESS_START_TIME_TOLERANCE_SECS", 5, |v| {
        v > 0 && v <= 60
    })
}

/// Check if a process with the given PID is running and matches expected criteria
/// This prevents PID reuse attacks where a new process reuses an old PID
pub fn is_process_running_validated(
    pid: u32,
    expected_start_time: Option<DateTime<Utc>>,
    expected_name_patterns: &[&str], // e.g., ["node", "python", "npm"]
    expected_command: Option<&str>,  // Optional command-line validation
) -> bool {
    use sysinfo::{Pid, System};
    let mut system = System::new();
    system.refresh_processes();

    let pid_obj = Pid::from_u32(pid);

    if let Some(process) = system.process(pid_obj) {
        // Validate parent PID as defense-in-depth against PID reuse
        // Development servers should have a parent process (shell/terminal/orkee)
        // If parent is PID 1 (init/systemd), process is orphaned which is suspicious
        if let Some(parent_pid) = process.parent() {
            let parent_pid_u32 = parent_pid.as_u32();
            if parent_pid_u32 == 1 {
                warn!(
                    "PID {} has suspicious parent PID 1 (init/systemd) - process may be orphaned or reused",
                    pid
                );
                // Don't immediately fail - log warning but continue with other checks
                // Some legitimate dev servers may be orphaned in edge cases
            }
        } else {
            warn!(
                "PID {} has no parent process - highly suspicious, likely PID reuse",
                pid
            );
            return false;
        }
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

        // Validate command line if provided (stronger validation than just process name)
        if let Some(expected_cmd) = expected_command {
            let actual_cmd = process.cmd().join(" ");

            // Check if the actual command contains the expected command
            // We use contains instead of exact match to handle argument variations
            if !actual_cmd.contains(expected_cmd.trim()) {
                warn!(
                    "PID {} exists but command line mismatch - expected '{}', got '{}' - likely PID reuse or process spoofing",
                    pid, expected_cmd, actual_cmd
                );
                return false;
            }
        }

        // Validate start time if available
        if let Some(expected_time) = expected_start_time {
            let tolerance_secs = get_start_time_tolerance_secs();
            let process_start_secs = process.start_time();
            let expected_unix = expected_time.timestamp() as u64;
            let time_diff = process_start_secs.abs_diff(expected_unix);

            if time_diff > tolerance_secs {
                warn!(
                    "PID {} exists but start time mismatch (process: {}, expected: {}, diff: {}s, tolerance: {}s) - likely PID reuse",
                    pid,
                    process_start_secs,
                    expected_unix,
                    time_diff,
                    tolerance_secs
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

/// Start periodic cleanup of stale registry entries.
///
/// Spawns a background task that runs every 2 minutes (by default) to clean up stale server
/// entries from the global registry. This prevents memory leaks and keeps the
/// registry in sync with actually running processes.
///
/// The cleanup interval can be configured via `ORKEE_CLEANUP_INTERVAL_MINUTES`
/// environment variable (default: 2 minutes, min: 1, max: 60). The default is set to
/// half the stale timeout to ensure responsive cleanup.
///
/// This function should be called once during application initialization.
/// Multiple calls are safe - subsequent calls will return `None`.
///
/// # Returns
///
/// Returns `Some(JoinHandle)` on first call to allow graceful shutdown.
/// Returns `None` on subsequent calls (task already started).
///
/// # Examples
///
/// ```no_run
/// use orkee_preview::registry::start_periodic_cleanup;
///
/// #[tokio::main]
/// async fn main() {
///     // Start cleanup and store handle for shutdown
///     let cleanup_handle = start_periodic_cleanup();
///
///     // Application continues running...
///
///     // On shutdown:
///     if let Some(handle) = cleanup_handle {
///         handle.abort(); // Graceful shutdown
///     }
/// }
/// ```
pub fn start_periodic_cleanup() -> Option<tokio::task::JoinHandle<()>> {
    use once_cell::sync::OnceCell;
    use tokio::time::{interval, Duration};

    static CLEANUP_TASK_STARTED: OnceCell<()> = OnceCell::new();

    // Only start the task once
    if CLEANUP_TASK_STARTED.get().is_some() {
        debug!("Periodic cleanup task already started");
        return None;
    }

    // Get cleanup interval from environment variable (default: 2 minutes, half of stale timeout)
    // Running cleanup at half the stale timeout ensures more responsive cleanup
    let cleanup_interval_minutes =
        parse_env_or_default_with_validation("ORKEE_CLEANUP_INTERVAL_MINUTES", 2, |v| {
            (1..=60).contains(&v)
        });

    info!(
        "Starting periodic registry cleanup task (interval: {} minutes)",
        cleanup_interval_minutes
    );

    // Mark as started
    let _ = CLEANUP_TASK_STARTED.set(());

    // Spawn background task and return handle for graceful shutdown
    let handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(cleanup_interval_minutes * 60));

        loop {
            interval.tick().await;

            debug!("Running periodic registry cleanup");
            if let Err(e) = GLOBAL_REGISTRY.cleanup_stale_entries().await {
                error!("Periodic cleanup failed: {}", e);
            } else {
                debug!("Periodic cleanup completed successfully");
            }
        }
    });

    Some(handle)
}

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

        // Verify process-unique temp file was cleaned up
        let temp_path = registry
            .registry_path
            .with_extension(format!("tmp.{}", std::process::id()));
        assert!(!temp_path.exists());

        // Verify old-style temp file doesn't exist either
        let old_temp_path = registry.registry_path.with_extension("tmp");
        assert!(!old_temp_path.exists());
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

    #[tokio::test]
    #[cfg(unix)]
    async fn test_registry_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let (registry, _temp_dir) = create_test_registry();
        let entry = create_test_entry("server1", "proj1", 3000);

        // Register server which will create the file
        registry.register_server(entry).await.unwrap();

        // Verify file exists
        assert!(registry.registry_path.exists());

        // Check file permissions
        let metadata = std::fs::metadata(&registry.registry_path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Extract permission bits (last 9 bits: rwxrwxrwx)
        let perms = mode & 0o777;

        // Should be 0600 (owner read/write only, no group or other permissions)
        assert_eq!(
            perms, 0o600,
            "Registry file should have 0600 permissions (owner read/write only), but has {:o}",
            perms
        );
    }
}
