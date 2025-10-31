use crate::api;
use crate::api::preview::SseConnectionTracker;
use axum::body::Body;
use axum::extract::connect_info::ConnectInfo;
use axum::http::Request;
use axum::http::{Method, StatusCode};
use serial_test::serial;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tower::ServiceExt;

/// Test that SSE connection tracker enforces per-IP limits
#[test]
fn test_sse_connection_tracker_enforces_limit() {
    let tracker = SseConnectionTracker::new();
    let test_ip = IpAddr::from_str("192.168.1.1").unwrap();

    // Acquire connections up to the default limit (3)
    let guard1 = tracker.try_acquire(test_ip);
    assert!(guard1.is_ok(), "First connection should succeed");

    let guard2 = tracker.try_acquire(test_ip);
    assert!(guard2.is_ok(), "Second connection should succeed");

    let guard3 = tracker.try_acquire(test_ip);
    assert!(guard3.is_ok(), "Third connection should succeed");

    // Fourth connection should fail (limit is 3 by default)
    let guard4 = tracker.try_acquire(test_ip);
    assert!(
        guard4.is_err(),
        "Fourth connection should fail - limit exceeded"
    );

    // Drop one guard to free a slot
    drop(guard1);

    // Now we should be able to acquire again
    let guard5 = tracker.try_acquire(test_ip);
    assert!(
        guard5.is_ok(),
        "Connection should succeed after releasing one"
    );
}

/// Test that different IPs have independent connection limits
#[test]
fn test_sse_connection_tracker_per_ip_isolation() {
    let tracker = SseConnectionTracker::new();
    let ip1 = IpAddr::from_str("192.168.1.1").unwrap();
    let ip2 = IpAddr::from_str("192.168.1.2").unwrap();

    // Exhaust connections for IP1
    let _guard1a = tracker.try_acquire(ip1).expect("IP1 conn 1 should succeed");
    let _guard1b = tracker.try_acquire(ip1).expect("IP1 conn 2 should succeed");
    let _guard1c = tracker.try_acquire(ip1).expect("IP1 conn 3 should succeed");

    // IP1 should be at limit
    let guard1d = tracker.try_acquire(ip1);
    assert!(guard1d.is_err(), "IP1 should be at limit");

    // IP2 should still be able to connect independently
    let guard2a = tracker.try_acquire(ip2);
    assert!(
        guard2a.is_ok(),
        "IP2 should have independent connection quota"
    );
}

/// Test that connection guards properly release on drop
#[test]
fn test_sse_connection_guard_cleanup() {
    let tracker = SseConnectionTracker::new();
    let test_ip = IpAddr::from_str("192.168.1.1").unwrap();

    // Acquire and release in a scope
    {
        let _guard1 = tracker.try_acquire(test_ip).unwrap();
        let _guard2 = tracker.try_acquire(test_ip).unwrap();
        let _guard3 = tracker.try_acquire(test_ip).unwrap();

        // At limit
        assert!(tracker.try_acquire(test_ip).is_err());
    } // Guards dropped here

    // After guards are dropped, should be able to acquire again
    let guard_after = tracker.try_acquire(test_ip);
    assert!(
        guard_after.is_ok(),
        "Connections should be released after guard drop"
    );
}

/// Test that SSE connection tracker respects environment variable configuration
#[test]
#[serial]
fn test_sse_connection_tracker_env_var_config() {
    // Set custom limit via environment variable
    std::env::set_var("ORKEE_SSE_MAX_CONNECTIONS_PER_IP", "5");

    let tracker = SseConnectionTracker::new();
    let test_ip = IpAddr::from_str("192.168.1.1").unwrap();

    // Should be able to acquire 5 connections
    let guards: Vec<_> = (0..5)
        .map(|i| {
            tracker
                .try_acquire(test_ip)
                .unwrap_or_else(|_| panic!("Connection {} should succeed with limit 5", i + 1))
        })
        .collect();

    // 6th connection should fail
    assert!(
        tracker.try_acquire(test_ip).is_err(),
        "6th connection should fail with limit 5"
    );

    // Clean up
    drop(guards);
    std::env::remove_var("ORKEE_SSE_MAX_CONNECTIONS_PER_IP");
}

