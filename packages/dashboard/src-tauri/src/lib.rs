use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use std::sync::Mutex;

// Store the CLI server process handle and ports globally
struct CliServerState {
    process: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
    api_port: u16,
}

/// Tauri command to get the API port that the CLI server is running on
#[tauri::command]
fn get_api_port(state: tauri::State<CliServerState>) -> u16 {
    state.api_port
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
        .setup(|app| {
            // Find available port dynamically
            let api_port = find_available_port();
            let ui_port = 5173; // UI port is fixed in dev mode (Vite already started)

            println!("Using dynamic API port: {}", api_port);

            // Start the Orkee CLI server as a sidecar
            let shell = app.shell();

            // Get the sidecar command for the orkee binary
            let sidecar_command = shell.sidecar("orkee")
                .expect("Failed to create sidecar command");

            // Spawn the CLI server with dashboard command
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

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Kill the CLI server when the app closes
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.app_handle().try_state::<CliServerState>() {
                    if let Ok(mut process) = state.process.lock() {
                        if let Some(mut child) = process.take() {
                            println!("Stopping Orkee CLI server...");
                            let _ = child.kill();
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![get_api_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
