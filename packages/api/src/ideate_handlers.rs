// ABOUTME: HTTP request handlers for PRD ideation operations
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
    CreateIdeateSessionInput, IdeateManager, IdeateMode, IdeateStatus, PRDGenerator,
    SkipSectionRequest, UpdateIdeateSessionInput,
};
use orkee_projects::DbState;

// TODO: Replace with proper user authentication
const DEFAULT_USER_ID: &str = "default-user";

/// Request body for starting a ideateing session
#[derive(Deserialize)]
pub struct StartIdeateRequest {
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "initialDescription")]
    pub initial_description: String,
    pub mode: IdeateMode,
}

/// Start a new ideateing session
pub async fn start_ideate(
    State(db): State<DbState>,
    Json(request): Json<StartIdeateRequest>,
) -> impl IntoResponse {
    info!(
        "Starting ideate session for project: {} (mode: {:?})",
        request.project_id, request.mode
    );

    let manager = IdeateManager::new(db.pool.clone());
    let input = CreateIdeateSessionInput {
        project_id: request.project_id,
        initial_description: request.initial_description,
        mode: request.mode,
    };

    let result = manager.create_session(input).await;
    created_or_internal_error(result, "Failed to start ideate session")
}

/// Get a ideateing session by ID
pub async fn get_ideate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting ideate session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_session(&session_id).await;
    ok_or_not_found(result, "Ideate session not found")
}

/// List all ideateing sessions for a project
pub async fn list_ideates(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing ideate sessions for project: {}", project_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.list_sessions(&project_id).await;
    ok_or_internal_error(result, "Failed to list ideate sessions")
}

/// Request body for updating a session
#[derive(Deserialize)]
pub struct UpdateIdeateRequest {
    #[serde(rename = "initialDescription")]
    pub initial_description: Option<String>,
    pub mode: Option<IdeateMode>,
    pub status: Option<IdeateStatus>,
    #[serde(rename = "skippedSections")]
    pub skipped_sections: Option<Vec<String>>,
}

/// Update a ideateing session
pub async fn update_ideate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<UpdateIdeateRequest>,
) -> impl IntoResponse {
    info!("Updating ideate session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let input = UpdateIdeateSessionInput {
        initial_description: request.initial_description,
        mode: request.mode,
        status: request.status,
        skipped_sections: request.skipped_sections,
        current_section: None,
    };

    let result = manager.update_session(&session_id, input).await;
    ok_or_internal_error(result, "Failed to update ideate session")
}

/// Delete a ideateing session
pub async fn delete_ideate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting ideate session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_session(&session_id).await;
    ok_or_internal_error(result, "Failed to delete ideate session")
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

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.skip_section(&session_id, request).await;
    ok_or_internal_error(result, "Failed to skip section")
}

/// Get session completion status
pub async fn get_status(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting completion status for session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_completion_status(&session_id).await;
    ok_or_internal_error(result, "Failed to get session status")
}

// ============================================================================
// QUICK MODE ENDPOINTS
// ============================================================================

/// Request body for quick PRD generation
#[derive(Deserialize)]
pub struct QuickGenerateRequest {
    // Empty for now - uses session's initial_description
}

/// Generate a complete PRD from the session's initial description (Quick Mode)
pub async fn quick_generate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(_request): Json<QuickGenerateRequest>,
) -> impl IntoResponse {
    info!("Generating complete PRD for session: {}", session_id);

    // Get the session to retrieve the description
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Generate PRD using the generator
    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .generate_complete_prd(DEFAULT_USER_ID, &session.initial_description)
        .await;

    match result {
        Ok(prd) => {
            // Convert to JSON for response
            let json_value = serde_json::to_value(&prd).unwrap_or_else(|_| serde_json::json!({}));
            ok_or_internal_error::<_, String>(Ok(json_value), "Failed to generate PRD")
        }
        Err(e) => ok_or_internal_error::<serde_json::Value, _>(Err(e), "Failed to generate PRD"),
    }
}

/// Request body for expanding a specific section
#[derive(Deserialize)]
pub struct QuickExpandRequest {
    pub section: String,
    #[serde(rename = "context")]
    pub context: Option<String>,
}

/// Expand a specific section of the PRD
pub async fn quick_expand(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<QuickExpandRequest>,
) -> impl IntoResponse {
    info!(
        "Expanding section '{}' for session: {}",
        request.section, session_id
    );

    // Get the session to retrieve the description
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Generate specific section
    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .generate_section(
            DEFAULT_USER_ID,
            &request.section,
            &session.initial_description,
            request.context.as_deref(),
        )
        .await;

    ok_or_internal_error(result, "Failed to expand section")
}

/// Get a preview of the generated PRD in markdown format
pub async fn get_preview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting PRD preview for session: {}", session_id);

    // Get the session
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Generate PRD
    let generator = PRDGenerator::new(db.pool.clone());
    let prd = match generator
        .generate_complete_prd(DEFAULT_USER_ID, &session.initial_description)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Failed to generate PRD")
        }
    };

    // Format as markdown
    let markdown = generator.format_prd_markdown(&prd);

    // Return as JSON with markdown string
    let response = serde_json::json!({
        "markdown": markdown,
        "prd": prd
    });

    ok_or_internal_error::<_, String>(Ok(response), "Failed to get preview")
}

/// Request body for saving PRD
#[derive(Deserialize)]
pub struct SaveAsPRDRequest {
    pub title: String,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: String,
}

