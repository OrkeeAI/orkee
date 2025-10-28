// ABOUTME: HTTP request handlers for PRD output template management
// ABOUTME: Handles CRUD operations for markdown templates used to format PRD content

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

use super::response::{bad_request, created_or_internal_error, ok_or_internal_error, ok_or_not_found};
use orkee_projects::DbState;

/// PRD output template structure
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PRDTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for creating a template
#[derive(Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    #[serde(default)]
    pub is_default: bool,
}

/// Request body for updating a template
#[derive(Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub is_default: Option<bool>,
}

/// List all PRD output templates
pub async fn list_templates(State(db): State<DbState>) -> impl IntoResponse {
    info!("Listing all PRD output templates");

    let result = fetch_all_templates(&db.pool).await;
    ok_or_internal_error(result, "Failed to list templates")
}

/// Get a specific PRD output template by ID
pub async fn get_template(
    State(db): State<DbState>,
    Path(template_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting template: {}", template_id);

    let result = fetch_template_by_id(&db.pool, &template_id).await;
    ok_or_not_found(result, "Template not found")
}

/// Create a new PRD output template
pub async fn create_template(
    State(db): State<DbState>,
    Json(request): Json<CreateTemplateRequest>,
) -> impl IntoResponse {
    info!("Creating new template: {}", request.name);

    // Validate input
    if request.name.trim().is_empty() {
        return bad_request("Name cannot be empty", "Invalid template name");
    }

    if request.content.trim().is_empty() {
        return bad_request("Content cannot be empty", "Invalid template content");
    }

    // Generate ID
    let template_id = format!("template-{}", chrono::Utc::now().timestamp_millis());

    // If this is being set as default, unset other defaults first
    if request.is_default {
        if let Err(e) = unset_all_defaults(&db.pool).await {
            return bad_request(e, "Failed to update default template");
        }
    }

    let result = insert_template(
        &db.pool,
        &template_id,
        &request.name,
        request.description.as_deref(),
        &request.content,
        request.is_default,
    )
    .await;

    created_or_internal_error(result, "Failed to create template")
}

/// Update an existing PRD output template
pub async fn update_template(
    State(db): State<DbState>,
    Path(template_id): Path<String>,
    Json(request): Json<UpdateTemplateRequest>,
) -> impl IntoResponse {
    info!("Updating template: {}", template_id);

    // Validate that template exists
    if let Err(e) = fetch_template_by_id(&db.pool, &template_id).await {
        return ok_or_not_found::<PRDTemplate, sqlx::Error>(Err(e), "Template not found");
    }

    // Validate input if provided
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return bad_request("Name cannot be empty", "Invalid template name");
        }
    }

    if let Some(ref content) = request.content {
        if content.trim().is_empty() {
            return bad_request("Content cannot be empty", "Invalid template content");
        }
    }

    // If setting as default, unset other defaults first
    if request.is_default == Some(true) {
        if let Err(e) = unset_all_defaults(&db.pool).await {
            return bad_request(e, "Failed to update default template");
        }
    }

    let result = update_template_fields(
        &db.pool,
        &template_id,
        request.name.as_deref(),
        request.description.as_ref().map(|s| Some(s.as_str())),
        request.content.as_deref(),
        request.is_default,
    )
    .await;

    ok_or_internal_error(result, "Failed to update template")
}

/// Delete a PRD output template
pub async fn delete_template(
    State(db): State<DbState>,
    Path(template_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting template: {}", template_id);

    // Check if it's the default template
    match fetch_template_by_id(&db.pool, &template_id).await {
        Ok(template) => {
            if template.is_default {
                return bad_request(
                    "Cannot delete default template",
                    "Default template cannot be deleted",
                );
            }
        }
        Err(_) => {
            return ok_or_not_found::<PRDTemplate, String>(
                Err("Template not found".to_string()),
                "Template not found",
            );
        }
    }

    let result = delete_template_by_id(&db.pool, &template_id).await;
    ok_or_internal_error(result, "Failed to delete template")
}

// Database operations

async fn fetch_all_templates(pool: &SqlitePool) -> Result<Vec<PRDTemplate>, sqlx::Error> {
    sqlx::query_as::<_, PRDTemplate>(
        r#"
        SELECT
            id,
            name,
            description,
            content,
            CASE WHEN is_default = 1 THEN 1 ELSE 0 END as is_default,
            created_at,
            updated_at
        FROM prd_output_templates
        ORDER BY is_default DESC, created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

async fn fetch_template_by_id(
    pool: &SqlitePool,
    template_id: &str,
) -> Result<PRDTemplate, sqlx::Error> {
    sqlx::query_as::<_, PRDTemplate>(
        r#"
        SELECT
            id,
            name,
            description,
            content,
            CASE WHEN is_default = 1 THEN 1 ELSE 0 END as is_default,
            created_at,
            updated_at
        FROM prd_output_templates
        WHERE id = ?
        "#,
    )
    .bind(template_id)
    .fetch_one(pool)
    .await
}

async fn insert_template(
    pool: &SqlitePool,
    template_id: &str,
    name: &str,
    description: Option<&str>,
    content: &str,
    is_default: bool,
) -> Result<PRDTemplate, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO prd_output_templates (id, name, description, content, is_default)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(template_id)
    .bind(name)
    .bind(description)
    .bind(content)
    .bind(if is_default { 1 } else { 0 })
    .execute(pool)
    .await?;

    fetch_template_by_id(pool, template_id).await
}

async fn update_template_fields(
    pool: &SqlitePool,
    template_id: &str,
    name: Option<&str>,
    description: Option<Option<&str>>,
    content: Option<&str>,
    is_default: Option<bool>,
) -> Result<PRDTemplate, sqlx::Error> {
    // Check if any fields need to be updated
    if name.is_none() && description.is_none() && content.is_none() && is_default.is_none() {
        return fetch_template_by_id(pool, template_id).await;
    }

    // Individual field updates
    if let Some(n) = name {
        sqlx::query("UPDATE prd_output_templates SET name = ? WHERE id = ?")
            .bind(n)
            .bind(template_id)
            .execute(pool)
            .await?;
    }

    if let Some(d) = description {
        sqlx::query("UPDATE prd_output_templates SET description = ? WHERE id = ?")
            .bind(d)
            .bind(template_id)
            .execute(pool)
            .await?;
    }

    if let Some(c) = content {
        sqlx::query("UPDATE prd_output_templates SET content = ? WHERE id = ?")
            .bind(c)
            .bind(template_id)
            .execute(pool)
            .await?;
    }

    if let Some(def) = is_default {
        sqlx::query("UPDATE prd_output_templates SET is_default = ? WHERE id = ?")
            .bind(if def { 1 } else { 0 })
            .bind(template_id)
            .execute(pool)
            .await?;
    }

    fetch_template_by_id(pool, template_id).await
}

async fn delete_template_by_id(pool: &SqlitePool, template_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM prd_output_templates WHERE id = ?")
        .bind(template_id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn unset_all_defaults(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query("UPDATE prd_output_templates SET is_default = 0")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
