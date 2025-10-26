// ABOUTME: Shared API response types and error handling
// ABOUTME: Provides consistent response format across all API endpoints

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::Serialize;

use crate::storage::StorageError;

/// Standard API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Convert storage errors to HTTP responses
impl IntoResponse for StorageError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            StorageError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            StorageError::Database(_) | StorageError::Sqlx(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, ResponseJson(ApiResponse::<()>::error(message))).into_response()
    }
}

/// Helper functions to reduce error handling boilerplate in handlers
/// Convert a Result into an HTTP response with OK (200) status on success
/// or INTERNAL_SERVER_ERROR (500) status on failure
pub fn ok_or_internal_error<T, E>(
    result: Result<T, E>,
    error_context: &str,
) -> axum::response::Response
where
    T: serde::Serialize,
    E: std::fmt::Display,
{
    match result {
        Ok(data) => (StatusCode::OK, ResponseJson(ApiResponse::success(data))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "{}: {}",
                error_context, e
            ))),
        )
            .into_response(),
    }
}

/// Convert a Result into an HTTP response with CREATED (201) status on success
/// or INTERNAL_SERVER_ERROR (500) status on failure
pub fn created_or_internal_error<T, E>(
    result: Result<T, E>,
    error_context: &str,
) -> axum::response::Response
where
    T: serde::Serialize,
    E: std::fmt::Display,
{
    match result {
        Ok(data) => (
            StatusCode::CREATED,
            ResponseJson(ApiResponse::success(data)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ApiResponse::<()>::error(format!(
                "{}: {}",
                error_context, e
            ))),
        )
            .into_response(),
    }
}

/// Convert a Result into an HTTP response with OK (200) status on success
/// or NOT_FOUND (404) status on failure
pub fn ok_or_not_found<T, E>(result: Result<T, E>, error_context: &str) -> axum::response::Response
where
    T: serde::Serialize,
    E: std::fmt::Display,
{
    match result {
        Ok(data) => (StatusCode::OK, ResponseJson(ApiResponse::success(data))).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error(format!(
                "{}: {}",
                error_context, e
            ))),
        )
            .into_response(),
    }
}

/// Create a BAD_REQUEST (400) response for validation errors
pub fn bad_request<E>(error: E, error_context: &str) -> axum::response::Response
where
    E: std::fmt::Display,
{
    (
        StatusCode::BAD_REQUEST,
        ResponseJson(ApiResponse::<()>::error(format!(
            "{}: {}",
            error_context, error
        ))),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;

    async fn response_body_to_json(response: axum::response::Response) -> serde_json::Value {
        let (_parts, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_ok_or_internal_error_with_success() {
        let result: Result<String, String> = Ok("test data".to_string());
        let response = ok_or_internal_error(result, "Failed to get data");

        assert_eq!(response.status(), StatusCode::OK);

        let json = response_body_to_json(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["data"], "test data");
        assert_eq!(json["error"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_ok_or_internal_error_with_error() {
        let result: Result<String, String> = Err("database error".to_string());
        let response = ok_or_internal_error(result, "Failed to get data");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = response_body_to_json(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["data"], serde_json::Value::Null);
        assert_eq!(json["error"], "Failed to get data: database error");
    }

    #[tokio::test]
    async fn test_created_or_internal_error_with_success() {
        let result: Result<i32, String> = Ok(42);
        let response = created_or_internal_error(result, "Failed to create resource");

        assert_eq!(response.status(), StatusCode::CREATED);

        let json = response_body_to_json(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["data"], 42);
        assert_eq!(json["error"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_created_or_internal_error_with_error() {
        let result: Result<i32, String> = Err("validation failed".to_string());
        let response = created_or_internal_error(result, "Failed to create resource");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = response_body_to_json(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["data"], serde_json::Value::Null);
        assert_eq!(
            json["error"],
            "Failed to create resource: validation failed"
        );
    }

    #[tokio::test]
    async fn test_ok_or_not_found_with_success() {
        #[derive(serde::Serialize)]
        struct TestData {
            id: i32,
            name: String,
        }

        let result: Result<TestData, String> = Ok(TestData {
            id: 1,
            name: "test".to_string(),
        });
        let response = ok_or_not_found(result, "Resource not found");

        assert_eq!(response.status(), StatusCode::OK);

        let json = response_body_to_json(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["id"], 1);
        assert_eq!(json["data"]["name"], "test");
        assert_eq!(json["error"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_ok_or_not_found_with_error() {
        let result: Result<String, String> = Err("not found in database".to_string());
        let response = ok_or_not_found(result, "Resource not found");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = response_body_to_json(response).await;
        assert_eq!(json["success"], false);
        assert_eq!(json["data"], serde_json::Value::Null);
        assert_eq!(json["error"], "Resource not found: not found in database");
    }
}
