// ABOUTME: Security tests for model preferences
// ABOUTME: Tests SQL injection prevention and input validation

use orkee_storage::model_preferences::{
    ModelPreferences, ModelPreferencesStorage, UpdateTaskModelRequest,
};
use sqlx::SqlitePool;

#[sqlx::test]
async fn test_update_task_model_rejects_sql_injection_attempts(pool: SqlitePool) {
    let storage = ModelPreferencesStorage::new(pool.clone());
    let user_id = "default-user"; // Use default user from migration

    // Create default preferences first
    storage.get_preferences(user_id).await.unwrap();

    // Test 1: Attempt SQL injection via task_type (should be rejected by match)
    let malicious_task_type = "chat'; DROP TABLE model_preferences; --";
    let request = UpdateTaskModelRequest {
        model: "claude-sonnet-4-5".to_string(),
        provider: "anthropic".to_string(),
    };

    let result = storage
        .update_task_model(user_id, malicious_task_type, &request)
        .await;

    // Should fail with InvalidInput error
    assert!(result.is_err());
    match result.unwrap_err() {
        orkee_storage::StorageError::InvalidInput(msg) => {
            assert!(msg.contains("Invalid task type"));
        }
        _ => panic!("Expected InvalidInput error"),
    }

    // Verify table still exists and data is intact
    let prefs = storage.get_preferences(user_id).await.unwrap();
    assert_eq!(prefs.user_id, user_id);

    // Test 2: Verify no SQL injection via string formatting
    // Even if a malicious task_type somehow passed validation,
    // the new match-based approach prevents SQL injection
    let tasks = vec!["chat", "prd_generation", "insight_extraction"];

    for task in tasks {
        let result = storage.update_task_model(user_id, task, &request).await;
        assert!(result.is_ok(), "Failed to update task: {}", task);
    }
}

#[sqlx::test]
async fn test_all_task_types_have_safe_queries(pool: SqlitePool) {
    let storage = ModelPreferencesStorage::new(pool.clone());
    let user_id = "default-user"; // Use default user from migration

    // Create default preferences
    storage.get_preferences(user_id).await.unwrap();

    // Test all valid task types work correctly with safe queries
    let valid_tasks = vec![
        "chat",
        "prd_generation",
        "prd_analysis",
        "insight_extraction",
        "spec_generation",
        "task_suggestions",
        "task_analysis",
        "spec_refinement",
        "research_generation",
        "markdown_generation",
    ];

    for task_type in valid_tasks {
        let request = UpdateTaskModelRequest {
            model: "claude-sonnet-4-5".to_string(),
            provider: "anthropic".to_string(),
        };

        let result = storage
            .update_task_model(user_id, task_type, &request)
            .await;

        assert!(result.is_ok(), "Failed to update task_type: {}", task_type);
    }

    // Verify updates were applied
    let prefs = storage.get_preferences(user_id).await.unwrap();
    assert_eq!(prefs.chat_model, "claude-sonnet-4-5");
    assert_eq!(prefs.chat_provider, "anthropic");
}

#[sqlx::test]
async fn test_invalid_task_type_rejected(pool: SqlitePool) {
    let storage = ModelPreferencesStorage::new(pool.clone());
    let user_id = "default-user"; // Use default user from migration

    storage.get_preferences(user_id).await.unwrap();

    // Test invalid task types are rejected
    let invalid_tasks = vec![
        "invalid_task",
        "DROP TABLE",
        "",
        "chat OR 1=1",
        "chat'; --",
        "../../../etc/passwd",
        "chat\0null",
    ];

    for task_type in invalid_tasks {
        let request = UpdateTaskModelRequest {
            model: "claude-sonnet-4-5".to_string(),
            provider: "anthropic".to_string(),
        };

        let result = storage
            .update_task_model(user_id, task_type, &request)
            .await;

        assert!(
            result.is_err(),
            "Should reject invalid task_type: {}",
            task_type
        );

        match result.unwrap_err() {
            orkee_storage::StorageError::InvalidInput(msg) => {
                assert!(msg.contains("Invalid task type"));
            }
            _ => panic!("Expected InvalidInput error for task_type: {}", task_type),
        }
    }
}

#[sqlx::test]
async fn test_parameterized_queries_prevent_injection(pool: SqlitePool) {
    let storage = ModelPreferencesStorage::new(pool.clone());
    let user_id = "default-user"; // Use default user from migration

    storage.get_preferences(user_id).await.unwrap();

    // Attempt SQL injection via model and provider fields
    // These should be safely handled as parameters, not concatenated
    let malicious_inputs = vec![
        ("'; DROP TABLE model_preferences; --", "anthropic"),
        ("claude-sonnet-4-5", "'; DELETE FROM model_preferences; --"),
        ("claude' OR '1'='1", "anthropic"),
        ("claude-sonnet-4-5", "anthropic' OR '1'='1"),
    ];

    for (malicious_model, malicious_provider) in malicious_inputs {
        let request = UpdateTaskModelRequest {
            model: malicious_model.to_string(),
            provider: malicious_provider.to_string(),
        };

        // This should either fail validation or safely store the literal string
        let _result = storage.update_task_model(user_id, "chat", &request).await;

        // Verify table still exists and data is intact
        let prefs = storage.get_preferences(user_id).await.unwrap();
        assert_eq!(prefs.user_id, user_id);
    }
}

