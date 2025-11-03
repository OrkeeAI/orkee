// ABOUTME: HTTP request handlers for model preferences operations
// ABOUTME: Handles per-task AI model configuration for Ideate/PRD/Task features

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::response::ok_or_internal_error;
use orkee_models::REGISTRY;
use orkee_projects::DbState;
use orkee_storage::model_preferences::{ModelPreferences, UpdateTaskModelRequest};

/// Get model preferences for a user
pub async fn get_model_preferences(
    State(db): State<DbState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting model preferences for user: {}", user_id);

    let result = db.model_preferences_storage.get_preferences(&user_id).await;
    ok_or_internal_error(result, "Failed to get model preferences")
}

/// Request body for updating all model preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModelPreferencesRequest {
    // Chat (Ideate mode)
    pub chat_model: String,
    pub chat_provider: String,

    // PRD Generation
    pub prd_generation_model: String,
    pub prd_generation_provider: String,

    // PRD Analysis
    pub prd_analysis_model: String,
    pub prd_analysis_provider: String,

    // Insight Extraction
    pub insight_extraction_model: String,
    pub insight_extraction_provider: String,

    // Spec Generation
    pub spec_generation_model: String,
    pub spec_generation_provider: String,

    // Task Suggestions
    pub task_suggestions_model: String,
    pub task_suggestions_provider: String,

    // Task Analysis
    pub task_analysis_model: String,
    pub task_analysis_provider: String,

    // Spec Refinement
    pub spec_refinement_model: String,
    pub spec_refinement_provider: String,

    // Research Generation
    pub research_generation_model: String,
    pub research_generation_provider: String,

    // Markdown Generation
    pub markdown_generation_model: String,
    pub markdown_generation_provider: String,
}

/// Update all model preferences for a user
pub async fn update_model_preferences(
    State(db): State<DbState>,
    Path(user_id): Path<String>,
    Json(request): Json<UpdateModelPreferencesRequest>,
) -> impl IntoResponse {
    info!("Updating model preferences for user: {}", user_id);

    // Validate all model IDs exist in registry
    let model_ids = vec![
        &request.chat_model,
        &request.prd_generation_model,
        &request.prd_analysis_model,
        &request.insight_extraction_model,
        &request.spec_generation_model,
        &request.task_suggestions_model,
        &request.task_analysis_model,
        &request.spec_refinement_model,
        &request.research_generation_model,
        &request.markdown_generation_model,
    ];

    for model_id in model_ids {
        if REGISTRY.get_model(model_id).is_none() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid model ID: {}", model_id)
                })),
            )
                .into_response();
        }
    }

    // Validate providers
    let providers = vec![
        &request.chat_provider,
        &request.prd_generation_provider,
        &request.prd_analysis_provider,
        &request.insight_extraction_provider,
        &request.spec_generation_provider,
        &request.task_suggestions_provider,
        &request.task_analysis_provider,
        &request.spec_refinement_provider,
        &request.research_generation_provider,
        &request.markdown_generation_provider,
    ];

    for provider in providers {
        if !matches!(provider.as_str(), "anthropic" | "openai" | "google" | "xai") {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid provider: {}", provider)
                })),
            )
                .into_response();
        }
    }

    // Convert to storage type
    let prefs = ModelPreferences {
        user_id: user_id.clone(),
        chat_model: request.chat_model,
        chat_provider: request.chat_provider,
        prd_generation_model: request.prd_generation_model,
        prd_generation_provider: request.prd_generation_provider,
        prd_analysis_model: request.prd_analysis_model,
        prd_analysis_provider: request.prd_analysis_provider,
        insight_extraction_model: request.insight_extraction_model,
        insight_extraction_provider: request.insight_extraction_provider,
        spec_generation_model: request.spec_generation_model,
        spec_generation_provider: request.spec_generation_provider,
        task_suggestions_model: request.task_suggestions_model,
        task_suggestions_provider: request.task_suggestions_provider,
        task_analysis_model: request.task_analysis_model,
        task_analysis_provider: request.task_analysis_provider,
        spec_refinement_model: request.spec_refinement_model,
        spec_refinement_provider: request.spec_refinement_provider,
        research_generation_model: request.research_generation_model,
        research_generation_provider: request.research_generation_provider,
        markdown_generation_model: request.markdown_generation_model,
        markdown_generation_provider: request.markdown_generation_provider,
        updated_at: String::new(), // Will be set by database
    };

    let result = db
        .model_preferences_storage
        .update_preferences(&prefs)
        .await
        .map(|_| serde_json::json!({"message": "Model preferences updated successfully"}));

    ok_or_internal_error(result, "Failed to update model preferences")
}

/// Update model preference for a specific task
pub async fn update_task_model(
    State(db): State<DbState>,
    Path((user_id, task_type)): Path<(String, String)>,
    Json(request): Json<UpdateTaskModelRequest>,
) -> impl IntoResponse {
    info!(
        "Updating {} model for user: {} to {}/{}",
        task_type, user_id, request.provider, request.model
    );

    // Validate model ID
    if REGISTRY.get_model(&request.model).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid model ID: {}", request.model)
            })),
        )
            .into_response();
    }

    // Validate provider
    if !matches!(
        request.provider.as_str(),
        "anthropic" | "openai" | "google" | "xai"
    ) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid provider: {}", request.provider)
            })),
        )
            .into_response();
    }

    // Update the task model and return full preferences
    match db
        .model_preferences_storage
        .update_task_model(&user_id, &task_type, &request)
        .await
    {
        Ok(_) => {
            // Fetch and return the updated preferences
            let result = db.model_preferences_storage.get_preferences(&user_id).await;
            ok_or_internal_error(result, "Failed to get updated preferences")
        }
        Err(e) => {
            let result: Result<ModelPreferences, _> = Err(e);
            ok_or_internal_error(result, "Failed to update task model")
        }
    }
}
