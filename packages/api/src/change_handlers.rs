// ABOUTME: HTTP request handlers for spec change operations
// ABOUTME: Handles CRUD operations for spec changes and deltas

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::auth::CurrentUser;
use super::response::{
    bad_request, created_or_internal_error, ok_or_internal_error, ok_or_not_found, ApiResponse,
};
use super::validation;
use openspec::db as openspec_db;
use openspec::types::{ChangeStatus, DeltaType, TaskUpdate};
use orkee_projects::pagination::{PaginatedResponse, PaginationParams};
use orkee_projects::DbState;

/// Validate if a status transition is allowed
fn is_valid_status_transition(from: ChangeStatus, to: ChangeStatus) -> bool {
    use ChangeStatus::*;

    match (from, to) {
        // Same status is always valid (no-op)
        (a, b) if a == b => true,

        // Draft can go to Review or Archived
        (Draft, Review) | (Draft, Archived) => true,

        // Review can go back to Draft, forward to Approved, or be Archived
        (Review, Draft) | (Review, Approved) | (Review, Archived) => true,

        // Approved can go back to Draft, forward to Implementing, or be Archived
        (Approved, Draft) | (Approved, Implementing) | (Approved, Archived) => true,

        // Implementing can go back to Approved, forward to Completed, or be Archived
        (Implementing, Approved) | (Implementing, Completed) | (Implementing, Archived) => true,

        // Completed can go back to Implementing or be Archived
        (Completed, Implementing) | (Completed, Archived) => true,

        // Archived is a final state - no transitions allowed
        (Archived, _) => false,

        // All other transitions are invalid
        _ => false,
    }
}

/// List all changes for a project
pub async fn list_changes(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Listing changes for project: {} (page: {})",
        project_id,
        pagination.page()
    );

    let result = openspec_db::get_spec_changes_by_project_paginated(
        &db.pool,
        &project_id,
        Some(pagination.limit()),
        Some(pagination.offset()),
    )
    .await
    .map(|(changes, total)| PaginatedResponse::new(changes, &pagination, total));

    ok_or_internal_error(result, "Failed to list changes")
}

/// Get a single change by ID
pub async fn get_change(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting change: {}", change_id);

    let result = openspec_db::get_spec_change(&db.pool, &change_id).await;
    ok_or_not_found(result, "Change not found")
}

/// Request body for creating a change
#[derive(Deserialize)]
pub struct CreateChangeRequest {
    #[serde(rename = "prdId")]
    pub prd_id: Option<String>,
    #[serde(rename = "proposalMarkdown")]
    pub proposal_markdown: String,
    #[serde(rename = "tasksMarkdown")]
    pub tasks_markdown: String,
    #[serde(rename = "designMarkdown")]
    pub design_markdown: Option<String>,
}

/// Create a new change
pub async fn create_change(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path(project_id): Path<String>,
    Json(request): Json<CreateChangeRequest>,
) -> impl IntoResponse {
    info!(
        "Creating change for project: {} (user: {})",
        project_id, current_user.id
    );

    // Validate and sanitize inputs
    let validated_proposal =
        match validation::validate_proposal_markdown(&request.proposal_markdown) {
            Ok(v) => v,
            Err(e) => return bad_request(e, "Invalid proposal markdown"),
        };

    let validated_tasks = match validation::validate_tasks_markdown(&request.tasks_markdown) {
        Ok(v) => v,
        Err(e) => return bad_request(e, "Invalid tasks markdown"),
    };

    let validated_design =
        match validation::validate_design_markdown(request.design_markdown.as_deref()) {
            Ok(v) => v,
            Err(e) => return bad_request(e, "Invalid design markdown"),
        };

    let result = openspec_db::create_spec_change(
        &db.pool,
        &project_id,
        request.prd_id.as_deref(),
        &validated_proposal,
        &validated_tasks,
        validated_design.as_deref(),
        &current_user.id,
    )
    .await;

    created_or_internal_error(result, "Failed to create change")
}

/// Request body for updating change status
#[derive(Deserialize)]
pub struct UpdateChangeStatusRequest {
    pub status: ChangeStatus,
    #[serde(rename = "approvedBy")]
    pub approved_by: Option<String>,
    pub notes: Option<String>,
}

