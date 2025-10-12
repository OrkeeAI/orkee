use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use std::sync::Mutex;
use std::time::Duration;
use std::str::FromStr;

mod tray;
use tray::TrayManager;

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

/// Manually refresh the system tray menu.
///
/// This Tauri command triggers an update of the tray menu to reflect the latest
/// server states. The tray menu normally updates automatically via polling, but
/// this can be called for immediate refresh.
///
/// # Arguments
///
/// * `_app` - Tauri application handle (unused)
/// * `tray_state` - Tauri-managed state containing the tray manager
///
/// # Returns
///
/// Returns `Ok(String)` with a success message, or `Err(String)` with an error message.
#[tauri::command]
async fn refresh_tray_menu(
    _app: tauri::AppHandle,
    tray_state: tauri::State<'_, TrayManager>,
) -> Result<String, String> {
    // Fetch latest servers from API
    let api_port = tray_state.api_port;
    let url = format!("http://localhost:{}/api/preview/servers", api_port);

    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                // For now just log that we refreshed
                println!("Tray menu refresh requested");
                Ok("Menu refreshed".to_string())
            } else {
                Err("Failed to fetch servers".to_string())
            }
        }
        Err(e) => Err(format!("Network error: {}", e))
    }
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

            // Spawn the CLI server with dashboard command
            #[cfg(debug_assertions)]
            let (_rx, child) = match sidecar_command
                .args([
                    "dashboard",
                    "--dev",  // Use local dashboard in dev mode
                    "--api-port", &api_port.to_string(),
                    "--ui-port", &ui_port.to_string(),
                ])
                .spawn()
            {
                Ok((rx, child)) => (rx, child),
                Err(e) => {
                    eprintln!("Failed to spawn orkee CLI server process: {}", e);
                    eprintln!("Check that the orkee binary has execute permissions and is not corrupted");
                    return Err(Box::new(e));
                }
            };

            #[cfg(not(debug_assertions))]
            let (_rx, child) = match sidecar_command
                .args([
                    "dashboard",
                    "--api-port", &api_port.to_string(),
                    "--ui-port", &ui_port.to_string(),
                ])
                .spawn()
            {
                Ok((rx, child)) => (rx, child),
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
        .invoke_handler(tauri::generate_handler![get_api_port, refresh_tray_menu])
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
                    // IMPORTANT: Block on cleanup to avoid race condition where CLI process
                    // is killed before dev servers are stopped

                    // Stop the tray polling loop first
                    if let Some(tray_manager) = window.app_handle().try_state::<TrayManager>() {
                        tray_manager.stop_polling();
                    }

                    if let Some(state) = window.app_handle().try_state::<CliServerState>() {
                        let api_port = state.api_port;

                        // Block on cleanup to ensure it completes before killing CLI process
                        // This prevents orphaned dev server processes
                        let runtime = match tokio::runtime::Handle::try_current() {
                            Ok(handle) => handle,
                            Err(_) => {
                                // If no runtime exists, create one
                                match tokio::runtime::Runtime::new() {
                                    Ok(rt) => rt.handle().clone(),
                                    Err(e) => {
                                        eprintln!("Failed to create tokio runtime for cleanup: {}", e);
                                        eprintln!("Skipping async cleanup - proceeding to kill CLI process");
                                        // Continue with process termination even if cleanup fails
                                        match state.process.lock() {
                                            Ok(mut process) => {
                                                if let Some(child) = process.take() {
                                                    kill_cli_process(child);
                                                }
                                            }
                                            Err(poisoned) => {
                                                eprintln!("=== MUTEX POISONING DETECTED ===");
                                                eprintln!("Location: Quick shutdown path (no async cleanup)");
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
                                        }
                                        return;
                                    }
                                }
                            }
                        };

                        let cleanup_result = runtime.block_on(async {
                            tokio::time::timeout(
                                Duration::from_secs(CLEANUP_TOTAL_TIMEOUT_SECS),
                                cleanup_servers(api_port)
                            ).await
                        });

                        match cleanup_result {
                            Ok(Ok(_)) => println!("Cleanup completed successfully"),
                            Ok(Err(e)) => eprintln!("Cleanup error: {}", e),
                            Err(_) => eprintln!("Cleanup timed out after {} seconds", CLEANUP_TOTAL_TIMEOUT_SECS),
                        }

                        // Now safe to kill CLI server process after cleanup completes
                        match state.process.lock() {
                            Ok(mut process) => {
                                if let Some(child) = process.take() {
                                    kill_cli_process(child);
                                }
                            }
                            Err(poisoned) => {
                                eprintln!("=== MUTEX POISONING DETECTED ===");
                                eprintln!("Location: Normal shutdown path (after async cleanup)");
                                eprintln!("Thread: {:?}", std::thread::current().id());
                                eprintln!("Status: CRITICAL - Process mutex was poisoned");
                                eprintln!("Cause: Another thread panicked while holding this mutex");
                                eprintln!("Action: Attempting recovery from poisoned state");
                                eprintln!("Note: Please report this issue if it occurs frequently at https://github.com/OrkeeAI/orkee/issues");
                                eprintln!("================================");

                                // Attempt to recover the process handle from poisoned mutex
                                let mut guard = poisoned.into_inner();
                                if let Some(child) = guard.take() {
                                    eprintln!("✓ Recovery successful: Process handle recovered from poisoned mutex");
                                    kill_cli_process(child);
                                } else {
                                    eprintln!("✗ FATAL: No process handle found in poisoned mutex - orphaned process likely");
                                    eprintln!("✗ Action required: You may need to manually kill the orkee CLI process");
                                }
                            }
                        }
                    }
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
                    println!("App exit event received, performing cleanup...");

                    // Stop the tray polling loop first
                    if let Some(tray_manager) = app_handle.try_state::<TrayManager>() {
                        tray_manager.stop_polling();
                    }

                    // Get the CLI server state and perform cleanup
                    if let Some(state) = app_handle.try_state::<CliServerState>() {
                        let api_port = state.api_port;

                        // Block on cleanup to ensure it completes before exit
                        // We use a short timeout to prevent hanging the exit
                        let runtime = match tokio::runtime::Runtime::new() {
                            Ok(rt) => rt,
                            Err(e) => {
                                eprintln!("Failed to create tokio runtime for exit cleanup: {}", e);
                                eprintln!("Proceeding directly to process termination");
                                // Directly kill CLI process if runtime creation fails
                                match state.process.lock() {
                                    Ok(mut process) => {
                                        if let Some(child) = process.take() {
                                            kill_cli_process(child);
                                        }
                                    }
                                    Err(poisoned) => {
                                        eprintln!("=== MUTEX POISONING DETECTED ===");
                                        eprintln!("Location: Exit event path (runtime creation failed)");
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
                                }
                                return;
                            }
                        };
                        let _ = runtime.block_on(async {
                            tokio::time::timeout(
                                Duration::from_secs(CLEANUP_TOTAL_TIMEOUT_SECS),
                                cleanup_servers(api_port)
                            ).await
                        });

                        // Kill CLI server process
                        match state.process.lock() {
                            Ok(mut process) => {
                                if let Some(child) = process.take() {
                                    kill_cli_process(child);
                                }
                            }
                            Err(poisoned) => {
                                eprintln!("=== MUTEX POISONING DETECTED ===");
                                eprintln!("Location: App exit event (normal shutdown after cleanup)");
                                eprintln!("Thread: {:?}", std::thread::current().id());
                                eprintln!("Status: CRITICAL - Process mutex was poisoned");
                                eprintln!("Cause: Another thread panicked while holding this mutex");
                                eprintln!("Action: Attempting recovery from poisoned state");
                                eprintln!("Note: Please report this issue if it occurs frequently at https://github.com/OrkeeAI/orkee/issues");
                                eprintln!("================================");

                                // Attempt to recover the process handle from poisoned mutex
                                let mut guard = poisoned.into_inner();
                                if let Some(child) = guard.take() {
                                    eprintln!("✓ Recovery successful: Process handle recovered from poisoned mutex");
                                    kill_cli_process(child);
                                } else {
                                    eprintln!("✗ FATAL: No process handle found in poisoned mutex - orphaned process likely");
                                    eprintln!("✗ Action required: You may need to manually kill the orkee CLI process");
                                }
                            }
                        }
                    }
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
