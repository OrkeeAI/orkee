// ABOUTME: HTTP request handlers for task operations
// ABOUTME: Handles CRUD operations for tasks with database integration

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use super::auth::CurrentUser;
use super::response::{created_or_internal_error, ok_or_internal_error};
use orkee_ideate::{AppendProgressInput, ExecutionTracker};
use orkee_projects::pagination::{PaginatedResponse, PaginationParams};
use orkee_projects::DbState;
use orkee_tasks::{TaskCreateInput, TaskPriority, TaskStatus, TaskUpdateInput};
use serde::Serialize;

/// Helper function to parse ISO 8601 date string
fn parse_due_date(date_str: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(date_str)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// List all tasks for a project
pub async fn list_tasks(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Listing tasks for project: {} (page: {})",
        project_id,
        pagination.page()
    );

    let result = db
        .task_storage
        .list_tasks_paginated(
            &project_id,
            Some(pagination.limit()),
            Some(pagination.offset()),
        )
        .await
        .map(|(tasks, total)| PaginatedResponse::new(tasks, &pagination, total));

    ok_or_internal_error(result, "Failed to list tasks")
}

/// Get a single task by ID
pub async fn get_task(
    State(db): State<DbState>,
    Path((_project_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting task: {}", task_id);

    let result = db.task_storage.get_task(&task_id).await;
    ok_or_internal_error(result, "Failed to get task")
}

/// Request body for creating a task
#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    #[serde(rename = "assignedAgentId")]
    pub assigned_agent_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub position: Option<i32>,
    pub dependencies: Option<Vec<String>>,
    #[serde(rename = "dueDate")]
    pub due_date: Option<String>,
    #[serde(rename = "estimatedHours")]
    pub estimated_hours: Option<f64>,
    #[serde(rename = "complexityScore")]
    pub complexity_score: Option<i32>,
    pub details: Option<String>,
    #[serde(rename = "testStrategy")]
    pub test_strategy: Option<String>,
    #[serde(rename = "acceptanceCriteria")]
    pub acceptance_criteria: Option<String>,
    pub prompt: Option<String>,
    pub context: Option<String>,
    #[serde(rename = "tagId")]
    pub tag_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
}

/// Create a new task
pub async fn create_task(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    current_user: CurrentUser,
    Json(request): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    info!(
        "Creating task '{}' for project: {}",
        request.title, project_id
    );

    let due_date = request.due_date.as_deref().and_then(parse_due_date);

    let input = TaskCreateInput {
        title: request.title,
        description: request.description,
        status: request.status,
        priority: request.priority,
        assigned_agent_id: request.assigned_agent_id,
        parent_id: request.parent_id,
        position: request.position,
        dependencies: request.dependencies,
        due_date,
        estimated_hours: request.estimated_hours,
        complexity_score: request.complexity_score,
        details: request.details,
        test_strategy: request.test_strategy,
        acceptance_criteria: request.acceptance_criteria,
        prompt: request.prompt,
        context: request.context,
        tag_id: request.tag_id,
        tags: request.tags,
        category: request.category,
        epic_id: None,
        parallel_group: None,
        depends_on: None,
        conflicts_with: None,
        task_type: None,
        size_estimate: None,
        technical_details: None,
        effort_hours: None,
        can_parallel: None,
    };

    let result = db
        .task_storage
        .create_task(&project_id, &current_user.id, input)
        .await;

    created_or_internal_error(result, "Failed to create task")
}

/// Request body for updating a task
#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    #[serde(rename = "assignedAgentId")]
    pub assigned_agent_id: Option<String>,
    pub position: Option<i32>,
    pub dependencies: Option<Vec<String>>,
    #[serde(rename = "dueDate")]
    pub due_date: Option<String>,
    #[serde(rename = "estimatedHours")]
    pub estimated_hours: Option<f64>,
    #[serde(rename = "actualHours")]
    pub actual_hours: Option<f64>,
    #[serde(rename = "complexityScore")]
    pub complexity_score: Option<i32>,
    pub details: Option<String>,
    #[serde(rename = "testStrategy")]
    pub test_strategy: Option<String>,
    #[serde(rename = "acceptanceCriteria")]
    pub acceptance_criteria: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
}

/// Update an existing task
pub async fn update_task(
    State(db): State<DbState>,
    Path((_project_id, task_id)): Path<(String, String)>,
    Json(request): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    info!("Updating task: {}", task_id);

    let due_date = request.due_date.as_deref().and_then(parse_due_date);

    let input = TaskUpdateInput {
        title: request.title,
        description: request.description,
        status: request.status,
        priority: request.priority,
        assigned_agent_id: request.assigned_agent_id,
        position: request.position,
        dependencies: request.dependencies,
        due_date,
        estimated_hours: request.estimated_hours,
        actual_hours: request.actual_hours,
        complexity_score: request.complexity_score,
        details: request.details,
        test_strategy: request.test_strategy,
        acceptance_criteria: request.acceptance_criteria,
        tags: request.tags,
        category: request.category,
        epic_id: None,
        parallel_group: None,
        depends_on: None,
        conflicts_with: None,
        task_type: None,
        size_estimate: None,
        technical_details: None,
        effort_hours: None,
        can_parallel: None,
    };

    let result = db.task_storage.update_task(&task_id, input).await;
    ok_or_internal_error(result, "Failed to update task")
}

