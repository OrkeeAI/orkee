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
use url::Url;

use super::auth::CurrentUser;
use orkee_auth::oauth::OAuthProvider;
use orkee_projects::DbState;

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

// Known valid API path prefixes for each provider
const ANTHROPIC_ALLOWED_PATHS: &[&str] = &["/v1/messages", "/v1/complete", "/v1/models"];
const OPENAI_ALLOWED_PATHS: &[&str] = &[
    "/v1/chat/completions",
    "/v1/completions",
    "/v1/models",
    "/v1/embeddings",
    "/v1/audio/transcriptions",
    "/v1/audio/translations",
    "/v1/images/generations",
];
const GOOGLE_ALLOWED_PATHS: &[&str] = &["/v1beta/models", "/v1/models"];
const XAI_ALLOWED_PATHS: &[&str] = &["/v1/chat/completions", "/v1/completions", "/v1/models"];

/// Validates that the API path is safe and matches known provider endpoints
fn validate_api_path(path: &str, provider: &str) -> Result<(), String> {
    // Check for path traversal attempts
    if path.contains("..") || path.contains("//") || path.contains('@') {
        return Err("Path contains invalid characters".to_string());
    }

    // Check for URL-encoded traversal attempts
    if path.contains("%2e%2e") || path.contains("%2f%2f") || path.contains("%40") {
        return Err("Path contains URL-encoded invalid characters".to_string());
    }

    // Get allowed paths for this provider
    let allowed_paths = match provider {
        "anthropic" => ANTHROPIC_ALLOWED_PATHS,
        "openai" => OPENAI_ALLOWED_PATHS,
        "google" => GOOGLE_ALLOWED_PATHS,
        "xai" => XAI_ALLOWED_PATHS,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };

    // Check if the path matches any allowed prefix
    let is_allowed = allowed_paths
        .iter()
        .any(|allowed| path.starts_with(allowed));

    if !is_allowed {
        return Err(format!(
            "Path '{}' not allowed for provider '{}'. Allowed paths: {}",
            path,
            provider,
            allowed_paths.join(", ")
        ));
    }

    Ok(())
}

/// Validates that the constructed URL matches the expected base URL
fn validate_target_url(url_str: &str, expected_base: &str) -> Result<(), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    let base = Url::parse(expected_base).map_err(|e| format!("Invalid base URL: {}", e))?;

    // Verify scheme matches (https)
    if url.scheme() != base.scheme() {
        return Err(format!(
            "URL scheme mismatch: expected {}, got {}",
            base.scheme(),
            url.scheme()
        ));
    }

    // Verify host matches exactly
    if url.host_str() != base.host_str() {
        return Err(format!(
            "URL host mismatch: expected {:?}, got {:?}",
            base.host_str(),
            url.host_str()
        ));
    }

    // Verify port matches (or both are default)
    if url.port() != base.port() {
        return Err(format!(
            "URL port mismatch: expected {:?}, got {:?}",
            base.port(),
            url.port()
        ));
    }

    Ok(())
}

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

