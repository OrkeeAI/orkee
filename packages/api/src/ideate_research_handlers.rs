// ABOUTME: HTTP request handlers for PRD research and competitor analysis
// ABOUTME: Handles competitor analysis, gap analysis, similar projects, and pattern extraction

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use orkee_ideate::{ResearchAnalyzer, SimilarProject};
use orkee_projects::DbState;
use serde::Deserialize;
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error};

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
