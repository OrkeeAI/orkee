use axum::{
    routing::{get, post},
    Router,
};

pub mod health;
pub mod directories;

pub fn create_router() -> Router {
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/status", get(health::status_check))
        .route("/api/browse-directories", post(directories::browse_directories))
        .nest("/api/projects", orkee_projects::create_projects_router())
}