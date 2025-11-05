// ABOUTME: Integration tests for OAuth token storage
// ABOUTME: Tests encryption, storage, retrieval, and deletion of OAuth tokens

use chrono::Utc;
use nanoid::nanoid;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tempfile::TempDir;

use orkee_auth::oauth::{
    storage::OAuthStorage,
    types::{OAuthProvider, OAuthProviderConfig, OAuthToken},
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
async fn test_store_and_retrieve_token() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let token = create_test_token("user-1", OAuthProvider::Claude);

    // Store token
    storage.store_token(&token).await.unwrap();

    // Retrieve token
    let retrieved = storage
        .get_token("user-1", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.user_id, token.user_id);
    assert_eq!(retrieved.provider, token.provider);
    assert_eq!(retrieved.access_token, token.access_token);
    assert_eq!(retrieved.refresh_token, token.refresh_token);
    assert_eq!(retrieved.expires_at, token.expires_at);
    assert_eq!(retrieved.subscription_type, token.subscription_type);
    assert_eq!(retrieved.account_email, token.account_email);
}

#[tokio::test]
async fn test_store_token_upsert() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let token1 = create_test_token("user-1", OAuthProvider::Claude);
    storage.store_token(&token1).await.unwrap();

    // Store again with updated access token (should upsert)
    let mut token2 = token1.clone();
    token2.access_token = "new_access_token".to_string();
    storage.store_token(&token2).await.unwrap();

    // Should have the new token
    let retrieved = storage
        .get_token("user-1", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.access_token, "new_access_token");
}

#[tokio::test]
async fn test_get_token_not_found() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let result = storage
        .get_token("nonexistent-user", OAuthProvider::Claude)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_token() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let token = create_test_token("user-1", OAuthProvider::OpenAI);
    storage.store_token(&token).await.unwrap();

    // Verify it exists
    let retrieved = storage
        .get_token("user-1", OAuthProvider::OpenAI)
        .await
        .unwrap();
    assert!(retrieved.is_some());

    // Delete it
    storage
        .delete_token("user-1", OAuthProvider::OpenAI)
        .await
        .unwrap();

    // Verify it's gone
    let retrieved = storage
        .get_token("user-1", OAuthProvider::OpenAI)
        .await
        .unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_multiple_providers_per_user() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    // Store tokens for multiple providers
    let token_claude = create_test_token("user-1", OAuthProvider::Claude);
    let token_openai = create_test_token("user-1", OAuthProvider::OpenAI);
    let token_google = create_test_token("user-1", OAuthProvider::Google);

    storage.store_token(&token_claude).await.unwrap();
    storage.store_token(&token_openai).await.unwrap();
    storage.store_token(&token_google).await.unwrap();

    // Retrieve each token
    let retrieved_claude = storage
        .get_token("user-1", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();
    let retrieved_openai = storage
        .get_token("user-1", OAuthProvider::OpenAI)
        .await
        .unwrap()
        .unwrap();
    let retrieved_google = storage
        .get_token("user-1", OAuthProvider::Google)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved_claude.provider, "claude");
    assert_eq!(retrieved_openai.provider, "openai");
    assert_eq!(retrieved_google.provider, "google");
}

#[tokio::test]
async fn test_multiple_users_same_provider() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let token_user1 = create_test_token("user-1", OAuthProvider::Claude);
    let token_user2 = create_test_token("user-2", OAuthProvider::Claude);

    storage.store_token(&token_user1).await.unwrap();
    storage.store_token(&token_user2).await.unwrap();

    // Retrieve tokens for different users
    let retrieved_user1 = storage
        .get_token("user-1", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();
    let retrieved_user2 = storage
        .get_token("user-2", OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved_user1.user_id, "user-1");
    assert_eq!(retrieved_user2.user_id, "user-2");
    assert_ne!(retrieved_user1.access_token, retrieved_user2.access_token);
}

#[tokio::test]
async fn test_store_and_retrieve_provider_config() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let config = OAuthProviderConfig {
        provider: "claude".to_string(),
        client_id: "test-client-id".to_string(),
        client_secret: Some("test-client-secret".to_string()),
        auth_url: "https://test.auth.url".to_string(),
        token_url: "https://test.token.url".to_string(),
        redirect_uri: "http://localhost:3737/callback".to_string(),
        scopes: vec!["scope1".to_string(), "scope2".to_string()],
        enabled: true,
    };

    // Store config
    storage.store_provider_config(&config).await.unwrap();

    // Retrieve config
    let retrieved = storage
        .get_provider_config(OAuthProvider::Claude)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.provider, config.provider);
    assert_eq!(retrieved.client_id, config.client_id);
    assert_eq!(retrieved.client_secret, config.client_secret);
    assert_eq!(retrieved.auth_url, config.auth_url);
    assert_eq!(retrieved.token_url, config.token_url);
    assert_eq!(retrieved.scopes, config.scopes);
    assert_eq!(retrieved.enabled, config.enabled);
}

#[tokio::test]
async fn test_provider_config_upsert() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let config1 = OAuthProviderConfig {
        provider: "openai".to_string(),
        client_id: "old-client-id".to_string(),
        client_secret: None,
        auth_url: "https://old.auth.url".to_string(),
        token_url: "https://old.token.url".to_string(),
        redirect_uri: "http://localhost:3737/callback".to_string(),
        scopes: vec!["scope1".to_string()],
        enabled: true,
    };

    storage.store_provider_config(&config1).await.unwrap();

    // Update config
    let config2 = OAuthProviderConfig {
        provider: "openai".to_string(),
        client_id: "new-client-id".to_string(),
        client_secret: Some("new-secret".to_string()),
        auth_url: "https://new.auth.url".to_string(),
        token_url: "https://new.token.url".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        scopes: vec!["scope1".to_string(), "scope2".to_string()],
        enabled: false,
    };

    storage.store_provider_config(&config2).await.unwrap();

    // Should have the new config
    let retrieved = storage
        .get_provider_config(OAuthProvider::OpenAI)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.client_id, "new-client-id");
    assert_eq!(retrieved.client_secret, Some("new-secret".to_string()));
    assert_eq!(retrieved.auth_url, "https://new.auth.url");
    assert_eq!(retrieved.scopes.len(), 2);
    assert!(!retrieved.enabled);
}

#[tokio::test]
async fn test_get_provider_config_not_found() {
    let (pool, _temp_dir) = setup_test_db().await;
    let storage = OAuthStorage::new(pool).unwrap();

    let result = storage
        .get_provider_config(OAuthProvider::XAI)
        .await
        .unwrap();
    assert!(result.is_none());
}
