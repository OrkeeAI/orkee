// ABOUTME: HTTP request handlers for tag operations
// ABOUTME: Handles CRUD operations for tags with database integration

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use super::response::ApiResponse;
use crate::db::DbState;
use crate::tags::{TagCreateInput, TagUpdateInput};

#[derive(Deserialize)]
pub struct ListTagsQuery {
    #[serde(default)]
    pub include_archived: bool,
}

/// List all tags
pub async fn list_tags(
    State(db): State<DbState>,
    Query(params): Query<ListTagsQuery>,
) -> impl IntoResponse {
    info!("Listing tags (include_archived: {})", params.include_archived);

    match db.tag_storage.list_tags(params.include_archived).await {
        Ok(tags) => (StatusCode::OK, ResponseJson(ApiResponse::success(tags))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get a single tag by ID
pub async fn get_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting tag: {}", tag_id);

    match db.tag_storage.get_tag(&tag_id).await {
        Ok(tag) => (StatusCode::OK, ResponseJson(ApiResponse::success(tag))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Request body for creating a tag
#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

/// Create a new tag
pub async fn create_tag(
    State(db): State<DbState>,
    Json(request): Json<CreateTagRequest>,
) -> impl IntoResponse {
    info!("Creating tag: {}", request.name);

    let input = TagCreateInput {
        name: request.name,
        color: request.color,
        description: request.description,
    };

    match db.tag_storage.create_tag(input).await {
        Ok(tag) => (StatusCode::CREATED, ResponseJson(ApiResponse::success(tag))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Request body for updating a tag
#[derive(Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "archivedAt")]
    pub archived_at: Option<String>,
}

/// Update a tag
pub async fn update_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
    Json(request): Json<UpdateTagRequest>,
) -> impl IntoResponse {
    info!("Updating tag: {}", tag_id);

    let archived_at = request
        .archived_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let input = TagUpdateInput {
        name: request.name,
        color: request.color,
        description: request.description,
        archived_at,
    };

    match db.tag_storage.update_tag(&tag_id, input).await {
        Ok(tag) => (StatusCode::OK, ResponseJson(ApiResponse::success(tag))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Archive a tag
pub async fn archive_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
) -> impl IntoResponse {
    info!("Archiving tag: {}", tag_id);

    match db.tag_storage.archive_tag(&tag_id).await {
        Ok(tag) => (StatusCode::OK, ResponseJson(ApiResponse::success(tag))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Unarchive a tag
pub async fn unarchive_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
) -> impl IntoResponse {
    info!("Unarchiving tag: {}", tag_id);

    match db.tag_storage.unarchive_tag(&tag_id).await {
        Ok(tag) => (StatusCode::OK, ResponseJson(ApiResponse::success(tag))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Delete a tag (only if unused)
pub async fn delete_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting tag: {}", tag_id);

    match db.tag_storage.delete_tag(&tag_id).await {
        Ok(_) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success("Tag deleted successfully")),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}
