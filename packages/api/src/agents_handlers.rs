// ABOUTME: HTTP request handlers for agent operations
// ABOUTME: Handles agent listing (from JSON) and user-agent management (from DB)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::ok_or_internal_error;
use orkee_models::REGISTRY;
use orkee_projects::pagination::{PaginatedResponse, PaginationParams};
use orkee_projects::DbState;

/// List all available agents from JSON configuration
pub async fn list_agents(Query(pagination): Query<PaginationParams>) -> impl IntoResponse {
    info!("Listing all agents from JSON registry");

    let mut agents = REGISTRY.list_agents();

    // Sort agents by name for consistent ordering
    agents.sort_by(|a, b| a.name.cmp(&b.name));

    let total = agents.len() as i64;
    let offset = pagination.offset() as usize;
    let limit = pagination.limit() as usize;

    // Apply pagination
    let paginated_agents: Vec<_> = agents
        .into_iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();

    let response = PaginatedResponse::new(paginated_agents, &pagination, total);
    (StatusCode::OK, Json(response))
}

/// Get a single agent by ID from JSON configuration
pub async fn get_agent(Path(agent_id): Path<String>) -> impl IntoResponse {
    info!("Getting agent: {} from JSON registry", agent_id);

    match REGISTRY.get_agent(&agent_id) {
        Some(agent) => (StatusCode::OK, Json(agent.clone())).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found"})),
        )
            .into_response(),
    }
}

/// List user's agent configurations
pub async fn list_user_agents(
    State(db): State<DbState>,
    Path(user_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Listing agents for user: {} (page: {})",
        user_id,
        pagination.page()
    );

    let result = db
        .agent_storage
        .list_user_agents_paginated(
            &user_id,
            Some(pagination.limit()),
            Some(pagination.offset()),
        )
        .await
        .map(|(user_agents, total)| PaginatedResponse::new(user_agents, &pagination, total));

    ok_or_internal_error(result, "Failed to list user agents")
}

/// Get a specific user-agent configuration
pub async fn get_user_agent(
    State(db): State<DbState>,
    Path((user_id, agent_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting user-agent: {} for user: {}", agent_id, user_id);

    let result = db.agent_storage.get_user_agent(&user_id, &agent_id).await;
    ok_or_internal_error(result, "Failed to get user agent")
}

/// Request body for activating an agent
#[derive(Deserialize)]
pub struct ActivateAgentRequest {
    #[serde(rename = "isActive")]
    pub is_active: bool,
}

/// Activate or deactivate an agent for a user
pub async fn update_agent_activation(
    State(db): State<DbState>,
    Path((user_id, agent_id)): Path<(String, String)>,
    Json(request): Json<ActivateAgentRequest>,
) -> impl IntoResponse {
    if request.is_active {
        info!("Activating agent {} for user {}", agent_id, user_id);
        let result = db
            .agent_storage
            .activate_agent(&user_id, &agent_id)
            .await
            .map(|_| serde_json::json!({"message": "Agent activated successfully"}));
        ok_or_internal_error(result, "Failed to activate agent")
    } else {
        info!("Deactivating agent {} for user {}", agent_id, user_id);
        let result = db
            .agent_storage
            .deactivate_agent(&user_id, &agent_id)
            .await
            .map(|_| serde_json::json!({"message": "Agent deactivated successfully"}));
        ok_or_internal_error(result, "Failed to deactivate agent")
    }
}
