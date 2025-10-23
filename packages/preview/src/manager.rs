use crate::registry::{ServerRegistryEntry, GLOBAL_REGISTRY};
use crate::types::*;
use chrono::Utc;
use orkee_config::constants;
use orkee_config::env::parse_env_or_default_with_validation;
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// SSE event broadcast channel capacity.
///
/// This determines how many events can be buffered per subscriber before the
/// subscriber is marked as lagged. A value of 100 allows for brief bursts of
/// rapid server changes without forcing clients to resync.
const SSE_CHANNEL_CAPACITY: usize = 100;

/// Result of spawning a development server process with associated metadata.
///
/// This struct contains the child process handle along with information about
/// the command that was executed and the detected framework.
#[derive(Debug)]
pub struct SpawnResult {
    /// The spawned child process handle
    pub child: Child,
    /// The complete command string that was executed (e.g., "npm run dev")
    pub command: String,
    /// The detected framework name (e.g., "Vite", "Next.js", "React")
    pub framework: String,
}

/// Crash-resistant preview server manager.
///
/// Manages multiple development servers across different projects, providing
/// automatic process recovery, log capture, and lifecycle management. This manager
/// is designed to be crash-resistant and can recover servers from previous sessions.
#[derive(Clone)]
pub struct PreviewManager {
    active_servers: Arc<RwLock<HashMap<String, ServerInfo>>>,
    server_logs: Arc<RwLock<HashMap<String, VecDeque<DevServerLog>>>>,
    event_tx: broadcast::Sender<ServerEvent>,
}

/// Information about a running development server.
///
/// Contains all relevant metadata about a development server including its process ID,
/// port, status, and framework information. This struct is used to track and manage
/// the lifecycle of development servers.
#[derive(Debug)]
pub struct ServerInfo {
    /// Unique identifier for this server instance
    pub id: Uuid,
    /// Project identifier that this server is associated with
    pub project_id: String,
    /// Port number the server is listening on
    pub port: u16,
    /// Process ID of the running server (if available)
    pub pid: Option<u32>,
    /// Current status of the server (Running, Stopped, Error, etc.)
    pub status: DevServerStatus,
    /// Full preview URL for accessing the server (e.g., "http://localhost:3000")
    pub preview_url: Option<String>,
    /// Optional handle to the child process (not cloned)
    pub child: Option<Arc<RwLock<Child>>>,
    /// The actual command that was executed to start the server
    pub actual_command: Option<String>,
    /// Detected or configured framework name (e.g., "Vite", "Next.js")
    pub framework_name: Option<String>,
    /// Background tasks for log capture (stdout/stderr readers)
    /// Must be aborted when server stops to prevent resource leaks
    pub log_tasks: Option<Vec<tokio::task::JoinHandle<()>>>,
    /// Source of the server (Orkee, External, or Discovered)
    pub source: crate::types::ServerSource,
    /// ID of the matched project (for external/discovered servers)
    pub matched_project_id: Option<String>,
}

impl Clone for ServerInfo {
    fn clone(&self) -> Self {
        ServerInfo {
            id: self.id,
            project_id: self.project_id.clone(),
            port: self.port,
            pid: self.pid,
            status: self.status.clone(),
            preview_url: self.preview_url.clone(),
            child: None, // Don't clone the child process handle
            actual_command: self.actual_command.clone(),
            framework_name: self.framework_name.clone(),
            log_tasks: None, // Don't clone the log capture tasks
            source: self.source,
            matched_project_id: self.matched_project_id.clone(),
        }
    }
}