#[sqlx::test]
async fn test_update_preferences_safe_with_special_chars(pool: SqlitePool) {
    let storage = ModelPreferencesStorage::new(pool.clone());
    let user_id = "default-user"; // Use default user from migration

    // Create preferences with special characters that could be dangerous
    // if not properly parameterized
    let prefs = ModelPreferences {
        user_id: user_id.to_string(),
        chat_model: "model-with-'quotes'".to_string(),
        chat_provider: "anthropic".to_string(),
        prd_generation_model: "model-with-\"double-quotes\"".to_string(),
        prd_generation_provider: "openai".to_string(),
        prd_analysis_model: "model-with-;semicolon".to_string(),
        prd_analysis_provider: "anthropic".to_string(),
        insight_extraction_model: "model-with--comment".to_string(),
        insight_extraction_provider: "anthropic".to_string(),
        spec_generation_model: "claude-sonnet-4-5".to_string(),
        spec_generation_provider: "anthropic".to_string(),
        task_suggestions_model: "claude-sonnet-4-5".to_string(),
        task_suggestions_provider: "anthropic".to_string(),
        task_analysis_model: "claude-sonnet-4-5".to_string(),
        task_analysis_provider: "anthropic".to_string(),
        spec_refinement_model: "claude-sonnet-4-5".to_string(),
        spec_refinement_provider: "anthropic".to_string(),
        research_generation_model: "claude-sonnet-4-5".to_string(),
        research_generation_provider: "anthropic".to_string(),
        markdown_generation_model: "claude-sonnet-4-5".to_string(),
        markdown_generation_provider: "anthropic".to_string(),
        updated_at: String::new(),
    };

    // Should safely store these values as literals
    let result = storage.update_preferences(&prefs).await;
    assert!(result.is_ok());

    // Retrieve and verify special characters are preserved as-is
    let retrieved = storage.get_preferences(user_id).await.unwrap();
    assert_eq!(retrieved.chat_model, "model-with-'quotes'");
    assert_eq!(
        retrieved.prd_generation_model,
        "model-with-\"double-quotes\""
    );
    assert_eq!(retrieved.prd_analysis_model, "model-with-;semicolon");
}

#[sqlx::test]
async fn test_concurrent_updates_maintain_integrity(pool: SqlitePool) {
    let storage = ModelPreferencesStorage::new(pool.clone());
    let user_id = "default-user"; // Use default user from migration

    storage.get_preferences(user_id).await.unwrap();

    // Simulate concurrent updates to different tasks
    let tasks = vec![
        ("chat", "claude-haiku-4-5"),
        ("prd_generation", "claude-sonnet-4-5"),
        ("insight_extraction", "claude-haiku-4-5"),
    ];

    let mut handles = vec![];

    for (task, model) in tasks {
        let storage_clone = ModelPreferencesStorage::new(pool.clone());
        let user_id = user_id.to_string();
        let task = task.to_string();
        let model = model.to_string();

        let handle = tokio::spawn(async move {
            let request = UpdateTaskModelRequest {
                model,
                provider: "anthropic".to_string(),
            };

            storage_clone
                .update_task_model(&user_id, &task, &request)
                .await
        });

        handles.push(handle);
    }

    // Wait for all updates
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all updates succeeded
    let prefs = storage.get_preferences(user_id).await.unwrap();
    assert_eq!(prefs.chat_model, "claude-haiku-4-5");
    assert_eq!(prefs.prd_generation_model, "claude-sonnet-4-5");
    assert_eq!(prefs.insight_extraction_model, "claude-haiku-4-5");
}

#[sqlx::test]
async fn test_unicode_and_special_chars_handled_safely(pool: SqlitePool) {
    let storage = ModelPreferencesStorage::new(pool.clone());
    let user_id = "default-user"; // Use default user from migration

    storage.get_preferences(user_id).await.unwrap();

    // Test various unicode and special characters
    let test_cases = vec![
        "model-émojis-🎉",
        "model-中文-support",
        "model-with\nnewline",
        "model-with\ttab",
        "model-with\\backslash",
    ];

    for test_model in test_cases {
        let request = UpdateTaskModelRequest {
            model: test_model.to_string(),
            provider: "anthropic".to_string(),
        };

        // Should safely handle unicode/special chars
        let _result = storage.update_task_model(user_id, "chat", &request).await;

        // Verify data integrity
        let prefs = storage.get_preferences(user_id).await.unwrap();
        assert_eq!(prefs.user_id, user_id);
    }
}
