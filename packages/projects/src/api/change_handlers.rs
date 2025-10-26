// ABOUTME: HTTP request handlers for spec change operations
// ABOUTME: Handles CRUD operations for spec changes and deltas

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::{
    bad_request, created_or_internal_error, ok_or_internal_error, ok_or_not_found,
};
use super::validation;
use crate::db::DbState;
use crate::openspec::db as openspec_db;
use crate::openspec::types::{ChangeStatus, DeltaType};
use crate::pagination::{PaginatedResponse, PaginationParams};

/// List all changes for a project
pub async fn list_changes(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Listing changes for project: {} (page: {})",
        project_id,
        pagination.page()
    );

    let result = openspec_db::get_spec_changes_by_project_paginated(
        &db.pool,
        &project_id,
        Some(pagination.limit()),
        Some(pagination.offset()),
    )
    .await
    .map(|(changes, total)| PaginatedResponse::new(changes, &pagination, total));

    ok_or_internal_error(result, "Failed to list changes")
}

/// Get a single change by ID
pub async fn get_change(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting change: {}", change_id);

    let result = openspec_db::get_spec_change(&db.pool, &change_id).await;
    ok_or_not_found(result, "Change not found")
}

/// Request body for creating a change
#[derive(Deserialize)]
pub struct CreateChangeRequest {
    #[serde(rename = "prdId")]
    pub prd_id: Option<String>,
    #[serde(rename = "proposalMarkdown")]
    pub proposal_markdown: String,
    #[serde(rename = "tasksMarkdown")]
    pub tasks_markdown: String,
    #[serde(rename = "designMarkdown")]
    pub design_markdown: Option<String>,
    #[serde(rename = "createdBy")]
    pub created_by: String,
}

/// Create a new change
pub async fn create_change(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateChangeRequest>,
) -> impl IntoResponse {
    info!("Creating change for project: {}", project_id);

    // Validate and sanitize inputs
    let validated_proposal =
        match validation::validate_proposal_markdown(&request.proposal_markdown) {
            Ok(v) => v,
            Err(e) => return bad_request(e, "Invalid proposal markdown"),
        };

    let validated_tasks = match validation::validate_tasks_markdown(&request.tasks_markdown) {
        Ok(v) => v,
        Err(e) => return bad_request(e, "Invalid tasks markdown"),
    };

    let validated_design =
        match validation::validate_design_markdown(request.design_markdown.as_deref()) {
            Ok(v) => v,
            Err(e) => return bad_request(e, "Invalid design markdown"),
        };

    let validated_user = match validation::validate_user_id(&request.created_by) {
        Ok(v) => v,
        Err(e) => return bad_request(e, "Invalid user ID"),
    };

    // TODO: Validate user exists in database once proper auth is implemented
    // For now, we accept any validated user ID

    let result = openspec_db::create_spec_change(
        &db.pool,
        &project_id,
        request.prd_id.as_deref(),
        &validated_proposal,
        &validated_tasks,
        validated_design.as_deref(),
        &validated_user,
    )
    .await;

    created_or_internal_error(result, "Failed to create change")
}

/// Request body for updating change status
#[derive(Deserialize)]
pub struct UpdateChangeStatusRequest {
    pub status: ChangeStatus,
    #[serde(rename = "approvedBy")]
    pub approved_by: Option<String>,
    pub notes: Option<String>,
}

/// Update change status
pub async fn update_change_status(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
    Json(request): Json<UpdateChangeStatusRequest>,
) -> impl IntoResponse {
    info!("Updating change status: {}", change_id);

    // Validate approvedBy if provided
    let validated_approved_by = match &request.approved_by {
        Some(user_id) => match validation::validate_user_id(user_id) {
            Ok(v) => Some(v),
            Err(e) => return bad_request(e, "Invalid approver user ID"),
        },
        None => None,
    };

    let result = openspec_db::update_spec_change_status(
        &db.pool,
        &change_id,
        request.status,
        validated_approved_by.as_deref(),
    )
    .await;

    ok_or_internal_error(result, "Failed to update change status")
}

/// Get all deltas for a change
pub async fn get_change_deltas(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Getting deltas for change: {} (page: {})",
        change_id,
        pagination.page()
    );

    let result = openspec_db::get_deltas_by_change_paginated(
        &db.pool,
        &change_id,
        Some(pagination.limit()),
        Some(pagination.offset()),
    )
    .await
    .map(|(deltas, total)| PaginatedResponse::new(deltas, &pagination, total));

    ok_or_internal_error(result, "Failed to get change deltas")
}

