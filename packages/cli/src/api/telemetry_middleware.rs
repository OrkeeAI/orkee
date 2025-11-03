// ABOUTME: Middleware for tracking API telemetry events
// ABOUTME: Automatically logs project CRUD operations, preview server actions, and other API calls

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{error, warn};

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
        let final_event_name = if is_client_error || is_server_error {
            format!("{}_failed", event_name)
        } else {
            event_name.to_string()
        };

        // Add status code and success flag to event data
        if let serde_json::Value::Object(ref mut map) = event_data {
            map.insert("success".to_string(), json!(is_success));
            map.insert("status_code".to_string(), json!(status.as_u16()));
            if !is_success {
                map.insert(
                    "error_category".to_string(),
                    json!(if is_client_error {
                        "client_error"
                    } else {
                        "server_error"
                    }),
                );
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

/// Helper function to extract ID from a URL path
fn extract_id_from_path(path: &str, prefix: &str) -> String {
    path.strip_prefix(prefix)
        .and_then(|s| s.split('/').next())
        .unwrap_or("unknown")
        .to_string()
}

/// Hash an ID using SHA256 to protect sensitive information in telemetry
pub(crate) fn hash_id(id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Check if a path is a task-related endpoint (not a project endpoint)
/// Returns true if the path matches /api/projects/{id}/tasks*
pub(crate) fn is_task_endpoint(path: &str) -> bool {
    // Match pattern: /api/projects/{something}/tasks (with optional trailing segments)
    // This avoids false positives from project names containing "tasks"
    if let Some(after_projects) = path.strip_prefix("/api/projects/") {
        // Split on / and filter empty segments to handle double slashes
        let segments: Vec<&str> = after_projects.split('/').filter(|s| !s.is_empty()).collect();
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