/// Save the generated PRD to the OpenSpec system
pub async fn save_as_prd(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<SaveAsPRDRequest>,
) -> impl IntoResponse {
    info!("Saving PRD for session: {}", session_id);

    // Get the session
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Create PRD in OpenSpec system
    use openspec::db::create_prd;
    use openspec::types::{PRDSource, PRDStatus};

    let prd = match create_prd(
        &db.pool,
        &session.project_id,
        &request.title,
        &request.content_markdown,
        PRDStatus::Draft,
        PRDSource::Generated,
        Some(DEFAULT_USER_ID),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(
                Err(ideate::IdeateError::AIService(e.to_string())),
                "Failed to save PRD",
            )
        }
    };

    // Update session with generated PRD ID
    let update_input = UpdateIdeateSessionInput {
        initial_description: None,
        mode: None,
        status: Some(IdeateStatus::Completed),
        skipped_sections: None,
        current_section: None,
    };

    if let Err(e) = manager.update_session(&session_id, update_input).await {
        tracing::warn!("Failed to update session status: {}", e);
    }

    // Also update the generated_prd_id in the database directly
    let _ = sqlx::query("UPDATE ideate_sessions SET generated_prd_id = ? WHERE id = ?")
        .bind(&prd.id)
        .bind(&session_id)
        .execute(&db.pool)
        .await;

    // Return the created PRD
    let response = serde_json::to_value(&prd).unwrap_or_else(|_| serde_json::json!({}));
    ok_or_internal_error::<_, String>(Ok(response), "Failed to save PRD")
}

// ============================================================================
// GUIDED MODE - SECTION ENDPOINTS
// ============================================================================

// Overview Section
pub async fn save_overview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(overview): Json<ideate::IdeateOverview>,
) -> impl IntoResponse {
    info!("Saving overview section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_overview(&session_id, overview).await;
    ok_or_internal_error(result, "Failed to save overview section")
}

pub async fn get_overview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting overview section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_overview(&session_id).await;
    ok_or_internal_error(result, "Failed to get overview section")
}

pub async fn delete_overview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting overview section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_overview(&session_id).await;
    ok_or_internal_error(result, "Failed to delete overview section")
}

// UX Section
pub async fn save_ux(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(ux): Json<ideate::IdeateUX>,
) -> impl IntoResponse {
    info!("Saving UX section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_ux(&session_id, ux).await;
    ok_or_internal_error(result, "Failed to save UX section")
}

pub async fn get_ux(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting UX section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_ux(&session_id).await;
    ok_or_internal_error(result, "Failed to get UX section")
}

pub async fn delete_ux(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting UX section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_ux(&session_id).await;
    ok_or_internal_error(result, "Failed to delete UX section")
}

// Technical Section
pub async fn save_technical(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(technical): Json<ideate::IdeateTechnical>,
) -> impl IntoResponse {
    info!("Saving technical section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_technical(&session_id, technical).await;
    ok_or_internal_error(result, "Failed to save technical section")
}

pub async fn get_technical(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting technical section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_technical(&session_id).await;
    ok_or_internal_error(result, "Failed to get technical section")
}

pub async fn delete_technical(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting technical section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_technical(&session_id).await;
    ok_or_internal_error(result, "Failed to delete technical section")
}

// Roadmap Section
pub async fn save_roadmap(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(roadmap): Json<ideate::IdeateRoadmap>,
) -> impl IntoResponse {
    info!("Saving roadmap section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_roadmap(&session_id, roadmap).await;
    ok_or_internal_error(result, "Failed to save roadmap section")
}

pub async fn get_roadmap(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting roadmap section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_roadmap(&session_id).await;
    ok_or_internal_error(result, "Failed to get roadmap section")
}

pub async fn delete_roadmap(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting roadmap section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_roadmap(&session_id).await;
    ok_or_internal_error(result, "Failed to delete roadmap section")
}

// Dependencies Section
pub async fn save_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(deps): Json<ideate::IdeateDependencies>,
) -> impl IntoResponse {
    info!("Saving dependencies section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_dependencies(&session_id, deps).await;
    ok_or_internal_error(result, "Failed to save dependencies section")
}

pub async fn get_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting dependencies section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_dependencies(&session_id).await;
    ok_or_internal_error(result, "Failed to get dependencies section")
}

pub async fn delete_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting dependencies section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_dependencies(&session_id).await;
    ok_or_internal_error(result, "Failed to delete dependencies section")
}

// Risks Section
pub async fn save_risks(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(risks): Json<ideate::IdeateRisks>,
) -> impl IntoResponse {
    info!("Saving risks section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_risks(&session_id, risks).await;
    ok_or_internal_error(result, "Failed to save risks section")
}

pub async fn get_risks(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting risks section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_risks(&session_id).await;
    ok_or_internal_error(result, "Failed to get risks section")
}

pub async fn delete_risks(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting risks section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_risks(&session_id).await;
    ok_or_internal_error(result, "Failed to delete risks section")
}

// Research Section
pub async fn save_research(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(research): Json<ideate::IdeateResearch>,
) -> impl IntoResponse {
    info!("Saving research section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_research(&session_id, research).await;
    ok_or_internal_error(result, "Failed to save research section")
}

pub async fn get_research(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting research section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_research(&session_id).await;
    ok_or_internal_error(result, "Failed to get research section")
}

pub async fn delete_research(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting research section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_research(&session_id).await;
    ok_or_internal_error(result, "Failed to delete research section")
}

// Navigation Endpoints
pub async fn get_next_section(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting next section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_next_section(&session_id).await;
    ok_or_internal_error(result, "Failed to get next section")
}

#[derive(Deserialize)]
pub struct NavigateToRequest {
    pub section: String,
}

pub async fn navigate_to(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<NavigateToRequest>,
) -> impl IntoResponse {
    info!(
        "Navigating to section '{}' for session: {}",
        request.section, session_id
    );
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.navigate_to(&session_id, &request.section).await;
    ok_or_internal_error(result, "Failed to navigate to section")
}
