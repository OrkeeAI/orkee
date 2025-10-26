// ABOUTME: Model and agent registry service
// ABOUTME: Loads JSON configurations and provides in-memory lookup for models and agents

use std::collections::HashMap;
use std::sync::LazyLock;

use super::types::{Agent, AgentsConfig, Model, ModelsConfig};

/// Global registry for models and agents, loaded from JSON at startup
pub static REGISTRY: LazyLock<ModelRegistry> = LazyLock::new(|| {
    ModelRegistry::new().unwrap_or_else(|e| {
        panic!(
            "FATAL: Failed to load model/agent configuration. \
             Check config/models.json and config/agents.json: {}",
            e
        )
    })
});

#[derive(Debug)]
pub struct ModelRegistry {
    models: HashMap<String, Model>,
    agents: HashMap<String, Agent>,
}

impl ModelRegistry {
    /// Create a new registry by loading from embedded JSON files
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load JSON files embedded at compile time
        let models_json = include_str!("../../config/models.json");
        let agents_json = include_str!("../../config/agents.json");

        // Parse JSON
        let models_config: ModelsConfig = serde_json::from_str(models_json)?;
        let agents_config: AgentsConfig = serde_json::from_str(agents_json)?;

        // Build HashMaps for fast lookup
        let models = models_config
            .models
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        let agents = agents_config
            .agents
            .into_iter()
            .map(|a| (a.id.clone(), a))
            .collect();

