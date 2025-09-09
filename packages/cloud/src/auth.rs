use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

use crate::client::CloudError;
use crate::subscription::CloudSubscription;

/// Authentication errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Authentication failed: {0}")]
    Failed(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Token not found")]
    TokenNotFound,
    #[error("Invalid token format")]
    InvalidToken,
    #[error("Network error: {0}")]
    Network(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type AuthResult<T> = Result<T, AuthError>;

/// Cloud authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub user_id: String,
    pub email: String,
}

impl CloudAuthToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token needs refresh (expires in less than 5 minutes)
    pub fn needs_refresh(&self) -> bool {
        Utc::now() + Duration::minutes(5) > self.expires_at
    }
}

/// OAuth callback response
#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// User information from auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub subscription: Option<CloudSubscription>,
}

/// Cloud authentication manager
pub struct CloudAuth {
    http_client: Client,
    project_url: String,
    anon_key: String,
    token_path: PathBuf,
}

impl CloudAuth {
    /// Create a new auth manager
    pub fn new(project_url: String, anon_key: String) -> Self {
        let token_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("orkee")
            .join(".cloud-token");

        Self {
            http_client: Client::new(),
            project_url,
            anon_key,
            token_path,
        }
    }

    /// Start OAuth authentication flow
    pub async fn login(&self) -> AuthResult<CloudAuthToken> {
        // Generate a secure callback token
        let callback_token = uuid::Uuid::new_v4().to_string();
        
        // Start local callback server
        let (tx, rx) = tokio::sync::oneshot::channel();
        let callback_server = self.start_callback_server(callback_token.clone(), tx);
        tokio::spawn(callback_server);

        // Build auth URL
        let auth_url = format!(
            "{}/auth/v1/authorize?provider=github&redirect_to=http://localhost:8899/auth/callback&token={}",
            self.project_url, callback_token
        );

        println!("🔗 Opening browser for authentication...");
        println!("→ If browser doesn't open, visit:");
        println!("  {}\n", auth_url);

        // Open browser
        if let Err(e) = open::that(&auth_url) {
            eprintln!("Failed to open browser: {}", e);
        }

        println!("⏳ Waiting for authentication...");
        
        // Wait for callback
        let auth_data = rx.await
            .map_err(|_| AuthError::Failed("Authentication cancelled".to_string()))?;

        // Convert to our token format
        let token = CloudAuthToken {
            access_token: auth_data.access_token,
            refresh_token: auth_data.refresh_token,
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(auth_data.expires_in),
            user_id: auth_data.user.id,
            email: auth_data.user.email.clone(),
        };

        // Save token
        self.save_token(&token).await?;

        println!("✅ Authenticated as: {}", auth_data.user.email);
        
        Ok(token)
    }

    /// Start local server for OAuth callback
    async fn start_callback_server(
        &self,
        expected_token: String,
        tx: tokio::sync::oneshot::Sender<OAuthCallback>,
    ) -> AuthResult<()> {
        use warp::Filter;

        let tx = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));
        let expected_token = warp::any().map(move || expected_token.clone());
        let tx_filter = warp::any().map(move || tx.clone());

        let callback = warp::path!("auth" / "callback")
            .and(warp::query::<OAuthCallback>())
            .and(expected_token)
            .and(tx_filter)
            .map(|callback: OAuthCallback, expected: String, tx: std::sync::Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<OAuthCallback>>>>| {
                // Send auth data back
                if let Some(sender) = tx.blocking_lock().take() {
                    let _ = sender.send(callback);
                }
                
                // Return success HTML
                warp::reply::html(r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Orkee Cloud - Success</title>
                        <style>
                            body {
                                font-family: system-ui;
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                height: 100vh;
                                margin: 0;
                                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                                color: white;
                            }
                            .success {
                                text-align: center;
                                animation: fadeIn 0.5s;
                            }
                            @keyframes fadeIn {
                                from { opacity: 0; transform: translateY(20px); }
                                to { opacity: 1; transform: translateY(0); }
                            }
                        </style>
                    </head>
                    <body>
                        <div class="success">
                            <h1>✅ Authentication Successful!</h1>
                            <p>You can close this window and return to your terminal.</p>
                        </div>
                        <script>
                            setTimeout(() => window.close(), 3000);
                        </script>
                    </body>
                    </html>
                "#)
            });

        // Run server on port 8899
        warp::serve(callback)
            .run(([127, 0, 0, 1], 8899))
            .await;

        Ok(())
    }

    /// Logout and clear stored token
    pub async fn logout(&self) -> AuthResult<()> {
        if self.token_path.exists() {
            fs::remove_file(&self.token_path).await?;
        }
        println!("✅ Logged out of Orkee Cloud");
        Ok(())
    }

    /// Load saved token
    pub async fn load_token(&self) -> AuthResult<CloudAuthToken> {
        if !self.token_path.exists() {
            return Err(AuthError::TokenNotFound);
        }

        let token_data = fs::read_to_string(&self.token_path).await?;
        let token: CloudAuthToken = serde_json::from_str(&token_data)?;

        if token.is_expired() {
            if let Some(refresh_token) = &token.refresh_token {
                // Try to refresh the token
                return self.refresh_token(refresh_token).await;
            }
            return Err(AuthError::TokenExpired);
        }

        Ok(token)
    }

    /// Save token to disk
    async fn save_token(&self, token: &CloudAuthToken) -> AuthResult<()> {
        // Ensure directory exists
        if let Some(parent) = self.token_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let token_data = serde_json::to_string_pretty(token)?;
        fs::write(&self.token_path, token_data).await?;

        // Set file permissions to user-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&self.token_path).await?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(&self.token_path, permissions).await?;
        }

        Ok(())
    }

    /// Refresh an expired token
    async fn refresh_token(&self, refresh_token: &str) -> AuthResult<CloudAuthToken> {
        let url = format!("{}/auth/v1/token?grant_type=refresh_token", self.project_url);
        
        let response = self.http_client
            .post(&url)
            .header("apikey", &self.anon_key)
            .json(&serde_json::json!({
                "refresh_token": refresh_token
            }))
            .send()
            .await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Failed("Token refresh failed".to_string()));
        }

        let auth_response: OAuthCallback = response.json().await
            .map_err(|e| AuthError::Failed(e.to_string()))?;

        let token = CloudAuthToken {
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token.or_else(|| Some(refresh_token.to_string())),
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(auth_response.expires_in),
            user_id: auth_response.user.id,
            email: auth_response.user.email,
        };

        self.save_token(&token).await?;
        Ok(token)
    }

    /// Get current user information
    pub async fn get_user(&self, token: &CloudAuthToken) -> AuthResult<UserInfo> {
        let url = format!("{}/auth/v1/user", self.project_url);
        
        let response = self.http_client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Failed("Failed to get user info".to_string()));
        }

        let user: UserInfo = response.json().await
            .map_err(|e| AuthError::Failed(e.to_string()))?;

        Ok(user)
    }

    /// Check if user is authenticated
    pub async fn is_authenticated(&self) -> bool {
        self.load_token().await.is_ok()
    }
}