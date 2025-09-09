use crate::tests::test_helpers;
use crate::tools::{tools_call, tools_list, CallToolRequest};
use rstest::rstest;
use serde_json::{json, Value};

#[tokio::test]
async fn test_tools_list() {
    let result = tools_list(None, None).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let tools = &response.tools;

    // Verify we have all expected tools
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    // The implementation has two tools: projects and project_manage
    assert!(tool_names.contains(&"projects".to_string()));
    assert!(tool_names.contains(&"project_manage".to_string()));
    assert_eq!(tools.len(), 2);
}

#[tokio::test]
async fn test_list_projects_tool() {
    // Create test context with isolated in-memory storage
    let context = test_helpers::create_test_context().await.unwrap();

    let request = CallToolRequest {
        name: "projects".to_string(),
        arguments: Some(json!({"action": "list"})),
    };

    let result = tools_call(Some(request), Some(context.clone())).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.content.is_empty());
}

#[tokio::test]
async fn test_create_project_tool() {
    // Create test context with isolated in-memory storage
    let context = test_helpers::create_test_context().await.unwrap();

    let request = CallToolRequest {
        name: "project_manage".to_string(),
        arguments: Some(json!({
            "action": "create",
            "name": "Test Project",
            "projectRoot": "/tmp/test-project",
            "description": "A test project",
            "tags": ["test", "mcp"]
        })),
    };

    let result = tools_call(Some(request), Some(context.clone())).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.content.is_empty());
    let content = &response.content[0].text;
    assert!(content.contains("Test Project"));
}

#[tokio::test]
async fn test_invalid_tool_name() {
    // Create test context with isolated in-memory storage
    let context = test_helpers::create_test_context().await.unwrap();

    let request = CallToolRequest {
        name: "non_existent_tool".to_string(),
        arguments: Some(json!({})),
    };

    let result = tools_call(Some(request), Some(context.clone())).await;
    assert!(result.is_ok()); // Returns Ok with error in content

    let response = result.unwrap();
    assert!(!response.content.is_empty());
    let content = &response.content[0].text;
    assert!(content.contains("Unknown tool"));
    assert_eq!(response.is_error, Some(true));
}

#[tokio::test]
async fn test_missing_required_arguments() {
    // Create test context with isolated in-memory storage
    let context = test_helpers::create_test_context().await.unwrap();

    let request = CallToolRequest {
        name: "project_manage".to_string(),
        arguments: Some(json!({
            "action": "create"
            // Missing required 'name' and 'projectRoot' for create action
        })),
    };

    let result = tools_call(Some(request), Some(context.clone())).await;
    assert!(result.is_ok()); // Returns Ok with error message in content

    let response = result.unwrap();
    let content = &response.content[0].text;
    assert!(content.contains("error") || content.contains("required"));
}

#[rstest]
#[case("projects", json!({"action": "list"}))]
#[case("projects", json!({"action": "get", "id": "test-id"}))]
#[case("project_manage", json!({"action": "create", "name": "Test", "projectRoot": "/tmp/test"}))]
#[case("project_manage", json!({"action": "update", "id": "test-id", "name": "Updated"}))]
#[tokio::test]
async fn test_tool_parameter_validation(#[case] tool_name: &str, #[case] arguments: Value) {
    // Create test context with isolated in-memory storage
    let context = test_helpers::create_test_context().await.unwrap();

    let request = CallToolRequest {
        name: tool_name.to_string(),
        arguments: Some(arguments),
    };

    let result = tools_call(Some(request), Some(context.clone())).await;
    // These should all execute without panicking
    // Some may return errors due to missing projects, but shouldn't panic
    let _ = result;
}

#[tokio::test]
async fn test_update_project_tool() {
    // Create test context with isolated in-memory storage
    let context = test_helpers::create_test_context().await.unwrap();

    // First create a project
    let create_request = CallToolRequest {
        name: "project_manage".to_string(),
        arguments: Some(json!({
            "action": "create",
            "name": "Original Name",
            "projectRoot": "/tmp/original"
        })),
    };

    let create_result = tools_call(Some(create_request), Some(context.clone())).await;
    assert!(create_result.is_ok());

    // Extract the project ID from response
    let response = create_result.unwrap();
    let text = &response.content[0].text;

    // Parse the ID from the response (assumes it's in the text)
    let id_start = text.find("ID: ").map(|i| i + 4);
    let id = if let Some(start) = id_start {
        let end = text[start..].find(')').unwrap_or(8);
        &text[start..start + end]
    } else {
        "test-id" // Fallback
    };

    // Update the project
    let update_request = CallToolRequest {
        name: "project_manage".to_string(),
        arguments: Some(json!({
            "action": "update",
            "id": id,
            "name": "Updated Name",
            "description": "Updated description"
        })),
    };

    let update_result = tools_call(Some(update_request), Some(context.clone())).await;
    assert!(update_result.is_ok());
}

#[tokio::test]
async fn test_delete_project_tool() {
    // Create test context with isolated in-memory storage
    let context = test_helpers::create_test_context().await.unwrap();

    // First create a project to delete
    let create_request = CallToolRequest {
        name: "project_manage".to_string(),
        arguments: Some(json!({
            "action": "create",
            "name": "To Delete",
            "projectRoot": "/tmp/to-delete"
        })),
    };

    let create_result = tools_call(Some(create_request), Some(context.clone())).await;
    assert!(create_result.is_ok());

    // Extract the project ID
    let response = create_result.unwrap();
    let text = &response.content[0].text;

    let id_start = text.find("ID: ").map(|i| i + 4);
    let id = if let Some(start) = id_start {
        let end = text[start..].find(')').unwrap_or(36); // UUID is 36 chars
        &text[start..start + end]
    } else {
        "test-id"
    };

    // Delete the project
    let delete_request = CallToolRequest {
        name: "project_manage".to_string(),
        arguments: Some(json!({
            "action": "delete",
            "id": id
        })),
    };

    let delete_result = tools_call(Some(delete_request), Some(context.clone())).await;
    assert!(delete_result.is_ok());

    // Verify project is deleted by trying to get it
    let get_request = CallToolRequest {
        name: "projects".to_string(),
        arguments: Some(json!({
            "action": "get",
            "id": id
        })),
    };

    let get_result = tools_call(Some(get_request), Some(context.clone())).await;
    assert!(get_result.is_ok());
    let get_response = get_result.unwrap();
    let get_text = &get_response.content[0].text;
    assert!(get_text.contains("not found") || get_text.contains("No project"));
}
