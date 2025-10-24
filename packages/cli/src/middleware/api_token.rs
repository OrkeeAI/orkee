// ABOUTME: API token authentication middleware for request authorization
// ABOUTME: Validates API tokens before allowing access to protected endpoints

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn};

use orkee_projects::DbState;

use crate::error::AppError;

/// Header name for API token
pub const API_TOKEN_HEADER: &str = "X-API-Token";

/// Paths that don't require authentication
const WHITELISTED_PATHS: &[&str] = &["/api/health", "/api/status", "/api/csrf-token"];

/// Extension key for storing authentication status in request
pub const AUTHENTICATED_EXTENSION: &str = "api_token_authenticated";

/// Check if a path requires authentication
fn requires_authentication(path: &str) -> bool {
    !WHITELISTED_PATHS
        .iter()
        .any(|&whitelisted| path.starts_with(whitelisted))
}

/// API token validation middleware
pub async fn api_token_middleware(
    State(db): State<DbState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();

    // Skip authentication for whitelisted paths
    if !requires_authentication(path) {
        debug!(path = %path, "Path whitelisted, skipping token validation");
        return Ok(next.run(request).await);
    }

    // Skip authentication in development mode
    if std::env::var("ORKEE_DEV_MODE").is_ok() {
        debug!(path = %path, "Development mode active, skipping token validation");
        return Ok(next.run(request).await);
    }

    // Extract token from header
    let token = request
        .headers()
        .get(API_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok());

    let token = match token {
        Some(t) => t,
        None => {
            warn!(path = %path, "Missing API token");
            return Err(AppError::Unauthorized {
                message: "API token required. Please include X-API-Token header.".to_string(),
            });
        }
    };

    // Verify token
    let token_info = db.token_storage.verify_token(token).await.map_err(|e| {
        warn!(error = %e, "Token verification failed");
        AppError::Unauthorized {
            message: "Invalid API token".to_string(),
        }
    })?;

    if token_info.is_none() {
        warn!(path = %path, "Invalid API token provided");
        return Err(AppError::Unauthorized {
            message: "Invalid API token".to_string(),
        });
    }

    // Update last used timestamp
    // Note: update_last_used expects a token_hash, not the plaintext token
    let token_hash = orkee_projects::api_tokens::TokenStorage::hash_token(token);
    if let Err(e) = db.token_storage.update_last_used(&token_hash).await {
        // Log error but don't fail the request
        warn!(error = %e, "Failed to update token last_used timestamp");
    }

    debug!(path = %path, "API token validated successfully");

    // Store authentication status in request extensions for downstream handlers
    let mut request = request;
    request.extensions_mut().insert(true); // Store boolean flag

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, extract::Request, http::StatusCode, middleware, routing::get, Router};
    use orkee_projects::DbState;
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    async fn setup_test_db() -> DbState {
        let pool = SqlitePool::connect(":memory:")
            .await
            .expect("Failed to create in-memory database");

        // Run migrations
        sqlx::migrate!("../projects/migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        DbState::new(pool).expect("Failed to create DbState")
    }

    async fn create_test_token(db: &DbState) -> String {
        let token_gen = db
            .token_storage
            .create_token("test-token")
            .await
            .expect("Failed to create token");
        token_gen.token
    }

    fn create_test_app(db: DbState) -> Router {
        Router::new()
            .route("/api/test", get(test_handler))
            .route("/api/health", get(test_handler))
            .layer(middleware::from_fn_with_state(
                db.clone(),
                api_token_middleware,
            ))
            .with_state(db)
    }

    #[tokio::test]
    async fn test_whitelisted_paths_bypass_auth() {
        let db = setup_test_db().await;
        let app = create_test_app(db);

        let request = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_missing_token_returns_401() {
        let db = setup_test_db().await;
        let app = create_test_app(db);

        let request = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token_returns_401() {
        let db = setup_test_db().await;
        let app = create_test_app(db);

        let request = Request::builder()
            .uri("/api/test")
            .header(API_TOKEN_HEADER, "invalid-token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_valid_token_allows_access() {
        let db = setup_test_db().await;
        let token = create_test_token(&db).await;
        let app = create_test_app(db);

        let request = Request::builder()
            .uri("/api/test")
            .header(API_TOKEN_HEADER, &token)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_token_updates_last_used() {
        let db = setup_test_db().await;
        let token = create_test_token(&db).await;

        // Get initial token info
        let tokens = db.token_storage.list_tokens().await.unwrap();
        let initial_token = tokens.first().unwrap();
        let initial_last_used = initial_token.last_used_at.clone();

        // Initially, last_used_at should be None
        assert!(initial_last_used.is_none());

        // Wait a bit to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let app = create_test_app(db.clone());

        let request = Request::builder()
            .uri("/api/test")
            .header(API_TOKEN_HEADER, &token)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify last_used_at was updated from None to Some
        let tokens = db.token_storage.list_tokens().await.unwrap();
        let updated_token = tokens.first().unwrap();
        assert!(
            updated_token.last_used_at.is_some(),
            "last_used_at should be set after use"
        );
    }

    #[tokio::test]
    async fn test_requires_authentication_logic() {
        assert!(!requires_authentication("/api/health"));
        assert!(!requires_authentication("/api/status"));
        assert!(!requires_authentication("/api/csrf-token"));
        assert!(requires_authentication("/api/projects"));
        assert!(requires_authentication("/api/test"));
        assert!(requires_authentication("/api/settings"));
    }
}
