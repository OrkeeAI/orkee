// ABOUTME: HTTP request handlers for AI usage log operations
// ABOUTME: Provides endpoints for querying AI usage statistics and cost tracking

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use super::response::ok_or_internal_error;
use ai::usage_logs::AiUsageQuery;
use orkee_projects::DbState;

#[derive(Deserialize)]
pub struct ListLogsQuery {
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(rename = "endDate")]
    pub end_date: Option<DateTime<Utc>>,
    pub operation: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List AI usage logs with optional filtering
pub async fn list_logs(
    State(db): State<DbState>,
    Query(params): Query<ListLogsQuery>,
) -> impl IntoResponse {
    info!(
        "Listing AI usage logs (project_id: {:?}, limit: {:?})",
        params.project_id, params.limit
    );

    let query = AiUsageQuery {
        project_id: params.project_id,
        start_date: params.start_date,
        end_date: params.end_date,
        operation: params.operation,
        model: params.model,
        provider: params.provider,
        limit: params.limit,
        offset: params.offset,
    };

    let result = db.ai_usage_log_storage.list_logs(query).await;
    ok_or_internal_error(result, "Failed to list AI usage logs")
}

#[derive(Deserialize)]
pub struct GetStatsQuery {
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(rename = "endDate")]
    pub end_date: Option<DateTime<Utc>>,
}

/// Get aggregate AI usage statistics
pub async fn get_stats(
    State(db): State<DbState>,
    Query(params): Query<GetStatsQuery>,
) -> impl IntoResponse {
    info!(
        "Getting AI usage stats (project_id: {:?})",
        params.project_id
    );

    let query = AiUsageQuery {
        project_id: params.project_id,
        start_date: params.start_date,
        end_date: params.end_date,
        operation: None,
        model: None,
        provider: None,
        limit: None,
        offset: None,
    };

    let result = db.ai_usage_log_storage.get_stats(query).await;
    ok_or_internal_error(result, "Failed to get AI usage stats")
}
