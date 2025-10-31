// ABOUTME: PRD (Product Requirements Document) type definitions
// ABOUTME: Structures for managing product requirements documents and their lifecycle

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PRDStatus {
    Draft,
    Approved,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PRDSource {
    Manual,
    Generated,
    Synced,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PRD {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub content_markdown: String,
    pub version: i32,
    pub status: PRDStatus,
    pub source: PRDSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
}
