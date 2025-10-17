// ABOUTME: PostHog-specific telemetry integration
// ABOUTME: Formats events for PostHog's API and handles authentication

use serde::Serialize;
use serde_json::Value;

// PostHog Project API Key (phc_...) - Loaded from Environment
// This is the PUBLIC project key that's safe to expose in client-side code.
// It only allows sending events (write-only), not reading data or admin operations.
// Get this from: https://app.posthog.com/project/settings
//
// The key is loaded in this priority order:
// 1. Compile-time: POSTHOG_API_KEY environment variable during build
// 2. Runtime: POSTHOG_API_KEY environment variable at execution
//
// If no key is available, telemetry will be disabled gracefully.
//
// NOTE: Do NOT use your Personal API Key (phx_...) here - that's for admin operations only!
pub fn get_posthog_api_key() -> Option<String> {
    // Try compile-time environment variable first (set during build)
    let key = option_env!("POSTHOG_API_KEY")
        .map(String::from)
        // Fall back to runtime environment variable
        .or_else(|| std::env::var("POSTHOG_API_KEY").ok())?;

    // Validate PostHog public key format (phc_...)
    // PostHog project API keys start with "phc_" and are at least 20 characters
    if key.starts_with("phc_") && key.len() > 10 {
        Some(key)
    } else {
        tracing::warn!(
            "Invalid PostHog API key format. Expected 'phc_...' (project key), got prefix: {}",
            key.chars().take(5).collect::<String>()
        );
        None
    }
}

// PostHog event structure
#[derive(Debug, Serialize)]
pub struct PostHogEvent {
    pub api_key: String,
    pub event: String,
    pub properties: PostHogProperties,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinct_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PostHogProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinct_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub app_version: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    #[serde(flatten)]
    pub custom_properties: Option<Value>,
}

// PostHog batch structure
#[derive(Debug, Serialize)]
pub struct PostHogBatch {
    pub api_key: String,
    pub batch: Vec<PostHogEvent>,
}

impl From<super::events::TelemetryEvent> for PostHogEvent {
    fn from(event: super::events::TelemetryEvent) -> Self {
        let event_name = match event.event_type {
            super::events::EventType::Usage => event.event_name,
            super::events::EventType::Error => format!("error_{}", event.event_name),
            super::events::EventType::Performance => format!("perf_{}", event.event_name),
        };

        // Extract error data if present
        let (error_message, stack_trace) = if let Some(ref data) = event.event_data {
            (
                data.get("message")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                data.get("stack_trace")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            )
        } else {
            (None, None)
        };

        // Determine distinct_id based on anonymity settings
        let distinct_id = if event.anonymous {
            event.machine_id.clone()
        } else {
            event.user_id.clone().or(event.machine_id.clone())
        };

        PostHogEvent {
            api_key: get_posthog_api_key().unwrap_or_default(),
            event: event_name,
            properties: PostHogProperties {
                distinct_id: distinct_id.clone(),
                machine_id: event.machine_id,
                session_id: event.session_id,
                app_version: event.version,
                platform: event.platform,
                error_message,
                stack_trace,
                custom_properties: event.event_data,
            },
            timestamp: event.timestamp.to_rfc3339(),
            distinct_id,
        }
    }
}

// Convert multiple telemetry events to PostHog batch
pub fn create_posthog_batch(events: Vec<super::events::TelemetryEvent>) -> PostHogBatch {
    PostHogBatch {
        api_key: get_posthog_api_key().unwrap_or_default(),
        batch: events.into_iter().map(PostHogEvent::from).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_valid_posthog_api_key() {
        env::set_var("POSTHOG_API_KEY", "phc_1234567890abcdef");
        let key = get_posthog_api_key();
        assert!(key.is_some());
        assert_eq!(key.unwrap(), "phc_1234567890abcdef");
        env::remove_var("POSTHOG_API_KEY");
    }

    #[test]
    #[serial]
    fn test_invalid_api_key_wrong_prefix() {
        env::set_var("POSTHOG_API_KEY", "phx_1234567890abcdef");
        let key = get_posthog_api_key();
        assert!(key.is_none(), "Should reject API key with wrong prefix (phx_)");
        env::remove_var("POSTHOG_API_KEY");
    }

    #[test]
    #[serial]
    fn test_invalid_api_key_too_short() {
        env::set_var("POSTHOG_API_KEY", "phc_12345");
        let key = get_posthog_api_key();
        assert!(key.is_none(), "Should reject API key that is too short");
        env::remove_var("POSTHOG_API_KEY");
    }

    #[test]
    #[serial]
    fn test_invalid_api_key_no_prefix() {
        env::set_var("POSTHOG_API_KEY", "1234567890abcdef");
        let key = get_posthog_api_key();
        assert!(key.is_none(), "Should reject API key without phc_ prefix");
        env::remove_var("POSTHOG_API_KEY");
    }

    #[test]
    #[serial]
    fn test_no_api_key_set() {
        env::remove_var("POSTHOG_API_KEY");
        let key = get_posthog_api_key();
        assert!(key.is_none(), "Should return None when no API key is set");
    }

    #[test]
    #[serial]
    fn test_empty_api_key() {
        env::set_var("POSTHOG_API_KEY", "");
        let key = get_posthog_api_key();
        assert!(key.is_none(), "Should reject empty API key");
        env::remove_var("POSTHOG_API_KEY");
    }
}
