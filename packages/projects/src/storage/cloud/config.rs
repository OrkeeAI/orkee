use super::types::SyncConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Cloud configuration for different providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub enabled: bool,
    pub default_provider: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,
    pub sync: SyncConfig,
    pub security: SecurityConfig,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_provider: None,
            providers: HashMap::new(),
            sync: SyncConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

/// Configuration for a specific cloud provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String,
    pub name: String,
    pub settings: HashMap<String, String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProviderConfig {
    /// Create new S3 provider configuration
    pub fn new_s3(name: String, bucket: String, region: String, endpoint: Option<String>) -> Self {
        let mut settings = HashMap::new();
        settings.insert("bucket".to_string(), bucket);
        settings.insert("region".to_string(), region);
        if let Some(endpoint) = endpoint {
            settings.insert("endpoint".to_string(), endpoint);
        }

        Self {
            provider_type: "s3".to_string(),
            name,
            settings,
            enabled: true,
            created_at: chrono::Utc::now(),
            last_used: None,
        }
    }

    /// Create new Cloudflare R2 provider configuration
    pub fn new_r2(name: String, bucket: String, account_id: String) -> Self {
        let mut settings = HashMap::new();
        settings.insert("bucket".to_string(), bucket);
        settings.insert("account_id".to_string(), account_id.clone());
        settings.insert("region".to_string(), "auto".to_string());
        settings.insert("endpoint".to_string(), format!("https://{}.r2.cloudflarestorage.com", account_id));

        Self {
            provider_type: "r2".to_string(),
            name,
            settings,
            enabled: true,
            created_at: chrono::Utc::now(),
            last_used: None,
        }
    }

    /// Get setting value
    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Set setting value
    pub fn set_setting(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }

    /// Mark provider as used
    pub fn mark_used(&mut self) {
        self.last_used = Some(chrono::Utc::now());
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encrypt_snapshots: bool,
    pub encryption_algorithm: String,
    pub key_derivation_iterations: u32,
    pub require_mfa: bool,
    pub max_credential_age_days: u32,
    pub audit_logging: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encrypt_snapshots: true,
            encryption_algorithm: "AES-256-GCM".to_string(),
            key_derivation_iterations: 100000,
            require_mfa: false,
            max_credential_age_days: 90,
            audit_logging: true,
        }
    }
}

/// Cloud configuration manager
pub struct CloudConfigManager {
    config_path: PathBuf,
    config: CloudConfig,
}

impl CloudConfigManager {
    /// Create new configuration manager
    pub fn new(config_dir: PathBuf) -> Self {
        let config_path = config_dir.join("cloud-config.toml");
        Self {
            config_path,
            config: CloudConfig::default(),
        }
    }

    /// Load configuration from file
    pub async fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config_path.exists() {
            // Create default config if it doesn't exist
            self.save().await?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path).await?;
        self.config = toml::from_str(&content)?;
        
        tracing::info!("Loaded cloud configuration from: {:?}", self.config_path);
        Ok(())
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(&self.config)?;
        let mut file = fs::File::create(&self.config_path).await?;
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        tracing::info!("Saved cloud configuration to: {:?}", self.config_path);
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &CloudConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn get_config_mut(&mut self) -> &mut CloudConfig {
        &mut self.config
    }

    /// Enable cloud sync
    pub async fn enable_cloud(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.enabled = true;
        self.save().await?;
        tracing::info!("Cloud sync enabled");
        Ok(())
    }

    /// Disable cloud sync
    pub async fn disable_cloud(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.enabled = false;
        self.save().await?;
        tracing::info!("Cloud sync disabled");
        Ok(())
    }

    /// Add a new provider
    pub async fn add_provider(
        &mut self,
        provider_config: ProviderConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = provider_config.name.clone();
        self.config.providers.insert(name.clone(), provider_config);
        
        // Set as default if it's the first provider
        if self.config.default_provider.is_none() {
            self.config.default_provider = Some(name.clone());
        }

        self.save().await?;
        tracing::info!("Added provider: {}", name);
        Ok(())
    }

