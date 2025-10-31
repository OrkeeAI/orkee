// ABOUTME: HTTP request handlers for Epic operations (CCPM workflow)
// ABOUTME: Handles CRUD operations, generation, task decomposition, and progress tracking for Epics

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};
use orkee_ideate::{
    CreateEpicInput, Epic, EpicComplexity, EpicManager, EpicStatus, EstimatedEffort,
    UpdateEpicInput,
};
use orkee_projects::DbState;

/// List all Epics for a project
pub async fn list_epics(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing epics for project: {}", project_id);

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.list_epics(&project_id).await;

    ok_or_internal_error(result, "Failed to list epics")
}

/// Get a single Epic by ID
pub async fn get_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting epic: {} for project: {}", epic_id, project_id);

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.get_epic(&project_id, &epic_id).await;

    ok_or_not_found(result, "Epic not found")
}

/// List Epics by PRD
pub async fn list_epics_by_prd(
    State(db): State<DbState>,
    Path((project_id, prd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Listing epics for PRD: {} in project: {}",
        prd_id, project_id
    );

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.list_epics_by_prd(&project_id, &prd_id).await;

    ok_or_internal_error(result, "Failed to list epics for PRD")
}

/// Request body for creating an Epic
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEpicRequest {
    pub prd_id: String,
    pub name: String,
    pub overview_markdown: String,
    pub architecture_decisions: Option<Vec<orkee_ideate::ArchitectureDecision>>,
    pub technical_approach: String,
    pub implementation_strategy: Option<String>,
    pub dependencies: Option<Vec<orkee_ideate::ExternalDependency>>,
    pub success_criteria: Option<Vec<orkee_ideate::SuccessCriterion>>,
    pub task_categories: Option<Vec<String>>,
    pub estimated_effort: Option<EstimatedEffort>,
    pub complexity: Option<EpicComplexity>,
}

/// Create a new Epic
pub async fn create_epic(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateEpicRequest>,
) -> impl IntoResponse {
    info!(
        "Creating epic '{}' for project: {}",
        request.name, project_id
    );

    let input = CreateEpicInput {
        prd_id: request.prd_id,
        name: request.name,
        overview_markdown: request.overview_markdown,
        architecture_decisions: request.architecture_decisions,
        technical_approach: request.technical_approach,
        implementation_strategy: request.implementation_strategy,
        dependencies: request.dependencies,
        success_criteria: request.success_criteria,
        task_categories: request.task_categories,
        estimated_effort: request.estimated_effort,
        complexity: request.complexity,
    };

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.create_epic(&project_id, input).await;

    created_or_internal_error(result, "Failed to create epic")
}

/// Request body for updating an Epic
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEpicRequest {
    pub name: Option<String>,
    pub overview_markdown: Option<String>,
    pub architecture_decisions: Option<Vec<orkee_ideate::ArchitectureDecision>>,
    pub technical_approach: Option<String>,
    pub implementation_strategy: Option<String>,
    pub dependencies: Option<Vec<orkee_ideate::ExternalDependency>>,
    pub success_criteria: Option<Vec<orkee_ideate::SuccessCriterion>>,
    pub task_categories: Option<Vec<String>>,
    pub estimated_effort: Option<EstimatedEffort>,
    pub complexity: Option<EpicComplexity>,
    pub status: Option<EpicStatus>,
    pub progress_percentage: Option<i32>,
}

/// Update an existing Epic
pub async fn update_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
    Json(request): Json<UpdateEpicRequest>,
) -> impl IntoResponse {
    info!("Updating epic: {} for project: {}", epic_id, project_id);

    let input = UpdateEpicInput {
        name: request.name,
        overview_markdown: request.overview_markdown,
        architecture_decisions: request.architecture_decisions,
        technical_approach: request.technical_approach,
        implementation_strategy: request.implementation_strategy,
        dependencies: request.dependencies,
        success_criteria: request.success_criteria,
        task_categories: request.task_categories,
        estimated_effort: request.estimated_effort,
        complexity: request.complexity,
        status: request.status,
        progress_percentage: request.progress_percentage,
    };

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.update_epic(&project_id, &epic_id, input).await;

    ok_or_internal_error(result, "Failed to update epic")
}

/// Delete an Epic
pub async fn delete_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Deleting epic: {} from project: {}", epic_id, project_id);

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.delete_epic(&project_id, &epic_id).await;

    ok_or_internal_error(result, "Failed to delete epic")
}

/// Get tasks for an Epic
pub async fn get_epic_tasks(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting tasks for epic: {} in project: {}",
        epic_id, project_id
    );

    // Query tasks table for this epic
    let result = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT id FROM tasks WHERE epic_id = ?
        "#,
    )
    .bind(&epic_id)
    .fetch_all(&db.pool)
    .await
    .map(|rows| rows.into_iter().map(|(id,)| id).collect::<Vec<_>>());

    ok_or_internal_error(result, "Failed to get epic tasks")
}

/// Calculate Epic progress
#[derive(Serialize)]
pub struct ProgressResponse {
    progress: i32,
}

pub async fn calculate_epic_progress(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Calculating progress for epic: {} in project: {}",
        epic_id, project_id
    );

    let manager = EpicManager::new(db.pool.clone());
    let result = manager
        .calculate_progress(&project_id, &epic_id)
        .await
        .map(|progress| ProgressResponse { progress });

    ok_or_internal_error(result, "Failed to calculate epic progress")
}

/// Request body for generating an Epic from a PRD
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateEpicRequest {
    pub prd_id: String,
    pub include_task_breakdown: Option<bool>,
}

/// Response for Epic generation
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateEpicResponse {
    pub epic_id: String,
    pub tasks_created: Option<usize>,
}

/// Generate an Epic from a PRD (placeholder - AI generation to be implemented)
pub async fn generate_epic_from_prd(
    State(_db): State<DbState>,
    Path(_project_id): Path<String>,
    Json(_request): Json<GenerateEpicRequest>,
) -> impl IntoResponse {
    info!("Epic generation from PRD - not yet implemented");

    // TODO: Implement AI-powered Epic generation
    // This will:
    // 1. Load PRD content
    // 2. Analyze technical requirements
    // 3. Generate architecture decisions
    // 4. Create implementation strategy
    // 5. Optionally decompose to tasks

    let error = orkee_ideate::IdeateError::NotImplemented(
        "Epic generation from PRD is not yet implemented".to_string(),
    );
    ok_or_internal_error(Err::<Epic, _>(error), "Epic generation not implemented")
}

/// Analyze work streams for an Epic (placeholder - to be implemented)
pub async fn analyze_work_streams(
    State(_db): State<DbState>,
    Path((_project_id, _epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Work stream analysis - not yet implemented");

    // TODO: Implement work stream analysis
    // This will:
    // 1. Load tasks for epic
    // 2. Analyze file patterns
    // 3. Group tasks into work streams
    // 4. Detect conflicts
    // 5. Generate parallelization strategy

    let error = orkee_ideate::IdeateError::NotImplemented(
        "Work stream analysis is not yet implemented".to_string(),
    );
    ok_or_internal_error(
        Err::<orkee_ideate::WorkAnalysis, _>(error),
        "Work stream analysis not implemented",
    )
}
