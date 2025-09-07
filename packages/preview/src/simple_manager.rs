use crate::types::*;
use chrono::Utc;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::fs;
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;
use tokio::process::Child;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::collections::VecDeque;
use serde_json;

/// Result of spawning a server with metadata
#[derive(Debug)]
pub struct SpawnResult {
    pub child: Child,
    pub command: String,
    pub framework: String,
}

/// Simplified, crash-resistant preview server manager
#[derive(Clone)]
pub struct SimplePreviewManager {
    active_servers: Arc<RwLock<HashMap<String, ServerInfo>>>,
    server_logs: Arc<RwLock<HashMap<String, VecDeque<DevServerLog>>>>,
}

/// Minimal server information with process handle
#[derive(Debug)]
pub struct ServerInfo {
    pub id: Uuid,
    pub project_id: String,
    pub port: u16,
    pub pid: Option<u32>,
    pub status: DevServerStatus,
    pub preview_url: Option<String>,
    pub child: Option<Arc<RwLock<Child>>>,
    pub actual_command: Option<String>,
    pub framework_name: Option<String>,
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
        }
    }
}

impl SimplePreviewManager {
    /// Create a new simple preview manager
    pub fn new() -> Self {
        Self {
            active_servers: Arc::new(RwLock::new(HashMap::new())),
            server_logs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new manager and recover existing servers from lock files
    pub async fn new_with_recovery() -> Self {
        let manager = Self::new();
        
        // Recover existing servers from lock files
        if let Err(e) = manager.recover_servers().await {
            warn!("Failed to recover servers: {}", e);
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
        let project_logs = logs.entry(project_id.to_string()).or_insert_with(VecDeque::new);
        
        // Add the log entry
        project_logs.push_back(log_entry);
        
        // Keep only the last 1000 entries to prevent memory issues
        if project_logs.len() > 1000 {
            project_logs.pop_front();
        }
    }

    /// Get logs for a project
    pub async fn get_server_logs(&self, project_id: &str, since: Option<chrono::DateTime<Utc>>, limit: Option<usize>) -> Vec<DevServerLog> {
        let logs = self.server_logs.read().await;
        
        if let Some(project_logs) = logs.get(project_id) {
            let mut filtered_logs: Vec<DevServerLog> = if let Some(since_time) = since {
                project_logs.iter()
                    .filter(|log| log.timestamp > since_time)
                    .cloned()
                    .collect()
            } else {
                project_logs.iter().cloned().collect()
            };
            
            // Apply limit if specified
            if let Some(max_count) = limit {
                if filtered_logs.len() > max_count {
                    filtered_logs = filtered_logs.into_iter().rev().take(max_count).rev().collect();
                }
            }
            
            filtered_logs
        } else {
            Vec::new()
        }
    }

    /// Clear logs for a project
    pub async fn clear_server_logs(&self, project_id: &str) {
        let mut logs = self.server_logs.write().await;
        logs.remove(project_id);
        info!("Cleared logs for project: {}", project_id);
    }

    /// Extract port from server log line if detected
    fn extract_port_from_log(&self, line: &str) -> Option<u16> {
        // Common patterns for dev server port detection
        let patterns = [
            r"Local:\s+http://localhost:(\d+)",  // Vite: "Local:   http://localhost:5174/"
            r"Local server:\s+http://localhost:(\d+)", // Some frameworks
            r"Running at http://localhost:(\d+)", // Express/other servers
            r"Server ready at http://localhost:(\d+)", // Next.js dev
            r"server running on port (\d+)", // Express: "Express server running on port 8476"
            r"📍 http://localhost:(\d+)", // Express with emoji: "📍 http://localhost:8476"
            r"🚀.*port (\d+)", // Express startup: "🚀 Express server running on port 8476"
            r"ready - started server on.*:(\d+)", // Next.js: "ready - started server on 0.0.0.0:3000"
            r"http://localhost:(\d+)", // Generic http://localhost pattern
            r"localhost:(\d+)", // Generic localhost pattern
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
                        return status_code >= 200 && status_code < 400;
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
                info!("Detected port change for project {}: {} -> {}", project_id, server_info.port, new_port);
                server_info.port = new_port;
                server_info.preview_url = Some(format!("http://localhost:{}", new_port));
                self.add_log(project_id, LogType::System, 
                    format!("Updated preview URL to http://localhost:{}", new_port)).await;
            }
        }
    }

    /// Capture logs from a spawned process
    async fn capture_process_logs(&self, project_id: String, mut child: Child) {
        let project_id_clone = project_id.clone();
        let manager = self.clone();
        
        // Get stdout handle
        if let Some(stdout) = child.stdout.take() {
            let project_id_stdout = project_id_clone.clone();
            let manager_stdout = manager.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Check for port detection in the log line
                    if let Some(detected_port) = manager_stdout.extract_port_from_log(&line) {
                        manager_stdout.update_server_port(&project_id_stdout, detected_port).await;
                    }
                    manager_stdout.add_log(&project_id_stdout, LogType::Stdout, line).await;
                }
            });
        }

