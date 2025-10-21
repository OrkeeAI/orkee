// ABOUTME: AI proxy handlers for secure API key management
// ABOUTME: Proxies requests to AI providers using database-stored credentials

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use reqwest::Client;
use tracing::{error, info, warn};

use super::auth::CurrentUser;
use crate::db::DbState;

// Request body size limit: 10MB
const MAX_REQUEST_SIZE: usize = 10_485_760;
// Response body size limit: 50MB
const MAX_RESPONSE_SIZE: usize = 52_428_800;
// Allowed Content-Type headers for AI API requests
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "application/json",
    "application/x-ndjson",
    "text/event-stream",
];

/// Build an error response with fallback in case Response builder fails
fn build_error_response(status: StatusCode, message: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message.clone()))
        .unwrap_or_else(|e| {
            warn!(
                "Failed to build error response: {}. Original message: {}",
                e, message
            );
            Response::new(Body::from("Internal server error"))
        })
}

/// Proxy requests to Anthropic API with API key from database
pub async fn proxy_anthropic(
    State(db): State<DbState>,
    current_user: CurrentUser,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(
        &db,
        &current_user.id,
        "anthropic",
        "https://api.anthropic.com",
        req,
    )
    .await
}

/// Proxy requests to OpenAI API with API key from database
pub async fn proxy_openai(
    State(db): State<DbState>,
    current_user: CurrentUser,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(
        &db,
        &current_user.id,
        "openai",
        "https://api.openai.com",
        req,
    )
    .await
}

/// Proxy requests to Google AI API with API key from database
pub async fn proxy_google(
    State(db): State<DbState>,
    current_user: CurrentUser,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(
        &db,
        &current_user.id,
        "google",
        "https://generativelanguage.googleapis.com",
        req,
    )
    .await
}

/// Proxy requests to xAI API with API key from database
pub async fn proxy_xai(
    State(db): State<DbState>,
    current_user: CurrentUser,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(&db, &current_user.id, "xai", "https://api.x.ai", req).await
}