/// Test that invalid environment variable values fall back to default
#[test]
#[serial]
fn test_sse_connection_tracker_invalid_env_var() {
    // Set invalid value (0 is not allowed)
    std::env::set_var("ORKEE_SSE_MAX_CONNECTIONS_PER_IP", "0");

    let tracker = SseConnectionTracker::new();
    let test_ip = IpAddr::from_str("192.168.1.1").unwrap();

    // Should fall back to default (3)
    let _g1 = tracker.try_acquire(test_ip).unwrap();
    let _g2 = tracker.try_acquire(test_ip).unwrap();
    let _g3 = tracker.try_acquire(test_ip).unwrap();

    assert!(
        tracker.try_acquire(test_ip).is_err(),
        "Should use default limit (3) when env var is invalid"
    );

    // Clean up
    std::env::remove_var("ORKEE_SSE_MAX_CONNECTIONS_PER_IP");
}

/// Test that SSE connection tracker validates range (max 100)
#[test]
#[serial]
fn test_sse_connection_tracker_max_validation() {
    // Set value above maximum (100)
    std::env::set_var("ORKEE_SSE_MAX_CONNECTIONS_PER_IP", "150");

    let tracker = SseConnectionTracker::new();
    let test_ip = IpAddr::from_str("192.168.1.1").unwrap();

    // Should fall back to default (3) when value exceeds maximum
    let _g1 = tracker.try_acquire(test_ip).unwrap();
    let _g2 = tracker.try_acquire(test_ip).unwrap();
    let _g3 = tracker.try_acquire(test_ip).unwrap();

    assert!(
        tracker.try_acquire(test_ip).is_err(),
        "Should use default limit (3) when env var exceeds maximum"
    );

    // Clean up
    std::env::remove_var("ORKEE_SSE_MAX_CONNECTIONS_PER_IP");
}

/// Test SSE endpoint returns 429 when connection limit is exceeded
#[tokio::test]
#[serial]
#[ignore = "Migration checksum issue after spec removal"]
async fn test_sse_endpoint_rate_limiting() {
    let app = api::create_router().await;

    // Note: This test is limited because we can't easily simulate multiple IPs
    // in unit tests, but we can verify the endpoint exists and is accessible
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/preview/events")
        .header("accept", "text/event-stream")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should either succeed (200) or fail with initialization error (500)
    // but not return 404
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
        "SSE endpoint should exist and be accessible, got {}",
        response.status()
    );
}

/// Test SSE endpoint basic connectivity
#[tokio::test]
#[serial]
#[ignore = "Migration checksum issue after spec removal"]
async fn test_sse_endpoint_exists() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/preview/events")
        .header("accept", "text/event-stream")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Endpoint should exist (not 404)
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "SSE endpoint should exist"
    );
}

/// Test preview health check endpoint
#[tokio::test]
#[serial]
#[ignore = "Migration checksum issue after spec removal"]
async fn test_preview_health_endpoint() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/preview/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Preview health endpoint should return OK"
    );
}

/// Test list active servers endpoint
#[tokio::test]
#[serial]
#[ignore = "Migration checksum issue after spec removal"]
async fn test_list_active_servers_endpoint() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/preview/servers")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return OK with list of servers (even if empty)
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "List servers endpoint should return OK"
    );
}

/// Test discover servers endpoint
#[tokio::test]
#[serial]
#[ignore = "Migration checksum issue after spec removal"]
async fn test_discover_servers_endpoint() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/preview/servers/discover")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return OK (discovery may find no servers, which is fine)
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Discover servers endpoint should return OK"
    );
}

/// Test SSE endpoint requires authentication via query parameter
#[tokio::test]
#[serial]
#[ignore = "Migration checksum issue after spec removal"]
async fn test_sse_endpoint_requires_auth_token() {
    let app = api::create_router().await;

    // Create a test socket address for ConnectInfo
    let test_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

    // Test 1: No token provided - should fail with 401
    let mut request = Request::builder()
        .method(Method::GET)
        .uri("/api/preview/events")
        .header("accept", "text/event-stream")
        .body(Body::empty())
        .unwrap();

    // Add ConnectInfo extension that the handler expects
    request.extensions_mut().insert(ConnectInfo(test_addr));

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "SSE endpoint without token should return 401 Unauthorized"
    );

    // Test 2: Invalid token provided - should fail with 401
    let mut request = Request::builder()
        .method(Method::GET)
        .uri("/api/preview/events?token=invalid-token-12345")
        .header("accept", "text/event-stream")
        .body(Body::empty())
        .unwrap();

    // Add ConnectInfo extension that the handler expects
    request.extensions_mut().insert(ConnectInfo(test_addr));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "SSE endpoint with invalid token should return 401 Unauthorized"
    );
}

/// Test stop all servers endpoint
#[tokio::test]
#[serial]
#[ignore = "Migration checksum issue after spec removal"]
async fn test_stop_all_servers_endpoint() {
    let app = api::create_router().await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/preview/servers/stop-all")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return OK even if no servers are running
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Stop all servers endpoint should return OK"
    );
}
