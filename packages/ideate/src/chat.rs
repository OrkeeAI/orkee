// ABOUTME: Chat mode types and data structures for chat-based PRD discovery
// ABOUTME: Defines chat messages, insights, quality metrics, and discovery questions

use serde::{Deserialize, Serialize};

/// Role of the message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Type of message in the chat flow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Discovery,
    Refinement,
    Validation,
    General,
}

/// Discovery status of the PRD chat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryStatus {
    Draft,
    Brainstorming,
    Refining,
    Validating,
    Finalized,
}

/// Type of insight extracted from chat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum InsightType {
    Requirement,
    Constraint,
    Risk,
    Assumption,
    Decision,
}

/// Category of discovery question
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum QuestionCategory {
    Problem,
    Users,
    Features,
    Technical,
    Risks,
    Constraints,
    Success,
}

/// A message in the chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub prd_id: Option<String>,
    pub message_order: i32,
    pub role: MessageRole,
    pub content: String,
    pub message_type: Option<MessageType>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

/// Input for creating a new message
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageInput {
    pub content: String,
    pub message_type: Option<MessageType>,
    pub role: Option<MessageRole>,
}

/// Discovery question for guiding chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryQuestion {
    pub id: String,
    pub category: QuestionCategory,
    pub question_text: String,
    pub follow_up_prompts: Option<Vec<String>>,
    pub context_keywords: Option<Vec<String>>,
    pub priority: i32,
    pub is_required: bool,
    pub display_order: i32,
    pub is_active: bool,
    pub created_at: String,
}

/// Insight extracted from chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInsight {
    pub id: String,
    pub session_id: String,
    pub insight_type: InsightType,
    pub insight_text: String,
    pub confidence_score: Option<f64>,
    pub source_message_ids: Option<Vec<String>>,
    pub applied_to_prd: bool,
    pub created_at: String,
}

/// Coverage tracking for different topic areas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCoverage {
    pub problem: bool,
    pub users: bool,
    pub features: bool,
    pub technical: bool,
    pub risks: bool,
    pub constraints: bool,
    pub success: bool,
}

/// Quality metrics for the chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub quality_score: i32,
    pub missing_areas: Vec<String>,
    pub coverage: TopicCoverage,
    pub is_ready_for_prd: bool,
}

/// Input for generating PRD from chat
#[derive(Debug, Clone, Deserialize)]
pub struct GeneratePRDFromChatInput {
    pub title: String,
}

/// Result of PRD generation from chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePRDFromChatResult {
    pub prd_id: String,
    pub content_markdown: String,
    pub quality_score: i32,
}

/// Validation result for chat readiness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub missing_required: Vec<String>,
    pub warnings: Vec<String>,
}

/// Input for creating a new insight
#[derive(Debug, Clone, Deserialize)]
pub struct CreateInsightInput {
    pub insight_type: InsightType,
    pub insight_text: String,
    pub confidence_score: Option<f64>,
    pub source_message_ids: Option<Vec<String>>,
}
