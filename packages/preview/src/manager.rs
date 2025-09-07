use crate::{
    detector::ProjectDetector,
    static_server::{StaticServer, StaticServerConfig},
    types::{
        DevServerConfig, DevServerInstance, DevServerLog, DevServerStatus, LogType, PreviewError,
        PreviewResult, ProjectType, ServerLockData,
    },
};
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    process::{Child, Command},
    sync::{Mutex, RwLock},
    task::JoinHandle,
    time::{interval, sleep},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Manages development server instances across projects
pub struct DevServerManager {
    /// Active server instances by project ID
    pub active_servers: Arc<RwLock<HashMap<String, DevServerInstance>>>,
    /// Running processes by project ID
    processes: Arc<Mutex<HashMap<String, Child>>>,
    /// Server logs by project ID
    logs: Arc<RwLock<HashMap<String, Vec<DevServerLog>>>>,
    /// Background tasks by project ID
    background_tasks: Arc<tokio::sync::Mutex<HashMap<String, JoinHandle<()>>>>,
    /// Lock directory path
    lock_dir: PathBuf,
}

impl DevServerManager {
    /// Create a new DevServerManager instance
    pub fn new() -> PreviewResult<Self> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let lock_dir = PathBuf::from(home_dir).join(".orkee").join("preview-locks");
        
        // Ensure lock directory exists
        if let Err(e) = std::fs::create_dir_all(&lock_dir) {
            eprintln!("Warning: Failed to create lock directory {}: {}", lock_dir.display(), e);
        }

