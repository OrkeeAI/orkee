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
#[serde(rename_all = "snake_case")]
pub struct DiscoveryProgressResponse {
    pub session_id: String,
    pub total_questions: usize,
    pub answered_questions: usize,
    pub current_question_number: usize,
    pub completion_percentage: f32,
    pub estimated_remaining: u32,
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
            error!("Failed to get session {}: {:?}", session_id, e);
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Ideate session '{}' not found. Please verify the session ID exists.", session_id)
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
                error!(
                    "Project '{}' not found for session '{}'",
                    session.project_id, session_id
                );
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!(
                            "Project '{}' not found for session '{}'. Please provide a project_path in the request body or ensure the project exists.",
                            session.project_id, session_id
                        )
                    })),
                )
                    .into_response();
            }
            Err(e) => {
                error!(
                    "Failed to get project '{}' for session '{}': {:?}",
                    session.project_id, session_id, e
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!(
                            "Failed to retrieve project '{}' from database: {}. Session ID: {}",
                            session.project_id, e, session_id
                        )
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
            error!(
                "Failed to analyze codebase for session '{}': {:?}",
                session_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!(
                        "Failed to analyze codebase for session '{}': {}. Please verify the project path is accessible and contains valid code.",
                        session_id, e
                    )
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

    let _discovery_manager = DiscoveryManager::new(db.pool.clone());

    // Get conversation messages to track actual progress
    let chat_manager = orkee_ideate::ChatManager::new(db.pool.clone());
    match chat_manager.get_history(&session_id).await {
        Ok(messages) => {
            // Count user messages (questions answered) and assistant messages (questions asked)
            let user_message_count = messages
                .iter()
                .filter(|m| m.role == orkee_ideate::MessageRole::User)
                .count();
            let assistant_message_count = messages
                .iter()
                .filter(|m| m.role == orkee_ideate::MessageRole::Assistant)
                .count();

            let answered_questions = user_message_count;
            let total_questions_asked = assistant_message_count;

            // Current question is the next one to be asked
            let current_question_number = total_questions_asked + 1;

            // Dynamic total based on actual conversation (minimum 5, expands as needed)
            // This gives a realistic sense of progress without false precision
            let total_questions = std::cmp::max(answered_questions + 3, 5);

            // Calculate completion percentage based on typical coverage
            // We expect ~5-10 exchanges for a good PRD
            let completion_percentage = if answered_questions == 0 {
                0.0
            } else {
                // Soft cap at 90% until we hit 10+ messages (never shows 100% during discovery)
                ((answered_questions as f32 / 10.0) * 90.0).min(90.0)
            };

            // Estimate remaining time: ~2 minutes per remaining question until we hit ~8 questions
            let estimated_remaining_questions = 8_usize.saturating_sub(answered_questions).min(5);
            let estimated_remaining = (estimated_remaining_questions * 2) as u32;

            let response = DiscoveryProgressResponse {
                session_id: session_id.clone(),
                total_questions,
                answered_questions,
                current_question_number,
                completion_percentage,
                estimated_remaining,
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
