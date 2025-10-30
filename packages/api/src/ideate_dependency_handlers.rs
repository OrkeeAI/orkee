// ABOUTME: HTTP request handlers for PRD dependency analysis and build optimization
// ABOUTME: Handles dependency CRUD, AI analysis, build order optimization, and visibility analysis

use ai::AIService;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use ideate::{
    BuildOptimizer, CreateDependencyInput, DependencyAnalyzer, DependencyStrength, DependencyType,
    OptimizationStrategy,
};
use orkee_projects::{DbState, StorageError};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};

// TODO: Replace with proper user authentication
const DEFAULT_USER_ID: &str = "default-user";

/// Request body for creating a dependency
#[derive(Deserialize)]
pub struct CreateDependencyRequest {
    #[serde(rename = "fromFeatureId")]
    pub from_feature_id: String,
    #[serde(rename = "toFeatureId")]
    pub to_feature_id: String,
    #[serde(rename = "dependencyType")]
    pub dependency_type: DependencyType,
    pub strength: DependencyStrength,
    pub reason: Option<String>,
}

/// Request body for build order optimization
#[derive(Deserialize)]
pub struct OptimizeBuildOrderRequest {
    pub strategy: OptimizationStrategy,
}

/// Get all dependencies for a session
pub async fn get_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting dependencies for session: {}", session_id);

    let ai_service = AIService::new();
    let analyzer = DependencyAnalyzer::new(db.pool.clone(), ai_service);

    let result = analyzer.get_dependencies(&session_id).await;
    ok_or_internal_error(result, "Failed to get dependencies")
}

/// Create a manual dependency
pub async fn create_dependency(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<CreateDependencyRequest>,
) -> impl IntoResponse {
    info!(
        "Creating dependency for session {}: {} -> {}",
        session_id, request.from_feature_id, request.to_feature_id
    );

    let ai_service = AIService::new();
    let analyzer = DependencyAnalyzer::new(db.pool.clone(), ai_service);

    let input = CreateDependencyInput {
        from_feature_id: request.from_feature_id,
        to_feature_id: request.to_feature_id,
        dependency_type: request.dependency_type,
        strength: request.strength,
        reason: request.reason,
    };

    let result = analyzer.create_dependency(&session_id, input).await;
    created_or_internal_error(result, "Failed to create dependency")
}

/// Delete a dependency
pub async fn delete_dependency(
    State(db): State<DbState>,
    Path((_session_id, dependency_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Deleting dependency: {}", dependency_id);

    let ai_service = AIService::new();
    let analyzer = DependencyAnalyzer::new(db.pool.clone(), ai_service);

    let result = analyzer.delete_dependency(&dependency_id).await;
    ok_or_internal_error(result, "Failed to delete dependency")
}

/// Analyze dependencies using AI
pub async fn analyze_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Analyzing dependencies for session: {}", session_id);

    // Get user's API key from database
    let user = match db.user_storage.get_user(DEFAULT_USER_ID).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Failed to retrieve user: {}", e);
            #[derive(Serialize)]
            struct ErrorResponse {
                error: String,
            }
            return ok_or_internal_error::<ErrorResponse, StorageError>(
                Err(e),
                "Failed to retrieve user",
            );
        }
    };

    let api_key = match user.anthropic_api_key {
        Some(key) => key,
        None => {
            warn!("No Anthropic API key found for user");
            #[derive(Serialize)]
            struct ApiKeyError {
                error: String,
            }
            return ok_or_internal_error::<ApiKeyError, ideate::IdeateError>(
                Err(ideate::IdeateError::AIService(
                    "No API key configured. Please add your Anthropic API key in Settings."
                        .to_string(),
                )),
                "API key required",
            );
        }
    };

    let ai_service = AIService::with_api_key(api_key);
    let analyzer = DependencyAnalyzer::new(db.pool.clone(), ai_service);

    let result = analyzer.analyze_dependencies(&session_id).await;
    ok_or_internal_error(result, "Failed to analyze dependencies")
}

/// Optimize build order
pub async fn optimize_build_order(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<OptimizeBuildOrderRequest>,
) -> impl IntoResponse {
    info!(
        "Optimizing build order for session: {} (strategy: {:?})",
        session_id, request.strategy
    );

    let optimizer = BuildOptimizer::new(db.pool.clone());
    let result = optimizer.optimize(&session_id, request.strategy).await;
    ok_or_internal_error(result, "Failed to optimize build order")
}

/// Get build order
pub async fn get_build_order(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting build order for session: {}", session_id);

    let optimizer = BuildOptimizer::new(db.pool.clone());
    let result = optimizer.get_build_order(&session_id).await;
    ok_or_not_found(result, "Build order not found")
}

/// Get circular dependencies
pub async fn get_circular_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting circular dependencies for session: {}", session_id);

    let optimizer = BuildOptimizer::new(db.pool.clone());
    let result = optimizer.get_circular_dependencies(&session_id).await;
    ok_or_internal_error(result, "Failed to get circular dependencies")
}

/// Suggest quick-win features (high value, low dependency)
pub async fn suggest_quick_wins(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Suggesting quick-win features for session: {}", session_id);

    #[derive(Serialize)]
    struct EmptyResponse {
        quick_wins: Vec<String>,
    }

    // Get user's API key from database
    let user = match db.user_storage.get_user(DEFAULT_USER_ID).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Failed to retrieve user: {}", e);
            return ok_or_internal_error::<EmptyResponse, StorageError>(
                Err(e),
                "Failed to retrieve user",
            );
        }
    };

    let api_key = match user.anthropic_api_key {
        Some(key) => key,
        None => {
            warn!("No Anthropic API key found for user");
            return ok_or_internal_error::<EmptyResponse, StorageError>(
                Ok(EmptyResponse { quick_wins: vec![] }),
                "API key required",
            );
        }
    };

    let ai_service = AIService::with_api_key(api_key);
    let analyzer = DependencyAnalyzer::new(db.pool.clone(), ai_service);

    // Get dependencies first
    let dependencies = match analyzer.get_dependencies(&session_id).await {
        Ok(deps) => deps,
        Err(e) => {
            return ok_or_internal_error(Err::<Vec<String>, _>(e), "Failed to get dependencies");
        }
    };

    // Simple heuristic: features with 0-1 dependencies are quick wins
    let quick_wins: Vec<String> = dependencies
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, dep| {
            *acc.entry(&dep.from_feature_id).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .filter(|(_, count)| *count <= 1)
        .map(|(id, _)| id.clone())
        .collect();

    #[derive(Serialize)]
    struct QuickWinsResponse {
        quick_wins: Vec<String>,
    }

    ok_or_internal_error::<QuickWinsResponse, ideate::IdeateError>(
        Ok(QuickWinsResponse { quick_wins }),
        "Failed to suggest quick wins",
    )
}
