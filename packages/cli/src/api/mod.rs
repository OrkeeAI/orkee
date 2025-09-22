use axum::{
    routing::{get, post},
    Router,
};

pub mod cloud;
pub mod config;
pub mod directories;
pub mod git;
pub mod health;
pub mod path_validator;
pub mod preview;

pub async fn create_router() -> Router {
    use crate::config::Config;
    use path_validator::PathValidator;
    use preview::PreviewState;
    use std::sync::Arc;

    // Create path validator from config
    let config = Config::from_env().expect("Failed to load config for PathValidator");
    let path_validator = Arc::new(PathValidator::new(&config));

    // Create the preview manager with recovery
    let preview_manager = match orkee_preview::init().await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            eprintln!("Failed to initialize preview manager: {}", e);
            // Return a router without preview functionality rather than panicking
            return Router::new()
                .route("/api/health", get(health::health_check))
                .route("/api/status", get(health::status_check))
                .route(
                    "/api/browse-directories",
                    post(directories::browse_directories),
                )
                .nest("/api/projects", orkee_projects::create_projects_router());
        }
    };

    // Create the project manager
    let project_manager = match orkee_projects::manager::ProjectsManager::new().await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            eprintln!("Failed to initialize project manager: {}", e);
            // Return a router without preview functionality rather than panicking
            return Router::new()
                .route("/api/health", get(health::health_check))
                .route("/api/status", get(health::status_check))
                .route(
                    "/api/browse-directories",
                    post(directories::browse_directories),
                )
                .nest("/api/projects", orkee_projects::create_projects_router());
        }
    };

    // Create preview state
    let preview_state = PreviewState {
        preview_manager: preview_manager.clone(),
        project_manager: project_manager.clone(),
    };

    // Create preview router with its own state
    let preview_router = Router::new()
        .route("/health", get(preview::health_check))
        .route("/servers", get(preview::list_active_servers))
        .route("/servers/:project_id/start", post(preview::start_server))
        .route("/servers/:project_id/stop", post(preview::stop_server))
        .route(
            "/servers/:project_id/status",
            get(preview::get_server_status),
        )
        .route("/servers/:project_id/logs", get(preview::get_server_logs))
        .route(
            "/servers/:project_id/logs/clear",
            post(preview::clear_server_logs),
        )
        .route(
            "/servers/:project_id/activity",
            post(preview::update_server_activity),
        )
        .with_state(preview_state);

    // Create git router
    let git_router = Router::new()
        .route("/:project_id/commits", get(git::get_commit_history))
        .route(
            "/:project_id/commits/:commit_id",
            get(git::get_commit_details),
        )
        .route(
            "/:project_id/diff/:commit_id/*file_path",
            get(git::get_file_diff),
        )
        .layer(axum::Extension(project_manager.clone()));

    // Create cloud router
    let cloud_router = Router::new()
        .route("/auth/init", post(cloud::init_oauth_flow))
        .route("/auth/callback", post(cloud::handle_oauth_callback))
        .route("/auth/status", get(cloud::get_auth_status))
        .route("/auth/logout", post(cloud::logout))
        .route("/sync/status", get(cloud::get_global_sync_status))
        .route(
            "/projects/:project_id/status",
            get(cloud::get_project_sync_status),
        )
        .route("/projects", get(cloud::list_cloud_projects))
        .route("/projects/sync-all", post(cloud::sync_all_projects))
        .route("/projects/:project_id/sync", post(cloud::sync_project))
        .route("/usage", get(cloud::get_usage_stats))
        .layer(axum::Extension(cloud::CloudState::new()));

    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/status", get(health::status_check))
        .route("/api/config", get(config::get_config))
        .route(
            "/api/browse-directories",
            post(directories::browse_directories),
        )
        .nest("/api/projects", orkee_projects::create_projects_router())
        .nest("/api/git", git_router)
        .nest("/api/preview", preview_router)
        .nest("/api/cloud", cloud_router)
        .layer(axum::Extension(path_validator))
}
