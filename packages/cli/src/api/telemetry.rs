// ABOUTME: API endpoints for telemetry settings and management
// ABOUTME: Provides REST API for frontend to manage telemetry preferences

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::telemetry::{TelemetryManager, TelemetrySettings};

#[derive(Debug, Serialize)]
pub struct TelemetryStatusResponse {
    pub first_run: bool,
    pub onboarding_completed: bool,
    pub telemetry_enabled: bool,
    pub settings: TelemetrySettingsResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetrySettingsResponse {
    pub error_reporting: bool,
    pub usage_metrics: bool,
    pub non_anonymous_metrics: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTelemetrySettingsRequest {
    pub error_reporting: bool,
    pub usage_metrics: bool,
    pub non_anonymous_metrics: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// GET /api/telemetry/status
/// Returns the current telemetry status and whether to show onboarding
pub async fn get_telemetry_status(
    Extension(telemetry_manager): Extension<Arc<TelemetryManager>>,
) -> impl IntoResponse {
    let settings = telemetry_manager.get_settings().await;
    let should_show_onboarding = telemetry_manager.should_show_onboarding().await;

    let response = TelemetryStatusResponse {
        first_run: settings.first_run,
        onboarding_completed: settings.onboarding_completed,
        telemetry_enabled: telemetry_manager.is_telemetry_enabled(),
        settings: TelemetrySettingsResponse {
            error_reporting: settings.error_reporting,
            usage_metrics: settings.usage_metrics,
            non_anonymous_metrics: settings.non_anonymous_metrics,
        },
    };

    Json(ApiResponse::success(response))
}

/// GET /api/telemetry/settings
/// Returns the current telemetry settings
pub async fn get_telemetry_settings(
    Extension(telemetry_manager): Extension<Arc<TelemetryManager>>,
) -> impl IntoResponse {
    let settings = telemetry_manager.get_settings().await;

    let response = TelemetrySettingsResponse {
        error_reporting: settings.error_reporting,
        usage_metrics: settings.usage_metrics,
        non_anonymous_metrics: settings.non_anonymous_metrics,
    };

    Json(ApiResponse::success(response))
}

/// PUT /api/telemetry/settings
/// Updates telemetry settings
pub async fn update_telemetry_settings(
    Extension(telemetry_manager): Extension<Arc<TelemetryManager>>,
    Json(request): Json<UpdateTelemetrySettingsRequest>,
) -> impl IntoResponse {
    let mut settings = telemetry_manager.get_settings().await;

    // Update settings
    settings.error_reporting = request.error_reporting;
    settings.usage_metrics = request.usage_metrics;
    settings.non_anonymous_metrics = request.non_anonymous_metrics;

    // Generate machine ID if enabling telemetry for the first time
    if (request.error_reporting || request.usage_metrics) && settings.machine_id.is_none() {
        settings.machine_id = Some(uuid::Uuid::new_v4().to_string());
    }

    // Save settings
    match telemetry_manager.update_settings(settings).await {
        Ok(_) => {
            info!("Telemetry settings updated successfully");
            Json(ApiResponse::success(TelemetrySettingsResponse {
                error_reporting: request.error_reporting,
                usage_metrics: request.usage_metrics,
                non_anonymous_metrics: request.non_anonymous_metrics,
            }))
        }
        Err(e) => {
            error!("Failed to update telemetry settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to update settings: {}", e))),
            )
                .into_response()
        }
    }
}

/// POST /api/telemetry/onboarding/complete
/// Marks onboarding as complete and sets initial preferences
pub async fn complete_telemetry_onboarding(
    Extension(telemetry_manager): Extension<Arc<TelemetryManager>>,
    Json(request): Json<UpdateTelemetrySettingsRequest>,
) -> impl IntoResponse {
    match telemetry_manager
        .complete_onboarding(
            request.error_reporting,
            request.usage_metrics,
            request.non_anonymous_metrics,
        )
        .await
    {
        Ok(_) => {
            info!("Telemetry onboarding completed");
            Json(ApiResponse::success(TelemetrySettingsResponse {
                error_reporting: request.error_reporting,
                usage_metrics: request.usage_metrics,
                non_anonymous_metrics: request.non_anonymous_metrics,
            }))
        }
        Err(e) => {
            error!("Failed to complete telemetry onboarding: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to complete onboarding: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}

/// DELETE /api/telemetry/data
/// Deletes all telemetry data
pub async fn delete_telemetry_data(
    Extension(pool): Extension<sqlx::SqlitePool>,
) -> impl IntoResponse {
    // Delete all telemetry events
    match sqlx::query!("DELETE FROM telemetry_events")
        .execute(&pool)
        .await
    {
        Ok(result) => {
            info!("Deleted {} telemetry events", result.rows_affected());

            // Reset statistics
            let _ = sqlx::query!("DELETE FROM telemetry_stats")
                .execute(&pool)
                .await;

            Json(ApiResponse::success(format!(
                "Deleted {} telemetry events",
                result.rows_affected()
            )))
        }
        Err(e) => {
            error!("Failed to delete telemetry data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to delete telemetry data: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}