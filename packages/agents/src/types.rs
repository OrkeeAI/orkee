// ABOUTME: User-agent configuration types
// ABOUTME: Structures for user-specific agent preferences and usage statistics

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use models::Agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgent {
    pub id: String,
    pub user_id: String,
    pub agent_id: String,
    /// Agent details loaded from models::REGISTRY
    pub agent: Option<Agent>,
    pub preferred_model_id: Option<String>,
    pub is_active: bool,
    pub custom_name: Option<String>,
    pub custom_system_prompt: Option<String>,
    pub custom_temperature: Option<f64>,
    pub custom_max_tokens: Option<i64>,
    pub tasks_assigned: i64,
    pub tasks_completed: i64,
    pub total_tokens_used: i64,
    pub total_cost_cents: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub custom_settings: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
