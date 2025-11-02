// ABOUTME: API handlers for discovery and codebase analysis (Phase 6A.1 CCPM)
// ABOUTME: Endpoints for PRD discovery questions, codebase analysis, and project context

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use orkee_ideate::{CodebaseAnalyzer, CodebaseContext, DiscoveryManager};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{error, info};

use orkee_projects::DbState;

/// Request body for analyzing codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCodebaseRequest {
    #[serde(rename = "projectPath")]
    pub project_path: Option<String>,
}

/// Request body for answering discovery question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerQuestionRequest {
    pub answer: String,
}

/// Response for discovery progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryProgressResponse {
    pub total_questions_asked: usize,
    pub total_questions_answered: usize,
    pub categories_covered: Vec<String>,
    pub completion_percentage: f32,
}

/// POST /api/ideate/sessions/:id/analyze-codebase
/// Analyze project codebase for patterns, similar features, and reusable components
pub async fn analyze_codebase(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<AnalyzeCodebaseRequest>,
) -> impl IntoResponse {
    info!(
        "Analyzing codebase for session: {} with path: {:?}",
        session_id, request.project_path
    );

    // Get the session to retrieve project ID
    let manager = orkee_ideate::IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get session: {:?}", e);
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Ideate session not found"
                })),
            )
                .into_response();
        }
    };

    // Determine project path
    let project_path = if let Some(path) = request.project_path {
        PathBuf::from(path)
    } else {
        // Try to get project path from projects database
        match orkee_projects::get_project(&session.project_id).await {
            Ok(Some(project)) => PathBuf::from(project.project_root),
            Ok(None) => {
                error!("Project not found: {}", session.project_id);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": "Project not found and no project_path provided"
                    })),
                )
                    .into_response();
            }
            Err(e) => {
                error!("Failed to get project: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to get project: {}", e)
                    })),
                )
                    .into_response();
            }
        }
    };

    // Analyze the codebase
    let analyzer = CodebaseAnalyzer::new(project_path);
    match analyzer.analyze_for_session(&session).await {
        Ok(context) => {
            // Store the codebase context in the session
            // For now, we'll just return it. In the future, we might want to persist it.
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "data": context
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to analyze codebase: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to analyze codebase: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/ideate/sessions/:id/codebase-context
/// Get stored codebase context for a session
pub async fn get_codebase_context(
    State(_db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting codebase context for session: {}", session_id);

    // For now, return empty context since we don't persist it yet
    // In the future, we might want to store this in the database
    let context = CodebaseContext::default();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": context
        })),
    )
        .into_response()
}

/// POST /api/ideate/sessions/:id/next-question
/// Get the next discovery question based on session context
pub async fn get_next_question(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!(
        "Getting next discovery question for session: {}",
        session_id
    );

    let discovery_manager = DiscoveryManager::new(db.pool.clone());

    match discovery_manager.get_next_question(&session_id).await {
        Ok(question) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": question
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get next question: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get next question: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/ideate/sessions/:id/discovery-progress
/// Get discovery progress statistics for a session
pub async fn get_discovery_progress(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting discovery progress for session: {}", session_id);

    let discovery_manager = DiscoveryManager::new(db.pool.clone());

    // Get all answers for this session
    match discovery_manager.get_answers(&session_id).await {
        Ok(answers) => {
            let total_questions_asked = answers.len();
            let total_questions_answered = answers.iter().filter(|a| a.answered_at.is_some()).count();

            // Calculate unique categories covered
            let categories_covered: Vec<String> = answers
                .iter()
                .filter_map(|a| {
                    // Simple keyword-based category inference
                    let text = a.question_text.to_lowercase();
                    if text.contains("problem") {
                        Some("problem".to_string())
                    } else if text.contains("user") || text.contains("audience") {
                        Some("users".to_string())
                    } else if text.contains("feature") || text.contains("capabilities") {
                        Some("features".to_string())
                    } else if text.contains("technical") || text.contains("approach") {
                        Some("technical".to_string())
                    } else if text.contains("risk") || text.contains("concern") {
                        Some("risks".to_string())
                    } else if text.contains("constraint") || text.contains("requirement") {
                        Some("constraints".to_string())
                    } else if text.contains("success") || text.contains("goal") {
                        Some("success".to_string())
                    } else {
                        None
                    }
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            // Estimate completion percentage (assuming 7-10 questions for a complete PRD)
            let completion_percentage = if total_questions_answered == 0 {
                0.0
            } else {
                (total_questions_answered as f32 / 8.0 * 100.0).min(100.0)
            };

            let response = DiscoveryProgressResponse {
                total_questions_asked,
                total_questions_answered,
                categories_covered,
                completion_percentage,
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "data": response
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to get discovery progress: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get discovery progress: {}", e)
                })),
            )
                .into_response()
        }
    }
}
