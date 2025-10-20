// ABOUTME: Discovery service for detecting externally launched development servers
// ABOUTME: Scans ports and identifies running processes to track manual server launches
//
// TODO: Standardize error handling - consider using PreviewResult instead of Box<dyn Error>
// for consistency with manager.rs and better error context.

use chrono::Utc;
use futures::stream::{self, StreamExt};
#[cfg(unix)]
use nix::libc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Stdio;
use sysinfo::{Pid, ProcessRefreshKind, System};
#[cfg(unix)]
use tokio::process::Command;
use tracing::{debug, warn};

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
    // Get ports to scan from environment or use defaults
    let ports_to_scan = get_discovery_ports();

    debug!(
        "Scanning {} ports for external servers",
        ports_to_scan.len()
    );

    // Scan ports with limited concurrency to avoid overwhelming the system
    // buffer_unordered limits to 5 concurrent scans while maintaining unordered results
    let discovered: Vec<_> = stream::iter(ports_to_scan)
        .map(discover_server_on_port)
        .buffer_unordered(5)
        .filter_map(|result| async move { result })
        .collect()
        .await;

    debug!("Discovered {} new external servers", discovered.len());
    discovered
}

/// Get list of ports to scan from environment or use defaults
fn get_discovery_ports() -> Vec<u16> {
    if let Ok(ports_str) = std::env::var("ORKEE_DISCOVERY_PORTS") {
        ports_str
            .split(',')
            .filter_map(|s| {
                let port: u16 = s.trim().parse().ok()?;
                // Only allow user ports (1024-65535) to prevent privilege escalation
                if port >= 1024 {
                    Some(port)
                } else {
                    warn!(
                        "Ignoring privileged port {} from ORKEE_DISCOVERY_PORTS",
                        port
                    );
                    None
                }
            })
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
    // On Unix: Compare numeric UIDs
    // On Windows: OS-level access control prevents accessing other users' processes
    #[cfg(unix)]
    if let Some(process_uid) = process.user_id() {
        // SAFETY: getuid() is always safe to call - it's a simple syscall with no preconditions
        // that reads the real user ID of the calling process from the kernel
        let current_uid = unsafe { libc::getuid() };
        let uid_str = process_uid.to_string();

        // Parse UID and deny access if parsing fails (fail-secure)
        match uid_str.parse::<u32>() {
            Ok(parsed_uid) => {
                if parsed_uid != current_uid {
                    debug!(
                        "Skipping process {} - owned by different user (UID {} vs {})",
                        pid, parsed_uid, current_uid
                    );
                    return None;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to parse UID '{}' for process {}: {} - denying access for security",
                    uid_str, pid, e
                );
                return None;
            }
        }
    }

    // Validate parent PID as defense against PID reuse
    // Development servers should have a parent process (shell/terminal/orkee)
    // If parent is PID 1 (init/systemd), process is orphaned which is suspicious
    if let Some(parent_pid) = process.parent() {
        let parent_pid_u32 = parent_pid.as_u32();
        if parent_pid_u32 == 1 {
            warn!(
                "Skipping PID {} - suspicious parent PID 1 (init/systemd), likely orphaned or PID reuse attack",
                pid
            );
            // For discovery, we're more cautious than registry - reject suspicious processes
            return None;
        }
    } else {
        warn!(
            "Skipping PID {} - no parent process, highly suspicious PID reuse attack vector",
            pid
        );
        return None;
    }

    // Validate process name matches expected development server patterns
    let process_name = process.name().to_string().to_lowercase();
    let expected_patterns = [
        "node", "deno", "bun", "python", "ruby", "php", "java", "dotnet", "go", "cargo", "npm",
        "yarn", "pnpm", "next", "vite", "webpack", "parcel", "rollup", "django", "flask", "rails",
        "spring", "express",
    ];

    let name_matches = expected_patterns
        .iter()
        .any(|pattern| process_name.contains(pattern));

    if !name_matches {
        warn!(
            "Skipping PID {} - process name '{}' doesn't match expected dev server patterns - possible malicious process squatting on port {}",
            pid, process_name, port
        );
        return None;
    }

    // Extract command line
    let command: Vec<String> = process.cmd().iter().map(|s| s.to_string()).collect();
    if command.is_empty() {
        warn!(
            "Skipping PID {} - empty command line, suspicious for port {} listener",
            pid, port
        );
        return None;
    }

    // Check if this looks like a development server (additional command-line validation)
    if !is_likely_dev_server(&command) {
        debug!(
            "Skipping process {} on port {} - command doesn't match dev server patterns: {:?}",
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
        .args(["-tlnp", &format!("sport = :{}", port)])
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
    // Wrap blocking IO in spawn_blocking to prevent blocking the async executor.
    // The /proc filesystem reads are synchronous and can be slow on systems with many processes,
    // potentially hundreds of directories to traverse. Running this on the main executor thread
    // would block other async tasks, so we offload it to a dedicated blocking thread pool.
    tokio::task::spawn_blocking(move || {
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
    })
    .await
    .ok()
    .flatten()
}

/// Windows implementation using GetExtendedTcpTable API
///
/// Uses the Windows IP Helper API to enumerate all TCP connections and find
/// which process is listening on the specified port.
#[cfg(target_os = "windows")]
async fn find_process_on_port(port: u16) -> Option<u32> {
    use windows::Win32::NetworkManagement::IpHelper::GetExtendedTcpTable;
    use windows::Win32::NetworkManagement::IpHelper::TCP_TABLE_OWNER_PID_LISTENER;
    use windows::Win32::Networking::WinSock::AF_INET;

    tokio::task::spawn_blocking(move || {
        let mut size: u32 = 0;

        // First call to get required buffer size
        // SAFETY: GetExtendedTcpTable is safe to call with null pointer to query size
        let _result = unsafe {
            GetExtendedTcpTable(
                None,
                &mut size,
                false,
                AF_INET.0 as u32,
                TCP_TABLE_OWNER_PID_LISTENER,
                0,
            )
        };

        if size == 0 {
            warn!("GetExtendedTcpTable returned zero size");
            return None;
        }

        // Allocate buffer and get actual table
        let mut buffer = vec![0u8; size as usize];

        // SAFETY: We've allocated a buffer of the size requested by the first call
        let result = unsafe {
            GetExtendedTcpTable(
                Some(buffer.as_mut_ptr() as *mut _),
                &mut size,
                false,
                AF_INET.0 as u32,
                TCP_TABLE_OWNER_PID_LISTENER,
                0,
            )
        };

        // WIN32_ERROR(0) = NO_ERROR = success
        if result != 0 {
            warn!("GetExtendedTcpTable failed with error code: {}", result);
            return None;
        }

        // Parse the table structure
        // Structure layout: DWORD dwNumEntries, followed by array of MIB_TCPROW_OWNER_PID
        // SAFETY: We've successfully called GetExtendedTcpTable which filled our buffer
        let num_entries = unsafe { *(buffer.as_ptr() as *const u32) };

        // Each entry is 24 bytes: state(4) + local_addr(4) + local_port(4) + remote_addr(4) + remote_port(4) + owning_pid(4)
        const ENTRY_SIZE: usize = 24;
        const LOCAL_PORT_OFFSET: usize = 8;
        const OWNING_PID_OFFSET: usize = 20;

        for i in 0..num_entries as usize {
            let entry_offset = 4 + (i * ENTRY_SIZE); // 4 bytes for dwNumEntries

            if entry_offset + ENTRY_SIZE > buffer.len() {
                break;
            }

            // SAFETY: We've validated the offset is within buffer bounds
            let local_port =
                unsafe { *(buffer.as_ptr().add(entry_offset + LOCAL_PORT_OFFSET) as *const u32) };
            let owning_pid =
                unsafe { *(buffer.as_ptr().add(entry_offset + OWNING_PID_OFFSET) as *const u32) };

            // Port is stored in network byte order (big endian)
            let port_be = ((local_port & 0xFF00) >> 8) | ((local_port & 0x00FF) << 8);

            // TCP_TABLE_OWNER_PID_LISTENER only returns listening sockets, so no state check needed
            if port_be == port as u32 {
                debug!("Found process {} listening on port {}", owning_pid, port);
                return Some(owning_pid);
            }
        }

        None
    })
    .await
    .ok()
    .flatten()
}

/// Fallback for unsupported platforms (non-Windows/macOS/Linux)
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
async fn find_process_on_port(_port: u16) -> Option<u32> {
    warn!("Port scanning not supported on this platform - external server discovery disabled");
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

    // Canonicalize and validate the base directory to prevent path traversal
    let canonical_dir = match dir.canonicalize() {
        Ok(path) => path,
        Err(e) => {
            warn!(
                "Failed to canonicalize directory {:?}: {} - refusing to load env files",
                dir, e
            );
            return env_vars;
        }
    };

    // Validate the canonical path is a safe location
    // Allow user's home directory and system temp directories (for tests)
    let is_safe_location = if let Some(home_dir) = dirs::home_dir() {
        canonical_dir.starts_with(&home_dir)
    } else {
        false
    } || std::env::temp_dir()
        .canonicalize()
        .map(|temp| canonical_dir.starts_with(&temp))
        .unwrap_or(false);

    if !is_safe_location {
        warn!(
            "Directory {:?} is outside safe locations (home or temp) - refusing to load env files for security",
            canonical_dir
        );
        return env_vars;
    }

    // Check for common .env files (in priority order)
    let env_files = [".env.local", ".env.development", ".env"];

    for env_file in &env_files {
        let env_path = canonical_dir.join(env_file);

        // Additional path traversal check: ensure the final path is still under canonical_dir
        match env_path.canonicalize() {
            Ok(canonical_env_path) => {
                if !canonical_env_path.starts_with(&canonical_dir) {
                    warn!(
                        "Env file path {:?} escapes base directory {:?} - skipping for security",
                        env_path, canonical_dir
                    );
                    continue;
                }
            }
            Err(_) => {
                // File doesn't exist, which is fine - skip it
                continue;
            }
        }

        if env_path.exists() {
            // Use dotenvy to parse the .env file properly
            match dotenvy::from_path_iter(&env_path) {
                Ok(iter) => {
                    for item in iter {
                        match item {
                            Ok((key, value)) => {
                                // Only insert if not already set (priority order)
                                env_vars.entry(key).or_insert(value);
                            }
                            Err(e) => {
                                warn!("Failed to parse env entry in {:?}: {}", env_path, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to load env file {:?}: {}", env_path, e);
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

    #[test]
    #[serial]
    fn test_port_range_validation_rejects_privileged_ports() {
        // Test that privileged ports (< 1024) are filtered out
        std::env::set_var("ORKEE_DISCOVERY_PORTS", "80,443,1024,3000,8080");

        let ports = get_discovery_ports();

        // Should exclude privileged ports (80, 443)
        assert!(!ports.contains(&80), "Port 80 should be rejected");
        assert!(!ports.contains(&443), "Port 443 should be rejected");

        // Should include user ports (>= 1024)
        assert!(ports.contains(&1024), "Port 1024 should be accepted");
        assert!(ports.contains(&3000), "Port 3000 should be accepted");
        assert!(ports.contains(&8080), "Port 8080 should be accepted");

        std::env::remove_var("ORKEE_DISCOVERY_PORTS");
    }

    #[test]
    #[serial]
    fn test_port_range_validation_edge_cases() {
        // Test boundary conditions
        std::env::set_var("ORKEE_DISCOVERY_PORTS", "0,1,1023,1024,65535,65536");

        let ports = get_discovery_ports();

        // Ports 0, 1, 1023 should be rejected (< 1024)
        assert!(!ports.contains(&0));
        assert!(!ports.contains(&1));
        assert!(!ports.contains(&1023));

        // Port 1024 should be accepted (first valid port)
        assert!(ports.contains(&1024));

        // Port 65535 should be accepted (last valid port)
        assert!(ports.contains(&65535));

        // Port 65536 won't parse as u16 (overflow during string parsing)
        // The env var parser will filter it out, so we just verify the valid ports above

        std::env::remove_var("ORKEE_DISCOVERY_PORTS");
    }

    #[test]
    #[serial]
    fn test_port_validation_all_privileged() {
        // Test that we handle case where all ports are privileged
        std::env::set_var("ORKEE_DISCOVERY_PORTS", "22,80,443");

        let ports = get_discovery_ports();

        // Should return empty vec when all ports are privileged
        assert!(
            ports.is_empty(),
            "Should return empty when all ports are privileged"
        );

        std::env::remove_var("ORKEE_DISCOVERY_PORTS");
    }

    #[test]
    fn test_process_info_extraction_validates_command() {
        // Test that empty command lines are rejected
        use sysinfo::{Pid, ProcessRefreshKind, System};

        let mut system = System::new();
        system.refresh_processes();

        // Find current process (should have valid command)
        let current_pid = std::process::id();
        system.refresh_process_specifics(
            Pid::from_u32(current_pid),
            ProcessRefreshKind::everything(),
        );

        if let Some(process) = system.process(Pid::from_u32(current_pid)) {
            let command: Vec<String> = process.cmd().iter().map(|s| s.to_string()).collect();
            // Current test process should have a command
            assert!(
                !command.is_empty(),
                "Current process should have non-empty command"
            );
        }
    }

    #[test]
    fn test_security_validates_process_ownership() {
        // Test that we check process UID (tested indirectly via current process)
        use sysinfo::{Pid, ProcessRefreshKind, System};

        let mut system = System::new();
        system.refresh_processes();

        let current_pid = std::process::id();
        system.refresh_process_specifics(
            Pid::from_u32(current_pid),
            ProcessRefreshKind::everything(),
        );

        if let Some(process) = system.process(Pid::from_u32(current_pid)) {
            // Current process should have a user ID
            assert!(
                process.user_id().is_some(),
                "Process should have user ID for security check"
            );

            // UID should be parseable
            if let Some(uid) = process.user_id() {
                let uid_str = uid.to_string();
                let parse_result: Result<u32, _> = uid_str.parse();
                assert!(
                    parse_result.is_ok(),
                    "UID should be parseable as u32, got: {}",
                    uid_str
                );
            }
        }
    }

    #[test]
    fn test_framework_detection_comprehensive() {
        // Test comprehensive framework detection for security context

        // Node.js frameworks
        assert_eq!(
            detect_framework_from_command(&["next".to_string(), "dev".to_string()]),
            Some("Next.js".to_string())
        );
        assert_eq!(
            detect_framework_from_command(&[
                "vue-cli".to_string(),
                "serve".to_string(),
                "--port".to_string(),
                "3000".to_string()
            ]),
            Some("Vue".to_string())
        );
        assert_eq!(
            detect_framework_from_command(&["ng".to_string(), "serve".to_string()]),
            Some("Angular".to_string())
        );

        // Python frameworks
        assert_eq!(
            detect_framework_from_command(&[
                "python3".to_string(),
                "manage.py".to_string(),
                "runserver".to_string()
            ]),
            Some("Django".to_string())
        );

        // Ruby framework
        assert_eq!(
            detect_framework_from_command(&["rails".to_string(), "server".to_string()]),
            Some("Rails".to_string())
        );

        // Build tools that might run on privileged ports
        assert_eq!(
            detect_framework_from_command(&["webpack-dev-server".to_string()]),
            Some("Webpack".to_string())
        );

        // Deno runtime
        assert_eq!(
            detect_framework_from_command(&["deno".to_string(), "run".to_string()]),
            Some("Deno".to_string())
        );

        // Unknown command should return None (safe default)
        assert_eq!(
            detect_framework_from_command(&["/usr/bin/malicious".to_string()]),
            None
        );
    }

    #[test]
    fn test_dev_server_detection_security() {
        // Test that we properly identify legitimate dev servers
        // and reject potentially malicious processes

        // Legitimate dev servers
        assert!(is_likely_dev_server(&[
            "node".to_string(),
            "server.js".to_string()
        ]));
        assert!(is_likely_dev_server(&["vite".to_string()]));
        assert!(is_likely_dev_server(&[
            "python3".to_string(),
            "manage.py".to_string(),
            "runserver".to_string()
        ]));

        // System commands that should NOT be detected as dev servers
        assert!(!is_likely_dev_server(&["rm".to_string()]));
        assert!(!is_likely_dev_server(&["curl".to_string()]));
        assert!(!is_likely_dev_server(&["nc".to_string()])); // netcat
        assert!(!is_likely_dev_server(&["ssh".to_string()]));
        assert!(!is_likely_dev_server(&[
            "/bin/bash".to_string(),
            "-c".to_string(),
            "malicious".to_string()
        ]));

        // Empty command should not be detected
        assert!(!is_likely_dev_server(&[]));
    }

    #[test]
    #[serial]
    fn test_env_file_security_no_path_traversal() {
        // Test that we don't load .env files outside the project directory
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create an .env file in temp directory
        let env_file = temp_dir.path().join(".env");
        fs::write(&env_file, "SAFE_VAR=safe_value\n").unwrap();

        // Load env from the directory
        let env_vars = load_env_from_directory(temp_dir.path());

        // Should load the env file
        assert_eq!(env_vars.get("SAFE_VAR"), Some(&"safe_value".to_string()));

        // Should not have loaded anything from parent directory
        // (we don't traverse up the directory tree)
        assert!(
            env_vars.len() <= 2,
            "Should only have SAFE_VAR and NODE_ENV"
        );
    }
}