/// Generic AI request proxy handler
async fn proxy_ai_request(
    db: &DbState,
    user_id: &str,
    provider: &str,
    base_url: &str,
    req: Request<Body>,
) -> Response<Body> {
    info!("Proxying {} API request", provider);

    // Get API key from database with env fallback
    let api_key = match db.user_storage.get_api_key(user_id, provider).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            error!("{} API key not configured", provider);
            return build_error_response(
                StatusCode::UNAUTHORIZED,
                format!(
                    "{} API key not configured. Please add it in Settings.",
                    provider
                ),
            );
        }
        Err(e) => {
            error!("Failed to get {} API key: {}", provider, e);
            return build_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve API key: {}", e),
            );
        }
    };

    // Build the target URL by preserving the path from the original request
    let path = req.uri().path();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    // Remove the /api/ai/{provider} prefix from the path
    let provider_prefix = format!("/api/ai/{}", provider);
    let target_path = path.strip_prefix(&provider_prefix).unwrap_or(path);
    let target_url = format!("{}{}{}", base_url, target_path, query);

    info!("Forwarding to: {}", target_url);

    // Create HTTP client
    let client = Client::new();

    // Build the proxied request
    let method = req.method().clone();
    let headers = req.headers().clone();

    // Validate Content-Type header
    if let Some(content_type) = headers.get("content-type") {
        let content_type_str = content_type.to_str().unwrap_or("");
        let is_allowed = ALLOWED_CONTENT_TYPES
            .iter()
            .any(|allowed| content_type_str.starts_with(allowed));

        if !is_allowed {
            error!(
                "Invalid Content-Type header for {} proxy: {}",
                provider, content_type_str
            );
            return build_error_response(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                format!(
                    "Content-Type '{}' not allowed. Supported types: {}",
                    content_type_str,
                    ALLOWED_CONTENT_TYPES.join(", ")
                ),
            );
        }
    }

    // Read the request body with size limit
    let body_bytes = match axum::body::to_bytes(req.into_body(), MAX_REQUEST_SIZE).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return build_error_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("Request body too large (max {} bytes)", MAX_REQUEST_SIZE),
            );
        }
    };

    // Validate request body size
    if body_bytes.len() > MAX_REQUEST_SIZE {
        error!(
            "Request body size ({} bytes) exceeds maximum allowed ({} bytes)",
            body_bytes.len(),
            MAX_REQUEST_SIZE
        );
        return build_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("Request body too large (max {} bytes)", MAX_REQUEST_SIZE),
        );
    }

    // Build the request to the AI provider
    let mut proxy_req = client
        .request(method, &target_url)
        .body(body_bytes.to_vec());

    // Copy relevant headers (exclude host, connection, etc.)
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !matches!(key_str.as_str(), "host" | "connection" | "content-length") {
            proxy_req = proxy_req.header(key, value);
        }
    }

    // Add the API key header based on provider
    proxy_req = match provider {
        "anthropic" => proxy_req.header("x-api-key", api_key),
        "openai" => proxy_req.header("Authorization", format!("Bearer {}", api_key)),
        "google" => proxy_req.header("x-goog-api-key", api_key),
        "xai" => proxy_req.header("Authorization", format!("Bearer {}", api_key)),
        _ => proxy_req,
    };

    // Send the request
    let response = match proxy_req.send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to proxy request to {}: {}", provider, e);
            return build_error_response(
                StatusCode::BAD_GATEWAY,
                format!("Failed to connect to {} API: {}", provider, e),
            );
        }
    };

    // Build response
    let status = response.status();
    let headers = response.headers().clone();

    // Read response with size limit
    let body_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read response body: {}", e);
            return build_error_response(
                StatusCode::BAD_GATEWAY,
                "Failed to read AI provider response".to_string(),
            );
        }
    };

    // Validate response body size
    if body_bytes.len() > MAX_RESPONSE_SIZE {
        error!(
            "Response body size ({} bytes) exceeds maximum allowed ({} bytes)",
            body_bytes.len(),
            MAX_RESPONSE_SIZE
        );
        return build_error_response(
            StatusCode::INSUFFICIENT_STORAGE,
            format!("Response body too large (max {} bytes)", MAX_RESPONSE_SIZE),
        );
    }

    // Build the final response
    let mut builder = Response::builder().status(status);

    // Copy response headers
    for (key, value) in headers.iter() {
        builder = builder.header(key, value);
    }

    builder.body(Body::from(body_bytes)).unwrap_or_else(|e| {
        error!("Failed to build final response: {}", e);
        build_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to build response from AI provider".to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_limits_constants() {
        // Verify size limits match PR review recommendations
        assert_eq!(MAX_REQUEST_SIZE, 10_485_760); // 10MB
        assert_eq!(MAX_RESPONSE_SIZE, 52_428_800); // 50MB
    }

    #[test]
    fn test_allowed_content_types() {
        // Verify all expected content types are allowed
        assert!(ALLOWED_CONTENT_TYPES.contains(&"application/json"));
        assert!(ALLOWED_CONTENT_TYPES.contains(&"application/x-ndjson"));
        assert!(ALLOWED_CONTENT_TYPES.contains(&"text/event-stream"));

        // Verify disallowed content types are not in the list
        assert!(!ALLOWED_CONTENT_TYPES.contains(&"text/html"));
        assert!(!ALLOWED_CONTENT_TYPES.contains(&"application/xml"));
    }

    #[test]
    fn test_content_type_validation_logic() {
        // Test that content type matching works with charset parameters
        let valid_json_with_charset = "application/json; charset=utf-8";
        let is_allowed = ALLOWED_CONTENT_TYPES
            .iter()
            .any(|allowed| valid_json_with_charset.starts_with(allowed));
        assert!(is_allowed);

        // Test that invalid content types are rejected
        let invalid_html = "text/html";
        let is_allowed = ALLOWED_CONTENT_TYPES
            .iter()
            .any(|allowed| invalid_html.starts_with(allowed));
        assert!(!is_allowed);
    }

    #[test]
    fn test_build_error_response() {
        let response = build_error_response(
            StatusCode::BAD_REQUEST,
            "Test error message".to_string(),
        );

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