/// Update change status
pub async fn update_change_status(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path((_project_id, change_id)): Path<(String, String)>,
    Json(request): Json<UpdateChangeStatusRequest>,
) -> impl IntoResponse {
    info!(
        "Updating change status: {} (user: {})",
        change_id, current_user.id
    );

    // Get current change to check current status
    let current_change = match openspec_db::get_spec_change(&db.pool, &change_id).await {
        Ok(change) => change,
        Err(openspec_db::DbError::NotFound(msg)) => {
            return (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error(msg)),
            )
                .into_response()
        }
        Err(e) => return ok_or_internal_error::<(), _>(Err(e), "Failed to get change"),
    };

    // Validate status transition
    if !is_valid_status_transition(current_change.status.clone(), request.status.clone()) {
        return bad_request(
            format!(
                "Invalid status transition from {:?} to {:?}",
                current_change.status, request.status
            ),
            "Invalid status transition",
        );
    }

    // Validate approvedBy if provided
    let validated_approved_by = match &request.approved_by {
        Some(user_id) => match validation::validate_user_id(user_id) {
            Ok(v) => Some(v),
            Err(e) => return bad_request(e, "Invalid approver user ID"),
        },
        None => None,
    };

    let result = openspec_db::update_spec_change_status(
        &db.pool,
        &change_id,
        request.status,
        validated_approved_by.as_deref(),
    )
    .await;

    ok_or_internal_error(result, "Failed to update change status")
}

/// Get all deltas for a change
pub async fn get_change_deltas(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    info!(
        "Getting deltas for change: {} (page: {})",
        change_id,
        pagination.page()
    );

    let result = openspec_db::get_deltas_by_change_paginated(
        &db.pool,
        &change_id,
        Some(pagination.limit()),
        Some(pagination.offset()),
    )
    .await
    .map(|(deltas, total)| PaginatedResponse::new(deltas, &pagination, total));

    ok_or_internal_error(result, "Failed to get change deltas")
}

/// Request body for creating a delta
#[derive(Deserialize)]
pub struct CreateDeltaRequest {
    #[serde(rename = "capabilityId")]
    pub capability_id: Option<String>,
    #[serde(rename = "capabilityName")]
    pub capability_name: String,
    #[serde(rename = "deltaType")]
    pub delta_type: DeltaType,
    #[serde(rename = "deltaMarkdown")]
    pub delta_markdown: String,
    pub position: i32,
}

/// Create a new delta for a change
pub async fn create_delta(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path((_project_id, change_id)): Path<(String, String)>,
    Json(request): Json<CreateDeltaRequest>,
) -> impl IntoResponse {
    info!(
        "Creating delta for change: {} (user: {})",
        change_id, current_user.id
    );

    // Validate and sanitize inputs
    let validated_name = match validation::validate_capability_name(&request.capability_name) {
        Ok(v) => v,
        Err(e) => return bad_request(e, "Invalid capability name"),
    };

    let validated_markdown = match validation::validate_delta_markdown(&request.delta_markdown) {
        Ok(v) => v,
        Err(e) => return bad_request(e, "Invalid delta markdown"),
    };

    let result = openspec_db::create_spec_delta(
        &db.pool,
        &change_id,
        request.capability_id.as_deref(),
        &validated_name,
        request.delta_type,
        &validated_markdown,
        request.position,
    )
    .await;

    created_or_internal_error(result, "Failed to create delta")
}

/// Get all tasks for a change (parsed from tasks_markdown)
pub async fn get_change_tasks(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting tasks for change: {}", change_id);

    let result = openspec_db::get_change_tasks(&db.pool, &change_id).await;
    ok_or_internal_error(result, "Failed to get change tasks")
}

/// Request body for updating a task
#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(rename = "completedBy")]
    pub completed_by: Option<String>,
}

/// Update a task's completion status
pub async fn update_task(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path((_project_id, _change_id, task_id)): Path<(String, String, String)>,
    Json(request): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    info!(
        "Updating task: {} (completed: {}, user: {})",
        task_id, request.is_completed, current_user.id
    );

    // Validate completedBy if provided
    let validated_completed_by = match &request.completed_by {
        Some(user_id) => match validation::validate_user_id(user_id) {
            Ok(v) => Some(v),
            Err(e) => return bad_request(e, "Invalid completed_by user ID"),
        },
        None => None,
    };

    let result = openspec_db::update_change_task(
        &db.pool,
        &task_id,
        request.is_completed,
        validated_completed_by.as_deref(),
    )
    .await;

    ok_or_internal_error(result, "Failed to update task")
}

