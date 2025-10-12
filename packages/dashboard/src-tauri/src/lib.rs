use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

mod tray;
use tray::TrayManager;

// Global runtime storage to prevent premature drop during cleanup
static GLOBAL_RUNTIME: Mutex<Option<Arc<tokio::runtime::Runtime>>> = Mutex::new(None);

// Track cleanup execution to prevent double cleanup
static CLEANUP_DONE: AtomicBool = AtomicBool::new(false);

// Timeout constants for cleanup operations
const CLEANUP_HTTP_TIMEOUT_SECS: u64 = 3;
const CLEANUP_CONNECT_TIMEOUT_SECS: u64 = 1;
const CLEANUP_TOTAL_TIMEOUT_SECS: u64 = 3;

/// Parse an environment variable with fallback support.
///
/// Attempts to parse an environment variable, falling back to a secondary variable
/// and finally to a default value if neither is set or parsing fails.
///
/// # Arguments
///
/// * `primary_var` - The primary environment variable name to check
/// * `fallback_var` - The fallback environment variable name if primary is not set
/// * `default` - The default value to use if neither variable is set or parseable
///
/// # Type Parameters
///
/// * `T` - The type to parse the environment variable into (must implement `FromStr`)
///
/// # Returns
///
/// Returns the parsed value from the primary variable, fallback variable, or default value.
///
/// # Examples
///
/// ```
/// let port: u16 = parse_env_with_fallback("ORKEE_UI_PORT", "VITE_PORT", 5173);
/// ```
fn parse_env_with_fallback<T>(primary_var: &str, fallback_var: &str, default: T) -> T
where
    T: FromStr,
{
    std::env::var(primary_var)
        .or_else(|_| std::env::var(fallback_var))
        .ok()
        .and_then(|v| v.parse::<T>().ok())
        .unwrap_or(default)
}

// Store the CLI server process handle and ports globally
struct CliServerState {
    process: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
    api_port: u16,
}

/// Perform cleanup of all development servers.
///
/// Gracefully stops all running development servers via the preview API before
/// the application exits. This prevents orphaned server processes. The cleanup
/// uses short timeouts to avoid blocking application shutdown.
///
/// # Arguments
///
/// * `api_port` - The API port to connect to for stopping servers
///
/// # Returns
///
/// Returns `Ok(())` on success. Errors during cleanup are logged but not propagated
/// to allow shutdown to continue.
///
/// # Errors
///
/// Returns an error if the HTTP request to stop servers fails, though this is
/// typically logged and ignored during shutdown.
async fn cleanup_servers(api_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting cleanup of dev servers...");

    // Create HTTP client with short timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(CLEANUP_HTTP_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(CLEANUP_CONNECT_TIMEOUT_SECS))
        .build()?;

    // Try to stop all preview servers via API
    let stop_url = format!("http://localhost:{}/api/preview/servers/stop-all", api_port);
    match client.post(&stop_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("Successfully stopped all dev servers");
            } else {
                eprintln!("Failed to stop dev servers: HTTP {}", response.status());
            }
        }
        Err(e) => {
            eprintln!("Failed to stop dev servers (API may be down): {}", e);
        }
    }

    Ok(())
}

/// Terminate the Orkee CLI server process.
///
/// Sends a kill signal to stop the CLI server process. This should be called
/// after cleanup of development servers to ensure graceful shutdown.
///
/// # Arguments
///
/// * `child` - The CLI server process handle to terminate
fn kill_cli_process(child: tauri_plugin_shell::process::CommandChild) {
    println!("Stopping Orkee CLI server...");
    match child.kill() {
        Ok(_) => println!("CLI server stopped successfully"),
        Err(e) => eprintln!("Failed to kill CLI server: {}", e),
    }
}

