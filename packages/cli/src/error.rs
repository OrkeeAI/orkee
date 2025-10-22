use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

/// Main application error type that all handlers should return
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource not found")]
    NotFound,

    #[error("Rate limit exceeded")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Forbidden: {message}")]
    Forbidden { message: String },

    #[error("Access denied to path: {0}")]
    PathAccessDenied(String),

    #[error("Path traversal attempt detected")]
    PathTraversal,

    #[error("Sensitive directory access blocked")]
    SensitiveDirectory,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    /// Wrap storage errors from the projects library
    #[error("Storage error")]
    Storage(#[from] orkee_projects::manager::ManagerError),

    /// Wrap path validation errors
    #[error("Path validation failed")]
    PathValidation(#[from] crate::api::path_validator::ValidationError),
}

/// Structured error response format for API consistency
#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorDetail,
    request_id: String,
}

/// Error detail structure with machine-readable codes
#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<HashMap<String, String>>,
}

impl AppError {
    /// Convert AppError to appropriate HTTP status code and error code
    fn to_status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::RateLimitExceeded { .. } => {
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED")
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::Forbidden { .. } => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::PathAccessDenied(_) => (StatusCode::FORBIDDEN, "PATH_ACCESS_DENIED"),
            AppError::PathTraversal => (StatusCode::FORBIDDEN, "PATH_TRAVERSAL"),
            AppError::SensitiveDirectory => (StatusCode::FORBIDDEN, "SENSITIVE_DIRECTORY"),
            AppError::Configuration(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "CONFIGURATION_ERROR")
            }
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            AppError::Storage(manager_error) => match manager_error {
                orkee_projects::manager::ManagerError::NotFound(_) => {
                    (StatusCode::NOT_FOUND, "RESOURCE_NOT_FOUND")
                }
                orkee_projects::manager::ManagerError::DuplicateName(_) => {
                    (StatusCode::CONFLICT, "DUPLICATE_NAME")
                }
                orkee_projects::manager::ManagerError::DuplicatePath(_) => {
                    (StatusCode::CONFLICT, "DUPLICATE_PATH")
                }
                orkee_projects::manager::ManagerError::Validation(_) => {
                    (StatusCode::BAD_REQUEST, "VALIDATION_ERROR")
                }
                orkee_projects::manager::ManagerError::Storage(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "STORAGE_ERROR")
                }
            },
            AppError::PathValidation(validation_error) => match validation_error {
                crate::api::path_validator::ValidationError::PathTraversal => {
                    (StatusCode::FORBIDDEN, "PATH_TRAVERSAL")
                }
                crate::api::path_validator::ValidationError::RootAccess => {
                    (StatusCode::FORBIDDEN, "ROOT_ACCESS")
                }
                crate::api::path_validator::ValidationError::BlockedPath(_) => {
                    (StatusCode::FORBIDDEN, "BLOCKED_PATH")
                }
                crate::api::path_validator::ValidationError::SensitiveDirectory(_) => {
                    (StatusCode::FORBIDDEN, "SENSITIVE_DIRECTORY")
                }
                crate::api::path_validator::ValidationError::NotInAllowedPaths => {
                    (StatusCode::FORBIDDEN, "NOT_IN_ALLOWED_PATHS")
                }
                crate::api::path_validator::ValidationError::SymlinkError => {
                    (StatusCode::FORBIDDEN, "SYMLINK_ERROR")
                }
                crate::api::path_validator::ValidationError::InvalidPath => {
                    (StatusCode::BAD_REQUEST, "INVALID_PATH")
                }
                crate::api::path_validator::ValidationError::PathDoesNotExist => {
                    (StatusCode::NOT_FOUND, "PATH_NOT_FOUND")
                }
                crate::api::path_validator::ValidationError::ExpansionError => {
                    (StatusCode::BAD_REQUEST, "PATH_EXPANSION_ERROR")
                }
            },
        }
    }

    /// Get user-friendly error message (sanitized for external consumption)
    fn to_user_message(&self) -> String {
        match self {
            AppError::Validation(msg) => format!("Validation failed: {}", msg),
            AppError::NotFound => "The requested resource was not found".to_string(),
            AppError::RateLimitExceeded { .. } => {
                "Too many requests. Please try again later".to_string()
            }
            AppError::Unauthorized => "Authentication required".to_string(),
            AppError::Forbidden { message } => message.clone(),
            AppError::PathAccessDenied(_) => "Access to this path is not allowed".to_string(),
            AppError::PathTraversal => "Path traversal detected and blocked".to_string(),
            AppError::SensitiveDirectory => "Access to sensitive directory blocked".to_string(),
            AppError::Configuration(_) => "Server configuration error".to_string(),
            AppError::Internal(_) => "An internal server error occurred".to_string(),
            AppError::Storage(manager_error) => match manager_error {
                orkee_projects::manager::ManagerError::NotFound(resource) => {
                    format!("{} not found", resource)
                }
                orkee_projects::manager::ManagerError::DuplicateName(name) => {
                    format!("A project with the name '{}' already exists", name)
                }
                orkee_projects::manager::ManagerError::DuplicatePath(path) => {
                    format!("A project already exists at path '{}'", path)
                }
                orkee_projects::manager::ManagerError::Validation(errors) => {
                    format!("Validation failed: {} error(s)", errors.len())
                }
                orkee_projects::manager::ManagerError::Storage(_) => {
                    "Data storage error".to_string()
                }
            },
            AppError::PathValidation(validation_error) => match validation_error {
                crate::api::path_validator::ValidationError::PathTraversal => {
                    "Path traversal detected and blocked".to_string()
                }
                crate::api::path_validator::ValidationError::RootAccess => {
                    "Root directory access denied".to_string()
                }
                crate::api::path_validator::ValidationError::BlockedPath(_) => {
                    "Access denied to blocked path".to_string()
                }
                crate::api::path_validator::ValidationError::SensitiveDirectory(_) => {
                    "Access to sensitive directory blocked".to_string()
                }
                crate::api::path_validator::ValidationError::NotInAllowedPaths => {
                    "Path is outside allowed directories".to_string()
                }
                crate::api::path_validator::ValidationError::SymlinkError => {
                    "Symlink validation failed".to_string()
                }
                crate::api::path_validator::ValidationError::InvalidPath => {
                    "Invalid path provided".to_string()
                }
                crate::api::path_validator::ValidationError::PathDoesNotExist => {
                    "Path does not exist".to_string()
                }
                crate::api::path_validator::ValidationError::ExpansionError => {
                    "Path expansion failed".to_string()
                }
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4().to_string();
        let (status_code, error_code) = self.to_status_and_code();
        let user_message = self.to_user_message();

        // Log internal errors with full context but don't expose details
        match &self {
            AppError::Internal(err) => {
                error!(
                    request_id = %request_id,
                    error = %err,
                    "Internal server error occurred"
                );
            }
            AppError::Configuration(msg) => {
                error!(
                    request_id = %request_id,
                    config_error = %msg,
                    "Configuration error"
                );
            }
            AppError::Storage(manager_err) => {
                if matches!(
                    manager_err,
                    orkee_projects::manager::ManagerError::Storage(_)
                ) {
                    error!(
                        request_id = %request_id,
                        storage_error = %manager_err,
                        "Storage system error"
                    );
                }
            }
            // Log security violations for monitoring
            AppError::PathAccessDenied(path) => {
                error!(
                    request_id = %request_id,
                    blocked_path = %path,
                    audit = true,
                    "Path access denied"
                );
            }
            AppError::PathTraversal => {
                error!(
                    request_id = %request_id,
                    audit = true,
                    "Path traversal attempt detected"
                );
            }
            AppError::RateLimitExceeded { retry_after } => {
                error!(
                    request_id = %request_id,
                    retry_after = %retry_after,
                    audit = true,
                    "Rate limit exceeded"
                );
            }
            _ => {
                // Log other errors at info level as they're expected business logic errors
                tracing::info!(
                    request_id = %request_id,
                    error_code = %error_code,
                    error = %self,
                    "API error response"
                );
            }
        }

        let mut error_detail = ErrorDetail {
            code: error_code.to_string(),
            message: user_message,
            retry_after: None,
            details: None,
        };

        // Add retry_after for rate limiting
        if let AppError::RateLimitExceeded { retry_after } = &self {
            error_detail.retry_after = Some(*retry_after);
        }

        let error_response = ErrorResponse {
            success: false,
            error: error_detail,
            request_id,
        };

        let mut response = Json(error_response).into_response();
        *response.status_mut() = status_code;

        // Add rate limiting headers
        if let AppError::RateLimitExceeded { retry_after } = &self {
            let headers = response.headers_mut();
            headers.insert("Retry-After", retry_after.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Limit", "30".parse().unwrap()); // Will be configurable
            headers.insert("X-RateLimit-Remaining", "0".parse().unwrap());
            headers.insert(
                "X-RateLimit-Reset",
                retry_after.to_string().parse().unwrap(),
            );
        }

        response
    }
}

/// Result type alias for API handlers
pub type ApiResult<T> = Result<T, AppError>;

/// Helper functions for common error scenarios
impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn not_found() -> Self {
        Self::NotFound
    }

    pub fn internal(err: impl Into<anyhow::Error>) -> Self {
        Self::Internal(err.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    pub fn rate_limited(retry_after: u64) -> Self {
        Self::RateLimitExceeded { retry_after }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_validation_error_status() {
        let error = AppError::validation("test error");
        let (status, code) = error.to_status_and_code();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_not_found_error() {
        let error = AppError::not_found();
        let (status, code) = error.to_status_and_code();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(code, "NOT_FOUND");
    }

    #[test]
    fn test_rate_limit_error() {
        let error = AppError::rate_limited(60);
        let (status, code) = error.to_status_and_code();
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(code, "RATE_LIMIT_EXCEEDED");
    }

    #[test]
    fn test_user_message_sanitization() {
        let internal_error = AppError::internal(anyhow::anyhow!(
            "Database connection failed with password xyz"
        ));
        let message = internal_error.to_user_message();
        assert_eq!(message, "An internal server error occurred");
        // Ensure no sensitive details are leaked
        assert!(!message.contains("password"));
        assert!(!message.contains("xyz"));
    }
}
