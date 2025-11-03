// ABOUTME: Middleware for tracking API telemetry events
// ABOUTME: Automatically logs project CRUD operations, preview server actions, and other API calls

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{error, warn};

type HmacSha256 = Hmac<Sha256>;

/// Rate limiter for failed request telemetry to prevent unbounded database growth
struct FailureRateLimiter {
    // Map of event_name -> (count, window_start)
    failures: Mutex<HashMap<String, (u32, Instant)>>,
    max_failures_per_hour: u32,
    window_duration: Duration,
    cleanup_threshold: usize,
}

impl FailureRateLimiter {
    fn new() -> Self {
        Self {
            failures: Mutex::new(HashMap::new()),
            max_failures_per_hour: 10,
            window_duration: Duration::from_secs(3600), // 1 hour
            cleanup_threshold: 100, // Clean up when we exceed 100 entries
        }
    }

    async fn should_track_failure(&self, event_name: &str) -> bool {
        let mut failures = self.failures.lock().await;
        let now = Instant::now();

        // Periodic cleanup: remove expired entries when map grows large
        if failures.len() > self.cleanup_threshold {
            failures.retain(|_, (_, window_start)| {
                now.duration_since(*window_start) <= self.window_duration
            });
        }

        let entry = failures.entry(event_name.to_string()).or_insert((0, now));

        // Reset counter if window has expired
        if now.duration_since(entry.1) > self.window_duration {
            *entry = (0, now);
        }

        // Atomic check-and-increment to prevent race conditions
        if entry.0 < self.max_failures_per_hour {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

// Global rate limiter instance
static FAILURE_RATE_LIMITER: OnceLock<FailureRateLimiter> = OnceLock::new();

fn get_failure_rate_limiter() -> &'static FailureRateLimiter {
    FAILURE_RATE_LIMITER.get_or_init(FailureRateLimiter::new)
}

/// Middleware that tracks telemetry events for API calls
pub async fn track_api_calls(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Call the next handler
    let response = next.run(request).await;

    // Track both successful (2xx) and failed (4xx/5xx) operations
    let status = response.status();
    let is_success = status.is_success();
    let is_client_error = status.is_client_error();
    let is_server_error = status.is_server_error();

    // Determine if this is an operation we want to track
    let event_name_and_data = match (method.as_str(), path.as_str()) {
        // Project CRUD operations
        ("POST", "/api/projects") | ("POST", "/api/projects/") => {
            Some(("project_created", json!({"action": "create"})))
        }
        ("PUT", path) if path.starts_with("/api/projects/") && !is_task_endpoint(path) => {
            let project_id = extract_id_from_path(path, "/api/projects/");
            let project_id_hash = hash_id(&project_id);
            Some((
                "project_updated",
                json!({"action": "update", "project_id_hash": project_id_hash}),
            ))
        }
        ("DELETE", path) if path.starts_with("/api/projects/") && !is_task_endpoint(path) => {
            let project_id = extract_id_from_path(path, "/api/projects/");
            let project_id_hash = hash_id(&project_id);
            Some((
                "project_deleted",
                json!({"action": "delete", "project_id_hash": project_id_hash}),
            ))
        }

        // Preview server operations
        ("POST", path) if path.contains("/api/preview/servers/") && path.ends_with("/start") => {
            Some(("preview_server_started", json!({"action": "start"})))
        }
        ("POST", path) if path.contains("/api/preview/servers/") && path.ends_with("/stop") => {
            Some(("preview_server_stopped", json!({"action": "stop"})))
        }
        ("POST", path) if path.contains("/api/preview/servers/") && path.ends_with("/restart") => {
            Some(("preview_server_restarted", json!({"action": "restart"})))
        }
        ("POST", "/api/preview/servers/stop-all") | ("POST", "/api/preview/servers/stop-all/") => {
            Some(("preview_servers_stopped_all", json!({"action": "stop_all"})))
        }

        // Ideate operations - just track the main endpoints
        ("POST", path) if path.starts_with("/api/ideate/") => {
            let operation = path.strip_prefix("/api/ideate/").unwrap_or("unknown");
            Some(("ideate_operation", json!({"operation": operation})))
        }

        // Don't track everything else (health checks, list operations, etc.)
        _ => None,
    };

    // Track the event if we have a match
    if let Some((event_name, mut event_data)) = event_name_and_data {
        // Append "_failed" suffix for error responses and add status info
        let is_failure = is_client_error || is_server_error;
        let final_event_name = if is_failure {
            format!("{}_failed", event_name)
        } else {
            event_name.to_string()
        };

        // Check rate limiter for failed events
        if is_failure
            && !get_failure_rate_limiter()
                .should_track_failure(&final_event_name)
                .await
        {
            // Rate limit exceeded, skip tracking but log a warning (only once per event)
            warn!(
                "Telemetry rate limit exceeded for {}, skipping event to prevent database growth",
                final_event_name
            );
            return response;
        }

        // Add status code, success flag, and error category to event data
        if let serde_json::Value::Object(ref mut map) = event_data {
            map.insert("success".to_string(), json!(is_success));
            map.insert("status_code".to_string(), json!(status.as_u16()));

            // Add error_category for all non-success responses
            if !is_success {
                let error_category = if is_client_error {
                    "client_error"
                } else if is_server_error {
                    "server_error"
                } else if status.is_redirection() {
                    "redirect"
                } else {
                    "unknown"
                };
                map.insert("error_category".to_string(), json!(error_category));
            }
        }

        // Track asynchronously, don't block the response
        tokio::spawn(async move {
            if let Err(e) = track_telemetry_event(&final_event_name, event_data).await {
                // Just log the error, don't fail the request
                warn!(
                    "Failed to track telemetry event {}: {}",
                    final_event_name, e
                );
            }
        });
    }

    response
}

/// Helper function to extract ID from a URL path with validation
pub(crate) fn extract_id_from_path(path: &str, prefix: &str) -> String {
    let id = path
        .strip_prefix(prefix)
        .and_then(|s| s.split('/').next())
        .unwrap_or("unknown");

    // Validate ID: alphanumeric + hyphens + underscores only
    // This prevents path traversal attacks and malformed input
    if id != "unknown"
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        id.to_string()
    } else {
        "unknown".to_string()
    }
}

/// Secret salt for HMAC-based ID hashing (derived from application context)
fn get_hmac_secret() -> &'static [u8] {
    static SECRET: OnceLock<Vec<u8>> = OnceLock::new();
    SECRET.get_or_init(|| {
        // Use a combination of compile-time constants and runtime context
        // This provides defense-in-depth against rainbow table attacks
        let app_salt = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        let combined = format!("{}:{}:orkee-telemetry-salt", app_salt, version);
        combined.as_bytes().to_vec()
    })
}

