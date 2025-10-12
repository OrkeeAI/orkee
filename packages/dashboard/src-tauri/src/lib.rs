use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use std::sync::Mutex;
use std::time::Duration;

mod tray;
use tray::TrayManager;

// Store the CLI server process handle and ports globally
struct CliServerState {
    process: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
    api_port: u16,
}

/// Perform cleanup of all dev servers and the CLI process
/// This is called on app exit to ensure no orphaned processes
async fn cleanup_servers(api_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting cleanup of dev servers...");

    // Create HTTP client with short timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(1))
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

/// Kill the CLI server process
fn kill_cli_process(child: tauri_plugin_shell::process::CommandChild) {
    println!("Stopping Orkee CLI server...");
    match child.kill() {
        Ok(_) => println!("CLI server stopped successfully"),
        Err(e) => eprintln!("Failed to kill CLI server: {}", e),
    }
}

/// Tauri command to get the API port that the CLI server is running on
#[tauri::command]
fn get_api_port(state: tauri::State<CliServerState>) -> u16 {
    state.api_port
}

/// Tauri command to manually refresh the tray menu
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

/// Find an available port dynamically
fn find_available_port() -> u16 {
    portpicker::pick_unused_port()
        .expect("Failed to find an available port")
}

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
            let api_port = find_available_port();
            // Get UI port from environment or use default
            let ui_port: u16 = std::env::var("ORKEE_UI_PORT")
                .or_else(|_| std::env::var("VITE_PORT"))
                .unwrap_or_else(|_| "5173".to_string())
                .parse()
                .unwrap_or(5173);

            println!("Using dynamic API port: {} and UI port: {}", api_port, ui_port);

            // Start the Orkee CLI server as a sidecar
            let shell = app.shell();

            // Get the sidecar command for the orkee binary
            let sidecar_command = shell.sidecar("orkee")
                .expect("Failed to create sidecar command");

            // Spawn the CLI server with dashboard command
            #[cfg(debug_assertions)]
            let (_rx, child) = sidecar_command
                .args([
                    "dashboard",
                    "--dev",  // Use local dashboard in dev mode
                    "--api-port", &api_port.to_string(),
                    "--ui-port", &ui_port.to_string(),
                ])
                .spawn()
                .expect("Failed to spawn orkee CLI server");

            #[cfg(not(debug_assertions))]
            let (_rx, child) = sidecar_command
                .args([
                    "dashboard",
                    "--api-port", &api_port.to_string(),
                    "--ui-port", &ui_port.to_string(),
                ])
                .spawn()
                .expect("Failed to spawn orkee CLI server");

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
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_api_port, refresh_tray_menu])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Instead of closing, hide the window (minimize to tray)
                    // Users can quit from the tray menu
                    window.hide().unwrap();
                    api.prevent_close();
                }
                tauri::WindowEvent::Destroyed => {
                    // When the window is actually destroyed (app quitting)
                    if let Some(state) = window.app_handle().try_state::<CliServerState>() {
                        let api_port = state.api_port;

                        // Cleanup dev servers asynchronously with timeout
                        tauri::async_runtime::spawn(async move {
                            // Use timeout to prevent hanging on exit
                            match tokio::time::timeout(
                                Duration::from_secs(5),
                                cleanup_servers(api_port)
                            ).await {
                                Ok(Ok(_)) => println!("Cleanup completed successfully"),
                                Ok(Err(e)) => eprintln!("Cleanup error: {}", e),
                                Err(_) => eprintln!("Cleanup timed out after 5 seconds"),
                            }
                        });

                        // Kill CLI server process
                        if let Ok(mut process) = state.process.lock() {
                            if let Some(child) = process.take() {
                                kill_cli_process(child);
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Handle app-level events including unexpected exits
            match event {
                tauri::RunEvent::Exit => {
                    println!("App exit event received, performing cleanup...");

                    // Get the CLI server state and perform cleanup
                    if let Some(state) = app_handle.try_state::<CliServerState>() {
                        let api_port = state.api_port;

                        // Block on cleanup to ensure it completes before exit
                        // We use a short timeout to prevent hanging the exit
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        let _ = runtime.block_on(async {
                            tokio::time::timeout(
                                Duration::from_secs(3),
                                cleanup_servers(api_port)
                            ).await
                        });

                        // Kill CLI server process
                        if let Ok(mut process) = state.process.lock() {
                            if let Some(child) = process.take() {
                                kill_cli_process(child);
                            }
                        }
                    }
                }
                tauri::RunEvent::ExitRequested { api, .. } => {
                    // Don't prevent exit, but ensure cleanup happens
                    println!("Exit requested, cleanup will occur in Exit event");
                    // Don't call prevent_exit - let it proceed to Exit event
                }
                _ => {}
            }
        });
}
