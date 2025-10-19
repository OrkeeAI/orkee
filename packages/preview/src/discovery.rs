// ABOUTME: Discovery service for detecting externally launched development servers
// ABOUTME: Scans ports and identifies running processes to track manual server launches

use chrono::Utc;
use nix::libc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use sysinfo::{Pid, ProcessRefreshKind, System};
use tokio::process::Command;
use tracing::debug;

use crate::registry::{ServerRegistryEntry, GLOBAL_REGISTRY};
use crate::types::{DevServerStatus, ServerSource};

/// Common development server ports to scan
const DEFAULT_DISCOVERY_PORTS: &[u16] = &[
    3000, 3001, 3002, 3003, // React, Next.js common ports
    4200, // Angular
    5000, 5001, 5173, 5174, // Flask, Vite
    8000, 8001, 8080, 8081, 8888, // General dev servers
    9000, 9001, // Additional common ports
];

/// Information about a discovered external server process
#[derive(Debug, Clone)]
pub struct DiscoveredServer {
    /// Process ID
    pub pid: u32,
    /// Port number
    pub port: u16,
    /// Working directory of the process
    pub working_dir: PathBuf,
    /// Full command line
    pub command: Vec<String>,
    /// Detected framework name (if identifiable)
    pub framework_name: Option<String>,
}

/// Discover external servers running on common development ports
pub async fn discover_external_servers() -> Vec<DiscoveredServer> {
    let mut discovered = Vec::new();

    // Get ports to scan from environment or use defaults
    let ports_to_scan = get_discovery_ports();

    debug!(
        "Scanning {} ports for external servers",
        ports_to_scan.len()
    );

    for port in ports_to_scan {
        if let Some(server) = discover_server_on_port(port).await {
            // Check if this server is already registered
            if !is_server_already_registered(server.pid, port).await {
                discovered.push(server);
            }
        }
    }

    debug!("Discovered {} new external servers", discovered.len());
    discovered
}

/// Check if a server is already registered in the global registry
async fn is_server_already_registered(pid: u32, port: u16) -> bool {
    let servers = GLOBAL_REGISTRY.get_all_servers().await;
    servers.iter().any(|s| s.pid == Some(pid) || s.port == port)
}

/// Get list of ports to scan from environment or use defaults
fn get_discovery_ports() -> Vec<u16> {
    if let Ok(ports_str) = std::env::var("ORKEE_DISCOVERY_PORTS") {
        ports_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    } else {
        DEFAULT_DISCOVERY_PORTS.to_vec()
    }
}

/// Discover server process on a specific port
async fn discover_server_on_port(port: u16) -> Option<DiscoveredServer> {
    // Get PID for the process listening on this port
    let pid = find_process_on_port(port).await?;

    // Get process information using sysinfo
    let mut system = System::new();
    system.refresh_process_specifics(Pid::from_u32(pid), ProcessRefreshKind::everything());

    let process = system.process(Pid::from_u32(pid))?;

    // Verify the process belongs to the current user (security check)
    if let Some(process_uid) = process.user_id() {
        let current_uid = unsafe { libc::getuid() };
        if process_uid.to_string().parse::<u32>().ok()? != current_uid {
            debug!("Skipping process {} - owned by different user", pid);
            return None;
        }
    }

    // Extract command line
    let command: Vec<String> = process.cmd().iter().map(|s| s.to_string()).collect();
    if command.is_empty() {
        return None;
    }

    // Check if this looks like a development server
    if !is_likely_dev_server(&command) {
        debug!(
            "Process {} on port {} doesn't look like a dev server: {:?}",
            pid, port, command
        );
        return None;
    }

    // Get working directory
    let working_dir = process.cwd()?.to_path_buf();

    // Detect framework from command line
    let framework_name = detect_framework_from_command(&command);

    debug!(
        "Discovered server on port {}: PID={}, command={:?}, cwd={:?}",
        port, pid, command, working_dir
    );

    Some(DiscoveredServer {
        pid,
        port,
        working_dir,
        command,
        framework_name,
    })
}

