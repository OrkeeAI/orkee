use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::db::DbState;

pub mod agents_handlers;
pub mod change_handlers;
pub mod executions_handlers;
pub mod handlers;
pub mod prd_handlers;
pub mod response;
pub mod spec_handlers;
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

/// Creates the PRD API router for Product Requirements Documents
pub fn create_prds_router() -> Router<DbState> {
    Router::new()
        .route("/:project_id/prds", get(prd_handlers::list_prds))
        .route("/:project_id/prds", post(prd_handlers::create_prd))
        .route("/:project_id/prds/:prd_id", get(prd_handlers::get_prd))
        .route("/:project_id/prds/:prd_id", put(prd_handlers::update_prd))
        .route("/:project_id/prds/:prd_id", delete(prd_handlers::delete_prd))
        .route("/:project_id/prds/:prd_id/capabilities", get(prd_handlers::get_prd_capabilities))
}

/// Creates the specs API router for OpenSpec capabilities
pub fn create_specs_router() -> Router<DbState> {
    Router::new()
        .route("/:project_id/specs", get(spec_handlers::list_capabilities))
        .route("/:project_id/specs", post(spec_handlers::create_capability))
        .route("/:project_id/specs/:capability_id", get(spec_handlers::get_capability))
        .route("/:project_id/specs/:capability_id", put(spec_handlers::update_capability))
        .route("/:project_id/specs/:capability_id", delete(spec_handlers::delete_capability))
        .route("/:project_id/specs/:capability_id/requirements", get(spec_handlers::get_capability_requirements))
        .route("/specs/validate", post(spec_handlers::validate_spec))
}

/// Creates the changes API router for spec changes and deltas
pub fn create_changes_router() -> Router<DbState> {
    Router::new()
        .route("/:project_id/changes", get(change_handlers::list_changes))
        .route("/:project_id/changes", post(change_handlers::create_change))
        .route("/:project_id/changes/:change_id", get(change_handlers::get_change))
        .route("/:project_id/changes/:change_id/status", put(change_handlers::update_change_status))
        .route("/:project_id/changes/:change_id/deltas", get(change_handlers::get_change_deltas))
        .route("/:project_id/changes/:change_id/deltas", post(change_handlers::create_delta))
}
