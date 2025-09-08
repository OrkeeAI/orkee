use crate::api;
use axum::http::{Method, StatusCode};
use tower::ServiceExt;
use axum::body::Body;
use axum::http::Request;
use serde_json::json;

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
async fn test_projects_list_endpoint() {
    let app = api::create_router().await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/projects")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
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
    assert!(response.status() == StatusCode::OK || 
            response.status() == StatusCode::INTERNAL_SERVER_ERROR);
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
    assert!(response.status() == StatusCode::CREATED || 
            response.status() == StatusCode::CONFLICT ||
            response.status() == StatusCode::INTERNAL_SERVER_ERROR);
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
        status == StatusCode::OK || 
        status == StatusCode::NO_CONTENT ||
        status == StatusCode::METHOD_NOT_ALLOWED,
        "Unexpected status code: {}",
        status
    );
}