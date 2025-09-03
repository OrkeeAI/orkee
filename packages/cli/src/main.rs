use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;

mod api;
mod config;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();
    
    let config = Config::from_env();
    
    println!("🚀 Starting Orkee CLI server...");
    println!("📡 Server will run on http://localhost:{}", config.port);
    println!("🔗 CORS origin: {}", config.cors_origin);

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<axum::http::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // Create the router with CORS
    let app = api::create_router()
        .layer(cors);

    // Create socket address
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    
    println!("✅ Server listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}