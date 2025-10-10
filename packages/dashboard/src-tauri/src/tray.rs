use tauri::{
    App, AppHandle, Manager, Wry,
    menu::{MenuBuilder, MenuItemBuilder, Menu, SubmenuBuilder},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct TrayManager {
    pub app_handle: AppHandle,
    pub api_port: u16,
    tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    current_menu: Arc<Mutex<Option<Menu<Wry>>>>,
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
    success: bool,
    data: Option<T>,
    error: Option<String>,
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
        }
    }

    pub fn init(&mut self, app: &App) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting tray initialization...");

        // Build initial menu
        let menu = Self::build_menu(&app.handle(), vec![])?;

        // Store the menu
        *self.current_menu.lock().unwrap() = Some(menu.clone());

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
        *self.tray_icon.lock().unwrap() = Some(tray);

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
        let show_item = MenuItemBuilder::with_id("show", "Show Dashboard").build(app_handle)?;
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
                let server_id = id.strip_prefix("open_").unwrap().to_string();
                Self::open_server_in_browser(api_port, server_id);
            }
            id if id.starts_with("copy_") => {
                let server_id = id.strip_prefix("copy_").unwrap().to_string();
                Self::copy_server_url(app.clone(), api_port, server_id);
            }
            id if id.starts_with("restart_") => {
                let project_id = id.strip_prefix("restart_").unwrap().to_string();
                Self::restart_server(api_port, project_id);
            }
            id if id.starts_with("stop_") => {
                let project_id = id.strip_prefix("stop_").unwrap().to_string();
                Self::stop_server(api_port, project_id);
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
            let url = format!("http://localhost:{}/api/preview/servers/{}/stop", api_port, project_id);
            match reqwest::Client::new().post(&url).send().await {
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
            // First stop the server
            let stop_url = format!("http://localhost:{}/api/preview/servers/{}/stop", api_port, project_id);
            match reqwest::Client::new().post(&stop_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Successfully stopped server: {}", project_id);

                        // Wait a moment for the server to fully stop
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                        // Then start it again
                        let start_url = format!("http://localhost:{}/api/preview/servers/{}/start", api_port, project_id);
                        match reqwest::Client::new().post(&start_url).send().await {
                            Ok(start_response) => {
                                if start_response.status().is_success() {
                                    println!("Successfully restarted server: {}", project_id);
                                } else {
                                    eprintln!("Failed to restart server: HTTP {}", start_response.status());
                                }
                            }
                            Err(e) => eprintln!("Failed to start server: {}", e),
                        }
                    } else {
                        eprintln!("Failed to stop server for restart: HTTP {}", response.status());
                    }
                }
                Err(e) => eprintln!("Failed to restart server: {}", e),
            }
        });
    }

    async fn fetch_servers(api_port: u16) -> Result<Vec<ServerInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("http://localhost:{}/api/preview/servers", api_port);
        let response = reqwest::get(&url).await?;

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
        let url = format!("http://localhost:{}/api/projects/{}", api_port, project_id);
        let response = reqwest::get(&url).await?;

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

    pub fn start_server_polling(&self) {
        let api_port = self.api_port;
        let app_handle = self.app_handle.clone();
        let tray_icon = self.tray_icon.clone();

        tauri::async_runtime::spawn(async move {
            let mut last_servers: Vec<ServerInfo> = vec![];

            loop {
                // Poll servers every 3 seconds
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                // Try to fetch servers from API
                match Self::fetch_servers(api_port).await {
                    Ok(servers) => {
                        // Check if servers have changed
                        if !servers_equal(&servers, &last_servers) {
                            println!("Server list changed, updating menu...");
                            println!("Found {} servers", servers.len());

                            // Rebuild the menu with new server information
                            match Self::build_menu(&app_handle, servers.clone()) {
                                Ok(new_menu) => {
                                    // Update the tray icon's menu
                                    if let Some(tray) = tray_icon.lock().unwrap().as_ref() {
                                        if let Err(e) = tray.set_menu(Some(new_menu)) {
                                            eprintln!("Failed to update tray menu: {}", e);
                                        } else {
                                            println!("Tray menu updated successfully");
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
                    Err(e) => {
                        // API might not be available yet
                        eprintln!("Failed to fetch servers: {}", e);
                    }
                }
            }
        });
    }

    pub fn get_tray(&self) -> Option<TrayIcon> {
        self.tray_icon.lock().unwrap().clone()
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