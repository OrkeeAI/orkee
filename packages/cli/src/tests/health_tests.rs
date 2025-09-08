use crate::api::health::{health_check, status_check};

#[tokio::test]
async fn test_health_check_returns_ok() {
    let result = health_check().await;
    assert!(result.is_ok());

    let json = result.unwrap();
    let value = json.0;

    assert_eq!(
        value.get("status").and_then(|v| v.as_str()),
        Some("healthy")
    );
    assert_eq!(
        value.get("service").and_then(|v| v.as_str()),
        Some("orkee-cli")
    );
    assert!(value.get("timestamp").is_some());
    assert!(value.get("version").is_some());
}

#[tokio::test]
async fn test_health_check_timestamp() {
    let result = health_check().await.unwrap();
    let value = result.0;

    let timestamp = value.get("timestamp").and_then(|v| v.as_u64());

    assert!(timestamp.is_some());
    // Timestamp should be reasonable (after year 2020)
    assert!(timestamp.unwrap() > 1577836800); // Jan 1, 2020
}

#[tokio::test]
async fn test_status_check_returns_ok() {
    let result = status_check().await;
    assert!(result.is_ok());

    let json = result.unwrap();
    let value = json.0;

    assert_eq!(
        value.get("status").and_then(|v| v.as_str()),
        Some("healthy")
    );
    assert_eq!(
        value.get("service").and_then(|v| v.as_str()),
        Some("orkee-cli")
    );
    assert!(value.get("timestamp").is_some());
    assert!(value.get("version").is_some());
    assert!(value.get("uptime").is_some());
    assert!(value.get("memory").is_some());
    assert!(value.get("connections").is_some());
}

#[tokio::test]
async fn test_status_check_memory_fields() {
    let result = status_check().await.unwrap();
    let value = result.0;

    let memory = value.get("memory").unwrap();
    assert!(memory.get("used").is_some());
    assert!(memory.get("available").is_some());
}

#[tokio::test]
async fn test_status_check_connections_fields() {
    let result = status_check().await.unwrap();
    let value = result.0;

    let connections = value.get("connections").unwrap();
    assert!(connections.get("active").is_some());
    assert!(connections.get("total").is_some());
}

#[tokio::test]
async fn test_health_endpoints_consistency() {
    let health = health_check().await.unwrap();
    let status = status_check().await.unwrap();

    let health_value = health.0;
    let status_value = status.0;

    // Both should report the same basic info
    assert_eq!(health_value.get("status"), status_value.get("status"));
    assert_eq!(health_value.get("service"), status_value.get("service"));
    assert_eq!(health_value.get("version"), status_value.get("version"));
}

#[tokio::test]
async fn test_concurrent_health_checks() {
    use futures::future::join_all;

    // Make multiple concurrent health check requests
    let handles: Vec<_> = (0..10)
        .map(|_| tokio::spawn(async { health_check().await }))
        .collect();

    let results = join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        let health_result = result.unwrap();
        assert!(health_result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_status_checks() {
    use futures::future::join_all;

    // Make multiple concurrent status check requests
    let handles: Vec<_> = (0..10)
        .map(|_| tokio::spawn(async { status_check().await }))
        .collect();

    let results = join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        let status_result = result.unwrap();
        assert!(status_result.is_ok());
    }
}
