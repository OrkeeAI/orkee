// ABOUTME: API handlers for task decomposition (Phase 4 CCPM)
// ABOUTME: Endpoints for breaking down epics into tasks with dependency analysis

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use orkee_ideate::{DecomposeEpicInput, ParentTask, TaskDecomposer};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use orkee_projects::DbState;

/// Request body for updating parent tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParentTasksRequest {
    pub parent_tasks: Vec<ParentTask>,
}

/// POST /api/projects/:project_id/epics/:epic_id/decompose
pub async fn decompose_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
    Json(input): Json<DecomposeEpicInput>,
) -> impl IntoResponse {
    info!("Decomposing epic {} for project {}", epic_id, project_id);

    // Validate epic_id matches
    if input.epic_id != epic_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Epic ID in path does not match request body"
            })),
        )
            .into_response();
    }

    let decomposer = TaskDecomposer::new(db.pool.clone());

    // Get current user (placeholder - you'd get this from auth)
    let user_id = "default_user"; // TODO: Get from auth context

    match decomposer.decompose_epic(&project_id, user_id, input).await {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": result
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to decompose epic: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("Failed to decompose epic: {}", e)}))).into_response()
        }
    }
}

/// POST /api/projects/:project_id/epics/:epic_id/analyze-work
pub async fn analyze_work_streams(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Analyzing work streams for epic {} in project {}",
        epic_id, project_id
    );

    let decomposer = TaskDecomposer::new(db.pool.clone());

    match decomposer.analyze_work_streams(&epic_id).await {
        Ok(analysis) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": analysis
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to analyze work streams: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("Failed to analyze work streams: {}", e)}))).into_response()
        }
    }
}

/// GET /api/projects/:project_id/epics/:epic_id/tasks
pub async fn get_epic_tasks(
    State(db): State<DbState>,
    Path((_project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Fetching tasks for epic {}", epic_id);

    let rows = match sqlx::query(
        "SELECT * FROM tasks WHERE epic_id = ? ORDER BY position, created_at",
    )
    .bind(&epic_id)
    .fetch_all(&db.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to fetch tasks for epic: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("Failed to fetch tasks: {}", e)}))).into_response();
        }
    };

    // Use TaskStorage's row conversion helper
    let storage = orkee_tasks::storage::TaskStorage::new(db.pool.clone());
    let mut result_tasks = Vec::new();

    for row in rows.iter() {
        // We need to get the task by ID since row_to_task_sync is private
        // For now, just get all task IDs and fetch them
        let task_id: String = match sqlx::Row::try_get(row, "id") {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get task ID: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("Failed to get task ID: {}", e)}))).into_response();
            }
        };

        match storage.get_task(&task_id).await {
            Ok(task) => result_tasks.push(task),
            Err(e) => {
                error!("Failed to get task {}: {:?}", task_id, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": format!("Failed to get task: {}", e)}))).into_response();
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": result_tasks
        })),
    )
        .into_response()
}

/// POST /api/projects/:project_id/epics/:epic_id/decompose-phase1
/// Phase 1 of two-phase task generation: Generate high-level parent tasks only
pub async fn decompose_phase1(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Phase 1: Generating parent tasks for epic {} in project {}",
        epic_id, project_id
    );

    let decomposer = TaskDecomposer::new(db.pool.clone());

    // TODO: Get codebase context if available
    // For now, pass None
    let codebase_context = None;

    match decomposer
        .generate_parent_tasks(&epic_id, codebase_context)
        .await
    {
        Ok(parent_tasks) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "parent_tasks": parent_tasks,
                    "count": parent_tasks.len()
                }
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to generate parent tasks: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to generate parent tasks: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/projects/:project_id/epics/:epic_id/parent-tasks
/// Get parent tasks for review (stored in epic)
pub async fn get_parent_tasks(
    State(db): State<DbState>,
    Path((_project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Fetching parent tasks for epic {}", epic_id);

    let decomposer = TaskDecomposer::new(db.pool.clone());

    match decomposer.get_stored_parent_tasks(&epic_id).await {
        Ok(parent_tasks) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "parent_tasks": parent_tasks,
                    "count": parent_tasks.len()
                }
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch parent tasks: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to fetch parent tasks: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// PUT /api/projects/:project_id/epics/:epic_id/parent-tasks
/// Update parent tasks before expansion (user review/editing)
pub async fn update_parent_tasks(
    State(db): State<DbState>,
    Path((_project_id, epic_id)): Path<(String, String)>,
    Json(request): Json<UpdateParentTasksRequest>,
) -> impl IntoResponse {
    info!(
        "Updating parent tasks for epic {} ({} tasks)",
        epic_id,
        request.parent_tasks.len()
    );

    let decomposer = TaskDecomposer::new(db.pool.clone());

    match decomposer
        .save_parent_tasks(&epic_id, &request.parent_tasks)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "parent_tasks": request.parent_tasks,
                    "count": request.parent_tasks.len()
                }
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to update parent tasks: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to update parent tasks: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// POST /api/projects/:project_id/epics/:epic_id/decompose-phase2
/// Phase 2 of two-phase task generation: Expand parent tasks into detailed subtasks
pub async fn decompose_phase2(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Phase 2: Expanding parent tasks to subtasks for epic {} in project {}",
        epic_id, project_id
    );

    let decomposer = TaskDecomposer::new(db.pool.clone());

    // Get current user (placeholder - you'd get this from auth)
    let user_id = "default_user"; // TODO: Get from auth context

    // Get stored parent tasks
    let parent_tasks = match decomposer.get_stored_parent_tasks(&epic_id).await {
        Ok(tasks) => tasks,
        Err(e) => {
            error!("Failed to fetch parent tasks for expansion: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to fetch parent tasks: {}", e)
                })),
            )
                .into_response();
        }
    };

    if parent_tasks.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "No parent tasks found. Run decompose-phase1 first."
            })),
        )
            .into_response();
    }

    // TODO: Get codebase context if available
    let codebase_context = None;

    match decomposer
        .expand_to_subtasks(
            &project_id,
            user_id,
            &epic_id,
            &parent_tasks,
            codebase_context,
        )
        .await
    {
        Ok(tasks) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "tasks": tasks,
                    "count": tasks.len(),
                    "parent_tasks_count": parent_tasks.len()
                }
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to expand parent tasks to subtasks: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to expand to subtasks: {}", e)
                })),
            )
                .into_response()
        }
    }
}