        // Get stderr handle
        if let Some(stderr) = child.stderr.take() {
            let project_id_stderr = project_id_clone.clone();
            let manager_stderr = manager.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Check for port detection in the log line (some servers log to stderr)
                    if let Some(detected_port) = manager_stderr.extract_port_from_log(&line) {
                        manager_stderr.update_server_port(&project_id_stderr, detected_port).await;
                    }
                    
                    // Filter out successful HTTP access logs from being marked as STDERR
                    let log_type = if manager_stderr.is_successful_http_log(&line) {
                        LogType::System  // HTTP access logs are informational, not errors
                    } else {
                        LogType::Stderr  // Real errors stay as STDERR
                    };
                    
                    manager_stderr.add_log(&project_id_stderr, log_type, line).await;
                }
            });
        }
    }

    /// Start a preview server for a project
    pub async fn start_server(&self, project_id: String, project_root: PathBuf) -> PreviewResult<ServerInfo> {
        info!("Starting simple preview server for: {}", project_id);

        // Check if server already exists
        {
            let servers = self.active_servers.read().await;
            if let Some(existing) = servers.get(&project_id) {
                if existing.status == DevServerStatus::Running {
                    info!("Server already running for project: {}", project_id);
                    return Ok(existing.clone());
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
        };

        // Try to start the server
        match self.spawn_server(&server_info, &project_root).await {
            Ok(spawn_result) => {
                let pid = spawn_result.child.id();
                
                // Start capturing logs from the process
                let project_id_for_logs = project_id.clone();
                let manager_for_logs = self.clone();
                tokio::spawn(async move {
                    manager_for_logs.capture_process_logs(project_id_for_logs, spawn_result.child).await;
                });

                let mut updated_info = server_info;
                updated_info.pid = pid;
                updated_info.status = DevServerStatus::Running;
                updated_info.actual_command = Some(spawn_result.command);
                updated_info.framework_name = Some(spawn_result.framework);

                // Store the server info
                {
                    let mut servers = self.active_servers.write().await;
                    servers.insert(project_id.clone(), updated_info.clone());
                }

                // Create lock file for persistence
                if let Err(e) = self.create_lock_file(&updated_info, &project_root).await {
                    warn!("Failed to create lock file for project {}: {}", project_id, e);
                }

                info!("Successfully started server for project: {} on port {}", project_id, port);
                Ok(updated_info)
            }
            Err(e) => {
                error!("Failed to start server for project {}: {}", project_id, e);
                
                let mut error_info = server_info;
                error_info.status = DevServerStatus::Error;
                
                // Store the error state
                {
                    let mut servers = self.active_servers.write().await;
                    servers.insert(project_id, error_info.clone());
                }
                
                Err(e)
            }
        }
    }

    /// Stop a preview server
    pub async fn stop_server(&self, project_id: &str) -> PreviewResult<()> {
        info!("Stopping server for project: {}", project_id);

        let server_info = {
            let servers = self.active_servers.read().await;
            servers.get(project_id).cloned()
        };

        if let Some(info) = server_info {
            // Add stop log
            self.add_log(project_id, LogType::System, 
                format!("Stopping server with PID: {:?}", info.pid)).await;

            if let Some(pid) = info.pid {
                // Try to kill the process
                self.kill_process(pid).await?;
            }
            
            // Remove from active servers
            {
                let mut servers = self.active_servers.write().await;
                servers.remove(project_id);
            }
            
            // Remove lock file
            if let Err(e) = self.remove_lock_file(project_id).await {
                warn!("Failed to remove lock file for project {}: {}", project_id, e);
            }
            
            info!("Successfully stopped server for project: {}", project_id);
        }

        Ok(())
    }

    /// Get server status
    pub async fn get_server_status(&self, project_id: &str) -> Option<ServerInfo> {
        let servers = self.active_servers.read().await;
        servers.get(project_id).cloned()
    }

    /// List all active servers
    pub async fn list_servers(&self) -> Vec<ServerInfo> {
        let servers = self.active_servers.read().await;
        servers.values().cloned().collect()
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
            info!("Using preferred port {} for project {}", preferred, project_id);
            return Ok(preferred);
        }
        
        // Scan range 8000-8999 starting from preferred
        for offset in 1..1000 {
            let port = 8000 + ((preferred - 8000 + offset) % 1000);
            if self.is_port_available(port).await {
                info!("Using alternative port {} for project {} (preferred {} was taken)", port, project_id, preferred);
                return Ok(port);
            }
        }
        
        error!("No available ports in range 8000-8999 for project {}", project_id);
        Err(PreviewError::PortInUse { port: preferred })
    }

    /// Check if port is available
    async fn is_port_available(&self, port: u16) -> bool {
        std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Spawn a server process based on project type
    async fn spawn_server(&self, server_info: &ServerInfo, project_root: &PathBuf) -> PreviewResult<SpawnResult> {
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
    async fn spawn_static_server(&self, server_info: &ServerInfo, project_root: &PathBuf) -> PreviewResult<SpawnResult> {
        // Use Python's built-in HTTP server as it's reliable and simple
        let mut cmd = Command::new("python3");
        cmd.args(["-m", "http.server", &server_info.port.to_string()])
            .current_dir(project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Add initial log
        self.add_log(&server_info.project_id, LogType::System, 
            format!("Starting static HTTP server on port {} in {}", server_info.port, project_root.display())).await;

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                info!("Spawned static server with PID: {:?}", pid);
                self.add_log(&server_info.project_id, LogType::System, 
                    format!("Static server started successfully with PID: {:?}", pid)).await;
                Ok(SpawnResult {
                    child,
                    command: "python3 -m http.server".to_string(),
                    framework: "Static HTTP Server".to_string(),
                })
            }
            Err(e) => {
                error!("Failed to spawn static server: {}", e);
                self.add_log(&server_info.project_id, LogType::System, 
                    format!("Failed to start static server: {}", e)).await;
                Err(PreviewError::ProcessSpawnError {
                    command: "python3 -m http.server".to_string(),
                    error: e.to_string(),
                })
            }
        }
    }

    /// Check if a package.json script exists
    async fn has_npm_script(&self, project_root: &PathBuf, script_name: &str) -> bool {
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
    async fn detect_framework(&self, command: &str, project_root: &PathBuf) -> String {
        // Check command first
        if command.contains("vite") || command.contains("npm run dev") && self.has_dependency(project_root, "vite").await {
            return "Vite".to_string();
        }
        if command.contains("next") || self.has_dependency(project_root, "next").await {
            return "Next.js".to_string();
        }
        if command.contains("react-scripts") || self.has_dependency(project_root, "react-scripts").await {
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
    async fn has_dependency(&self, project_root: &PathBuf, dep_name: &str) -> bool {
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
    async fn spawn_dev_command(&self, server_info: &ServerInfo, project_root: &PathBuf) -> PreviewResult<SpawnResult> {
        let port_str = server_info.port.to_string();
        
        // Add initial log
        self.add_log(&server_info.project_id, LogType::System, 
            format!("Attempting to start development server on port {} in {}", server_info.port, project_root.display())).await;

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

            self.add_log(&server_info.project_id, LogType::System, 
                format!("Trying command: {} {}", cmd, args.join(" "))).await;

            if let Ok(child) = command.spawn() {
                let pid = child.id();
                info!("Spawned dev server with command '{}' and PID: {:?}", cmd, pid);
                
                let command_str = format!("{} {}", cmd, args.join(" "));
                let framework = self.detect_framework(&command_str, project_root).await;
                
                self.add_log(&server_info.project_id, LogType::System, 
                    format!("Development server started successfully with command '{}' and PID: {:?}", cmd, pid)).await;
                
                return Ok(SpawnResult {
                    child,
                    command: command_str,
                    framework,
                });
            }
        }

        self.add_log(&server_info.project_id, LogType::System, 
            "No suitable development server command found".to_string()).await;

        Err(PreviewError::ProcessSpawnError {
            command: "No suitable dev command found".to_string(),
            error: "Could not start any development server".to_string(),
        })
    }

    /// Kill a process by PID
    async fn kill_process(&self, pid: u32) -> PreviewResult<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let pid = Pid::from_raw(pid as i32);
            match kill(pid, Signal::SIGTERM) {
                Ok(_) => {
                    info!("Successfully killed process with PID: {}", pid);
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to kill process with PID {}: {}", pid, e);
                    Err(PreviewError::ProcessKillError {
                        pid: pid.as_raw() as u32,
                        error: e.to_string(),
                    })
                }
            }
        }
        
        #[cfg(not(unix))]
        {
            // On non-Unix systems, we can't easily kill processes
            warn!("Process killing not implemented for this platform");
            Ok(())
        }
    }

    // === PERSISTENCE METHODS ===

    /// Get lock file path for a project
    fn get_lock_file_path(&self, project_id: &str) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".orkee")
            .join("preview-locks")
            .join(format!("{}.json", project_id))
    }

    /// Check if a process is running (Unix only)
    fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            
            // Signal 0 checks if process exists without actually sending a signal
            match kill(Pid::from_raw(pid as i32), Some(Signal::SIGCONT)) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        
        #[cfg(not(unix))]
        {
            // On Windows, assume process is running (safer default)
            warn!("Process checking not implemented for this platform, assuming PID {} is running", pid);
            true
        }
    }

    /// Create lock file when starting server
    async fn create_lock_file(&self, server_info: &ServerInfo, project_root: &PathBuf) -> PreviewResult<()> {
        let lock_data = ServerLockData {
            project_id: server_info.project_id.clone(),
            pid: server_info.pid.unwrap_or(0),
            port: server_info.port,
            started_at: Utc::now(),
            preview_url: server_info.preview_url.clone().unwrap_or_default(),
            project_root: project_root.to_string_lossy().to_string(),
        };

        let lock_path = self.get_lock_file_path(&server_info.project_id);
        let lock_json = serde_json::to_string_pretty(&lock_data)
            .map_err(|e| PreviewError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Ensure directory exists
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| PreviewError::IoError(e))?;
        }

        fs::write(&lock_path, lock_json).await
            .map_err(|e| PreviewError::IoError(e))?;
        
        info!("Created lock file for project: {} at {:?}", server_info.project_id, lock_path);
        Ok(())
    }

    /// Remove lock file when stopping server
    async fn remove_lock_file(&self, project_id: &str) -> PreviewResult<()> {
        let lock_path = self.get_lock_file_path(project_id);
        if lock_path.exists() {
            fs::remove_file(&lock_path).await
                .map_err(|e| PreviewError::IoError(e))?;
            info!("Removed lock file for project: {}", project_id);
        }
        Ok(())
    }

    /// Recover servers from lock files on startup
    pub async fn recover_servers(&self) -> PreviewResult<()> {
        let lock_dir = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            .join(".orkee")
            .join("preview-locks");

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
            if path.extension() == Some(std::ffi::OsStr::new("json")) {
                if self.recover_single_server(path).await.is_ok() {
                    recovered_count += 1;
                }
            }
        }

        if recovered_count > 0 {
            info!("Recovered {} preview servers from lock files", recovered_count);
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
                return Err(PreviewError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e)));
            }
        };

        // Check if process is still running
        if self.is_process_running(lock_data.pid) {
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
            };

            let mut servers = self.active_servers.write().await;
            servers.insert(lock_data.project_id.clone(), server_info);

            info!("Recovered running server for project: {} on port {}", lock_data.project_id, lock_data.port);
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
}