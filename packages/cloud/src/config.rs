use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::client::{CloudError, CloudResult};

/// Cloud mode - whether cloud is enabled or disabled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudMode {
    Disabled,
    Enabled,
}

impl Default for CloudMode {
    fn default() -> Self {
        CloudMode::Disabled
    }
}

/// Cloud configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// Whether cloud is enabled
    pub mode: CloudMode,
    
    /// Supabase project URL
    pub project_url: String,
    
    /// Supabase anonymous key
    pub anon_key: String,
    
    /// Sync settings
    pub sync: SyncConfig,
    
    /// Encryption settings
    pub encryption: EncryptionConfig,
}

impl CloudConfig {
    /// Get the configuration file path
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("orkee")
            .join("cloud-config.toml")
    }

    /// Load configuration from disk
    pub async fn load() -> CloudResult<Self> {
        let path = Self::config_path();
        
        if !path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).await
            .map_err(|e| CloudError::Configuration(format!("Failed to read config: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| CloudError::Configuration(format!("Invalid config format: {}", e)))
    }

    /// Save configuration to disk
    pub async fn save(&self) -> CloudResult<()> {
        let path = Self::config_path();
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| CloudError::Configuration(format!("Failed to create config dir: {}", e)))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| CloudError::Configuration(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&path, content).await
            .map_err(|e| CloudError::Configuration(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Check if configuration file exists
    pub async fn exists() -> bool {
        Self::config_path().exists()
    }

    /// Initialize with Supabase credentials
    pub fn with_credentials(project_url: String, anon_key: String) -> Self {
        Self {
            mode: CloudMode::Enabled,
            project_url,
            anon_key,
            sync: SyncConfig::default(),
            encryption: EncryptionConfig::default(),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> CloudResult<()> {
        if self.mode == CloudMode::Enabled {
            if self.project_url.is_empty() {
                return Err(CloudError::Configuration("Project URL is required".to_string()));
            }
            if self.anon_key.is_empty() {
                return Err(CloudError::Configuration("Anonymous key is required".to_string()));
            }
            if !self.project_url.starts_with("https://") {
                return Err(CloudError::Configuration("Project URL must use HTTPS".to_string()));
            }
        }
        Ok(())
    }
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            mode: CloudMode::Disabled,
            project_url: String::new(),
            anon_key: String::new(),
            sync: SyncConfig::default(),
            encryption: EncryptionConfig::default(),
        }
    }
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Enable automatic sync (paid tiers only)
    pub auto_sync: bool,
    
    /// Sync interval in hours
    pub interval_hours: u32,
    
    /// Conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync: false,
            interval_hours: 24,
            conflict_strategy: ConflictStrategy::PreferLocal,
        }
    }
}

/// How to handle sync conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictStrategy {
    PreferLocal,
    PreferCloud,
    Manual,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption for cloud storage
    pub enabled: bool,
    
    /// Algorithm to use
    pub algorithm: String,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES-256-GCM".to_string(),
        }
    }
}

/// Configuration builder for interactive setup
pub struct CloudConfigBuilder {
    config: CloudConfig,
}

impl CloudConfigBuilder {
    /// Start building a new configuration
    pub fn new() -> Self {
        Self {
            config: CloudConfig::default(),
        }
    }

    /// Set Supabase project URL
    pub fn project_url(mut self, url: String) -> Self {
        self.config.project_url = url;
        self
    }

    /// Set Supabase anonymous key
    pub fn anon_key(mut self, key: String) -> Self {
        self.config.anon_key = key;
        self
    }

    /// Enable cloud
    pub fn enable(mut self) -> Self {
        self.config.mode = CloudMode::Enabled;
        self
    }

    /// Set auto sync
    pub fn auto_sync(mut self, enabled: bool) -> Self {
        self.config.sync.auto_sync = enabled;
        self
    }

    /// Set encryption
    pub fn encryption(mut self, enabled: bool) -> Self {
        self.config.encryption.enabled = enabled;
        self
    }

    /// Build and validate the configuration
    pub fn build(self) -> CloudResult<CloudConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = CloudConfig::default();
        assert!(config.validate().is_ok()); // Disabled mode should validate

        config.mode = CloudMode::Enabled;
        assert!(config.validate().is_err()); // Should fail without credentials

        config.project_url = "https://example.supabase.co".to_string();
        config.anon_key = "test-key".to_string();
        assert!(config.validate().is_ok()); // Should pass with credentials
    }

    #[test]
    fn test_config_builder() {
        let config = CloudConfigBuilder::new()
            .project_url("https://example.supabase.co".to_string())
            .anon_key("test-key".to_string())
            .enable()
            .auto_sync(true)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.mode, CloudMode::Enabled);
        assert!(config.sync.auto_sync);
    }
}