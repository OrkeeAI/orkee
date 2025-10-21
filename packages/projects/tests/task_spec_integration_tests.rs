// ABOUTME: Integration tests for Task-Spec Integration API endpoints
// ABOUTME: Tests task-requirement linking, validation, task generation, and orphan detection

mod common;

use common::{create_test_project, get, post_json, setup_test_server};
use serde_json::json;

// Use the default "tag-main" tag created by migrations
const DEFAULT_TAG_ID: &str = "tag-main";

// Helper function to create a test task
async fn create_test_task(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    title: &str,
) -> String {
    let task_id = nanoid::nanoid!(8);
    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    sqlx::query(
        "INSERT INTO tasks (id, project_id, tag_id, title, description, status, priority, complexity_score, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&task_id)
    .bind(project_id)
    .bind(DEFAULT_TAG_ID)
    .bind(title)
    .bind("Test task description")
    .bind("pending")
    .bind("medium")
    .bind(5)
    .bind(&created_at)
    .bind(&created_at)
    .execute(pool)
    .await
    .expect("Failed to create test task");
    task_id
}

// Helper function to create a test capability
async fn create_test_capability(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    name: &str,
) -> String {
    let cap_id = nanoid::nanoid!(8);
    sqlx::query(
        "INSERT INTO spec_capabilities (id, project_id, name, spec_markdown, status) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&cap_id)
    .bind(project_id)
    .bind(name)
    .bind(format!("# {}\n\nTest capability", name))
    .bind("active")
    .execute(pool)
    .await
    .expect("Failed to create test capability");
    cap_id
}

// Helper function to create a test requirement
async fn create_test_requirement(
    pool: &sqlx::SqlitePool,
    capability_id: &str,
    name: &str,
) -> String {
    let req_id = nanoid::nanoid!(8);
    sqlx::query(
        "INSERT INTO spec_requirements (id, capability_id, name, content_markdown) VALUES (?, ?, ?, ?)",
    )
    .bind(&req_id)
    .bind(capability_id)
    .bind(name)
    .bind("Test requirement description")
    .execute(pool)
    .await
    .expect("Failed to create test requirement");
    req_id
}

#[tokio::test]
async fn test_link_task_to_requirement() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;
    let task_id = create_test_task(&ctx.pool, &project_id, "Test Task").await;

    // Create a capability and requirement
    let cap_id = create_test_capability(&ctx.pool, &project_id, "Test Capability").await;
    let req_id = create_test_requirement(&ctx.pool, &cap_id, "Test Requirement").await;

    // Link task to requirement
    let response = post_json(
        &ctx.base_url,
        &format!("/tasks/{}/link-spec", task_id),
        &json!({
            "requirementId": req_id
        }),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
}

#[tokio::test]
async fn test_get_task_spec_links() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;
    let task_id = create_test_task(&ctx.pool, &project_id, "Test Task").await;

    // Create a capability and requirement, then link them
    let cap_id = create_test_capability(&ctx.pool, &project_id, "Test Capability").await;
    let req_id = create_test_requirement(&ctx.pool, &cap_id, "Test Requirement").await;

    // Link task to requirement
    sqlx::query("INSERT INTO task_spec_links (task_id, requirement_id) VALUES (?, ?)")
        .bind(&task_id)
        .bind(&req_id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to create link");

    // Get spec links for task
    let response = get(&ctx.base_url, &format!("/tasks/{}/spec-links", task_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_get_task_spec_links_empty() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;
    let task_id = create_test_task(&ctx.pool, &project_id, "Orphan Task").await;

    // Get spec links for task without any links
    let response = get(&ctx.base_url, &format!("/tasks/{}/spec-links", task_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_validate_task_against_spec() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;
    let task_id = create_test_task(&ctx.pool, &project_id, "Test Task").await;

    // Validate task against spec (should work even without links)
    let response = post_json(
        &ctx.base_url,
        &format!("/tasks/{}/validate-spec", task_id),
        &json!({}),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
}

#[tokio::test]
async fn test_suggest_spec_from_task() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;
    let task_id = create_test_task(&ctx.pool, &project_id, "Test Task").await;

    // Suggest spec from task (placeholder endpoint)
    let response = post_json(
        &ctx.base_url,
        &format!("/tasks/{}/suggest-spec", task_id),
        &json!({}),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    // Should return suggestion structure with placeholder data
    assert!(body["data"]["note"].is_string());
}

#[tokio::test]
async fn test_generate_tasks_from_spec() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a capability with requirements
    let cap_id = create_test_capability(&ctx.pool, &project_id, "Test Capability").await;
    let _req_id = create_test_requirement(&ctx.pool, &cap_id, "Test Requirement").await;

    // Generate tasks from spec
    let response = post_json(
        &ctx.base_url,
        &format!("/{}/tasks/generate-from-spec", project_id),
        &json!({
            "capabilityId": cap_id,
            "tagId": DEFAULT_TAG_ID
        }),
    )
    .await;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["taskIds"].is_array());
    assert!(body["data"]["count"].is_number());
}

#[tokio::test]
async fn test_find_orphan_tasks() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create some orphan tasks (tasks without spec links)
    let _task1 = create_test_task(&ctx.pool, &project_id, "Orphan Task 1").await;
    let _task2 = create_test_task(&ctx.pool, &project_id, "Orphan Task 2").await;

    // Create a linked task
    let task3 = create_test_task(&ctx.pool, &project_id, "Linked Task").await;
    let cap_id = create_test_capability(&ctx.pool, &project_id, "Test Capability").await;
    let req_id = create_test_requirement(&ctx.pool, &cap_id, "Test Requirement").await;
    sqlx::query("INSERT INTO task_spec_links (task_id, requirement_id) VALUES (?, ?)")
        .bind(&task3)
        .bind(&req_id)
        .execute(&ctx.pool)
        .await
        .unwrap();

    // Find orphan tasks
    let response = get(&ctx.base_url, &format!("/{}/tasks/orphans", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["data"].is_array());
    // Should return 2 orphan tasks (task1 and task2, but not task3)
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"]["pagination"]["totalItems"], 2);
}

#[tokio::test]
async fn test_find_orphan_tasks_empty() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Find orphan tasks in empty project
    let response = get(&ctx.base_url, &format!("/{}/tasks/orphans", project_id)).await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_find_orphan_tasks_with_pagination() {
    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create 5 orphan tasks
    for i in 1..=5 {
        create_test_task(&ctx.pool, &project_id, &format!("Orphan Task {}", i)).await;
    }

    // Get first page with limit=2
    let response = get(
        &ctx.base_url,
        &format!("/{}/tasks/orphans?limit=2", project_id),
    )
    .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"]["pagination"]["totalItems"], 5);
    assert_eq!(body["data"]["pagination"]["page"], 1);
}
