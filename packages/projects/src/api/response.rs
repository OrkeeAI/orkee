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
