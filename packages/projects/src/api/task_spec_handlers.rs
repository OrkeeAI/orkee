// ABOUTME: HTTP request handlers for task-spec integration operations
// ABOUTME: Handles linking tasks to requirements, validation, and task generation from specs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::response::ApiResponse;
use crate::db::DbState;
use crate::openspec::integration;

/// Link a task to a spec requirement
pub async fn link_task_to_requirement(
    State(db): State<DbState>,
    Path(task_id): Path<String>,
    Json(request): Json<LinkSpecRequest>,
) -> impl IntoResponse {
    info!(
        "Linking task {} to requirement {}",
        task_id, request.requirement_id
    );

    match integration::link_task_to_requirement(&db.pool, &task_id, &request.requirement_id).await {
        Ok(_) => (StatusCode::OK, ResponseJson(ApiResponse::success(true))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "Failed to link task to requirement: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// Request body for linking a task to a spec requirement
#[derive(Deserialize)]
pub struct LinkSpecRequest {
    #[serde(rename = "requirementId")]
    pub requirement_id: String,
}

/// Get all spec requirements linked to a task
pub async fn get_task_spec_links(
    State(db): State<DbState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting spec links for task: {}", task_id);

    match integration::get_task_requirements(&db.pool, &task_id).await {
        Ok(requirements) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success(requirements)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "Failed to get task spec links: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// Validate a task against its linked spec scenarios
pub async fn validate_task_against_spec(
    State(db): State<DbState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("Validating task {} against spec scenarios", task_id);

    match integration::validate_task_completion(&db.pool, &task_id).await {
        Ok(validation_result) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success(validation_result)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "Failed to validate task: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// AI suggest spec from task (placeholder for future AI integration)
pub async fn suggest_spec_from_task(Path(task_id): Path<String>) -> impl IntoResponse {
    info!("AI suggestion requested for task: {}", task_id);

    let response = SuggestSpecResponse {
        suggested_requirement: Some("AI-suggested requirement name".to_string()),
        suggested_content: Some("AI-suggested spec content based on task description".to_string()),
        confidence: 0.0,
        note: "AI integration not yet implemented".to_string(),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
}

/// Response for AI spec suggestion
#[derive(Serialize)]
pub struct SuggestSpecResponse {
    #[serde(rename = "suggestedRequirement")]
    pub suggested_requirement: Option<String>,
    #[serde(rename = "suggestedContent")]
    pub suggested_content: Option<String>,
    pub confidence: f32,
    pub note: String,
}

/// Request body for generating tasks from a spec
#[derive(Deserialize)]
pub struct GenerateTasksRequest {
    #[serde(rename = "capabilityId")]
    pub capability_id: String,
    #[serde(rename = "tagId")]
    pub tag_id: String,
}

/// Generate tasks from a spec capability
pub async fn generate_tasks_from_spec(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Json(request): Json<GenerateTasksRequest>,
) -> impl IntoResponse {
    info!(
        "Generating tasks from capability {} for project {}",
        request.capability_id, project_id
    );

    match integration::generate_tasks_from_capability(
        &db.pool,
        &request.capability_id,
        &project_id,
        &request.tag_id,
    )
    .await
    {
        Ok(task_ids) => {
            let response = GenerateTasksResponse {
                task_ids: task_ids.clone(),
                count: task_ids.len(),
            };
            (
                StatusCode::CREATED,
                ResponseJson(ApiResponse::success(response)),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "Failed to generate tasks from spec: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// Response for task generation
#[derive(Serialize)]
pub struct GenerateTasksResponse {
    #[serde(rename = "taskIds")]
    pub task_ids: Vec<String>,
    pub count: usize,
}

/// Find tasks without spec links (orphan tasks)
pub async fn find_orphan_tasks(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Finding orphan tasks for project: {}", project_id);

    // Query for tasks in this project that don't have any spec links
    let query = r#"
        SELECT t.id, t.title, t.status, t.priority, t.created_at
        FROM tasks t
        WHERE t.project_id = ?
        AND NOT EXISTS (
            SELECT 1 FROM task_spec_links tsl WHERE tsl.task_id = t.id
        )
        ORDER BY t.created_at DESC
    "#;

    match sqlx::query_as::<_, OrphanTask>(query)
        .bind(&project_id)
        .fetch_all(&db.pool)
        .await
    {
        Ok(orphan_tasks) => {
            let count = orphan_tasks.len();
            let response = OrphanTasksResponse {
                orphan_tasks,
                count,
            };
            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "Failed to find orphan tasks: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// Task without spec links
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct OrphanTask {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Response for orphan tasks query
#[derive(Serialize)]
pub struct OrphanTasksResponse {
    #[serde(rename = "orphanTasks")]
    pub orphan_tasks: Vec<OrphanTask>,
    pub count: usize,
}
