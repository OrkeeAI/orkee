// ABOUTME: Agent registry for loading and managing AI agent configurations
// ABOUTME: Loads agent definitions from config/agents.json at runtime

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Failed to load agents config: {0}")]
    LoadError(String),
    #[error("Agent not found: {0}")]
    NotFound(String),
    #[error("Invalid agent configuration: {0}")]
    InvalidConfig(String),
}

type Result<T> = std::result::Result<T, AgentError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModel {
    pub model_id: String,
    pub is_default: bool,
    pub is_recommended: bool,
    pub display_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefaultConfig {
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub description: String,
    pub avatar_url: Option<String>,
    pub required_providers: Vec<String>,
    pub supported_models: Vec<AgentModel>,
    pub default_config: AgentDefaultConfig,
    pub is_available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentsConfig {
    version: String,
    agents: Vec<Agent>,
}

pub struct AgentRegistry {
    agents: HashMap<String, Agent>,
}

impl AgentRegistry {
    /// Create a new AgentRegistry by loading agents from config file
    pub fn new() -> Result<Self> {
        let config_json = include_str!("../config/agents.json");
        let config: AgentsConfig = serde_json::from_str(config_json)
            .map_err(|e| AgentError::LoadError(e.to_string()))?;

        let mut agents = HashMap::new();
        for agent in config.agents {
            agents.insert(agent.id.clone(), agent);
        }

        Ok(Self { agents })
    }

    /// Get an agent by ID
    pub fn get(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    /// List all available agents
    pub fn list(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    /// List agents by provider
    pub fn list_by_provider(&self, provider: &str) -> Vec<&Agent> {
        self.agents
            .values()
            .filter(|agent| agent.required_providers.contains(&provider.to_string()))
            .collect()
    }

    /// Check if an agent exists
    pub fn exists(&self, id: &str) -> bool {
        self.agents.contains_key(id)
    }

    /// Get the default model for an agent
    pub fn get_default_model(&self, agent_id: &str) -> Option<&str> {
        self.agents
            .get(agent_id)
            .and_then(|a| a.supported_models.iter().find(|m| m.is_default))
            .map(|m| m.model_id.as_str())
    }

    /// Validate that an agent ID references a valid agent
    pub fn validate_agent_id(&self, agent_id: &str) -> Result<()> {
        if self.exists(agent_id) {
            Ok(())
        } else {
            Err(AgentError::NotFound(agent_id.to_string()))
        }
    }

    /// Validate that a model is supported by an agent
    pub fn validate_model_for_agent(&self, agent_id: &str, model: &str) -> Result<()> {
        let agent = self.get(agent_id)
            .ok_or_else(|| AgentError::NotFound(agent_id.to_string()))?;

        if agent.supported_models.iter().any(|m| m.model_id == model) {
            Ok(())
        } else {
            Err(AgentError::InvalidConfig(
                format!("Model '{}' not supported by agent '{}'", model, agent_id)
            ))
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to load agent registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_agents() {
        let registry = AgentRegistry::new().unwrap();
        assert!(!registry.agents.is_empty());
    }

    #[test]
    fn test_get_agent() {
        let registry = AgentRegistry::new().unwrap();
        let claude = registry.get("claude-code");
        assert!(claude.is_some());
        assert_eq!(claude.unwrap().name, "Claude Code");
    }

    #[test]
    fn test_list_agents() {
        let registry = AgentRegistry::new().unwrap();
        let agents = registry.list();
        assert!(agents.len() >= 5);
    }

    #[test]
    fn test_list_by_provider() {
        let registry = AgentRegistry::new().unwrap();
        let anthropic_agents = registry.list_by_provider("anthropic");
        assert!(!anthropic_agents.is_empty());
    }

    #[test]
    fn test_validate_agent_id() {
        let registry = AgentRegistry::new().unwrap();
        assert!(registry.validate_agent_id("claude-code").is_ok());
        assert!(registry.validate_agent_id("invalid").is_err());
    }

    #[test]
    fn test_validate_model_for_agent() {
        let registry = AgentRegistry::new().unwrap();
        assert!(registry.validate_model_for_agent("claude-code", "claude-sonnet-4-5-20250929").is_ok());
        assert!(registry.validate_model_for_agent("claude-code", "gpt-5").is_err());
    }
}
