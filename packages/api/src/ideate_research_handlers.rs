// ABOUTME: HTTP request handlers for PRD research and competitor analysis
// ABOUTME: Handles competitor analysis, gap analysis, similar projects, and pattern extraction

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use orkee_ideate::{IdeateManager, ResearchAnalyzer, SimilarProject};
use orkee_projects::DbState;
use serde::Deserialize;
use tracing::{info, warn};

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};

// TODO: Replace with proper user authentication
const DEFAULT_USER_ID: &str = "default-user";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Request body for analyzing a competitor
#[derive(Deserialize)]
pub struct AnalyzeCompetitorRequest {
    pub url: String,
    #[serde(rename = "projectDescription")]
    pub project_description: Option<String>,
}

/// Request body for gap analysis
#[derive(Deserialize)]
pub struct GapAnalysisRequest {
    #[serde(rename = "yourFeatures")]
    pub your_features: Vec<String>,
}

/// Request body for UI pattern extraction
#[derive(Deserialize)]
pub struct ExtractPatternsRequest {
    pub url: String,
    #[serde(rename = "projectDescription")]
    pub project_description: Option<String>,
}

/// Request body for adding a similar project
#[derive(Deserialize)]
pub struct AddSimilarProjectRequest {
    pub name: String,
    pub url: String,
    #[serde(rename = "positiveAspects")]
    pub positive_aspects: Vec<String>,
    #[serde(rename = "negativeAspects")]
    pub negative_aspects: Vec<String>,
    #[serde(rename = "patternsToAdopt")]
    pub patterns_to_adopt: Vec<String>,
}

/// Request body for extracting lessons
#[derive(Deserialize)]
pub struct ExtractLessonsRequest {
    #[serde(rename = "projectName")]
    pub project_name: String,
    #[serde(rename = "projectDescription")]
    pub project_description: Option<String>,
}

/// Analyze a competitor URL
pub async fn analyze_competitor(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<AnalyzeCompetitorRequest>,
) -> impl IntoResponse {
    info!(
        "Analyzing competitor for session {}: {}",
        session_id, request.url
    );

    // Get session to extract project description if not provided
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to retrieve session: {}", e);
            return ok_or_not_found::<(), orkee_ideate::IdeateError>(Err(e), "Session not found");
        }
    };

    let project_description = request
        .project_description
        .unwrap_or_else(|| session.initial_description.clone());

    let analyzer = ResearchAnalyzer::new(db.pool.clone());

    let result = analyzer
        .analyze_competitor(
            &session_id,
            &project_description,
            &request.url,
            DEFAULT_USER_ID,
            DEFAULT_MODEL,
        )
        .await;
    ok_or_internal_error(result, "Failed to analyze competitor")
}

/// Get all competitors for a session
pub async fn get_competitors(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting competitors for session: {}", session_id);

    let analyzer = ResearchAnalyzer::new(db.pool.clone());
    let result = analyzer.get_competitors(&session_id).await;
    ok_or_internal_error(result, "Failed to get competitors")
}

/// Perform gap analysis
pub async fn analyze_gaps(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<GapAnalysisRequest>,
) -> impl IntoResponse {
    info!("Performing gap analysis for session: {}", session_id);

    // Get session for project description
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_not_found::<(), orkee_ideate::IdeateError>(Err(e), "Session not found");
        }
    };

    let analyzer = ResearchAnalyzer::new(db.pool.clone());

    let result = analyzer
        .analyze_gaps(
            &session_id,
            &session.initial_description,
            request.your_features,
            DEFAULT_USER_ID,
            DEFAULT_MODEL,
        )
        .await;
    ok_or_internal_error(result, "Failed to analyze gaps")
}

/// Extract UI/UX patterns from a URL
pub async fn extract_patterns(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<ExtractPatternsRequest>,
) -> impl IntoResponse {
    info!(
        "Extracting UI patterns from {} for session: {}",
        request.url, session_id
    );

    // Get session for project description if not provided
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_not_found::<(), orkee_ideate::IdeateError>(Err(e), "Session not found");
        }
    };

    let project_description = request
        .project_description
        .unwrap_or_else(|| session.initial_description.clone());

    let analyzer = ResearchAnalyzer::new(db.pool.clone());

    let result = analyzer
        .extract_ui_patterns(&project_description, &request.url, DEFAULT_USER_ID, DEFAULT_MODEL)
        .await;
    ok_or_internal_error(result, "Failed to extract UI patterns")
}

/// Add a similar project
pub async fn add_similar_project(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<AddSimilarProjectRequest>,
) -> impl IntoResponse {
    info!(
        "Adding similar project '{}' for session: {}",
        request.name, session_id
    );

    let analyzer = ResearchAnalyzer::new(db.pool.clone());

    let project = SimilarProject {
        name: request.name,
        url: Some(request.url),
        positive_aspects: request.positive_aspects,
        negative_aspects: request.negative_aspects,
        patterns_to_adopt: request.patterns_to_adopt,
    };

    let result = analyzer.add_similar_project(&session_id, project).await;
    created_or_internal_error(result, "Failed to add similar project")
}

/// Get similar projects for a session
pub async fn get_similar_projects(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting similar projects for session: {}", session_id);

    let analyzer = ResearchAnalyzer::new(db.pool.clone());
    let result = analyzer.get_similar_projects(&session_id).await;
    ok_or_internal_error(result, "Failed to get similar projects")
}

/// Extract lessons from a similar project
pub async fn extract_lessons(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<ExtractLessonsRequest>,
) -> impl IntoResponse {
    info!(
        "Extracting lessons from '{}' for session: {}",
        request.project_name, session_id
    );

    // Get session for project description if not provided
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to retrieve session: {}", e);
            return ok_or_not_found::<(), orkee_ideate::IdeateError>(Err(e), "Session not found");
        }
    };

    let project_description = request
        .project_description
        .unwrap_or_else(|| session.initial_description.clone());

    // Get the similar project
    let analyzer = ResearchAnalyzer::new(db.pool.clone());
    let similar_projects = match analyzer.get_similar_projects(&session_id).await {
        Ok(projects) => projects,
        Err(e) => {
            return ok_or_internal_error::<(), orkee_ideate::IdeateError>(
                Err(e),
                "Failed to get similar projects",
            );
        }
    };

    let similar_project = match similar_projects
        .iter()
        .find(|p| p.name == request.project_name)
    {
        Some(p) => p,
        None => {
            return ok_or_not_found::<(), orkee_ideate::IdeateError>(
                Err(orkee_ideate::IdeateError::InvalidInput(format!(
                    "Similar project '{}' not found",
                    request.project_name
                ))),
                "Similar project not found",
            );
        }
    };

    let result = analyzer
        .extract_lessons(
            &project_description,
            similar_project,
            DEFAULT_USER_ID,
            DEFAULT_MODEL,
        )
        .await;
    ok_or_internal_error(result, "Failed to extract lessons")
}

/// Synthesize all research findings
pub async fn synthesize_research(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Synthesizing research for session: {}", session_id);

    // Get session for project description
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_not_found::<(), orkee_ideate::IdeateError>(Err(e), "Session not found");
        }
    };

    let analyzer = ResearchAnalyzer::new(db.pool.clone());

    let result = analyzer
        .synthesize_research(
            &session_id,
            &session.initial_description,
            DEFAULT_USER_ID,
            DEFAULT_MODEL,
        )
        .await;
    ok_or_internal_error(result, "Failed to synthesize research")
}
