//! Integration tests for Orkee Cloud client

use orkee_cloud::{CloudClient, CloudError};

#[tokio::test]
async fn test_cloud_client_creation() {
    // Test client creation with valid URL
    let result = CloudClient::new("https://api.test.com".to_string()).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert!(!client.is_authenticated()); // Should not be authenticated initially
    assert!(client.user_info().is_none()); // No user info initially
}

#[tokio::test]
async fn test_cloud_client_status_unauthenticated() {
    let client = CloudClient::new("https://api.test.com".to_string())
        .await
        .unwrap();

    let status = client.get_status().await.unwrap();
    assert!(!status.authenticated);
    assert!(status.user_email.is_none());
    assert!(status.user_name.is_none());
    assert_eq!(status.projects_count, 0);
    assert!(status.subscription_tier.is_none());
}

#[tokio::test]
async fn test_unauthenticated_api_calls() {
    let client = CloudClient::new("https://api.test.com".to_string())
        .await
        .unwrap();

    // These should fail because we're not authenticated
    let list_result = client.list_projects().await;
    assert!(list_result.is_err());

    let usage_result = client.get_usage().await;
    assert!(usage_result.is_err());
}

#[test]
fn test_cloud_error_types() {
    let auth_error = CloudError::auth("Test auth error");
    assert!(auth_error.is_auth_error());
    assert!(!auth_error.is_network_error());

    let api_error = CloudError::api("Test API error");
    assert!(!api_error.is_auth_error());
    assert!(!api_error.is_network_error());

    let config_error = CloudError::config("Test config error");
    assert!(!config_error.is_auth_error());
    assert!(!config_error.is_network_error());
}

#[test]
fn test_cloud_error_display() {
    let auth_error = CloudError::auth("Invalid token");
    assert_eq!(
        auth_error.to_string(),
        "Authentication error: Invalid token"
    );

    let api_error = CloudError::api("Server error");
    assert_eq!(api_error.to_string(), "API error: Server error");

    let config_error = CloudError::config("Missing config");
    assert_eq!(
        config_error.to_string(),
        "Configuration error: Missing config"
    );
}

#[tokio::test]
async fn test_init_function() {
    // Test the init function works
    let result = orkee_cloud::init().await;
    assert!(result.is_ok());

    let client = result.unwrap();
    assert!(!client.is_authenticated());
}

#[tokio::test]
async fn test_config_builder() {
    let result = orkee_cloud::CloudConfigBuilder::new()
        .api_url("https://api.test.com".to_string())
        .token("test_token".to_string())
        .build()
        .await;
    assert!(result.is_ok());
}

#[test]
fn test_api_response_parsing() {
    use orkee_cloud::orkee_api::{ApiError, ApiResponse};

    // Test successful response
    let success_response: ApiResponse<String> = ApiResponse {
        success: true,
        data: Some("test data".to_string()),
        error: None,
    };

    assert!(success_response.is_success());
    let result = success_response.into_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test data");

    // Test error response
    let error_response: ApiResponse<String> = ApiResponse {
        success: false,
        data: None,
        error: Some(ApiError {
            error: "test_error".to_string(),
            message: "Test error message".to_string(),
            details: None,
        }),
    };

    assert!(!error_response.is_success());
    let result = error_response.into_result();
    assert!(result.is_err());

    match result {
        Err(CloudError::Api(msg)) => {
            assert!(msg.contains("test_error"));
            assert!(msg.contains("Test error message"));
        }
        _ => panic!("Expected API error"),
    }
}

#[test]
fn test_token_info_expiry() {
    use chrono::{Duration, Utc};
    use orkee_cloud::TokenInfo;

    // Test expired token
    let expired_token = TokenInfo {
        token: "test".to_string(),
        expires_at: Utc::now() - Duration::minutes(10),
        user_email: "test@example.com".to_string(),
        user_name: "Test User".to_string(),
        user_id: "123".to_string(),
    };

    assert!(expired_token.is_expired());
    assert!(!expired_token.is_valid());

    // Test valid token
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
fn test_callback_server_url_parsing() {
    use orkee_cloud::CallbackServer;

    // Test URL parsing for OAuth callback
    let request =
        "GET /auth/callback?code=abc123&state=xyz789 HTTP/1.1\r\nHost: localhost:3737\r\n";
    let code = CallbackServer::extract_auth_code(request);
    assert_eq!(code, Some("abc123".to_string()));

    // Test URL without parameters
    let bad_request = "GET /auth/callback HTTP/1.1\r\nHost: localhost:3737\r\n";
    let no_code = CallbackServer::extract_auth_code(bad_request);
    assert_eq!(no_code, None);
}

#[test]
fn test_cloud_project_conversion() {
    use chrono::Utc;
    use orkee_cloud::orkee_api::CloudProject;

    let cloud_project = CloudProject {
        id: "test-123".to_string(),
        name: "Test Project".to_string(),
        path: "/tmp/test".to_string(),
        description: Some("Test description".to_string()),
        setup_script: None,
        dev_script: None,
        cleanup_script: None,
        tags: vec![],
        status: "active".to_string(),
        priority: "medium".to_string(),
        rank: None,
        task_source: None,
        mcp_servers: std::collections::HashMap::new(),
        git_repository: None,
        manual_tasks: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_sync: None,
    };

    assert_eq!(cloud_project.id, "test-123");
    assert_eq!(cloud_project.name, "Test Project");
    assert_eq!(cloud_project.path, "/tmp/test");
    assert!(cloud_project.description.is_some());
    assert!(cloud_project.last_sync.is_none());
}

#[test]
fn test_auth_manager_config_path() {
    use orkee_cloud::auth::AuthManager;

    // Test that AuthManager can be created (basic constructor test)
    let auth_manager = AuthManager::new();
    assert!(auth_manager.is_ok());

    let manager = auth_manager.unwrap();
    assert!(!manager.is_authenticated()); // Should start unauthenticated
    assert!(manager.user_info().is_none()); // No user info initially
}

#[test]
fn test_encryption_manager_placeholder() {
    use orkee_cloud::encryption::EncryptionManager;

    // Test placeholder encryption manager
    let manager = EncryptionManager::new().unwrap();
    let test_data = b"test encryption data";

    // In Phase 3, encryption is a no-op (placeholder)
    let encrypted = manager.encrypt(test_data).unwrap();
    let decrypted = manager.decrypt(&encrypted).unwrap();

    assert_eq!(encrypted, test_data); // Should be unchanged in placeholder
    assert_eq!(decrypted, test_data);
}

#[tokio::test]
async fn test_cloud_client_without_authentication() {
    let client = CloudClient::new("https://api.test.com".to_string())
        .await
        .unwrap();

    // Test that we can get status without authentication
    let status = client.get_status().await;
    assert!(status.is_ok());

    let status = status.unwrap();
    assert!(!status.authenticated);

    // Test that API calls requiring authentication fail appropriately
    let projects = client.list_projects().await;
    assert!(projects.is_err());

    let usage = client.get_usage().await;
    assert!(usage.is_err());
}
