// ABOUTME: HTTP request handlers for agent operations
// ABOUTME: Handles agent listing and user-agent management

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::{ok_or_internal_error};
use crate::db::DbState;
use crate::pagination::{PaginatedResponse, PaginationParams};

/// List all available agents
pub async fn list_agents(
    State(db): State<DbState>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Listing all agents (page: {})", pagination.page());

    let result = db.agent_storage.list_agents_paginated(Some(pagination.limit()), Some(pagination.offset()))
        .await
        .map(|(agents, total)| PaginatedResponse::new(agents, &pagination, total));

    ok_or_internal_error(result, "Failed to list agents")
}

/// Get a single agent by ID
pub async fn get_agent(
    State(db): State<DbState>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting agent: {}", agent_id);

    let result = db.agent_storage.get_agent(&agent_id).await;
    ok_or_internal_error(result, "Failed to get agent")
}

/// List user's agent configurations
pub async fn list_user_agents(
    State(db): State<DbState>,
    Path(user_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Listing agents for user: {} (page: {})", user_id, pagination.page());

    let result = db.agent_storage.list_user_agents_paginated(&user_id, Some(pagination.limit()), Some(pagination.offset()))
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
        let result = db.agent_storage.activate_agent(&user_id, &agent_id)
            .await
            .map(|_| serde_json::json!({"message": "Agent activated successfully"}));
        ok_or_internal_error(result, "Failed to activate agent")
    } else {
        info!("Deactivating agent {} for user {}", agent_id, user_id);
        let result = db.agent_storage.deactivate_agent(&user_id, &agent_id)
            .await
            .map(|_| serde_json::json!({"message": "Agent deactivated successfully"}));
        ok_or_internal_error(result, "Failed to deactivate agent")
    }
}
