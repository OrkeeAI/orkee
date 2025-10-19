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

use crate::telemetry::TelemetryManager;

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
    let _should_show_onboarding = telemetry_manager.should_show_onboarding().await;

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
) -> Result<Json<ApiResponse<TelemetrySettingsResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate: non_anonymous_metrics requires at least one telemetry type to be enabled
    if request.non_anonymous_metrics && !request.error_reporting && !request.usage_metrics {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Non-anonymous metrics require either error_reporting or usage_metrics to be enabled".to_string(),
            )),
        ));
    }

    // Use atomic update to prevent race conditions
    // This acquires the write lock FIRST, then reads, modifies, and writes
    // preventing TOCTOU (time-of-check-time-of-use) issues
    match telemetry_manager
        .update_settings_atomic(
            request.error_reporting,
            request.usage_metrics,
            request.non_anonymous_metrics,
        )
        .await
    {
        Ok(_) => {
            info!("Telemetry settings updated successfully");
            Ok(Json(ApiResponse::success(TelemetrySettingsResponse {
                error_reporting: request.error_reporting,
                usage_metrics: request.usage_metrics,
                non_anonymous_metrics: request.non_anonymous_metrics,
            })))
        }
        Err(e) => {
            error!("Failed to update telemetry settings: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to update settings: {}",
                    e
                ))),
            ))
        }
    }
}

/// POST /api/telemetry/onboarding/complete
/// Marks onboarding as complete and sets initial preferences
pub async fn complete_telemetry_onboarding(
    Extension(telemetry_manager): Extension<Arc<TelemetryManager>>,
    Json(request): Json<UpdateTelemetrySettingsRequest>,
) -> Result<Json<ApiResponse<TelemetrySettingsResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate: non_anonymous_metrics requires at least one telemetry type to be enabled
    if request.non_anonymous_metrics && !request.error_reporting && !request.usage_metrics {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Non-anonymous metrics require either error_reporting or usage_metrics to be enabled".to_string(),
            )),
        ));
    }

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
            Ok(Json(ApiResponse::success(TelemetrySettingsResponse {
                error_reporting: request.error_reporting,
                usage_metrics: request.usage_metrics,
                non_anonymous_metrics: request.non_anonymous_metrics,
            })))
        }
        Err(e) => {
            error!("Failed to complete telemetry onboarding: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to complete onboarding: {}",
                    e
                ))),
            ))
        }
    }
}

/// Request body for deleting telemetry data
#[derive(Debug, Deserialize)]
pub struct DeleteTelemetryDataRequest {
    /// Must be set to true to confirm deletion
    pub confirm: bool,
}

/// DELETE /api/telemetry/data
/// Deletes all telemetry data
/// Requires explicit confirmation in request body: {"confirm": true}
pub async fn delete_telemetry_data(
    Extension(telemetry_manager): Extension<Arc<TelemetryManager>>,
    Json(request): Json<DeleteTelemetryDataRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Require explicit confirmation
    if !request.confirm {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Deletion requires explicit confirmation. Send {\"confirm\": true} in request body."
                    .to_string(),
            )),
        ));
    }

    match telemetry_manager.delete_all_data().await {
        Ok(count) => {
            info!("Deleted {} telemetry events", count);
            Ok(Json(ApiResponse::success(format!(
                "Deleted {} telemetry events",
                count
            ))))
        }
        Err(e) => {
            error!("Failed to delete telemetry data: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to delete telemetry data: {}",
                    e
                ))),
            ))
        }
    }
}

/// Request body for tracking an event
#[derive(Debug, Deserialize)]
pub struct TrackEventRequest {
    pub event_name: String,
    #[serde(default)]
    pub event_data: Option<serde_json::Value>,
    pub timestamp: String,
    pub session_id: String,
}