    /// Remove a provider
    pub async fn remove_provider(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(_) = self.config.providers.remove(name) {
            // If this was the default provider, clear it
            if self.config.default_provider.as_ref() == Some(&name.to_string()) {
                self.config.default_provider = None;
                
                // Set a new default if other providers exist
                if let Some(new_default) = self.config.providers.keys().next() {
                    self.config.default_provider = Some(new_default.clone());
                }
            }

            self.save().await?;
            tracing::info!("Removed provider: {}", name);
        }
        Ok(())
    }

    /// Set default provider
    pub async fn set_default_provider(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.providers.contains_key(name) {
            self.config.default_provider = Some(name.to_string());
            self.save().await?;
            tracing::info!("Set default provider: {}", name);
        } else {
            return Err(format!("Provider '{}' not found", name).into());
        }
        Ok(())
    }

    /// Get default provider configuration
    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        self.config.default_provider
            .as_ref()
            .and_then(|name| self.config.providers.get(name))
    }

    /// Get provider configuration by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.config.providers.get(name)
    }

    /// List all provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.config.providers.keys().cloned().collect()
    }

    /// Update sync configuration
    pub async fn update_sync_config(&mut self, sync_config: SyncConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config.sync = sync_config;
        self.save().await?;
        tracing::info!("Updated sync configuration");
        Ok(())
    }

    /// Update security configuration
    pub async fn update_security_config(&mut self, security_config: SecurityConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.config.security = security_config;
        self.save().await?;
        tracing::info!("Updated security configuration");
        Ok(())
    }

    /// Mark provider as used
    pub async fn mark_provider_used(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(provider) = self.config.providers.get_mut(name) {
            provider.mark_used();
            self.save().await?;
        }
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.config.enabled && self.config.providers.is_empty() {
            return Err("Cloud sync is enabled but no providers are configured".to_string());
        }

        if self.config.enabled && self.config.default_provider.is_none() {
            return Err("Cloud sync is enabled but no default provider is set".to_string());
        }

        if let Some(default_name) = &self.config.default_provider {
            if !self.config.providers.contains_key(default_name) {
                return Err(format!("Default provider '{}' not found in providers", default_name));
            }
        }

        // Validate each provider
        for (name, provider) in &self.config.providers {
            match provider.provider_type.as_str() {
                "s3" => {
                    if provider.get_setting("bucket").is_none() {
                        return Err(format!("S3 provider '{}' missing bucket setting", name));
                    }
                    if provider.get_setting("region").is_none() {
                        return Err(format!("S3 provider '{}' missing region setting", name));
                    }
                }
                "r2" => {
                    if provider.get_setting("bucket").is_none() {
                        return Err(format!("R2 provider '{}' missing bucket setting", name));
                    }
                    if provider.get_setting("account_id").is_none() {
                        return Err(format!("R2 provider '{}' missing account_id setting", name));
                    }
                }
                _ => {
                    return Err(format!("Unknown provider type: {}", provider.provider_type));
                }
            }
        }

        // Validate sync configuration
        if self.config.sync.sync_interval_hours == 0 {
            return Err("Sync interval cannot be zero".to_string());
        }

        if self.config.sync.max_snapshots == 0 {
            return Err("Maximum snapshots cannot be zero".to_string());
        }

        // Validate security configuration
        if self.config.security.max_credential_age_days == 0 {
            return Err("Maximum credential age cannot be zero".to_string());
        }

        Ok(())
    }

    /// Get configuration summary for display
    pub fn get_summary(&self) -> ConfigSummary {
        let total_providers = self.config.providers.len();
        let enabled_providers = self.config.providers.values()
            .filter(|p| p.enabled)
            .count();

        let provider_types: std::collections::HashSet<_> = self.config.providers.values()
            .map(|p| p.provider_type.clone())
            .collect();

        ConfigSummary {
            cloud_enabled: self.config.enabled,
            total_providers,
            enabled_providers,
            default_provider: self.config.default_provider.clone(),
            provider_types: provider_types.into_iter().collect(),
            auto_sync_enabled: self.config.sync.auto_sync_enabled,
            sync_interval_hours: self.config.sync.sync_interval_hours,
            encryption_enabled: self.config.security.encrypt_snapshots,
            last_validation: chrono::Utc::now(),
        }
    }
}