        Ok(Self { models, agents })
    }

    /// Get a model by ID
    pub fn get_model(&self, model_id: &str) -> Option<&Model> {
        self.models.get(model_id)
    }

    /// Get an agent by ID
    pub fn get_agent(&self, agent_id: &str) -> Option<&Agent> {
        self.agents.get(agent_id)
    }

    /// List all models
    pub fn list_models(&self) -> Vec<&Model> {
        self.models.values().collect()
    }

    /// List all agents
    pub fn list_agents(&self) -> Vec<&Agent> {
        self.agents.values().collect()
    }

    /// Get models supported by an agent
    pub fn get_agent_models(&self, agent_id: &str) -> Option<Vec<&Model>> {
        let agent = self.agents.get(agent_id)?;

        let mut models: Vec<&Model> = agent
            .supported_models
            .iter()
            .filter_map(|model_ref| self.models.get(&model_ref.model_id))
            .collect();

        // Sort by display_order from agent's model references
        models.sort_by_key(|m| {
            agent
                .supported_models
                .iter()
                .find(|mr| mr.model_id == m.id)
                .map(|mr| mr.display_order)
                .unwrap_or(u32::MAX)
        });

        Some(models)
    }

    /// Get the default model for an agent
    pub fn get_agent_default_model(&self, agent_id: &str) -> Option<&Model> {
        let agent = self.agents.get(agent_id)?;

        let default_model_ref = agent.supported_models.iter().find(|mr| mr.is_default)?;

        self.models.get(&default_model_ref.model_id)
    }

    /// Validate that an agent supports a specific model
    pub fn validate_agent_model(&self, agent_id: &str, model_id: &str) -> bool {
        if let Some(agent) = self.agents.get(agent_id) {
            agent
                .supported_models
                .iter()
                .any(|mr| mr.model_id == model_id)
        } else {
            false
        }
    }

    /// Check if a model exists
    pub fn model_exists(&self, model_id: &str) -> bool {
        self.models.contains_key(model_id)
    }

    /// Check if an agent exists
    pub fn agent_exists(&self, agent_id: &str) -> bool {
        self.agents.contains_key(agent_id)
    }

    /// Get all models for a specific provider
    pub fn get_models_by_provider(&self, provider: &str) -> Vec<&Model> {
        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    /// Get recommended models for an agent
    pub fn get_agent_recommended_models(&self, agent_id: &str) -> Option<Vec<&Model>> {
        let agent = self.agents.get(agent_id)?;

        let mut models: Vec<&Model> = agent
            .supported_models
            .iter()
            .filter(|mr| mr.is_recommended)
            .filter_map(|model_ref| self.models.get(&model_ref.model_id))
            .collect();

        // Sort by display_order
        models.sort_by_key(|m| {
            agent
                .supported_models
                .iter()
                .find(|mr| mr.model_id == m.id)
                .map(|mr| mr.display_order)
                .unwrap_or(u32::MAX)
        });

        Some(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_loads() {
        let registry = ModelRegistry::new().expect("Registry should load");
        assert!(!registry.models.is_empty(), "Should have models");
        assert!(!registry.agents.is_empty(), "Should have agents");
    }

    #[test]
    fn test_get_model() {
        let registry = ModelRegistry::new().unwrap();
        let model = registry.get_model("claude-sonnet-4-5-20250929");
        assert!(model.is_some(), "Should find Claude Sonnet 4.5");
        assert_eq!(model.unwrap().name, "Claude Sonnet 4.5");
    }

    #[test]
    fn test_get_agent() {
        let registry = ModelRegistry::new().unwrap();
        let agent = registry.get_agent("claude-code");
        assert!(agent.is_some(), "Should find Claude Code agent");
        assert_eq!(agent.unwrap().name, "Claude Code");
    }

    #[test]
    fn test_get_agent_models() {
        let registry = ModelRegistry::new().unwrap();
        let models = registry.get_agent_models("claude-code");
        assert!(models.is_some(), "Should get models for claude-code");

        let models = models.unwrap();
        assert!(!models.is_empty(), "Should have at least one model");

        // Verify models are sorted by display_order
        for i in 0..models.len() - 1 {
            let agent = registry.get_agent("claude-code").unwrap();
            let order1 = agent
                .supported_models
                .iter()
                .find(|mr| mr.model_id == models[i].id)
                .unwrap()
                .display_order;
            let order2 = agent
                .supported_models
                .iter()
                .find(|mr| mr.model_id == models[i + 1].id)
                .unwrap()
                .display_order;
            assert!(order1 <= order2, "Models should be sorted by display_order");
        }
    }

    #[test]
    fn test_get_agent_default_model() {
        let registry = ModelRegistry::new().unwrap();
        let model = registry.get_agent_default_model("claude-code");
        assert!(model.is_some(), "Should have default model");
        assert_eq!(
            model.unwrap().id,
            "claude-sonnet-4-5-20250929",
            "Default should be Sonnet 4.5"
        );
    }

    #[test]
    fn test_validate_agent_model() {
        let registry = ModelRegistry::new().unwrap();

        // Valid combinations for claude-code (Claude-only agent)
        assert!(
            registry.validate_agent_model("claude-code", "claude-sonnet-4-5-20250929"),
            "claude-code should support sonnet-4.5"
        );
        assert!(
            registry.validate_agent_model("claude-code", "claude-haiku-4-5-20251001"),
            "claude-code should support haiku-4.5"
        );
        assert!(
            registry.validate_agent_model("claude-code", "claude-opus-4-1-20250805"),
            "claude-code should support opus-4.1"
        );

        // Valid combinations for aider (multi-provider agent)
        assert!(
            registry.validate_agent_model("aider", "claude-sonnet-4-5-20250929"),
            "aider should support sonnet-4.5"
        );
        assert!(
            registry.validate_agent_model("aider", "gpt-4o"),
            "aider should support gpt-4o"
        );
        assert!(
            registry.validate_agent_model("aider", "claude-opus-4-1-20250805"),
            "aider should support opus-4.1"
        );

        // Invalid combinations - models not in agent's supported list
        assert!(
            !registry.validate_agent_model("aider", "claude-haiku-4-5-20251001"),
            "aider should not support haiku-4.5 (not in its supported models list)"
        );
        assert!(
            !registry.validate_agent_model("claude-code", "gpt-4o"),
            "claude-code should not support gpt-4o (Claude-only agent)"
        );

        // Invalid combinations - nonexistent agent or model
        assert!(
            !registry.validate_agent_model("nonexistent-agent", "claude-sonnet-4-5-20250929"),
            "nonexistent agent should not validate"
        );
        assert!(
            !registry.validate_agent_model("aider", "nonexistent-model"),
            "nonexistent model should not validate"
        );
    }

    #[test]
    fn test_model_exists() {
        let registry = ModelRegistry::new().unwrap();
        assert!(registry.model_exists("claude-sonnet-4-5-20250929"));
        assert!(!registry.model_exists("nonexistent-model"));
    }

    #[test]
    fn test_agent_exists() {
        let registry = ModelRegistry::new().unwrap();
        assert!(registry.agent_exists("claude-code"));
        assert!(!registry.agent_exists("nonexistent-agent"));
    }

    #[test]
    fn test_get_models_by_provider() {
        let registry = ModelRegistry::new().unwrap();
        let anthropic_models = registry.get_models_by_provider("anthropic");
        assert!(!anthropic_models.is_empty(), "Should have Anthropic models");
        assert!(
            anthropic_models.iter().all(|m| m.provider == "anthropic"),
            "All models should be from Anthropic"
        );
    }

    #[test]
    fn test_get_agent_recommended_models() {
        let registry = ModelRegistry::new().unwrap();
        let models = registry.get_agent_recommended_models("claude-code");
        assert!(models.is_some(), "Should have recommended models");

        let models = models.unwrap();
        assert!(
            !models.is_empty(),
            "Should have at least one recommended model"
        );
    }

    #[test]
    fn test_codex_agent() {
        let registry = ModelRegistry::new().unwrap();

        // Verify Codex agent exists
        assert!(registry.agent_exists("codex"), "Codex agent should exist");

        // Verify Codex agent details
        let agent = registry.get_agent("codex");
        assert!(agent.is_some(), "Should find Codex agent");
        let agent = agent.unwrap();
        assert_eq!(agent.name, "Codex CLI");
        assert_eq!(agent.id, "codex");

        // Verify Codex supports GPT-5 models
        assert!(
            registry.validate_agent_model("codex", "gpt-5-codex"),
            "codex should support gpt-5-codex"
        );
        assert!(
            registry.validate_agent_model("codex", "gpt-5"),
            "codex should support gpt-5"
        );
        assert!(
            registry.validate_agent_model("codex", "gpt-5-mini"),
            "codex should support gpt-5-mini"
        );
        assert!(
            registry.validate_agent_model("codex", "gpt-4o"),
            "codex should support gpt-4o"
        );

        // Verify Codex default model is GPT-5-Codex
        let default_model = registry.get_agent_default_model("codex");
        assert!(default_model.is_some(), "Should have default model");
        assert_eq!(
            default_model.unwrap().id,
            "gpt-5-codex",
            "Default should be GPT-5-Codex"
        );

        // Verify Codex does not support Claude models
        assert!(
            !registry.validate_agent_model("codex", "claude-sonnet-4-5-20250929"),
            "codex should not support Claude models"
        );
    }

    #[test]
    fn test_gpt5_models() {
        let registry = ModelRegistry::new().unwrap();

        // Verify GPT-5 exists
        assert!(registry.model_exists("gpt-5"), "GPT-5 should exist");
        let model = registry.get_model("gpt-5");
        assert!(model.is_some(), "Should find GPT-5");
        let model = model.unwrap();
        assert_eq!(model.name, "GPT-5");
        assert_eq!(model.provider, "openai");

        // Verify GPT-5 Mini exists
        assert!(
            registry.model_exists("gpt-5-mini"),
            "GPT-5 Mini should exist"
        );
        let mini_model = registry.get_model("gpt-5-mini");
        assert!(mini_model.is_some(), "Should find GPT-5 Mini");
        assert_eq!(mini_model.unwrap().name, "GPT-5 Mini");

        // Verify GPT-5-Codex exists
        assert!(
            registry.model_exists("gpt-5-codex"),
            "GPT-5-Codex should exist"
        );
        let codex_model = registry.get_model("gpt-5-codex");
        assert!(codex_model.is_some(), "Should find GPT-5-Codex");
        let codex_model = codex_model.unwrap();
        assert_eq!(codex_model.name, "GPT-5 Codex");
        assert_eq!(codex_model.provider, "openai");
    }

    #[test]
    fn test_gemini_cli_agent() {
        let registry = ModelRegistry::new().unwrap();

        // Verify Gemini CLI agent exists
        assert!(
            registry.agent_exists("gemini-cli"),
            "Gemini CLI agent should exist"
        );

        // Verify Gemini CLI agent details
        let agent = registry.get_agent("gemini-cli");
        assert!(agent.is_some(), "Should find Gemini CLI agent");
        let agent = agent.unwrap();
        assert_eq!(agent.name, "Gemini CLI");
        assert_eq!(agent.id, "gemini-cli");

        // Verify Gemini CLI supports Gemini 2.5 models
        assert!(
            registry.validate_agent_model("gemini-cli", "gemini-2.5-pro"),
            "gemini-cli should support gemini-2.5-pro"
        );
        assert!(
            registry.validate_agent_model("gemini-cli", "gemini-2.5-flash"),
            "gemini-cli should support gemini-2.5-flash"
        );
        assert!(
            registry.validate_agent_model("gemini-cli", "gemini-2.5-flash-lite"),
            "gemini-cli should support gemini-2.5-flash-lite"
        );

        // Verify Gemini CLI default model is Gemini 2.5 Pro
        let default_model = registry.get_agent_default_model("gemini-cli");
        assert!(default_model.is_some(), "Should have default model");
        assert_eq!(
            default_model.unwrap().id,
            "gemini-2.5-pro",
            "Default should be Gemini 2.5 Pro"
        );

        // Verify Gemini CLI does not support OpenAI or Anthropic models
        assert!(
            !registry.validate_agent_model("gemini-cli", "gpt-5"),
            "gemini-cli should not support GPT-5"
        );
        assert!(
            !registry.validate_agent_model("gemini-cli", "claude-sonnet-4-5-20250929"),
            "gemini-cli should not support Claude models"
        );
    }

    #[test]
    fn test_gemini25_models() {
        let registry = ModelRegistry::new().unwrap();

        // Verify Gemini 2.5 Pro exists
        assert!(
            registry.model_exists("gemini-2.5-pro"),
            "Gemini 2.5 Pro should exist"
        );
        let model = registry.get_model("gemini-2.5-pro");
        assert!(model.is_some(), "Should find Gemini 2.5 Pro");
        let model = model.unwrap();
        assert_eq!(model.name, "Gemini 2.5 Pro");
        assert_eq!(model.provider, "google");

        // Verify Gemini 2.5 Flash exists
        assert!(
            registry.model_exists("gemini-2.5-flash"),
            "Gemini 2.5 Flash should exist"
        );
        let flash_model = registry.get_model("gemini-2.5-flash");
        assert!(flash_model.is_some(), "Should find Gemini 2.5 Flash");
        assert_eq!(flash_model.unwrap().name, "Gemini 2.5 Flash");

        // Verify Gemini 2.5 Flash-Lite exists
        assert!(
            registry.model_exists("gemini-2.5-flash-lite"),
            "Gemini 2.5 Flash-Lite should exist"
        );
        let lite_model = registry.get_model("gemini-2.5-flash-lite");
        assert!(lite_model.is_some(), "Should find Gemini 2.5 Flash-Lite");
        let lite_model = lite_model.unwrap();
        assert_eq!(lite_model.name, "Gemini 2.5 Flash-Lite");
        assert_eq!(lite_model.provider, "google");
    }
}
