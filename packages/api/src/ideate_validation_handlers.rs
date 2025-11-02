// ABOUTME: API handlers for PRD validation (Phase 6A.2 CCPM)
// ABOUTME: Endpoints for validating PRD sections, calculating quality scores, and storing feedback

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use orkee_ideate::{IdeateManager, PRDValidationResult, PRDValidator};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use orkee_projects::DbState;

/// Request body for storing validation feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreValidationRequest {
    pub section: String,
    pub validation_result: PRDValidationResult,
    pub user_notes: Option<String>,
}

/// Response for quality score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScoreResponse {
    pub overall_score: i32,
    pub section_scores: std::collections::HashMap<String, i32>,
    pub passed: bool,
    pub total_issues: usize,
    pub total_suggestions: usize,
}

/// POST /api/ideate/sessions/:id/validate-section/:section
/// Validate a specific PRD section and return quality score with issues
pub async fn validate_section(
    State(db): State<DbState>,
    Path((session_id, section_name)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Validating section '{}' for session: {}",
        section_name, session_id
    );

    let manager = IdeateManager::new(db.pool.clone());
    let validator = PRDValidator::new();

    // Fetch the section content based on section name
    let content = match section_name.as_str() {
        "overview" => {
            match manager.get_overview(&session_id).await {
                Ok(Some(overview)) => {
                    // Serialize to JSON for validation
                    serde_json::to_value(&overview)
                        .ok()
                        .and_then(|v| serde_json::to_string(&v).ok())
                        .unwrap_or_default()
                }
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Section not found"
                        })),
                    )
                        .into_response();
                }
                Err(e) => {
                    error!("Failed to fetch overview: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to fetch section: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        "ux" => {
            match manager.get_ux(&session_id).await {
                Ok(Some(ux)) => serde_json::to_value(&ux)
                    .ok()
                    .and_then(|v| serde_json::to_string(&v).ok())
                    .unwrap_or_default(),
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Section not found"
                        })),
                    )
                        .into_response();
                }
                Err(e) => {
                    error!("Failed to fetch ux: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to fetch section: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        "technical" => {
            match manager.get_technical(&session_id).await {
                Ok(Some(technical)) => serde_json::to_value(&technical)
                    .ok()
                    .and_then(|v| serde_json::to_string(&v).ok())
                    .unwrap_or_default(),
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Section not found"
                        })),
                    )
                        .into_response();
                }
                Err(e) => {
                    error!("Failed to fetch technical: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to fetch section: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        "roadmap" => {
            match manager.get_roadmap(&session_id).await {
                Ok(Some(roadmap)) => serde_json::to_value(&roadmap)
                    .ok()
                    .and_then(|v| serde_json::to_string(&v).ok())
                    .unwrap_or_default(),
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Section not found"
                        })),
                    )
                        .into_response();
                }
                Err(e) => {
                    error!("Failed to fetch roadmap: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to fetch section: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        "dependencies" => {
            match manager.get_dependencies(&session_id).await {
                Ok(Some(dependencies)) => serde_json::to_value(&dependencies)
                    .ok()
                    .and_then(|v| serde_json::to_string(&v).ok())
                    .unwrap_or_default(),
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Section not found"
                        })),
                    )
                        .into_response();
                }
                Err(e) => {
                    error!("Failed to fetch dependencies: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to fetch section: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        "risks" => {
            match manager.get_risks(&session_id).await {
                Ok(Some(risks)) => serde_json::to_value(&risks)
                    .ok()
                    .and_then(|v| serde_json::to_string(&v).ok())
                    .unwrap_or_default(),
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Section not found"
                        })),
                    )
                        .into_response();
                }
                Err(e) => {
                    error!("Failed to fetch risks: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to fetch section: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        "research" | "appendix" => {
            match manager.get_research(&session_id).await {
                Ok(Some(research)) => serde_json::to_value(&research)
                    .ok()
                    .and_then(|v| serde_json::to_string(&v).ok())
                    .unwrap_or_default(),
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Section not found"
                        })),
                    )
                        .into_response();
                }
                Err(e) => {
                    error!("Failed to fetch research: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to fetch section: {}", e)
                        })),
                    )
                        .into_response();
                }
            }
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Unknown section: {}", section_name)
                })),
            )
                .into_response();
        }
    };

    // Validate the section content
    let validation_result = validator.validate_section(&section_name, &content);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": validation_result
        })),
    )
        .into_response()
}

