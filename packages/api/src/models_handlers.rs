// ABOUTME: HTTP request handlers for AI model operations
// ABOUTME: Handles model listing from JSON configuration with pricing information

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use tracing::info;

use models::REGISTRY;
use orkee_projects::pagination::{PaginatedResponse, PaginationParams};

/// Model DTO for frontend with normalized field names
#[derive(Debug, Clone, Serialize)]
pub struct ModelDTO {
    pub id: String,
    pub provider: String,
    pub model: String,  // model_identifier from config
    pub display_name: String,  // name from config
    pub description: String,
    pub cost_per_1k_input_tokens: f64,
    pub cost_per_1k_output_tokens: f64,
    pub max_context_tokens: u64,
    pub is_available: bool,
}

impl From<&models::Model> for ModelDTO {
    fn from(model: &models::Model) -> Self {
        ModelDTO {
            id: model.id.clone(),
            provider: model.provider.clone(),
            model: model.model_identifier.clone(),
            display_name: model.name.clone(),
            description: model.description.clone(),
            // Convert from per-million to per-thousand
            cost_per_1k_input_tokens: model.pricing.input_per_million_tokens / 1000.0,
            cost_per_1k_output_tokens: model.pricing.output_per_million_tokens / 1000.0,
            max_context_tokens: model.max_context_tokens,
            is_available: model.is_available,
        }
    }
}

/// List all available models from JSON configuration
pub async fn list_models(Query(pagination): Query<PaginationParams>) -> impl IntoResponse {
    info!("Listing all models from JSON registry");

    let mut models = REGISTRY.list_models();

    // Sort models by provider then name for consistent ordering
    models.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.name.cmp(&b.name))
    });

    let total = models.len() as i64;
    let offset = pagination.offset() as usize;
    let limit = pagination.limit() as usize;

    // Apply pagination and convert to DTO
    let paginated_models: Vec<ModelDTO> = models
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(ModelDTO::from)
        .collect();

    let response = PaginatedResponse::new(paginated_models, &pagination, total);
    (StatusCode::OK, Json(response))
}

/// Get a single model by ID from JSON configuration
pub async fn get_model(Path(model_id): Path<String>) -> impl IntoResponse {
    info!("Getting model: {} from JSON registry", model_id);

    match REGISTRY.get_model(&model_id) {
        Some(model) => {
            let dto = ModelDTO::from(model);
            (StatusCode::OK, Json(dto)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Model not found"})),
        )
            .into_response(),
    }
}

/// List models for a specific provider
pub async fn list_models_by_provider(
    Path(provider): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Listing models for provider: {}", provider);

    let mut models = REGISTRY.get_models_by_provider(&provider);

    // Sort by name for consistent ordering
    models.sort_by(|a, b| a.name.cmp(&b.name));

    let total = models.len() as i64;
    let offset = pagination.offset() as usize;
    let limit = pagination.limit() as usize;

    // Apply pagination and convert to DTO
    let paginated_models: Vec<ModelDTO> = models
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(ModelDTO::from)
        .collect();

    let response = PaginatedResponse::new(paginated_models, &pagination, total);
    (StatusCode::OK, Json(response))
}
