// ABOUTME: CSRF protection middleware for state-changing operations
// ABOUTME: Validates CSRF tokens on POST/PUT/DELETE requests to sensitive endpoints

use axum::{body::Body, extract::Request, http::Method, middleware::Next, response::Response};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::AppError;

/// CSRF token header name
pub const CSRF_TOKEN_HEADER: &str = "X-CSRF-Token";

/// Token rotation interval (24 hours)
const TOKEN_ROTATION_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// CSRF token data with rotation support
struct CsrfTokenData {
    token: String,
    created_at: SystemTime,
}

/// CSRF protection layer with server-generated token and rotation
#[derive(Clone)]
pub struct CsrfLayer {
    /// Server-generated CSRF token with rotation support
    data: Arc<RwLock<CsrfTokenData>>,
}

impl CsrfLayer {
    /// Create new CSRF layer with generated token
    pub fn new() -> Self {
        let token = Uuid::new_v4().to_string();
        debug!("Generated CSRF token");
        Self {
            data: Arc::new(RwLock::new(CsrfTokenData {
                token,
                created_at: SystemTime::now(),
            })),
        }
    }

    /// Create CSRF layer with specific token (for testing)
    #[cfg(test)]
    pub fn with_token(token: String) -> Self {
        Self {
            data: Arc::new(RwLock::new(CsrfTokenData {
                token,
                created_at: SystemTime::now(),
            })),
        }
    }

    /// Get the current CSRF token
    pub async fn token(&self) -> String {
        let data = self.data.read().await;
        data.token.clone()
    }

    /// Check if token rotation is needed
    async fn needs_rotation(&self) -> bool {
        let data = self.data.read().await;
        match data.created_at.elapsed() {
            Ok(elapsed) => elapsed >= TOKEN_ROTATION_INTERVAL,
            Err(_) => {
                warn!("System time error when checking token age, forcing rotation");
                true
            }
        }
    }

    /// Rotate the CSRF token
    async fn rotate_token(&self) {
        let mut data = self.data.write().await;
        let new_token = Uuid::new_v4().to_string();
        info!("Rotating CSRF token (token age exceeded 24 hours)");
        data.token = new_token;
        data.created_at = SystemTime::now();
    }

    /// Check and rotate token if needed
    async fn check_and_rotate(&self) {
        if self.needs_rotation().await {
            self.rotate_token().await;
        }
    }

    /// Check if path requires CSRF protection
    fn requires_csrf_protection(path: &str, method: &Method) -> bool {
        // Only protect state-changing methods
        if !matches!(method, &Method::POST | &Method::PUT | &Method::DELETE) {
            return false;
        }

        // Protect security endpoints (password management)
        path.contains("/security/set-password")
            || path.contains("/security/change-password")
            || path.contains("/security/remove-password")
    }
}

impl Default for CsrfLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// CSRF validation middleware
pub async fn csrf_middleware(
    axum::extract::Extension(layer): axum::extract::Extension<CsrfLayer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();
    let method = request.method();

    // Check and rotate token if needed
    layer.check_and_rotate().await;

    // Skip CSRF check if not required
    if !CsrfLayer::requires_csrf_protection(path, method) {
        return Ok(next.run(request).await);
    }

    // Get current token
    let current_token = layer.token().await;

    // Check for CSRF token header
    let token_header = request
        .headers()
        .get(CSRF_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok());

    match token_header {
        Some(provided_token) if provided_token == current_token => {
            debug!(
                path = %path,
                method = %method,
                "CSRF token validated successfully"
            );
            Ok(next.run(request).await)
        }
        Some(_) => {
            warn!(
                path = %path,
                method = %method,
                audit = true,
                "CSRF token validation failed: invalid token"
            );
            Err(AppError::Forbidden {
                message: "Invalid CSRF token".to_string(),
            })
        }
        None => {
            warn!(
                path = %path,
                method = %method,
                audit = true,
                "CSRF token validation failed: missing token"
            );
            Err(AppError::Forbidden {
                message: format!("CSRF token required. Include {} header.", CSRF_TOKEN_HEADER),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_csrf_layer_generates_valid_uuid() {
        let layer = CsrfLayer::new();
        let token = layer.token().await;

        // Should be a valid UUID format (36 characters with hyphens)
        assert_eq!(token.len(), 36);
        assert!(Uuid::parse_str(&token).is_ok());
    }

    #[tokio::test]
    async fn test_csrf_layer_tokens_are_unique() {
        let layer1 = CsrfLayer::new();
        let layer2 = CsrfLayer::new();

        let token1 = layer1.token().await;
        let token2 = layer2.token().await;
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_requires_csrf_protection_for_password_endpoints() {
        assert!(CsrfLayer::requires_csrf_protection(
            "/api/security/set-password",
            &Method::POST
        ));
        assert!(CsrfLayer::requires_csrf_protection(
            "/api/security/change-password",
            &Method::POST
        ));
        assert!(CsrfLayer::requires_csrf_protection(
            "/api/security/remove-password",
            &Method::POST
        ));
    }

    #[test]
    fn test_does_not_require_csrf_for_get_requests() {
        assert!(!CsrfLayer::requires_csrf_protection(
            "/api/security/set-password",
            &Method::GET
        ));
        assert!(!CsrfLayer::requires_csrf_protection(
            "/api/security/status",
            &Method::GET
        ));
    }

    #[test]
    fn test_does_not_require_csrf_for_non_security_endpoints() {
        assert!(!CsrfLayer::requires_csrf_protection(
            "/api/projects",
            &Method::POST
        ));
        assert!(!CsrfLayer::requires_csrf_protection(
            "/api/users/credentials",
            &Method::PUT
        ));
    }

    #[tokio::test]
    async fn test_csrf_layer_with_custom_token() {
        let custom_token = "test-token-123";
        let layer = CsrfLayer::with_token(custom_token.to_string());

        let token = layer.token().await;
        assert_eq!(token, custom_token);
    }

    #[test]
    fn test_requires_csrf_for_all_state_changing_methods() {
        let path = "/api/security/set-password";

        assert!(CsrfLayer::requires_csrf_protection(path, &Method::POST));
        assert!(CsrfLayer::requires_csrf_protection(path, &Method::PUT));
        assert!(CsrfLayer::requires_csrf_protection(path, &Method::DELETE));
        assert!(!CsrfLayer::requires_csrf_protection(path, &Method::GET));
        assert!(!CsrfLayer::requires_csrf_protection(path, &Method::HEAD));
        assert!(!CsrfLayer::requires_csrf_protection(path, &Method::OPTIONS));
    }
}
