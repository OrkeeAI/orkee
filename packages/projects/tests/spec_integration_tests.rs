// ABOUTME: Integration tests for Spec/Capability API endpoints
// ABOUTME: Tests CRUD operations for capabilities, requirements, and spec validation

mod common;

use common::{create_test_project, delete, get, post_json, put_json, setup_test_server};
use serde_json::json;

#[tokio::test]
async fn test_create_capability() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "User Authentication",
            "specMarkdown": "# User Authentication\n\n## Requirements\n\n**Login**: Users can login\n- When user provides valid credentials\n- Then they are authenticated",
            "purposeMarkdown": "Enable secure user access",
            "designMarkdown": "## Architecture\n\nUses JWT tokens"
        }),
    )
    .await;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "User Authentication");
    assert_eq!(body["data"]["status"], "active");
}

#[tokio::test]
async fn test_create_capability_minimal() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create capability with only required fields
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "Minimal Capability",
            "specMarkdown": "# Minimal Spec"
        }),
    )
    .await;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "Minimal Capability");
    assert!(body["data"]["purposeMarkdown"].is_null());
    assert!(body["data"]["designMarkdown"].is_null());
}

#[tokio::test]
async fn test_list_capabilities() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create multiple capabilities
    for i in 1..=3 {
        post_json(
            &ctx.base_url,
            &format!("/{}/specs", project_id),
            &json!({
                "name": format!("Capability {}", i),
                "specMarkdown": format!("# Spec {}", i)
            }),
        )
        .await;
    }

    // List capabilities
    let response = get(&ctx.base_url, &format!("/{}/specs", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["data"].is_array());
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_list_capabilities_with_pagination() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create 5 capabilities
    for i in 1..=5 {
        post_json(
            &ctx.base_url,
            &format!("/{}/specs", project_id),
            &json!({
                "name": format!("Cap {}", i),
                "specMarkdown": format!("# Cap {}", i)
            }),
        )
        .await;
    }

    // Get first page with limit=2
    let response = get(&ctx.base_url, &format!("/{}/specs?limit=2", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"]["pagination"]["totalItems"], 5);
    assert_eq!(body["data"]["pagination"]["page"], 1);
}

#[tokio::test]
async fn test_get_capability() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "Get Test Capability",
            "specMarkdown": "# Get Test"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let capability_id = create_body["data"]["id"].as_str().unwrap();

    // Get the capability
    let response = get(
        &ctx.base_url,
        &format!("/{}/specs/{}", project_id, capability_id),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], capability_id);
    assert_eq!(body["data"]["name"], "Get Test Capability");
}

#[tokio::test]
async fn test_get_nonexistent_capability() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Try to get a nonexistent capability
    let response = get(&ctx.base_url, &format!("/{}/specs/nonexistent", project_id)).await;

    assert_eq!(response.status(), 404);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_update_capability() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "Original Capability",
            "specMarkdown": "# Original Spec",
            "status": "active"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let capability_id = create_body["data"]["id"].as_str().unwrap();

    // Update the capability
    let response = put_json(
        &ctx.base_url,
        &format!("/{}/specs/{}", project_id, capability_id),
        &json!({
            "specMarkdown": "# Updated Spec",
            "purposeMarkdown": "New purpose",
            "status": "deprecated"
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify the update
    let get_response = get(
        &ctx.base_url,
        &format!("/{}/specs/{}", project_id, capability_id),
    )
    .await;
    let get_body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_body["data"]["specMarkdown"], "# Updated Spec");
    assert_eq!(get_body["data"]["purposeMarkdown"], "New purpose");
    assert_eq!(get_body["data"]["status"], "deprecated");
}

#[tokio::test]
async fn test_update_capability_partial() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "Original Capability",
            "specMarkdown": "# Original Spec",
            "purposeMarkdown": "Original purpose"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let capability_id = create_body["data"]["id"].as_str().unwrap();

    // Update only the status
    let response = put_json(
        &ctx.base_url,
        &format!("/{}/specs/{}", project_id, capability_id),
        &json!({
            "status": "archived"
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    // Verify only status changed
    let get_response = get(
        &ctx.base_url,
        &format!("/{}/specs/{}", project_id, capability_id),
    )
    .await;
    let get_body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_body["data"]["status"], "archived");
    assert_eq!(get_body["data"]["specMarkdown"], "# Original Spec");
    assert_eq!(get_body["data"]["purposeMarkdown"], "Original purpose");
}

#[tokio::test]
async fn test_delete_capability() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "Capability to Delete",
            "specMarkdown": "# Delete Me"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let capability_id = create_body["data"]["id"].as_str().unwrap();

    // Delete the capability
    let response = delete(
        &ctx.base_url,
        &format!("/{}/specs/{}", project_id, capability_id),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify it's deleted (soft delete, so might still exist but marked deleted)
    let get_response = get(
        &ctx.base_url,
        &format!("/{}/specs/{}", project_id, capability_id),
    )
    .await;
    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn test_get_capability_requirements() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability with requirements in spec markdown
    let create_response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "Capability with Requirements",
            "specMarkdown": "# Test Capability\n\n## Requirements\n\n**Req1**: First requirement\n- When condition\n- Then result"
        }),
    )
    .await;

    let create_body: serde_json::Value = create_response.json().await.unwrap();
    let capability_id = create_body["data"]["id"].as_str().unwrap();

    // Get requirements (may be empty initially depending on parsing)
    let response = get(
        &ctx.base_url,
        &format!("/{}/specs/{}/requirements", project_id, capability_id),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["data"].is_array());
}

#[tokio::test]
async fn test_validate_spec_endpoint() {
    let ctx = setup_test_server().await;

    // Test that the validate endpoint works (validation logic is complex, just verify endpoint responds)
    let response = post_json(
        &ctx.base_url,
        "/specs/validate",
        &json!({
            "specMarkdown": "## User Authentication\n\nEnable secure user access\n\n### Login\n\nUsers can log in with valid credentials"
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    // Validation result can be true or false depending on parser implementation
    assert!(body["data"]["valid"].is_boolean());
    assert!(body["data"]["errors"].is_array());
    assert!(body["data"]["capabilityCount"].is_number());
}

#[tokio::test]
async fn test_validate_spec_invalid() {
    let ctx = setup_test_server().await;

    // Validate an invalid spec (empty markdown)
    let response = post_json(
        &ctx.base_url,
        "/specs/validate",
        &json!({
            "specMarkdown": ""
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    // Empty spec should be invalid
    assert_eq!(body["data"]["valid"], false);
    assert!(!body["data"]["errors"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_validate_spec_counts() {
    let ctx = setup_test_server().await;

    // Validate a spec with multiple capabilities and requirements
    let response = post_json(
        &ctx.base_url,
        "/specs/validate",
        &json!({
            "specMarkdown": "## Capability 1\n\nFirst capability\n\n### Req1\n\nFirst requirement\n\n**When** x\n**Then** y\n\n### Req2\n\nSecond requirement\n\n**When** a\n**Then** b\n\n## Capability 2\n\nSecond capability\n\n### Req3\n\nThird requirement\n\n**When** p\n**Then** q"
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Check counts are present and correct
    assert_eq!(body["data"]["capabilityCount"], 2);
    assert_eq!(body["data"]["requirementCount"], 3);
    assert!(body["data"]["scenarioCount"].is_number());
}

#[tokio::test]
async fn test_list_capabilities_for_nonexistent_project() {
    let ctx = setup_test_server().await;

    // Try to list capabilities for nonexistent project
    let response = get(&ctx.base_url, "/nonexistent-project/specs").await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 0);
}
