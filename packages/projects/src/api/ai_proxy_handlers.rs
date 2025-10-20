// ABOUTME: AI proxy handlers for secure API key management
// ABOUTME: Proxies requests to AI providers using database-stored credentials

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use reqwest::Client;
use tracing::{error, info};

use crate::db::DbState;

/// Proxy requests to Anthropic API with API key from database
pub async fn proxy_anthropic(
    State(db): State<DbState>,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(&db, "anthropic", "https://api.anthropic.com", req).await
}

/// Proxy requests to OpenAI API with API key from database
pub async fn proxy_openai(
    State(db): State<DbState>,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(&db, "openai", "https://api.openai.com", req).await
}

/// Proxy requests to Google AI API with API key from database
pub async fn proxy_google(
    State(db): State<DbState>,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(&db, "google", "https://generativelanguage.googleapis.com", req).await
}

/// Proxy requests to xAI API with API key from database
pub async fn proxy_xai(
    State(db): State<DbState>,
    req: Request<Body>,
) -> impl IntoResponse {
    proxy_ai_request(&db, "xai", "https://api.x.ai", req).await
}

/// Generic AI request proxy handler
async fn proxy_ai_request(
    db: &DbState,
    provider: &str,
    base_url: &str,
    req: Request<Body>,
) -> Response<Body> {
    info!("Proxying {} API request", provider);

    // Get API key from database with env fallback
    let api_key = match db.user_storage.get_api_key("default-user", provider).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            error!("{} API key not configured", provider);
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(format!("{} API key not configured. Please add it in Settings.", provider)))
                .unwrap();
        }
        Err(e) => {
            error!("Failed to get {} API key: {}", provider, e);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to retrieve API key: {}", e)))
                .unwrap();
        }
    };

    // Build the target URL by preserving the path from the original request
    let path = req.uri().path();
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();

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

    // Read the request body
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Failed to read request body"))
                .unwrap();
        }
    };

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
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Failed to connect to {} API: {}", provider, e)))
                .unwrap();
        }
    };

    // Build response
    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read response body: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from("Failed to read AI provider response"))
                .unwrap();
        }
    };

    // Build the final response
    let mut builder = Response::builder().status(status);

    // Copy response headers
    for (key, value) in headers.iter() {
        builder = builder.header(key, value);
    }

    builder.body(Body::from(body_bytes)).unwrap()
}
