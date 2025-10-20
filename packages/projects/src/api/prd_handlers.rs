// ABOUTME: HTTP request handlers for PRD (Product Requirements Document) operations
// ABOUTME: Handles CRUD operations for PRDs with database integration

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::ApiResponse;
use crate::db::DbState;
use crate::openspec::db as openspec_db;
use crate::openspec::types::{PRDSource, PRDStatus};

/// List all PRDs for a project
pub async fn list_prds(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing PRDs for project: {}", project_id);

    match openspec_db::get_prds_by_project(&db.pool, &project_id).await {
        Ok(prds) => (StatusCode::OK, ResponseJson(ApiResponse::success(prds))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!("Failed to list PRDs: {}", e))),
        )
            .into_response(),
    }
}

/// Get a single PRD by ID
pub async fn get_prd(
    State(db): State<DbState>,
    Path((_project_id, prd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting PRD: {}", prd_id);

    match openspec_db::get_prd(&db.pool, &prd_id).await {
        Ok(prd) => (StatusCode::OK, ResponseJson(ApiResponse::success(prd))).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error(format!("PRD not found: {}", e))),
        )
            .into_response(),
    }
}

/// Request body for creating a PRD
#[derive(Deserialize)]
pub struct CreatePRDRequest {
    pub title: String,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: String,
    pub status: Option<PRDStatus>,
    pub source: Option<PRDSource>,
    #[serde(rename = "createdBy")]
    pub created_by: Option<String>,
}

/// Create a new PRD
pub async fn create_prd(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Json(request): Json<CreatePRDRequest>,
) -> impl IntoResponse {
    info!("Creating PRD '{}' for project: {}", request.title, project_id);

    let status = request.status.unwrap_or(PRDStatus::Draft);
    let source = request.source.unwrap_or(PRDSource::Manual);

    match openspec_db::create_prd(
        &db.pool,
        &project_id,
        &request.title,
        &request.content_markdown,
        status,
        source,
        request.created_by.as_deref(),
    )
    .await
    {
        Ok(prd) => (StatusCode::CREATED, ResponseJson(ApiResponse::success(prd))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!("Failed to create PRD: {}", e))),
        )
            .into_response(),
    }
}

/// Request body for updating a PRD
#[derive(Deserialize)]
pub struct UpdatePRDRequest {
    pub title: Option<String>,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: Option<String>,
    pub status: Option<PRDStatus>,
}

/// Update an existing PRD
pub async fn update_prd(
    State(db): State<DbState>,
    Path((_project_id, prd_id)): Path<(String, String)>,
    Json(request): Json<UpdatePRDRequest>,
) -> impl IntoResponse {
    info!("Updating PRD: {}", prd_id);

    match openspec_db::update_prd(
        &db.pool,
        &prd_id,
        request.title.as_deref(),
        request.content_markdown.as_deref(),
        request.status,
    )
    .await
    {
        Ok(prd) => (StatusCode::OK, ResponseJson(ApiResponse::success(prd))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!("Failed to update PRD: {}", e))),
        )
            .into_response(),
    }
}

/// Delete a PRD
pub async fn delete_prd(
    State(db): State<DbState>,
    Path((_project_id, prd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Deleting PRD: {}", prd_id);

    match openspec_db::delete_prd(&db.pool, &prd_id).await {
        Ok(_) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success(())),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!("Failed to delete PRD: {}", e))),
        )
            .into_response(),
    }
}

/// Get all capabilities associated with a PRD
pub async fn get_prd_capabilities(
    State(db): State<DbState>,
    Path((_project_id, prd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting capabilities for PRD: {}", prd_id);

    match openspec_db::get_capabilities_by_prd(&db.pool, &prd_id).await {
        Ok(capabilities) => {
            (StatusCode::OK, ResponseJson(ApiResponse::success(capabilities))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "Failed to get PRD capabilities: {}",
                e
            ))),
        )
            .into_response(),
    }
}