/// Parse tasks from a change's tasks_markdown and store them
pub async fn parse_change_tasks(
    State(db): State<DbState>,
    Path((_project_id, change_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Parsing tasks for change: {}", change_id);

    let result = openspec_db::parse_and_store_change_tasks(&db.pool, &change_id).await;
    ok_or_internal_error(result, "Failed to parse change tasks")
}

/// Request body for bulk task updates
#[derive(Deserialize)]
pub struct BulkUpdateTasksRequest {
    pub tasks: Vec<openspec::TaskUpdate>,
}

/// Update multiple tasks at once
pub async fn bulk_update_tasks(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path((_project_id, _change_id)): Path<(String, String)>,
    Json(request): Json<BulkUpdateTasksRequest>,
) -> impl IntoResponse {
    info!(
        "Bulk updating {} tasks (user: {})",
        request.tasks.len(),
        current_user.id
    );

    // Validate all completedBy fields in the batch
    let mut validated_tasks = Vec::with_capacity(request.tasks.len());
    for task_update in request.tasks {
        let validated_completed_by = match &task_update.completed_by {
            Some(user_id) => match validation::validate_user_id(user_id) {
                Ok(v) => Some(v),
                Err(e) => {
                    return bad_request(
                        e,
                        &format!("Invalid user ID in task {}", task_update.task_id),
                    )
                }
            },
            None => None,
        };

        validated_tasks.push(TaskUpdate {
            task_id: task_update.task_id,
            is_completed: task_update.is_completed,
            completed_by: validated_completed_by,
        });
    }

    let result = openspec_db::bulk_update_change_tasks(&db.pool, validated_tasks).await;
    ok_or_internal_error(result, "Failed to bulk update tasks")
}

/// Request body for validation
#[derive(Deserialize)]
pub struct ValidateChangeRequest {
    pub strict: Option<bool>,
}

/// Response for validation
#[derive(serde::Serialize)]
pub struct ValidationResultResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    #[serde(rename = "deltasValidated")]
    pub deltas_validated: usize,
}

/// Validate a change's deltas against OpenSpec format
pub async fn validate_change(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path((project_id, change_id)): Path<(String, String)>,
    Query(request): Query<ValidateChangeRequest>,
) -> impl IntoResponse {
    info!(
        "Validating change: {} (user: {})",
        change_id, current_user.id
    );

    let strict = request.strict.unwrap_or(false);

    // Verify project exists
    let project_exists: Result<bool, sqlx::Error> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)")
            .bind(&project_id)
            .fetch_one(&db.pool)
            .await;

    match project_exists {
        Ok(true) => {} // Project exists, continue
        Ok(false) => {
            return (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<ValidationResultResponse>::error(format!(
                    "Project not found: {}",
                    project_id
                ))),
            )
                .into_response()
        }
        Err(e) => {
            return ok_or_internal_error::<ValidationResultResponse, _>(
                Err(e),
                "Failed to verify project exists",
            )
        }
    }

    // Verify the change exists
    match openspec_db::get_spec_change(&db.pool, &change_id).await {
        Ok(_) => {} // Change exists, continue
        Err(openspec_db::DbError::NotFound(msg)) => {
            return (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<ValidationResultResponse>::error(msg)),
            )
                .into_response()
        }
        Err(e) => {
            return ok_or_internal_error::<ValidationResultResponse, _>(
                Err(e),
                "Failed to verify change exists",
            )
        }
    }

    // Get all deltas for the change
    let deltas = match openspec_db::get_deltas_by_change(&db.pool, &change_id).await {
        Ok(d) => d,
        Err(e) => {
            return ok_or_internal_error::<ValidationResultResponse, _>(
                Err(e),
                "Failed to get change deltas",
            )
        }
    };

    // Validate each delta
    let validator = openspec::markdown_validator::OpenSpecMarkdownValidator::new(strict);
    let mut all_errors = Vec::new();

    for delta in &deltas {
        let errors = validator.validate_delta_markdown(&delta.delta_markdown);
        all_errors.extend(errors.into_iter().map(|e| e.message));
    }

    let response = ValidationResultResponse {
        valid: all_errors.is_empty(),
        errors: all_errors.clone(),
        deltas_validated: deltas.len(),
    };

    // Audit log: Record validation operation
    let audit_record = serde_json::json!({
        "operation": "validate_change",
        "change_id": change_id,
        "strict_mode": strict,
        "valid": all_errors.is_empty(),
        "errors_count": all_errors.len(),
        "deltas_validated": deltas.len(),
    });

    // Get change to find associated PRD
    if let Ok(change) = openspec_db::get_spec_change(&db.pool, &change_id).await {
        if let Some(prd_id) = &change.prd_id {
            let audit_id = orkee_core::generate_project_id();
            if let Err(e) = sqlx::query(
                "INSERT INTO prd_spec_sync_history (id, prd_id, direction, changes_json, performed_by)
                 VALUES (?, ?, 'task_to_spec', ?, ?)",
            )
            .bind(audit_id)
            .bind(prd_id)
            .bind(audit_record.to_string())
            .bind(&current_user.id)
            .execute(&db.pool)
            .await
            {
                tracing::warn!("Failed to write audit log for validation: {}", e);
            }
        }
    }

    ok_or_internal_error::<ValidationResultResponse, openspec::DbError>(
        Ok(response),
        "Failed to validate change",
    )
}

