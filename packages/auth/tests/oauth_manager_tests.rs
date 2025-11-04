// ABOUTME: Integration tests for OAuth manager
// ABOUTME: Tests logout, get_status, and get_token manager methods

use chrono::Utc;
use nanoid::nanoid;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tempfile::TempDir;

use orkee_auth::oauth::{
    manager::OAuthManager,
    provider::OAuthProvider,
    types::OAuthToken,
};

/// Helper to create a test database with schema
async fn setup_test_db() -> (SqlitePool, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    // Create schema
    sqlx::query(
        r#"
        CREATE TABLE oauth_tokens (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            expires_at INTEGER NOT NULL,
            token_type TEXT DEFAULT 'Bearer',
            scope TEXT,
            subscription_type TEXT,
            account_email TEXT,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
            UNIQUE(user_id, provider)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE oauth_providers (
            provider TEXT PRIMARY KEY,
            client_id TEXT NOT NULL,
            client_secret TEXT,
            auth_url TEXT NOT NULL,
            token_url TEXT NOT NULL,
            redirect_uri TEXT NOT NULL,
            scopes TEXT NOT NULL,
            enabled BOOLEAN DEFAULT 1,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch())
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create encryption settings table for ApiKeyEncryption
    sqlx::query(
        r#"
        CREATE TABLE encryption_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            mode TEXT NOT NULL CHECK (mode IN ('machine', 'password')) DEFAULT 'machine',
            salt BLOB,
            password_hash BLOB,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch())
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert default encryption settings (machine-based)
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO encryption_settings (id, mode)
        VALUES (1, 'machine')
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    (pool, temp_dir)
}

/// Helper to create a test OAuth token
fn create_test_token(user_id: &str, provider: OAuthProvider) -> OAuthToken {
    OAuthToken {
        id: nanoid!(),
        user_id: user_id.to_string(),
        provider: provider.to_string(),
        access_token: format!("test_access_token_{}", nanoid!()),
        refresh_token: Some(format!("test_refresh_token_{}", nanoid!())),
        expires_at: Utc::now().timestamp() + 3600, // 1 hour from now
        token_type: "Bearer".to_string(),
        scope: Some("model:claude account:read".to_string()),
        subscription_type: Some("pro".to_string()),
        account_email: Some("test@example.com".to_string()),
    }
}

#[tokio::test]
async fn test_get_token_not_found() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool).unwrap();

    let result = manager
        .get_token("nonexistent-user", OAuthProvider::Claude)
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_token_returns_valid_token() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool.clone()).unwrap();

    // Store a token directly via storage layer
    let storage = orkee_auth::oauth::storage::OAuthStorage::new(pool).unwrap();
    let token = create_test_token("user-1", OAuthProvider::Claude);
    storage.store_token(&token).await.unwrap();

    // Get token via manager
    let retrieved = manager
        .get_token("user-1", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.user_id, token.user_id);
    assert_eq!(retrieved.provider, token.provider);
    assert_eq!(retrieved.access_token, token.access_token);
}

#[tokio::test]
async fn test_get_token_returns_none_for_expired_without_refresh() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool.clone()).unwrap();

    // Store an expired token without refresh token
    let storage = orkee_auth::oauth::storage::OAuthStorage::new(pool).unwrap();
    let mut token = create_test_token("user-1", OAuthProvider::OpenAI);
    token.expires_at = Utc::now().timestamp() - 3600; // Expired 1 hour ago
    token.refresh_token = None; // No refresh token
    storage.store_token(&token).await.unwrap();

    // Get token via manager - should return None since it's expired and can't be refreshed
    let result = manager
        .get_token("user-1", OAuthProvider::OpenAI)
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_logout_removes_token() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool.clone()).unwrap();

    // Store a token
    let storage = orkee_auth::oauth::storage::OAuthStorage::new(pool).unwrap();
    let token = create_test_token("user-1", OAuthProvider::Google);
    storage.store_token(&token).await.unwrap();

    // Verify it exists
    let before_logout = storage
        .get_token("user-1", OAuthProvider::Google)
        .await
        .unwrap();
    assert!(before_logout.is_some());

    // Logout
    manager
        .logout("user-1", OAuthProvider::Google)
        .await
        .unwrap();

    // Verify it's gone
    let after_logout = storage
        .get_token("user-1", OAuthProvider::Google)
        .await
        .unwrap();
    assert!(after_logout.is_none());
}

