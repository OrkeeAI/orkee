// ABOUTME: HTTP request handlers for agent operations
// ABOUTME: Handles agent listing and user-agent management

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::ApiResponse;
use crate::db::DbState;
use crate::pagination::{PaginatedResponse, PaginationParams};

/// List all available agents
pub async fn list_agents(
    State(db): State<DbState>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Listing all agents (page: {})", pagination.page());

    match db.agent_storage.list_agents_paginated(Some(pagination.limit()), Some(pagination.offset())).await {
        Ok((agents, total)) => {
            let response = PaginatedResponse::new(agents, &pagination, total);
            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Get a single agent by ID
pub async fn get_agent(
    State(db): State<DbState>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting agent: {}", agent_id);

    match db.agent_storage.get_agent(&agent_id).await {
        Ok(agent) => (StatusCode::OK, ResponseJson(ApiResponse::success(agent))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// List user's agent configurations
pub async fn list_user_agents(
    State(db): State<DbState>,
    Path(user_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Listing agents for user: {} (page: {})", user_id, pagination.page());

    match db.agent_storage.list_user_agents_paginated(&user_id, Some(pagination.limit()), Some(pagination.offset())).await {
        Ok((user_agents, total)) => {
            let response = PaginatedResponse::new(user_agents, &pagination, total);
            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Get a specific user-agent configuration
pub async fn get_user_agent(
    State(db): State<DbState>,
    Path((user_id, agent_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting user-agent: {} for user: {}", agent_id, user_id);

    match db.agent_storage.get_user_agent(&user_id, &agent_id).await {
        Ok(user_agent) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success(user_agent)),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
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
        match db.agent_storage.activate_agent(&user_id, &agent_id).await {
            Ok(()) => (
                StatusCode::OK,
                ResponseJson(ApiResponse::success(serde_json::json!({
                    "message": "Agent activated successfully"
                }))),
            )
                .into_response(),
            Err(e) => e.into_response(),
        }
    } else {
        info!("Deactivating agent {} for user {}", agent_id, user_id);
        match db.agent_storage.deactivate_agent(&user_id, &agent_id).await {
            Ok(()) => (
                StatusCode::OK,
                ResponseJson(ApiResponse::success(serde_json::json!({
                    "message": "Agent deactivated successfully"
                }))),
            )
                .into_response(),
            Err(e) => e.into_response(),
        }
    }
}
