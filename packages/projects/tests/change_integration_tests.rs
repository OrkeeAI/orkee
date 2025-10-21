// ABOUTME: Integration tests for Change Management API endpoints
// ABOUTME: Tests CRUD operations for spec changes and deltas

mod common;

use common::{create_test_project, get, post_json, put_json, setup_test_server};
use serde_json::json;

#[tokio::test]
async fn test_create_change() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a change
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/changes", project_id),
        &json!({
            "proposalMarkdown": "# Proposal\n\nAdd user auth",
            "tasksMarkdown": "# Tasks\n\n- Implement JWT",
            "createdBy": "test-user"
        }),
    )
    .await;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "draft");
}

#[tokio::test]
async fn test_list_changes() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create multiple changes
    for i in 1..=3 {
        post_json(
            &ctx.base_url,
            &format!("/{}/changes", project_id),
            &json!({
                "proposalMarkdown": format!("# Proposal {}", i),
                "tasksMarkdown": format!("# Tasks {}", i),
                "createdBy": "test-user"
            }),
        )
        .await;
    }

    // List changes
    let response = get(&ctx.base_url, &format!("/{}/changes", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["data"].is_array());
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_get_change() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a change
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/changes", project_id),
        &json!({
            "proposalMarkdown": "# Test Proposal",
            "tasksMarkdown": "# Test Tasks",
            "createdBy": "test-user"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let change_id = create_body["data"]["id"].as_str().unwrap();

    // Get the change
    let response = get(
        &ctx.base_url,
        &format!("/{}/changes/{}", project_id, change_id),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], change_id);
}

#[tokio::test]
async fn test_update_change_status() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a change
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/changes", project_id),
        &json!({
            "proposalMarkdown": "# Proposal",
            "tasksMarkdown": "# Tasks",
            "createdBy": "test-user"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let change_id = create_body["data"]["id"].as_str().unwrap();

    // Update status
    let response = put_json(
        &ctx.base_url,
        &format!("/{}/changes/{}/status", project_id, change_id),
        &json!({
            "status": "approved",
            "approvedBy": "approver-user"
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify the update
    let get_response = get(
        &ctx.base_url,
        &format!("/{}/changes/{}", project_id, change_id),
    )
    .await;
    let get_body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_body["data"]["status"], "approved");
}

#[tokio::test]
async fn test_get_change_deltas() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a change
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/changes", project_id),
        &json!({
            "proposalMarkdown": "# Proposal",
            "tasksMarkdown": "# Tasks",
            "createdBy": "test-user"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let change_id = create_body["data"]["id"].as_str().unwrap();

    // Get deltas
    let response = get(
        &ctx.base_url,
        &format!("/{}/changes/{}/deltas", project_id, change_id),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["data"].is_array());
}

#[tokio::test]
async fn test_create_delta() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a change first
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/changes", project_id),
        &json!({
            "proposalMarkdown": "# Proposal",
            "tasksMarkdown": "# Tasks",
            "createdBy": "test-user"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let change_id = create_body["data"]["id"].as_str().unwrap();

    // Create a delta (for a new capability, so no capability_id)
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/changes/{}/deltas", project_id, change_id),
        &json!({
            "capabilityName": "New Capability",
            "deltaType": "added",
            "deltaMarkdown": "# New capability markdown",
            "position": 0
        }),
    )
    .await;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
}

#[tokio::test]
async fn test_list_changes_with_pagination() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create 5 changes
    for i in 1..=5 {
        post_json(
            &ctx.base_url,
            &format!("/{}/changes", project_id),
            &json!({
                "proposalMarkdown": format!("# Proposal {}", i),
                "tasksMarkdown": format!("# Tasks {}", i),
                "createdBy": "test-user"
            }),
        )
        .await;
    }

    // Get first page with limit=2
    let response = get(&ctx.base_url, &format!("/{}/changes?limit=2", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"]["pagination"]["totalItems"], 5);
}
