// ABOUTME: Type definitions for AI models and coding agents
// ABOUTME: Structures that mirror the JSON configuration files for models and agents

use serde::{Deserialize, Serialize};

/// AI model configuration from config/models.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub model_identifier: String,
    pub description: String,
    pub max_context_tokens: u64,
    pub capabilities: ModelCapabilities,
    pub pricing: ModelPricing,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub tools: bool,
    pub vision: bool,
    pub web_search: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Price per 1 million input tokens (USD)
    pub input_per_million_tokens: f64,
    /// Price per 1 million output tokens (USD)
    pub output_per_million_tokens: f64,
}

/// CLI coding agent configuration from config/agents.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub description: String,
    pub avatar_url: Option<String>,
    pub required_providers: Vec<String>,
    pub supported_models: Vec<AgentModelRef>,
    pub default_config: AgentConfig,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelRef {
    pub model_id: String,
    pub is_default: bool,
    pub is_recommended: bool,
    pub display_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub temperature: f64,
    pub max_tokens: u32,
    pub system_prompt: Option<String>,
}

/// Container for models JSON file
#[derive(Debug, Deserialize)]
pub struct ModelsConfig {
    pub version: String,
    pub models: Vec<Model>,
}

/// Container for agents JSON file
#[derive(Debug, Deserialize)]
pub struct AgentsConfig {
    pub version: String,
    pub agents: Vec<Agent>,
}