/// Handle mutex poisoning and attempt to recover the CLI process.
///
/// When the process mutex is poisoned (due to a panic while locked), this function
/// logs diagnostic information and attempts to recover the process handle from the
/// poisoned state. If recovery succeeds, it properly terminates the CLI process.
///
/// # Arguments
///
/// * `poisoned` - The poisoned mutex error containing the guard
/// * `location` - Human-readable description of where the poisoning was detected
///
/// # Notes
///
/// This function will always attempt recovery, even if the mutex was poisoned.
/// The poisoning indicates a previous panic, but the process handle may still
/// be valid and needs to be properly cleaned up.
fn recover_cli_process(
    poisoned: std::sync::PoisonError<
        std::sync::MutexGuard<Option<tauri_plugin_shell::process::CommandChild>>,
    >,
    location: &str,
) {
    eprintln!("=== MUTEX POISONING DETECTED ===");
    eprintln!("Location: {}", location);
    eprintln!("Thread: {:?}", std::thread::current().id());
    eprintln!("Status: CRITICAL - Process mutex was poisoned");
    eprintln!("Cause: Another thread panicked while holding this mutex");
    eprintln!("Action: Attempting recovery from poisoned state");
    eprintln!("Note: Please report this issue if it occurs frequently at https://github.com/OrkeeAI/orkee/issues");
    eprintln!("================================");

    let mut guard = poisoned.into_inner();
    if let Some(child) = guard.take() {
        eprintln!("✓ Recovery successful: Process handle recovered from poisoned mutex");
        kill_cli_process(child);
    } else {
        eprintln!("✗ FATAL: No process handle found in poisoned mutex - orphaned process likely");
    }
}

/// Perform cleanup of all servers and processes before shutdown.
///
/// Centralizes the cleanup logic to avoid duplication across different shutdown paths.
/// This function stops the tray polling, gracefully stops dev servers, and terminates
/// the CLI process.
///
/// # Arguments
///
/// * `app_handle` - The Tauri application handle
/// * `context` - Human-readable description of the cleanup context for logging
///
/// # Returns
///
/// Returns `Ok(())` on successful cleanup. Errors are logged but don't prevent
/// the process termination from completing.
fn perform_cleanup(
    app_handle: &tauri::AppHandle,
    context: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting cleanup ({})...", context);

    // Stop tray polling first
    if let Some(tray_manager) = app_handle.try_state::<TrayManager>() {
        tray_manager.stop_polling();
    }

    // Get the CLI server state
    let Some(state) = app_handle.try_state::<CliServerState>() else {
        println!("No CLI server state found, cleanup complete");
        return Ok(()); // No cleanup needed if state doesn't exist
    };

    let api_port = state.api_port;

    // Get or create tokio runtime for async cleanup
    let runtime = match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(_) => {
            // Create new runtime if none exists
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    // Store runtime in Arc to prevent drop
                    let runtime = Arc::new(rt);
                    *GLOBAL_RUNTIME.lock().unwrap() = Some(runtime.clone());
                    runtime.handle().clone()
                }
                Err(e) => {
                    eprintln!("Failed to create tokio runtime for cleanup: {}", e);
                    eprintln!("Skipping async cleanup - proceeding to kill CLI process");
                    // Kill CLI process directly without async cleanup
                    match state.process.lock() {
                        Ok(mut process) => {
                            if let Some(child) = process.take() {
                                kill_cli_process(child);
                            }
                        }
                        Err(poisoned) => {
                            recover_cli_process(
                                poisoned,
                                &format!("Cleanup without runtime ({})", context),
                            );
                        }
                    }
                    return Err(Box::new(e));
                }
            }
        }
    };

    // Block on cleanup to ensure dev servers are stopped before killing CLI
    let cleanup_result = runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(CLEANUP_TOTAL_TIMEOUT_SECS),
            cleanup_servers(api_port),
        )
        .await
    });

    match cleanup_result {
        Ok(Ok(_)) => println!("Cleanup completed successfully"),
        Ok(Err(e)) => eprintln!("Cleanup error: {}", e),
        Err(_) => eprintln!(
            "Cleanup timed out after {} seconds",
            CLEANUP_TOTAL_TIMEOUT_SECS
        ),
    }

    // Now safe to kill CLI server process after cleanup completes
    match state.process.lock() {
        Ok(mut process) => {
            if let Some(child) = process.take() {
                kill_cli_process(child);
            }
        }
        Err(poisoned) => {
            recover_cli_process(poisoned, &format!("After cleanup ({})", context));
        }
    }

    Ok(())
}

