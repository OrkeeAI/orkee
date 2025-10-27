// ABOUTME: Type definitions for expert roundtable discussions
// ABOUTME: Defines expert personas, roundtable sessions, messages, and insights for AI-powered discussions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// ENUMS
// ============================================================================

/// Roundtable discussion status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RoundtableStatus {
    /// Initial setup phase (selecting experts)
    Setup,
    /// Discussion in progress
    Discussing,
    /// Discussion completed successfully
    Completed,
    /// Discussion cancelled by user
    Cancelled,
}

/// Message role in roundtable discussion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from an expert AI persona
    Expert,
    /// Message from the user (interjection)
    User,
    /// Message from the moderator AI
    Moderator,
    /// System message (e.g., "Discussion started")
    System,
}

/// Insight priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum InsightPriority {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// EXPERT PERSONA TYPES
// ============================================================================

/// Expert persona definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertPersona {
    pub id: String,
    pub name: String,
    pub role: String,
    pub expertise: Vec<String>,
    pub system_prompt: String,
    pub bio: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a custom expert persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExpertPersonaInput {
    pub name: String,
    pub role: String,
    pub expertise: Vec<String>,
    pub system_prompt: String,
    pub bio: Option<String>,
}

/// Expert suggestion from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertSuggestion {
    pub id: String,
    pub session_id: String,
    pub expert_name: String,
    pub role: String,
    pub expertise_area: String,
    pub reason: String,
    pub relevance_score: Option<f32>,
    pub created_at: DateTime<Utc>,
}

/// Request to suggest experts for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestExpertsRequest {
    pub project_description: String,
    pub existing_content: Option<String>,
    pub num_experts: Option<i32>,
}

/// Response containing suggested experts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestExpertsResponse {
    pub suggestions: Vec<ExpertSuggestion>,
}

// ============================================================================
// ROUNDTABLE SESSION TYPES
// ============================================================================

/// Roundtable discussion session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableSession {
    pub id: String,
    pub session_id: String,
    pub status: RoundtableStatus,
    pub topic: String,
    pub num_experts: i32,
    pub moderator_persona: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Input for starting a roundtable discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRoundtableRequest {
    pub topic: String,
    pub expert_ids: Vec<String>,
    pub duration_minutes: Option<i32>,
}

/// Participant in a roundtable (join table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableParticipant {
    pub id: String,
    pub roundtable_id: String,
    pub expert_id: String,
    pub joined_at: DateTime<Utc>,
}

/// Roundtable with full participant details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableWithParticipants {
    pub session: RoundtableSession,
    pub participants: Vec<ExpertPersona>,
}

// ============================================================================
// MESSAGE TYPES
// ============================================================================

/// Message in a roundtable discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableMessage {
    pub id: String,
    pub roundtable_id: String,
    pub message_order: i32,
    pub role: MessageRole,
    pub expert_id: Option<String>,
    pub expert_name: Option<String>,
    pub content: String,
    pub metadata: Option<String>, // JSON string for extensibility
    pub created_at: DateTime<Utc>,
}

/// Input for user interjection during discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInterjectionInput {
    pub message: String,
}

/// Response after sending user interjection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInterjectionResponse {
    pub message_id: String,
    pub acknowledged: bool,
}

/// Metadata for message (parsed from JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub response_time_ms: Option<u64>,
    pub token_count: Option<u32>,
    pub interjection_acknowledged: Option<bool>,
}

// ============================================================================
// INSIGHT TYPES
// ============================================================================

/// Extracted insight from roundtable discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableInsight {
    pub id: String,
    pub roundtable_id: String,
    pub insight_text: String,
    pub category: String,
    pub priority: InsightPriority,
    pub source_experts: Vec<String>,
    pub source_message_ids: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
}

/// Input for extracting insights from discussion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractInsightsRequest {
    pub roundtable_id: String,
    pub categories: Option<Vec<String>>,
}

