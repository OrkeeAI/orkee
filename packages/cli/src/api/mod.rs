use axum::{
    routing::{get, post},
    Router,
};
use tracing::error;

pub mod cloud;
pub mod config;
pub mod directories;
pub mod git;
pub mod health;
pub mod path_validator;
pub mod preview;
pub mod settings_handlers;
pub mod taskmaster;
pub mod telemetry;
pub mod telemetry_middleware;

pub async fn create_router() -> Router {
    let (router, _db_state) = create_router_with_options(None, None).await;
    router
}

pub async fn create_router_with_options(
    dashboard_path: Option<std::path::PathBuf>,
    database_path: Option<std::path::PathBuf>,
) -> (Router, orkee_projects::DbState) {
    use crate::config::Config;
    use path_validator::PathValidator;
    use preview::PreviewState;
    use std::sync::Arc;

    // Create path validator from config
    let config = Config::from_env().expect("Failed to load config for PathValidator");
    let path_validator = Arc::new(PathValidator::new(&config));

    // Create the project manager
    let project_manager = match orkee_projects::manager::ProjectsManager::new().await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            error!("Failed to initialize project manager: {}", e);
            // Create minimal in-memory DbState for error case
            let pool = sqlx::SqlitePool::connect(":memory:")
                .await
                .expect("Failed to create in-memory database for error fallback");
            sqlx::migrate!("../storage/migrations")
                .run(&pool)
                .await
                .expect("Failed to run migrations for error fallback");
            let minimal_db = orkee_projects::DbState::new(pool)
                .expect("Failed to create minimal DbState for error fallback");

            // Return a router without project manager functionality rather than panicking
            let router = Router::new()
                .route("/api/health", get(health::health_check))
                .route("/api/status", get(health::status_check))
                .route(
                    "/api/browse-directories",
                    post(directories::browse_directories),
                )
                .nest("/api/projects", orkee_api::create_projects_router());
            return (router, minimal_db);
        }
    };

    // Initialize database state for tasks/agents/users
    let db_path_for_error = database_path.clone();
    let db_state = match orkee_projects::DbState::init_with_path(database_path).await {
        Ok(state) => state,
        Err(e) => {
            error!("CRITICAL: Failed to initialize database state: {}", e);
            error!("This will cause API endpoints to return 500 errors");
            error!("Database path: {:?}", db_path_for_error);
            error!("Please check:");
            error!("  1. Database file permissions");
            error!("  2. Disk space availability");
            error!("  3. SQLite migrations status");
            error!("  4. Encryption key initialization (machine ID/hostname)");

            // Create minimal in-memory DbState for error fallback
            let pool = sqlx::SqlitePool::connect(":memory:")
                .await
                .expect("Failed to create in-memory database for error fallback");
            sqlx::migrate!("../storage/migrations")
                .run(&pool)
                .await
                .expect("Failed to run migrations for error fallback");
            let minimal_db = orkee_projects::DbState::new(pool)
                .expect("Failed to create minimal DbState for error fallback");

            // Return a router without task/agent/user functionality
            let router = Router::new()
                .route("/api/health", get(health::health_check))
                .route("/api/status", get(health::status_check))
                .route(
                    "/api/browse-directories",
                    post(directories::browse_directories),
                )
                .nest("/api/projects", orkee_api::create_projects_router());
            return (router, minimal_db);
        }
    };

    // Create a SqliteStorage from the existing db_state pool
    let storage_config = orkee_storage::StorageConfig {
        provider: orkee_storage::StorageProvider::Sqlite {
            path: db_path_for_error.unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".orkee").join("orkee.db"))
                    .expect("Failed to get home directory for default database path")
            }),
        },
        max_connections: 10,
        busy_timeout_seconds: 30,
        enable_wal: true,
        enable_fts: true, // Enable full-text search
    };
    let storage = match orkee_storage::sqlite::SqliteStorage::new(storage_config).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create storage for preview manager: {}", e);
            // Return minimal router without preview functionality
            let router = Router::new()
                .route("/api/health", get(health::health_check))
                .route("/api/status", get(health::status_check))
                .nest("/api/projects", orkee_api::create_projects_router());
            return (router, db_state);
        }
    };

    // Create the preview manager with crash recovery
    let preview_manager = match orkee_preview::init(&storage).await {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            error!("Failed to initialize preview manager: {}", e);
            error!("Preview functionality will be limited");
            // Continue without preview manager - UI will handle gracefully
            // For now, create a minimal preview manager (TODO: handle this better)
            let registry = match orkee_preview::registry::ServerRegistry::new(&storage).await {
                Ok(reg) => reg,
                Err(e2) => {
                    error!("Failed to create fallback registry: {}", e2);
                    // Return minimal router without preview functionality
                    let router = Router::new()
                        .route("/api/health", get(health::health_check))
                        .route("/api/status", get(health::status_check))
                        .nest("/api/projects", orkee_api::create_projects_router());
                    return (router, db_state);
                }
            };
            Arc::new(orkee_preview::manager::PreviewManager::new(registry))
        }
    };

    // Create preview state
    let preview_state = PreviewState {
        preview_manager: preview_manager.clone(),
        project_manager: project_manager.clone(),
        sse_tracker: preview::SseConnectionTracker::new(),
        db_state: db_state.clone(),
    };

    // Create preview router with its own state
    let preview_router = Router::new()
        .route("/health", get(preview::health_check))
        .route("/events", get(preview::server_events))
        .route("/servers", get(preview::list_active_servers))
        .route("/servers/discover", get(preview::discover_servers))
        .route("/servers/stop-all", post(preview::stop_all_servers))
        .route("/servers/{project_id}/start", post(preview::start_server))
        .route("/servers/{project_id}/stop", post(preview::stop_server))
        .route(
            "/servers/{project_id}/status",
            get(preview::get_server_status),
        )
        .route("/servers/{project_id}/logs", get(preview::get_server_logs))
        .route(
            "/servers/{project_id}/logs/clear",
            post(preview::clear_server_logs),
        )
        .route(
            "/servers/{project_id}/activity",
            post(preview::update_server_activity),
        )
        .route(
            "/servers/external/{server_id}/restart",
            post(preview::restart_external_server),
        )
        .route(
            "/servers/external/{server_id}/stop",
            post(preview::stop_external_server),
        )
        .with_state(preview_state);

    // Create git router
    let git_router = Router::new()
        .route("/{project_id}/commits", get(git::get_commit_history))
        .route(
            "/{project_id}/commits/{commit_id}",
            get(git::get_commit_details),
        )
        .route(
            "/{project_id}/diff/{commit_id}/{*file_path}",
            get(git::get_file_diff),
        )
        .layer(axum::Extension(project_manager.clone()));

    // Create taskmaster router
    let taskmaster_router = Router::new()
        .route("/tasks", post(taskmaster::get_tasks))
        .route("/tasks/save", post(taskmaster::save_tasks));

    // Create cloud router
    let cloud_router = Router::new()
        .route("/auth/init", post(cloud::init_oauth_flow))
        .route("/auth/callback", post(cloud::handle_oauth_callback))
        .route("/auth/status", get(cloud::get_auth_status))
        .route("/auth/logout", post(cloud::logout))
        .route("/sync/status", get(cloud::get_global_sync_status))
        .route(
            "/projects/{project_id}/status",
            get(cloud::get_project_sync_status),
        )
        .route("/projects", get(cloud::list_cloud_projects))
        .route("/projects/sync-all", post(cloud::sync_all_projects))
        .route("/projects/{project_id}/sync", post(cloud::sync_project))
        .route("/usage", get(cloud::get_usage_stats))
        .layer(axum::Extension(cloud::CloudState::new()));

    // Initialize telemetry manager
    // If it fails, log the error but continue without telemetry endpoints
    let telemetry_router = match crate::telemetry::init_telemetry_manager().await {
        Ok(manager) => {
            let telemetry_manager = Arc::new(manager);

            // Start background collector task
            // This spawns an async task that periodically sends buffered telemetry events to PostHog
            let pool = crate::telemetry::get_database_pool()
                .await
                .expect("Failed to get database pool for telemetry collector");
            let endpoint = telemetry_manager.get_endpoint();
            let collector = Arc::new(crate::telemetry::TelemetryCollector::new(
                telemetry_manager.clone(),
                pool,
                endpoint,
            ));

            // Spawn the background task
            tokio::spawn(async move {
                use tracing::info;
                info!("Starting telemetry background collector task");
                collector.start_background_task().await;
            });

            Router::new()
                .route("/status", get(telemetry::get_telemetry_status))
                .route("/settings", get(telemetry::get_telemetry_settings))
                .route(
                    "/settings",
                    axum::routing::put(telemetry::update_telemetry_settings),
                )
                .route(
                    "/onboarding/complete",
                    post(telemetry::complete_telemetry_onboarding),
                )
                .route(
                    "/data",
                    axum::routing::delete(telemetry::delete_telemetry_data),
                )
                .route("/track", post(telemetry::track_event))
                .layer(axum::Extension(telemetry_manager))
        }
        Err(e) => {
            error!("Failed to initialize telemetry manager: {}", e);
            // Return empty router - telemetry endpoints won't be available
            Router::new()
        }
    };

    // Create settings router
    let settings_router = Router::new()
        .route("/", get(settings_handlers::get_settings))
        .route(
            "/category/{category}",
            get(settings_handlers::get_settings_by_category),
        )
        .route(
            "/key/{key}",
            axum::routing::put(settings_handlers::update_setting),
        )
        .route(
            "/bulk",
            axum::routing::put(settings_handlers::bulk_update_settings),
        )
        .with_state(db_state.clone());

    let mut router = Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/status", get(health::status_check))
        .route("/api/csrf-token", get(health::get_csrf_token))
        .route(
            "/api/config",
            get(config::get_config).with_state(db_state.clone()),
        )
        .route(
            "/api/browse-directories",
            post(directories::browse_directories),
        )
        .nest(
            "/api",
            orkee_api::create_ai_proxy_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/projects",
            orkee_api::create_projects_router().layer(axum::middleware::from_fn(
                telemetry_middleware::track_api_calls,
            )),
        )
        .nest(
            "/api/projects/{project_id}/tasks",
            orkee_api::create_tasks_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/projects",
            orkee_api::create_prds_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/projects",
            orkee_api::create_epics_router().with_state(db_state.clone()),
        )
        .nest(
            "/api",
            orkee_api::create_github_sync_router().with_state(db_state.clone()),
        )
        .nest(
            "/api",
            orkee_api::create_ideate_router()
                .with_state(db_state.clone())
                .layer(axum::middleware::from_fn(
                    telemetry_middleware::track_api_calls,
                )),
        )
        .nest(
            "/api",
            orkee_api::create_templates_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/projects",
            orkee_api::create_graph_router().with_state(db_state.clone()),
        )
        .nest("/api/git", git_router)
        .nest(
            "/api/preview",
            preview_router.layer(axum::middleware::from_fn(
                telemetry_middleware::track_api_calls,
            )),
        )
        .nest("/api/cloud", cloud_router)
        .nest("/api/taskmaster", taskmaster_router)
        .nest("/api/telemetry", telemetry_router)
        .nest("/api/settings", settings_router)
        .nest(
            "/api/agents",
            orkee_api::create_agents_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/models",
            orkee_api::create_models_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/users",
            orkee_api::create_users_router().with_state(db_state.clone()),
        )
        .nest(
            "/api",
            orkee_api::create_security_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/auth",
            orkee_api::create_oauth_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/tags",
            orkee_api::create_tags_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/executions",
            orkee_api::create_executions_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/sandbox",
            orkee_api::create_sandbox_router().with_state(db_state.clone()),
        )
        .nest(
            "/api/sandboxes",
            orkee_api::create_sandboxes_router().with_state(db_state.clone()),
        )
        // Phase 6: AI handler routes removed - all AI calls now happen client-side
        // .nest(
        //     "/api",
        //     orkee_api::create_ai_router().with_state(db_state.clone()),
        // )
        .nest(
            "/api/ai-usage",
            orkee_api::create_ai_usage_router().with_state(db_state.clone()),
        )
        .layer(axum::Extension(path_validator));

    // If dashboard path is provided, serve static files
    if let Some(dashboard_path) = dashboard_path {
        use tower_http::services::{ServeDir, ServeFile};

        let dist_dir = dashboard_path.join("dist");
        let index_path = dist_dir.join("index.html");

        if dist_dir.exists() && index_path.exists() {
            // Serve static files with fallback to index.html for client-side routing
            let serve_dir = ServeDir::new(&dist_dir).not_found_service(ServeFile::new(&index_path));

            router = router.fallback_service(serve_dir);
        }
    }

    (router, db_state)
}
