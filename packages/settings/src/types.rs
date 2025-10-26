// ABOUTME: Type definitions for system settings
// ABOUTME: Structures for runtime configuration management

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSetting {
    pub key: String,
    pub value: String,
    pub category: String,
    pub description: Option<String>,
    pub data_type: String,
    pub is_secret: bool,
    pub requires_restart: bool,
    pub is_env_only: bool,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingUpdate {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkSettingUpdate {
    pub settings: Vec<SettingUpdateItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingUpdateItem {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub settings: Vec<SystemSetting>,
    pub requires_restart: bool,
}

/// Category enum for settings organization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingCategory {
    Server,
    Security,
    Tls,
    RateLimiting,
    Cloud,
    Telemetry,
    Advanced,
}

impl SettingCategory {
    pub fn as_str(&self) -> &str {
        match self {
            SettingCategory::Server => "server",
            SettingCategory::Security => "security",
            SettingCategory::Tls => "tls",
            SettingCategory::RateLimiting => "rate_limiting",
            SettingCategory::Cloud => "cloud",
            SettingCategory::Telemetry => "telemetry",
            SettingCategory::Advanced => "advanced",
        }
    }
}