        Ok(Self {
            active_servers: Arc::new(RwLock::new(HashMap::new())),
            processes: Arc::new(Mutex::new(HashMap::new())),
            logs: Arc::new(RwLock::new(HashMap::new())),
            background_tasks: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            lock_dir,
        })
    }

    /// Start a development server for a project
    pub async fn start_dev_server(
        &self,
        project_id: String,
        project_root: PathBuf,
        custom_port: Option<u16>,
    ) -> PreviewResult<DevServerInstance> {
        info!(
            "Starting dev server for project: {} at {}",
            project_id,
            project_root.display()
        );

        // Check for existing server from lock file
        if let Some(existing) = self.get_existing_server_lock(&project_id).await? {
            info!("Found existing server from lock file: {}", project_id);
            let mut servers = self.active_servers.write().await;
            servers.insert(project_id.clone(), existing.clone());
            return Ok(existing);
        }

        // Check if server is already running in memory
        {
            let servers = self.active_servers.read().await;
            if let Some(existing) = servers.get(&project_id) {
                if existing.status == DevServerStatus::Running {
                    info!("Server already running: {}", project_id);
                    return Ok(existing.clone());
                }
            }
        }

        // Detect project configuration
        let detection = ProjectDetector::detect_project(&project_root).await?;
        debug!("Project detection result: {:?}", detection);

        // Find available port
        let port = if let Some(custom) = custom_port {
            if !Self::is_port_available(custom).await? {
                return Err(PreviewError::PortInUse { port: custom });
            }
            custom
        } else {
            Self::find_available_port(detection.port).await?
        };

        // Create server configuration
        let config = DevServerConfig {
            project_type: detection.project_type,
            dev_command: detection.dev_command,
            port,
            package_manager: detection.package_manager,
            framework: detection.framework,
        };

        // Create server instance
        let instance = DevServerInstance {
            id: Uuid::new_v4(),
            project_id: project_id.clone(),
            config,
            status: DevServerStatus::Starting,
            preview_url: Some(format!("http://127.0.0.1:{}", port)),
            started_at: Some(Utc::now()),
            last_activity: Some(Utc::now()),
            error: None,
            pid: None,
        };

        // Store instance and initialize logs
        {
            let mut servers = self.active_servers.write().await;
            servers.insert(project_id.clone(), instance.clone());
        }
        {
            let mut logs = self.logs.write().await;
            logs.insert(project_id.clone(), Vec::new());
        }

        // Start the server process
        let mut instance = instance;
        match self
            .spawn_dev_server(&mut instance, &project_root)
            .await
        {
            Ok(_) => {
                instance.status = DevServerStatus::Running;
                instance.last_activity = Some(Utc::now());

                // Update stored instance
                {
                    let mut servers = self.active_servers.write().await;
                    servers.insert(project_id.clone(), instance.clone());
                }

                // Create lock file
                if let Err(e) = self.create_server_lock(&project_id, &instance, &project_root).await {
                    warn!("Failed to create server lock: {}", e);
                }

                info!(
                    "Dev server started successfully: {} on port {}",
                    project_id, instance.config.port
                );
                Ok(instance)
            }
            Err(e) => {
                instance.status = DevServerStatus::Error;
                instance.error = Some(e.to_string());

                // Update stored instance
                {
                    let mut servers = self.active_servers.write().await;
                    servers.insert(project_id.clone(), instance.clone());
                }

                Err(e)
            }
        }
    }

    /// Stop a development server
    pub async fn stop_dev_server(&self, project_id: &str) -> PreviewResult<()> {
        info!("Stopping dev server: {}", project_id);

        // Update status to stopping
        {
            let mut servers = self.active_servers.write().await;
            if let Some(instance) = servers.get_mut(project_id) {
                instance.status = DevServerStatus::Stopping;
            }
        }

        // Kill background tasks
        {
            let mut tasks = self.background_tasks.lock().await;
            if let Some(task) = tasks.remove(project_id) {
                task.abort();
            }
        }

        // Kill the process
        let child_opt = {
            let mut processes = self.processes.lock().await;
            processes.remove(project_id)
        };
        
        if let Some(mut child) = child_opt {
            if let Err(e) = child.kill().await {
                warn!("Failed to kill process for {}: {}", project_id, e);
            }
        }

        // Update status and remove from active servers
        {
            let mut servers = self.active_servers.write().await;
            if let Some(mut instance) = servers.remove(project_id) {
                instance.status = DevServerStatus::Stopped;
                self.add_log(
                    project_id,
                    LogType::System,
                    "Server stopped by user request",
                )
                .await;
            }
        }

        // Remove lock file
        if let Err(e) = self.remove_server_lock(project_id).await {
            debug!("Failed to remove server lock: {}", e);
        }

        info!("Dev server stopped: {}", project_id);
        Ok(())
    }

    /// Get server status
    pub async fn get_server_status(&self, project_id: &str) -> Option<DevServerInstance> {
        // First check memory
        {
            let servers = self.active_servers.read().await;
            if let Some(instance) = servers.get(project_id) {
                return Some(instance.clone());
            }
        }

        // Check lock file
        if let Ok(Some(instance)) = self.get_existing_server_lock(project_id).await {
            // Add to memory for future calls
            let mut servers = self.active_servers.write().await;
            servers.insert(project_id.to_string(), instance.clone());
            return Some(instance);
        }

        None
    }

    /// Get server logs
    pub async fn get_server_logs(
        &self,
        project_id: &str,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Vec<DevServerLog> {
        let logs = self.logs.read().await;
        let project_logs = logs.get(project_id).cloned().unwrap_or_default();

        let filtered_logs = if let Some(since_time) = since {
            project_logs
                .into_iter()
                .filter(|log| log.timestamp >= since_time)
                .collect()
        } else {
            project_logs
        };

        if let Some(limit) = limit {
            filtered_logs.into_iter().take(limit).collect()
        } else {
            filtered_logs
        }
    }

    /// Clear logs for a project
    pub async fn clear_logs(&self, project_id: &str) {
        let mut logs = self.logs.write().await;
        logs.insert(project_id.to_string(), Vec::new());
        info!("Cleared logs for project: {}", project_id);
    }

    /// Update server activity timestamp
    pub async fn update_activity(&self, project_id: &str) {
        let mut servers = self.active_servers.write().await;
        if let Some(instance) = servers.get_mut(project_id) {
            instance.last_activity = Some(Utc::now());
            debug!("Updated activity for project: {}", project_id);
        }
    }

    /// Spawn the development server process
    async fn spawn_dev_server(
        &self,
        instance: &mut DevServerInstance,
        project_root: &Path,
    ) -> PreviewResult<()> {
        let project_id = instance.project_id.clone();
        let config = &instance.config;

        self.add_log(
            &project_id,
            LogType::System,
            &format!(
                "Starting {} server on port {}",
                config.framework.as_ref().map(|f| f.name.as_str()).unwrap_or("development"),
                config.port
            ),
        )
        .await;

        match config.project_type {
            ProjectType::Static => {
                self.spawn_static_server(instance, project_root).await
            }
            ProjectType::Python => {
                self.spawn_python_server(instance, project_root).await
            }
            _ => {
                self.spawn_nodejs_server(instance, project_root).await
            }
        }
    }

    /// Spawn a static file server
    async fn spawn_static_server(
        &self,
        instance: &mut DevServerInstance,
        project_root: &Path,
    ) -> PreviewResult<()> {
        let project_id = instance.project_id.clone();
        let config = StaticServerConfig::new(project_root.to_path_buf(), instance.config.port);

        // Start static server in background
        let server = StaticServer::new(config);
        let task_handle = tokio::spawn(async move {
            if let Err(e) = server.start().await {
                error!("Static server error: {}", e);
            }
        });

        // Store the task handle
        {
            let mut tasks = self.background_tasks.lock().await;
            tasks.insert(project_id.clone(), task_handle);
        }

        // Wait a moment for the server to start
        sleep(Duration::from_millis(500)).await;

        self.add_log(
            &project_id,
            LogType::System,
            "Static file server started successfully",
        )
        .await;

        Ok(())
    }

    /// Spawn a Python HTTP server
    async fn spawn_python_server(
        &self,
        instance: &mut DevServerInstance,
        project_root: &Path,
    ) -> PreviewResult<()> {
        let mut cmd = Command::new("python3");
        cmd.args(["-m", "http.server", &instance.config.port.to_string()])
            .current_dir(project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        self.spawn_process(instance, cmd).await
    }

    /// Spawn a Node.js development server
    async fn spawn_nodejs_server(
        &self,
        instance: &mut DevServerInstance,
        project_root: &Path,
    ) -> PreviewResult<()> {
        let (command, args) = self.parse_dev_command(&instance.config.dev_command);

        let mut cmd = Command::new(&command);
        cmd.args(&args)
            .current_dir(project_root)
            .env("PORT", instance.config.port.to_string())
            .env("HOST", "0.0.0.0")
            .env("BROWSER", "none")
            .env("NODE_ENV", "development")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        self.spawn_process(instance, cmd).await
    }

    /// Spawn and manage a process
    async fn spawn_process(
        &self,
        instance: &mut DevServerInstance,
        mut cmd: Command,
    ) -> PreviewResult<()> {
        let project_id = instance.project_id.clone();

        self.add_log(
            &project_id,
            LogType::System,
            &format!("Command: {:?}", cmd.as_std()),
        )
        .await;

        let child = cmd.spawn().map_err(|e| PreviewError::ProcessStartFailed {
            reason: e.to_string(),
        })?;

        instance.pid = child.id();

        // Store the child process
        {
            let mut processes = self.processes.lock().await;
            processes.insert(project_id.clone(), child);
        }

        // Start log streaming tasks
        if let Some(child) = self.processes.lock().await.get_mut(&project_id) {
            if let Some(stdout) = child.stdout.take() {
                self.start_log_stream(project_id.clone(), LogType::Stdout, stdout)
                    .await;
            }

            if let Some(stderr) = child.stderr.take() {
                self.start_log_stream(project_id.clone(), LogType::Stderr, stderr)
                    .await;
            }
        }

        self.add_log(
            &project_id,
            LogType::System,
            &format!("Process spawned with PID: {:?}", instance.pid),
        )
        .await;

        Ok(())
    }

    /// Start streaming logs from a process output
    async fn start_log_stream<R>(&self, project_id: String, log_type: LogType, reader: R)
    where
        R: tokio::io::AsyncRead + Send + Unpin + 'static,
    {
        let logs = self.logs.clone();
        let active_servers = self.active_servers.clone();

        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let log_entry = DevServerLog {
                    timestamp: Utc::now(),
                    log_type: log_type.clone(),
                    message: line.trim().to_string(),
                };

                // Store log
                {
                    let mut logs_guard = logs.write().await;
                    if let Some(project_logs) = logs_guard.get_mut(&project_id) {
                        project_logs.push(log_entry);

                        // Keep only last 1000 entries
                        if project_logs.len() > 1000 {
                            project_logs.drain(0..project_logs.len() - 1000);
                        }
                    }
                }

                // Update activity
                {
                    let mut servers = active_servers.write().await;
                    if let Some(instance) = servers.get_mut(&project_id) {
                        instance.last_activity = Some(Utc::now());
                    }
                }
            }
        });
    }

    /// Parse a development command into command and arguments
    fn parse_dev_command(&self, dev_command: &str) -> (String, Vec<String>) {
        let parts: Vec<&str> = dev_command.split_whitespace().collect();
        if parts.is_empty() {
            ("echo".to_string(), vec!["No command specified".to_string()])
        } else {
            (parts[0].to_string(), parts[1..].iter().map(|s| s.to_string()).collect())
        }
    }

    /// Check if a port is available
    async fn is_port_available(port: u16) -> PreviewResult<bool> {
        match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Find an available port starting from the preferred port
    async fn find_available_port(preferred_port: u16) -> PreviewResult<u16> {
        for i in 0..50 {
            let port = preferred_port.saturating_add(i);
            if port == u16::MAX {
                // Reached maximum port value
                break;
            }
            if Self::is_port_available(port).await? {
                return Ok(port);
            }
        }
        Err(PreviewError::PortInUse {
            port: preferred_port,
        })
    }

    /// Add a log entry
    async fn add_log(&self, project_id: &str, log_type: LogType, message: &str) {
        let log_entry = DevServerLog {
            timestamp: Utc::now(),
            log_type,
            message: message.to_string(),
        };

        let mut logs = self.logs.write().await;
        if let Some(project_logs) = logs.get_mut(project_id) {
            project_logs.push(log_entry);

            // Keep only last 1000 entries
            if project_logs.len() > 1000 {
                project_logs.drain(0..project_logs.len() - 1000);
            }
        }
    }

    /// Get lock file path for a project
    fn get_lock_file_path(&self, project_id: &str) -> PathBuf {
        self.lock_dir.join(format!("{}.json", project_id))
    }

    /// Get existing server lock from file
    async fn get_existing_server_lock(
        &self,
        project_id: &str,
    ) -> PreviewResult<Option<DevServerInstance>> {
        let lock_file = self.get_lock_file_path(project_id);

        if !lock_file.exists() {
            return Ok(None);
        }

        let lock_data = match fs::read_to_string(&lock_file).await {
            Ok(data) => data,
            Err(_) => return Ok(None),
        };

        let lock: ServerLockData = match serde_json::from_str(&lock_data) {
            Ok(lock) => lock,
            Err(_) => {
                // Invalid lock file, remove it
                let _ = fs::remove_file(&lock_file).await;
                return Ok(None);
            }
        };

        // Check if process is still running
        if !Self::is_process_running(lock.pid) {
            // Stale lock file, remove it
            let _ = fs::remove_file(&lock_file).await;
            return Ok(None);
        }

        // Reconstruct instance from lock data
        let instance = DevServerInstance {
            id: Uuid::new_v4(),
            project_id: lock.project_id,
            config: DevServerConfig {
                project_type: ProjectType::Unknown, // We don't store this in lock
                dev_command: "unknown".to_string(),
                port: lock.port,
                package_manager: crate::types::PackageManager::Npm,
                framework: None,
            },
            status: DevServerStatus::Running,
            preview_url: Some(lock.preview_url),
            started_at: Some(lock.started_at),
            last_activity: Some(Utc::now()),
            error: None,
            pid: Some(lock.pid),
        };

        Ok(Some(instance))
    }

    /// Create server lock file
    async fn create_server_lock(
        &self,
        project_id: &str,
        instance: &DevServerInstance,
        project_root: &PathBuf,
    ) -> PreviewResult<()> {
        // Ensure lock directory exists
        fs::create_dir_all(&self.lock_dir).await?;

        let lock_data = ServerLockData {
            project_id: project_id.to_string(),
            pid: instance.pid.unwrap_or(0),
            port: instance.config.port,
            started_at: instance.started_at.unwrap_or(Utc::now()),
            preview_url: instance.preview_url.clone().unwrap_or_default(),
            project_root: project_root.to_string_lossy().to_string(),
        };

        let lock_file = self.get_lock_file_path(project_id);
        let lock_json = serde_json::to_string_pretty(&lock_data)?;
        fs::write(lock_file, lock_json).await?;

        debug!("Created server lock for project: {}", project_id);
        Ok(())
    }

    /// Remove server lock file
    async fn remove_server_lock(&self, project_id: &str) -> PreviewResult<()> {
        let lock_file = self.get_lock_file_path(project_id);
        if lock_file.exists() {
            fs::remove_file(lock_file).await?;
            debug!("Removed server lock for project: {}", project_id);
        }
        Ok(())
    }

    /// Check if a process is running by PID
    fn is_process_running(pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => true,
                Err(nix::errno::Errno::ESRCH) => false, // Process not found
                Err(_) => true, // Process exists but we can't signal it (permission denied, etc.)
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we'd need to use winapi to check process existence
            // For now, assume process exists if we can't verify
            true
        }
    }

    /// Start cleanup task for idle servers
    pub fn start_cleanup_task(&self) -> JoinHandle<()> {
        let active_servers = self.active_servers.clone();
        let manager = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(600)); // 10 minutes

            loop {
                interval.tick().await;

                let now = Utc::now();
                let idle_threshold = Duration::from_secs(3600); // 1 hour

                let servers_to_cleanup: Vec<String> = {
                    let servers = active_servers.read().await;
                    servers
                        .iter()
                        .filter_map(|(project_id, instance)| {
                            if let Some(last_activity) = instance.last_activity {
                                let idle_duration = now.signed_duration_since(last_activity);
                                if idle_duration.num_seconds() as u64 > idle_threshold.as_secs() {
                                    Some(project_id.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect()
                };

                for project_id in servers_to_cleanup {
                    info!("Cleaning up idle server: {}", project_id);
                    if let Err(e) = manager.stop_dev_server(&project_id).await {
                        warn!("Failed to cleanup idle server {}: {}", project_id, e);
                    }
                }
            }
        })
    }

    /// Shutdown all servers
    pub async fn shutdown(&self) -> PreviewResult<()> {
        info!("Shutting down all dev servers");

        let project_ids: Vec<String> = {
            let servers = self.active_servers.read().await;
            servers.keys().cloned().collect()
        };

        for project_id in project_ids {
            if let Err(e) = self.stop_dev_server(&project_id).await {
                warn!("Failed to stop server during shutdown {}: {}", project_id, e);
            }
        }

        info!("All dev servers shut down");
        Ok(())
    }
}

impl Clone for DevServerManager {
    fn clone(&self) -> Self {
        Self {
            active_servers: self.active_servers.clone(),
            processes: self.processes.clone(),
            logs: self.logs.clone(),
            background_tasks: self.background_tasks.clone(),
            lock_dir: self.lock_dir.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = DevServerManager::new().unwrap();
        assert!(manager.lock_dir.to_string_lossy().contains(".orkee"));
    }

    #[tokio::test]
    async fn test_port_availability() {
        // Test with a likely available port
        let available = DevServerManager::is_port_available(0).await.unwrap();
        assert!(available);
    }

    #[tokio::test]
    async fn test_find_available_port() {
        let port = DevServerManager::find_available_port(3000).await.unwrap();
        assert!(port >= 3000);
        assert!(port <= 3050);
    }

    #[tokio::test]
    async fn test_parse_dev_command() {
        let manager = DevServerManager::new().unwrap();

        let (cmd, args) = manager.parse_dev_command("npm run dev");
        assert_eq!(cmd, "npm");
        assert_eq!(args, vec!["run", "dev"]);

        let (cmd, args) = manager.parse_dev_command("node server.js");
        assert_eq!(cmd, "node");
        assert_eq!(args, vec!["server.js"]);
    }
}