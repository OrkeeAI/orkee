// ABOUTME: HTTP request handlers for PRD dependency management and build optimization
// ABOUTME: Handles dependency CRUD, build order optimization, and visibility analysis

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use orkee_ideate::{
    BuildOptimizer, CreateDependencyInput, DependencyAnalyzer, DependencyStrength, DependencyType,
    OptimizationStrategy,
};
use orkee_projects::DbState;
use serde::Deserialize;
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};

// TODO: AI functionality moved to frontend - see packages/dashboard/src/services/dependency-ai.ts (to be created)

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

    let analyzer = DependencyAnalyzer::new(db.pool.clone());

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

    let analyzer = DependencyAnalyzer::new(db.pool.clone());

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

    let analyzer = DependencyAnalyzer::new(db.pool.clone());

    let result = analyzer.delete_dependency(&dependency_id).await;
    ok_or_internal_error(result, "Failed to delete dependency")
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
