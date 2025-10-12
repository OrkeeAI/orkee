use tauri::{
    App, AppHandle, Manager, Wry,
    menu::{MenuBuilder, MenuItemBuilder, Menu, SubmenuBuilder},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use serde::{Deserialize, Serialize};

// Timeout constants for HTTP operations
const HTTP_REQUEST_TIMEOUT_SECS: u64 = 5;
const HTTP_CONNECT_TIMEOUT_SECS: u64 = 2;

// Polling and debouncing constants
const SERVER_POLLING_INTERVAL_SECS: u64 = 5;
const MENU_REBUILD_DEBOUNCE_SECS: u64 = 2;

// Server restart polling constants
const SERVER_RESTART_MAX_WAIT_SECS: u64 = 10;
const SERVER_RESTART_POLL_INTERVAL_MS: u64 = 100;

#[derive(Clone)]
pub struct TrayManager {
    pub app_handle: AppHandle,
    pub api_port: u16,
    tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    current_menu: Arc<Mutex<Option<Menu<Wry>>>>,
    shutdown_signal: Arc<AtomicBool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ServerInfo {
    id: String,
    project_id: String,
    project_name: Option<String>,
    port: u16,
    url: String,
    status: String,
    framework_name: Option<String>,
    started_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct ServersResponse {
    servers: Vec<ServerInfo>,
}

impl TrayManager {
    pub fn new(app_handle: AppHandle, api_port: u16) -> Self {
        Self {
            app_handle,
            api_port,
            tray_icon: Arc::new(Mutex::new(None)),
            current_menu: Arc::new(Mutex::new(None)),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create an HTTP client with configured timeouts to prevent hangs
    ///
    /// Returns an error if the client cannot be created with the specified configuration.
    /// This should never fail in practice unless there are system-level issues.
    fn create_http_client() -> Result<reqwest::Client, reqwest::Error> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(HTTP_REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(HTTP_CONNECT_TIMEOUT_SECS))
            .build()
    }

    pub fn init(&mut self, app: &App) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting tray initialization...");

        // Build initial menu
        let menu = Self::build_menu(&app.handle(), vec![])?;

        // Store the menu
        match self.current_menu.lock() {
            Ok(mut current_menu) => *current_menu = Some(menu.clone()),
            Err(e) => {
                eprintln!("Failed to lock current_menu during init: {}", e);
                return Err(format!("Mutex lock failed: {}", e).into());
            }
        }

        println!("Menu built successfully");

        // Load the template icon (white frame with transparent cutout for menu bar)
        // macOS will automatically adapt the color based on light/dark mode
        let icon_bytes = include_bytes!("../icons/icon-template.png");
        let icon = Image::from_bytes(icon_bytes)?;

        println!("Icon loaded successfully");

        // Build the tray icon
        let api_port = self.api_port;

        let tray = TrayIconBuilder::new()
            .icon(icon)
            .icon_as_template(true) // Enable macOS template mode for automatic color adaptation
            .menu(&menu)
            .tooltip("Orkee - Development Server Manager")
            .show_menu_on_left_click(true)
            .on_menu_event(move |app, event| {
                Self::handle_menu_event(app, event, api_port);
            })
            .build(app)?;

        // Store the tray icon
        match self.tray_icon.lock() {
            Ok(mut tray_icon) => *tray_icon = Some(tray),
            Err(e) => {
                eprintln!("Failed to lock tray_icon during init: {}", e);
                return Err(format!("Mutex lock failed: {}", e).into());
            }
        }

        println!("Tray icon initialized and stored successfully");

        // Start polling for server updates
        self.start_server_polling();

        Ok(())
    }

    fn build_menu(app_handle: &AppHandle, servers: Vec<ServerInfo>) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
        println!("Building menu with {} servers", servers.len());
        for (i, server) in servers.iter().enumerate() {
            println!("  Server {}: id={}, project_id={}, port={}", i, server.id, server.project_id, server.port);
        }

        let mut menu_builder = MenuBuilder::new(app_handle);

        // Dashboard item
        let show_item = MenuItemBuilder::with_id("show", "Show Orkee Dashboard").build(app_handle)?;
        menu_builder = menu_builder.item(&show_item);
        menu_builder = menu_builder.separator();

        // Servers section
        if servers.is_empty() {
            println!("WARNING: Servers list is empty in build_menu!");
            let no_servers = MenuItemBuilder::with_id("no_servers", "No servers running")
                .enabled(false)
                .build(app_handle)?;
            menu_builder = menu_builder.item(&no_servers);
        } else {
            let servers_label = MenuItemBuilder::with_id("servers_label", "Dev Servers")
                .enabled(false)
                .build(app_handle)?;
            menu_builder = menu_builder.item(&servers_label);

            for server in servers {
                let server_name = server.project_name.as_deref().unwrap_or(&server.project_id);

                // Build submenu for this server
                let mut submenu_builder = SubmenuBuilder::new(
                    app_handle,
                    format!("{} - Port {}", server_name, server.port)
                );

                // Open in browser
                let open_item = MenuItemBuilder::with_id(
                    format!("open_{}", server.id),
                    "Open in Browser"
                ).build(app_handle)?;
                submenu_builder = submenu_builder.item(&open_item);

                // Copy URL
                let copy_item = MenuItemBuilder::with_id(
                    format!("copy_{}", server.id),
                    format!("Copy URL ({})", server.url)
                ).build(app_handle)?;
                submenu_builder = submenu_builder.item(&copy_item);

                submenu_builder = submenu_builder.separator();

                // Restart server
                let restart_item = MenuItemBuilder::with_id(
                    format!("restart_{}", server.project_id),
                    "Restart Server"
                ).build(app_handle)?;
                submenu_builder = submenu_builder.item(&restart_item);

                // Stop server
                let stop_item = MenuItemBuilder::with_id(
                    format!("stop_{}", server.project_id),
                    "Stop Server"
                ).build(app_handle)?;
                submenu_builder = submenu_builder.item(&stop_item);

                let submenu = submenu_builder.build()?;
                menu_builder = menu_builder.item(&submenu);
            }
        }

        // Refresh item
        let refresh_item = MenuItemBuilder::with_id("refresh", "Refresh").build(app_handle)?;
        menu_builder = menu_builder.item(&refresh_item);

        menu_builder = menu_builder.separator();

        // Quit item
        let quit_item = MenuItemBuilder::with_id("quit", "Quit Orkee").build(app_handle)?;
        menu_builder = menu_builder.item(&quit_item);

        Ok(menu_builder.build()?)
    }

    fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent, api_port: u16) {
        let event_id = event.id.as_ref();
        println!("Menu event received: {}", event_id);

        match event_id {
            "quit" => {
                println!("Quitting application");
                app.exit(0);
            }
            "show" => {
                println!("Showing main window");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "refresh" => {
                println!("Refreshing server list");
                // Refresh will happen automatically via polling
            }
            id if id.starts_with("open_") => {
                if let Some(server_id) = id.strip_prefix("open_") {
                    Self::open_server_in_browser(api_port, server_id.to_string());
                } else {
                    eprintln!("Invalid menu event ID format: {}", id);
                }
            }
            id if id.starts_with("copy_") => {
                if let Some(server_id) = id.strip_prefix("copy_") {
                    Self::copy_server_url(app.clone(), api_port, server_id.to_string());
                } else {
                    eprintln!("Invalid menu event ID format: {}", id);
                }
            }
            id if id.starts_with("restart_") => {
                if let Some(project_id) = id.strip_prefix("restart_") {
                    Self::restart_server(api_port, project_id.to_string());
                } else {
                    eprintln!("Invalid menu event ID format: {}", id);
                }
            }
            id if id.starts_with("stop_") => {
                if let Some(project_id) = id.strip_prefix("stop_") {
                    Self::stop_server(api_port, project_id.to_string());
                } else {
                    eprintln!("Invalid menu event ID format: {}", id);
                }
            }
            _ => {}
        }
    }

    fn open_server_in_browser(api_port: u16, server_id: String) {
        tauri::async_runtime::spawn(async move {
            match Self::fetch_servers(api_port).await {
                Ok(servers) => {
                    if let Some(server) = servers.iter().find(|s| s.id == server_id) {
                        let _ = open::that(&server.url);
                    }
                }
                Err(e) => eprintln!("Failed to fetch servers: {}", e),
            }
        });
    }

    fn copy_server_url(app: AppHandle, api_port: u16, server_id: String) {
        tauri::async_runtime::spawn(async move {
            match Self::fetch_servers(api_port).await {
                Ok(servers) => {
                    if let Some(server) = servers.iter().find(|s| s.id == server_id) {
                        // Use the clipboard plugin to copy the URL
                        use tauri_plugin_clipboard_manager::ClipboardExt;
                        match app.clipboard().write_text(&server.url) {
                            Ok(_) => println!("Copied URL to clipboard: {}", server.url),
                            Err(e) => eprintln!("Failed to copy URL to clipboard: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("Failed to fetch servers: {}", e),
            }
        });
    }

    fn stop_server(api_port: u16, project_id: String) {
        tauri::async_runtime::spawn(async move {
            let client = match Self::create_http_client() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to create HTTP client for stopping server: {}", e);
                    return;
                }
            };
            let url = format!("http://localhost:{}/api/preview/servers/{}/stop", api_port, project_id);
            match client.post(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Successfully stopped server: {}", project_id);
                    } else {
                        eprintln!("Failed to stop server: HTTP {}", response.status());
                    }
                }
                Err(e) => eprintln!("Failed to stop server: {}", e),
            }
        });
    }

    fn restart_server(api_port: u16, project_id: String) {
        tauri::async_runtime::spawn(async move {
            let client = match Self::create_http_client() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to create HTTP client for restarting server: {}", e);
                    return;
                }
            };

            // Step 1: Stop the server
            let stop_url = format!("http://localhost:{}/api/preview/servers/{}/stop", api_port, project_id);
            match client.post(&stop_url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        eprintln!("Failed to stop server for restart: HTTP {}", response.status());
                        return;
                    }
                    println!("Successfully stopped server: {}", project_id);
                }
                Err(e) => {
                    eprintln!("Failed to stop server: {}", e);
                    return;
                }
            }

            // Step 2: Poll and verify server is actually stopped
            let status_url = format!("http://localhost:{}/api/preview/servers/{}/status", api_port, project_id);
            let max_attempts = (SERVER_RESTART_MAX_WAIT_SECS * 1000) / SERVER_RESTART_POLL_INTERVAL_MS;

            let mut stopped = false;
            for attempt in 0..max_attempts {
                tokio::time::sleep(Duration::from_millis(SERVER_RESTART_POLL_INTERVAL_MS)).await;

                // Check if server is no longer running
                match client.get(&status_url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            // Server still exists, check its status
                            if let Ok(status_json) = response.json::<serde_json::Value>().await {
                                if let Some(data) = status_json.get("data") {
                                    if data.get("instance").is_none() {
                                        // Server is stopped
                                        stopped = true;
                                        println!("Server confirmed stopped after {}ms", attempt * SERVER_RESTART_POLL_INTERVAL_MS);
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Server not found (404 or similar) - it's stopped
                            stopped = true;
                            println!("Server confirmed stopped (no longer exists) after {}ms", attempt * SERVER_RESTART_POLL_INTERVAL_MS);
                            break;
                        }
                    }
                    Err(_) => {
                        // API error might mean server is down, consider it stopped
                        stopped = true;
                        println!("Server appears stopped (API unreachable) after {}ms", attempt * SERVER_RESTART_POLL_INTERVAL_MS);
                        break;
                    }
                }
            }

            if !stopped {
                eprintln!("Timeout waiting for server to stop after {} seconds", SERVER_RESTART_MAX_WAIT_SECS);
                return;
            }

            // Step 3: Start the server with retry logic for port availability
            // OS-level port cleanup can take time after process termination
            // Instead of a fixed delay, we retry with exponential backoff if port isn't ready
            let start_url = format!("http://localhost:{}/api/preview/servers/{}/start", api_port, project_id);
            let max_start_attempts = 5;
            let mut start_delay_ms = SERVER_RESTART_POLL_INTERVAL_MS;

            for attempt in 0..max_start_attempts {
                if attempt > 0 {
                    // Wait with exponential backoff before retrying
                    tokio::time::sleep(Duration::from_millis(start_delay_ms)).await;
                    start_delay_ms = (start_delay_ms * 2).min(2000); // Cap at 2 seconds
                }

                match client.post(&start_url).send().await {
                    Ok(start_response) => {
                        if start_response.status().is_success() {
                            println!("Successfully restarted server: {} (attempt {})", project_id, attempt + 1);
                            return;
                        } else if start_response.status().as_u16() == 409 {
                            // 409 Conflict typically means port is still in use
                            println!("Port not yet available for server: {} (attempt {})", project_id, attempt + 1);
                            continue;
                        } else {
                            eprintln!("Failed to start server: HTTP {} (attempt {})", start_response.status(), attempt + 1);
                            if attempt == max_start_attempts - 1 {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to start server: {} (attempt {})", e, attempt + 1);
                        if attempt == max_start_attempts - 1 {
                            return;
                        }
                    }
                }
            }

            eprintln!("Failed to restart server after {} attempts", max_start_attempts);
        });
    }

    async fn fetch_servers(api_port: u16) -> Result<Vec<ServerInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let client = Self::create_http_client()?;
        let url = format!("http://localhost:{}/api/preview/servers", api_port);
        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<ServersResponse> = response.json().await?;
            if let Some(data) = api_response.data {
                let mut servers = data.servers;

                // Enrich servers with project names from projects API
                for server in &mut servers {
                    if server.project_name.is_none() {
                        if let Ok(project_name) = Self::fetch_project_name(api_port, &server.project_id).await {
                            server.project_name = Some(project_name);
                        }
                    }
                }

                Ok(servers)
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    async fn fetch_project_name(api_port: u16, project_id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = Self::create_http_client()?;
        let url = format!("http://localhost:{}/api/projects/{}", api_port, project_id);
        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<serde_json::Value> = response.json().await?;
            if let Some(data) = api_response.data {
                if let Some(name) = data.get("name").and_then(|n| n.as_str()) {
                    return Ok(name.to_string());
                }
            }
        }

        // Fallback to project_id if we can't fetch the name
        Ok(project_id.to_string())
    }

    /// Stop the server polling loop
    pub fn stop_polling(&self) {
        self.shutdown_signal.store(true, Ordering::Relaxed);
    }

    pub fn start_server_polling(&self) {
        let api_port = self.api_port;
        let app_handle = self.app_handle.clone();
        let tray_icon = self.tray_icon.clone();
        let shutdown_signal = self.shutdown_signal.clone();

        tauri::async_runtime::spawn(async move {
            let mut last_servers: Vec<ServerInfo> = vec![];
            let mut last_rebuild_time = std::time::Instant::now();
            let min_rebuild_interval = Duration::from_secs(MENU_REBUILD_DEBOUNCE_SECS);

            loop {
                // Check for shutdown signal
                if shutdown_signal.load(Ordering::Relaxed) {
                    println!("Tray polling loop received shutdown signal, exiting");
                    break;
                }

                // Poll servers every 5 seconds to reduce resource usage
                // Servers can be manually refreshed via the tray menu
                tokio::time::sleep(Duration::from_secs(SERVER_POLLING_INTERVAL_SECS)).await;

                // Check again after sleep in case shutdown was signaled during sleep
                if shutdown_signal.load(Ordering::Relaxed) {
                    println!("Tray polling loop received shutdown signal after sleep, exiting");
                    break;
                }

                // Try to fetch servers from API
                match Self::fetch_servers(api_port).await {
                    Ok(servers) => {
                        // Check if servers have changed and enough time has passed since last rebuild
                        // This debouncing prevents rapid menu rebuilds during server state transitions
                        if !servers_equal(&servers, &last_servers) {
                            let now = std::time::Instant::now();
                            if now.duration_since(last_rebuild_time) >= min_rebuild_interval {
                                println!("Server list changed, updating menu...");
                                println!("Found {} servers", servers.len());
                                last_rebuild_time = now;

                            // Rebuild the menu with new server information
                            match Self::build_menu(&app_handle, servers.clone()) {
                                Ok(new_menu) => {
                                    // Update the tray icon's menu
                                    match tray_icon.lock() {
                                        Ok(tray_guard) => {
                                            if let Some(tray) = tray_guard.as_ref() {
                                                if let Err(e) = tray.set_menu(Some(new_menu)) {
                                                    eprintln!("Failed to update tray menu: {}", e);
                                                } else {
                                                    println!("Tray menu updated successfully");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to lock tray_icon during polling: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to build menu: {}", e);
                                }
                            }

                            last_servers = servers;
                        }
                    }
                }
                Err(e) => {
                    // API might not be available yet
                    eprintln!("Failed to fetch servers: {}", e);
                }
                }
            }
        });
    }
}

fn servers_equal(a: &[ServerInfo], b: &[ServerInfo]) -> bool {
    use std::collections::HashSet;

    if a.len() != b.len() {
        return false;
    }

    // Create sets of (id, status, port) tuples for comparison
    // This is order-independent
    let set_a: HashSet<(&str, &str, u16)> = a
        .iter()
        .map(|s| (s.id.as_str(), s.status.as_str(), s.port))
        .collect();

    let set_b: HashSet<(&str, &str, u16)> = b
        .iter()
        .map(|s| (s.id.as_str(), s.status.as_str(), s.port))
        .collect();

    set_a == set_b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_server(id: &str, project_id: &str, status: &str, port: u16) -> ServerInfo {
        ServerInfo {
            id: id.to_string(),
            project_id: project_id.to_string(),
            project_name: Some(format!("Project {}", project_id)),
            port,
            url: format!("http://localhost:{}", port),
            status: status.to_string(),
            framework_name: Some("test-framework".to_string()),
            started_at: Some("2024-01-01T00:00:00Z".to_string()),
        }
    }

    #[test]
    fn test_servers_equal_empty_lists() {
        let a: Vec<ServerInfo> = vec![];
        let b: Vec<ServerInfo> = vec![];
        assert!(servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_identical_single_server() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server1", "proj1", "running", 3000)];
        assert!(servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_different_order() {
        let a = vec![
            create_test_server("server1", "proj1", "running", 3000),
            create_test_server("server2", "proj2", "running", 3001),
        ];
        let b = vec![
            create_test_server("server2", "proj2", "running", 3001),
            create_test_server("server1", "proj1", "running", 3000),
        ];
        assert!(servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_different_lengths() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![
            create_test_server("server1", "proj1", "running", 3000),
            create_test_server("server2", "proj2", "running", 3001),
        ];
        assert!(!servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_different_status() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server1", "proj1", "stopped", 3000)];
        assert!(!servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_different_port() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server1", "proj1", "running", 3001)];
        assert!(!servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_different_id() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server2", "proj1", "running", 3000)];
        assert!(!servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_ignores_project_name() {
        let mut a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let mut b = vec![create_test_server("server1", "proj1", "running", 3000)];

        // Different project names should not affect equality
        a[0].project_name = Some("Name A".to_string());
        b[0].project_name = Some("Name B".to_string());

        assert!(servers_equal(&a, &b));
    }

    #[test]
    fn test_servers_equal_ignores_url() {
        let mut a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let mut b = vec![create_test_server("server1", "proj1", "running", 3000)];

        // Different URLs should not affect equality
        a[0].url = "http://localhost:3000".to_string();
        b[0].url = "http://127.0.0.1:3000".to_string();

        assert!(servers_equal(&a, &b));
    }

    #[test]
    fn test_server_info_deserialization() {
        let json = r#"{
            "id": "test-id",
            "project_id": "test-project",
            "project_name": "Test Project",
            "port": 3000,
            "url": "http://localhost:3000",
            "status": "running",
            "framework_name": "vite",
            "started_at": "2024-01-01T00:00:00Z"
        }"#;

        let server: ServerInfo = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(server.id, "test-id");
        assert_eq!(server.project_id, "test-project");
        assert_eq!(server.project_name, Some("Test Project".to_string()));
        assert_eq!(server.port, 3000);
        assert_eq!(server.url, "http://localhost:3000");
        assert_eq!(server.status, "running");
        assert_eq!(server.framework_name, Some("vite".to_string()));
    }

    #[test]
    fn test_server_info_deserialization_optional_fields() {
        let json = r#"{
            "id": "test-id",
            "project_id": "test-project",
            "port": 3000,
            "url": "http://localhost:3000",
            "status": "running"
        }"#;

        let server: ServerInfo = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(server.id, "test-id");
        assert_eq!(server.project_name, None);
        assert_eq!(server.framework_name, None);
        assert_eq!(server.started_at, None);
    }

    #[test]
    fn test_api_response_with_data() {
        let json = r#"{
            "data": {"servers": []}
        }"#;

        let response: ApiResponse<ServersResponse> =
            serde_json::from_str(json).expect("Failed to deserialize");
        assert!(response.data.is_some());
    }

    #[test]
    fn test_api_response_without_data() {
        let json = r#"{}"#;

        let response: ApiResponse<ServersResponse> =
            serde_json::from_str(json).expect("Failed to deserialize");
        assert!(response.data.is_none());
    }

    #[test]
    fn test_servers_response_deserialization() {
        let json = r#"{
            "servers": [
                {
                    "id": "server1",
                    "project_id": "proj1",
                    "port": 3000,
                    "url": "http://localhost:3000",
                    "status": "running"
                },
                {
                    "id": "server2",
                    "project_id": "proj2",
                    "port": 3001,
                    "url": "http://localhost:3001",
                    "status": "stopped"
                }
            ]
        }"#;

        let response: ServersResponse =
            serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.servers.len(), 2);
        assert_eq!(response.servers[0].id, "server1");
        assert_eq!(response.servers[1].id, "server2");
    }
}