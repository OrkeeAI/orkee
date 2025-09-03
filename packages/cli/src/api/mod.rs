use axum::{
    routing::get,
    Router,
};

pub mod health;

pub fn create_router() -> Router {
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/status", get(health::status_check))
}