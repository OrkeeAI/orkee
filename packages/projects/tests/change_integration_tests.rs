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

#[tokio::test]
async fn test_concurrent_change_creation_no_duplicates() {
    use orkee_projects::api::ai_handlers::PRDAnalysisData;
    use orkee_projects::openspec::change_builder::create_change_from_analysis;
    use std::collections::HashSet;

    let ctx = setup_test_server().await;
    let project_id = create_test_project(&ctx.pool, "Test Project", "/test/path").await;

    // Create a PRD first
    let prd_response = post_json(
        &ctx.base_url,
        &format!("/{}/prds", project_id),
        &json!({
            "title": "Test PRD",
            "contentMarkdown": "# Test PRD\n\nThis is a test PRD",
            "createdBy": "test-user"
        }),
    )
    .await;

    let prd_body: serde_json::Value = prd_response.json().await.unwrap();
    let prd_id = prd_body["data"]["id"].as_str().unwrap().to_string();

    // Create 5 concurrent changes with the same verb
    // (SQLite has limited concurrency, so we test with a smaller number)
    let mut handles = vec![];
    for i in 0..5 {
        let pool = ctx.pool.clone();
        let project_id = project_id.clone();
        let prd_id = prd_id.clone();

        let handle = tokio::spawn(async move {
            // Add small jitter to reduce likelihood of exact simultaneous execution
            if i > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(i as u64 * 2)).await;
            }

            // Create analysis data inside each task to avoid clone issues
            let analysis = PRDAnalysisData {
                summary: "Add new authentication feature".to_string(),
                capabilities: vec![],
                suggested_tasks: vec![],
                dependencies: None,
                technical_considerations: None,
            };

            create_change_from_analysis(&pool, &project_id, &prd_id, &analysis, "test-user").await
        });

        handles.push(handle);
    }

    // Wait for all to complete
    let mut changes = vec![];
    for handle in handles {
        match handle.await {
            Ok(Ok(change)) => changes.push(change),
            Ok(Err(e)) => panic!("Change creation failed: {:?}", e),
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    assert_eq!(changes.len(), 5, "All 5 changes should be created");

    // Verify all change numbers are unique
    let change_numbers: HashSet<_> = changes
        .iter()
        .filter_map(|c| c.change_number)
        .collect();

    assert_eq!(
        change_numbers.len(),
        5,
        "All 5 change numbers should be unique"
    );

    // Verify all change numbers are in the range [1, 5]
    for num in change_numbers.iter() {
        assert!(*num >= 1 && *num <= 5, "Change number should be between 1 and 5");
    }

    // Verify all have the same verb prefix
    let verb_prefixes: HashSet<_> = changes
        .iter()
        .filter_map(|c| c.verb_prefix.as_ref())
        .collect();

    assert_eq!(
        verb_prefixes.len(),
        1,
        "All changes should have the same verb prefix"
    );
    assert!(verb_prefixes.contains(&"add".to_string()));
}
