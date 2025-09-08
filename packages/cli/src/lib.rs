use axum::http::{Method, HeaderValue, header};
use std::{net::SocketAddr, time::Duration};
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

pub mod api;
pub mod config;

#[cfg(test)]
mod tests;

use config::Config;

fn validate_cors_origin(origin: &str) -> Result<AllowOrigin, Box<dyn std::error::Error>> {
    // Local development origins only
    const ALLOWED_ORIGINS: &[&str] = &[
        "http://localhost:5173",
        "http://localhost:5174", 
        "http://localhost:5175",
        "http://localhost:3000",
        "http://127.0.0.1:5173",
        "http://127.0.0.1:3000",
        "http://[::1]:5173",  // IPv6 localhost
    ];
    
    // Check if origin is in allowed list
    if !ALLOWED_ORIGINS.contains(&origin.as_ref()) {
        return Err(format!(
            "Invalid CORS origin: {}. Only localhost origins are allowed",
            origin
        ).into());
    }
    
    // Parse and validate origin format
    let header_value = HeaderValue::from_str(origin)?;
    Ok(AllowOrigin::exact(header_value))
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;

    // Create CORS layer with specific headers only
    let allowed_headers = AllowHeaders::list([
        header::CONTENT_TYPE,
        header::ACCEPT,
        header::AUTHORIZATION,  // For future API key
        header::HeaderName::from_static("x-api-key"),
    ]);

    // Validate and restrict origin
    let cors_origin = validate_cors_origin(&config.cors_origin)?;

    let cors = CorsLayer::new()
        .allow_origin(cors_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(allowed_headers)
        .allow_credentials(false)  // Explicitly disable credentials for local use
        .max_age(Duration::from_secs(3600));

    // Create the router with CORS
    let app = api::create_router().await.layer(cors);

    // Create socket address
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    println!("✅ Server listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