/// Hash an ID using HMAC-SHA256 to protect sensitive information in telemetry
/// Uses application-specific secret to prevent rainbow table attacks
pub(crate) fn hash_id(id: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(get_hmac_secret())
        .expect("HMAC can take key of any size");
    mac.update(id.as_bytes());
    let result = mac.finalize();
    format!("{:x}", result.into_bytes())
}

/// Check if a path is a task-related endpoint (not a project endpoint)
/// Returns true if the path matches /api/projects/{id}/tasks*
pub(crate) fn is_task_endpoint(path: &str) -> bool {
    // Match pattern: /api/projects/{something}/tasks (with optional trailing segments)
    // This avoids false positives from project names containing "tasks"
    if let Some(after_projects) = path.strip_prefix("/api/projects/") {
        // Split on / and filter empty segments to handle double slashes
        let segments: Vec<&str> = after_projects
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        segments.len() >= 2 && segments[1] == "tasks"
    } else {
        false
    }
}

/// Track a telemetry event to the database
async fn track_telemetry_event(
    event_name: &str,
    event_data: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get the telemetry database pool
    let pool = match crate::telemetry::get_database_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            // Telemetry is optional - if it fails, just return OK
            error!("Failed to get telemetry database pool: {}", e);
            return Ok(());
        }
    };

    // Convert the JSON value to a HashMap
    let mut properties = HashMap::new();
    if let serde_json::Value::Object(map) = event_data {
        for (key, value) in map {
            properties.insert(key, value);
        }
    }

    // Track the event
    crate::telemetry::events::track_event(&pool, event_name, Some(properties), None).await?;

    Ok(())
}
