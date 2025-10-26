// ABOUTME: Tag type definitions
// ABOUTME: Structures for tags used to organize tasks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCreateInput {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagUpdateInput {
    pub name: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
}