/// Find the PID of a process listening on a given port
#[cfg(target_os = "macos")]
async fn find_process_on_port(port: u16) -> Option<u32> {
    // Use lsof on macOS
    let output = Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout.trim().lines().next()?.parse().ok()
}

/// Find the PID of a process listening on a given port (Linux)
#[cfg(target_os = "linux")]
async fn find_process_on_port(port: u16) -> Option<u32> {
    // Try using ss first (modern Linux)
    if let Some(pid) = try_ss_for_port(port).await {
        return Some(pid);
    }

    // Fallback to reading /proc/net/tcp
    try_proc_net_tcp_for_port(port).await
}

#[cfg(target_os = "linux")]
async fn try_ss_for_port(port: u16) -> Option<u32> {
    let output = Command::new("ss")
        .args(&["-tlnp", &format!("sport = :{}", port)])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    // Parse ss output to extract PID
    for line in stdout.lines().skip(1) {
        if let Some(pid_part) = line.split("pid=").nth(1) {
            if let Some(pid_str) = pid_part.split(',').next() {
                return pid_str.parse().ok();
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
async fn try_proc_net_tcp_for_port(port: u16) -> Option<u32> {
    use std::fs;

    // Convert port to hex
    let port_hex = format!("{:04X}", port);

    // Read /proc/net/tcp
    let content = fs::read_to_string("/proc/net/tcp").ok()?;

    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }

        // Local address is in format IP:PORT
        let local_addr = parts[1];
        if let Some((_ip, port_part)) = local_addr.split_once(':') {
            if port_part == port_hex {
                // Inode is at parts[9]
                if let Ok(inode) = parts[9].parse::<u64>() {
                    // Find PID by inode
                    return find_pid_by_inode(inode).await;
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
async fn find_pid_by_inode(inode: u64) -> Option<u32> {
    use std::fs;

    let proc_dir = fs::read_dir("/proc").ok()?;

    for entry in proc_dir.flatten() {
        let pid_str = entry.file_name();
        if let Ok(pid) = pid_str.to_str()?.parse::<u32>() {
            let fd_dir = format!("/proc/{}/fd", pid);
            if let Ok(fd_entries) = fs::read_dir(fd_dir) {
                for fd_entry in fd_entries.flatten() {
                    if let Ok(link) = fs::read_link(fd_entry.path()) {
                        if link.to_str()? == format!("socket:[{}]", inode) {
                            return Some(pid);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Fallback for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
async fn find_process_on_port(_port: u16) -> Option<u32> {
    warn!("Port scanning not supported on this platform");
    None
}

/// Check if a command line suggests a development server
fn is_likely_dev_server(command: &[String]) -> bool {
    let command_str = command.join(" ").to_lowercase();

    // Common development server indicators
    let indicators = [
        "npm",
        "yarn",
        "pnpm",
        "bun", // Package managers
        "node",
        "deno", // JavaScript runtimes
        "vite",
        "next",
        "react-scripts",
        "webpack", // Build tools
        "flask",
        "django",
        "uvicorn",
        "gunicorn", // Python servers
        "rails",
        "puma", // Ruby
        "cargo",
        "trunk", // Rust
        "dev",
        "serve",
        "start", // Common script names
        "http.server",
        "SimpleHTTPServer", // Python HTTP servers
    ];

    indicators
        .iter()
        .any(|&indicator| command_str.contains(indicator))
}

/// Detect framework from command line
fn detect_framework_from_command(command: &[String]) -> Option<String> {
    let command_str = command.join(" ").to_lowercase();

    // Match frameworks in priority order (most specific first)
    if command_str.contains("next") || command_str.contains("next.js") {
        return Some("Next.js".to_string());
    }
    if command_str.contains("vite") {
        return Some("Vite".to_string());
    }
    if command_str.contains("react-scripts") || command_str.contains("create-react-app") {
        return Some("React".to_string());
    }
    if command_str.contains("webpack-dev-server") || command_str.contains("webpack") {
        return Some("Webpack".to_string());
    }
    if command_str.contains("vue-cli") || command_str.contains("@vue/cli") {
        return Some("Vue".to_string());
    }
    if command_str.contains("angular") || command_str.contains("ng serve") {
        return Some("Angular".to_string());
    }
    if command_str.contains("flask") {
        return Some("Flask".to_string());
    }
    if command_str.contains("django") || command_str.contains("manage.py") {
        return Some("Django".to_string());
    }
    if command_str.contains("rails") || command_str.contains("puma") {
        return Some("Rails".to_string());
    }
    if command_str.contains("deno") {
        return Some("Deno".to_string());
    }

    None
}

/// Register a discovered server in the global registry
pub async fn register_discovered_server(
    server: DiscoveredServer,
    project_id: Option<String>,
    project_name: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let server_id = uuid::Uuid::new_v4().to_string();

    let entry = ServerRegistryEntry {
        id: server_id.clone(),
        project_id: project_id
            .clone()
            .unwrap_or_else(|| format!("external-{}", server.port)),
        project_name,
        project_root: server.working_dir.clone(),
        port: server.port,
        pid: Some(server.pid),
        status: DevServerStatus::Running,
        preview_url: Some(format!("http://localhost:{}", server.port)),
        framework_name: server.framework_name,
        actual_command: Some(server.command.join(" ")),
        started_at: Utc::now(),
        last_seen: Utc::now(),
        api_port: std::env::var("ORKEE_API_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4001),
        source: if project_id.is_some() {
            ServerSource::Discovered
        } else {
            ServerSource::External
        },
        matched_project_id: project_id,
    };

    GLOBAL_REGISTRY.register_server(entry).await?;

    Ok(server_id)
}

/// Get environment variables from .env files in a directory
pub fn load_env_from_directory(dir: &Path) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    // Check for common .env files
    let env_files = [".env", ".env.local", ".env.development"];

    for env_file in &env_files {
        let env_path = dir.join(env_file);
        if env_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&env_path) {
                for line in contents.lines() {
                    let line = line.trim();

                    // Skip comments and empty lines
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Parse KEY=VALUE
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"').trim_matches('\'');
                        env_vars.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }

    // Ensure NODE_ENV is set for Node.js projects
    if !env_vars.contains_key("NODE_ENV") {
        env_vars.insert("NODE_ENV".to_string(), "development".to_string());
    }

    env_vars
}

/// Start periodic discovery of external servers
///
/// Spawns a background task that runs every 30 seconds (by default) to discover
/// external servers running on common development ports. Newly discovered servers
/// are automatically registered in the global registry.
///
/// The discovery interval can be configured via `ORKEE_DISCOVERY_INTERVAL_SECS`
/// environment variable (default: 30 seconds, min: 10, max: 300).
///
/// Discovery can be disabled by setting `ORKEE_DISCOVERY_ENABLED=false`.
///
/// This function should be called once during application initialization.
/// Multiple calls are safe - subsequent calls will return `None`.
///
/// # Returns
///
/// Returns `Some(JoinHandle)` on first call to allow graceful shutdown.
/// Returns `None` on subsequent calls (task already started) or if discovery is disabled.
///
/// # Examples
///
/// ```no_run
/// use orkee_preview::discovery::start_periodic_discovery;
///
/// #[tokio::main]
/// async fn main() {
///     // Start discovery and store handle for shutdown
///     let discovery_handle = start_periodic_discovery();
///
///     // Application continues running...
///
///     // On shutdown:
///     if let Some(handle) = discovery_handle {
///         handle.abort(); // Graceful shutdown
///     }
/// }
/// ```
pub fn start_periodic_discovery() -> Option<tokio::task::JoinHandle<()>> {
    use once_cell::sync::OnceCell;
    use tokio::time::{interval, Duration};
    use tracing::info;

    static DISCOVERY_TASK_STARTED: OnceCell<()> = OnceCell::new();

    // Check if discovery is enabled
    let discovery_enabled = std::env::var("ORKEE_DISCOVERY_ENABLED")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(true); // Enabled by default

    if !discovery_enabled {
        debug!("External server discovery is disabled via ORKEE_DISCOVERY_ENABLED");
        return None;
    }

    // Only start the task once
    if DISCOVERY_TASK_STARTED.get().is_some() {
        debug!("Periodic discovery task already started");
        return None;
    }

    // Get discovery interval from environment variable (default: 30 seconds)
    let discovery_interval_secs = std::env::var("ORKEE_DISCOVERY_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(10, 300)) // Min 10s, max 5 minutes
        .unwrap_or(30);

    info!(
        "Starting periodic external server discovery task (interval: {} seconds)",
        discovery_interval_secs
    );

    // Mark as started
    let _ = DISCOVERY_TASK_STARTED.set(());

    // Spawn background task and return handle for graceful shutdown
    let handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(discovery_interval_secs));

        loop {
            interval.tick().await;

            debug!("Running periodic external server discovery");

            // Discover servers
            let discovered = discover_external_servers().await;

            if !discovered.is_empty() {
                debug!("Discovered {} external servers", discovered.len());

                // Auto-register discovered servers
                for server in discovered {
                    // Register without project association (will be External source)
                    // Users can manually associate them with projects later via the UI
                    match register_discovered_server(server.clone(), None, None).await {
                        Ok(server_id) => {
                            debug!("Auto-registered external server: {}", server_id);
                        }
                        Err(e) => {
                            debug!(
                                "Failed to auto-register external server on port {}: {}",
                                server.port, e
                            );
                        }
                    }
                }
            }
        }
    });

    Some(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_is_likely_dev_server() {
        // Should detect npm scripts
        assert!(is_likely_dev_server(&[
            "npm".to_string(),
            "run".to_string(),
            "dev".to_string()
        ]));
        assert!(is_likely_dev_server(&[
            "yarn".to_string(),
            "dev".to_string()
        ]));
        assert!(is_likely_dev_server(&[
            "pnpm".to_string(),
            "start".to_string()
        ]));

        // Should detect node processes
        assert!(is_likely_dev_server(&[
            "node".to_string(),
            "server.js".to_string()
        ]));
        assert!(is_likely_dev_server(&[
            "/usr/bin/node".to_string(),
            "index.js".to_string()
        ]));

        // Should detect Python servers
        assert!(is_likely_dev_server(&[
            "python3".to_string(),
            "-m".to_string(),
            "http.server".to_string()
        ]));
        assert!(is_likely_dev_server(&[
            "flask".to_string(),
            "run".to_string()
        ]));
        assert!(is_likely_dev_server(&[
            "uvicorn".to_string(),
            "main:app".to_string()
        ]));

        // Should detect build tools
        assert!(is_likely_dev_server(&["vite".to_string()]));
        assert!(is_likely_dev_server(&["webpack-dev-server".to_string()]));

        // Should reject non-dev processes
        assert!(!is_likely_dev_server(&[
            "ls".to_string(),
            "-la".to_string()
        ]));
        assert!(!is_likely_dev_server(&[
            "grep".to_string(),
            "pattern".to_string()
        ]));
        assert!(!is_likely_dev_server(&["bash".to_string()]));
    }

    #[test]
    fn test_detect_framework_from_command() {
        // Next.js
        assert_eq!(
            detect_framework_from_command(&["next".to_string(), "dev".to_string()]),
            Some("Next.js".to_string())
        );
        assert_eq!(
            detect_framework_from_command(&[
                "npm".to_string(),
                "run".to_string(),
                "next".to_string()
            ]),
            Some("Next.js".to_string())
        );

        // Vite
        assert_eq!(
            detect_framework_from_command(&["vite".to_string()]),
            Some("Vite".to_string())
        );
        assert_eq!(
            detect_framework_from_command(&[
                "npm".to_string(),
                "run".to_string(),
                "vite".to_string()
            ]),
            Some("Vite".to_string())
        );

        // React
        assert_eq!(
            detect_framework_from_command(&["react-scripts".to_string(), "start".to_string()]),
            Some("React".to_string())
        );

        // Flask
        assert_eq!(
            detect_framework_from_command(&["flask".to_string(), "run".to_string()]),
            Some("Flask".to_string())
        );

        // Django
        assert_eq!(
            detect_framework_from_command(&[
                "python".to_string(),
                "manage.py".to_string(),
                "runserver".to_string()
            ]),
            Some("Django".to_string())
        );

        // Unknown
        assert_eq!(
            detect_framework_from_command(&["unknown".to_string(), "command".to_string()]),
            None
        );
    }

    #[test]
    #[serial]
    fn test_get_discovery_ports_default() {
        // Clear the environment variable if it exists
        std::env::remove_var("ORKEE_DISCOVERY_PORTS");

        let ports = get_discovery_ports();

        // Should return default ports
        assert!(ports.contains(&3000));
        assert!(ports.contains(&5173));
        assert!(ports.contains(&8080));
        assert!(!ports.is_empty());
    }

    #[test]
    #[serial]
    fn test_get_discovery_ports_custom() {
        // Set custom ports
        std::env::set_var("ORKEE_DISCOVERY_PORTS", "4000,5000,6000");

        let ports = get_discovery_ports();

        // Should return custom ports
        assert_eq!(ports, vec![4000, 5000, 6000]);

        // Cleanup
        std::env::remove_var("ORKEE_DISCOVERY_PORTS");
    }

    #[test]
    #[serial]
    fn test_get_discovery_ports_invalid() {
        // Set ports with some invalid values
        std::env::set_var("ORKEE_DISCOVERY_PORTS", "4000,invalid,5000");

        let ports = get_discovery_ports();

        // Should skip invalid values
        assert_eq!(ports, vec![4000, 5000]);

        // Cleanup
        std::env::remove_var("ORKEE_DISCOVERY_PORTS");
    }

    #[test]
    fn test_load_env_from_directory() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");

        // Create a test .env file
        fs::write(
            &env_file,
            "DATABASE_URL=postgres://localhost\nAPI_KEY=secret123\n",
        )
        .unwrap();

        let env_vars = load_env_from_directory(temp_dir.path());

        assert_eq!(
            env_vars.get("DATABASE_URL"),
            Some(&"postgres://localhost".to_string())
        );
        assert_eq!(env_vars.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(env_vars.get("NODE_ENV"), Some(&"development".to_string())); // Auto-added
    }

    #[test]
    fn test_load_env_from_directory_quotes() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");

        // Test with quoted values
        fs::write(
            &env_file,
            "QUOTED=\"value with spaces\"\nSINGLE='another value'\n",
        )
        .unwrap();

        let env_vars = load_env_from_directory(temp_dir.path());

        assert_eq!(
            env_vars.get("QUOTED"),
            Some(&"value with spaces".to_string())
        );
        assert_eq!(env_vars.get("SINGLE"), Some(&"another value".to_string()));
    }

    #[test]
    fn test_load_env_from_directory_comments() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");

        // Test with comments and empty lines
        fs::write(
            &env_file,
            "# This is a comment\nVALID=value\n\n# Another comment\n",
        )
        .unwrap();

        let env_vars = load_env_from_directory(temp_dir.path());

        assert_eq!(env_vars.get("VALID"), Some(&"value".to_string()));
        assert!(!env_vars.contains_key("#"));
    }
}
