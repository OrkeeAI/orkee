//! Authentication and token management for Orkee Cloud

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::{
    api::{AuthRequest, AuthResponse},
    error::{CloudError, CloudResult},
};

/// OAuth configuration
const AUTH_URL: &str = "/auth/cli";
const TOKEN_EXCHANGE_URL: &str = "/auth/token/exchange";
const CLI_CALLBACK_URL: &str = "http://localhost:3737/auth/callback";

/// Token information stored locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user_email: String,
    pub user_name: String,
    pub user_id: String,
}

impl TokenInfo {
    /// Check if the token is expired (with 5 minute buffer)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let buffer = Duration::minutes(5);
        self.expires_at < now + buffer
    }
    
    /// Check if the token is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Authentication manager
#[derive(Clone)]
pub struct AuthManager {
    config_path: PathBuf,
    token_info: Option<TokenInfo>,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> CloudResult<Self> {
        let config_path = Self::config_file_path()?;
        Ok(Self {
            config_path,
            token_info: None,
        })
    }
    
    /// Initialize the auth manager by loading existing token
    pub async fn init(&mut self) -> CloudResult<()> {
        // Try to load existing token
        if let Err(e) = self.load_token().await {
            tracing::debug!("Could not load existing token: {}", e);
        }
        Ok(())
    }
    
    /// Get the path to the configuration file
    fn config_file_path() -> CloudResult<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| CloudError::config("Could not determine home directory"))?;
        
        let config_dir = home_dir.join(".orkee");
        Ok(config_dir.join("auth.toml"))
    }
    
    /// Load token from file
    async fn load_token(&mut self) -> CloudResult<()> {
        if !self.config_path.exists() {
            return Err(CloudError::config("No auth configuration found"));
        }
        
        let content = fs::read_to_string(&self.config_path).await?;
        let token_info: TokenInfo = toml::from_str(&content)
            .map_err(|e| CloudError::config(format!("Invalid auth configuration: {}", e)))?;
        
        self.token_info = Some(token_info);
        Ok(())
    }
    
    /// Save token to file
    async fn save_token(&self, token_info: &TokenInfo) -> CloudResult<()> {
        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let toml_content = toml::to_string_pretty(token_info)
            .map_err(|e| CloudError::config(format!("Failed to serialize token: {}", e)))?;
        
        fs::write(&self.config_path, toml_content).await?;
        Ok(())
    }
    
    /// Get a valid token, refreshing if necessary
    pub async fn get_valid_token(&self) -> CloudResult<String> {
        match &self.token_info {
            Some(token_info) if token_info.is_valid() => Ok(token_info.token.clone()),
            Some(token_info) if !token_info.is_expired() => {
                // Token is in the buffer zone, try to refresh
                self.refresh_token_internal(token_info.token.clone()).await
            }
            _ => Err(CloudError::auth("No valid token available. Please run 'orkee cloud login'")),
        }
    }
    
    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.token_info
            .as_ref()
            .map(|t| t.is_valid())
            .unwrap_or(false)
    }
    
    /// Get user information if authenticated
    pub fn user_info(&self) -> Option<(&str, &str, &str)> {
        self.token_info
            .as_ref()
            .filter(|t| t.is_valid())
            .map(|t| (t.user_id.as_str(), t.user_email.as_str(), t.user_name.as_str()))
    }
    
    /// Start OAuth flow by opening browser
    pub async fn start_oauth_flow(&self, api_base_url: &str) -> CloudResult<String> {
        // Generate a random state parameter for CSRF protection
        let state = uuid::Uuid::new_v4().to_string();
        
        let auth_url = format!(
            "{}{}?client_id=orkee-cli&redirect_uri={}&state={}",
            api_base_url,
            AUTH_URL,
            urlencoding::encode(CLI_CALLBACK_URL),
            state
        );
        
        println!("🔐 Opening browser for authentication...");
        println!("If the browser doesn't open automatically, visit:");
        println!("  {}", auth_url);
        println!();
        
        // Open browser
        if let Err(e) = open::that(&auth_url) {
            println!("⚠️  Could not open browser automatically: {}", e);
            println!("Please manually visit the URL above.");
        }
        
        Ok(state)
    }
    
    /// Exchange authorization code for token
    pub async fn exchange_code(&mut self, auth_code: String, http_client: &reqwest::Client, api_base_url: &str) -> CloudResult<TokenInfo> {
        let url = format!("{}{}", api_base_url, TOKEN_EXCHANGE_URL);
        let request = AuthRequest { auth_code };
        
        let response = http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(CloudError::auth(format!(
                "Token exchange failed: {}",
                response.status()
            )));
        }
        
        let auth_response: AuthResponse = response.json().await?;
        let token_info = TokenInfo {
            token: auth_response.token,
            expires_at: auth_response.expires_at,
            user_email: auth_response.user.email,
            user_name: auth_response.user.name,
            user_id: auth_response.user.id,
        };
        
        // Save the token
        self.save_token(&token_info).await?;
        self.token_info = Some(token_info.clone());
        
        Ok(token_info)
    }
    
    /// Refresh token
    async fn refresh_token_internal(&self, _current_token: String) -> CloudResult<String> {
        // This would typically make a request to refresh the token
        // For now, return an error to prompt re-authentication
        Err(CloudError::auth("Token expired. Please run 'orkee cloud login' again"))
    }
    
    /// Clear stored token (logout)
    pub async fn logout(&mut self) -> CloudResult<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path).await?;
        }
        self.token_info = None;
        Ok(())
    }
    
    /// Update token information after successful auth
    pub async fn set_token(&mut self, token_info: TokenInfo) -> CloudResult<()> {
        self.save_token(&token_info).await?;
        self.token_info = Some(token_info);
        Ok(())
    }
}