/// Delete a task
pub async fn delete_task(
    State(db): State<DbState>,
    Path((_project_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Deleting task: {}", task_id);

    let result = db.task_storage.delete_task(&task_id).await.map(|_| {
        serde_json::json!({
            "message": format!("Task {} deleted successfully", task_id)
        })
    });

    ok_or_internal_error(result, "Failed to delete task")
}

/// Response for task execution steps
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStep {
    step_number: usize,
    action: String,
    test_command: Option<String>,
    expected_output: String,
    estimated_minutes: u8,
}

/// Generate TDD execution steps for a task
pub async fn generate_task_steps(
    State(db): State<DbState>,
    Path((_project_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Generating execution steps for task: {}", task_id);

    // This is a placeholder - in a real implementation, this would use AI to generate
    // execution steps based on the task description
    // For now, we return a standard TDD workflow

    let steps = vec![
        TaskStep {
            step_number: 1,
            action: "Write failing test for the functionality".to_string(),
            test_command: Some("cargo test <test_name>".to_string()),
            expected_output: "Test fails as expected".to_string(),
            estimated_minutes: 5,
        },
        TaskStep {
            step_number: 2,
            action: "Create minimal implementation stub".to_string(),
            test_command: None,
            expected_output: "Function signature created".to_string(),
            estimated_minutes: 3,
        },
        TaskStep {
            step_number: 3,
            action: "Verify test still fails correctly".to_string(),
            test_command: Some("cargo test <test_name>".to_string()),
            expected_output: "Test fails with correct assertion message".to_string(),
            estimated_minutes: 2,
        },
        TaskStep {
            step_number: 4,
            action: "Implement core functionality".to_string(),
            test_command: None,
            expected_output: "Implementation complete".to_string(),
            estimated_minutes: 15,
        },
        TaskStep {
            step_number: 5,
            action: "Run test to verify success".to_string(),
            test_command: Some("cargo test <test_name>".to_string()),
            expected_output: "Test passes".to_string(),
            estimated_minutes: 2,
        },
        TaskStep {
            step_number: 6,
            action: "Refactor if needed".to_string(),
            test_command: Some("cargo test".to_string()),
            expected_output: "All tests still pass".to_string(),
            estimated_minutes: 5,
        },
        TaskStep {
            step_number: 7,
            action: "Commit changes".to_string(),
            test_command: Some("git add . && git commit -m 'message'".to_string()),
            expected_output: "Changes committed".to_string(),
            estimated_minutes: 2,
        },
    ];

    ok_or_internal_error::<Vec<TaskStep>, orkee_storage::StorageError>(
        Ok(steps),
        "Failed to generate execution steps",
    )
}

/// Append progress to a task (append-only)
pub async fn append_task_progress(
    State(db): State<DbState>,
    Path((_project_id, task_id)): Path<(String, String)>,
    _user: CurrentUser,
    Json(input): Json<AppendProgressInput>,
) -> impl IntoResponse {
    info!("Appending progress to task: {}", task_id);

    let tracker = ExecutionTracker::new(db.pool.clone());
    let result = tracker.append_progress(input).await;

    ok_or_internal_error(result, "Failed to append progress")
}

/// Get validation history for a task
pub async fn get_task_validation_history(
    State(db): State<DbState>,
    Path((_project_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting validation history for task: {}", task_id);

    let tracker = ExecutionTracker::new(db.pool.clone());
    let result = tracker.get_task_validation_history(&task_id).await;

    ok_or_internal_error(result, "Failed to get validation history")
}

/// Get execution checkpoints for a task
pub async fn get_task_checkpoints(
    State(db): State<DbState>,
    Path((_project_id, task_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting checkpoints for task: {}", task_id);

    // Get task to find its epic
    let task_result = db.task_storage.get_task(&task_id).await;

    let task = match task_result {
        Ok(task) => task,
        Err(e) => {
            return ok_or_internal_error::<Vec<orkee_ideate::ExecutionCheckpoint>, orkee_storage::StorageError>(
                Err(e),
                "Failed to get task",
            )
        }
    };

    // If task has an epic_id, get epic checkpoints
    if let Some(ref epic_id) = task.epic_id {
        let tracker = ExecutionTracker::new(db.pool.clone());
        let result = tracker.get_epic_checkpoints(epic_id).await;
        ok_or_internal_error(result, "Failed to get task checkpoints")
    } else {
        // Task not part of an epic, no checkpoints
        ok_or_internal_error::<Vec<orkee_ideate::ExecutionCheckpoint>, orkee_storage::StorageError>(
            Ok(Vec::new()),
            "No checkpoints",
        )
    }
}