/// Try to get OAuth token for a provider
/// Returns Some(token) if OAuth token exists and is valid, None otherwise
async fn try_get_oauth_token(db: &DbState, user_id: &str, provider: &str) -> Option<String> {
    // Parse provider string into OAuthProvider enum
    let oauth_provider = match provider {
        "anthropic" => OAuthProvider::Claude,
        "openai" => OAuthProvider::OpenAI,
        "google" => OAuthProvider::Google,
        "xai" => OAuthProvider::XAI,
        _ => {
            warn!("Unknown provider for OAuth: {}", provider);
            return None;
        }
    };

    // Create OAuth manager
    let manager = match orkee_auth::OAuthManager::new(db.pool.clone()) {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to initialize OAuth manager: {}", e);
            return None;
        }
    };

    // Try to get valid OAuth token
    match manager.get_token(user_id, oauth_provider).await {
        Ok(Some(token)) => {
            info!(
                "Found valid OAuth token for {} (expires: {})",
                provider, token.expires_at
            );
            Some(token.access_token)
        }
        Ok(None) => {
            info!("No valid OAuth token found for {}", provider);
            None
        }
        Err(e) => {
            warn!("Failed to get OAuth token for {}: {}", provider, e);
            None
        }
    }
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

    // Try OAuth token first, then fall back to API key
    let api_key = match try_get_oauth_token(db, user_id, provider).await {
        Some(token) => {
            info!("Using OAuth token for {} provider", provider);
            token
        }
        None => {
            info!(
                "No OAuth token found, trying API key for {} provider",
                provider
            );
            // Get API key from database with env fallback
            match db.user_storage.get_api_key(user_id, provider).await {
                Ok(Some(key)) => key,
                Ok(None) => {
                    error!("{} API key or OAuth token not configured", provider);
                    return build_error_response(
                        StatusCode::UNAUTHORIZED,
                        format!(
                            "{} API key or OAuth token not configured. Please add it in Settings or authenticate with OAuth.",
                            provider
                        ),
                    );
                }
                Err(e) => {
                    error!("Failed to get {} API key: {}", provider, e);
                    return build_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to retrieve API key. Please check server logs for details."
                            .to_string(),
                    );
                }
            }
        }
    };

    // Build the target URL by preserving the path from the original request
    let path = req.uri().path();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    // Remove the /ai/{provider} prefix from the path
    // Note: /api prefix is already handled by Axum router
    let provider_prefix = format!("/ai/{}", provider);
    let target_path = path.strip_prefix(&provider_prefix).unwrap_or(path);

    // Validate the API path against provider whitelist
    if let Err(e) = validate_api_path(target_path, provider) {
        error!(
            "Invalid API path for {} proxy: {} (path: {})",
            provider, e, target_path
        );
        return build_error_response(StatusCode::BAD_REQUEST, format!("Invalid API path: {}", e));
    }

    let target_url = format!("{}{}{}", base_url, target_path, query);

    // Validate that the constructed URL matches the expected base
    if let Err(e) = validate_target_url(&target_url, base_url) {
        error!(
            "Target URL validation failed for {} proxy: {} (url: {})",
            provider, e, target_url
        );
        return build_error_response(
            StatusCode::BAD_REQUEST,
            format!("Invalid target URL: {}", e),
        );
    }

    info!("Forwarding to: {}", target_url);

    // Create HTTP client
    // With the gzip, brotli, and deflate Cargo features enabled,
    // reqwest automatically decompresses response bodies
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
    } else {
        warn!(
            "Content-Type header missing for {} proxy request. This may cause issues with the API.",
            provider
        );
    }

    // Check for Accept header (important for content negotiation)
    if !headers.contains_key("accept") {
        warn!(
            "Accept header missing for {} proxy request. API may return unexpected content type.",
            provider
        );
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

    // Debug: Log the request body to see what model is being sent
    if let Ok(body_str) = std::str::from_utf8(&body_bytes) {
        info!("Request body: {}", body_str);
    }

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
    info!(
        "Building proxy request - body size: {} bytes",
        body_bytes.len()
    );
    let mut proxy_req = client
        .request(method, &target_url)
        .body(body_bytes.to_vec());

    // Copy relevant headers (exclude host, connection, browser-specific headers, etc.)
    info!("Copying request headers (excluding auth and browser-specific headers)");
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !matches!(
            key_str.as_str(),
            "host"
                | "connection"
                | "content-length"
                | "x-api-key"
                | "authorization"
                | "origin"
                | "referer"
                | "user-agent"
                | "sec-fetch-site"
                | "sec-fetch-mode"
                | "sec-fetch-dest"
                | "sec-ch-ua"
                | "sec-ch-ua-mobile"
                | "sec-ch-ua-platform"
        ) {
            if let Ok(val_str) = value.to_str() {
                info!("  Copying header: {} = {}", key_str, val_str);
            }
            proxy_req = proxy_req.header(key, value);
        } else {
            info!("  Excluding header: {}", key_str);
        }
    }

    // Add custom User-Agent header for proper API behavior
    proxy_req = proxy_req.header("User-Agent", "Orkee/1.0");

    // Add the API key header and provider-specific headers
    info!("Adding provider-specific headers for: {}", provider);
    proxy_req = match provider {
        "anthropic" => {
            info!(
                "  Adding x-api-key: {}...",
                &api_key[..8.min(api_key.len())]
            );
            info!("  Adding anthropic-version: 2023-06-01");
            proxy_req
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
        }
        "openai" => proxy_req.header("Authorization", format!("Bearer {}", api_key)),
        "google" => proxy_req.header("x-goog-api-key", api_key),
        "xai" => proxy_req.header("Authorization", format!("Bearer {}", api_key)),
        _ => proxy_req,
    };

    // Send the request
    info!("Sending request to {}", target_url);
    let response = match proxy_req.send().await {
        Ok(resp) => {
            info!("Request sent successfully");
            resp
        }
        Err(e) => {
            error!("Failed to proxy request to {}: {}", provider, e);
            error!("Error details: {:?}", e);
            return build_error_response(
                StatusCode::BAD_GATEWAY,
                format!(
                    "Failed to connect to {} API. Please check server logs for details.",
                    provider
                ),
            );
        }
    };

    // Build response
    let status = response.status();
    let headers = response.headers().clone();
    info!("Received response with status: {}", status);
    info!("Response headers:");
    for (key, value) in headers.iter() {
        if let Ok(val_str) = value.to_str() {
            info!("  {} = {}", key, val_str);
        }
    }

    // Read response with size limit
    info!("Reading response body...");
    let body_bytes = match response.bytes().await {
        Ok(bytes) => {
            info!("Successfully read {} bytes from response", bytes.len());
            bytes
        }
        Err(e) => {
            error!("Failed to read response body: {}", e);
            error!("Error details: {:?}", e);
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
    info!("Building final response with status: {}", status);
    let mut builder = Response::builder().status(status);

    // Copy response headers (excluding hop-by-hop headers)
    // We've already read and decoded the full body, so we shouldn't copy
    // transfer-encoding, content-encoding, or other hop-by-hop headers
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !matches!(
            key_str.as_str(),
            "transfer-encoding"
                | "content-encoding"
                | "connection"
                | "keep-alive"
                | "proxy-authenticate"
                | "proxy-authorization"
                | "te"
                | "trailer"
                | "upgrade"
        ) {
            builder = builder.header(key, value);
        } else {
            info!("  Excluding hop-by-hop header: {}", key_str);
        }
    }

    info!(
        "Proxy request completed successfully - returning {} bytes",
        body_bytes.len()
    );
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
        let response =
            build_error_response(StatusCode::BAD_REQUEST, "Test error message".to_string());

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_validate_api_path_anthropic() {
        // Valid paths
        assert!(validate_api_path("/v1/messages", "anthropic").is_ok());
        assert!(validate_api_path("/v1/complete", "anthropic").is_ok());
        assert!(validate_api_path("/v1/models", "anthropic").is_ok());

        // Invalid paths
        assert!(validate_api_path("/v1/admin", "anthropic").is_err());
        assert!(validate_api_path("/../../etc/passwd", "anthropic").is_err());
        assert!(validate_api_path("/v1/../admin", "anthropic").is_err());
    }

    #[test]
    fn test_validate_api_path_traversal_attacks() {
        // Path traversal attempts should be rejected
        assert!(validate_api_path("/../etc/passwd", "openai").is_err());
        assert!(validate_api_path("/v1/../../admin", "anthropic").is_err());
        assert!(validate_api_path("//etc/passwd", "google").is_err());
        assert!(validate_api_path("/v1/@attacker.com/endpoint", "xai").is_err());

        // URL-encoded traversal attempts
        assert!(validate_api_path("/v1/%2e%2e/admin", "anthropic").is_err());
        assert!(validate_api_path("/%2f%2fetc/passwd", "openai").is_err());
        assert!(validate_api_path("/v1/%40attacker.com", "google").is_err());
    }

    #[test]
    fn test_validate_api_path_openai() {
        // Valid paths
        assert!(validate_api_path("/v1/chat/completions", "openai").is_ok());
        assert!(validate_api_path("/v1/completions", "openai").is_ok());
        assert!(validate_api_path("/v1/models", "openai").is_ok());
        assert!(validate_api_path("/v1/embeddings", "openai").is_ok());

        // Invalid paths
        assert!(validate_api_path("/v2/chat/completions", "openai").is_err());
        assert!(validate_api_path("/admin", "openai").is_err());
    }

    #[test]
    fn test_validate_target_url() {
        // Valid URLs matching base
        assert!(validate_target_url(
            "https://api.anthropic.com/v1/messages",
            "https://api.anthropic.com"
        )
        .is_ok());

        assert!(validate_target_url(
            "https://api.openai.com/v1/chat/completions",
            "https://api.openai.com"
        )
        .is_ok());

        // Invalid - host mismatch
        assert!(validate_target_url(
            "https://attacker.com/v1/messages",
            "https://api.anthropic.com"
        )
        .is_err());

        // Invalid - scheme mismatch
        assert!(validate_target_url(
            "http://api.anthropic.com/v1/messages",
            "https://api.anthropic.com"
        )
        .is_err());

        // Invalid - port mismatch
        assert!(validate_target_url(
            "https://api.anthropic.com:8080/v1/messages",
            "https://api.anthropic.com"
        )
        .is_err());
    }

    #[test]
    fn test_validate_api_path_unknown_provider() {
        assert!(validate_api_path("/v1/messages", "unknown-provider").is_err());
    }
}
