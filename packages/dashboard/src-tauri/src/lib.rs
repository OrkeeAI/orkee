use orkee_config::constants;
use orkee_config::env::parse_env_with_fallback;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tracing::{debug, error, info, warn};

mod server_restart;
mod tray;
use tray::TrayManager;

// Track cleanup execution to prevent double cleanup
static CLEANUP_DONE: AtomicBool = AtomicBool::new(false);

// Store the CLI server process handle and ports globally
struct CliServerState {
    process: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
    api_port: u16,
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
    info!("Dev servers will continue running in the background");

    // Stop tray polling first
    if let Some(tray_manager) = app_handle.try_state::<TrayManager>() {
        tray_manager.stop_polling();
    }

    // Get the CLI server state
    let Some(state) = app_handle.try_state::<CliServerState>() else {
        debug!("No CLI server state found, cleanup complete");
        return Ok(()); // No cleanup needed if state doesn't exist
    };

    // Kill CLI server process
    // Dev servers will continue running and will be recovered from registry on next launch
    match state.process.lock() {
        Ok(mut process) => {
            if let Some(child) = process.take() {
                kill_cli_process(child);
            }
        }
        Err(poisoned) => {
            recover_cli_process(poisoned, &format!("Cleanup ({})", context));
        }
    }

    info!("Cleanup complete");
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

/// Get the API token for authenticating with the CLI server.
///
/// Reads the API token from ~/.orkee/api-token file. This token is required
/// for authenticating API requests to the backend server.
///
/// # Returns
///
/// Returns `Ok(String)` with the API token, or `Err(String)` if the token
/// file cannot be read.
///
/// # Errors
///
/// Returns error if:
/// - Home directory cannot be determined
/// - Token file does not exist
/// - Token file cannot be read
/// - Token is empty or invalid
#[tauri::command]
fn get_api_token() -> Result<String, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?;

    let token_path = home_dir.join(".orkee").join("api-token");

    if !token_path.exists() {
        return Err("API token file not found. Please restart the Orkee server to generate a new token.".to_string());
    }

    let token = std::fs::read_to_string(&token_path)
        .map_err(|e| format!("Failed to read API token: {}", e))?
        .trim()
        .to_string();

    if token.is_empty() {
        return Err("API token is empty. Please restart the Orkee server to generate a new token.".to_string());
    }

    Ok(token)
}

/// Check if the orkee CLI binary is installed in the system PATH.
///
/// Uses the `which` command on Unix systems to determine if `orkee` is available.
///
/// # Returns
///
/// Returns `true` if the CLI is installed, `false` otherwise.
#[tauri::command]
fn check_cli_installed() -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("which")
            .arg("orkee")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        // On Windows, check using 'where' command
        std::process::Command::new("where")
            .arg("orkee")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Install the orkee CLI binary to /usr/local/bin on macOS.
///
/// Copies the binary from the app bundle to `/usr/local/bin/orkee` using sudo
/// for privilege escalation. This allows users to access the CLI from any terminal.
///
/// # Arguments
///
/// * `app_handle` - The Tauri application handle to access resource paths
///
/// # Returns
///
/// Returns `Ok(String)` with success message, or `Err(String)` with error details.
///
/// # Errors
///
/// Returns error if:
/// - Not running on macOS
/// - Binary not found in app bundle
/// - Sudo command fails
/// - File operations fail
#[tauri::command]
async fn install_cli_macos(_app_handle: tauri::AppHandle) -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    {
        return Err("This command is only available on macOS".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let app_handle = _app_handle; // Use on macOS
        // Get the binary path from the app bundle (MacOS directory, not Resources)
        // On macOS, Tauri places externalBin files in Contents/MacOS/
        let resource_dir = app_handle
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource directory: {}", e))?;

        // Navigate from Resources to MacOS directory
        // resource_dir is typically /Applications/Orkee.app/Contents/Resources
        // We need /Applications/Orkee.app/Contents/MacOS
        let macos_dir = resource_dir
            .parent() // Go up to Contents/
            .ok_or_else(|| "Failed to get Contents directory".to_string())?
            .join("MacOS");

        let source_path = macos_dir.join("orkee");

        // Verify source binary exists
        if !source_path.exists() {
            return Err(format!(
                "orkee binary not found in app bundle at: {}",
                source_path.display()
            ));
        }

        let target_path = "/usr/local/bin/orkee";

        // Use osascript to prompt for admin password and run installation
        // This provides a native macOS authentication dialog
        let script = format!(
            r#"do shell script "mkdir -p /usr/local/bin && cp '{}' '{}' && chmod +x '{}'" with administrator privileges"#,
            source_path.display(),
            target_path,
            target_path
        );

        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| format!("Failed to execute installation command: {}", e))?;

        if output.status.success() {
            Ok(format!("CLI successfully installed to {}", target_path))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Installation failed: {}", stderr))
        }
    }
}

/// Get the user's preference for showing the CLI installation prompt.
///
/// Reads from ~/.orkee/config.json to determine if the prompt should be shown.
///
/// # Returns
///
/// Returns one of: "show", "later", or "never"
#[tauri::command]
fn get_cli_prompt_preference() -> String {
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => return "show".to_string(), // Default to showing if can't read home
    };

    let config_path = home_dir.join(".orkee").join("config.json");

    // If config doesn't exist, default to showing prompt
    if !config_path.exists() {
        return "show".to_string();
    }

    // Read and parse config
    match std::fs::read_to_string(&config_path) {
        Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(config) => config
                .get("cli_prompt_preference")
                .and_then(|v| v.as_str())
                .unwrap_or("show")
                .to_string(),
            Err(_) => "show".to_string(),
        },
        Err(_) => "show".to_string(),
    }
}

/// Set the user's preference for showing the CLI installation prompt.
///
/// Writes to ~/.orkee/config.json to persist the user's choice.
///
/// # Arguments
///
/// * `preference` - One of: "show", "later", or "never"
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err(String)` with error details.
#[tauri::command]
fn set_cli_prompt_preference(preference: String) -> Result<(), String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let orkee_dir = home_dir.join(".orkee");
    let config_path = orkee_dir.join("config.json");

    // Create .orkee directory if it doesn't exist
    std::fs::create_dir_all(&orkee_dir)
        .map_err(|e| format!("Failed to create .orkee directory: {}", e))?;

    // Read existing config or create new one
    let mut config = if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        serde_json::from_str::<serde_json::Value>(&contents).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Update the preference
    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "cli_prompt_preference".to_string(),
            serde_json::Value::String(preference),
        );
    }

    // Write back to file
    let contents = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, contents)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Force an immediate refresh of the system tray menu
///
/// Triggers the tray manager to fetch the latest server list and update
/// the menu immediately, bypassing the polling interval. Useful for instant
/// UI updates when servers start or stop.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle to access the tray manager
///
/// # Returns
///
/// Returns `Ok(())` if the refresh was triggered successfully.
///
/// # Errors
///
/// Returns `Err(String)` if the tray manager is not available in app state.
#[tauri::command]
fn force_refresh_tray(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(tray_manager) = app_handle.try_state::<TrayManager>() {
        tray_manager.force_refresh();
        Ok(())
    } else {
        Err("Tray manager not available".to_string())
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
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
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
        .invoke_handler(tauri::generate_handler![
            get_api_port,
            get_api_token,
            check_cli_installed,
            install_cli_macos,
            get_cli_prompt_preference,
            set_cli_prompt_preference,
            force_refresh_tray
        ])
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