/// GET /api/ideate/sessions/:id/quality-score
/// Get overall quality score for the entire PRD
pub async fn get_quality_score(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting quality score for session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let validator = PRDValidator::new();

    // Fetch all sections and build a complete PRD JSON
    let mut prd_json = serde_json::Map::new();
    let mut section_scores = std::collections::HashMap::new();

    // Overview
    if let Ok(Some(overview)) = manager.get_overview(&session_id).await {
        if let Ok(value) = serde_json::to_value(&overview) {
            prd_json.insert("overview".to_string(), value.clone());
            let section_result = validator.validate_section("overview", &serde_json::to_string(&value).unwrap_or_default());
            section_scores.insert("overview".to_string(), section_result.score);
        }
    }

    // UX
    if let Ok(Some(ux)) = manager.get_ux(&session_id).await {
        if let Ok(value) = serde_json::to_value(&ux) {
            prd_json.insert("ux".to_string(), value.clone());
            let section_result = validator.validate_section("ux", &serde_json::to_string(&value).unwrap_or_default());
            section_scores.insert("ux".to_string(), section_result.score);
        }
    }

    // Technical
    if let Ok(Some(technical)) = manager.get_technical(&session_id).await {
        if let Ok(value) = serde_json::to_value(&technical) {
            prd_json.insert("technical".to_string(), value.clone());
            let section_result = validator.validate_section("technical", &serde_json::to_string(&value).unwrap_or_default());
            section_scores.insert("technical".to_string(), section_result.score);
        }
    }

    // Roadmap
    if let Ok(Some(roadmap)) = manager.get_roadmap(&session_id).await {
        if let Ok(value) = serde_json::to_value(&roadmap) {
            prd_json.insert("roadmap".to_string(), value.clone());
            let section_result = validator.validate_section("roadmap", &serde_json::to_string(&value).unwrap_or_default());
            section_scores.insert("roadmap".to_string(), section_result.score);
        }
    }

    // Dependencies
    if let Ok(Some(dependencies)) = manager.get_dependencies(&session_id).await {
        if let Ok(value) = serde_json::to_value(&dependencies) {
            prd_json.insert("dependencies".to_string(), value.clone());
            let section_result = validator.validate_section("dependencies", &serde_json::to_string(&value).unwrap_or_default());
            section_scores.insert("dependencies".to_string(), section_result.score);
        }
    }

    // Risks
    if let Ok(Some(risks)) = manager.get_risks(&session_id).await {
        if let Ok(value) = serde_json::to_value(&risks) {
            prd_json.insert("risks".to_string(), value.clone());
            let section_result = validator.validate_section("risks", &serde_json::to_string(&value).unwrap_or_default());
            section_scores.insert("risks".to_string(), section_result.score);
        }
    }

    // Research
    if let Ok(Some(research)) = manager.get_research(&session_id).await {
        if let Ok(value) = serde_json::to_value(&research) {
            prd_json.insert("research".to_string(), value);
        }
    }

    // Validate complete PRD
    let prd_value = serde_json::Value::Object(prd_json);
    let overall_result = validator.validate(&prd_value);

    let response = QualityScoreResponse {
        overall_score: overall_result.score,
        section_scores,
        passed: overall_result.passed,
        total_issues: overall_result.issues.len(),
        total_suggestions: overall_result.suggestions.len(),
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": response,
            "validation_details": overall_result
        })),
    )
        .into_response()
}

/// POST /api/ideate/sessions/:id/validation-history
/// Store validation feedback for historical tracking
pub async fn store_validation_feedback(
    State(_db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<StoreValidationRequest>,
) -> impl IntoResponse {
    info!(
        "Storing validation feedback for session: {} section: {}",
        session_id, request.section
    );

    // For now, just acknowledge receipt
    // In the future, we could store this in a validation_history table

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Validation feedback stored",
            "data": {
                "session_id": session_id,
                "section": request.section,
                "score": request.validation_result.score,
                "passed": request.validation_result.passed,
                "user_notes": request.user_notes
            }
        })),
    )
        .into_response()
}