/// Perform cleanup exactly once, preventing double cleanup from multiple shutdown paths.
///
/// Uses atomic compare-and-swap to ensure cleanup runs only once even if called
/// from both WindowEvent::Destroyed and RunEvent::Exit handlers.
///
/// # Arguments
///
/// * `app_handle` - The Tauri application handle
/// * `context` - Human-readable description of the cleanup context for logging
fn perform_cleanup_once(app_handle: &tauri::AppHandle, context: &str) {
    if CLEANUP_DONE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_ok()
    {
        let _ = perform_cleanup(app_handle, context);
    } else {
        println!(
            "Cleanup already performed, skipping duplicate call from {}",
            context
        );
    }
}

/// Get the API port that the CLI server is running on.
///
/// This Tauri command is exposed to the frontend to allow it to discover
/// which port the backend API is listening on.
///
/// # Arguments
///
/// * `state` - Tauri-managed state containing the CLI server information
///
/// # Returns
///
/// Returns the API port number as a `u16`.
#[tauri::command]
fn get_api_port(state: tauri::State<CliServerState>) -> u16 {
    state.api_port
}

/// Find an available port dynamically.
///
/// Searches for an unused port on the system that can be bound for the API server.
///
/// # Returns
///
/// Returns `Ok(u16)` with an available port number.
///
/// # Errors
///
/// Returns `Err(String)` if no available port can be found in the system.
fn find_available_port() -> Result<u16, String> {
    portpicker::pick_unused_port()
        .ok_or_else(|| "Failed to find an available port in the system".to_string())
}

/// Log output from the CLI server sidecar process.
///
/// Spawns a background task that listens to the sidecar's stdout and stderr streams
/// and logs them to the console. This is essential for debugging server startup issues
/// and monitoring server behavior.
///
/// # Arguments
///
/// * `rx` - Receiver for command events (stdout, stderr, errors, termination)
///
/// # Notes
///
/// The task runs until the receiver is closed (when the sidecar process exits).
/// Output is logged with appropriate prefixes to distinguish stdout from stderr.
fn log_sidecar_output(mut rx: tauri::async_runtime::Receiver<CommandEvent>) {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    if let Ok(output) = String::from_utf8(line) {
                        print!("[CLI Server] {}", output);
                    }
                }
                CommandEvent::Stderr(line) => {
                    if let Ok(output) = String::from_utf8(line) {
                        eprint!("[CLI Server Error] {}", output);
                    }
                }
                CommandEvent::Error(err) => {
                    eprintln!("[CLI Server] Command error: {}", err);
                }
                CommandEvent::Terminated(payload) => {
                    if let Some(code) = payload.code {
                        println!("[CLI Server] Process terminated with exit code: {}", code);
                    } else {
                        println!("[CLI Server] Process terminated");
                    }
                }
                _ => {}
            }
        }
    });
}