impl Default for PreviewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate project_id to prevent path traversal attacks
///
/// Project IDs must contain only alphanumeric characters, hyphens, and underscores.
/// This prevents attackers from using path traversal sequences like "../../../etc/passwd"
/// to access arbitrary files on the filesystem.
fn validate_project_id(project_id: &str) -> Result<(), PreviewError> {
    if project_id.is_empty() {
        return Err(PreviewError::InvalidProjectId {
            project_id: project_id.to_string(),
            reason: "Project ID cannot be empty".to_string(),
        });
    }

    // Check for path traversal sequences
    if project_id.contains("..") || project_id.contains('/') || project_id.contains('\\') {
        return Err(PreviewError::InvalidProjectId {
            project_id: project_id.to_string(),
            reason: "Project ID cannot contain path traversal sequences (.. / \\)".to_string(),
        });
    }

    // Only allow alphanumeric, dash, and underscore
    if !project_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(PreviewError::InvalidProjectId {
            project_id: project_id.to_string(),
            reason: "Project ID can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        });
    }

    Ok(())
}

impl PreviewManager {
    /// Create a new preview manager without recovery.
    ///
    /// Creates a fresh preview manager instance with no active servers or logs.
    /// For most use cases, prefer [`new_with_recovery`](Self::new_with_recovery) which
    /// automatically recovers previously running servers.
    ///
    /// # Returns
    ///
    /// Returns a new `PreviewManager` instance with empty server and log collections.
    pub fn new() -> Self {
        // Make channel capacity configurable via environment variable
        let capacity = parse_env_or_default_with_validation(
            "ORKEE_EVENT_CHANNEL_SIZE",
            SSE_CHANNEL_CAPACITY,
            |v| (10..=10000).contains(&v),
        );

        let (event_tx, _rx) = broadcast::channel(capacity);
        Self {
            active_servers: Arc::new(RwLock::new(HashMap::new())),
            server_logs: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Subscribe to server events for real-time updates
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.event_tx.subscribe()
    }

    /// Create a new manager and recover existing servers from lock files.
    ///
    /// This is the recommended way to create a `PreviewManager`. It performs the following:
    /// 1. Syncs from preview-locks directory (backwards compatibility)
    /// 2. Recovers servers from the central registry
    /// 3. Validates that processes are still running before restoring them
    ///
    /// # Returns
    ///
    /// Returns a new `PreviewManager` instance with recovered servers loaded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orkee_preview::manager::PreviewManager;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let manager = PreviewManager::new_with_recovery().await;
    ///     let servers = manager.list_servers().await;
    ///     println!("Recovered {} servers", servers.len());
    /// }
    /// ```
    pub async fn new_with_recovery() -> Self {
        let manager = Self::new();

        // Get the API port from environment variable or use default
        let api_port = std::env::var(constants::ORKEE_API_PORT)
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4001);

        // Load existing registry from disk first
        if let Err(e) = GLOBAL_REGISTRY.load_registry().await {
            warn!("Failed to load registry from disk: {}", e);
        }

        // Sync from preview-locks to central registry
        if let Err(e) = GLOBAL_REGISTRY.sync_from_preview_locks(api_port).await {
            warn!("Failed to sync from preview locks: {}", e);
        }

        // Clean up stale entries from previous sessions
        if let Err(e) = GLOBAL_REGISTRY.cleanup_stale_entries().await {
            warn!("Failed to cleanup stale registry entries: {}", e);
        }

        // Recover existing servers from lock files (backwards compatibility)
        if let Err(e) = manager.recover_servers().await {
            warn!("Failed to recover servers: {}", e);
        }

        // Also load servers from the central registry
        let registry_servers = GLOBAL_REGISTRY.get_all_servers().await;
        for entry in registry_servers {
            // Only add if not already in our local list
            let mut servers = manager.active_servers.write().await;
            if let std::collections::hash_map::Entry::Vacant(e) =
                servers.entry(entry.project_id.clone())
            {
                let server_info = ServerInfo {
                    id: Uuid::new_v4(), // Generate new ID
                    project_id: entry.project_id.clone(),
                    port: entry.port,
                    pid: entry.pid,
                    status: entry.status,
                    preview_url: entry.preview_url,
                    child: None,
                    actual_command: entry.actual_command,
                    framework_name: entry.framework_name,
                    log_tasks: None,
                    source: entry.source,
                    matched_project_id: entry.matched_project_id.clone(),
                };
                e.insert(server_info);
            }
        }

        manager
    }

    /// Add a log entry for a project
    async fn add_log(&self, project_id: &str, log_type: LogType, message: String) {
        let log_entry = DevServerLog {
            timestamp: Utc::now(),
            log_type,
            message,
        };

        let mut logs = self.server_logs.write().await;
        let project_logs = logs
            .entry(project_id.to_string())
            .or_insert_with(VecDeque::new);

        // Add the log entry
        project_logs.push_back(log_entry);

        // Keep only the last 1000 entries to prevent memory issues
        if project_logs.len() > 1000 {
            project_logs.pop_front();
        }
    }

    /// Get logs for a development server.
    ///
    /// Retrieves log entries for a specific project's development server. Logs can be
    /// filtered by timestamp and limited to a maximum number of entries.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The unique identifier of the project
    /// * `since` - Optional timestamp to filter logs newer than this time
    /// * `limit` - Optional maximum number of log entries to return
    ///
    /// # Returns
    ///
    /// Returns a `Vec<DevServerLog>` containing the filtered log entries. If no logs
    /// exist for the project, returns an empty vector.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orkee_preview::manager::PreviewManager;
    /// use chrono::Utc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let manager = PreviewManager::new_with_recovery().await;
    ///
    ///     // Get last 50 logs from the last 5 minutes
    ///     let five_mins_ago = Utc::now() - chrono::Duration::minutes(5);
    ///     let logs = manager.get_server_logs("my-project", Some(five_mins_ago), Some(50)).await;
    ///
    ///     for log in logs {
    ///         println!("[{:?}] {}", log.log_type, log.message);
    ///     }
    /// }
    /// ```
    pub async fn get_server_logs(
        &self,
        project_id: &str,
        since: Option<chrono::DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<DevServerLog> {
        let logs = self.server_logs.read().await;

        if let Some(project_logs) = logs.get(project_id) {
            let mut filtered_logs: Vec<DevServerLog> = if let Some(since_time) = since {
                project_logs
                    .iter()
                    .filter(|log| log.timestamp > since_time)
                    .cloned()
                    .collect()
            } else {
                project_logs.iter().cloned().collect()
            };

            // Apply limit if specified
            if let Some(max_count) = limit {
                if filtered_logs.len() > max_count {
                    filtered_logs = filtered_logs
                        .into_iter()
                        .rev()
                        .take(max_count)
                        .rev()
                        .collect();
                }
            }

            filtered_logs
        } else {
            Vec::new()
        }
    }

    /// Clear all logs for a project.
    ///
    /// Removes all log entries associated with a specific project. This is useful
    /// for freeing memory or starting with a clean log state.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The unique identifier of the project whose logs should be cleared
    pub async fn clear_server_logs(&self, project_id: &str) {
        let mut logs = self.server_logs.write().await;
        logs.remove(project_id);
        info!("Cleared logs for project: {}", project_id);
    }

    /// Extract port from server log line if detected
    fn extract_port_from_log(&self, line: &str) -> Option<u16> {
        // Common patterns for dev server port detection
        let patterns = [
            r"Local:\s+http://localhost:(\d+)", // Vite: "Local:   http://localhost:5174/"
            r"Local server:\s+http://localhost:(\d+)", // Some frameworks
            r"Running at http://localhost:(\d+)", // Express/other servers
            r"Server ready at http://localhost:(\d+)", // Next.js dev
            r"server running on port (\d+)",    // Express: "Express server running on port 8476"
            r"📍 http://localhost:(\d+)",       // Express with emoji: "📍 http://localhost:8476"
            r"🚀.*port (\d+)", // Express startup: "🚀 Express server running on port 8476"
            r"ready - started server on.*:(\d+)", // Next.js: "ready - started server on 0.0.0.0:3000"
            r"http://localhost:(\d+)",            // Generic http://localhost pattern
            r"localhost:(\d+)",                   // Generic localhost pattern
        ];

        for pattern in &patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(line) {
                    if let Some(port_match) = captures.get(1) {
                        if let Ok(port) = port_match.as_str().parse::<u16>() {
                            return Some(port);
                        }
                    }
                }
            }
        }
        None
    }

    /// Check if a log line is a successful HTTP access log (not an error)
    fn is_successful_http_log(&self, line: &str) -> bool {
        // Pattern for HTTP access logs: IP - - [timestamp] "METHOD path HTTP/version" status -
        // Example: ::1 - - [07/Sep/2025 12:25:39] "GET / HTTP/1.1" 200 -
        let http_log_pattern = r#"^[:\w\.-]+ - - \[[^\]]+\] "[A-Z]+ [^"]+ HTTP/[\d\.]+" (\d{3}) -"#;

        if let Ok(regex) = regex::Regex::new(http_log_pattern) {
            if let Some(captures) = regex.captures(line) {
                if let Some(status_match) = captures.get(1) {
                    if let Ok(status_code) = status_match.as_str().parse::<u16>() {
                        // HTTP status codes 200-399 are success/redirect (not errors)
                        return (200..400).contains(&status_code);
                    }
                }
            }
        }

        false
    }

    /// Update server with detected port
    async fn update_server_port(&self, project_id: &str, new_port: u16) {
        let mut servers = self.active_servers.write().await;
        if let Some(server_info) = servers.get_mut(project_id) {
            if server_info.port != new_port {
                info!(
                    "Detected port change for project {}: {} -> {}",
                    project_id, server_info.port, new_port
                );
                server_info.port = new_port;
                server_info.preview_url = Some(format!("http://localhost:{}", new_port));
                self.add_log(
                    project_id,
                    LogType::System,
                    format!("Updated preview URL to http://localhost:{}", new_port),
                )
                .await;
            }
        }
    }

    /// Capture logs from a child process handle (takes mutable reference)
    ///
    /// This method takes stdout/stderr from the child process and spawns tasks
    /// to capture and log output. The child handle remains available for cleanup.
    /// Returns JoinHandles for the spawned tasks so they can be aborted when stopping.
    async fn capture_process_logs_from_handle(
        &self,
        project_id: &str,
        child: &mut Child,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let project_id_clone = project_id.to_string();
        let manager = self.clone();
        let mut handles = Vec::new();

        // Get stdout handle
        if let Some(stdout) = child.stdout.take() {
            let project_id_stdout = project_id_clone.clone();
            let manager_stdout = manager.clone();
            let handle = tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Check for port detection in the log line
                    if let Some(detected_port) = manager_stdout.extract_port_from_log(&line) {
                        manager_stdout
                            .update_server_port(&project_id_stdout, detected_port)
                            .await;
                    }
                    manager_stdout
                        .add_log(&project_id_stdout, LogType::Stdout, line)
                        .await;
                }
            });
            handles.push(handle);
        }

        // Get stderr handle
        if let Some(stderr) = child.stderr.take() {
            let project_id_stderr = project_id_clone.clone();
            let manager_stderr = manager.clone();
            let handle = tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Check for port detection in the log line (some servers log to stderr)
                    if let Some(detected_port) = manager_stderr.extract_port_from_log(&line) {
                        manager_stderr
                            .update_server_port(&project_id_stderr, detected_port)
                            .await;
                    }

                    // Filter out successful HTTP access logs from being marked as STDERR
                    let log_type = if manager_stderr.is_successful_http_log(&line) {
                        LogType::System // HTTP access logs are informational, not errors
                    } else {
                        LogType::Stderr // Real errors stay as STDERR
                    };

                    manager_stderr
                        .add_log(&project_id_stderr, log_type, line)
                        .await;
                }
            });
            handles.push(handle);
        }

        handles
    }

    /// Start a development server for a project.
    ///
    /// Spawns a new development server process for the specified project. This method:
    /// - Detects the project type and framework automatically
    /// - Allocates an available port (preferring consistent ports per project)
    /// - Starts the appropriate development command (npm run dev, vite, etc.)
    /// - Captures stdout/stderr logs automatically
    /// - Creates persistence lock files for crash recovery
    ///
    /// If a server is already running for this project, returns the existing server info.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Unique identifier for the project
    /// * `project_root` - Absolute path to the project's root directory
    ///
    /// # Returns
    ///
    /// Returns `Ok(ServerInfo)` containing details about the started server, including
    /// the allocated port and preview URL.
    ///
    /// # Errors
    ///
    /// * `PreviewError::PortInUse` - No available ports in range 8000-8999
    /// * `PreviewError::ProcessSpawnError` - Failed to spawn the server process
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orkee_preview::manager::PreviewManager;
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let manager = PreviewManager::new_with_recovery().await;
    ///     let project_root = PathBuf::from("/path/to/my-app");
    ///
    ///     match manager.start_server("my-app".to_string(), project_root).await {
    ///         Ok(info) => println!("Server started at {}", info.preview_url.unwrap()),
    ///         Err(e) => eprintln!("Failed to start server: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn start_server(
        &self,
        project_id: String,
        project_root: PathBuf,
    ) -> PreviewResult<ServerInfo> {
        info!("Starting preview server for: {}", project_id);

        // Check if server already exists or is in the process of stopping
        {
            let servers = self.active_servers.read().await;
            if let Some(existing) = servers.get(&project_id) {
                match existing.status {
                    DevServerStatus::Running => {
                        info!("Server already running for project: {}", project_id);
                        return Ok(existing.clone());
                    }
                    DevServerStatus::Stopping => {
                        info!(
                            "Server is currently stopping for project: {}, waiting for it to finish...",
                            project_id
                        );
                        // Release read lock before waiting
                        drop(servers);

                        // Wait for the background task to complete cleanup
                        // This prevents race condition where start() is called while stop() is in progress
                        let max_wait_ms = 6000; // Wait slightly longer than stop's 5s timeout
                        let poll_interval_ms = 100;
                        let mut elapsed_ms = 0;

                        while elapsed_ms < max_wait_ms {
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                poll_interval_ms,
                            ))
                            .await;
                            elapsed_ms += poll_interval_ms;

                            let servers = self.active_servers.read().await;
                            if !servers.contains_key(&project_id) {
                                info!(
                                    "Previous server stopped after {}ms, proceeding with start",
                                    elapsed_ms
                                );
                                break;
                            }
                        }

                        if elapsed_ms >= max_wait_ms {
                            warn!(
                                "Server for project {} is still stopping after {}ms - cannot start yet",
                                project_id, max_wait_ms
                            );
                            return Err(PreviewError::ServerAlreadyRunning {
                                project_id: project_id.clone(),
                            });
                        }
                    }
                    _ => {
                        // Other states (Starting, Crashed, etc.) - allow restart
                    }
                }
            }
        }

        // Find available port using project-based allocation (8000-8999 range)
        let port = self.find_available_port(&project_id).await?;

        // Create server info
        let server_info = ServerInfo {
            id: Uuid::new_v4(),
            project_id: project_id.clone(),
            port,
            pid: None,
            status: DevServerStatus::Starting,
            preview_url: Some(format!("http://localhost:{}", port)),
            child: None,
            actual_command: None,
            framework_name: None,
            log_tasks: None,
            source: crate::types::ServerSource::Orkee,
            matched_project_id: None,
        };

        // Try to start the server
        match self.spawn_server(&server_info, &project_root).await {
            Ok(spawn_result) => {
                let pid = spawn_result.child.id();

                // Wrap child in Arc<RwLock<>> so we can store it and use it for log capture
                let child_handle = Arc::new(RwLock::new(spawn_result.child));

                // Start capturing logs from the process and store the task handles
                // This prevents resource leaks when servers are stopped
                let log_handles = {
                    let mut child = child_handle.write().await;
                    self.capture_process_logs_from_handle(&project_id, &mut child)
                        .await
                };

                let mut updated_info = server_info;
                updated_info.pid = pid;
                updated_info.status = DevServerStatus::Running;
                updated_info.child = Some(child_handle); // Store the child handle
                updated_info.log_tasks = Some(log_handles); // Store log capture task handles
                updated_info.actual_command = Some(spawn_result.command);
                updated_info.framework_name = Some(spawn_result.framework);

                // Store the server info
                {
                    let mut servers = self.active_servers.write().await;
                    servers.insert(project_id.clone(), updated_info.clone());
                }

                // Create lock file for persistence
                if let Err(e) = self.create_lock_file(&updated_info, &project_root).await {
                    warn!(
                        "Failed to create lock file for project {}: {}",
                        project_id, e
                    );
                }

                // Emit ServerStarted event AFTER all state updates
                // Ordering: (1) in-memory state updated, (2) disk persistence attempted, (3) event emitted
                // This ensures subscribers receive events only after state is fully consistent
                let _ = self.event_tx.send(ServerEvent::ServerStarted {
                    project_id: project_id.clone(),
                    pid: pid.unwrap_or(0),
                    port,
                    framework: updated_info.framework_name.clone(),
                });

                info!(
                    "Successfully started server for project: {} on port {}",
                    project_id, port
                );
                Ok(updated_info)
            }
            Err(e) => {
                error!("Failed to start server for project {}: {}", project_id, e);

                // Emit ServerError event
                let _ = self.event_tx.send(ServerEvent::ServerError {
                    project_id: project_id.clone(),
                    error: e.to_string(),
                });

                // Don't store failed server attempts in active_servers to avoid port allocation leaks.
                // The port was never actually bound, so storing the error entry would mislead
                // other parts of the system into thinking the port is in use.

                Err(e)
            }
        }
    }

    /// Stop a running development server.
    ///
    /// Stops the development server for the specified project by:
    /// - Sending a termination signal to the process
    /// - Removing the server from active tracking
    /// - Cleaning up lock files
    /// - Removing from the central registry
    ///
    /// This method is safe to call even if the server is not running or has already stopped.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The unique identifier of the project whose server should be stopped
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server was successfully stopped or was not running.
    ///
    /// # Errors
    ///
    /// Returns an error if lock file cleanup fails, though the server process is still terminated.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orkee_preview::manager::PreviewManager;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let manager = PreviewManager::new_with_recovery().await;
    ///
    ///     if let Err(e) = manager.stop_server("my-app").await {
    ///         eprintln!("Error stopping server: {}", e);
    ///     }
    /// }
    /// ```
    pub async fn stop_server(&self, project_id: &str) -> PreviewResult<()> {
        info!("Stopping server for project: {}", project_id);

        let server_info = {
            let servers = self.active_servers.read().await;
            servers.get(project_id).cloned()
        };

        if let Some(mut info) = server_info {
            // Add stop log
            self.add_log(
                project_id,
                LogType::System,
                format!("Stopping server with PID: {:?}", info.pid),
            )
            .await;

            // Immediately set status to Stopping for instant UI feedback
            info.status = DevServerStatus::Stopping;
            {
                let mut servers = self.active_servers.write().await;
                servers.insert(project_id.to_string(), info.clone());
            }

            // Abort log capture tasks to prevent resource leaks
            if let Some(log_tasks) = info.log_tasks {
                let task_count = log_tasks.len();
                for task in log_tasks {
                    task.abort();
                }
                info!(
                    "Aborted {} log capture tasks for project {}",
                    task_count, project_id
                );
            }

            // Try to kill using child handle first (preferred method)
            let mut killed_via_handle = false;
            if let Some(child_handle) = &info.child {
                match child_handle.write().await.kill().await {
                    Ok(_) => {
                        info!(
                            "Successfully sent kill signal to process for project {} using child handle",
                            project_id
                        );
                        self.add_log(
                            project_id,
                            LogType::System,
                            "Kill signal sent via child handle".to_string(),
                        )
                        .await;
                        killed_via_handle = true;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to kill process for project {} using child handle: {}",
                            project_id, e
                        );
                    }
                }
            }

            // Fall back to PID-based kill if child handle wasn't available or failed
            if !killed_via_handle {
                if let Some(pid) = info.pid {
                    // Try to kill the process - ignore errors if process is already dead
                    if let Err(e) = self.kill_process(pid).await {
                        warn!(
                            "Failed to kill process {} for project {}: {} (process may already be dead)",
                            pid, project_id, e
                        );
                        self.add_log(
                            project_id,
                            LogType::System,
                            format!("Process {} was not running (already stopped)", pid),
                        )
                        .await;
                    }
                }
            }

            // Spawn background task to monitor process termination
            // This prevents blocking the API response while waiting for process to die
            let manager = self.clone();
            let project_id_owned = project_id.to_string();
            let pid_to_monitor = info.pid;

            tokio::spawn(async move {
                // Wait for process to actually terminate before removing from registry
                if let Some(pid) = pid_to_monitor {
                    let max_wait_ms = 5000; // Wait up to 5 seconds
                    let poll_interval_ms = 50; // Check every 50ms
                    let mut elapsed_ms = 0;

                    while elapsed_ms < max_wait_ms {
                        if !manager.is_process_running(pid) {
                            info!(
                                "Process {} confirmed terminated after {}ms",
                                pid, elapsed_ms
                            );
                            manager
                                .add_log(
                                    &project_id_owned,
                                    LogType::System,
                                    format!("Process {} terminated (verified)", pid),
                                )
                                .await;
                            break;
                        }

                        tokio::time::sleep(tokio::time::Duration::from_millis(poll_interval_ms))
                            .await;
                        elapsed_ms += poll_interval_ms;
                    }

                    if elapsed_ms >= max_wait_ms {
                        warn!(
                            "Process {} for project {} did not terminate within {}ms, proceeding anyway",
                            pid, project_id_owned, max_wait_ms
                        );
                    }
                }

                // Remove from active servers (only after process is actually stopped)
                {
                    let mut servers = manager.active_servers.write().await;
                    servers.remove(&project_id_owned);
                }

                // Clean up logs to prevent unbounded memory growth
                {
                    let mut logs = manager.server_logs.write().await;
                    if let Some(removed_logs) = logs.remove(&project_id_owned) {
                        debug!(
                            "Cleaned up {} log entries for stopped server: {}",
                            removed_logs.len(),
                            project_id_owned
                        );
                    }
                }

                // Remove lock file
                if let Err(e) = manager.remove_lock_file(&project_id_owned).await {
                    warn!(
                        "Failed to remove lock file for project {}: {}",
                        project_id_owned, e
                    );
                }

                // Emit ServerStopped event
                let _ = manager.event_tx.send(ServerEvent::ServerStopped {
                    project_id: project_id_owned.clone(),
                });

                info!(
                    "Successfully stopped server for project: {}",
                    project_id_owned
                );
            });
        }

        Ok(())
    }

    /// Get the status of a development server.
    ///
    /// Retrieves information about a running or previously running development server
    /// for the specified project. Checks both local servers and the central registry.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The unique identifier of the project
    ///
    /// # Returns
    ///
    /// Returns `Some(ServerInfo)` if a server exists for this project, or `None` if
    /// no server has been started or it has been stopped and removed.
    pub async fn get_server_status(&self, project_id: &str) -> Option<ServerInfo> {
        // Check local servers first
        {
            let servers = self.active_servers.read().await;
            if let Some(server_info) = servers.get(project_id) {
                return Some(server_info.clone());
            }
        }

        // If not found locally, check the global registry
        let registry_servers = GLOBAL_REGISTRY.get_all_servers().await;
        for entry in registry_servers {
            if entry.project_id == project_id {
                // Parse UUID with fallback to new UUID if invalid
                let id = match Uuid::parse_str(&entry.id) {
                    Ok(uuid) => uuid,
                    Err(err) => {
                        warn!(
                            "Invalid UUID '{}' in registry entry for project {}: {}. Generating new UUID.",
                            entry.id, entry.project_id, err
                        );
                        Uuid::new_v4()
                    }
                };

                return Some(ServerInfo {
                    id,
                    project_id: entry.project_id.clone(),
                    port: entry.port,
                    pid: entry.pid,
                    status: entry.status,
                    preview_url: entry.preview_url,
                    child: None,
                    actual_command: entry.actual_command,
                    framework_name: entry.framework_name,
                    log_tasks: None,
                    source: entry.source,
                    matched_project_id: entry.matched_project_id.clone(),
                });
            }
        }

        None
    }

    /// List all active development servers.
    ///
    /// Returns a combined list of servers from both the local manager and the central
    /// registry. This provides a complete view of all development servers across all
    /// Orkee instances on the system.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<ServerInfo>` containing information about all tracked servers.
    /// Servers from the central registry are included if they are not already in the
    /// local list to avoid duplicates.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orkee_preview::manager::PreviewManager;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let manager = PreviewManager::new_with_recovery().await;
    ///     let servers = manager.list_servers().await;
    ///
    ///     for server in servers {
    ///         println!("Project: {} - Status: {:?} - Port: {}",
    ///             server.project_id, server.status, server.port);
    ///     }
    /// }
    /// ```
    pub async fn list_servers(&self) -> Vec<ServerInfo> {
        // Get local servers first
        let local_servers = self.active_servers.read().await;
        let mut all_servers: HashMap<String, ServerInfo> = local_servers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Also get servers from the global registry
        let registry_servers = GLOBAL_REGISTRY.get_all_servers().await;
        for entry in registry_servers {
            // Add servers from registry if not already in local list
            if let std::collections::hash_map::Entry::Vacant(e) =
                all_servers.entry(entry.project_id.clone())
            {
                // Parse UUID with fallback to new UUID if invalid
                let id = match Uuid::parse_str(&entry.id) {
                    Ok(uuid) => uuid,
                    Err(err) => {
                        warn!(
                            "Invalid UUID '{}' in registry entry for project {}: {}. Generating new UUID.",
                            entry.id, entry.project_id, err
                        );
                        Uuid::new_v4()
                    }
                };

                let server_info = ServerInfo {
                    id,
                    project_id: entry.project_id.clone(),
                    port: entry.port,
                    pid: entry.pid,
                    status: entry.status,
                    preview_url: entry.preview_url,
                    child: None,
                    actual_command: entry.actual_command,
                    framework_name: entry.framework_name,
                    log_tasks: None,
                    source: entry.source,
                    matched_project_id: entry.matched_project_id.clone(),
                };
                e.insert(server_info);
            }
        }

        all_servers.into_values().collect()
    }

    /// Get preferred port for a project (consistent across restarts)
    fn get_preferred_port(&self, project_id: &str) -> u16 {
        let mut hasher = DefaultHasher::new();
        project_id.hash(&mut hasher);
        let hash = hasher.finish();
        8000 + (hash % 1000) as u16
    }

    /// Find an available port starting from project's preferred port in range 8000-8999
    async fn find_available_port(&self, project_id: &str) -> PreviewResult<u16> {
        let preferred = self.get_preferred_port(project_id);

        // Try preferred port first
        if self.is_port_available(preferred).await {
            info!(
                "Using preferred port {} for project {}",
                preferred, project_id
            );
            return Ok(preferred);
        }

        // Scan range 8000-8999 starting from preferred
        for offset in 1..1000 {
            let port = 8000 + ((preferred - 8000 + offset) % 1000);
            if self.is_port_available(port).await {
                info!(
                    "Using alternative port {} for project {} (preferred {} was taken)",
                    port, project_id, preferred
                );
                return Ok(port);
            }
        }

        error!(
            "No available ports in range 8000-8999 for project {}",
            project_id
        );
        Err(PreviewError::PortInUse { port: preferred })
    }

    /// Check if port is available
    /// Checks both TCP binding and global registry to prevent conflicts with external servers
    async fn is_port_available(&self, port: u16) -> bool {
        // First check if we can bind to the port
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_err() {
            return false;
        }

        // Also check the global registry for external/discovered servers
        // This prevents race conditions where an external server is registered but not yet in active_servers
        let all_servers = GLOBAL_REGISTRY.get_all_servers().await;
        for server in all_servers {
            if server.port == port {
                debug!(
                    "Port {} is in use by server {} (source: {:?})",
                    port, server.id, server.source
                );
                return false;
            }
        }

        true
    }

    /// Spawn a server process based on project type
    async fn spawn_server(
        &self,
        server_info: &ServerInfo,
        project_root: &Path,
    ) -> PreviewResult<SpawnResult> {
        // Check for package.json first - if it has dev scripts, prefer dev commands
        if project_root.join("package.json").exists() {
            // Try development commands for Node.js projects
            self.spawn_dev_command(server_info, project_root).await
        } else if project_root.join("index.html").exists() {
            // Simple static file server for pure HTML projects
            self.spawn_static_server(server_info, project_root).await
        } else {
            // For other projects, try common dev commands as fallback
            self.spawn_dev_command(server_info, project_root).await
        }
    }

    /// Spawn a simple static file server
    async fn spawn_static_server(
        &self,
        server_info: &ServerInfo,
        project_root: &Path,
    ) -> PreviewResult<SpawnResult> {
        // Use Python's built-in HTTP server as it's reliable and simple
        let mut cmd = Command::new("python3");
        cmd.args(["-m", "http.server", &server_info.port.to_string()])
            .current_dir(project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Add initial log
        self.add_log(
            &server_info.project_id,
            LogType::System,
            format!(
                "Starting static HTTP server on port {} in {}",
                server_info.port,
                project_root.display()
            ),
        )
        .await;

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                info!("Spawned static server with PID: {:?}", pid);
                self.add_log(
                    &server_info.project_id,
                    LogType::System,
                    format!("Static server started successfully with PID: {:?}", pid),
                )
                .await;
                Ok(SpawnResult {
                    child,
                    command: "python3 -m http.server".to_string(),
                    framework: "Static HTTP Server".to_string(),
                })
            }
            Err(e) => {
                error!("Failed to spawn static server: {}", e);
                self.add_log(
                    &server_info.project_id,
                    LogType::System,
                    format!("Failed to start static server: {}", e),
                )
                .await;
                Err(PreviewError::ProcessSpawnError {
                    command: "python3 -m http.server".to_string(),
                    error: e.to_string(),
                })
            }
        }
    }

    /// Check if a package.json script exists
    async fn has_npm_script(&self, project_root: &Path, script_name: &str) -> bool {
        let package_json_path = project_root.join("package.json");
        if let Ok(content) = fs::read_to_string(package_json_path).await {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
                return package_json
                    .get("scripts")
                    .and_then(|scripts| scripts.get(script_name))
                    .is_some();
            }
        }
        false
    }

    /// Detect framework based on command and project files
    async fn detect_framework(&self, command: &str, project_root: &Path) -> String {
        // Check command first
        if command.contains("vite")
            || command.contains("npm run dev") && self.has_dependency(project_root, "vite").await
        {
            return "Vite".to_string();
        }
        if command.contains("next") || self.has_dependency(project_root, "next").await {
            return "Next.js".to_string();
        }
        if command.contains("react-scripts")
            || self.has_dependency(project_root, "react-scripts").await
        {
            return "Create React App".to_string();
        }
        if command.contains("vue") || self.has_dependency(project_root, "vue").await {
            return "Vue".to_string();
        }
        if command.contains("angular") || command.contains("ng serve") {
            return "Angular".to_string();
        }
        if command.contains("python") {
            return "Python HTTP Server".to_string();
        }

        // Default
        "Development Server".to_string()
    }

    /// Check if project has a dependency in package.json
    async fn has_dependency(&self, project_root: &Path, dep_name: &str) -> bool {
        let package_json_path = project_root.join("package.json");
        if let Ok(content) = fs::read_to_string(package_json_path).await {
            if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check dependencies and devDependencies
                for deps_key in ["dependencies", "devDependencies"] {
                    if let Some(deps) = package_json.get(deps_key).and_then(|d| d.as_object()) {
                        if deps.contains_key(dep_name) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Spawn a development command
    async fn spawn_dev_command(
        &self,
        server_info: &ServerInfo,
        project_root: &Path,
    ) -> PreviewResult<SpawnResult> {
        let port_str = server_info.port.to_string();

        // Add initial log
        self.add_log(
            &server_info.project_id,
            LogType::System,
            format!(
                "Attempting to start development server on port {} in {}",
                server_info.port,
                project_root.display()
            ),
        )
        .await;

        // Check for npm/yarn scripts first if package.json exists
        let mut commands = Vec::new();

        if project_root.join("package.json").exists() {
            // Check for common dev scripts in order of preference
            if self.has_npm_script(project_root, "dev").await {
                commands.push(("npm", vec!["run", "dev"]));
            }
            if self.has_npm_script(project_root, "start").await {
                commands.push(("npm", vec!["start"]));
            }
            // Add yarn alternatives if npm scripts exist
            if self.has_npm_script(project_root, "dev").await {
                commands.push(("yarn", vec!["dev"]));
            }
        }

        // Add fallback commands
        commands.push(("python3", vec!["-m", "http.server", port_str.as_str()]));

        for (cmd, args) in &commands {
            let mut command = Command::new(cmd);
            command
                .args(args)
                .current_dir(project_root)
                .env("PORT", server_info.port.to_string())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::null());

            self.add_log(
                &server_info.project_id,
                LogType::System,
                format!("Trying command: {} {}", cmd, args.join(" ")),
            )
            .await;

            if let Ok(child) = command.spawn() {
                let pid = child.id();
                info!(
                    "Spawned dev server with command '{}' and PID: {:?}",
                    cmd, pid
                );

                let command_str = format!("{} {}", cmd, args.join(" "));
                let framework = self.detect_framework(&command_str, project_root).await;

                self.add_log(
                    &server_info.project_id,
                    LogType::System,
                    format!(
                        "Development server started successfully with command '{}' and PID: {:?}",
                        cmd, pid
                    ),
                )
                .await;

                return Ok(SpawnResult {
                    child,
                    command: command_str,
                    framework,
                });
            }
        }

        self.add_log(
            &server_info.project_id,
            LogType::System,
            "No suitable development server command found".to_string(),
        )
        .await;

        Err(PreviewError::ProcessSpawnError {
            command: "No suitable dev command found".to_string(),
            error: "Could not start any development server".to_string(),
        })
    }

    /// Kill a process by PID with graceful shutdown and verification
    async fn kill_process(&self, _pid: u32) -> PreviewResult<()> {
        #[cfg(unix)]
        {
            let pid = _pid; // Use the parameter on Unix
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            use sysinfo::{Pid as SysPid, System};
            use tokio::time::{sleep, Duration};

            let nix_pid = Pid::from_raw(pid as i32);
            let sys_pid = SysPid::from_u32(pid);

            // Send SIGTERM for graceful shutdown
            match kill(nix_pid, Signal::SIGTERM) {
                Ok(_) => {
                    info!("Sent SIGTERM to process with PID: {}", pid);
                }
                Err(e) => {
                    // Process might already be dead
                    warn!("Failed to send SIGTERM to PID {}: {}", pid, e);
                    return Err(PreviewError::ProcessKillError {
                        pid,
                        error: format!("Failed to send SIGTERM: {}", e),
                    });
                }
            }

            // Wait up to 5 seconds for graceful shutdown
            let mut attempts = 0;
            let max_attempts = 10; // Check every 500ms for 5 seconds
            while attempts < max_attempts {
                sleep(Duration::from_millis(500)).await;

                let mut system = System::new();
                system.refresh_processes();

                if system.process(sys_pid).is_none() {
                    info!("Process {} terminated gracefully after SIGTERM", pid);
                    return Ok(());
                }

                attempts += 1;
            }

            // Process still running, send SIGKILL
            warn!(
                "Process {} did not respond to SIGTERM, sending SIGKILL",
                pid
            );
            match kill(nix_pid, Signal::SIGKILL) {
                Ok(_) => {
                    info!("Sent SIGKILL to process with PID: {}", pid);
                }
                Err(e) => {
                    warn!("Failed to send SIGKILL to PID {}: {}", pid, e);
                    return Err(PreviewError::ProcessKillError {
                        pid,
                        error: format!("Failed to send SIGKILL: {}", e),
                    });
                }
            }

            // Wait for forced termination (shorter timeout)
            attempts = 0;
            let max_force_attempts = 4; // Check every 500ms for 2 seconds
            while attempts < max_force_attempts {
                sleep(Duration::from_millis(500)).await;

                let mut system = System::new();
                system.refresh_processes();

                if system.process(sys_pid).is_none() {
                    info!("Process {} terminated after SIGKILL", pid);
                    return Ok(());
                }

                attempts += 1;
            }

            // Process still exists after SIGKILL - this is very unusual
            error!("Process {} did not terminate even after SIGKILL", pid);
            Err(PreviewError::ProcessKillError {
                pid,
                error: "Process did not terminate even after SIGKILL".to_string(),
            })
        }

        #[cfg(not(unix))]
        {
            warn!("Process killing not implemented for this platform");
            Ok(())
        }
    }

    // === PERSISTENCE METHODS ===

    /// Get lock file path for a project
    fn get_lock_file_path(&self, project_id: &str) -> Result<PathBuf, PreviewError> {
        // Validate project_id to prevent path traversal attacks
        validate_project_id(project_id)?;

        // Use dirs crate for more reliable home directory detection across platforms
        let home_dir = dirs::home_dir().unwrap_or_else(|| {
            warn!("Could not determine home directory, using current directory");
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

        Ok(home_dir
            .join(".orkee")
            .join("preview-locks")
            .join(format!("{}.json", project_id)))
    }

    /// Check if a process is running by PID (simple check, no validation)
    /// Used for confirming process termination after kill signal
    fn is_process_running(&self, pid: u32) -> bool {
        use sysinfo::{Pid, System};

        let mut system = System::new();
        system.refresh_processes();
        system.process(Pid::from_u32(pid)).is_some()
    }

    /// Check if a process is running and matches our spawned process
    /// This prevents PID reuse attacks where a new process reuses an old PID
    fn is_process_running_validated(
        &self,
        pid: u32,
        _expected_start_time: Option<chrono::DateTime<Utc>>,
        _expected_cwd: Option<&Path>,
    ) -> bool {
        #[cfg(unix)]
        {
            let expected_start_time = _expected_start_time; // Use on Unix
            let expected_cwd = _expected_cwd; // Use on Unix
            use sysinfo::{Pid, System};

            let mut system = System::new();
            system.refresh_processes();
            let pid_obj = Pid::from_u32(pid);

            if let Some(process) = system.process(pid_obj) {
                // Validate process name (should be dev server related)
                let name = process.name().to_string().to_lowercase();
                let is_dev_process = ["node", "python", "npm", "yarn", "bun", "pnpm", "deno"]
                    .iter()
                    .any(|pattern| name.contains(pattern));

                if !is_dev_process {
                    warn!(
                        "PID {} exists but process '{}' is not a dev server (likely PID reuse)",
                        pid, name
                    );
                    return false;
                }

                // Validate start time if we have it
                if let Some(expected) = expected_start_time {
                    let process_start_secs = process.start_time();
                    let expected_unix = expected.timestamp() as u64;

                    // Use minimal tolerance (1 second) to prevent PID reuse attacks
                    // Configurable via ORKEE_PROCESS_START_TIME_TOLERANCE_SECS (max 1 second)
                    let tolerance_secs = parse_env_or_default_with_validation(
                        constants::ORKEE_PROCESS_START_TIME_TOLERANCE_SECS,
                        1,
                        |v| v > 0 && v <= 1,
                    );

                    if process_start_secs.abs_diff(expected_unix) > tolerance_secs {
                        warn!(
                            "PID {} exists but start time mismatch (process: {}, expected: {}, diff: {}s, tolerance: {}s) - likely PID reuse",
                            pid,
                            process_start_secs,
                            expected_unix,
                            process_start_secs.abs_diff(expected_unix),
                            tolerance_secs
                        );
                        return false;
                    }
                }

                // Validate current working directory if we have it
                // This is the most reliable way to detect PID reuse
                if let Some(expected_path) = expected_cwd {
                    if let Some(process_cwd) = process.cwd() {
                        // Canonicalize both paths for comparison
                        let expected_canonical = expected_path.canonicalize().ok();
                        let process_canonical = process_cwd.canonicalize().ok();

                        if expected_canonical.is_none()
                            || process_canonical.is_none()
                            || expected_canonical != process_canonical
                        {
                            warn!(
                                "PID {} exists but cwd mismatch (process: {:?}, expected: {:?}) - likely PID reuse",
                                pid, process_cwd, expected_path
                            );
                            return false;
                        }
                    } else {
                        warn!(
                            "PID {} exists but cannot read cwd (process may have changed directories) - rejecting for security",
                            pid
                        );
                        return false;
                    }
                }

                true
            } else {
                false
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, use sysinfo instead of assuming
            use sysinfo::{Pid, System};

            let mut system = System::new();
            system.refresh_processes();
            system.process(Pid::from_u32(pid)).is_some()
        }
    }

    /// Create lock file when starting server
    async fn create_lock_file(
        &self,
        server_info: &ServerInfo,
        project_root: &Path,
    ) -> PreviewResult<()> {
        let lock_data = ServerLockData {
            project_id: server_info.project_id.clone(),
            pid: server_info.pid.unwrap_or(0),
            port: server_info.port,
            started_at: Utc::now(),
            preview_url: server_info.preview_url.clone().unwrap_or_default(),
            project_root: project_root.to_string_lossy().to_string(),
        };

        let lock_path = self.get_lock_file_path(&server_info.project_id)?;
        let lock_json = serde_json::to_string_pretty(&lock_data)
            .map_err(|e| PreviewError::IoError(std::io::Error::other(e)))?;

        // Ensure directory exists
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(PreviewError::IoError)?;
        }

        fs::write(&lock_path, lock_json)
            .await
            .map_err(PreviewError::IoError)?;

        info!(
            "Created lock file for project: {} at {:?}",
            server_info.project_id, lock_path
        );

        // Also register with the central registry
        let registry_entry = ServerRegistryEntry {
            id: server_info.id.to_string(),
            project_id: server_info.project_id.clone(),
            project_name: None, // Could be fetched from project manager if available
            project_root: project_root.to_path_buf(),
            port: server_info.port,
            pid: server_info.pid,
            status: server_info.status.clone(),
            preview_url: server_info.preview_url.clone(),
            framework_name: server_info.framework_name.clone(),
            actual_command: server_info.actual_command.clone(),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: std::env::var(constants::ORKEE_API_PORT)
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4001),
            source: server_info.source,
            matched_project_id: server_info.matched_project_id.clone(),
        };

        if let Err(e) = GLOBAL_REGISTRY.register_server(registry_entry).await {
            warn!("Failed to register server in central registry: {}", e);
        }

        Ok(())
    }

    /// Remove lock file when stopping server
    async fn remove_lock_file(&self, project_id: &str) -> PreviewResult<()> {
        let lock_path = self.get_lock_file_path(project_id)?;
        if lock_path.exists() {
            fs::remove_file(&lock_path)
                .await
                .map_err(PreviewError::IoError)?;
            info!("Removed lock file for project: {}", project_id);
        }

        // Also remove from the central registry (need to find the server ID first)
        let registry_servers = GLOBAL_REGISTRY.get_all_servers().await;
        for entry in registry_servers {
            if entry.project_id == project_id {
                if let Err(e) = GLOBAL_REGISTRY.unregister_server(&entry.id).await {
                    warn!("Failed to unregister server from central registry: {}", e);
                }
                break;
            }
        }

        Ok(())
    }

    /// Recover development servers from lock files on startup.
    ///
    /// Scans the `~/.orkee/preview-locks` directory for lock files and attempts to
    /// recover previously running development servers. This method:
    /// - Validates that the process is still running using PID and start time
    /// - Verifies the process is a legitimate development server (not PID reuse)
    /// - Removes stale lock files for dead processes
    ///
    /// This is automatically called by [`new_with_recovery`](Self::new_with_recovery).
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success. Errors during recovery are logged but do not
    /// prevent the method from completing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orkee_preview::manager::PreviewManager;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let manager = PreviewManager::new();
    ///     manager.recover_servers().await.expect("Failed to recover servers");
    /// }
    /// ```
    pub async fn recover_servers(&self) -> PreviewResult<()> {
        // Use dirs crate for more reliable home directory detection across platforms
        let home_dir = dirs::home_dir().unwrap_or_else(|| {
            warn!("Could not determine home directory for recovery, using current directory");
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

        let lock_dir = home_dir.join(".orkee").join("preview-locks");

        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&lock_dir).await {
            warn!("Failed to create lock directory: {}", e);
            return Ok(());
        }

        // Read all lock files
        let mut entries = match fs::read_dir(&lock_dir).await {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read lock directory: {}", e);
                return Ok(());
            }
        };

        let mut recovered_count = 0;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("json"))
                && self.recover_single_server(path).await.is_ok()
            {
                recovered_count += 1;
            }
        }

        if recovered_count > 0 {
            info!(
                "Recovered {} preview servers from lock files",
                recovered_count
            );
        }

        Ok(())
    }

    /// Recover a single server from lock file
    async fn recover_single_server(&self, lock_path: PathBuf) -> PreviewResult<()> {
        let content = match fs::read_to_string(&lock_path).await {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read lock file {:?}: {}", lock_path, e);
                return Err(PreviewError::IoError(e));
            }
        };

        let lock_data: ServerLockData = match serde_json::from_str(&content) {
            Ok(data) => data,
            Err(e) => {
                warn!("Invalid lock file {:?}, removing: {}", lock_path, e);
                let _ = fs::remove_file(&lock_path).await;
                return Err(PreviewError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )));
            }
        };

        // Check if process is still running with validation
        // Pass project_root for cwd validation to prevent PID reuse
        let project_root_path = Path::new(&lock_data.project_root);
        if self.is_process_running_validated(
            lock_data.pid,
            Some(lock_data.started_at),
            Some(project_root_path),
        ) {
            // Restore to active_servers
            let server_info = ServerInfo {
                id: Uuid::new_v4(),
                project_id: lock_data.project_id.clone(),
                port: lock_data.port,
                pid: Some(lock_data.pid),
                status: DevServerStatus::Running,
                preview_url: Some(lock_data.preview_url),
                child: None,
                actual_command: None,
                framework_name: None,
                log_tasks: None,
                source: crate::types::ServerSource::Orkee,
                matched_project_id: None,
            };

            let mut servers = self.active_servers.write().await;
            servers.insert(lock_data.project_id.clone(), server_info);

            info!(
                "Recovered running server for project: {} on port {}",
                lock_data.project_id, lock_data.port
            );
        } else {
            // Stale lock, remove it
            if let Err(e) = fs::remove_file(&lock_path).await {
                warn!("Failed to remove stale lock file {:?}: {}", lock_path, e);
            } else {
                info!("Removed stale lock for project: {}", lock_data.project_id);
            }
        }

        Ok(())
    }

    /// Register an external server discovered via port scanning
    ///
    /// This allows tracking of servers that were started manually outside of Orkee.
    /// The server will be added to both the active servers and the global registry.
    ///
    /// # Arguments
    ///
    /// * `discovered` - Information about the discovered server
    /// * `project_id` - Optional project ID to associate with (if matched)
    /// * `project_name` - Optional project name for display
    ///
    /// # Returns
    ///
    /// Returns the server ID on success
    pub async fn register_external_server(
        &self,
        discovered: crate::discovery::DiscoveredServer,
        project_id: Option<String>,
        project_name: Option<String>,
    ) -> PreviewResult<String> {
        let server_id = Uuid::new_v4();
        let effective_project_id = project_id
            .clone()
            .unwrap_or_else(|| format!("external-{}", discovered.port));

        let server_info = ServerInfo {
            id: server_id,
            project_id: effective_project_id.clone(),
            port: discovered.port,
            pid: Some(discovered.pid),
            status: DevServerStatus::Running,
            preview_url: Some(format!("http://localhost:{}", discovered.port)),
            child: None, // External servers have no child handle
            actual_command: Some(discovered.command.join(" ")),
            framework_name: discovered.framework_name.clone(),
            log_tasks: None,
            source: if project_id.is_some() {
                crate::types::ServerSource::Discovered
            } else {
                crate::types::ServerSource::External
            },
            matched_project_id: project_id.clone(),
        };

        // Add to active servers
        {
            let mut servers = self.active_servers.write().await;
            servers.insert(effective_project_id.clone(), server_info.clone());
        }

        // Register in global registry
        let registry_entry = crate::registry::ServerRegistryEntry {
            id: server_id.to_string(),
            project_id: effective_project_id,
            project_name,
            project_root: discovered.working_dir,
            port: discovered.port,
            pid: Some(discovered.pid),
            status: DevServerStatus::Running,
            preview_url: Some(format!("http://localhost:{}", discovered.port)),
            framework_name: discovered.framework_name,
            actual_command: Some(discovered.command.join(" ")),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: std::env::var(constants::ORKEE_API_PORT)
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4001),
            source: server_info.source,
            matched_project_id: project_id,
        };

        GLOBAL_REGISTRY
            .register_server(registry_entry)
            .await
            .map_err(|e| PreviewError::ProcessStartFailed {
                reason: format!("Failed to register in global registry: {}", e),
            })?;

        info!(
            "Registered external server on port {} with ID {}",
            discovered.port, server_id
        );

        Ok(server_id.to_string())
    }

    /// Restart an external server using its project configuration
    ///
    /// This stops the current process and restarts it using the project's dev_script
    /// and environment variables. Only works for external servers that have been
    /// matched to a project in the database.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The UUID of the server to restart
    /// * `project_root` - Path to the project root directory
    /// * `dev_command` - The development command to execute
    /// * `environment` - Environment variables to set
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success
    pub async fn restart_external_server(
        &self,
        server_id: &str,
        project_root: &Path,
        dev_command: &str,
        environment: &HashMap<String, String>,
    ) -> PreviewResult<()> {
        // Find the server
        let server_info = {
            let servers = self.active_servers.read().await;
            servers
                .values()
                .find(|s| s.id.to_string() == server_id)
                .cloned()
        };

        let server_info = server_info.ok_or_else(|| PreviewError::ServerNotRunning {
            project_id: server_id.to_string(),
        })?;

        // Verify it's an external server
        if server_info.source == crate::types::ServerSource::Orkee {
            return Err(PreviewError::ProcessStartFailed {
                reason: "Cannot restart Orkee-managed server using external restart method"
                    .to_string(),
            });
        }

        // Stop the current process
        if let Some(pid) = server_info.pid {
            info!("Stopping external server PID {} before restart", pid);
            if let Err(e) = self.kill_process(pid).await {
                warn!("Failed to kill external server process {}: {}", pid, e);
            }
            // Give the process time to stop
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // Start new process using project configuration
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(dev_command)
            .current_dir(project_root)
            .env("PORT", server_info.port.to_string())
            .envs(environment)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        let child = cmd.spawn().map_err(|e| PreviewError::ProcessSpawnError {
            command: dev_command.to_string(),
            error: e.to_string(),
        })?;

        let new_pid = child.id().ok_or_else(|| PreviewError::ProcessStartFailed {
            reason: "Failed to get PID of spawned process".to_string(),
        })?;

        info!(
            "Restarted external server on port {} with new PID {}",
            server_info.port, new_pid
        );

        // Update server info with new PID
        let mut updated_info = server_info.clone();
        updated_info.pid = Some(new_pid);
        updated_info.status = DevServerStatus::Running;
        updated_info.child = Some(Arc::new(RwLock::new(child)));

        // Update active servers
        {
            let mut servers = self.active_servers.write().await;
            servers.insert(server_info.project_id.clone(), updated_info.clone());
        }

        // Update registry
        let registry_entry = crate::registry::ServerRegistryEntry {
            id: server_id.to_string(),
            project_id: server_info.project_id.clone(),
            project_name: None,
            project_root: project_root.to_path_buf(),
            port: server_info.port,
            pid: Some(new_pid),
            status: DevServerStatus::Running,
            preview_url: server_info.preview_url.clone(),
            framework_name: server_info.framework_name.clone(),
            actual_command: Some(dev_command.to_string()),
            started_at: Utc::now(),
            last_seen: Utc::now(),
            api_port: std::env::var(constants::ORKEE_API_PORT)
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4001),
            source: server_info.source,
            matched_project_id: server_info.matched_project_id.clone(),
        };

        GLOBAL_REGISTRY
            .register_server(registry_entry)
            .await
            .map_err(|e| PreviewError::ProcessStartFailed {
                reason: format!("Failed to update registry after restart: {}", e),
            })?;

        Ok(())
    }

    /// Stop an external server by its server ID
    ///
    /// This terminates the external server process and removes it from tracking.
    ///
    /// # Arguments
    ///
    /// * `server_id` - The UUID of the server to stop
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success
    pub async fn stop_external_server(&self, server_id: &str) -> PreviewResult<()> {
        // Find the server
        let server_info = {
            let servers = self.active_servers.read().await;
            servers
                .values()
                .find(|s| s.id.to_string() == server_id)
                .cloned()
        };

        let server_info = server_info.ok_or_else(|| PreviewError::ServerNotRunning {
            project_id: server_id.to_string(),
        })?;

        // Kill the process if we have a PID
        if let Some(pid) = server_info.pid {
            info!("Stopping external server PID {}", pid);
            if let Err(e) = self.kill_process(pid).await {
                warn!("Failed to kill external server process {}: {}", pid, e);
            }
        }

        // Remove from active servers
        {
            let mut servers = self.active_servers.write().await;
            servers.remove(&server_info.project_id);
        }

        // Unregister from global registry
        GLOBAL_REGISTRY
            .unregister_server(server_id)
            .await
            .map_err(|e| PreviewError::ProcessStopFailed {
                reason: format!("Failed to unregister from global registry: {}", e),
            })?;

        info!("Stopped and unregistered external server {}", server_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_id_valid() {
        // Valid project IDs
        assert!(validate_project_id("my-project").is_ok());
        assert!(validate_project_id("my_project").is_ok());
        assert!(validate_project_id("project123").is_ok());
        assert!(validate_project_id("Project-Name_123").is_ok());
    }

    #[test]
    fn test_validate_project_id_path_traversal() {
        // Path traversal attacks
        assert!(validate_project_id("../../../etc/passwd").is_err());
        assert!(validate_project_id("..\\..\\..\\windows\\system32").is_err());
        assert!(validate_project_id("project/../etc/passwd").is_err());
        assert!(validate_project_id("../project").is_err());
        assert!(validate_project_id("project/subdir").is_err());
        assert!(validate_project_id("project\\subdir").is_err());
    }

    #[test]
    fn test_validate_project_id_empty() {
        assert!(validate_project_id("").is_err());
    }

    #[test]
    fn test_validate_project_id_special_characters() {
        // Invalid special characters
        assert!(validate_project_id("project@name").is_err());
        assert!(validate_project_id("project name").is_err());
        assert!(validate_project_id("project!name").is_err());
        assert!(validate_project_id("project#name").is_err());
        assert!(validate_project_id("project$name").is_err());
        assert!(validate_project_id("project%name").is_err());
    }

    #[test]
    fn test_get_lock_file_path_valid() {
        let manager = PreviewManager::new();
        let result = manager.get_lock_file_path("valid-project");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("valid-project.json"));
        assert!(path.to_string_lossy().contains(".orkee/preview-locks"));
    }

    #[test]
    fn test_get_lock_file_path_prevents_traversal() {
        let manager = PreviewManager::new();

        // These should all fail validation
        assert!(manager.get_lock_file_path("../../../etc/passwd").is_err());
        assert!(manager.get_lock_file_path("..\\..\\windows").is_err());
        assert!(manager.get_lock_file_path("project/etc").is_err());
    }
}