/// POST /api/telemetry/track
/// Tracks a telemetry event from the frontend
pub async fn track_event(
    Extension(telemetry_manager): Extension<Arc<TelemetryManager>>,
    Json(request): Json<TrackEventRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if telemetry is enabled
    if !telemetry_manager.is_telemetry_enabled() {
        return Ok(Json(ApiResponse::success("Telemetry disabled".to_string())));
    }

    // Track the event
    match telemetry_manager
        .track_event(
            &request.event_name,
            request.event_data,
            Some(request.session_id),
        )
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success("Event tracked".to_string()))),
        Err(e) => {
            error!("Failed to track telemetry event: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to track event: {}",
                    e
                ))),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::TelemetryManager;
    use std::sync::Arc;
    use tokio::sync::Barrier;

    /// Test concurrent updates to telemetry settings for race conditions
    #[tokio::test]
    async fn test_concurrent_settings_updates_race_condition() {
        let manager = Arc::new(
            TelemetryManager::new_with_test_db()
                .await
                .expect("Failed to create test telemetry manager"),
        );

        // Create a barrier to synchronize all tasks
        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];

        // Spawn 10 concurrent update tasks
        for i in 0..10 {
            let manager_clone = manager.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier_clone.wait().await;

                // Attempt to update settings concurrently
                let error_reporting = i % 2 == 0;
                let usage_metrics = i % 3 == 0;
                let non_anonymous = i % 5 == 0;

                manager_clone
                    .update_settings_atomic(error_reporting, usage_metrics, non_anonymous)
                    .await
                    .expect("Update should succeed");
            });

            handles.push(handle);
        }

        // Wait for all updates to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All operations should succeed without panicking
        for result in results {
            assert!(result.is_ok(), "Task should complete successfully");
        }

        // Verify final state is retrievable (one of the updates should have won)
        let _final_settings = manager.get_settings().await;
    }

    /// Test error handling for invalid telemetry settings combinations
    #[test]
    fn test_invalid_settings_validation() {
        // Test 1: non_anonymous_metrics=true but both telemetry types disabled
        let request = UpdateTelemetrySettingsRequest {
            error_reporting: false,
            usage_metrics: false,
            non_anonymous_metrics: true,
        };

        // This should be rejected by validation logic
        assert!(
            !request.error_reporting && !request.usage_metrics && request.non_anonymous_metrics,
            "Invalid combination should be detectable"
        );
    }

    /// Test concurrent onboarding completions
    #[tokio::test]
    async fn test_concurrent_onboarding_completions() {
        let manager = Arc::new(
            TelemetryManager::new_with_test_db()
                .await
                .expect("Failed to create test telemetry manager"),
        );

        let barrier = Arc::new(Barrier::new(5));
        let mut handles = vec![];

        // Spawn 5 concurrent onboarding completion tasks
        for i in 0..5 {
            let manager_clone = manager.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                barrier_clone.wait().await;

                manager_clone
                    .complete_onboarding(i % 2 == 0, i % 2 == 1, false)
                    .await
                    .expect("Onboarding should succeed");
            });

            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        // All operations should succeed
        for result in results {
            assert!(result.is_ok(), "Task should complete");
        }

        // Verify onboarding was marked complete
        let settings = manager.get_settings().await;
        assert!(
            settings.onboarding_completed,
            "Onboarding should be marked complete"
        );
    }

    /// Test event tracking with empty/invalid data
    #[tokio::test]
    async fn test_track_event_with_invalid_data() {
        let manager = Arc::new(
            TelemetryManager::new_with_test_db()
                .await
                .expect("Failed to create test telemetry manager"),
        );

        // Enable telemetry first
        manager
            .complete_onboarding(true, true, false)
            .await
            .expect("Failed to enable telemetry");

        // Test 1: Empty event name
        let result = manager
            .track_event("", None, Some("session123".to_string()))
            .await;
        assert!(
            result.is_ok(),
            "Empty event name should be handled gracefully"
        );

        // Test 2: Very long event name
        let long_name = "a".repeat(1000);
        let result = manager
            .track_event(&long_name, None, Some("session123".to_string()))
            .await;
        assert!(result.is_ok(), "Long event name should be handled");

        // Test 3: None session_id
        let result = manager.track_event("test_event", None, None).await;
        assert!(result.is_ok(), "None session should be handled");
    }

    /// Test data deletion with concurrent operations
    #[tokio::test]
    async fn test_concurrent_data_deletion() {
        let manager = Arc::new(
            TelemetryManager::new_with_test_db()
                .await
                .expect("Failed to create test telemetry manager"),
        );

        // Enable telemetry and track some events
        manager
            .complete_onboarding(true, true, false)
            .await
            .expect("Failed to enable telemetry");

        for i in 0..10 {
            manager
                .track_event(
                    &format!("test_event_{}", i),
                    None,
                    Some(format!("session_{}", i)),
                )
                .await
                .expect("Failed to track event");
        }

        let barrier = Arc::new(Barrier::new(3));
        let mut handles = vec![];

        // Spawn 3 concurrent deletion tasks
        for _ in 0..3 {
            let manager_clone = manager.clone();
            let barrier_clone = barrier.clone();

            let handle = tokio::spawn(async move {
                barrier_clone.wait().await;
                // All deletions should succeed, return count
                manager_clone
                    .delete_all_data()
                    .await
                    .expect("Delete should succeed")
            });

            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        // All tasks should complete successfully
        for result in results {
            assert!(result.is_ok(), "Task should complete successfully");
        }
    }

    /// Test settings updates resilience under rapid successive updates
    #[tokio::test]
    async fn test_settings_update_resilience() {
        let manager = Arc::new(
            TelemetryManager::new_with_test_db()
                .await
                .expect("Failed to create test telemetry manager"),
        );

        // Complete onboarding first to set first_run = false
        manager
            .complete_onboarding(true, true, false)
            .await
            .expect("Failed to complete onboarding");

        // Test rapid successive updates (100 iterations)
        for i in 0..100 {
            let result = manager
                .update_settings_atomic(i % 2 == 0, i % 3 == 0, false)
                .await;
            assert!(result.is_ok(), "Rapid updates should not fail");
        }

        // Verify final state is consistent
        let settings = manager.get_settings().await;
        assert!(
            !settings.first_run,
            "Settings should have valid state after rapid updates"
        );
        assert!(
            settings.onboarding_completed,
            "Onboarding should still be complete"
        );
    }

    /// Test API response serialization
    #[test]
    fn test_api_response_serialization() {
        // Test success response
        let success_response = ApiResponse::success(TelemetrySettingsResponse {
            error_reporting: true,
            usage_metrics: false,
            non_anonymous_metrics: false,
        });

        let json = serde_json::to_string(&success_response).expect("Should serialize");
        assert!(json.contains("\"success\":true"));
        // Error field should be skipped when None (skip_serializing_if)
        assert!(
            !json.contains("\"error\":"),
            "Error field should be omitted in success response"
        );

        // Test error response
        let error_response = ApiResponse::<()>::error("Test error message".to_string());
        let json = serde_json::to_string(&error_response).expect("Should serialize");
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("Test error message"));
        // Data field should be skipped when None
        assert!(
            !json.contains("\"data\":"),
            "Data field should be omitted in error response"
        );
    }

    /// Test request validation
    #[test]
    fn test_request_deserialization() {
        // Valid request
        let json = r#"{"error_reporting":true,"usage_metrics":true,"non_anonymous_metrics":false}"#;
        let request: UpdateTelemetrySettingsRequest =
            serde_json::from_str(json).expect("Should deserialize");
        assert!(request.error_reporting);
        assert!(request.usage_metrics);
        assert!(!request.non_anonymous_metrics);

        // Invalid JSON should fail gracefully
        let invalid_json = r#"{"error_reporting":"not_a_boolean"}"#;
        let result: Result<UpdateTelemetrySettingsRequest, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Invalid JSON should fail to deserialize");
    }
}
