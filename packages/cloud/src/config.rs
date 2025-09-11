use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub api_token: String,
    pub api_url: String,
    pub auto_sync: bool,
    pub sync_mode: String,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            api_token: String::new(),
            api_url: "https://api.orkee.ai".to_string(),
            auto_sync: false,
            sync_mode: "incremental".to_string(),
        }
    }
}

impl CloudConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let contents = std::fs::read_to_string(config_path)?;
        let config: CloudConfig = toml::from_str(&contents)?;
        
        // Override with environment variables if set
        let mut config = config;
        if let Ok(token) = std::env::var("ORKEE_CLOUD_TOKEN") {
            config.api_token = token;
        }
        if let Ok(url) = std::env::var("ORKEE_CLOUD_API_URL") {
            config.api_url = url;
        }
        
        Ok(config)
    }
    
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(config_path, contents)?;
        Ok(())
    }
    
    pub fn config_path() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".orkee").join("cloud_config.toml"))
    }
    
    pub fn is_enabled(&self) -> bool {
        !self.api_token.is_empty()
    }
}