/// Configuration summary for display purposes
#[derive(Debug, Clone)]
pub struct ConfigSummary {
    pub cloud_enabled: bool,
    pub total_providers: usize,
    pub enabled_providers: usize,
    pub default_provider: Option<String>,
    pub provider_types: Vec<String>,
    pub auto_sync_enabled: bool,
    pub sync_interval_hours: u32,
    pub encryption_enabled: bool,
    pub last_validation: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for ConfigSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cloud Sync Configuration Summary:")?;
        writeln!(f, "  Status: {}", if self.cloud_enabled { "Enabled" } else { "Disabled" })?;
        writeln!(f, "  Providers: {}/{} enabled", self.enabled_providers, self.total_providers)?;
        
        if let Some(default) = &self.default_provider {
            writeln!(f, "  Default Provider: {}", default)?;
        } else {
            writeln!(f, "  Default Provider: None")?;
        }
        
        writeln!(f, "  Provider Types: {}", self.provider_types.join(", "))?;
        writeln!(f, "  Auto Sync: {}", if self.auto_sync_enabled { "Enabled" } else { "Disabled" })?;
        writeln!(f, "  Sync Interval: {} hours", self.sync_interval_hours)?;
        writeln!(f, "  Encryption: {}", if self.encryption_enabled { "Enabled" } else { "Disabled" })?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CloudConfigManager::new(temp_dir.path().to_path_buf());
        
        assert!(!manager.config.enabled);
        assert!(manager.config.providers.is_empty());
    }

    #[tokio::test]
    async fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CloudConfigManager::new(temp_dir.path().to_path_buf());
        
        // Modify config
        manager.get_config_mut().enabled = true;
        manager.save().await.unwrap();
        
        // Create new manager and load
        let mut new_manager = CloudConfigManager::new(temp_dir.path().to_path_buf());
        new_manager.load().await.unwrap();
        
        assert!(new_manager.get_config().enabled);
    }

    #[test]
    fn test_provider_config_s3() {
        let config = ProviderConfig::new_s3(
            "test-s3".to_string(),
            "my-bucket".to_string(),
            "us-east-1".to_string(),
            None,
        );
        
        assert_eq!(config.provider_type, "s3");
        assert_eq!(config.get_setting("bucket").unwrap(), "my-bucket");
        assert_eq!(config.get_setting("region").unwrap(), "us-east-1");
        assert!(config.enabled);
    }

    #[test]
    fn test_provider_config_r2() {
        let config = ProviderConfig::new_r2(
            "test-r2".to_string(),
            "my-bucket".to_string(),
            "account123".to_string(),
        );
        
        assert_eq!(config.provider_type, "r2");
        assert_eq!(config.get_setting("bucket").unwrap(), "my-bucket");
        assert_eq!(config.get_setting("account_id").unwrap(), "account123");
        assert_eq!(config.get_setting("region").unwrap(), "auto");
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_provider_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CloudConfigManager::new(temp_dir.path().to_path_buf());
        
        let s3_config = ProviderConfig::new_s3(
            "my-s3".to_string(),
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            None,
        );
        
        manager.add_provider(s3_config).await.unwrap();
        
        assert_eq!(manager.list_providers().len(), 1);
        assert!(manager.get_provider("my-s3").is_some());
        assert_eq!(manager.get_default_provider().unwrap().name, "my-s3");
        
        manager.remove_provider("my-s3").await.unwrap();
        assert_eq!(manager.list_providers().len(), 0);
        assert!(manager.get_default_provider().is_none());
    }

    #[test]
    fn test_config_validation() {
        let config = CloudConfig::default();
        let manager = CloudConfigManager {
            config_path: PathBuf::new(),
            config,
        };
        
        // Should be valid when disabled
        assert!(manager.validate().is_ok());
        
        // Should fail when enabled with no providers
        let manager = CloudConfigManager {
            config_path: PathBuf::new(),
            config: CloudConfig { enabled: true, ..Default::default() },
        };
        assert!(manager.validate().is_err());
    }
}