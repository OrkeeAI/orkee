use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::db::DbState;

pub mod agents_handlers;
pub mod executions_handlers;
pub mod handlers;
pub mod response;
pub mod tags_handlers;
pub mod tasks_handlers;
pub mod users_handlers;

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
        .route("/check-taskmaster", post(handlers::check_taskmaster))
        .route("/open-in-editor", post(handlers::open_in_editor))
        .route("/open-in-editor", get(handlers::test_editor_config))
        // Task management endpoints
        .route("/tasks", post(handlers::get_tasks))
        .route("/tasks/create", post(handlers::create_task))
        .route("/tasks/update", post(handlers::update_task))
        .route("/tasks/delete", post(handlers::delete_task))
        // Database export/import endpoints
        .route("/export", get(handlers::export_database))
        .route("/import", post(handlers::import_database))
}

/// Creates the tasks API router (nested under /api/projects/:project_id/tasks)
pub fn create_tasks_router() -> Router<DbState> {
    Router::new()
        .route("/", get(tasks_handlers::list_tasks))
        .route("/", post(tasks_handlers::create_task))
        .route("/:task_id", get(tasks_handlers::get_task))
        .route("/:task_id", put(tasks_handlers::update_task))
        .route("/:task_id", delete(tasks_handlers::delete_task))
}

/// Creates the agents API router
pub fn create_agents_router() -> Router<DbState> {
    Router::new()
        .route("/", get(agents_handlers::list_agents))
        .route("/:agent_id", get(agents_handlers::get_agent))
        .route(
            "/users/:user_id",
            get(agents_handlers::list_user_agents),
        )
        .route(
            "/users/:user_id/:agent_id",
            get(agents_handlers::get_user_agent),
        )
        .route(
            "/users/:user_id/:agent_id/activation",
            put(agents_handlers::update_agent_activation),
        )
}

/// Creates the users API router
pub fn create_users_router() -> Router<DbState> {
    Router::new()
        .route("/current", get(users_handlers::get_current_user))
        .route("/:user_id", get(users_handlers::get_user))
        .route(
            "/:user_id/default-agent",
            put(users_handlers::set_default_agent),
        )
        .route("/:user_id/theme", put(users_handlers::update_theme))
}

/// Creates the tags API router
pub fn create_tags_router() -> Router<DbState> {
    Router::new()
        .route("/", get(tags_handlers::list_tags))
        .route("/", post(tags_handlers::create_tag))
        .route("/:tag_id", get(tags_handlers::get_tag))
        .route("/:tag_id", put(tags_handlers::update_tag))
        .route("/:tag_id", delete(tags_handlers::delete_tag))
        .route("/:tag_id/archive", post(tags_handlers::archive_tag))
        .route("/:tag_id/unarchive", post(tags_handlers::unarchive_tag))
}

/// Creates the executions API router for task executions
pub fn create_executions_router() -> Router<DbState> {
    Router::new()
        .route("/tasks/:task_id/executions", get(executions_handlers::list_executions))
        .route("/executions", post(executions_handlers::create_execution))
        .route("/executions/:execution_id", get(executions_handlers::get_execution))
        .route("/executions/:execution_id", put(executions_handlers::update_execution))
        .route("/executions/:execution_id", delete(executions_handlers::delete_execution))
        .route("/executions/:execution_id/reviews", get(executions_handlers::list_reviews))
        .route("/reviews", post(executions_handlers::create_review))
        .route("/reviews/:review_id", get(executions_handlers::get_review))
        .route("/reviews/:review_id", put(executions_handlers::update_review))
        .route("/reviews/:review_id", delete(executions_handlers::delete_review))
}
