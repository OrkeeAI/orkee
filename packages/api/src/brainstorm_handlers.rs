// ABOUTME: HTTP request handlers for brainstorming and PRD ideation operations
// ABOUTME: Handles session management, section updates, and completion status

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};
use ideate::{
    BrainstormManager, BrainstormMode, BrainstormStatus, CreateBrainstormSessionInput,
    SkipSectionRequest, UpdateBrainstormSessionInput,
};
use orkee_projects::DbState;

/// Request body for starting a brainstorming session
#[derive(Deserialize)]
pub struct StartBrainstormRequest {
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "initialDescription")]
    pub initial_description: String,
    pub mode: BrainstormMode,
}

/// Start a new brainstorming session
pub async fn start_brainstorm(
    State(db): State<DbState>,
    Json(request): Json<StartBrainstormRequest>,
) -> impl IntoResponse {
    info!(
        "Starting brainstorm session for project: {} (mode: {:?})",
        request.project_id, request.mode
    );

    let manager = BrainstormManager::new(db.pool.clone());
    let input = CreateBrainstormSessionInput {
        project_id: request.project_id,
        initial_description: request.initial_description,
        mode: request.mode,
    };

    let result = manager.create_session(input).await;
    created_or_internal_error(result, "Failed to start brainstorm session")
}

/// Get a brainstorming session by ID
pub async fn get_brainstorm(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting brainstorm session: {}", session_id);

    let manager = BrainstormManager::new(db.pool.clone());
    let result = manager.get_session(&session_id).await;
    ok_or_not_found(result, "Brainstorm session not found")
}

/// List all brainstorming sessions for a project
pub async fn list_brainstorms(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing brainstorm sessions for project: {}", project_id);

    let manager = BrainstormManager::new(db.pool.clone());
    let result = manager.list_sessions(&project_id).await;
    ok_or_internal_error(result, "Failed to list brainstorm sessions")
}

/// Request body for updating a session
#[derive(Deserialize)]
pub struct UpdateBrainstormRequest {
    #[serde(rename = "initialDescription")]
    pub initial_description: Option<String>,
    pub mode: Option<BrainstormMode>,
    pub status: Option<BrainstormStatus>,
    #[serde(rename = "skippedSections")]
    pub skipped_sections: Option<Vec<String>>,
}

/// Update a brainstorming session
pub async fn update_brainstorm(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<UpdateBrainstormRequest>,
) -> impl IntoResponse {
    info!("Updating brainstorm session: {}", session_id);

    let manager = BrainstormManager::new(db.pool.clone());
    let input = UpdateBrainstormSessionInput {
        initial_description: request.initial_description,
        mode: request.mode,
        status: request.status,
        skipped_sections: request.skipped_sections,
    };

    let result = manager.update_session(&session_id, input).await;
    ok_or_internal_error(result, "Failed to update brainstorm session")
}

/// Delete a brainstorming session
pub async fn delete_brainstorm(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting brainstorm session: {}", session_id);

    let manager = BrainstormManager::new(db.pool.clone());
    let result = manager.delete_session(&session_id).await;
    ok_or_internal_error(result, "Failed to delete brainstorm session")
}

/// Skip a section with optional AI fill
pub async fn skip_section(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<SkipSectionRequest>,
) -> impl IntoResponse {
    info!(
        "Skipping section '{}' for session: {} (AI fill: {})",
        request.section, session_id, request.ai_fill
    );

    let manager = BrainstormManager::new(db.pool.clone());
    let result = manager.skip_section(&session_id, request).await;
    ok_or_internal_error(result, "Failed to skip section")
}

/// Get session completion status
pub async fn get_status(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting completion status for session: {}", session_id);

    let manager = BrainstormManager::new(db.pool.clone());
    let result = manager.get_completion_status(&session_id).await;
    ok_or_internal_error(result, "Failed to get session status")
}
