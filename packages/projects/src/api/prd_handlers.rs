// ABOUTME: HTTP request handlers for PRD (Product Requirements Document) operations
// ABOUTME: Handles CRUD operations for PRDs with database integration

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};
use crate::db::DbState;
use openspec::db as openspec_db;
use openspec::types::{PRDSource, PRDStatus};
use crate::pagination::{PaginatedResponse, PaginationParams};

/// List all PRDs for a project
pub async fn list_prds(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Listing PRDs for project: {} (page: {})",
        project_id,
        pagination.page()
    );

    let result = openspec_db::get_prds_by_project_paginated(
        &db.pool,
        &project_id,
        Some(pagination.limit()),
        Some(pagination.offset()),
    )
    .await
    .map(|(prds, total)| PaginatedResponse::new(prds, &pagination, total));

    ok_or_internal_error(result, "Failed to list PRDs")
}

/// Get a single PRD by ID
pub async fn get_prd(
    State(db): State<DbState>,
    Path((_project_id, prd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting PRD: {}", prd_id);

    let result = openspec_db::get_prd(&db.pool, &prd_id).await;
    ok_or_not_found(result, "PRD not found")
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
    info!(
        "Creating PRD '{}' for project: {}",
        request.title, project_id
    );

    let status = request.status.unwrap_or(PRDStatus::Draft);
    let source = request.source.unwrap_or(PRDSource::Manual);

    let result = openspec_db::create_prd(
        &db.pool,
        &project_id,
        &request.title,
        &request.content_markdown,
        status,
        source,
        request.created_by.as_deref(),
    )
    .await;

    created_or_internal_error(result, "Failed to create PRD")
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

    let result = openspec_db::update_prd(
        &db.pool,
        &prd_id,
        request.title.as_deref(),
        request.content_markdown.as_deref(),
        request.status,
    )
    .await;

    ok_or_internal_error(result, "Failed to update PRD")
}

/// Delete a PRD
pub async fn delete_prd(
    State(db): State<DbState>,
    Path((_project_id, prd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Deleting PRD: {}", prd_id);

    let result = openspec_db::delete_prd(&db.pool, &prd_id).await;
    ok_or_internal_error(result, "Failed to delete PRD")
}

/// Get all capabilities associated with a PRD
pub async fn get_prd_capabilities(
    State(db): State<DbState>,
    Path((_project_id, prd_id)): Path<(String, String)>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Getting capabilities for PRD: {} (page: {})",
        prd_id,
        pagination.page()
    );

    let result = openspec_db::get_capabilities_by_prd_paginated(
        &db.pool,
        &prd_id,
        Some(pagination.limit()),
        Some(pagination.offset()),
    )
    .await
    .map(|(capabilities, total)| PaginatedResponse::new(capabilities, &pagination, total));

    ok_or_internal_error(result, "Failed to get PRD capabilities")
}
