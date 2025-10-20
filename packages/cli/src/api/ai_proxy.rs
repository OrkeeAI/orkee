// ABOUTME: Proxy endpoint for Anthropic API to solve CORS and secure API keys
// ABOUTME: Forwards requests from frontend to Anthropic with server-side API key

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use reqwest::Client;
use std::env;
use tracing::{error, info};

/// Proxy requests to Anthropic API
/// This solves CORS issues and keeps API keys secure on the server
pub async fn anthropic_proxy(request: Request) -> impl IntoResponse {
    // Get API key from environment
    let api_key = match env::var("ANTHROPIC_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            error!("ANTHROPIC_API_KEY not configured");
            return Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from(
                    r#"{"error": "Anthropic API key not configured on server"}"#,
                ))
                .unwrap();
        }
    };

    // Read the request body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!(
                    r#"{{"error": "Failed to read request body: {}"}}"#,
                    e
                )))
                .unwrap();
        }
    };

    info!(
        "Proxying request to Anthropic API (body size: {} bytes)",
        body_bytes.len()
    );

    // Create HTTP client
    let client = Client::new();

    // Forward request to Anthropic
    let anthropic_response = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(body_bytes)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to call Anthropic API: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!(
                    r#"{{"error": "Failed to call Anthropic API: {}"}}"#,
                    e
                )))
                .unwrap();
        }
    };

    let status = anthropic_response.status();

    // Get response body
    let response_bytes = match anthropic_response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read Anthropic response: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!(
                    r#"{{"error": "Failed to read Anthropic response: {}"}}"#,
                    e
                )))
                .unwrap();
        }
    };

    info!(
        "Anthropic API response: status={}, size={} bytes",
        status,
        response_bytes.len()
    );

    // Return Anthropic's response with CORS headers
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "POST, OPTIONS")
        .header("access-control-allow-headers", "content-type, anthropic-version, x-api-key")
        .body(Body::from(response_bytes))
        .unwrap()
}

/// Handle OPTIONS preflight requests for CORS
pub async fn anthropic_proxy_options() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "POST, OPTIONS")
        .header("access-control-allow-headers", "content-type, anthropic-version, x-api-key")
        .header("access-control-max-age", "86400")
        .body(Body::empty())
        .unwrap()
}
