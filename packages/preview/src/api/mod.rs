pub mod handlers;

use axum::{
    routing::get,
    Router,
};
use handlers::AppState;

/// Create a basic preview API router (for standalone use)
/// In practice, the CLI server creates its own router with integrated handlers
pub fn create_preview_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
}