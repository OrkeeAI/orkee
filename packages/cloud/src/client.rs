//! HTTP client implementation for Orkee Cloud API

use reqwest::{header, Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::{
    api::*,
    auth::AuthManager,
    error::{CloudError, CloudResult},
};

/// HTTP client for Orkee Cloud API
pub struct HttpClient {
    client: Client,
    base_url: String,
    auth_manager: AuthManager,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(base_url: String, auth_manager: AuthManager) -> CloudResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("orkee-cli/{}", env!("CARGO_PKG_VERSION")))
            .build()?;
        
        Ok(Self {
            client,
            base_url,
            auth_manager,
        })
    }
    
    /// Make an authenticated GET request
    pub async fn get<T>(&self, path: &str) -> CloudResult<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth_manager.get_valid_token().await?;
        
        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Make an authenticated POST request
    pub async fn post<B, T>(&self, path: &str, body: &B) -> CloudResult<T>
    where
        B: serde::Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth_manager.get_valid_token().await?;
        
        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Make an authenticated DELETE request
    pub async fn delete<T>(&self, path: &str) -> CloudResult<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth_manager.get_valid_token().await?;
        
        let response = self
            .client
            .delete(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Make an unauthenticated POST request (for initial auth)
    pub async fn post_unauth<B, T>(&self, path: &str, body: &B) -> CloudResult<T>
    where
        B: serde::Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        
        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Handle HTTP response and parse JSON
    async fn handle_response<T>(&self, response: Response) -> CloudResult<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let response_text = response.text().await?;
        
        match status {
            StatusCode::OK | StatusCode::CREATED => {
                // Try to parse as ApiResponse<T> first
                match serde_json::from_str::<ApiResponse<T>>(&response_text) {
                    Ok(api_response) => api_response.into_result(),
                    Err(_) => {
                        // Fallback to direct parsing
                        serde_json::from_str(&response_text)
                            .map_err(CloudError::Serialization)
                    }
                }
            }
            StatusCode::UNAUTHORIZED => {
                Err(CloudError::TokenExpired)
            }
            StatusCode::NOT_FOUND => {
                Err(CloudError::api("Resource not found"))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                Err(CloudError::api("Rate limit exceeded. Please try again later"))
            }
            StatusCode::BAD_REQUEST => {
                // Try to parse error details
                match serde_json::from_str::<ApiError>(&response_text) {
                    Ok(error) => Err(CloudError::api(format!("{}: {}", error.error, error.message))),
                    Err(_) => Err(CloudError::api("Bad request")),
                }
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                Err(CloudError::api("Internal server error"))
            }
            _ => {
                Err(CloudError::api(format!(
                    "HTTP {} - {}",
                    status,
                    response_text
                )))
            }
        }
    }
    
    /// Get the auth manager (for direct token operations)
    pub fn auth_manager(&self) -> &AuthManager {
        &self.auth_manager
    }
    
    /// Get mutable access to auth manager
    pub fn auth_manager_mut(&mut self) -> &mut AuthManager {
        &mut self.auth_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::MockServer;
    
    #[tokio::test]
    async fn test_http_client_creation() {
        let auth_manager = AuthManager::new().unwrap();
        let client = HttpClient::new("https://api.example.com".to_string(), auth_manager);
        assert!(client.is_ok());
    }
    
    #[tokio::test]
    async fn test_http_client_auth_error() {
        let mock_server = MockServer::start().await;
        
        let auth_manager = AuthManager::new().unwrap();
        let client = HttpClient::new(mock_server.uri(), auth_manager).unwrap();
        
        // Should fail with authentication error because no token is available
        let result: Result<serde_json::Value, _> = client.get("/test").await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, CloudError::Authentication(_)));
    }
}