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
use orkee_projects::pagination::{PaginatedResponse, PaginationParams};
use orkee_projects::DbState;
use tasks::{TaskCreateInput, TaskPriority, TaskStatus, TaskUpdateInput};

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
