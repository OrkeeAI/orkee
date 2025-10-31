// ABOUTME: HTTP API layer for Orkee providing REST endpoints and routing
// ABOUTME: Integration layer that depends on all domain packages

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use orkee_projects::DbState;

pub mod agents_handlers;
pub mod ai_handlers;
pub mod ai_proxy_handlers;
pub mod ai_usage_log_handlers;
pub mod auth;
pub mod change_handlers;
pub mod context_handlers;
pub mod epic_handlers;
pub mod executions_handlers;
pub mod github_sync_handlers;
pub mod graph_handlers;
pub mod handlers;
pub mod ideate_conversational_handlers;
pub mod ideate_dependency_handlers;
pub mod ideate_generation_handlers;
pub mod ideate_handlers;
pub mod ideate_research_handlers;
pub mod ideate_roundtable_handlers;
pub mod models_handlers;
pub mod prd_handlers;
pub mod response;
pub mod security_handlers;
pub mod spec_handlers;
pub mod tags_handlers;
pub mod task_decomposition_handlers;
pub mod task_spec_handlers;
pub mod tasks_handlers;
pub mod template_handlers;
pub mod users_handlers;
pub mod validation;

/// Creates the projects API router
pub fn create_projects_router() -> Router {
    Router::new()
        .route("/", get(handlers::list_projects))
        .route("/", post(handlers::create_project))
        .route("/{id}", get(handlers::get_project))
        .route("/{id}", put(handlers::update_project))
        .route("/{id}", delete(handlers::delete_project))
        .route("/by-name/{name}", get(handlers::get_project_by_name))
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

/// Creates the tasks API router (nested under /api/projects/{project_id}/tasks)
pub fn create_tasks_router() -> Router<DbState> {
    Router::new()
        .route("/", get(tasks_handlers::list_tasks))
        .route("/", post(tasks_handlers::create_task))
        .route("/{task_id}", get(tasks_handlers::get_task))
        .route("/{task_id}", put(tasks_handlers::update_task))
        .route("/{task_id}", delete(tasks_handlers::delete_task))
}

/// Creates the agents API router
pub fn create_agents_router() -> Router<DbState> {
    Router::new()
        .route("/", get(agents_handlers::list_agents))
        .route("/{agent_id}", get(agents_handlers::get_agent))
        .route("/users/{user_id}", get(agents_handlers::list_user_agents))
        .route(
            "/users/{user_id}/{agent_id}",
            get(agents_handlers::get_user_agent),
        )
        .route(
            "/users/{user_id}/{agent_id}/activation",
            put(agents_handlers::update_agent_activation),
        )
}

/// Creates the models API router
pub fn create_models_router() -> Router<DbState> {
    Router::new()
        .route("/", get(models_handlers::list_models))
        .route("/{model_id}", get(models_handlers::get_model))
        .route(
            "/provider/{provider}",
            get(models_handlers::list_models_by_provider),
        )
}

/// Creates the users API router
pub fn create_users_router() -> Router<DbState> {
    Router::new()
        .route("/current", get(users_handlers::get_current_user))
        .route("/{user_id}", get(users_handlers::get_user))
        .route(
            "/{user_id}/default-agent",
            put(users_handlers::set_default_agent),
        )
        .route("/{user_id}/theme", put(users_handlers::update_theme))
        .route("/credentials", put(users_handlers::update_credentials))
        .route("/anthropic-key", get(users_handlers::get_anthropic_key))
}

/// Creates the tags API router
pub fn create_tags_router() -> Router<DbState> {
    Router::new()
        .route("/", get(tags_handlers::list_tags))
        .route("/", post(tags_handlers::create_tag))
        .route("/{tag_id}", get(tags_handlers::get_tag))
        .route("/{tag_id}", put(tags_handlers::update_tag))
        .route("/{tag_id}", delete(tags_handlers::delete_tag))
        .route("/{tag_id}/archive", post(tags_handlers::archive_tag))
        .route("/{tag_id}/unarchive", post(tags_handlers::unarchive_tag))
}

/// Creates the executions API router for task executions
pub fn create_executions_router() -> Router<DbState> {
    Router::new()
        .route(
            "/tasks/{task_id}/executions",
            get(executions_handlers::list_executions),
        )
        .route("/executions", post(executions_handlers::create_execution))
        .route(
            "/executions/{execution_id}",
            get(executions_handlers::get_execution),
        )
        .route(
            "/executions/{execution_id}",
            put(executions_handlers::update_execution),
        )
        .route(
            "/executions/{execution_id}",
            delete(executions_handlers::delete_execution),
        )
        .route(
            "/executions/{execution_id}/reviews",
            get(executions_handlers::list_reviews),
        )
        .route("/reviews", post(executions_handlers::create_review))
        .route("/reviews/{review_id}", get(executions_handlers::get_review))
        .route(
            "/reviews/{review_id}",
            put(executions_handlers::update_review),
        )
        .route(
            "/reviews/{review_id}",
            delete(executions_handlers::delete_review),
        )
}

/// Creates the PRD API router for Product Requirements Documents
pub fn create_prds_router() -> Router<DbState> {
    Router::new()
        .route("/{project_id}/prds", get(prd_handlers::list_prds))
        .route("/{project_id}/prds", post(prd_handlers::create_prd))
        .route("/{project_id}/prds/{prd_id}", get(prd_handlers::get_prd))
        .route("/{project_id}/prds/{prd_id}", put(prd_handlers::update_prd))
        .route(
            "/{project_id}/prds/{prd_id}",
            delete(prd_handlers::delete_prd),
        )
        .route(
            "/{project_id}/prds/{prd_id}/capabilities",
            get(prd_handlers::get_prd_capabilities),
        )
        .route(
            "/{project_id}/prds/{prd_id}/epics",
            get(epic_handlers::list_epics_by_prd),
        )
}

/// Creates the Epic API router for Epic management (CCPM workflow)
pub fn create_epics_router() -> Router<DbState> {
    Router::new()
        .route("/{project_id}/epics", get(epic_handlers::list_epics))
        .route("/{project_id}/epics", post(epic_handlers::create_epic))
        .route(
            "/{project_id}/epics/generate",
            post(epic_handlers::generate_epic_from_prd),
        )
        .route(
            "/{project_id}/epics/{epic_id}",
            get(epic_handlers::get_epic),
        )
        .route(
            "/{project_id}/epics/{epic_id}",
            put(epic_handlers::update_epic),
        )
        .route(
            "/{project_id}/epics/{epic_id}",
            delete(epic_handlers::delete_epic),
        )
        .route(
            "/{project_id}/epics/{epic_id}/tasks",
            get(task_decomposition_handlers::get_epic_tasks),
        )
        .route(
            "/{project_id}/epics/{epic_id}/progress",
            get(epic_handlers::calculate_epic_progress),
        )
        .route(
            "/{project_id}/epics/{epic_id}/analyze-work",
            post(task_decomposition_handlers::analyze_work_streams),
        )
        .route(
            "/{project_id}/epics/{epic_id}/decompose",
            post(task_decomposition_handlers::decompose_epic),
        )
}

/// Creates the Brainstorm API router for PRD ideation and ideateing
pub fn create_ideate_router() -> Router<DbState> {
    Router::new()
        .route("/ideate/start", post(ideate_handlers::start_ideate))
        .route("/ideate/{session_id}", get(ideate_handlers::get_ideate))
        .route("/ideate/{session_id}", put(ideate_handlers::update_ideate))
        .route(
            "/ideate/{session_id}",
            delete(ideate_handlers::delete_ideate),
        )
        .route(
            "/ideate/{session_id}/skip-section",
            post(ideate_handlers::skip_section),
        )
        .route(
            "/ideate/{session_id}/status",
            get(ideate_handlers::get_status),
        )
        .route(
            "/{project_id}/ideate/sessions",
            get(ideate_handlers::list_ideates),
        )
        // Quick Mode routes
        .route(
            "/ideate/{session_id}/quick-generate",
            post(ideate_handlers::quick_generate),
        )
        .route(
            "/ideate/{session_id}/quick-expand",
            post(ideate_handlers::quick_expand),
        )
        .route(
            "/ideate/{session_id}/preview",
            get(ideate_handlers::get_preview),
        )
        .route(
            "/ideate/{session_id}/save-as-prd",
            post(ideate_handlers::save_as_prd),
        )
        .route(
            "/ideate/{session_id}/save-sections",
            post(ideate_handlers::save_sections),
        )
        // Guided Mode - Section routes
        // Overview
        .route(
            "/ideate/{session_id}/overview",
            post(ideate_handlers::save_overview),
        )
        .route(
            "/ideate/{session_id}/overview",
            get(ideate_handlers::get_overview),
        )
        .route(
            "/ideate/{session_id}/overview",
            delete(ideate_handlers::delete_overview),
        )
        // UX
        .route("/ideate/{session_id}/ux", post(ideate_handlers::save_ux))
        .route("/ideate/{session_id}/ux", get(ideate_handlers::get_ux))
        .route(
            "/ideate/{session_id}/ux",
            delete(ideate_handlers::delete_ux),
        )
        // Technical
        .route(
            "/ideate/{session_id}/technical",
            post(ideate_handlers::save_technical),
        )
        .route(
            "/ideate/{session_id}/technical",
            get(ideate_handlers::get_technical),
        )
        .route(
            "/ideate/{session_id}/technical",
            delete(ideate_handlers::delete_technical),
        )
        // Roadmap
        .route(
            "/ideate/{session_id}/roadmap",
            post(ideate_handlers::save_roadmap),
        )
        .route(
            "/ideate/{session_id}/roadmap",
            get(ideate_handlers::get_roadmap),
        )
        .route(
            "/ideate/{session_id}/roadmap",
            delete(ideate_handlers::delete_roadmap),
        )
        // Dependencies
        .route(
            "/ideate/{session_id}/dependencies",
            post(ideate_handlers::save_dependencies),
        )
        .route(
            "/ideate/{session_id}/dependencies",
            get(ideate_handlers::get_dependencies),
        )
        .route(
            "/ideate/{session_id}/dependencies",
            delete(ideate_handlers::delete_dependencies),
        )
        // Risks
        .route(
            "/ideate/{session_id}/risks",
            post(ideate_handlers::save_risks),
        )
        .route(
            "/ideate/{session_id}/risks",
            get(ideate_handlers::get_risks),
        )
        .route(
            "/ideate/{session_id}/risks",
            delete(ideate_handlers::delete_risks),
        )
        // Research
        .route(
            "/ideate/{session_id}/research",
            post(ideate_handlers::save_research),
        )
        .route(
            "/ideate/{session_id}/research",
            get(ideate_handlers::get_research),
        )
        .route(
            "/ideate/{session_id}/research",
            delete(ideate_handlers::delete_research),
        )
        // Navigation
        .route(
            "/ideate/{session_id}/next-section",
            get(ideate_handlers::get_next_section),
        )
        .route(
            "/ideate/{session_id}/navigate",
            post(ideate_handlers::navigate_to),
        )
        // Phase 4: Dependency Intelligence routes
        .route(
            "/ideate/{session_id}/features/dependencies",
            get(ideate_dependency_handlers::get_dependencies),
        )
        .route(
            "/ideate/{session_id}/features/dependencies",
            post(ideate_dependency_handlers::create_dependency),
        )
        .route(
            "/ideate/{session_id}/features/dependencies/{dependency_id}",
            delete(ideate_dependency_handlers::delete_dependency),
        )
        .route(
            "/ideate/{session_id}/dependencies/analyze",
            post(ideate_dependency_handlers::analyze_dependencies),
        )
        .route(
            "/ideate/{session_id}/dependencies/optimize",
            post(ideate_dependency_handlers::optimize_build_order),
        )
        .route(
            "/ideate/{session_id}/dependencies/build-order",
            get(ideate_dependency_handlers::get_build_order),
        )
        .route(
            "/ideate/{session_id}/dependencies/circular",
            get(ideate_dependency_handlers::get_circular_dependencies),
        )
        .route(
            "/ideate/{session_id}/features/suggest-visible",
            get(ideate_dependency_handlers::suggest_quick_wins),
        )
        // Phase 5: Comprehensive Mode - Research & Competitor Analysis routes
        .route(
            "/ideate/{session_id}/research/competitors/analyze",
            post(ideate_research_handlers::analyze_competitor),
        )
        .route(
            "/ideate/{session_id}/research/competitors",
            get(ideate_research_handlers::get_competitors),
        )
        .route(
            "/ideate/{session_id}/research/gaps/analyze",
            post(ideate_research_handlers::analyze_gaps),
        )
        .route(
            "/ideate/{session_id}/research/patterns/extract",
            post(ideate_research_handlers::extract_patterns),
        )
        .route(
            "/ideate/{session_id}/research/similar-projects",
            post(ideate_research_handlers::add_similar_project),
        )
        .route(
            "/ideate/{session_id}/research/similar-projects",
            get(ideate_research_handlers::get_similar_projects),
        )
        .route(
            "/ideate/{session_id}/research/lessons/extract",
            post(ideate_research_handlers::extract_lessons),
        )
        .route(
            "/ideate/{session_id}/research/synthesize",
            post(ideate_research_handlers::synthesize_research),
        )
        // Phase 6: Comprehensive Mode - Expert Roundtable routes
        // Expert management
        .route(
            "/ideate/{session_id}/experts",
            get(ideate_roundtable_handlers::list_experts),
        )
        .route(
            "/ideate/{session_id}/experts",
            post(ideate_roundtable_handlers::create_expert),
        )
        .route(
            "/ideate/{session_id}/experts/suggest",
            post(ideate_roundtable_handlers::suggest_experts),
        )
        // Roundtable session management
        .route(
            "/ideate/{session_id}/roundtable",
            post(ideate_roundtable_handlers::create_roundtable),
        )
        .route(
            "/ideate/{session_id}/roundtables",
            get(ideate_roundtable_handlers::list_roundtables),
        )
        .route(
            "/ideate/roundtable/{roundtable_id}",
            get(ideate_roundtable_handlers::get_roundtable),
        )
        .route(
            "/ideate/roundtable/{roundtable_id}/participants",
            post(ideate_roundtable_handlers::add_participants),
        )
        .route(
            "/ideate/roundtable/{roundtable_id}/participants",
            get(ideate_roundtable_handlers::get_participants),
        )
        // Discussion operations
        .route(
            "/ideate/roundtable/{roundtable_id}/start",
            post(ideate_roundtable_handlers::start_discussion),
        )
        .route(
            "/ideate/roundtable/{roundtable_id}/stream",
            get(ideate_roundtable_handlers::stream_discussion),
        )
        .route(
            "/ideate/roundtable/{roundtable_id}/interjection",
            post(ideate_roundtable_handlers::send_interjection),
        )
        .route(
            "/ideate/roundtable/{roundtable_id}/messages",
            get(ideate_roundtable_handlers::get_messages),
        )
        // Insight extraction
        .route(
            "/ideate/roundtable/{roundtable_id}/insights/extract",
            post(ideate_roundtable_handlers::extract_insights),
        )
        .route(
            "/ideate/roundtable/{roundtable_id}/insights",
            get(ideate_roundtable_handlers::get_insights),
        )
        // Statistics
        .route(
            "/ideate/roundtable/{roundtable_id}/statistics",
            get(ideate_roundtable_handlers::get_statistics),
        )
        // Phase 7: PRD Generation & Export routes
        .route(
            "/ideate/{session_id}/prd/generate",
            post(ideate_generation_handlers::generate_prd),
        )
        .route(
            "/ideate/{session_id}/prd/fill-sections",
            post(ideate_generation_handlers::fill_skipped_sections),
        )
        .route(
            "/ideate/{session_id}/prd/regenerate-section",
            post(ideate_generation_handlers::regenerate_section),
        )
        .route(
            "/ideate/{session_id}/prd/regenerate-template",
            post(ideate_generation_handlers::regenerate_prd_with_template),
        )
        .route(
            "/ideate/{session_id}/prd/regenerate-template-stream",
            post(ideate_generation_handlers::regenerate_prd_with_template_stream),
        )
        .route(
            "/ideate/{session_id}/prd/preview",
            get(ideate_generation_handlers::get_prd_preview),
        )
        .route(
            "/ideate/{session_id}/prd/export",
            post(ideate_generation_handlers::export_prd),
        )
        .route(
            "/ideate/{session_id}/prd/completeness",
            get(ideate_generation_handlers::get_completeness),
        )
        .route(
            "/ideate/{session_id}/prd/history",
            get(ideate_generation_handlers::get_generation_history),
        )
        .route(
            "/ideate/{session_id}/prd/validation",
            get(ideate_generation_handlers::validate_prd),
        )
        // Phase 8: Templates routes
        .route("/ideate/templates", get(ideate_handlers::list_templates))
        .route("/ideate/templates", post(ideate_handlers::create_template))
        .route(
            "/ideate/templates/{template_id}",
            get(ideate_handlers::get_template),
        )
        .route(
            "/ideate/templates/{template_id}",
            put(ideate_handlers::update_template),
        )
        .route(
            "/ideate/templates/{template_id}",
            delete(ideate_handlers::delete_template),
        )
        .route(
            "/ideate/templates/by-type/{project_type}",
            get(ideate_handlers::get_templates_by_type),
        )
        .route(
            "/ideate/templates/suggest",
            post(ideate_handlers::suggest_template),
        )
        // Conversational Mode routes (CCPM)
        .route(
            "/ideate/conversational/{session_id}/history",
            get(ideate_conversational_handlers::get_history),
        )
        .route(
            "/ideate/conversational/{session_id}/message",
            post(ideate_conversational_handlers::send_message),
        )
        .route(
            "/ideate/conversational/questions",
            get(ideate_conversational_handlers::get_discovery_questions),
        )
        .route(
            "/ideate/conversational/{session_id}/suggested-questions",
            get(ideate_conversational_handlers::get_suggested_questions),
        )
        .route(
            "/ideate/conversational/{session_id}/insights",
            get(ideate_conversational_handlers::get_insights),
        )
        .route(
            "/ideate/conversational/{session_id}/insights",
            post(ideate_conversational_handlers::create_insight),
        )
        .route(
            "/ideate/conversational/{session_id}/quality",
            get(ideate_conversational_handlers::get_quality_metrics),
        )
        .route(
            "/ideate/conversational/{session_id}/status",
            put(ideate_conversational_handlers::update_status),
        )
        .route(
            "/ideate/conversational/{session_id}/generate-prd",
            post(ideate_conversational_handlers::generate_prd),
        )
        .route(
            "/ideate/conversational/{session_id}/validate",
            get(ideate_conversational_handlers::validate_for_prd),
        )
}

/// Creates the specs API router for OpenSpec capabilities
pub fn create_specs_router() -> Router<DbState> {
    Router::new()
        .route("/{project_id}/specs", get(spec_handlers::list_capabilities))
        .route(
            "/{project_id}/specs",
            post(spec_handlers::create_capability),
        )
        .route(
            "/{project_id}/specs/{capability_id}",
            get(spec_handlers::get_capability),
        )
        .route(
            "/{project_id}/specs/{capability_id}",
            put(spec_handlers::update_capability),
        )
        .route(
            "/{project_id}/specs/{capability_id}",
            delete(spec_handlers::delete_capability),
        )
        .route(
            "/{project_id}/specs/{capability_id}/requirements",
            get(spec_handlers::get_capability_requirements),
        )
        .route("/specs/validate", post(spec_handlers::validate_spec))
}

/// Creates the changes API router for spec changes and deltas
pub fn create_changes_router() -> Router<DbState> {
    Router::new()
        .route("/{project_id}/changes", get(change_handlers::list_changes))
        .route(
            "/{project_id}/changes",
            post(change_handlers::create_change),
        )
        .route(
            "/{project_id}/changes/{change_id}",
            get(change_handlers::get_change),
        )
        .route(
            "/{project_id}/changes/{change_id}/status",
            put(change_handlers::update_change_status),
        )
        .route(
            "/{project_id}/changes/{change_id}/validate",
            get(change_handlers::validate_change),
        )
        .route(
            "/{project_id}/changes/{change_id}/archive",
            post(change_handlers::archive_change),
        )
        .route(
            "/{project_id}/changes/{change_id}/deltas",
            get(change_handlers::get_change_deltas),
        )
        .route(
            "/{project_id}/changes/{change_id}/deltas",
            post(change_handlers::create_delta),
        )
        // Task completion tracking routes
        .route(
            "/{project_id}/changes/{change_id}/tasks",
            get(change_handlers::get_change_tasks),
        )
        .route(
            "/{project_id}/changes/{change_id}/tasks/parse",
            post(change_handlers::parse_change_tasks),
        )
        .route(
            "/{project_id}/changes/{change_id}/tasks/bulk",
            put(change_handlers::bulk_update_tasks),
        )
        .route(
            "/{project_id}/changes/{change_id}/tasks/{task_id}",
            put(change_handlers::update_task),
        )
}

/// Creates the task-spec integration API router
pub fn create_task_spec_router() -> Router<DbState> {
    Router::new()
        .route(
            "/tasks/{task_id}/link-spec",
            post(task_spec_handlers::link_task_to_requirement),
        )
        .route(
            "/tasks/{task_id}/spec-links",
            get(task_spec_handlers::get_task_spec_links),
        )
        .route(
            "/tasks/{task_id}/validate-spec",
            post(task_spec_handlers::validate_task_against_spec),
        )
        .route(
            "/tasks/{task_id}/suggest-spec",
            post(task_spec_handlers::suggest_spec_from_task),
        )
        .route(
            "/{project_id}/tasks/generate-from-spec",
            post(task_spec_handlers::generate_tasks_from_spec),
        )
        .route(
            "/{project_id}/tasks/orphans",
            get(task_spec_handlers::find_orphan_tasks),
        )
}

/// Creates the AI proxy API router for AI-powered operations
pub fn create_ai_router() -> Router<DbState> {
    Router::new()
        .route("/ai/analyze-prd", post(ai_handlers::analyze_prd))
        .route("/ai/generate-spec", post(ai_handlers::generate_spec))
        .route("/ai/suggest-tasks", post(ai_handlers::suggest_tasks))
        .route("/ai/refine-spec", post(ai_handlers::refine_spec))
        .route(
            "/ai/validate-completion",
            post(ai_handlers::validate_completion),
        )
}

/// Creates the AI usage logs API router for cost tracking
pub fn create_ai_usage_router() -> Router<DbState> {
    Router::new()
        .route("/logs", get(ai_usage_log_handlers::list_logs))
        .route("/stats", get(ai_usage_log_handlers::get_stats))
}

/// Creates the AI proxy API router for secure credential management
pub fn create_ai_proxy_router() -> Router<DbState> {
    Router::new()
        .route(
            "/ai/anthropic/{*path}",
            post(ai_proxy_handlers::proxy_anthropic),
        )
        .route("/ai/openai/{*path}", post(ai_proxy_handlers::proxy_openai))
        .route("/ai/google/{*path}", post(ai_proxy_handlers::proxy_google))
        .route("/ai/xai/{*path}", post(ai_proxy_handlers::proxy_xai))
}

/// Creates the security API router for encryption and key management
pub fn create_security_router() -> Router<DbState> {
    Router::new()
        .route(
            "/security/status",
            get(security_handlers::get_security_status),
        )
        .route(
            "/security/keys-status",
            get(security_handlers::get_keys_status),
        )
        .route(
            "/security/set-password",
            post(security_handlers::set_password),
        )
        .route(
            "/security/change-password",
            post(security_handlers::change_password),
        )
        .route(
            "/security/remove-password",
            post(security_handlers::remove_password),
        )
}

/// Creates the context API router for context generation and management
pub fn create_context_router() -> Router<DbState> {
    Router::new()
        // Basic context generation
        .route(
            "/{project_id}/context/generate",
            post(context_handlers::generate_context),
        )
        .route(
            "/{project_id}/files",
            get(context_handlers::list_project_files),
        )
        .route(
            "/{project_id}/context/configurations",
            get(context_handlers::list_configurations),
        )
        .route(
            "/{project_id}/context/configurations",
            post(context_handlers::save_configuration),
        )
        // OpenSpec integration
        .route(
            "/{project_id}/context/from-prd",
            post(context_handlers::generate_prd_context),
        )
        .route(
            "/{project_id}/context/from-task",
            post(context_handlers::generate_task_context),
        )
        .route(
            "/{project_id}/context/validate-spec",
            post(context_handlers::validate_spec),
        )
        // History and analytics
        .route(
            "/{project_id}/context/history",
            get(context_handlers::get_context_history),
        )
        .route(
            "/{project_id}/context/stats",
            get(context_handlers::get_context_stats),
        )
        .route(
            "/{project_id}/context/restore",
            post(context_handlers::restore_context_snapshot),
        )
}

/// Creates the graph API router for code visualization
pub fn create_graph_router() -> Router<DbState> {
    Router::new()
        .route(
            "/{project_id}/graph/dependencies",
            get(graph_handlers::get_dependency_graph),
        )
        .route(
            "/{project_id}/graph/symbols",
            get(graph_handlers::get_symbol_graph),
        )
        .route(
            "/{project_id}/graph/modules",
            get(graph_handlers::get_module_graph),
        )
        .route(
            "/{project_id}/graph/spec-mapping",
            get(graph_handlers::get_spec_mapping_graph),
        )
}

/// Creates the templates API router for PRD output template management
pub fn create_templates_router() -> Router<DbState> {
    Router::new()
        .route("/templates", get(template_handlers::list_templates))
        .route("/templates", post(template_handlers::create_template))
        .route(
            "/templates/{template_id}",
            get(template_handlers::get_template),
        )
        .route(
            "/templates/{template_id}",
            put(template_handlers::update_template),
        )
        .route(
            "/templates/{template_id}",
            delete(template_handlers::delete_template),
        )
}

/// Creates the GitHub sync API router for syncing Epics and Tasks to GitHub
pub fn create_github_sync_router() -> Router<DbState> {
    Router::new()
        .route(
            "/github/sync/epic/{epic_id}",
            post(github_sync_handlers::sync_epic),
        )
        .route(
            "/github/sync/tasks/{epic_id}",
            post(github_sync_handlers::sync_tasks),
        )
        .route(
            "/github/sync/status/{project_id}",
            get(github_sync_handlers::get_sync_status),
        )
}
