// ABOUTME: HTTP request handlers for spec change operations
// ABOUTME: Handles CRUD operations for spec changes and deltas

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};
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

    let result = openspec_db::create_spec_change(
        &db.pool,
        &project_id,
        request.prd_id.as_deref(),
        &request.proposal_markdown,
        &request.tasks_markdown,
        request.design_markdown.as_deref(),
        &request.created_by,
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

    let result = openspec_db::update_spec_change_status(
        &db.pool,
        &change_id,
        request.status,
        request.approved_by.as_deref(),
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

    let result = openspec_db::create_spec_delta(
        &db.pool,
        &change_id,
        request.capability_id.as_deref(),
        &request.capability_name,
        request.delta_type,
        &request.delta_markdown,
        request.position,
    )
    .await;

    created_or_internal_error(result, "Failed to create delta")
}
