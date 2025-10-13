use orkee_config::constants;
use orkee_config::env::parse_env_with_fallback;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tracing::{debug, error, info, warn};

mod tray;
mod server_restart;
use tray::TrayManager;

// Track cleanup execution to prevent double cleanup
static CLEANUP_DONE: AtomicBool = AtomicBool::new(false);

// Timeout constants for cleanup operations
const CLEANUP_HTTP_TIMEOUT_SECS: u64 = 3;
const CLEANUP_CONNECT_TIMEOUT_SECS: u64 = 1;
const CLEANUP_TOTAL_TIMEOUT_SECS: u64 = 3;

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
    info!("Starting cleanup of dev servers...");

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
                info!("Successfully stopped all dev servers");
            } else {
                error!("Failed to stop dev servers: HTTP {}", response.status());
            }
        }
        Err(e) => {
            error!("Failed to stop dev servers (API may be down): {}", e);
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
    info!("Stopping Orkee CLI server...");
    match child.kill() {
        Ok(_) => info!("CLI server stopped successfully"),
        Err(e) => error!("Failed to kill CLI server: {}", e),
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
    error!("=== MUTEX POISONING DETECTED ===");
    error!("Location: {}", location);
    error!("Thread: {:?}", std::thread::current().id());
    error!("Status: CRITICAL - Process mutex was poisoned");
    error!("Cause: Another thread panicked while holding this mutex");
    error!("Action: Attempting recovery from poisoned state");
    error!("Note: Please report this issue if it occurs frequently at https://github.com/OrkeeAI/orkee/issues");
    error!("================================");

    let mut guard = poisoned.into_inner();
    if let Some(child) = guard.take() {
        info!("✓ Recovery successful: Process handle recovered from poisoned mutex");
        kill_cli_process(child);
    } else {
        error!("✗ FATAL: No process handle found in poisoned mutex - orphaned process likely");
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
    info!("Starting cleanup ({})...", context);

    // Stop tray polling first
    if let Some(tray_manager) = app_handle.try_state::<TrayManager>() {
        tray_manager.stop_polling();
    }

    // Get the CLI server state
    let Some(state) = app_handle.try_state::<CliServerState>() else {
        debug!("No CLI server state found, cleanup complete");
        return Ok(()); // No cleanup needed if state doesn't exist
    };

    let api_port = state.api_port;

    // Create a dedicated runtime for cleanup that we own and control
    // This ensures the runtime won't be dropped before cleanup completes
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime for cleanup: {}", e);
            warn!("Proceeding to kill CLI process directly");
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
    };

    // Block on cleanup to ensure dev servers are stopped before killing CLI
    // Using block_on ensures the async operations complete before we continue
    let cleanup_result = runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(CLEANUP_TOTAL_TIMEOUT_SECS),
            cleanup_servers(api_port),
        )
        .await
    });

    match cleanup_result {
        Ok(Ok(_)) => info!("Cleanup completed successfully"),
        Ok(Err(e)) => error!("Cleanup error: {}", e),
        Err(_) => error!(
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

    // Explicitly drop the runtime to ensure all tasks are terminated
    drop(runtime);

    Ok(())
}

/// Perform cleanup exactly once, preventing double cleanup from multiple shutdown paths.
///
/// Uses atomic compare-and-swap to ensure cleanup runs only once even if called
/// from both WindowEvent::Destroyed and RunEvent::Exit handlers. Wraps the cleanup
/// in `catch_unwind` to handle panics gracefully - if cleanup panics, the flag is
/// reset to allow retry attempts.
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
        // Wrap cleanup in catch_unwind to handle panics gracefully
        // This prevents the CLEANUP_DONE flag from being stuck in 'true' state
        // if cleanup panics, allowing retry attempts
        let app_handle_clone = app_handle.clone();
        let context_owned = context.to_string();

        let panic_result = catch_unwind(AssertUnwindSafe(move || {
            perform_cleanup(&app_handle_clone, &context_owned)
        }));

        match panic_result {
            Ok(_) => {
                // Cleanup completed successfully (or with non-panic errors)
                debug!("Cleanup completed for context: {}", context);
            }
            Err(panic_payload) => {
                // Cleanup panicked - reset flag to allow retry
                error!("=== CLEANUP PANIC DETECTED ===");
                error!("Context: {}", context);
                error!("Thread: {:?}", std::thread::current().id());
                error!("Status: CRITICAL - Cleanup function panicked");
                error!("Action: Resetting CLEANUP_DONE flag to allow retry");
                error!("Panic payload: {:?}", panic_payload);
                error!("================================");

                // Reset the flag to allow cleanup retry
                CLEANUP_DONE.store(false, Ordering::SeqCst);

                // Log that the flag was reset
                warn!("CLEANUP_DONE flag reset to false - cleanup can be retried");
            }
        }
    } else {
        debug!(
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
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    if let Ok(output) = String::from_utf8(line) {
                        info!("[CLI Server] {}", output.trim_end());
                    }
                }
                CommandEvent::Stderr(line) => {
                    if let Ok(output) = String::from_utf8(line) {
                        warn!("[CLI Server Error] {}", output.trim_end());
                    }
                }
                CommandEvent::Error(err) => {
                    error!("[CLI Server] Command error: {}", err);
                }
                CommandEvent::Terminated(payload) => {
                    if let Some(code) = payload.code {
                        info!("[CLI Server] Process terminated with exit code: {}", code);
                    } else {
                        info!("[CLI Server] Process terminated");
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
    // Initialize tracing subscriber for structured logging
    // This enables logging from the tray module and other components
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .with_target(false) // Don't show module paths in logs
        .compact() // Use compact format for cleaner output
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // When another instance tries to launch, bring existing window to front
            info!("Another instance attempted to start (argv: {:?}, cwd: {:?})", argv, cwd);
            info!("Bringing existing window to front instead");

            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.show() {
                    error!("Failed to show window from second instance: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    error!("Failed to focus window from second instance: {}", e);
                }
            } else {
                error!("Main window not found when handling second instance");
            }
        }))
        .setup(|app| {
            // Set the activation policy on macOS
            //
            // This API requires `macOSPrivateApi: true` in tauri.conf.json to access
            // private macOS APIs for controlling app activation behavior.
            //
            // Activation Policy Options:
            // - Regular: Shows in Dock and Cmd+Tab (standard app) - CURRENT
            // - Accessory: No Dock icon, no Cmd+Tab (menu bar only)
            // - Prohibited: Hidden by default but can show in some contexts
            //
            // Fallback Behavior:
            // If macOSPrivateApi is disabled or the API fails, the app will use macOS
            // default behavior (Regular policy). The app will continue to function
            // normally, but without custom activation policy control.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);

            // Find available port dynamically
            let api_port = match find_available_port() {
                Ok(port) => port,
                Err(e) => {
                    error!("Critical error: {}", e);
                    error!("Cannot start application without an available port");
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, e)));
                }
            };
            // Get UI port from environment or use default
            let ui_port: u16 = parse_env_with_fallback(constants::ORKEE_UI_PORT, "VITE_PORT", 5173);

            info!("Using dynamic API port: {} and UI port: {}", api_port, ui_port);

            // Start the Orkee CLI server as a sidecar
            let shell = app.shell();

            // Get the sidecar command for the orkee binary
            let sidecar_command = match shell.sidecar("orkee") {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!("Failed to create sidecar command for orkee binary: {}", e);
                    error!("This usually means the orkee binary is not found or not properly configured");
                    return Err(Box::new(e));
                }
            };

            // Build args dynamically based on build profile
            let mut args = vec!["dashboard"];
            #[cfg(debug_assertions)]
            args.push("--dev");  // Use local dashboard in dev mode
            let api_port_str = api_port.to_string();
            let ui_port_str = ui_port.to_string();
            args.extend(["--api-port", &api_port_str, "--ui-port", &ui_port_str]);

            // Spawn the CLI server with dashboard command and log its output
            let child = match sidecar_command
                .args(args)
                .spawn()
            {
                Ok((rx, child)) => {
                    log_sidecar_output(rx);
                    child
                }
                Err(e) => {
                    error!("Failed to spawn orkee CLI server process: {}", e);
                    error!("Check that the orkee binary has execute permissions and is not corrupted");
                    return Err(Box::new(e));
                }
            };

            info!("Started Orkee CLI server on port {}", api_port);

            // Store the process handle and port so we can access them later
            app.manage(CliServerState {
                process: Mutex::new(Some(child)),
                api_port,
            });

            // Initialize the tray
            let mut tray_manager = TrayManager::new(app.handle().clone(), api_port);
            match tray_manager.init(app) {
                Ok(_) => info!("Tray initialized successfully"),
                Err(e) => error!("Failed to initialize tray: {}", e),
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
                    warn!("Could not open devtools - main window not found");
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
                        error!("Failed to hide window on close: {}", e);
                    }
                    api.prevent_close();
                }
                tauri::WindowEvent::Destroyed => {
                    // When the window is actually destroyed (app quitting)
                    perform_cleanup_once(window.app_handle(), "window destroyed");
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .map_err(|e| {
            error!("FATAL: Error building Tauri application: {}", e);
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
                    debug!("Exit requested, cleanup will occur in Exit event");
                    // Don't call prevent_exit - let it proceed to Exit event
                }
                _ => {}
            }
        });
}
