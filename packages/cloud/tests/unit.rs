//! Unit tests for Orkee Cloud client components

#[cfg(test)]
mod cloud_unit_tests {
    use chrono::Utc;
    use orkee_cloud::{
        api::{ApiError, ApiResponse, CloudProject},
        CloudError,
    };

    #[test]
    fn test_cloud_error_creation() {
        let auth_error = CloudError::auth("Test auth error");
        assert!(matches!(auth_error, CloudError::Authentication(_)));
        assert!(auth_error.is_auth_error());

        let api_error = CloudError::api("Test API error");
        assert!(matches!(api_error, CloudError::Api(_)));
        assert!(!api_error.is_auth_error());

        let config_error = CloudError::config("Test config error");
        assert!(matches!(config_error, CloudError::Configuration(_)));
        assert!(!config_error.is_auth_error());
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse {
            success: true,
            data: Some("test data".to_string()),
            error: None,
        };

        assert!(response.is_success());
        let result = response.into_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test data");
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                error: "test_error".to_string(),
                message: "Test message".to_string(),
                details: None,
            }),
        };

        assert!(!response.is_success());
        let result = response.into_result();
        assert!(result.is_err());

        match result.unwrap_err() {
            CloudError::Api(msg) => {
                assert!(msg.contains("test_error"));
                assert!(msg.contains("Test message"));
            }
            _ => panic!("Expected API error"),
        }
    }

    #[test]
    fn test_cloud_project_struct() {
        let project = CloudProject {
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

        assert_eq!(project.id, "test-123");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.path, "/tmp/test");
        assert!(project.description.is_some());
        assert!(project.last_sync.is_none());
        assert_eq!(project.status, "active");
        assert_eq!(project.priority, "medium");
    }

    #[test]
    fn test_error_display() {
        let error = CloudError::auth("Invalid credentials");
        let display = format!("{}", error);
        assert_eq!(display, "Authentication error: Invalid credentials");

        let error = CloudError::api("Server unavailable");
        let display = format!("{}", error);
        assert_eq!(display, "API error: Server unavailable");
    }
}