/// Response containing extracted insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractInsightsResponse {
    pub insights: Vec<RoundtableInsight>,
    pub summary: Option<String>,
}

/// Grouped insights by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsByCategory {
    pub category: String,
    pub insights: Vec<RoundtableInsight>,
}

// ============================================================================
// SSE STREAM TYPES
// ============================================================================

/// Event types for Server-Sent Events stream
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RoundtableEvent {
    /// Initial connection established
    Connected {
        roundtable_id: String,
    },
    /// Discussion started
    Started {
        roundtable_id: String,
        participants: Vec<ExpertPersona>,
    },
    /// New message in discussion
    Message {
        message: RoundtableMessage,
    },
    /// Expert is typing (optional feature)
    Typing {
        expert_name: String,
    },
    /// User interjection acknowledged
    InterjectionAcknowledged {
        message_id: String,
    },
    /// Discussion completed
    Completed {
        roundtable_id: String,
        message_count: i32,
    },
    /// Error occurred
    Error {
        error: String,
    },
    /// Heartbeat to keep connection alive
    Heartbeat,
}

// ============================================================================
// ROUNDTABLE STATISTICS
// ============================================================================

/// Statistics for a roundtable session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableStatistics {
    pub roundtable_id: String,
    pub message_count: i32,
    pub expert_count: i32,
    pub user_interjection_count: i32,
    pub insight_count: i32,
    pub duration_seconds: Option<i64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

impl ExpertPersona {
    /// Check if this is a system default expert
    pub fn is_system_default(&self) -> bool {
        self.is_default
    }

    /// Get primary expertise area
    pub fn primary_expertise(&self) -> Option<&str> {
        self.expertise.first().map(|s| s.as_str())
    }
}

impl RoundtableSession {
    /// Check if roundtable is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, RoundtableStatus::Discussing)
    }

    /// Check if roundtable can be started
    pub fn can_start(&self) -> bool {
        matches!(self.status, RoundtableStatus::Setup)
    }

    /// Check if roundtable is finished
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            RoundtableStatus::Completed | RoundtableStatus::Cancelled
        )
    }

    /// Calculate duration in seconds if completed
    pub fn duration_seconds(&self) -> Option<i64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some((end - start).num_seconds()),
            _ => None,
        }
    }
}

impl RoundtableMessage {
    /// Check if message is from an expert
    pub fn is_expert_message(&self) -> bool {
        matches!(self.role, MessageRole::Expert)
    }

    /// Check if message is from user
    pub fn is_user_message(&self) -> bool {
        matches!(self.role, MessageRole::User)
    }

    /// Check if message is from moderator
    pub fn is_moderator_message(&self) -> bool {
        matches!(self.role, MessageRole::Moderator)
    }

    /// Parse metadata from JSON string
    pub fn parse_metadata(&self) -> Option<MessageMetadata> {
        self.metadata
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
    }
}

impl RoundtableInsight {
    /// Check if insight is high priority
    pub fn is_high_priority(&self) -> bool {
        matches!(
            self.priority,
            InsightPriority::High | InsightPriority::Critical
        )
    }

    /// Get source expert count
    pub fn source_expert_count(&self) -> usize {
        self.source_experts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtable_status_transitions() {
        let status = RoundtableStatus::Setup;
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            r#""setup""#
        );

        let status = RoundtableStatus::Discussing;
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            r#""discussing""#
        );
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::Expert;
        assert_eq!(serde_json::to_string(&role).unwrap(), r#""expert""#);

        let role = MessageRole::User;
        assert_eq!(serde_json::to_string(&role).unwrap(), r#""user""#);
    }

    #[test]
    fn test_insight_priority_ordering() {
        let low = InsightPriority::Low;
        let high = InsightPriority::High;

        assert!(!matches!(low, InsightPriority::High | InsightPriority::Critical));
        assert!(matches!(
            high,
            InsightPriority::High | InsightPriority::Critical
        ));
    }
}
