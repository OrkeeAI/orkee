// ABOUTME: Integration tests for AI Usage Log API endpoints
// ABOUTME: Tests log listing and statistics aggregation

mod common;

use common::{get, setup_test_server};

#[tokio::test]
async fn test_list_logs_empty() {
    let ctx = setup_test_server().await;

    // List logs (should be empty initially)
    let response = get(&ctx.base_url, "/logs").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_logs_with_filters() {
    let ctx = setup_test_server().await;

    // List logs with query parameters
    let response = get(
        &ctx.base_url,
        "/logs?projectId=test-project&limit=10&offset=0",
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_list_logs_with_date_filters() {
    let ctx = setup_test_server().await;

    // List logs with date range
    let response = get(
        &ctx.base_url,
        "/logs?startDate=2024-01-01T00:00:00Z&endDate=2024-12-31T23:59:59Z",
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_list_logs_with_operation_filter() {
    let ctx = setup_test_server().await;

    // List logs filtered by operation
    let response = get(&ctx.base_url, "/logs?operation=analyze-prd").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_list_logs_with_model_filter() {
    let ctx = setup_test_server().await;

    // List logs filtered by model
    let response = get(&ctx.base_url, "/logs?model=claude-3-5-sonnet-20241022").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_list_logs_with_provider_filter() {
    let ctx = setup_test_server().await;

    // List logs filtered by provider
    let response = get(&ctx.base_url, "/logs?provider=anthropic").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_get_stats_empty() {
    let ctx = setup_test_server().await;

    // Get stats (should return zero counts initially)
    let response = get(&ctx.base_url, "/stats").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_object());
    // Stats should have structure even if counts are zero (uses snake_case field names)
    assert!(body["data"]["total_requests"].is_number());
    assert!(body["data"]["total_cost"].is_number());
}

#[tokio::test]
async fn test_get_stats_with_project_filter() {
    let ctx = setup_test_server().await;

    // Get stats filtered by project
    let response = get(&ctx.base_url, "/stats?projectId=test-project").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_object());
}

#[tokio::test]
async fn test_get_stats_with_date_range() {
    let ctx = setup_test_server().await;

    // Get stats with date range
    let response = get(
        &ctx.base_url,
        "/stats?startDate=2024-01-01T00:00:00Z&endDate=2024-12-31T23:59:59Z",
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_object());
}

#[tokio::test]
async fn test_list_logs_pagination() {
    let ctx = setup_test_server().await;

    // Test pagination parameters
    let response = get(&ctx.base_url, "/logs?limit=5&offset=10").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    // Should return at most 5 items
    assert!(body["data"].as_array().unwrap().len() <= 5);
}
