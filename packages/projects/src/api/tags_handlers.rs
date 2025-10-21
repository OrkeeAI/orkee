// ABOUTME: HTTP request handlers for tag operations
// ABOUTME: Handles CRUD operations for tags with database integration

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error};
use crate::db::DbState;
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::tags::{TagCreateInput, TagUpdateInput};

#[derive(Deserialize)]
pub struct ListTagsQuery {
    #[serde(default)]
    pub include_archived: bool,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// List all tags
pub async fn list_tags(
    State(db): State<DbState>,
    Query(params): Query<ListTagsQuery>,
) -> impl IntoResponse {
    info!(
        "Listing tags (include_archived: {}, page: {})",
        params.include_archived,
        params.pagination.page()
    );

    let result = db
        .tag_storage
        .list_tags_paginated(params.include_archived, Some(params.pagination.limit()), Some(params.pagination.offset()))
        .await
        .map(|(tags, total)| PaginatedResponse::new(tags, &params.pagination, total));

    ok_or_internal_error(result, "Failed to list tags")
}

/// Get a single tag by ID
pub async fn get_tag(State(db): State<DbState>, Path(tag_id): Path<String>) -> impl IntoResponse {
    info!("Getting tag: {}", tag_id);

    let result = db.tag_storage.get_tag(&tag_id).await;
    ok_or_internal_error(result, "Failed to get tag")
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

    let result = db.tag_storage.create_tag(input).await;
    created_or_internal_error(result, "Failed to create tag")
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

    let result = db.tag_storage.update_tag(&tag_id, input).await;
    ok_or_internal_error(result, "Failed to update tag")
}

/// Archive a tag
pub async fn archive_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
) -> impl IntoResponse {
    info!("Archiving tag: {}", tag_id);

    let result = db.tag_storage.archive_tag(&tag_id).await;
    ok_or_internal_error(result, "Failed to archive tag")
}

/// Unarchive a tag
pub async fn unarchive_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
) -> impl IntoResponse {
    info!("Unarchiving tag: {}", tag_id);

    let result = db.tag_storage.unarchive_tag(&tag_id).await;
    ok_or_internal_error(result, "Failed to unarchive tag")
}

/// Delete a tag (only if unused)
pub async fn delete_tag(
    State(db): State<DbState>,
    Path(tag_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting tag: {}", tag_id);

    let result = db.tag_storage.delete_tag(&tag_id).await.map(|_| "Tag deleted successfully");
    ok_or_internal_error(result, "Failed to delete tag")
}
