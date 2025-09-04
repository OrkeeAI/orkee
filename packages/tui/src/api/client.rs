use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

/// HTTP API client for communicating with the Orkee CLI server
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
    
    /// Check server health
    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/api/health", self.base_url))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    /// Get all projects
    pub async fn get_projects(&self) -> Result<Value> {
        let response = self.client
            .get(&format!("{}/api/projects", self.base_url))
            .send()
            .await?;
            
        let projects = response.json::<Value>().await?;
        Ok(projects)
    }
}