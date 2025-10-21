// ABOUTME: Tests for API key masking in responses
// ABOUTME: Ensures sensitive API keys are never exposed in API responses

#[cfg(test)]
mod tests {
    use super::super::types::{MaskedUser, User};
    use chrono::Utc;

    fn create_test_user_with_api_keys() -> User {
        User {
            id: "test-user-123".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            default_agent_id: None,
            theme: Some("dark".to_string()),
            openai_api_key: Some("sk-openai-super-secret-key-12345".to_string()),
            anthropic_api_key: Some("sk-ant-super-secret-key-67890".to_string()),
            google_api_key: Some("goog-super-secret-key-abcdef".to_string()),
            xai_api_key: Some("xai-super-secret-key-xyz789".to_string()),
            ai_gateway_enabled: true,
            ai_gateway_url: Some("https://gateway.example.com".to_string()),
            ai_gateway_key: Some("gateway-super-secret-key-qwerty".to_string()),
            preferences: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        }
    }

    #[test]
    fn test_masked_user_does_not_expose_api_keys() {
        let user = create_test_user_with_api_keys();
        let masked: MaskedUser = user.into();

        // Serialize to JSON to ensure keys aren't in the serialized output
        let json = serde_json::to_string(&masked).unwrap();

        // API keys should NOT appear in JSON
        assert!(!json.contains("sk-openai-super-secret"));
        assert!(!json.contains("sk-ant-super-secret"));
        assert!(!json.contains("goog-super-secret"));
        assert!(!json.contains("xai-super-secret"));
        assert!(!json.contains("gateway-super-secret"));

        // Should not contain any common API key prefixes
        assert!(!json.contains("sk-openai-"));
        assert!(!json.contains("sk-ant-"));
        assert!(!json.contains("goog-"));
        assert!(!json.contains("xai-"));
    }

    #[test]
    fn test_masked_user_shows_presence_flags_only() {
        let user = create_test_user_with_api_keys();
        let masked: MaskedUser = user.into();

        // Should show that keys are present, but not the actual keys
        assert!(masked.has_openai_api_key);
        assert!(masked.has_anthropic_api_key);
        assert!(masked.has_google_api_key);
        assert!(masked.has_xai_api_key);
        assert!(masked.has_ai_gateway_key);
    }

    #[test]
    fn test_masked_user_preserves_non_sensitive_fields() {
        let user = create_test_user_with_api_keys();
        let masked: MaskedUser = user.clone().into();

        // Non-sensitive fields should be preserved
        assert_eq!(masked.id, user.id);
        assert_eq!(masked.email, user.email);
        assert_eq!(masked.name, user.name);
        assert_eq!(masked.avatar_url, user.avatar_url);
        assert_eq!(masked.default_agent_id, user.default_agent_id);
        assert_eq!(masked.theme, user.theme);
        assert_eq!(masked.ai_gateway_enabled, user.ai_gateway_enabled);
        assert_eq!(masked.ai_gateway_url, user.ai_gateway_url);
        assert_eq!(masked.preferences, user.preferences);
        assert_eq!(masked.created_at, user.created_at);
        assert_eq!(masked.updated_at, user.updated_at);
        assert_eq!(masked.last_login_at, user.last_login_at);
    }

    #[test]
    fn test_masked_user_with_no_api_keys() {
        let mut user = create_test_user_with_api_keys();
        user.openai_api_key = None;
        user.anthropic_api_key = None;
        user.google_api_key = None;
        user.xai_api_key = None;
        user.ai_gateway_key = None;

        let masked: MaskedUser = user.into();

        // Should show false for all keys
        assert!(!masked.has_openai_api_key);
        assert!(!masked.has_anthropic_api_key);
        assert!(!masked.has_google_api_key);
        assert!(!masked.has_xai_api_key);
        assert!(!masked.has_ai_gateway_key);
    }

    #[test]
    fn test_masked_user_with_partial_api_keys() {
        let mut user = create_test_user_with_api_keys();
        user.openai_api_key = Some("sk-openai-key".to_string());
        user.anthropic_api_key = None;
        user.google_api_key = Some("goog-key".to_string());
        user.xai_api_key = None;
        user.ai_gateway_key = None;

        let masked: MaskedUser = user.into();

        // Should show correct presence flags
        assert!(masked.has_openai_api_key);
        assert!(!masked.has_anthropic_api_key);
        assert!(masked.has_google_api_key);
        assert!(!masked.has_xai_api_key);
        assert!(!masked.has_ai_gateway_key);

        // Serialize and verify no keys are exposed
        let json = serde_json::to_string(&masked).unwrap();
        assert!(!json.contains("sk-openai-key"));
        assert!(!json.contains("goog-key"));
    }

    #[test]
    fn test_json_serialization_never_includes_api_keys() {
        let user = create_test_user_with_api_keys();
        let masked: MaskedUser = user.into();

        // Serialize to JSON (this is what API responses would return)
        let json = serde_json::to_value(&masked).unwrap();

        // Verify JSON object doesn't have key fields
        assert!(json.get("openai_api_key").is_none());
        assert!(json.get("anthropic_api_key").is_none());
        assert!(json.get("google_api_key").is_none());
        assert!(json.get("xai_api_key").is_none());
        assert!(json.get("ai_gateway_key").is_none());

        // Verify JSON has presence flags instead
        assert_eq!(
            json.get("has_openai_api_key").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            json.get("has_anthropic_api_key").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            json.get("has_google_api_key").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            json.get("has_xai_api_key").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            json.get("has_ai_gateway_key").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_debug_output_does_not_expose_keys() {
        let user = create_test_user_with_api_keys();
        let masked: MaskedUser = user.into();

        // Debug output should not contain actual API keys
        let debug_output = format!("{:?}", masked);

        assert!(!debug_output.contains("sk-openai-super-secret"));
        assert!(!debug_output.contains("sk-ant-super-secret"));
        assert!(!debug_output.contains("goog-super-secret"));
        assert!(!debug_output.contains("xai-super-secret"));
        assert!(!debug_output.contains("gateway-super-secret"));
    }

    #[test]
    fn test_empty_string_api_keys_are_masked_correctly() {
        let mut user = create_test_user_with_api_keys();
        user.openai_api_key = Some("".to_string());
        user.anthropic_api_key = Some("".to_string());

        let masked: MaskedUser = user.into();

        // Empty strings should still show as "has key" = true
        // This matches the behavior where Some("") is different from None
        assert!(masked.has_openai_api_key);
        assert!(masked.has_anthropic_api_key);
    }

    #[test]
    fn test_masking_works_with_realistic_api_key_formats() {
        let mut user = create_test_user_with_api_keys();

        // Test with realistic API key formats
        user.openai_api_key = Some("sk-proj-1234567890abcdefghijklmnopqrstuvwxyz".to_string());
        user.anthropic_api_key =
            Some("sk-ant-api03-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx-yyyyyyyyyyyyyyyyyyyy".to_string());
        user.google_api_key = Some("AIzaSyD1234567890abcdefghijklmnopqrstuv".to_string());

        let masked: MaskedUser = user.into();
        let json = serde_json::to_string(&masked).unwrap();

        // Verify no parts of actual keys appear
        assert!(!json.contains("sk-proj-"));
        assert!(!json.contains("1234567890abcdef"));
        assert!(!json.contains("sk-ant-api03-"));
        assert!(!json.contains("AIzaSyD"));
    }
}
