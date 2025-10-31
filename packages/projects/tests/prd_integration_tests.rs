// ABOUTME: Integration tests for PRD (Product Requirements Document) API endpoints
// ABOUTME: Tests CRUD operations for PRDs including list, create, get, update, delete, and capabilities

mod common;

use common::{create_test_project, delete, get, post_json, put_json, setup_test_server};
use serde_json::json;

#[tokio::test]
async fn test_create_prd() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a PRD
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "Test PRD",
            "contentMarkdown": "# Test PRD\n\nThis is a test PRD",
            "status": "draft",
            "source": "manual",
            "createdBy": "test-user"
        }),
    )
    .await;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["title"], "Test PRD");
    assert_eq!(body["data"]["status"], "draft");
}

#[tokio::test]
async fn test_list_prds() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create multiple PRDs
    for i in 1..=3 {
        post_json(
            &ctx.base_url,
            &format!("/{}/prds", project_id),
            &json!({
                "title": format!("Test PRD {}", i),
                "contentMarkdown": format!("# PRD {}", i),
            }),
        )
        .await;
    }

    // List PRDs
    let response = get(&ctx.base_url, &format!("/{}/prds", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["data"].is_array());
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_list_prds_with_pagination() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create 5 PRDs
    for i in 1..=5 {
        post_json(
            &ctx.base_url,
            &format!("/{}/prds", project_id),
            &json!({
                "title": format!("PRD {}", i),
                "contentMarkdown": format!("# PRD {}", i),
            }),
        )
        .await;
    }

    // Get first page with limit=2
    let response = get(&ctx.base_url, &format!("/{}/prds?limit=2", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"]["pagination"]["totalItems"], 5);
    assert_eq!(body["data"]["pagination"]["page"], 1);
}

#[tokio::test]
async fn test_get_prd() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a PRD
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "Get Test PRD",
            "contentMarkdown": "# Get Test",
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let prd_id = create_body["data"]["id"].as_str().unwrap();

    // Get the PRD
    let response = get(&ctx.base_url, &format!("/{}/prds/{}", project_id, prd_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], prd_id);
    assert_eq!(body["data"]["title"], "Get Test PRD");
}

#[tokio::test]
async fn test_get_nonexistent_prd() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Try to get a nonexistent PRD
    let response = get(&ctx.base_url, &format!("/{}/prds/nonexistent", project_id)).await;

    assert_eq!(response.status(), 404);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_update_prd() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a PRD
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "Original Title",
            "contentMarkdown": "# Original Content",
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let prd_id = create_body["data"]["id"].as_str().unwrap();

    // Update the PRD
    let response = put_json(
        &ctx.base_url,
        &format!("/{}/prds/{}", project_id, prd_id),
        &json!({
            "title": "Updated Title",
            "contentMarkdown": "# Updated Content",
            "status": "approved"
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify the update
    let get_response = get(&ctx.base_url, &format!("/{}/prds/{}", project_id, prd_id)).await;
    let get_body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_body["data"]["title"], "Updated Title");
    assert_eq!(get_body["data"]["contentMarkdown"], "# Updated Content");
    assert_eq!(get_body["data"]["status"], "approved");
}

#[tokio::test]
async fn test_update_prd_partial() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a PRD
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "Original Title",
            "contentMarkdown": "# Original Content",
            "status": "draft"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let prd_id = create_body["data"]["id"].as_str().unwrap();

    // Update only the title
    let response = put_json(
        &ctx.base_url,
        &format!("/{}/prds/{}", project_id, prd_id),
        &json!({
            "title": "New Title Only"
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    // Verify only title changed
    let get_response = get(&ctx.base_url, &format!("/{}/prds/{}", project_id, prd_id)).await;
    let get_body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_body["data"]["title"], "New Title Only");
    assert_eq!(get_body["data"]["contentMarkdown"], "# Original Content");
    assert_eq!(get_body["data"]["status"], "draft");
}

#[tokio::test]
async fn test_delete_prd() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a PRD
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "PRD to Delete",
            "contentMarkdown": "# Delete Me",
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let prd_id = create_body["data"]["id"].as_str().unwrap();

    // Delete the PRD
    let response = delete(&ctx.base_url, &format!("/{}/prds/{}", project_id, prd_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify it's deleted
    let get_response = get(&ctx.base_url, &format!("/{}/prds/{}", project_id, prd_id)).await;
    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn test_create_prd_with_defaults() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create PRD with minimal fields
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "Minimal PRD",
            "contentMarkdown": "# Minimal",
        }),
    )
    .await;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "draft"); // Default status
    assert_eq!(body["data"]["source"], "manual"); // Default source
}

#[tokio::test]
async fn test_create_prd_validation() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Try to create PRD without required fields
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "No Content"
            // Missing contentMarkdown
        }),
    )
    .await;

    // Should fail validation
    assert!(!response.status().is_success());
}

#[tokio::test]
async fn test_list_prds_for_nonexistent_project() {
    let ctx = setup_test_server().await;

    // Try to list PRDs for nonexistent project
    let response = get(&ctx.base_url, "/nonexistent-project/prds").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 0);
}
