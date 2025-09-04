use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub mod handlers;

/// Creates the projects API router
pub fn create_projects_router() -> Router {
    Router::new()
        .route("/", get(handlers::list_projects))
        .route("/", post(handlers::create_project))
        .route("/:id", get(handlers::get_project))
        .route("/:id", put(handlers::update_project))
        .route("/:id", delete(handlers::delete_project))
        .route("/by-name/:name", get(handlers::get_project_by_name))
        .route("/by-path", post(handlers::get_project_by_path))
}