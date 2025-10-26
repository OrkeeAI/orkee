// ABOUTME: User type definitions
// ABOUTME: Structures for user accounts, settings, and preferences

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub default_agent_id: Option<String>,
    pub theme: Option<String>,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub xai_api_key: Option<String>,
    pub ai_gateway_enabled: bool,
    pub ai_gateway_url: Option<String>,
    pub ai_gateway_key: Option<String>,
    pub preferences: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdateInput {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub xai_api_key: Option<String>,
    pub ai_gateway_enabled: Option<bool>,
    pub ai_gateway_url: Option<String>,
    pub ai_gateway_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub default_agent_id: Option<String>,
    pub theme: Option<String>,
    pub has_openai_api_key: bool,
    pub has_anthropic_api_key: bool,
    pub has_google_api_key: bool,
    pub has_xai_api_key: bool,
    pub ai_gateway_enabled: bool,
    pub has_ai_gateway_key: bool,
    pub ai_gateway_url: Option<String>,
    pub preferences: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for MaskedUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            default_agent_id: user.default_agent_id,
            theme: user.theme,
            has_openai_api_key: user.openai_api_key.is_some(),
            has_anthropic_api_key: user.anthropic_api_key.is_some(),
            has_google_api_key: user.google_api_key.is_some(),
            has_xai_api_key: user.xai_api_key.is_some(),
            ai_gateway_enabled: user.ai_gateway_enabled,
            has_ai_gateway_key: user.ai_gateway_key.is_some(),
            ai_gateway_url: user.ai_gateway_url,
            preferences: user.preferences,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        }
    }
}