/// Request body for archiving a change
#[derive(Deserialize)]
pub struct ArchiveChangeRequest {
    #[serde(rename = "applySpecs")]
    pub apply_specs: bool,
}

/// Response for archive operation
#[derive(serde::Serialize)]
pub struct ArchiveResultResponse {
    #[serde(rename = "changeId")]
    pub change_id: String,
    pub archived: bool,
    #[serde(rename = "specsApplied")]
    pub specs_applied: bool,
    #[serde(rename = "capabilitiesCreated")]
    pub capabilities_created: usize,
}

/// Archive a completed change and optionally apply its deltas
pub async fn archive_change(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path((project_id, change_id)): Path<(String, String)>,
    Json(request): Json<ArchiveChangeRequest>,
) -> impl IntoResponse {
    info!(
        "Archiving change: {} (apply_specs: {}, user: {})",
        change_id, request.apply_specs, current_user.id
    );

    // Verify project exists
    let project_exists: Result<bool, sqlx::Error> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)")
            .bind(&project_id)
            .fetch_one(&db.pool)
            .await;

    match project_exists {
        Ok(true) => {} // Project exists, continue
        Ok(false) => {
            return (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<ArchiveResultResponse>::error(format!(
                    "Project not found: {}",
                    project_id
                ))),
            )
                .into_response()
        }
        Err(e) => {
            return ok_or_internal_error::<ArchiveResultResponse, _>(
                Err(e),
                "Failed to verify project exists",
            )
        }
    }

    let result = openspec::archive::archive_change(&db.pool, &change_id, request.apply_specs).await;

    match result {
        Ok(capabilities_created) => {
            let response = ArchiveResultResponse {
                change_id: change_id.clone(),
                archived: true,
                specs_applied: request.apply_specs,
                capabilities_created,
            };

            // Audit log: Record archive operation
            let audit_record = serde_json::json!({
                "operation": "archive_change",
                "change_id": &change_id,
                "specs_applied": request.apply_specs,
                "capabilities_created": capabilities_created,
            });

            // Get change to find associated PRD
            if let Ok(change) = openspec_db::get_spec_change(&db.pool, &change_id).await {
                if let Some(prd_id) = &change.prd_id {
                    let audit_id = orkee_core::generate_project_id();
                    if let Err(e) = sqlx::query(
                        "INSERT INTO prd_spec_sync_history (id, prd_id, direction, changes_json, performed_by)
                         VALUES (?, ?, 'spec_to_prd', ?, ?)",
                    )
                    .bind(audit_id)
                    .bind(prd_id)
                    .bind(audit_record.to_string())
                    .bind(&current_user.id)
                    .execute(&db.pool)
                    .await
                    {
                        tracing::warn!("Failed to write audit log for archive: {}", e);
                    }
                }
            }

            ok_or_internal_error::<ArchiveResultResponse, openspec::ArchiveError>(
                Ok(response),
                "Failed to archive change",
            )
        }
        Err(e) => ok_or_internal_error::<ArchiveResultResponse, openspec::ArchiveError>(
            Err(e),
            "Failed to archive change",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use openspec::db::{create_prd, create_spec_change, create_spec_delta};
    use openspec::types::{PRDSource, PRDStatus};
    use sqlx::{Pool, Sqlite};
    use tower::ServiceExt;

    async fn setup_test_db() -> DbState {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("../storage/migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        DbState::new(pool).expect("Failed to create DbState")
    }

    async fn setup_test_data(pool: &Pool<Sqlite>, project_id: &str) -> (String, String) {
        // Create test project
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, description, created_at, updated_at)
             VALUES (?, 'Test Project', '/tmp/test', 'Test', datetime('now'), datetime('now'))",
        )
        .bind(project_id)
        .execute(pool)
        .await
        .unwrap();

        // Create PRD
        let prd = create_prd(
            pool,
            project_id,
            "Test PRD",
            "Content",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        // Create change
        let change = create_spec_change(
            pool,
            project_id,
            Some(&prd.id),
            "## Proposal\nTest proposal",
            "## Tasks\n- [ ] Task 1",
            Some("## Design\nTest design"),
            "test-user",
        )
        .await
        .unwrap();

        (prd.id, change.id)
    }

    #[tokio::test]
    async fn test_validate_change_with_valid_delta() {
        let db_state = setup_test_db().await;
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        // Add a valid delta
        let valid_delta = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        create_spec_delta(
            &db_state.pool,
            &change_id,
            None,
            "user-auth",
            DeltaType::Added,
            valid_delta,
            0,
        )
        .await
        .unwrap();

        let app = crate::create_changes_router().with_state(db_state);

        // Validate change
        let request = Request::builder()
            .method("GET")
            .uri(format!(
                "/{}/changes/{}/validate?strict=true",
                project_id, change_id
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check response body
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["valid"], true);
        assert_eq!(json["data"]["errors"].as_array().unwrap().len(), 0);
        assert_eq!(json["data"]["deltasValidated"], 1);
    }

    #[tokio::test]
    async fn test_validate_change_with_invalid_delta() {
        let db_state = setup_test_db().await;
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        // Add an invalid delta (missing WHEN/THEN format)
        let invalid_delta = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
WHEN valid credentials are provided
THEN a JWT token is returned
"#;

        create_spec_delta(
            &db_state.pool,
            &change_id,
            None,
            "user-auth",
            DeltaType::Added,
            invalid_delta,
            0,
        )
        .await
        .unwrap();

        let app = crate::create_changes_router().with_state(db_state);

        // Validate change
        let request = Request::builder()
            .method("GET")
            .uri(format!(
                "/{}/changes/{}/validate?strict=false",
                project_id, change_id
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check response body
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["valid"], false);
        assert!(json["data"]["errors"].as_array().unwrap().len() > 0);
        assert_eq!(json["data"]["deltasValidated"], 1);
    }

    #[tokio::test]
    async fn test_validate_change_not_found() {
        let db_state = setup_test_db().await;
        let project_id = "test-project";

        // Create project only (no change)
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, description, created_at, updated_at)
             VALUES (?, 'Test Project', '/tmp/test', 'Test', datetime('now'), datetime('now'))",
        )
        .bind(project_id)
        .execute(&db_state.pool)
        .await
        .unwrap();

        let app = crate::create_changes_router().with_state(db_state);

        // Try to validate non-existent change
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}/changes/nonexistent/validate", project_id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Non-existent change should return 404 Not Found
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Check response contains error message
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], false);
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains("Spec change not found"));
    }

    #[tokio::test]
    async fn test_archive_change_success() {
        let db_state = setup_test_db().await;
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        // Add a valid delta
        let valid_delta = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        create_spec_delta(
            &db_state.pool,
            &change_id,
            None,
            "user-auth",
            DeltaType::Added,
            valid_delta,
            0,
        )
        .await
        .unwrap();

        let app = crate::create_changes_router().with_state(db_state.clone());

        // Archive change with apply_specs=true
        let request_body = serde_json::json!({
            "applySpecs": true
        });

        let request = Request::builder()
            .method("POST")
            .uri(format!("/{}/changes/{}/archive", project_id, change_id))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check response body
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["archived"], true);
        assert_eq!(json["data"]["specsApplied"], true);

        // Verify change is archived in database
        let change = openspec_db::get_spec_change(&db_state.pool, &change_id)
            .await
            .unwrap();
        assert_eq!(change.status, ChangeStatus::Archived);

        // Verify capability was created
        let caps: Vec<String> =
            sqlx::query_scalar("SELECT name FROM spec_capabilities WHERE project_id = ?")
                .bind(project_id)
                .fetch_all(&db_state.pool)
                .await
                .unwrap();

        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "user-auth");
    }

    #[tokio::test]
    async fn test_archive_change_without_applying_specs() {
        let db_state = setup_test_db().await;
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        // Add a valid delta
        let valid_delta = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        create_spec_delta(
            &db_state.pool,
            &change_id,
            None,
            "user-auth",
            DeltaType::Added,
            valid_delta,
            0,
        )
        .await
        .unwrap();

        let app = crate::create_changes_router().with_state(db_state.clone());

        // Archive change with apply_specs=false
        let request_body = serde_json::json!({
            "applySpecs": false
        });

        let request = Request::builder()
            .method("POST")
            .uri(format!("/{}/changes/{}/archive", project_id, change_id))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check response body
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"]["archived"], true);
        assert_eq!(json["data"]["specsApplied"], false);

        // Verify change is archived in database
        let change = openspec_db::get_spec_change(&db_state.pool, &change_id)
            .await
            .unwrap();
        assert_eq!(change.status, ChangeStatus::Archived);

        // Verify NO capability was created
        let caps: Vec<String> =
            sqlx::query_scalar("SELECT name FROM spec_capabilities WHERE project_id = ?")
                .bind(project_id)
                .fetch_all(&db_state.pool)
                .await
                .unwrap();

        assert_eq!(caps.len(), 0);
    }

    #[tokio::test]
    async fn test_archive_already_archived_change() {
        let db_state = setup_test_db().await;
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        // Add a valid delta
        let valid_delta = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        create_spec_delta(
            &db_state.pool,
            &change_id,
            None,
            "user-auth",
            DeltaType::Added,
            valid_delta,
            0,
        )
        .await
        .unwrap();

        // Archive it once
        openspec::archive::archive_change(&db_state.pool, &change_id, false)
            .await
            .unwrap();

        let app = crate::create_changes_router().with_state(db_state);

        // Try to archive again
        let request_body = serde_json::json!({
            "applySpecs": false
        });

        let request = Request::builder()
            .method("POST")
            .uri(format!("/{}/changes/{}/archive", project_id, change_id))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Check error message
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], false);
        assert!(json["error"].as_str().unwrap().contains("already archived"));
    }

    #[tokio::test]
    async fn test_archive_change_with_invalid_delta() {
        let db_state = setup_test_db().await;
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        // Add an invalid delta (missing proper formatting)
        let invalid_delta = r#"## ADDED Requirements

### Requirement: User Authentication
The system should provide authentication.

No scenario here!
"#;

        create_spec_delta(
            &db_state.pool,
            &change_id,
            None,
            "user-auth",
            DeltaType::Added,
            invalid_delta,
            0,
        )
        .await
        .unwrap();

        let app = crate::create_changes_router().with_state(db_state);

        // Try to archive change with invalid delta
        let request_body = serde_json::json!({
            "applySpecs": true
        });

        let request = Request::builder()
            .method("POST")
            .uri(format!("/{}/changes/{}/archive", project_id, change_id))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Check error message indicates validation failure
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], false);
        // Error handler wraps ArchiveError with "Failed to archive change:" prefix
        // Archive module ValidationFailed error has format "Validation failed: {details}"
        let error_msg = json["error"].as_str().unwrap();
        assert!(
            error_msg.starts_with("Failed to archive change: Validation failed:"),
            "Expected consistent error format, got: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_validate_change_project_not_found() {
        let db_state = setup_test_db().await;

        // Create project and test data
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        let app = crate::create_changes_router().with_state(db_state);

        // Try to validate change for non-existent project
        let request = Request::builder()
            .method("GET")
            .uri(format!(
                "/nonexistent-project/changes/{}/validate",
                change_id
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should return 404 Not Found for non-existent project
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Check error message
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], false);
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains("Project not found: nonexistent-project"));
    }

    #[tokio::test]
    async fn test_archive_change_project_not_found() {
        let db_state = setup_test_db().await;

        // Create project and test data
        let project_id = "test-project";
        let (_prd_id, change_id) = setup_test_data(&db_state.pool, project_id).await;

        let app = crate::create_changes_router().with_state(db_state);

        // Try to archive change for non-existent project
        let request = Request::builder()
            .method("POST")
            .uri(format!(
                "/nonexistent-project/changes/{}/archive",
                change_id
            ))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"applySpecs":true}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should return 404 Not Found for non-existent project
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Check error message
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], false);
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains("Project not found: nonexistent-project"));
    }
}
