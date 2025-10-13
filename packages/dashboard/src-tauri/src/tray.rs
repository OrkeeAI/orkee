// ABOUTME: System tray manager for Orkee desktop application
// ABOUTME: Provides menu bar integration with live server monitoring and control

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{
    image::Image,
    menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::{TrayIcon, TrayIconBuilder},
    App, AppHandle, Manager, Wry,
};
use tracing::{debug, error, info, warn};
use urlencoding::encode;

// Timeout constants for HTTP operations
const HTTP_REQUEST_TIMEOUT_SECS: u64 = 5;
const HTTP_CONNECT_TIMEOUT_SECS: u64 = 2;

// Polling and debouncing constants
// Default polling interval (can be overridden via ORKEE_TRAY_POLL_INTERVAL_SECS env var)
const DEFAULT_SERVER_POLLING_INTERVAL_SECS: u64 = 5;
// Fast polling when servers are changing (starting/stopping)
const FAST_POLLING_INTERVAL_SECS: u64 = 1;
// Slow polling when servers are stable
const SLOW_POLLING_INTERVAL_SECS: u64 = 10;
// Number of consecutive stable polls before switching to slow mode
const STABLE_THRESHOLD: u32 = 3;
const MENU_REBUILD_DEBOUNCE_SECS: u64 = 2;

// Circuit breaker constants
const MAX_CONSECUTIVE_FAILURES: u32 = 5;
const CIRCUIT_BREAKER_RESET_SECS: u64 = 30;

// Server restart polling constants
const SERVER_RESTART_MAX_WAIT_SECS: u64 = 10;
const SERVER_RESTART_POLL_INTERVAL_MS: u64 = 100;

// API Host Configuration
// NOTE: All API calls use localhost by default because the Tauri desktop app launches
// and manages its own local Orkee CLI server process. This is not a remote
// API - it's a sidecar process running on 127.0.0.1. Using localhost is
// intentional and correct for this architecture.
// Can be overridden via ORKEE_API_HOST environment variable for extensibility.

/// Validate that an API host is safe to connect to.
///
/// By default, only localhost addresses are allowed (localhost, 127.0.0.1, ::1).
/// Remote hosts can be enabled by setting ORKEE_ALLOW_REMOTE_API=true environment variable.
/// This prevents accidental exposure of the API to network access.
fn validate_api_host(host: &str) -> Result<(), String> {
    // Check for empty host
    if host.is_empty() {
        return Err("API host cannot be empty".to_string());
    }

    // Check for suspicious hosts that could be used in SSRF attacks
    if host == "0.0.0.0" || host == "[::]" {
        return Err(format!(
            "Suspicious API host '{}' is not allowed. Use 'localhost' or '127.0.0.1' instead.",
            host
        ));
    }

    // Allow localhost by name
    if host == "localhost" {
        return Ok(());
    }

    // Validate as IP address and check if it's loopback
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if ip.is_loopback() {
            return Ok(());
        }
    }

    // Check if remote API access is explicitly enabled
    if std::env::var("ORKEE_ALLOW_REMOTE_API")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
    {
        warn!(
            "Remote API access is enabled for host: {}",
            host
        );
        warn!("This bypasses localhost-only security restrictions.");
        warn!("Ensure this is intentional and the host is trusted.");
        return Ok(());
    }

    // Reject non-localhost hosts by default
    Err(format!(
        "API host '{}' is not a localhost address and remote access is not enabled.\n\
        \n\
        For security, Orkee only connects to localhost by default.\n\
        \n\
        To connect to a remote API (not recommended), set:\n\
        export ORKEE_ALLOW_REMOTE_API=true\n\
        \n\
        Allowed localhost addresses:\n\
        - localhost\n\
        - 127.0.0.1\n\
        - ::1\n\
        - 127.x.x.x",
        host
    ))
}

fn get_api_host() -> String {
    let host = std::env::var("ORKEE_API_HOST").unwrap_or_else(|_| "localhost".to_string());

    // Validate the host before using it
    if let Err(e) = validate_api_host(&host) {
        error!("Invalid API host configuration: {}", e);
        warn!("Falling back to localhost for security.");
        return "localhost".to_string();
    }

    host
}

