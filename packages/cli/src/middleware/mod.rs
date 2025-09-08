//! Middleware modules for security, rate limiting, and error handling

pub mod security_headers;
pub mod rate_limit;
pub mod https_redirect;

pub use rate_limit::{RateLimitLayer, RateLimitConfig};
pub use security_headers::SecurityHeadersLayer;

use tower_http::catch_panic::CatchPanicLayer;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

/// Create a panic handler that returns consistent error responses
pub fn create_panic_handler() -> CatchPanicLayer<fn(Box<dyn std::any::Any + Send + 'static>) -> Response> {
    CatchPanicLayer::custom(handle_panic)
}

/// Handle panic with proper logging and sanitized response
fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    // Extract panic message safely
    let panic_message = if let Some(s) = err.downcast_ref::<String>() {
        s.as_str()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s
    } else {
        "unknown panic occurred"
    };
    
    // Log the panic with full context for debugging
    error!(
        request_id = %request_id,
        panic_message = %panic_message,
        audit = true,
        "Server panic occurred"
    );
    
    // Return sanitized error response without exposing panic details
    let error_response = json!({
        "success": false,
        "error": {
            "code": "INTERNAL_ERROR",
            "message": "An internal server error occurred",
            "request_id": request_id
        }
    });
    
    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode};
    use serde_json::Value;
    
    #[tokio::test]
    async fn test_panic_handler_response_format() {
        let panic_err = Box::new("test panic".to_string());
        let response = handle_panic(panic_err);
        
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        
        // Verify the response body has the expected format
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap();
        
        assert_eq!(body["success"], false);
        assert_eq!(body["error"]["code"], "INTERNAL_ERROR");
        assert_eq!(body["error"]["message"], "An internal server error occurred");
        assert!(body["error"]["request_id"].is_string());
        
        // Ensure panic details are not exposed
        let response_str = serde_json::to_string(&body).unwrap();
        assert!(!response_str.contains("test panic"));
    }
}