/// Request body for creating a delta
#[derive(Deserialize)]
pub struct CreateDeltaRequest {
    #[serde(rename = "capabilityId")]
    pub capability_id: Option<String>,
    #[serde(rename = "capabilityName")]
    pub capability_name: String,
    #[serde(rename = "deltaType")]
    pub delta_type: DeltaType,
    #[serde(rename = "deltaMarkdown")]
    pub delta_markdown: String,
    pub position: i32,
}

/// Create a new delta for a change
pub async fn create_delta(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
    Json(request): Json<CreateDeltaRequest>,
) -> impl IntoResponse {
    info!("Creating delta for change: {}", change_id);

    // Validate and sanitize inputs
    let validated_name = match validation::validate_capability_name(&request.capability_name) {
        Ok(v) => v,
        Err(e) => return bad_request(e, "Invalid capability name"),
    };

    let validated_markdown = match validation::validate_delta_markdown(&request.delta_markdown) {
        Ok(v) => v,
        Err(e) => return bad_request(e, "Invalid delta markdown"),
    };

    let result = openspec_db::create_spec_delta(
        &db.pool,
        &change_id,
        request.capability_id.as_deref(),
        &validated_name,
        request.delta_type,
        &validated_markdown,
        request.position,
    )
    .await;

    created_or_internal_error(result, "Failed to create delta")
}

/// Get all tasks for a change (parsed from tasks_markdown)
pub async fn get_change_tasks(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting tasks for change: {}", change_id);

    let result = openspec_db::get_change_tasks(&db.pool, &change_id).await;
    ok_or_internal_error(result, "Failed to get change tasks")
}

/// Request body for updating a task
#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(rename = "completedBy")]
    pub completed_by: Option<String>,
}

/// Update a task's completion status
pub async fn update_task(
    State(db): State<DbState>,
    Path((_project_id, _change_id, task_id)): Path<(String, String, String)>,
    Json(request): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    info!(
        "Updating task: {} (completed: {})",
        task_id, request.is_completed
    );

    // Validate completedBy if provided
    let validated_completed_by = match &request.completed_by {
        Some(user_id) => match validation::validate_user_id(user_id) {
            Ok(v) => Some(v),
            Err(e) => return bad_request(e, "Invalid completed_by user ID"),
        },
        None => None,
    };

    let result = openspec_db::update_change_task(
        &db.pool,
        &task_id,
        request.is_completed,
        validated_completed_by.as_deref(),
    )
    .await;

    ok_or_internal_error(result, "Failed to update task")
}

/// Parse tasks from a change's tasks_markdown and store them
pub async fn parse_change_tasks(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Parsing tasks for change: {}", change_id);

    let result = openspec_db::parse_and_store_change_tasks(&db.pool, &change_id).await;
    ok_or_internal_error(result, "Failed to parse change tasks")
}

/// Request body for bulk task updates
#[derive(Deserialize)]
pub struct BulkUpdateTasksRequest {
    pub tasks: Vec<TaskUpdate>,
}

#[derive(Deserialize)]
pub struct TaskUpdate {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(rename = "completedBy")]
    pub completed_by: Option<String>,
}

/// Update multiple tasks at once
pub async fn bulk_update_tasks(
    State(db): State<DbState>,
    Path((_project_id, _change_id)): Path<(String, String)>,
    Json(request): Json<BulkUpdateTasksRequest>,
) -> impl IntoResponse {
    info!("Bulk updating {} tasks", request.tasks.len());

    // Validate all completedBy fields in the batch
    let mut validated_tasks = Vec::with_capacity(request.tasks.len());
    for task_update in request.tasks {
        let validated_completed_by = match &task_update.completed_by {
            Some(user_id) => match validation::validate_user_id(user_id) {
                Ok(v) => Some(v),
                Err(e) => {
                    return bad_request(
                        e,
                        &format!("Invalid user ID in task {}", task_update.task_id),
                    )
                }
            },
            None => None,
        };

        validated_tasks.push(TaskUpdate {
            task_id: task_update.task_id,
            is_completed: task_update.is_completed,
            completed_by: validated_completed_by,
        });
    }

    let result = openspec_db::bulk_update_change_tasks(&db.pool, validated_tasks).await;
    ok_or_internal_error(result, "Failed to bulk update tasks")
}
