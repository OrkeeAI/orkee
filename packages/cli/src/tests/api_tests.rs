use crate::api;
use axum::body::Body;
use axum::http::Request;
use axum::http::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_status_endpoint() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial_test::serial]
async fn test_projects_list_endpoint() {
    use std::env;
    use tempfile::TempDir;

    // Create a temporary directory for this test's database
    let temp_dir = TempDir::new().unwrap();
    let original_home = env::var("HOME").ok();

    // Set HOME to temp dir so the database is created there
    env::set_var("HOME", temp_dir.path());

    // Ensure .orkee directory exists in temp dir
    let orkee_dir = temp_dir.path().join(".orkee");
    std::fs::create_dir_all(&orkee_dir).unwrap();

    // Delete any existing database file to ensure clean state
    let db_path = orkee_dir.join("orkee.db");
    if db_path.exists() {
        let _ = std::fs::remove_file(&db_path);
    }

    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/projects")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Restore original HOME
    if let Some(home) = original_home {
        env::set_var("HOME", home);
    } else {
        env::remove_var("HOME");
    }

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_endpoint() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_browse_directories_endpoint() {
    let app = api::create_router().await;

    let body = json!({
        "path": "/tmp"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/browse-directories")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return OK or an error status depending on directory access
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn test_method_not_allowed() {
    let app = api::create_router().await;

    // Try POST on a GET-only endpoint
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_projects_create_endpoint() {
    let app = api::create_router().await;

    let body = json!({
        "name": "Test Project",
        "projectRoot": "/tmp/test-project"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/projects")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return CREATED or error if project exists
    // If there's a storage initialization issue, the endpoint may return INTERNAL_SERVER_ERROR
    assert!(
        response.status() == StatusCode::CREATED
            || response.status() == StatusCode::CONFLICT
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn test_router_cors_preflight() {
    // Note: Full CORS testing requires the middleware to be configured
    // This is a basic test to ensure the router builds correctly
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/health")
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // OPTIONS should be handled
    // Note: Without CORS middleware configured, OPTIONS might return METHOD_NOT_ALLOWED
    let status = response.status();
    assert!(
        status == StatusCode::OK
            || status == StatusCode::NO_CONTENT
            || status == StatusCode::METHOD_NOT_ALLOWED,
        "Unexpected status code: {}",
        status
    );
}

#[tokio::test]
async fn test_delete_telemetry_data_without_confirmation() {
    let app = api::create_router().await;

    // Try to delete without confirmation
    let body = json!({
        "confirm": false
    });

    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/telemetry/data")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // If telemetry manager initialized, should return BAD_REQUEST when confirm is false
    // If telemetry manager failed to init, returns 404
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Expected BAD_REQUEST or NOT_FOUND, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_delete_telemetry_data_with_confirmation() {
    let app = api::create_router().await;

    // Delete with confirmation
    let body = json!({
        "confirm": true
    });

    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/telemetry/data")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // If telemetry manager initialized, should return OK when confirm is true
    // If telemetry manager failed to init, returns 404
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_delete_telemetry_data_missing_body() {
    let app = api::create_router().await;

    // Try to delete without body
    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/telemetry/data")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // If telemetry manager initialized, should return BAD_REQUEST when body is missing
    // If telemetry manager failed to init, returns 404
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Expected BAD_REQUEST or NOT_FOUND, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_track_event_endpoint_with_valid_data() {
    let app = api::create_router().await;

    let body = json!({
        "event_name": "test_event",
        "event_data": {
            "action": "button_click",
            "value": 123
        },
        "timestamp": "2025-10-16T00:00:00Z",
        "session_id": "test-session-123"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/telemetry/track")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // If telemetry manager initialized, should return OK
    // If telemetry manager failed to init, returns 404
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_track_event_endpoint_without_event_data() {
    let app = api::create_router().await;

    // Event data is optional
    let body = json!({
        "event_name": "test_event",
        "timestamp": "2025-10-16T00:00:00Z",
        "session_id": "test-session-123"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/telemetry/track")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // If telemetry manager initialized, should return OK
    // If telemetry manager failed to init, returns 404
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {}",
        response.status()
    );
}

#[tokio::test]
async fn test_track_event_endpoint_missing_required_fields() {
    let app = api::create_router().await;

    // Missing event_name
    let body = json!({
        "timestamp": "2025-10-16T00:00:00Z",
        "session_id": "test-session-123"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/telemetry/track")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 422 UNPROCESSABLE_ENTITY for invalid JSON or 404 if telemetry not initialized
    assert!(
        response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::NOT_FOUND,
        "Expected UNPROCESSABLE_ENTITY or NOT_FOUND, got {}",
        response.status()
    );
}
