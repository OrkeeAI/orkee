// ABOUTME: API handlers for epic alternative approaches (Phase 6A.3 CCPM)
// ABOUTME: Endpoints for generating, viewing, and selecting technical approaches

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use orkee_ideate::{
    ApproachComparison, ApproachGenerator, CodebaseAnalyzer, CodebaseContext, EpicManager,
    TechnicalApproach,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{error, info};

use orkee_projects::DbState;

/// Response for generated alternatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativesResponse {
    pub approaches: Vec<TechnicalApproach>,
    pub comparison: ApproachComparison,
    pub codebase_analyzed: bool,
}

/// Request body for selecting an approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectApproachRequest {
    pub approach_name: String,
    pub rationale: Option<String>,
}

/// POST /api/epics/:id/generate-alternatives
/// Generate 2-3 alternative technical approaches with trade-off analysis
pub async fn generate_alternatives(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Generating alternative approaches for epic: {}", epic_id);

    let epic_manager = EpicManager::new(db.pool.clone());

    // Fetch the epic
    let epic = match epic_manager.get_epic(&project_id, &epic_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            error!("Epic not found: {}", epic_id);
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Epic not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to fetch epic: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to fetch epic: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Try to get codebase context
    let codebase_context = if let Some(stored_context) = &epic.codebase_context {
        // Try to deserialize stored context
        serde_json::from_value::<CodebaseContext>(stored_context.clone())
            .unwrap_or_else(|_| CodebaseContext::default())
    } else {
        // Try to analyze codebase if we have project info
        match orkee_projects::get_project(&epic.project_id).await {
            Ok(Some(project)) => {
                let analyzer = CodebaseAnalyzer::new(PathBuf::from(project.project_root));

                // Create a temporary IdeateSession-like struct for compatibility
                let temp_session = orkee_ideate::IdeateSession {
                    id: epic_id.clone(),
                    project_id: epic.project_id.clone(),
                    initial_description: epic.overview_markdown.clone(),
                    mode: orkee_ideate::IdeateMode::Quick,
                    status: orkee_ideate::IdeateStatus::Completed,
                    current_section: None,
                    skipped_sections: None,
                    generated_prd_id: Some(epic.prd_id.clone()),
                    research_tools_enabled: false,
                    non_goals: None,
                    open_questions: None,
                    constraints_assumptions: None,
                    success_metrics: None,
                    alternative_approaches: None,
                    validation_checkpoints: None,
                    codebase_context: None,
                    created_at: epic.created_at,
                    updated_at: epic.updated_at,
                };

                match analyzer.analyze_for_session(&temp_session).await {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        info!("Failed to analyze codebase, using default context: {:?}", e);
                        CodebaseContext::default()
                    }
                }
            }
            _ => {
                info!("No project found, using default codebase context");
                CodebaseContext::default()
            }
        }
    };

    // Generate alternative approaches
    let generator = ApproachGenerator::new(epic, codebase_context.clone());
    match generator.generate_alternatives().await {
        Ok(approaches) => {
            let comparison = ApproachGenerator::compare_approaches(&approaches);
            let codebase_analyzed = !codebase_context.patterns.is_empty()
                || !codebase_context.similar_features.is_empty();

            let response = AlternativesResponse {
                approaches,
                comparison,
                codebase_analyzed,
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
            error!("Failed to generate alternatives: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to generate alternatives: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/epics/:id/alternatives
/// Get previously generated alternatives (if cached)
pub async fn get_alternatives(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting cached alternatives for epic: {}", epic_id);

    // For now, regenerate since we don't persist alternatives yet
    // In the future, we could add an alternatives field to the Epic model
    generate_alternatives(State(db), Path((project_id, epic_id))).await
}

/// PUT /api/epics/:id/select-approach
/// Select a preferred technical approach and update epic
pub async fn select_approach(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
    Json(request): Json<SelectApproachRequest>,
) -> impl IntoResponse {
    info!(
        "Selecting approach '{}' for epic: {}",
        request.approach_name, epic_id
    );

    let epic_manager = EpicManager::new(db.pool.clone());

    // Fetch the epic
    let epic = match epic_manager.get_epic(&project_id, &epic_id).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            error!("Epic not found: {}", epic_id);
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Epic not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to fetch epic: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to fetch epic: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Generate alternatives to validate the selection
    let codebase_context = epic
        .codebase_context
        .as_ref()
        .and_then(|v| serde_json::from_value::<CodebaseContext>(v.clone()).ok())
        .unwrap_or_default();

    let generator = ApproachGenerator::new(epic.clone(), codebase_context);
    let approaches = match generator.generate_alternatives().await {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to generate alternatives: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to validate approach selection: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Find the selected approach
    let selected_approach = approaches.iter().find(|a| a.name == request.approach_name);

    if selected_approach.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Approach '{}' not found in generated alternatives", request.approach_name)
            })),
        )
            .into_response();
    }

    let selected = selected_approach.unwrap();

    // Build updated technical_approach text
    let mut technical_approach = format!(
        "**Selected Approach**: {}\n\n{}\n\n**Rationale**: {}",
        selected.name,
        selected.description,
        request.rationale.as_ref().unwrap_or(&selected.reasoning)
    );

    // Add pros and cons
    technical_approach.push_str("\n\n**Pros**:\n");
    for pro in &selected.pros {
        technical_approach.push_str(&format!("- {}\n", pro));
    }

    technical_approach.push_str("\n**Cons**:\n");
    for con in &selected.cons {
        technical_approach.push_str(&format!("- {}\n", con));
    }

    technical_approach.push_str(&format!(
        "\n**Estimated Timeline**: {} days\n**Complexity**: {:?}",
        selected.estimated_days, selected.complexity
    ));

    // Update the epic with the selected approach
    let update_input = orkee_ideate::UpdateEpicInput {
        name: None,
        overview_markdown: None,
        architecture_decisions: None,
        technical_approach: Some(technical_approach.clone()),
        implementation_strategy: None,
        dependencies: None,
        success_criteria: None,
        task_categories: None,
        estimated_effort: None,
        complexity: None,
        status: None,
        progress_percentage: None,
    };

    match epic_manager
        .update_epic(&project_id, &epic_id, update_input)
        .await
    {
        Ok(updated_epic) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "epic": updated_epic,
                    "selected_approach": selected,
                    "technical_approach": technical_approach
                }
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to update epic: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to update epic: {}", e)
                })),
            )
                .into_response()
        }
    }
}
