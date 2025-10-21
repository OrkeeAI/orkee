// ABOUTME: Tests for user storage layer
// ABOUTME: Verifies encryption of API keys including ai_gateway_key

#[cfg(test)]
mod tests {
    use super::super::storage::UserStorage;
    use super::super::types::UserUpdateInput;
    use crate::security::ApiKeyEncryption;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                name TEXT NOT NULL,
                avatar_url TEXT,
                default_agent_id TEXT,
                theme TEXT,
                openai_api_key TEXT,
                anthropic_api_key TEXT,
                google_api_key TEXT,
                xai_api_key TEXT,
                ai_gateway_enabled INTEGER DEFAULT 0,
                ai_gateway_url TEXT,
                ai_gateway_key TEXT,
                preferences TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
                last_login_at TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, name)
            VALUES ('test-user', 'test@example.com', 'Test User')
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_ai_gateway_key_is_encrypted_on_save() {
        let pool = setup_test_db().await;
        let storage = UserStorage::new(pool.clone()).unwrap();

        let plaintext_key = "gateway-secret-key-12345";
        let input = UserUpdateInput {
            openai_api_key: None,
            anthropic_api_key: None,
            google_api_key: None,
            xai_api_key: None,
            ai_gateway_enabled: Some(true),
            ai_gateway_url: Some("https://gateway.example.com".to_string()),
            ai_gateway_key: Some(plaintext_key.to_string()),
        };

        storage
            .update_credentials("test-user", input)
            .await
            .unwrap();

        let raw_value: Option<String> = sqlx::query_scalar(
            "SELECT ai_gateway_key FROM users WHERE id = 'test-user'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(raw_value.is_some());
        let encrypted = raw_value.unwrap();

        assert_ne!(encrypted, plaintext_key, "Key should be encrypted in DB");
        assert!(
            ApiKeyEncryption::is_encrypted(&encrypted),
            "Stored value should be detected as encrypted"
        );
    }

    #[tokio::test]
    async fn test_ai_gateway_key_is_decrypted_on_read() {
        let pool = setup_test_db().await;
        let storage = UserStorage::new(pool.clone()).unwrap();

        let plaintext_key = "gateway-secret-key-12345";
        let input = UserUpdateInput {
            openai_api_key: None,
            anthropic_api_key: None,
            google_api_key: None,
            xai_api_key: None,
            ai_gateway_enabled: Some(true),
            ai_gateway_url: Some("https://gateway.example.com".to_string()),
            ai_gateway_key: Some(plaintext_key.to_string()),
        };

        storage
            .update_credentials("test-user", input)
            .await
            .unwrap();

        let user = storage.get_user("test-user").await.unwrap();

        assert_eq!(
            user.ai_gateway_key.as_deref(),
            Some(plaintext_key),
            "Key should be decrypted when reading user"
        );
    }

    #[tokio::test]
    async fn test_ai_gateway_key_roundtrip() {
        let pool = setup_test_db().await;
        let storage = UserStorage::new(pool).unwrap();

        let original_key = "my-gateway-key-567890";
        let input = UserUpdateInput {
            openai_api_key: None,
            anthropic_api_key: None,
            google_api_key: None,
            xai_api_key: None,
            ai_gateway_enabled: Some(true),
            ai_gateway_url: Some("https://gateway.example.com".to_string()),
            ai_gateway_key: Some(original_key.to_string()),
        };

        let updated_user = storage
            .update_credentials("test-user", input)
            .await
            .unwrap();

        assert_eq!(
            updated_user.ai_gateway_key.as_deref(),
            Some(original_key),
            "Roundtrip should return original key"
        );
    }

    #[tokio::test]
    async fn test_all_api_keys_are_encrypted() {
        let pool = setup_test_db().await;
        let storage = UserStorage::new(pool.clone()).unwrap();

        let input = UserUpdateInput {
            openai_api_key: Some("sk-openai-test".to_string()),
            anthropic_api_key: Some("sk-ant-test".to_string()),
            google_api_key: Some("goog-test".to_string()),
            xai_api_key: Some("xai-test".to_string()),
            ai_gateway_enabled: Some(true),
            ai_gateway_url: Some("https://gateway.example.com".to_string()),
            ai_gateway_key: Some("gateway-test".to_string()),
        };

        storage
            .update_credentials("test-user", input)
            .await
            .unwrap();

        let row: (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = sqlx::query_as(
            r#"
            SELECT openai_api_key, anthropic_api_key, google_api_key, xai_api_key, ai_gateway_key
            FROM users WHERE id = 'test-user'
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(
            ApiKeyEncryption::is_encrypted(&row.0.unwrap()),
            "OpenAI key should be encrypted"
        );
        assert!(
            ApiKeyEncryption::is_encrypted(&row.1.unwrap()),
            "Anthropic key should be encrypted"
        );
        assert!(
            ApiKeyEncryption::is_encrypted(&row.2.unwrap()),
            "Google key should be encrypted"
        );
        assert!(
            ApiKeyEncryption::is_encrypted(&row.3.unwrap()),
            "xAI key should be encrypted"
        );
        assert!(
            ApiKeyEncryption::is_encrypted(&row.4.unwrap()),
            "AI Gateway key should be encrypted"
        );
    }

    #[tokio::test]
    async fn test_empty_ai_gateway_key_not_encrypted() {
        let pool = setup_test_db().await;
        let storage = UserStorage::new(pool.clone()).unwrap();

        let input = UserUpdateInput {
            openai_api_key: None,
            anthropic_api_key: None,
            google_api_key: None,
            xai_api_key: None,
            ai_gateway_enabled: Some(true),
            ai_gateway_url: Some("https://gateway.example.com".to_string()),
            ai_gateway_key: Some("".to_string()),
        };

        storage
            .update_credentials("test-user", input)
            .await
            .unwrap();

        let raw_value: Option<String> = sqlx::query_scalar(
            "SELECT ai_gateway_key FROM users WHERE id = 'test-user'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            raw_value.as_deref(),
            Some(""),
            "Empty string should remain empty"
        );
    }

    #[tokio::test]
    async fn test_backward_compatibility_with_plaintext_keys() {
        let pool = setup_test_db().await;
        let storage = UserStorage::new(pool.clone()).unwrap();

        let plaintext_key = "legacy-plaintext-key";
        sqlx::query("UPDATE users SET ai_gateway_key = ? WHERE id = 'test-user'")
            .bind(plaintext_key)
            .execute(&pool)
            .await
            .unwrap();

        let user = storage.get_user("test-user").await.unwrap();

        assert_eq!(
            user.ai_gateway_key.as_deref(),
            Some(plaintext_key),
            "Should handle legacy plaintext keys gracefully"
        );
    }

    #[tokio::test]
    async fn test_null_ai_gateway_key() {
        let pool = setup_test_db().await;
        let storage = UserStorage::new(pool).unwrap();

        let user = storage.get_user("test-user").await.unwrap();

        assert!(
            user.ai_gateway_key.is_none(),
            "Should handle NULL ai_gateway_key"
        );
    }
}
