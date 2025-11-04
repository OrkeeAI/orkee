// ABOUTME: OAuth callback server for handling authorization redirects
// ABOUTME: Listens on localhost:3737 for OAuth callbacks and extracts authorization codes

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tracing::{debug, error, info};

use crate::error::{AuthError, AuthResult};

/// OAuth callback server configuration
pub struct CallbackServer {
    port: u16,
}

impl Default for CallbackServer {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackServer {
    /// Create a new callback server (defaults to port 3737)
    pub fn new() -> Self {
        Self { port: 3737 }
    }

    /// Create callback server with custom port
    pub fn with_port(port: u16) -> Self {
        Self { port }
    }

    /// Get the callback URL for this server
    pub fn callback_url(&self) -> String {
        format!("http://localhost:{}/auth/callback", self.port)
    }

    /// Start server and wait for OAuth callback
    ///
    /// This blocks until either:
    /// - An authorization code and state are received
    /// - An error occurs
    /// - Connection timeout
    ///
    /// Returns (authorization_code, state)
    pub async fn wait_for_callback(&self) -> AuthResult<(String, String)> {
        info!("Starting OAuth callback server on port {}", self.port);

        // Bind to localhost
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| AuthError::CallbackServer(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("📡 Waiting for OAuth callback on {}", addr);

        // Accept one connection
        let (mut stream, peer_addr) = listener.accept().await.map_err(|e| {
            AuthError::CallbackServer(format!("Failed to accept connection: {}", e))
        })?;

        debug!("Received connection from {}", peer_addr);

        // Read HTTP request
        let mut buffer = vec![0; 2048];
        let n = stream
            .read(&mut buffer)
            .await
            .map_err(|e| AuthError::CallbackServer(format!("Failed to read request: {}", e)))?;

        let request = String::from_utf8_lossy(&buffer[..n]);
        debug!("Received request:\n{}", request);

        // Extract authorization code and state
        if let (Some(code), Some(state)) = (
            Self::extract_auth_code(&request),
            Self::extract_state(&request),
        ) {
            // Send success response
            let response = Self::success_response();
            if let Err(e) = stream.write_all(response.as_bytes()).await {
                error!("Failed to send success response: {}", e);
            }

            info!("✅ Successfully received authorization code and state");
            Ok((code, state))
        } else if let Some(error_msg) = Self::extract_error(&request) {
            // OAuth provider returned an error
            let response = Self::error_response(&error_msg);
            let _ = stream.write_all(response.as_bytes()).await;

            Err(AuthError::OAuthFailed(format!(
                "Provider error: {}",
                error_msg
            )))
        } else {
            // No code or error found in request
            let response = Self::error_response("No authorization code or state found in request");
            let _ = stream.write_all(response.as_bytes()).await;

            Err(AuthError::CallbackServer(
                "No authorization code or state found in callback".to_string(),
            ))
        }
    }

    /// Extract authorization code from HTTP request
    fn extract_auth_code(request: &str) -> Option<String> {
        // Look for GET /auth/callback?code=... in the request
        let lines: Vec<&str> = request.lines().collect();
        if let Some(first_line) = lines.first() {
            if let Some(query_start) = first_line.find("?code=") {
                let code_start = query_start + 6;
                let code_part = &first_line[code_start..];

                // Find end of code parameter (either & or space)
                if let Some(code_end) = code_part.find(&['&', ' '][..]) {
                    return Some(code_part[..code_end].to_string());
                } else {
                    // Code continues to end of query string
                    let parts: Vec<&str> = code_part.split_whitespace().collect();
                    return parts.first().map(|s| s.to_string());
                }
            }
        }
        None
    }

    /// Extract state parameter from HTTP request
    fn extract_state(request: &str) -> Option<String> {
        // Look for state= parameter in the query string
        let lines: Vec<&str> = request.lines().collect();
        if let Some(first_line) = lines.first() {
            // Find state parameter (could be ?state= or &state=)
            for pattern in ["?state=", "&state="] {
                if let Some(state_start) = first_line.find(pattern) {
                    let state_part = &first_line[state_start + pattern.len()..];
                    if let Some(state_end) = state_part.find(&['&', ' '][..]) {
                        return Some(state_part[..state_end].to_string());
                    } else {
                        let parts: Vec<&str> = state_part.split_whitespace().collect();
                        return parts.first().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    /// Extract error from OAuth callback
    fn extract_error(request: &str) -> Option<String> {
        let lines: Vec<&str> = request.lines().collect();
        if let Some(first_line) = lines.first() {
            if let Some(error_start) = first_line.find("?error=") {
                let error_part = &first_line[error_start + 7..];
                if let Some(error_end) = error_part.find(&['&', ' '][..]) {
                    return Some(error_part[..error_end].to_string());
                } else {
                    let parts: Vec<&str> = error_part.split_whitespace().collect();
                    return parts.first().map(|s| s.to_string());
                }
            }
        }
        None
    }

    /// Generate success HTML response
    fn success_response() -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            SUCCESS_HTML.len(),
            SUCCESS_HTML
        )
    }

    /// Generate error HTML response
    fn error_response(error_msg: &str) -> String {
        let html = format!(
            r#"<html><body><h1>❌ Authentication Failed</h1><p>{}</p><p>You can close this tab and return to your terminal.</p></body></html>"#,
            error_msg
        );
        format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        )
    }
}

const SUCCESS_HTML: &str = r#"<html>
<head>
    <title>Authentication Successful</title>
    <style>
        body { font-family: system-ui, -apple-system, sans-serif; max-width: 600px; margin: 100px auto; text-align: center; }
        h1 { color: #22c55e; }
        p { color: #64748b; }
    </style>
</head>
<body>
    <h1>✅ Authentication Successful!</h1>
    <p>You have successfully authenticated with the provider.</p>
    <p>You can now close this tab and return to your terminal.</p>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_auth_code() {
        let request =
            "GET /auth/callback?code=abc123&state=xyz789 HTTP/1.1\r\nHost: localhost:3737\r\n";
        let code = CallbackServer::extract_auth_code(request);
        assert_eq!(code, Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_state() {
        let request =
            "GET /auth/callback?code=abc123&state=xyz789 HTTP/1.1\r\nHost: localhost:3737\r\n";
        let state = CallbackServer::extract_state(request);
        assert_eq!(state, Some("xyz789".to_string()));
    }

    #[test]
    fn test_extract_state_as_first_param() {
        let request =
            "GET /auth/callback?state=xyz789&code=abc123 HTTP/1.1\r\nHost: localhost:3737\r\n";
        let state = CallbackServer::extract_state(request);
        assert_eq!(state, Some("xyz789".to_string()));
    }

    #[test]
    fn test_extract_auth_code_no_params() {
        let request = "GET /auth/callback HTTP/1.1\r\nHost: localhost:3737\r\n";
        let code = CallbackServer::extract_auth_code(request);
        assert_eq!(code, None);
    }

    #[test]
    fn test_extract_error() {
        let request = "GET /auth/callback?error=access_denied HTTP/1.1\r\nHost: localhost:3737\r\n";
        let error = CallbackServer::extract_error(request);
        assert_eq!(error, Some("access_denied".to_string()));
    }

    #[test]
    fn test_callback_url() {
        let server = CallbackServer::new();
        assert_eq!(server.callback_url(), "http://localhost:3737/auth/callback");

        let server = CallbackServer::with_port(8080);
        assert_eq!(server.callback_url(), "http://localhost:8080/auth/callback");
    }
}
