// ABOUTME: HTTP request handlers for AI usage log operations
// ABOUTME: Provides endpoints for querying AI usage statistics and cost tracking

use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::response::{ok_or_internal_error, ApiResponse};
use orkee_ai::usage_logs::{AiUsageLog, AiUsageQuery};
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

#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    pub operation: String,
    pub model: String,
    pub provider: String,
    #[serde(rename = "inputTokens")]
    pub input_tokens: i32,
    #[serde(rename = "outputTokens")]
    pub output_tokens: i32,
    #[serde(rename = "totalTokens")]
    pub total_tokens: i32,
    #[serde(rename = "estimatedCost")]
    pub estimated_cost: f64,
    #[serde(rename = "durationMs")]
    pub duration_ms: i32,
    #[serde(rename = "toolCallsCount")]
    pub tool_calls_count: Option<i32>,
    #[serde(rename = "toolCallsJson")]
    pub tool_calls_json: Option<String>,
    #[serde(rename = "responseMetadata")]
    pub response_metadata: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateLogResponse {
    pub id: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// Create a new AI usage log entry from frontend telemetry
pub async fn create_log(
    State(db): State<DbState>,
    Json(request): Json<CreateLogRequest>,
) -> impl IntoResponse {
    info!(
        "Creating AI usage log (operation: {}, model: {})",
        request.operation, request.model
    );

    // Validate required fields
    if request.operation.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "operation field is required and cannot be empty".to_string(),
            )),
        )
            .into_response();
    }

    if request.model.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "model field is required and cannot be empty".to_string(),
            )),
        )
            .into_response();
    }

    if request.provider.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "provider field is required and cannot be empty".to_string(),
            )),
        )
            .into_response();
    }

    // Validate token counts are non-negative
    if request.input_tokens < 0 || request.output_tokens < 0 || request.total_tokens < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "token counts must be non-negative".to_string(),
            )),
        )
            .into_response();
    }

    // Validate cost is non-negative
    if request.estimated_cost < 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "estimated cost must be non-negative".to_string(),
            )),
        )
            .into_response();
    }

    // Validate duration is non-negative
    if request.duration_ms < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "duration must be non-negative".to_string(),
            )),
        )
            .into_response();
    }

    // Validate tool_calls_json is valid JSON if provided
    if let Some(ref json_str) = request.tool_calls_json {
        if serde_json::from_str::<serde_json::Value>(json_str).is_err() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    "toolCallsJson must be valid JSON".to_string(),
                )),
            )
                .into_response();
        }
    }

    // Validate response_metadata is valid JSON if provided
    if let Some(ref json_str) = request.response_metadata {
        if serde_json::from_str::<serde_json::Value>(json_str).is_err() {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    "responseMetadata must be valid JSON".to_string(),
                )),
            )
                .into_response();
        }
    }

    // Create the usage log
    let now = Utc::now();
    let log_id = nanoid::nanoid!(10);

    let usage_log = AiUsageLog {
        id: log_id.clone(),
        project_id: request.project_id.unwrap_or_else(|| "unknown".to_string()),
        request_id: request.request_id,
        operation: request.operation,
        model: request.model,
        provider: request.provider,
        input_tokens: Some(request.input_tokens as i64),
        output_tokens: Some(request.output_tokens as i64),
        total_tokens: Some(request.total_tokens as i64),
        estimated_cost: Some(request.estimated_cost),
        duration_ms: Some(request.duration_ms as i64),
        error: request.error,
        tool_calls_count: request.tool_calls_count.map(|c| c as i64),
        tool_calls_json: request.tool_calls_json,
        response_metadata: request.response_metadata,
        created_at: now,
    };

    // Save to database
    match db.ai_usage_log_storage.create_log(&usage_log).await {
        Ok(_) => {
            info!("Successfully created AI usage log: {}", log_id);
            let response = CreateLogResponse {
                id: log_id,
                created_at: now,
            };
            (StatusCode::CREATED, Json(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            error!("Failed to create AI usage log: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to save AI usage log: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}

/// Get tool usage statistics
pub async fn get_tool_stats(
    State(db): State<DbState>,
    Query(params): Query<GetStatsQuery>,
) -> impl IntoResponse {
    info!(
        "Getting tool usage stats (project_id: {:?})",
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

    let result = db.ai_usage_log_storage.get_tool_stats(&query).await;
    ok_or_internal_error(result, "Failed to get tool usage stats")
}

#[derive(Deserialize)]
pub struct GetTimeSeriesQuery {
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(rename = "endDate")]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default = "default_interval")]
    pub interval: String, // 'hour', 'day', 'week', 'month'
}

fn default_interval() -> String {
    "day".to_string()
}

/// Get time-series data for usage charts
pub async fn get_time_series(
    State(db): State<DbState>,
    Query(params): Query<GetTimeSeriesQuery>,
) -> impl IntoResponse {
    info!(
        "Getting time-series data (project_id: {:?}, interval: {})",
        params.project_id, params.interval
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

    let result = db
        .ai_usage_log_storage
        .get_time_series(&query, &params.interval)
        .await;
    ok_or_internal_error(result, "Failed to get time-series data")
}
