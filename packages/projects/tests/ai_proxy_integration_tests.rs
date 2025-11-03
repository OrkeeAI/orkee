// ABOUTME: Integration tests for AI Proxy API endpoints
// ABOUTME: Tests AI-powered operations (analyze PRD, generate spec, suggest tasks, refine spec, validate completion)

mod common;

use common::{create_test_project, get, post_json, setup_test_server};
use serde_json::json;

#[tokio::test]
#[ignore = "AI endpoints moved to frontend in Phase 6 - backend routes removed"]
async fn test_analyze_prd_endpoint() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a PRD first
    let prd_response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "Test PRD",
            "contentMarkdown": "# Product Vision\n\nBuild a user authentication system with login and registration."
        }),
    )
    .await;

    let prd_body: serde_json::Value = prd_response.json().await.unwrap();
    let prd_id = prd_body["data"]["id"].as_str().unwrap();

    // Test analyze-prd endpoint (will fail without API key, but should accept request)
    let response = post_json(
        &ctx.base_url,
        "/ai/analyze-prd",
        &json!({
            "prdId": prd_id,
            "projectId": project_id,
            "content": "# Product Vision\n\nBuild a user authentication system."
        }),
    )
    .await;

    // Should return 200, 422 (validation error), or 500, not 404
    assert!(
        response.status() == 200 || response.status() == 422 || response.status() == 500,
        "Expected 200, 422, or 500, got {}",
        response.status()
    );

    // Only parse JSON for successful responses
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["success"].is_boolean());
    }
}

#[tokio::test]
async fn test_analyze_prd_missing_fields() {
    let ctx = setup_test_server().await;

    // Test with missing required fields
    let response = post_json(
        &ctx.base_url,
        "/ai/analyze-prd",
        &json!({
            "prdId": "test-prd-id"
            // Missing projectId and content
        }),
    )
    .await;

    // Should fail validation
    assert!(!response.status().is_success());
}

#[tokio::test]
#[ignore = "AI endpoints moved to frontend in Phase 6 - backend routes removed"]
async fn test_generate_spec_endpoint() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Test generate-spec endpoint
    let response = post_json(
        &ctx.base_url,
        "/ai/generate-spec",
        &json!({
            "projectId": project_id,
            "capabilityName": "User Authentication",
            "requirements": [
                "Users can register with email and password",
                "Users can login with credentials",
                "Sessions expire after 24 hours"
            ],
            "context": "Building a secure web application"
        }),
    )
    .await;

    // Should accept request (may fail without API key, but endpoint should work)
    assert!(
        response.status() == 200 || response.status() == 422 || response.status() == 500,
        "Expected 200, 422, or 500, got {}",
        response.status()
    );

    // Only parse JSON for successful responses
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["success"].is_boolean());
    }
}

#[tokio::test]
#[ignore = "AI endpoints moved to frontend in Phase 6 - backend routes removed"]
async fn test_generate_spec_minimal() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Test with minimal fields (empty requirements)
    let response = post_json(
        &ctx.base_url,
        "/ai/generate-spec",
        &json!({
            "projectId": project_id,
            "capabilityName": "Minimal Capability",
            "requirements": []
        }),
    )
    .await;

    assert!(
        response.status() == 200 || response.status() == 422 || response.status() == 500,
        "Expected 200, 422, or 500, got {}",
        response.status()
    );
}

#[tokio::test]
#[ignore = "Specs endpoint no longer exists - needs update for new ideate API"]
async fn test_suggest_tasks_endpoint() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability first
    let cap_response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "User Authentication",
            "specMarkdown": "## User Authentication\n\n### Login\n\nUsers can login\n\n**When** user provides credentials\n**Then** they are authenticated"
        }),
    )
    .await;

    let cap_body: serde_json::Value = cap_response.json().await.unwrap();
    let capability_id = cap_body["data"]["id"].as_str().unwrap();

    // Test suggest-tasks endpoint
    let response = post_json(
        &ctx.base_url,
        "/ai/suggest-tasks",
        &json!({
            "projectId": project_id,
            "capabilityId": capability_id,
            "specContent": "## User Authentication\n\nUsers can login with email and password"
        }),
    )
    .await;

    assert!(
        response.status() == 200 || response.status() == 422 || response.status() == 500,
        "Expected 200, 422, or 500, got {}",
        response.status()
    );

    // Only parse JSON for successful responses
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["success"].is_boolean());
    }
}

