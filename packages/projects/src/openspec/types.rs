// ABOUTME: OpenSpec type definitions for spec-driven development
// ABOUTME: Structures for PRDs, capabilities, requirements, scenarios, and changes

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CapabilityStatus {
    Active,
    Deprecated,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpecCapability {
    pub id: String,
    pub project_id: String,
    pub prd_id: Option<String>,
    pub name: String,
    pub purpose_markdown: Option<String>,
    pub spec_markdown: String,
    pub design_markdown: Option<String>,
    pub requirement_count: i32,
    pub version: i32,
    pub status: CapabilityStatus,
    pub change_id: Option<String>,
    pub is_openspec_compliant: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpecRequirement {
    pub id: String,
    pub capability_id: String,
    pub name: String,
    pub content_markdown: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpecScenario {
    pub id: String,
    pub requirement_id: String,
    pub name: String,
    pub when_clause: String,
    pub then_clause: String,
    #[sqlx(json)]
    pub and_clauses: Option<Vec<String>>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    Draft,
    Review,
    Approved,
    Implementing,
    Completed,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpecChange {
    pub id: String,
    pub project_id: String,
    pub prd_id: Option<String>,
    pub proposal_markdown: String,
    pub tasks_markdown: String,
    pub design_markdown: Option<String>,
    pub status: ChangeStatus,
    pub verb_prefix: Option<String>,
    pub change_number: Option<i32>,
    pub validation_status: ValidationStatus,
    pub validation_errors: Option<String>,
    pub tasks_completion_percentage: Option<i32>,
    pub tasks_parsed_at: Option<DateTime<Utc>>,
    pub tasks_total_count: Option<i32>,
    pub tasks_completed_count: Option<i32>,
    pub created_by: String,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DeltaType {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpecDelta {
    pub id: String,
    pub change_id: String,
    pub capability_id: Option<String>,
    pub capability_name: String,
    pub delta_type: DeltaType,
    pub delta_markdown: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpecChangeTask {
    pub id: String,
    pub change_id: String,
    pub task_number: String,
    pub task_text: String,
    pub is_completed: bool,
    pub completed_by: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub display_order: i32,
    pub parent_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Parsed structures from markdown

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequirement {
    pub name: String,
    pub description: String,
    pub scenarios: Vec<ParsedScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedScenario {
    pub name: String,
    pub when: String,
    pub then: String,
    pub and: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCapability {
    pub name: String,
    pub purpose: String,
    pub requirements: Vec<ParsedRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpec {
    pub capabilities: Vec<ParsedCapability>,
    pub raw_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SpecMaterialization {
    pub id: String,
    pub project_id: String,
    pub path: String,
    pub materialized_at: DateTime<Utc>,
    pub sha256_hash: String,
}