/// Main entry point for the Tauri application.
///
/// Initializes and runs the Orkee dashboard application with the following features:
/// - Spawns the Orkee CLI server as a sidecar process
/// - Manages system tray with server status
/// - Handles graceful shutdown and cleanup
/// - Configures window behavior (minimize to tray, macOS activation policy)
///
/// The application performs these key operations on startup:
/// 1. Finds an available port for the API server
/// 2. Spawns the CLI server with appropriate flags (dev mode in debug builds)
/// 3. Initializes the system tray
/// 4. Shows and focuses the main window
/// 5. Opens DevTools in debug builds
///
/// On shutdown:
/// 1. Stops the tray polling loop
/// 2. Gracefully stops all development servers via API
/// 3. Terminates the CLI server process
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Set the activation policy on macOS
            // Options:
            // - Accessory: No Dock icon, no Cmd+Tab (menu bar only) - CURRENT
            // - Regular: Shows in Dock and Cmd+Tab (standard app)
            // - Prohibited: Hidden by default but can show in some contexts
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);

            // Find available port dynamically
            let api_port = match find_available_port() {
                Ok(port) => port,
                Err(e) => {
                    eprintln!("Critical error: {}", e);
                    eprintln!("Cannot start application without an available port");
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, e)));
                }
            };
            // Get UI port from environment or use default
            let ui_port: u16 = parse_env_with_fallback("ORKEE_UI_PORT", "VITE_PORT", 5173);

            println!("Using dynamic API port: {} and UI port: {}", api_port, ui_port);

            // Start the Orkee CLI server as a sidecar
            let shell = app.shell();

            // Get the sidecar command for the orkee binary
            let sidecar_command = match shell.sidecar("orkee") {
                Ok(cmd) => cmd,
                Err(e) => {
                    eprintln!("Failed to create sidecar command for orkee binary: {}", e);
                    eprintln!("This usually means the orkee binary is not found or not properly configured");
                    return Err(Box::new(e));
                }
            };

            // Spawn the CLI server with dashboard command and log its output
            #[cfg(debug_assertions)]
            let child = match sidecar_command
                .args([
                    "dashboard",
                    "--dev",  // Use local dashboard in dev mode
                    "--api-port", &api_port.to_string(),
                    "--ui-port", &ui_port.to_string(),
                ])
                .spawn()
            {
                Ok((rx, child)) => {
                    log_sidecar_output(rx);
                    child
                }
                Err(e) => {
                    eprintln!("Failed to spawn orkee CLI server process: {}", e);
                    eprintln!("Check that the orkee binary has execute permissions and is not corrupted");
                    return Err(Box::new(e));
                }
            };

            #[cfg(not(debug_assertions))]
            let child = match sidecar_command
                .args([
                    "dashboard",
                    "--api-port", &api_port.to_string(),
                    "--ui-port", &ui_port.to_string(),
                ])
                .spawn()
            {
                Ok((rx, child)) => {
                    log_sidecar_output(rx);
                    child
                }
                Err(e) => {
                    eprintln!("Failed to spawn orkee CLI server process: {}", e);
                    eprintln!("Check that the orkee binary has execute permissions and is not corrupted");
                    return Err(Box::new(e));
                }
            };

            println!("Started Orkee CLI server on port {}", api_port);

            // Store the process handle and port so we can access them later
            app.manage(CliServerState {
                process: Mutex::new(Some(child)),
                api_port,
            });

            // Initialize the tray
            let mut tray_manager = TrayManager::new(app.handle().clone(), api_port);
            match tray_manager.init(app) {
                Ok(_) => println!("Tray initialized successfully"),
                Err(e) => eprintln!("Failed to initialize tray: {}", e),
            }
            app.manage(tray_manager);

            // Show and focus the main window on startup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                } else {
                    eprintln!("Warning: Could not open devtools - main window not found");
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_api_port])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Instead of closing, hide the window (minimize to tray)
                    // Users can quit from the tray menu
                    if let Err(e) = window.hide() {
                        eprintln!("Failed to hide window on close: {}", e);
                    }
                    api.prevent_close();
                }
                tauri::WindowEvent::Destroyed => {
                    // When the window is actually destroyed (app quitting)
                    perform_cleanup_once(&window.app_handle(), "window destroyed");
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .map_err(|e| {
            eprintln!("FATAL: Error building Tauri application: {}", e);
            std::process::exit(1);
        })
        .unwrap() // Safe: map_err calls exit, so this only runs on success
        .run(|app_handle, event| {
            // Handle app-level events including unexpected exits
            match event {
                tauri::RunEvent::Exit => {
                    perform_cleanup_once(app_handle, "app exit");
                }
                tauri::RunEvent::ExitRequested { .. } => {
                    // Don't prevent exit, but ensure cleanup happens
                    println!("Exit requested, cleanup will occur in Exit event");
                    // Don't call prevent_exit - let it proceed to Exit event
                }
                _ => {}
            }
        });
}
