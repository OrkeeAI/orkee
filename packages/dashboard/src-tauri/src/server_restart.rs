// ABOUTME: Server restart logic for Tauri tray menu
// ABOUTME: Handles stopping, verifying shutdown, and restarting development servers with retry logic

use crate::tray::validate_project_id;
use std::time::Duration;
use tracing::{debug, error, info};
use urlencoding::encode;

// Server restart polling constants
const SERVER_RESTART_MAX_WAIT_SECS: u64 = 10;
const SERVER_RESTART_POLL_INTERVAL_MS: u64 = 100;

/// Get the API host from environment variable or use default localhost.
fn get_api_host() -> String {
    std::env::var("ORKEE_API_HOST").unwrap_or_else(|_| "localhost".to_string())
}

/// Create an HTTP client with configured timeouts to prevent hangs.
fn create_http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(2))
        .build()
}

/// Restart a development server by stopping it, verifying shutdown, and starting it again.
///
/// This function performs a three-step restart process:
/// 1. Stop the server via API
/// 2. Poll to verify the server is actually stopped (with exponential backoff)
/// 3. Start the server with retry logic for port availability
///
/// # Arguments
///
/// * `api_port` - The API port to connect to
/// * `project_id` - The project ID of the server to restart
///
/// # Notes
///
/// This function spawns an async task and returns immediately. All error handling
/// is done within the spawned task via logging.
pub fn restart_server(api_port: u16, project_id: String) {
    tauri::async_runtime::spawn(async move {
        // Validate project_id before making API calls
        if let Err(e) = validate_project_id(&project_id) {
            error!("Refusing to restart server with invalid project ID: {}", e);
            return;
        }

        let client = match create_http_client() {
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
