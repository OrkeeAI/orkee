// ABOUTME: API handlers for task decomposition (Phase 4 CCPM)
// ABOUTME: Endpoints for breaking down epics into tasks with dependency analysis

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ideate::{DecomposeEpicInput, TaskDecomposer};
use tracing::{error, info};

use orkee_projects::DbState;

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
    let storage = tasks::storage::TaskStorage::new(db.pool.clone());
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
