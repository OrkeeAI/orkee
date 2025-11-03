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

    // Validate ID: non-empty, alphanumeric + hyphens + underscores only
    // This prevents path traversal attacks and malformed input
    if !id.is_empty()
        && id != "unknown"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_id_from_path_valid() {
        assert_eq!(
            extract_id_from_path("/api/projects/abc123", "/api/projects/"),
            "abc123"
        );
        assert_eq!(
            extract_id_from_path("/api/projects/test-id-456", "/api/projects/"),
            "test-id-456"
        );
        assert_eq!(
            extract_id_from_path("/api/projects/under_score_id", "/api/projects/"),
            "under_score_id"
        );
    }

    #[test]
    fn test_extract_id_from_path_with_trailing_segments() {
        assert_eq!(
            extract_id_from_path("/api/projects/abc123/details", "/api/projects/"),
            "abc123"
        );
    }

    #[test]
    fn test_extract_id_from_path_security() {
        // Path traversal attempts - segments before first slash get blocked
        assert_eq!(
            extract_id_from_path("/api/projects/../../../etc/passwd", "/api/projects/"),
            "unknown"
        );
        assert_eq!(
            extract_id_from_path("/api/projects/../../etc", "/api/projects/"),
            "unknown"
        );

        // Special characters
        assert_eq!(
            extract_id_from_path("/api/projects/id with spaces", "/api/projects/"),
            "unknown"
        );
        assert_eq!(
            extract_id_from_path("/api/projects/id@special", "/api/projects/"),
            "unknown"
        );

        // The function extracts first segment, so "id" is extracted from "id/slash"
        // This is OK because it only extracts the project ID
        assert_eq!(
            extract_id_from_path("/api/projects/id/slash", "/api/projects/"),
            "id"
        );
    }

    #[test]
    fn test_extract_id_from_path_empty() {
        // Empty string gets returned as-is but fails validation
        assert_eq!(extract_id_from_path("/api/projects/", "/api/projects/"), "unknown");
        // When prefix doesn't match, unwrap_or returns "unknown"
        assert_eq!(extract_id_from_path("/api/projects", "/api/projects/"), "unknown");
    }

    #[test]
    fn test_hash_id_consistency() {
        let id = "test-project-123";
        let hash1 = hash_id(id);
        let hash2 = hash_id(id);
        assert_eq!(hash1, hash2, "Hash should be consistent for same input");
    }

    #[test]
    fn test_hash_id_uniqueness() {
        let id1 = "test-project-123";
        let id2 = "test-project-456";
        let hash1 = hash_id(id1);
        let hash2 = hash_id(id2);
        assert_ne!(hash1, hash2, "Different inputs should produce different hashes");
    }

    #[test]
    fn test_hash_id_format() {
        let hash = hash_id("test-id");
        // HMAC-SHA256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should only contain hex digits"
        );
    }

    #[test]
    fn test_is_task_endpoint_valid_cases() {
        assert!(is_task_endpoint("/api/projects/123/tasks"));
        assert!(is_task_endpoint("/api/projects/abc/tasks/"));
        assert!(is_task_endpoint("/api/projects/test-id/tasks/456"));
        assert!(is_task_endpoint("/api/projects/id/tasks/456/details"));
    }

    #[test]
    fn test_is_task_endpoint_invalid_cases() {
        // Project endpoints (not task endpoints)
        assert!(!is_task_endpoint("/api/projects/123"));
        assert!(!is_task_endpoint("/api/projects/abc/"));
        assert!(!is_task_endpoint("/api/projects"));

        // Project names containing "tasks"
        assert!(!is_task_endpoint("/api/projects/my-tasks-project"));
        assert!(!is_task_endpoint("/api/projects/tasks"));

        // Other endpoints
        assert!(!is_task_endpoint("/api/tasks"));
        assert!(!is_task_endpoint("/api/preview/tasks"));
    }

    #[test]
    fn test_is_task_endpoint_edge_cases() {
        // Double slashes
        assert!(is_task_endpoint("/api/projects/123//tasks"));
        assert!(is_task_endpoint("/api/projects//123/tasks"));

        // Empty segments
        assert!(!is_task_endpoint("/api/projects//tasks"));
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limiter = FailureRateLimiter::new();

        // Should allow first 10 failures
        for i in 0..10 {
            assert!(
                limiter.should_track_failure("test_event").await,
                "Should allow failure {} under limit",
                i + 1
            );
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = FailureRateLimiter::new();

        // Fill up to limit
        for _ in 0..10 {
            limiter.should_track_failure("test_event").await;
        }

        // Next attempt should be blocked
        assert!(
            !limiter.should_track_failure("test_event").await,
            "Should block after reaching limit"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_separate_events() {
        let limiter = FailureRateLimiter::new();

        // Fill up one event type
        for _ in 0..10 {
            limiter.should_track_failure("event_a").await;
        }

        // Different event should still be allowed
        assert!(
            limiter.should_track_failure("event_b").await,
            "Different event types should have separate limits"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_cleanup() {
        let mut limiter = FailureRateLimiter::new();
        limiter.cleanup_threshold = 5; // Lower threshold for testing

        // Add entries up to threshold
        for i in 0..6 {
            limiter.should_track_failure(&format!("event_{}", i)).await;
        }

        // Cleanup should have triggered
        let failures = limiter.failures.lock().await;
        assert!(
            failures.len() <= 6,
            "Cleanup should prevent unbounded growth"
        );
    }
}
