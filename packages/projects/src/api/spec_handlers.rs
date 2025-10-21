// ABOUTME: HTTP request handlers for spec/capability operations
// ABOUTME: Handles CRUD operations for OpenSpec capabilities and validation

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found, ApiResponse};
use crate::db::DbState;
use crate::openspec::db as openspec_db;
use crate::openspec::parser;
use crate::openspec::types::CapabilityStatus;
use crate::openspec::validator;
use crate::pagination::{PaginatedResponse, PaginationParams};

/// List all capabilities for a project
pub async fn list_capabilities(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Listing capabilities for project: {} (page: {})", project_id, pagination.page());

    let result = openspec_db::get_capabilities_by_project_paginated(&db.pool, &project_id, Some(pagination.limit()), Some(pagination.offset()))
        .await
        .map(|(capabilities, total)| PaginatedResponse::new(capabilities, &pagination, total));

    ok_or_internal_error(result, "Failed to list capabilities")
}

/// List all capabilities with their requirements for a project (optimized)
pub async fn list_capabilities_with_requirements(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!(
        "Listing capabilities with requirements for project: {}",
        project_id
    );

    let result = openspec_db::get_capabilities_with_requirements_by_project(&db.pool, &project_id).await;
    ok_or_internal_error(result, "Failed to list capabilities with requirements")
}

/// Get a single capability by ID
pub async fn get_capability(
    State(db): State<DbState>,
    Path((_project_id, capability_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting capability: {}", capability_id);

    let result = openspec_db::get_capability(&db.pool, &capability_id).await;
    ok_or_not_found(result, "Capability not found")
}

/// Request body for creating a capability
#[derive(Deserialize)]
pub struct CreateCapabilityRequest {
    pub name: String,
    #[serde(rename = "prdId")]
    pub prd_id: Option<String>,
    #[serde(rename = "purposeMarkdown")]
    pub purpose_markdown: Option<String>,
    #[serde(rename = "specMarkdown")]
    pub spec_markdown: String,
    #[serde(rename = "designMarkdown")]
    pub design_markdown: Option<String>,
}

/// Create a new capability
pub async fn create_capability(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateCapabilityRequest>,
) -> impl IntoResponse {
    info!(
        "Creating capability '{}' for project: {}",
        request.name, project_id
    );

    let result = openspec_db::create_capability(
        &db.pool,
        &project_id,
        request.prd_id.as_deref(),
        &request.name,
        request.purpose_markdown.as_deref(),
        &request.spec_markdown,
        request.design_markdown.as_deref(),
    )
    .await;

    created_or_internal_error(result, "Failed to create capability")
}

/// Request body for updating a capability
#[derive(Deserialize)]
pub struct UpdateCapabilityRequest {
    #[serde(rename = "purposeMarkdown")]
    pub purpose_markdown: Option<String>,
    #[serde(rename = "specMarkdown")]
    pub spec_markdown: Option<String>,
    #[serde(rename = "designMarkdown")]
    pub design_markdown: Option<String>,
    pub status: Option<CapabilityStatus>,
}

/// Update an existing capability
pub async fn update_capability(
    State(db): State<DbState>,
    Path((_project_id, capability_id)): Path<(String, String)>,
    Json(request): Json<UpdateCapabilityRequest>,
) -> impl IntoResponse {
    info!("Updating capability: {}", capability_id);

    let result = openspec_db::update_capability(
        &db.pool,
        &capability_id,
        request.spec_markdown.as_deref(),
        request.purpose_markdown.as_deref(),
        request.design_markdown.as_deref(),
        request.status,
    )
    .await;

    ok_or_internal_error(result, "Failed to update capability")
}

/// Delete a capability (soft delete)
pub async fn delete_capability(
    State(db): State<DbState>,
    Path((_project_id, capability_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Soft deleting capability: {}", capability_id);

    match openspec_db::delete_capability(&db.pool, &capability_id).await {
        Ok(_) => (StatusCode::OK, ResponseJson(ApiResponse::success(()))).into_response(),
        Err(openspec_db::DbError::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error(msg)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "Failed to delete capability: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// Get all requirements for a capability
pub async fn get_capability_requirements(
    State(db): State<DbState>,
    Path((_project_id, capability_id)): Path<(String, String)>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Getting requirements for capability: {} (page: {})", capability_id, pagination.page());

    let result = openspec_db::get_requirements_by_capability_paginated(&db.pool, &capability_id, Some(pagination.limit()), Some(pagination.offset()))
        .await
        .map(|(requirements, total)| PaginatedResponse::new(requirements, &pagination, total));

    ok_or_internal_error(result, "Failed to get capability requirements")
}

/// Request body for validating spec markdown
#[derive(Deserialize)]
pub struct ValidateSpecRequest {
    #[serde(rename = "specMarkdown")]
    pub spec_markdown: String,
}

/// Response for validation with detailed error info
#[derive(Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(rename = "capabilityCount")]
    pub capability_count: usize,
    #[serde(rename = "requirementCount")]
    pub requirement_count: usize,
    #[serde(rename = "scenarioCount")]
    pub scenario_count: usize,
}

/// Validate spec markdown format
pub async fn validate_spec(Json(request): Json<ValidateSpecRequest>) -> impl IntoResponse {
    info!(
        "Validating spec markdown ({} bytes)",
        request.spec_markdown.len()
    );

    // Parse the markdown
    match parser::parse_spec_markdown(&request.spec_markdown) {
        Ok(parsed_spec) => {
            // Validate the parsed spec
            let validation_result = validator::validate_spec(&parsed_spec);

            let capability_count = parsed_spec.capabilities.len();
            let requirement_count: usize = parsed_spec
                .capabilities
                .iter()
                .map(|c| c.requirements.len())
                .sum();
            let scenario_count: usize = parsed_spec
                .capabilities
                .iter()
                .flat_map(|c| &c.requirements)
                .map(|r| r.scenarios.len())
                .sum();

            let response = ValidationResponse {
                valid: validation_result.is_ok(),
                errors: validation_result
                    .err()
                    .map(|e| vec![e.to_string()])
                    .unwrap_or_default(),
                warnings: Vec::new(), // Could add warnings from validator later
                capability_count,
                requirement_count,
                scenario_count,
            };

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            let response = ValidationResponse {
                valid: false,
                errors: vec![format!("Parse error: {}", e)],
                warnings: Vec::new(),
                capability_count: 0,
                requirement_count: 0,
                scenario_count: 0,
            };

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
    }
}
