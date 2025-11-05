// ABOUTME: Sandbox provider registry for loading and managing sandbox provider configurations
// ABOUTME: Loads provider definitions from config/providers.json at runtime

pub mod providers;
pub mod settings;
pub mod storage;

pub use providers::{DockerProvider, Provider as SandboxProvider};
pub use settings::{ProviderSettings, SandboxSettings, SettingsManager};
pub use storage::{
    EnvVar, ExecutionStatus, Sandbox, SandboxExecution, SandboxStatus, SandboxStorage,
    StorageError, Volume,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Failed to load providers config: {0}")]
    LoadError(String),
    #[error("Provider not found: {0}")]
    NotFound(String),
    #[error("Invalid provider configuration: {0}")]
    InvalidConfig(String),
}

type Result<T> = std::result::Result<T, ProviderError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub gpu: bool,
    pub persistent_storage: bool,
    pub public_urls: bool,
    pub ssh_access: bool,
    pub auto_scaling: bool,
    pub regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPricing {
    pub base_cost: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_hour: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_gb_memory: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_vcpu: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_per_hour: Option<HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_million_requests: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_gb_bandwidth: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_requests: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_bandwidth_gb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_cpu_hour: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_gb_hour: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_execution: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_gb_storage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_gb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_vcpus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_storage_gb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_runtime_hours: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_execution_time_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_script_size_kb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_runtime_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size_mb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub provider_type: String,
    pub capabilities: ProviderCapabilities,
    pub pricing: ProviderPricing,
    pub limits: ProviderLimits,
    pub default_config: serde_json::Value,
    pub is_available: bool,
    pub requires_auth: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProvidersConfig {
    version: String,
    providers: Vec<Provider>,
}

pub struct ProviderRegistry {
    providers: HashMap<String, Provider>,
}

impl ProviderRegistry {
    /// Create a new ProviderRegistry by loading providers from config file
    pub fn new() -> Result<Self> {
        let config_json = include_str!("../config/providers.json");
        let config: ProvidersConfig = serde_json::from_str(config_json)
            .map_err(|e| ProviderError::LoadError(e.to_string()))?;

        let mut providers = HashMap::new();
        for provider in config.providers {
            providers.insert(provider.id.clone(), provider);
        }

        Ok(Self { providers })
    }

    /// Get a provider by ID
    pub fn get(&self, id: &str) -> Option<&Provider> {
        self.providers.get(id)
    }

    /// List all available providers
    pub fn list(&self) -> Vec<&Provider> {
        self.providers.values().collect()
    }

    /// List providers by type
    pub fn list_by_type(&self, provider_type: &str) -> Vec<&Provider> {
        self.providers
            .values()
            .filter(|provider| provider.provider_type == provider_type)
            .collect()
    }

    /// List providers that are available
    pub fn list_available(&self) -> Vec<&Provider> {
        self.providers
            .values()
            .filter(|provider| provider.is_available)
            .collect()
    }

    /// Check if a provider exists
    pub fn exists(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    /// Validate that a provider ID references a valid provider
    pub fn validate_provider_id(&self, provider_id: &str) -> Result<()> {
        if self.exists(provider_id) {
            Ok(())
        } else {
            Err(ProviderError::NotFound(provider_id.to_string()))
        }
    }

    /// Get providers that support GPU
    pub fn list_gpu_providers(&self) -> Vec<&Provider> {
        self.providers
            .values()
            .filter(|provider| provider.capabilities.gpu)
            .collect()
    }

    /// Get providers with persistent storage
    pub fn list_persistent_storage_providers(&self) -> Vec<&Provider> {
        self.providers
            .values()
            .filter(|provider| provider.capabilities.persistent_storage)
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to load provider registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_providers() {
        let registry = ProviderRegistry::new().unwrap();
        assert!(!registry.providers.is_empty());
    }

    #[test]
    fn test_get_provider() {
        let registry = ProviderRegistry::new().unwrap();
        let local = registry.get("local");
        assert!(local.is_some());
        assert_eq!(local.unwrap().name, "Local Docker");
    }

    #[test]
    fn test_list_providers() {
        let registry = ProviderRegistry::new().unwrap();
        let providers = registry.list();
        assert!(providers.len() >= 8);
    }

    #[test]
    fn test_list_by_type() {
        let registry = ProviderRegistry::new().unwrap();
        let docker_providers = registry.list_by_type("docker");
        assert!(!docker_providers.is_empty());
    }

    #[test]
    fn test_list_available() {
        let registry = ProviderRegistry::new().unwrap();
        let available = registry.list_available();
        assert!(!available.is_empty());
    }

    #[test]
    fn test_validate_provider_id() {
        let registry = ProviderRegistry::new().unwrap();
        assert!(registry.validate_provider_id("local").is_ok());
        assert!(registry.validate_provider_id("invalid").is_err());
    }

    #[test]
    fn test_list_gpu_providers() {
        let registry = ProviderRegistry::new().unwrap();
        let gpu_providers = registry.list_gpu_providers();
        assert!(!gpu_providers.is_empty());
    }

    #[test]
    fn test_list_persistent_storage_providers() {
        let registry = ProviderRegistry::new().unwrap();
        let storage_providers = registry.list_persistent_storage_providers();
        assert!(!storage_providers.is_empty());
    }
}