#[tokio::test]
#[ignore = "Specs endpoint no longer exists - needs update for new ideate API"]
async fn test_refine_spec_endpoint() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability first
    let cap_response = post_json(
        &ctx.base_url,
        &format!("/{}/specs", project_id),
        &json!({
            "name": "User Authentication",
            "specMarkdown": "## User Authentication\n\n### Login\n\nBasic login functionality"
        }),
    )
    .await;

    let cap_body: serde_json::Value = cap_response.json().await.unwrap();
    let capability_id = cap_body["data"]["id"].as_str().unwrap();

    // Test refine-spec endpoint
    let response = post_json(
        &ctx.base_url,
        "/ai/refine-spec",
        &json!({
            "projectId": project_id,
            "capabilityId": capability_id,
            "currentSpec": "## User Authentication\n\nBasic login",
            "feedback": "Add password reset functionality and two-factor authentication"
        }),
    )
    .await;

    assert!(
        response.status() == 200 || response.status() == 422 || response.status() == 500,
        "Expected 200, 422, or 500, got {}",
        response.status()
    );

    // Only parse JSON for successful responses
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["success"].is_boolean());
    }
}

#[tokio::test]
async fn test_refine_spec_missing_feedback() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Test without feedback field
    let response = post_json(
        &ctx.base_url,
        "/ai/refine-spec",
        &json!({
            "projectId": project_id,
            "capabilityId": "test-cap",
            "currentSpec": "Some spec"
            // Missing feedback
        }),
    )
    .await;

    // Should fail validation
    assert!(!response.status().is_success());
}

#[tokio::test]
#[ignore = "AI endpoints moved to frontend in Phase 6 - backend routes removed"]
async fn test_validate_completion_endpoint() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Test validate-completion endpoint
    let response = post_json(
        &ctx.base_url,
        "/ai/validate-completion",
        &json!({
            "projectId": project_id,
            "taskId": "test-task-id",
            "taskTitle": "Implement login endpoint",
            "taskDescription": "Create POST /api/login endpoint with email/password validation",
            "linkedScenarios": [
                "When user provides valid credentials, then they receive an auth token",
                "When user provides invalid credentials, then they receive an error"
            ]
        }),
    )
    .await;

    assert!(
        response.status() == 200 || response.status() == 422 || response.status() == 500,
        "Expected 200, 422, or 500, got {}",
        response.status()
    );

    // Only parse JSON for successful responses
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["success"].is_boolean());
    }
}

#[tokio::test]
#[ignore = "AI endpoints moved to frontend in Phase 6 - backend routes removed"]
async fn test_validate_completion_empty_scenarios() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Test with empty scenarios array
    let response = post_json(
        &ctx.base_url,
        "/ai/validate-completion",
        &json!({
            "projectId": project_id,
            "taskId": "test-task-id",
            "taskTitle": "Implement feature",
            "taskDescription": "Some description",
            "linkedScenarios": []
        }),
    )
    .await;

    assert!(
        response.status() == 200 || response.status() == 422 || response.status() == 500,
        "Expected 200, 422, or 500, got {}",
        response.status()
    );
}

#[tokio::test]
#[ignore = "AI endpoints moved to frontend in Phase 6 - backend routes removed"]
async fn test_ai_endpoints_route_correctly() {
    let ctx = setup_test_server().await;

    // Test that all AI endpoints are mounted at /ai/*
    let endpoints = vec![
        "/ai/analyze-prd",
        "/ai/generate-spec",
        "/ai/suggest-tasks",
        "/ai/refine-spec",
        "/ai/validate-completion",
    ];

    for endpoint in endpoints {
        // GET should not be allowed (these are POST endpoints)
        let response = get(&ctx.base_url, endpoint).await;

        // Should return 405 Method Not Allowed or 404 if routing is wrong
        // We just want to verify the route exists
        assert_ne!(
            response.status(),
            404,
            "Endpoint {} not found - routing may be incorrect",
            endpoint
        );
    }
}