/// Simple HTTP server for handling OAuth callback
pub struct CallbackServer {
    port: u16,
}

impl Default for CallbackServer {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackServer {
    pub fn new() -> Self {
        Self { port: 3737 }
    }
    
    /// Start the callback server and wait for auth code
    pub async fn wait_for_callback(&self) -> CloudResult<String> {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        
        let auth_code = Arc::new(Mutex::new(None::<String>));
        let auth_code_clone = auth_code.clone();
        
        // Simple HTTP listener for the callback
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;
        println!("📡 Waiting for authentication callback...");
        
        // Accept one connection
        let (mut stream, _) = listener.accept().await?;
        
        // Read the HTTP request
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..n]);
        
        // Parse the auth code from the request
        if let Some(code) = Self::extract_auth_code(&request) {
            *auth_code_clone.lock().await = Some(code.clone());
            
            // Send success response
            let response = "HTTP/1.1 200 OK\r\nContent-Length: 133\r\n\r\n<html><body><h1>✅ Authentication Successful!</h1><p>You can now close this tab and return to your terminal.</p></body></html>";
            stream.write_all(response.as_bytes()).await?;
            
            Ok(code)
        } else {
            // Send error response
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 120\r\n\r\n<html><body><h1>❌ Authentication Failed</h1><p>No authorization code found in request.</p></body></html>";
            stream.write_all(response.as_bytes()).await?;
            
            Err(CloudError::auth("No authorization code found in callback"))
        }
    }
    
    /// Extract auth code from HTTP request
    pub fn extract_auth_code(request: &str) -> Option<String> {
        // Look for GET /auth/callback?code=... in the request
        let lines: Vec<&str> = request.lines().collect();
        if let Some(first_line) = lines.first() {
            if let Some(query_start) = first_line.find("?code=") {
                let code_start = query_start + 6;
                let code_part = &first_line[code_start..];
                if let Some(code_end) = code_part.find(&['&', ' '][..]) {
                    return Some(code_part[..code_end].to_string());
                } else {
                    // Code continues to end of line
                    let parts: Vec<&str> = code_part.split_whitespace().collect();
                    return parts.first().map(|s| s.to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_expiry() {
        let expired_token = TokenInfo {
            token: "test".to_string(),
            expires_at: Utc::now() - Duration::minutes(10),
            user_email: "test@example.com".to_string(),
            user_name: "Test User".to_string(),
            user_id: "123".to_string(),
        };
        
        assert!(expired_token.is_expired());
        assert!(!expired_token.is_valid());
        
        let valid_token = TokenInfo {
            token: "test".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            user_email: "test@example.com".to_string(),
            user_name: "Test User".to_string(),
            user_id: "123".to_string(),
        };
        
        assert!(!valid_token.is_expired());
        assert!(valid_token.is_valid());
    }
    
    #[test]
    fn test_extract_auth_code() {
        let request = "GET /auth/callback?code=abc123&state=xyz789 HTTP/1.1\r\nHost: localhost:3737\r\n";
        let code = CallbackServer::extract_auth_code(request);
        assert_eq!(code, Some("abc123".to_string()));
    }
}