#[tokio::test]
async fn test_get_status_shows_all_providers() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool.clone()).unwrap();

    // Store tokens for some providers
    let storage = orkee_auth::oauth::storage::OAuthStorage::new(pool).unwrap();

    let claude_token = create_test_token("user-1", OAuthProvider::Claude);
    storage.store_token(&claude_token).await.unwrap();

    let openai_token = create_test_token("user-1", OAuthProvider::OpenAI);
    storage.store_token(&openai_token).await.unwrap();

    // Get status
    let statuses = manager.get_status("user-1").await.unwrap();

    // Should have status for all 4 providers
    assert_eq!(statuses.len(), 4);

    // Check Claude status (authenticated)
    let claude_status = statuses
        .iter()
        .find(|s| s.provider == OAuthProvider::Claude)
        .unwrap();
    assert!(claude_status.authenticated);
    assert_eq!(claude_status.account_email, Some("test@example.com".to_string()));

    // Check OpenAI status (authenticated)
    let openai_status = statuses
        .iter()
        .find(|s| s.provider == OAuthProvider::OpenAI)
        .unwrap();
    assert!(openai_status.authenticated);

    // Check Google status (not authenticated)
    let google_status = statuses
        .iter()
        .find(|s| s.provider == OAuthProvider::Google)
        .unwrap();
    assert!(!google_status.authenticated);
    assert!(google_status.expires_at.is_none());

    // Check XAI status (not authenticated)
    let xai_status = statuses
        .iter()
        .find(|s| s.provider == OAuthProvider::XAI)
        .unwrap();
    assert!(!xai_status.authenticated);
}

#[tokio::test]
async fn test_get_status_shows_expired_as_not_authenticated() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool.clone()).unwrap();

    // Store an expired token
    let storage = orkee_auth::oauth::storage::OAuthStorage::new(pool).unwrap();
    let mut token = create_test_token("user-1", OAuthProvider::Claude);
    token.expires_at = Utc::now().timestamp() - 3600; // Expired 1 hour ago
    storage.store_token(&token).await.unwrap();

    // Get status
    let statuses = manager.get_status("user-1").await.unwrap();

    // Claude status should show as not authenticated (expired)
    let claude_status = statuses
        .iter()
        .find(|s| s.provider == OAuthProvider::Claude)
        .unwrap();
    assert!(!claude_status.authenticated);
    assert!(claude_status.expires_at.is_some()); // But still has the expired timestamp
}

#[tokio::test]
async fn test_logout_from_nonexistent_token_succeeds() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool).unwrap();

    // Logout from a provider that was never authenticated - should not error
    let result = manager.logout("user-1", OAuthProvider::XAI).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_users_separate_tokens() {
    let (pool, _temp_dir) = setup_test_db().await;
    let manager = OAuthManager::new(pool.clone()).unwrap();

    // Store tokens for different users
    let storage = orkee_auth::oauth::storage::OAuthStorage::new(pool).unwrap();
    let user1_token = create_test_token("user-1", OAuthProvider::Claude);
    let user2_token = create_test_token("user-2", OAuthProvider::Claude);
    storage.store_token(&user1_token).await.unwrap();
    storage.store_token(&user2_token).await.unwrap();

    // Get tokens for each user
    let user1_retrieved = manager
        .get_token("user-1", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();
    let user2_retrieved = manager
        .get_token("user-2", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();

    // Tokens should be different
    assert_eq!(user1_retrieved.user_id, "user-1");
    assert_eq!(user2_retrieved.user_id, "user-2");
    assert_ne!(user1_retrieved.access_token, user2_retrieved.access_token);

    // Logout user-1
    manager
        .logout("user-1", OAuthProvider::Claude)
        .await
        .unwrap();

    // user-1 should have no token
    let user1_after_logout = manager
        .get_token("user-1", OAuthProvider::Claude)
        .await
        .unwrap();
    assert!(user1_after_logout.is_none());

    // user-2 should still have their token
    let user2_after_user1_logout = manager
        .get_token("user-2", OAuthProvider::Claude)
        .await
        .unwrap();
    assert!(user2_after_user1_logout.is_some());
}