/// Sanitize text for safe display in system menu.
///
/// Filters out potentially malicious characters and limits length to prevent
/// menu injection attacks. Only allows alphanumeric characters, whitespace,
/// and common safe punctuation.
///
/// # Arguments
///
/// * `text` - The text to sanitize
///
/// # Returns
///
/// Returns a sanitized string safe for menu display (max 100 characters).
fn sanitize_menu_text(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.".contains(*c))
        .take(100)
        .collect()
}

#[derive(Clone)]
pub struct TrayManager {
    pub app_handle: AppHandle,
    pub api_port: u16,
    tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    shutdown_signal: Arc<AtomicBool>,
    http_client: Arc<reqwest::Client>,
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
        // Create HTTP client once with connection pooling enabled by default
        let http_client = Self::create_http_client().unwrap_or_else(|e| {
            error!("Failed to create HTTP client: {}. Using default client.", e);
            reqwest::Client::new()
        });

        Self {
            app_handle,
            api_port,
            tray_icon: Arc::new(Mutex::new(None)),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            http_client: Arc::new(http_client),
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
        info!("Starting tray initialization...");

        // Build initial menu
        let menu = Self::build_menu(&app.handle(), vec![])?;

        info!("Menu built successfully");

        // Load the template icon (white frame with transparent cutout for menu bar)
        // macOS will automatically adapt the color based on light/dark mode
        let icon_bytes = include_bytes!("../icons/icon-template.png");
        let icon = Image::from_bytes(icon_bytes)?;

        info!("Icon loaded successfully");

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
                error!("Failed to lock tray_icon during init: {}", e);
                return Err(format!("Mutex lock failed: {}", e).into());
            }
        }

        info!("Tray icon initialized and stored successfully");

        // Start polling for server updates
        self.start_server_polling();

        Ok(())
    }

    fn build_menu(
        app_handle: &AppHandle,
        servers: Vec<ServerInfo>,
    ) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
        debug!("Building menu with {} servers", servers.len());
        for (i, server) in servers.iter().enumerate() {
            debug!(
                "  Server {}: id={}, project_id={}, port={}",
                i, server.id, server.project_id, server.port
            );
        }

        let mut menu_builder = MenuBuilder::new(app_handle);

        // Dashboard item
        let show_item =
            MenuItemBuilder::with_id("show", "Show Orkee Dashboard").build(app_handle)?;
        menu_builder = menu_builder.item(&show_item);
        menu_builder = menu_builder.separator();

        // Servers section
        if servers.is_empty() {
            warn!("Servers list is empty in build_menu!");
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
                let sanitized_name = sanitize_menu_text(server_name);

                // Build submenu for this server
                let mut submenu_builder = SubmenuBuilder::new(
                    app_handle,
                    format!("{} - Port {}", sanitized_name, server.port),
                );

                // Open in browser - uses server.id for direct server reference
                let open_item =
                    MenuItemBuilder::with_id(format!("open_{}", server.id), "Open in Browser")
                        .build(app_handle)?;
                submenu_builder = submenu_builder.item(&open_item);

                // Copy URL - uses server.id for direct server reference
                let copy_item = MenuItemBuilder::with_id(
                    format!("copy_{}", server.id),
                    format!("Copy URL ({})", server.url),
                )
                .build(app_handle)?;
                submenu_builder = submenu_builder.item(&copy_item);

                submenu_builder = submenu_builder.separator();

                // Restart server - uses project_id for project-level API operations
                let restart_item = MenuItemBuilder::with_id(
                    format!("restart_{}", server.project_id),
                    "Restart Server",
                )
                .build(app_handle)?;
                submenu_builder = submenu_builder.item(&restart_item);

                // Stop server - uses project_id for project-level API operations
                let stop_item =
                    MenuItemBuilder::with_id(format!("stop_{}", server.project_id), "Stop Server")
                        .build(app_handle)?;
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
        debug!("Menu event received: {}", event_id);

        match event_id {
            "quit" => {
                info!("Quitting application");
                app.exit(0);
            }
            "show" => {
                info!("Showing main window");
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(e) = window.show() {
                        error!("Failed to show window: {}", e);
                    }
                    if let Err(e) = window.set_focus() {
                        error!("Failed to set window focus: {}", e);
                    }
                }
            }
            "refresh" => {
                info!("Refreshing server list");
                // Refresh will happen automatically via polling
            }
            id if id.starts_with("open_") => {
                if let Some(server_id) = id.strip_prefix("open_") {
                    Self::open_server_in_browser(api_port, server_id.to_string());
                } else {
                    error!("Invalid menu event ID format: {}", id);
                }
            }
            id if id.starts_with("copy_") => {
                if let Some(server_id) = id.strip_prefix("copy_") {
                    Self::copy_server_url(app.clone(), api_port, server_id.to_string());
                } else {
                    error!("Invalid menu event ID format: {}", id);
                }
            }
            id if id.starts_with("restart_") => {
                if let Some(project_id) = id.strip_prefix("restart_") {
                    Self::restart_server(api_port, project_id.to_string());
                } else {
                    error!("Invalid menu event ID format: {}", id);
                }
            }
            id if id.starts_with("stop_") => {
                if let Some(project_id) = id.strip_prefix("stop_") {
                    Self::stop_server(api_port, project_id.to_string());
                } else {
                    error!("Invalid menu event ID format: {}", id);
                }
            }
            _ => {}
        }
    }

    fn open_server_in_browser(api_port: u16, server_id: String) {
        tauri::async_runtime::spawn(async move {
            match Self::fetch_servers_static(api_port).await {
                Ok(servers) => {
                    if let Some(server) = servers.iter().find(|s| s.id == server_id) {
                        if let Err(e) = open::that(&server.url) {
                            error!("Failed to open browser for {}: {}", server.url, e);
                        }
                    } else {
                        warn!("Server {} no longer exists", server_id);
                    }
                }
                Err(e) => error!("Failed to fetch servers: {}", e),
            }
        });
    }

    fn copy_server_url(app: AppHandle, api_port: u16, server_id: String) {
        tauri::async_runtime::spawn(async move {
            match Self::fetch_servers_static(api_port).await {
                Ok(servers) => {
                    if let Some(server) = servers.iter().find(|s| s.id == server_id) {
                        // Use the clipboard plugin to copy the URL
                        use tauri_plugin_clipboard_manager::ClipboardExt;
                        match app.clipboard().write_text(&server.url) {
                            Ok(_) => info!("Copied URL to clipboard: {}", server.url),
                            Err(e) => error!("Failed to copy URL to clipboard: {}", e),
                        }
                    } else {
                        warn!("Server {} no longer exists", server_id);
                    }
                }
                Err(e) => error!("Failed to fetch servers: {}", e),
            }
        });
    }

    fn stop_server(api_port: u16, project_id: String) {
        tauri::async_runtime::spawn(async move {
            let client = match Self::create_http_client() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create HTTP client for stopping server: {}", e);
                    return;
                }
            };
            let url = format!(
                "http://{}:{}/api/preview/servers/{}/stop",
                get_api_host(),
                api_port,
                encode(&project_id)
            );
            match client.post(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Successfully stopped server: {}", project_id);
                    } else {
                        error!("Failed to stop server: HTTP {}", response.status());
                    }
                }
                Err(e) => error!("Failed to stop server: {}", e),
            }
        });
    }

    fn restart_server(api_port: u16, project_id: String) {
        tauri::async_runtime::spawn(async move {
            let client = match Self::create_http_client() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create HTTP client for restarting server: {}", e);
                    return;
                }
            };

            // Step 1: Stop the server
            let stop_url = format!(
                "http://{}:{}/api/preview/servers/{}/stop",
                get_api_host(),
                api_port,
                encode(&project_id)
            );
            match client.post(&stop_url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        error!(
                            "Failed to stop server for restart: HTTP {}",
                            response.status()
                        );
                        return;
                    }
                    info!("Successfully stopped server: {}", project_id);
                }
                Err(e) => {
                    error!("Failed to stop server: {}", e);
                    return;
                }
            }

            // Step 2: Poll and verify server is actually stopped with exponential backoff
            let status_url = format!(
                "http://{}:{}/api/preview/servers/{}/status",
                get_api_host(),
                api_port,
                encode(&project_id)
            );
            let max_wait_ms = SERVER_RESTART_MAX_WAIT_SECS * 1000;

            let mut stopped = false;
            let mut wait_ms = SERVER_RESTART_POLL_INTERVAL_MS; // Start with 100ms
            let mut elapsed_ms = 0;
            let mut attempt = 0;

            while elapsed_ms < max_wait_ms {
                tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                elapsed_ms += wait_ms;
                attempt += 1;

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
                                        debug!(
                                            "Server confirmed stopped after {}ms (attempt {})",
                                            elapsed_ms, attempt
                                        );
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Server not found (404 or similar) - it's stopped
                            stopped = true;
                            debug!("Server confirmed stopped (no longer exists) after {}ms (attempt {})", elapsed_ms, attempt);
                            break;
                        }
                    }
                    Err(_) => {
                        // API error might mean server is down, consider it stopped
                        stopped = true;
                        debug!(
                            "Server appears stopped (API unreachable) after {}ms (attempt {})",
                            elapsed_ms, attempt
                        );
                        break;
                    }
                }

                // Exponential backoff: 100ms → 200ms → 400ms → 800ms → 1000ms (capped)
                wait_ms = (wait_ms * 2).min(1000);
            }

            if !stopped {
                error!(
                    "Timeout waiting for server to stop after {} seconds",
                    SERVER_RESTART_MAX_WAIT_SECS
                );
                return;
            }

            // Step 3: Start the server with retry logic for port availability
            // OS-level port cleanup can take time after process termination
            // Instead of a fixed delay, we retry with exponential backoff if port isn't ready
            let start_url = format!(
                "http://{}:{}/api/preview/servers/{}/start",
                get_api_host(),
                api_port,
                encode(&project_id)
            );
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
                            info!(
                                "Successfully restarted server: {} (attempt {})",
                                project_id,
                                attempt + 1
                            );
                            return;
                        } else if start_response.status().as_u16() == 409 {
                            // 409 Conflict typically means port is still in use
                            debug!(
                                "Port not yet available for server: {} (attempt {})",
                                project_id,
                                attempt + 1
                            );
                            continue;
                        } else {
                            error!(
                                "Failed to start server: HTTP {} (attempt {})",
                                start_response.status(),
                                attempt + 1
                            );
                            if attempt == max_start_attempts - 1 {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to start server: {} (attempt {})", e, attempt + 1);
                        if attempt == max_start_attempts - 1 {
                            return;
                        }
                    }
                }
            }

            error!(
                "Failed to restart server after {} attempts",
                max_start_attempts
            );
        });
    }

    async fn fetch_servers_static(
        api_port: u16,
    ) -> Result<Vec<ServerInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let client = Self::create_http_client()?;
        let url = format!("http://{}:{}/api/preview/servers", get_api_host(), api_port);
        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<ServersResponse> = response.json().await?;
            if let Some(data) = api_response.data {
                Ok(data.servers)
            } else {
                Err("API response missing data field".into())
            }
        } else {
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }

    async fn fetch_servers(
        &self,
    ) -> Result<Vec<ServerInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "http://{}:{}/api/preview/servers",
            get_api_host(),
            self.api_port
        );
        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<ServersResponse> = response.json().await?;
            if let Some(data) = api_response.data {
                // Project names are now included in the API response
                // No need for additional fetching - this eliminates the N+1 query problem
                Ok(data.servers)
            } else {
                // Return error instead of empty vec to prevent clearing menu on API issues
                Err("API response missing data field".into())
            }
        } else {
            // Return error instead of empty vec to prevent clearing menu on HTTP errors
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }

    /// Get the polling interval from environment variable or use default.
    ///
    /// The polling interval determines how often the tray menu checks for server updates.
    /// Can be configured via the ORKEE_TRAY_POLL_INTERVAL_SECS environment variable.
    ///
    /// # Returns
    ///
    /// Returns the polling interval in seconds (default: 5, min: 1, max: 60).
    fn get_polling_interval_secs() -> u64 {
        match std::env::var("ORKEE_TRAY_POLL_INTERVAL_SECS") {
            Ok(raw_value) => match raw_value.parse::<u64>() {
                Ok(parsed_value) => {
                    if (1..=60).contains(&parsed_value) {
                        parsed_value
                    } else {
                        warn!(
                            "ORKEE_TRAY_POLL_INTERVAL_SECS has invalid value '{}' (must be 1-60), using default: {}",
                            raw_value,
                            DEFAULT_SERVER_POLLING_INTERVAL_SECS
                        );
                        DEFAULT_SERVER_POLLING_INTERVAL_SECS
                    }
                }
                Err(_) => {
                    warn!(
                        "ORKEE_TRAY_POLL_INTERVAL_SECS has unparseable value '{}', using default: {}",
                        raw_value,
                        DEFAULT_SERVER_POLLING_INTERVAL_SECS
                    );
                    DEFAULT_SERVER_POLLING_INTERVAL_SECS
                }
            },
            Err(_) => DEFAULT_SERVER_POLLING_INTERVAL_SECS,
        }
    }

    /// Stop the server polling loop
    pub fn stop_polling(&self) {
        self.shutdown_signal.store(true, Ordering::Relaxed);
    }

    pub fn start_server_polling(&self) {
        let app_handle = self.app_handle.clone();
        let tray_icon = self.tray_icon.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let manager = self.clone();

        tauri::async_runtime::spawn(async move {
            let mut last_servers_hash: u64 = 0; // Cache hash of last server list
            let mut last_rebuild_time = std::time::Instant::now();
            let min_rebuild_interval = Duration::from_secs(MENU_REBUILD_DEBOUNCE_SECS);
            let base_poll_interval_secs = Self::get_polling_interval_secs();
            let mut consecutive_failures = 0;
            let mut circuit_breaker_open = false;
            let mut circuit_breaker_opened_at: Option<std::time::Instant> = None;

            // Adaptive polling state
            #[allow(unused_assignments)]
            let mut consecutive_stable_polls = 0;
            let mut current_poll_interval_secs =
                if base_poll_interval_secs == DEFAULT_SERVER_POLLING_INTERVAL_SECS {
                    FAST_POLLING_INTERVAL_SECS // Start with fast polling by default
                } else {
                    base_poll_interval_secs // Respect custom polling interval
                };

            info!(
                "Tray polling starting with adaptive interval: {} seconds",
                current_poll_interval_secs
            );

            loop {
                // Check for shutdown signal
                if shutdown_signal.load(Ordering::Relaxed) {
                    info!("Tray polling loop received shutdown signal, exiting");
                    break;
                }

                // Adaptive polling: fast when changing, slow when stable
                // Break sleep into 1-second chunks for responsive shutdown
                for _ in 0..current_poll_interval_secs {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    if shutdown_signal.load(Ordering::Relaxed) {
                        info!(
                            "Tray polling loop received shutdown signal during sleep, exiting"
                        );
                        return;
                    }
                }

                // Circuit breaker: check if we should attempt API call
                if circuit_breaker_open {
                    if let Some(opened_at) = circuit_breaker_opened_at {
                        let elapsed = std::time::Instant::now().duration_since(opened_at);
                        if elapsed >= Duration::from_secs(CIRCUIT_BREAKER_RESET_SECS) {
                            debug!(
                                "Circuit breaker: attempting to close after {} seconds",
                                elapsed.as_secs()
                            );
                            circuit_breaker_open = false;
                            circuit_breaker_opened_at = None;
                            consecutive_failures = 0;
                        } else {
                            // Skip API call while circuit breaker is open
                            continue;
                        }
                    }
                }

                // Try to fetch servers from API
                match manager.fetch_servers().await {
                    Ok(servers) => {
                        // Success - reset failure counter and close circuit breaker
                        consecutive_failures = 0;
                        if circuit_breaker_open {
                            debug!("Circuit breaker: closed after successful API call");
                            circuit_breaker_open = false;
                            circuit_breaker_opened_at = None;
                        }

                        // Compute hash of current server list and compare with cached hash
                        let current_hash = compute_servers_hash(&servers);
                        let servers_changed = current_hash != last_servers_hash;

                        if servers_changed {
                            // Servers changed - switch to fast polling
                            consecutive_stable_polls = 0;
                            if base_poll_interval_secs == DEFAULT_SERVER_POLLING_INTERVAL_SECS
                                && current_poll_interval_secs != FAST_POLLING_INTERVAL_SECS
                            {
                                current_poll_interval_secs = FAST_POLLING_INTERVAL_SECS;
                                debug!(
                                    "Servers changing - switching to fast polling ({} seconds)",
                                    current_poll_interval_secs
                                );
                            }

                            let now = std::time::Instant::now();
                            if now.duration_since(last_rebuild_time) >= min_rebuild_interval {
                                info!("Server list changed, updating menu...");
                                debug!("Found {} servers", servers.len());

                                match Self::build_menu(&app_handle, servers.clone()) {
                                    Ok(new_menu) => {
                                        match tray_icon.lock() {
                                            Ok(tray_guard) => {
                                                if let Some(tray) = tray_guard.as_ref() {
                                                    if let Err(e) = tray.set_menu(Some(new_menu)) {
                                                        error!(
                                                            "Failed to update tray menu: {}",
                                                            e
                                                        );
                                                    } else {
                                                        info!("Tray menu updated successfully");
                                                        last_servers_hash = current_hash; // Update cached hash
                                                        last_rebuild_time = now;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!(
                                                    "Failed to lock tray_icon during polling: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to build menu: {}", e);
                                    }
                                }
                            }
                            // If rebuild interval hasn't elapsed, skip rebuild but keep fast polling
                        } else {
                            // Servers haven't changed - increment stability counter
                            consecutive_stable_polls += 1;

                            // Switch to slow polling after threshold
                            if base_poll_interval_secs == DEFAULT_SERVER_POLLING_INTERVAL_SECS
                                && consecutive_stable_polls >= STABLE_THRESHOLD
                                && current_poll_interval_secs != SLOW_POLLING_INTERVAL_SECS
                            {
                                current_poll_interval_secs = SLOW_POLLING_INTERVAL_SECS;
                                debug!("Servers stable for {} polls - switching to slow polling ({} seconds)",
                                consecutive_stable_polls, current_poll_interval_secs);
                            }
                        }
                    }
                    Err(e) => {
                        // API failure - increment counter and potentially open circuit breaker
                        consecutive_failures += 1;
                        error!(
                            "Failed to fetch servers (attempt {}/{}): {}",
                            consecutive_failures, MAX_CONSECUTIVE_FAILURES, e
                        );

                        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES && !circuit_breaker_open
                        {
                            circuit_breaker_open = true;
                            circuit_breaker_opened_at = Some(std::time::Instant::now());
                            error!("Circuit breaker: opened after {} consecutive failures. Will retry in {} seconds.",
                            consecutive_failures, CIRCUIT_BREAKER_RESET_SECS);
                        }
                    }
                }
            }
        });
    }
}

/// Compute a hash of the server list for efficient comparison.
///
/// This function computes a stable hash based on server id, status, and port.
/// The hash is order-independent by sorting servers before hashing.
/// This avoids allocating HashSets on every poll.
fn compute_servers_hash(servers: &[ServerInfo]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Create a sorted vector of (id, status, port) tuples for stable hashing
    let mut tuples: Vec<(&str, &str, u16)> = servers
        .iter()
        .map(|s| (s.id.as_str(), s.status.as_str(), s.port))
        .collect();

    // Sort to ensure order-independence
    tuples.sort_unstable();

    // Hash the sorted tuples
    tuples.hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Global mutex to serialize environment variable tests
    static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

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
    fn test_servers_hash_empty_lists() {
        let a: Vec<ServerInfo> = vec![];
        let b: Vec<ServerInfo> = vec![];
        assert_eq!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_identical_single_server() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server1", "proj1", "running", 3000)];
        assert_eq!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_different_order() {
        let a = vec![
            create_test_server("server1", "proj1", "running", 3000),
            create_test_server("server2", "proj2", "running", 3001),
        ];
        let b = vec![
            create_test_server("server2", "proj2", "running", 3001),
            create_test_server("server1", "proj1", "running", 3000),
        ];
        // Hash should be the same regardless of order
        assert_eq!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_different_lengths() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![
            create_test_server("server1", "proj1", "running", 3000),
            create_test_server("server2", "proj2", "running", 3001),
        ];
        assert_ne!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_different_status() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server1", "proj1", "stopped", 3000)];
        assert_ne!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_different_port() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server1", "proj1", "running", 3001)];
        assert_ne!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_different_id() {
        let a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let b = vec![create_test_server("server2", "proj1", "running", 3000)];
        assert_ne!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_ignores_project_name() {
        let mut a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let mut b = vec![create_test_server("server1", "proj1", "running", 3000)];

        // Different project names should not affect hash
        a[0].project_name = Some("Name A".to_string());
        b[0].project_name = Some("Name B".to_string());

        assert_eq!(compute_servers_hash(&a), compute_servers_hash(&b));
    }

    #[test]
    fn test_servers_hash_ignores_url() {
        let mut a = vec![create_test_server("server1", "proj1", "running", 3000)];
        let mut b = vec![create_test_server("server1", "proj1", "running", 3000)];

        // Different URLs should not affect hash
        a[0].url = "http://localhost:3000".to_string();
        b[0].url = "http://127.0.0.1:3000".to_string();

        assert_eq!(compute_servers_hash(&a), compute_servers_hash(&b));
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

        let response: ServersResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.servers.len(), 2);
        assert_eq!(response.servers[0].id, "server1");
        assert_eq!(response.servers[1].id, "server2");
    }

    #[test]
    fn test_url_encoding_for_special_characters() {
        use urlencoding::encode;

        // Test cases that would cause URL injection without encoding
        let dangerous_ids = vec![
            "../../../etc/passwd",
            "project?param=value",
            "project#fragment",
            "project/../../admin",
            "project%2F..%2F..%2Fadmin",
        ];

        for project_id in dangerous_ids {
            let encoded = encode(project_id);
            let api_port = 4001;
            let api_host = get_api_host();
            let url = format!(
                "http://{}:{}/api/preview/servers/{}/stop",
                api_host, api_port, encoded
            );

            // Verify that special characters are properly encoded
            assert!(
                !url.contains("../"),
                "URL should not contain path traversal sequences"
            );
            assert!(
                !url.contains("?"),
                "URL should not contain unencoded query parameters"
            );
            assert!(
                !url.contains("#"),
                "URL should not contain unencoded fragments"
            );

            // Verify the encoded path segment is in the correct position
            assert!(url.starts_with(&format!(
                "http://{}:{}/api/preview/servers/",
                api_host, api_port
            )));
            assert!(url.ends_with("/stop"));
        }
    }

    #[test]
    fn test_validate_api_host_allows_localhost() {
        assert!(validate_api_host("localhost").is_ok());
    }

    #[test]
    fn test_validate_api_host_allows_127_0_0_1() {
        assert!(validate_api_host("127.0.0.1").is_ok());
    }

    #[test]
    fn test_validate_api_host_allows_ipv6_loopback() {
        assert!(validate_api_host("::1").is_ok());
    }

    #[test]
    fn test_validate_api_host_blocks_remote_hosts_by_default() {
        // Serialize environment variable tests to avoid race conditions
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        // Ensure env var is not set for this test
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");

        assert!(validate_api_host("example.com").is_err());
        assert!(validate_api_host("192.168.1.1").is_err());
        assert!(validate_api_host("10.0.0.1").is_err());
    }

    #[test]
    fn test_validate_api_host_remote_access_flag_truthy_values() {
        // Serialize environment variable tests to avoid race conditions
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        // Test that only "true" and "1" enable remote access

        // Test "true" (lowercase)
        std::env::set_var("ORKEE_ALLOW_REMOTE_API", "true");
        assert!(validate_api_host("example.com").is_ok());
        assert!(validate_api_host("192.168.1.1").is_ok());
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");

        // Test "TRUE" (uppercase)
        std::env::set_var("ORKEE_ALLOW_REMOTE_API", "TRUE");
        assert!(validate_api_host("example.com").is_ok());
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");

        // Test "1"
        std::env::set_var("ORKEE_ALLOW_REMOTE_API", "1");
        assert!(validate_api_host("example.com").is_ok());
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");
    }

    #[test]
    fn test_validate_api_host_remote_access_flag_falsy_values() {
        // Serialize environment variable tests to avoid race conditions
        let _guard = ENV_TEST_MUTEX.lock().unwrap();

        // Test that "false", "0", empty string, and other values do NOT enable remote access

        // Test "false" (lowercase)
        std::env::set_var("ORKEE_ALLOW_REMOTE_API", "false");
        assert!(validate_api_host("example.com").is_err());
        assert!(validate_api_host("192.168.1.1").is_err());
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");

        // Test "FALSE" (uppercase)
        std::env::set_var("ORKEE_ALLOW_REMOTE_API", "FALSE");
        assert!(validate_api_host("example.com").is_err());
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");

        // Test "0"
        std::env::set_var("ORKEE_ALLOW_REMOTE_API", "0");
        assert!(validate_api_host("example.com").is_err());
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");

        // Test empty string
        std::env::set_var("ORKEE_ALLOW_REMOTE_API", "");
        assert!(validate_api_host("example.com").is_err());
        std::env::remove_var("ORKEE_ALLOW_REMOTE_API");
    }

    #[test]
    fn test_validate_api_host_blocks_empty_string() {
        assert!(validate_api_host("").is_err());
    }

    #[test]
    fn test_validate_api_host_blocks_suspicious_hosts() {
        // Test potential SSRF attacks
        assert!(validate_api_host("0.0.0.0").is_err());
        assert!(validate_api_host("[::]").is_err());
    }
}
