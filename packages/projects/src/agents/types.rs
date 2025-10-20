// ABOUTME: Agent type definitions
// ABOUTME: Structures for AI agents, human collaborators, and user-agent configurations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum AgentType {
    AI,
    Human,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: AgentType,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub frameworks: Option<String>,
    pub max_context_tokens: Option<i64>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_web_search: bool,
    pub api_endpoint: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i64>,
    pub system_prompt: Option<String>,
    pub cost_per_1k_input_tokens: Option<f64>,
    pub cost_per_1k_output_tokens: Option<f64>,
    pub is_available: bool,
    pub requires_api_key: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgent {
    pub id: String,
    pub user_id: String,
    pub agent_id: String,
    pub agent: Option<Agent>,
    pub is_active: bool,
    pub is_favorite: bool,
    pub custom_name: Option<String>,
    pub custom_system_prompt: Option<String>,
    pub custom_temperature: Option<f64>,
    pub custom_max_tokens: Option<i64>,
    pub tasks_assigned: i64,
    pub tasks_completed: i64,
    pub total_tokens_used: i64,
    pub total_cost_cents: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub preferences: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
