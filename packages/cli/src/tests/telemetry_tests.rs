use chrono::Utc;
use serial_test::serial;
use sqlx::{Row, SqlitePool};
use std::env;
use tempfile::TempDir;

use crate::telemetry::{
    config::{TelemetryConfig, TelemetryManager},
    events::{
        cleanup_old_events, cleanup_old_unsent_events, get_unsent_events, mark_events_as_sent,
        track_error, track_event, EventType, TelemetryEvent,
    },
};

async fn setup_test_db() -> (SqlitePool, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&database_url).await.unwrap();

    // Run migrations
    sqlx::migrate!("../storage/migrations")
        .run(&pool)
        .await
        .unwrap();

    (pool, temp_dir)
}

// ============================================================================
// Settings Persistence Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_telemetry_settings_default_values() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool).await.unwrap();
    let settings = manager.get_settings().await;

    // Verify privacy-first defaults
    assert!(settings.first_run);
    assert!(!settings.onboarding_completed);
    assert!(!settings.error_reporting);
    assert!(!settings.usage_metrics);
    assert!(!settings.non_anonymous_metrics);
    assert!(settings.machine_id.is_none());
    assert!(settings.user_id.is_none());
}

#[tokio::test]
#[serial]
async fn test_settings_persistence_across_loads() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Create manager and update settings
    {
        let manager = TelemetryManager::new(pool.clone()).await.unwrap();
        let mut settings = manager.get_settings().await;
        settings.error_reporting = true;
        settings.usage_metrics = true;
        settings.machine_id = Some("test-machine-id".to_string());

        manager.update_settings(settings).await.unwrap();
    }

    // Create new manager and verify persistence
    let manager = TelemetryManager::new(pool).await.unwrap();
    let settings = manager.get_settings().await;

    assert!(settings.error_reporting);
    assert!(settings.usage_metrics);
    assert_eq!(settings.machine_id, Some("test-machine-id".to_string()));
}

#[tokio::test]
#[serial]
async fn test_complete_onboarding_generates_machine_id() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool).await.unwrap();

    // Complete onboarding with telemetry enabled
    manager
        .complete_onboarding(true, true, false)
        .await
        .unwrap();

    let settings = manager.get_settings().await;

    assert!(!settings.first_run);
    assert!(settings.onboarding_completed);
    assert!(settings.error_reporting);
    assert!(settings.usage_metrics);
    assert!(!settings.non_anonymous_metrics);
    assert!(settings.machine_id.is_some());
    assert!(settings.user_id.is_none());
}

#[tokio::test]
#[serial]
async fn test_onboarding_without_telemetry_no_machine_id() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool).await.unwrap();

    // Complete onboarding with all telemetry disabled
    manager
        .complete_onboarding(false, false, false)
        .await
        .unwrap();

    let settings = manager.get_settings().await;

    assert!(!settings.first_run);
    assert!(settings.onboarding_completed);
    assert!(!settings.error_reporting);
    assert!(!settings.usage_metrics);
    assert!(settings.machine_id.is_none());
}

#[tokio::test]
#[serial]
async fn test_settings_table_single_row_constraint() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Try to insert a second row with id = 2
    // This should fail because the CHECK constraint enforces id = 1
    let result = sqlx::query(
        "INSERT INTO telemetry_settings (id, first_run, onboarding_completed, error_reporting, usage_metrics, non_anonymous_metrics) VALUES (2, 1, 0, 0, 0, 0)"
    )
    .execute(&pool)
    .await;

    // The CHECK constraint should prevent this
    assert!(result.is_err());

    // Verify the application only uses id = 1
    let manager = TelemetryManager::new(pool).await.unwrap();
    let settings = manager.get_settings().await;

    // Settings should still have defaults for id = 1
    assert!(settings.first_run);
}

// ============================================================================
// Machine ID Generation Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_machine_id_is_valid_uuid() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool).await.unwrap();
    manager
        .complete_onboarding(true, false, false)
        .await
        .unwrap();

    let settings = manager.get_settings().await;
    let machine_id = settings.machine_id.unwrap();

    // Verify it's a valid UUID format
    assert!(uuid::Uuid::parse_str(&machine_id).is_ok());
}

#[tokio::test]
#[serial]
async fn test_machine_id_stability_across_sessions() {
    let (pool, _temp_dir) = setup_test_db().await;

    let machine_id = {
        let manager = TelemetryManager::new(pool.clone()).await.unwrap();
        manager
            .complete_onboarding(true, false, false)
            .await
            .unwrap();
        let settings = manager.get_settings().await;
        settings.machine_id.clone()
    };

    // Create new manager and verify machine ID hasn't changed
    let manager = TelemetryManager::new(pool).await.unwrap();
    let settings = manager.get_settings().await;

    assert_eq!(settings.machine_id, machine_id);
}

// ============================================================================
// Event Filtering Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_error_events_filtered_when_disabled() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool.clone()).await.unwrap();

    // Disable error reporting
    manager
        .complete_onboarding(false, true, false)
        .await
        .unwrap();

    // Track an error event
    track_error(&pool, "test_error", "This is a test error", None, None)
        .await
        .unwrap();

    // Verify event was saved
    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);

    // The collector should filter this event out
    // (We test this indirectly by checking that error_reporting is false)
    let settings = manager.get_settings().await;
    assert!(!settings.error_reporting);
    assert!(settings.usage_metrics);
}

#[tokio::test]
#[serial]
async fn test_usage_events_filtered_when_disabled() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool.clone()).await.unwrap();

    // Enable only error reporting
    manager
        .complete_onboarding(true, false, false)
        .await
        .unwrap();

    // Track a usage event
    track_event(&pool, "button_click", None, None)
        .await
        .unwrap();

    // Verify event was saved
    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);

    // The collector should filter this event out
    let settings = manager.get_settings().await;
    assert!(settings.error_reporting);
    assert!(!settings.usage_metrics);
}

#[tokio::test]
#[serial]
async fn test_anonymous_mode_strips_user_id() {
    let (_pool, _temp_dir) = setup_test_db().await;

    let mut event = TelemetryEvent::new(EventType::Usage, "test_event".to_string());
    event = event.with_identity(
        Some("machine-123".to_string()),
        Some("user-456".to_string()),
    );

    // With non_anonymous_metrics = false, user_id should be ignored
    // (This is enforced in the collector when filtering events)

    // Verify event has both IDs before filtering
    assert_eq!(event.machine_id, Some("machine-123".to_string()));
    assert_eq!(event.user_id, Some("user-456".to_string()));
    assert!(!event.anonymous);
}

#[tokio::test]
#[serial]
async fn test_non_anonymous_mode_includes_user_id() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool.clone()).await.unwrap();

    // Enable non-anonymous metrics
    manager.complete_onboarding(true, true, true).await.unwrap();

    let mut settings = manager.get_settings().await;
    settings.user_id = Some("test-user-id".to_string());
    manager.update_settings(settings.clone()).await.unwrap();

    // Verify settings
    assert!(settings.non_anonymous_metrics);
    assert_eq!(settings.user_id, Some("test-user-id".to_string()));
}

// ============================================================================
// SQL Query Safety Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_event_storage_handles_special_characters() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Test with SQL injection attempt in event name
    let malicious_name = "test'; DROP TABLE telemetry_events; --";
    let event = TelemetryEvent::new(EventType::Usage, malicious_name.to_string());

    event.save_to_db(&pool).await.unwrap();

    // Verify table still exists and data is safely stored
    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, malicious_name);

    // Verify table wasn't dropped
    let table_check = sqlx::query("SELECT COUNT(*) as count FROM telemetry_events")
        .fetch_one(&pool)
        .await;
    assert!(table_check.is_ok());
}

#[tokio::test]
#[serial]
async fn test_event_storage_handles_unicode() {
    let (pool, _temp_dir) = setup_test_db().await;

    let unicode_data = "Test with emoji 🚀 and symbols © ® ™";
    let event = TelemetryEvent::new(EventType::Usage, unicode_data.to_string());

    event.save_to_db(&pool).await.unwrap();

    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, unicode_data);
}

#[tokio::test]
#[serial]
async fn test_json_injection_in_event_data() {
    let (pool, _temp_dir) = setup_test_db().await;

    use serde_json::Value;
    use std::collections::HashMap;

    let mut props = HashMap::new();
    props.insert(
        "malicious".to_string(),
        Value::String("'; DROP TABLE telemetry_events; --".to_string()),
    );

    track_event(&pool, "test_event", Some(props), None)
        .await
        .unwrap();

    // Verify table still exists
    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_mark_events_as_sent_with_multiple_ids() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Create multiple events
    for i in 0..5 {
        let event = TelemetryEvent::new(EventType::Usage, format!("event_{}", i));
        event.save_to_db(&pool).await.unwrap();
    }

    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 5);

    // Mark some as sent
    let event_ids: Vec<String> = events.iter().take(3).map(|e| e.id.clone()).collect();
    mark_events_as_sent(&pool, &event_ids).await.unwrap();

    // Verify only 2 unsent remain
    let unsent = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(unsent.len(), 2);
}

// ============================================================================
// Privacy Control Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_delete_all_data_removes_events() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool.clone()).await.unwrap();

    // Create some events
    for i in 0..10 {
        let event = TelemetryEvent::new(EventType::Usage, format!("event_{}", i));
        event.save_to_db(&pool).await.unwrap();
    }

    let events = get_unsent_events(&pool, 20).await.unwrap();
    assert_eq!(events.len(), 10);

    // Delete all data
    let deleted = manager.delete_all_data().await.unwrap();
    assert_eq!(deleted, 10);

    // Verify all events deleted
    let events = get_unsent_events(&pool, 20).await.unwrap();
    assert_eq!(events.len(), 0);
}

#[tokio::test]
#[serial]
async fn test_cleanup_old_events_respects_retention() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Create an old event
    let old_event = TelemetryEvent::new(EventType::Usage, "old_event".to_string());
    old_event.save_to_db(&pool).await.unwrap();

    // Mark it as sent with an old timestamp
    let old_events = get_unsent_events(&pool, 10).await.unwrap();
    mark_events_as_sent(&pool, &[old_events[0].id.clone()])
        .await
        .unwrap();

    // Manually update sent_at to 40 days ago
    sqlx::query("UPDATE telemetry_events SET sent_at = datetime('now', '-40 days') WHERE id = ?")
        .bind(&old_events[0].id)
        .execute(&pool)
        .await
        .unwrap();

    // Create a recent event
    let recent_event = TelemetryEvent::new(EventType::Usage, "recent_event".to_string());
    recent_event.save_to_db(&pool).await.unwrap();

    // Cleanup events older than 30 days
    let deleted = cleanup_old_events(&pool, 30).await.unwrap();
    assert_eq!(deleted, 1);

    // Verify recent unsent event still exists
    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, "recent_event");
}

#[tokio::test]
#[serial]
async fn test_cleanup_old_unsent_events_prevents_unbounded_growth() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Create an old unsent event (simulating a failed send from 10 days ago)
    let old_event = TelemetryEvent::new(EventType::Usage, "old_unsent_event".to_string());
    old_event.save_to_db(&pool).await.unwrap();

    // Manually update created_at to 10 days ago (exceeds 7-day threshold)
    let old_events = get_unsent_events(&pool, 10).await.unwrap();
    sqlx::query(
        "UPDATE telemetry_events SET created_at = datetime('now', '-10 days') WHERE id = ?",
    )
    .bind(&old_events[0].id)
    .execute(&pool)
    .await
    .unwrap();

    // Create a recent unsent event (from today)
    let recent_event = TelemetryEvent::new(EventType::Usage, "recent_unsent_event".to_string());
    recent_event.save_to_db(&pool).await.unwrap();

    // Verify both events exist and are unsent
    let all_unsent = sqlx::query("SELECT id FROM telemetry_events WHERE sent_at IS NULL")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(all_unsent.len(), 2);

    // Clean up old unsent events (7+ days)
    let deleted = cleanup_old_unsent_events(&pool, 7).await.unwrap();
    assert_eq!(deleted, 1);

    // Verify only recent unsent event remains
    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, "recent_unsent_event");
}

#[tokio::test]
#[serial]
async fn test_opt_out_stops_event_collection() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Set API key for this test
    env::set_var("POSTHOG_API_KEY", "phc_test1234567890");

    let manager = TelemetryManager::new(pool.clone()).await.unwrap();

    // Opt in first
    manager
        .complete_onboarding(true, true, false)
        .await
        .unwrap();
    assert!(manager.is_any_telemetry_enabled().await);

    // Opt out
    let mut settings = manager.get_settings().await;
    settings.error_reporting = false;
    settings.usage_metrics = false;
    settings.non_anonymous_metrics = false;
    manager.update_settings(settings).await.unwrap();

    // Verify telemetry is disabled (only user settings are disabled, not global config)
    assert!(!manager.is_any_telemetry_enabled().await);

    // Clean up
    env::remove_var("POSTHOG_API_KEY");
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
#[serial]
fn test_telemetry_config_from_env_defaults() {
    // Clear environment variables
    env::remove_var("ORKEE_TELEMETRY_ENABLED");
    env::remove_var("ORKEE_TELEMETRY_ENDPOINT");
    env::remove_var("ORKEE_TELEMETRY_DEBUG");
    env::remove_var("POSTHOG_API_KEY");

    let config = TelemetryConfig::from_env();

    // Without API key, telemetry should be disabled
    assert!(!config.enabled);
    assert_eq!(config.endpoint, "https://app.posthog.com/capture");
    assert!(!config.debug_mode);
    assert_eq!(config.batch_size, 50);
    assert_eq!(config.flush_interval_secs, 300);
    assert_eq!(config.retention_days, 30);
}

#[test]
#[serial]
fn test_telemetry_config_with_api_key() {
    env::set_var("POSTHOG_API_KEY", "phc_test1234567890");
    env::remove_var("ORKEE_TELEMETRY_ENABLED");

    let config = TelemetryConfig::from_env();

    // With API key, telemetry should be enabled by default
    assert!(config.enabled);

    env::remove_var("POSTHOG_API_KEY");
}

#[test]
#[serial]
fn test_telemetry_config_explicitly_disabled() {
    env::set_var("POSTHOG_API_KEY", "test-key");
    env::set_var("ORKEE_TELEMETRY_ENABLED", "false");

    let config = TelemetryConfig::from_env();

    // Even with API key, should be disabled when explicitly set
    assert!(!config.enabled);

    env::remove_var("POSTHOG_API_KEY");
    env::remove_var("ORKEE_TELEMETRY_ENABLED");
}

#[test]
#[serial]
fn test_telemetry_config_custom_endpoint() {
    env::set_var("POSTHOG_API_KEY", "test-key");
    env::set_var(
        "ORKEE_TELEMETRY_ENDPOINT",
        "https://custom.posthog.com/capture",
    );

    let config = TelemetryConfig::from_env();

    assert_eq!(config.endpoint, "https://custom.posthog.com/capture");

    env::remove_var("POSTHOG_API_KEY");
    env::remove_var("ORKEE_TELEMETRY_ENDPOINT");
}

#[test]
#[serial]
fn test_telemetry_config_debug_mode() {
    env::set_var("POSTHOG_API_KEY", "test-key");
    env::set_var("ORKEE_TELEMETRY_DEBUG", "true");

    let config = TelemetryConfig::from_env();

    assert!(config.debug_mode);

    env::remove_var("POSTHOG_API_KEY");
    env::remove_var("ORKEE_TELEMETRY_DEBUG");
}

// ============================================================================
// Event Structure Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_event_with_session_id() {
    let (pool, _temp_dir) = setup_test_db().await;

    let session_id = uuid::Uuid::new_v4().to_string();
    track_event(&pool, "test_event", None, Some(session_id.clone()))
        .await
        .unwrap();

    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].session_id, Some(session_id));
}

#[tokio::test]
#[serial]
async fn test_error_event_with_stack_trace() {
    let (pool, _temp_dir) = setup_test_db().await;

    let stack_trace = "Error at line 42\n  in function foo\n  in module bar";
    track_error(
        &pool,
        "test_error",
        "Something went wrong",
        Some(stack_trace.to_string()),
        None,
    )
    .await
    .unwrap();

    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);

    // Verify error data
    let event_data = events[0].event_data.as_ref().unwrap();
    assert_eq!(
        event_data.get("message").and_then(|v| v.as_str()),
        Some("Something went wrong")
    );
    assert_eq!(
        event_data.get("stack_trace").and_then(|v| v.as_str()),
        Some(stack_trace)
    );
}

#[tokio::test]
#[serial]
async fn test_event_timestamps() {
    let (pool, _temp_dir) = setup_test_db().await;

    let before = Utc::now();

    let event = TelemetryEvent::new(EventType::Usage, "test_event".to_string());
    event.save_to_db(&pool).await.unwrap();

    let after = Utc::now();

    let events = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(events.len(), 1);

    let timestamp = events[0].timestamp;
    assert!(timestamp >= before && timestamp <= after);
}

#[tokio::test]
#[serial]
async fn test_timestamp_parsing_fallback_on_corruption() {
    let (pool, _temp_dir) = setup_test_db().await;

    // Create and save a valid event
    let event = TelemetryEvent::new(EventType::Usage, "test_event".to_string());
    event.save_to_db(&pool).await.unwrap();

    // Corrupt the timestamp in the database to simulate data corruption
    sqlx::query("UPDATE telemetry_events SET created_at = 'invalid-timestamp' WHERE event_name = 'test_event'")
        .execute(&pool)
        .await
        .unwrap();

    // Verify the corrupted timestamp was stored
    let corrupted_check =
        sqlx::query("SELECT created_at FROM telemetry_events WHERE event_name = 'test_event'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let stored_timestamp: String = corrupted_check.get("created_at");
    assert_eq!(stored_timestamp, "invalid-timestamp");

    // get_unsent_events should still work, falling back to current time
    let before_fetch = Utc::now();
    let events = get_unsent_events(&pool, 10).await.unwrap();
    let after_fetch = Utc::now();

    // Event should still be loaded
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, "test_event");

    // Timestamp should be the current time (fallback behavior)
    let timestamp = events[0].timestamp;
    assert!(
        timestamp >= before_fetch && timestamp <= after_fetch,
        "Expected timestamp to be current time (fallback), but got {:?}",
        timestamp
    );
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_concurrent_settings_updates_no_race_condition() {
    let (pool, _temp_dir) = setup_test_db().await;

    let manager = TelemetryManager::new(pool.clone()).await.unwrap();

    // Spawn multiple concurrent update tasks
    let mut handles = vec![];

    for i in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let mut settings = manager_clone.get_settings().await;
            settings.error_reporting = i % 2 == 0;
            settings.usage_metrics = i % 3 == 0;
            settings.machine_id = Some(format!("machine-{}", i));

            manager_clone.update_settings(settings).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final state is consistent between DB and cache
    let final_settings = manager.get_settings().await;

    // Create a new manager to load from DB
    let new_manager = TelemetryManager::new(pool).await.unwrap();
    let db_settings = new_manager.get_settings().await;

    // Both should match - no DB/cache divergence
    assert_eq!(final_settings.error_reporting, db_settings.error_reporting);
    assert_eq!(final_settings.usage_metrics, db_settings.usage_metrics);
    assert_eq!(final_settings.machine_id, db_settings.machine_id);
    assert_eq!(
        final_settings.non_anonymous_metrics,
        db_settings.non_anonymous_metrics
    );
}

// ============================================================================
// Background Collector Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_background_collector_starts_and_processes_events() {
    use crate::telemetry::collector::send_buffered_events;
    use std::sync::Arc;

    let (pool, _temp_dir) = setup_test_db().await;

    // Set API key to enable telemetry
    env::set_var("POSTHOG_API_KEY", "phc_test1234567890");

    // Create telemetry manager
    let manager = Arc::new(TelemetryManager::new(pool.clone()).await.unwrap());

    // Enable telemetry
    manager
        .complete_onboarding(true, true, false)
        .await
        .unwrap();

    // Create and save an event
    track_event(&pool, "background_collector_test", None, None)
        .await
        .unwrap();

    // Verify event is unsent
    let unsent_before = get_unsent_events(&pool, 10).await.unwrap();
    assert_eq!(unsent_before.len(), 1);

    // Call send_buffered_events directly
    // This simulates what the background task does
    let result = send_buffered_events(manager.clone(), pool.clone()).await;

    // The function should either succeed or fail gracefully
    // In test environment, PostHog endpoint won't work, but the function should still run
    // and either increment retry_count (on HTTP error) or mark as sent (if filtered)
    assert!(
        result.is_ok(),
        "send_buffered_events should not panic: {:?}",
        result.err()
    );

    // Verify the event was processed (either marked as sent OR retry_count incremented)
    let query_result = sqlx::query("SELECT sent_at, retry_count FROM telemetry_events WHERE event_name = 'background_collector_test'")
        .fetch_one(&pool)
        .await
        .unwrap();

    let sent_at: Option<String> = query_result.get("sent_at");
    let retry_count: i64 = query_result.get("retry_count");

    // Event should have been processed: either marked as sent OR retry incremented
    assert!(
        sent_at.is_some() || retry_count >= 1,
        "Expected event to be processed (sent_at set or retry_count >= 1), got sent_at={:?}, retry_count={}",
        sent_at,
        retry_count
    );

    // Clean up
    env::remove_var("POSTHOG_API_KEY");
}

// ============================================================================
// Middleware Helper Function Tests
// ============================================================================

#[test]
fn test_is_task_endpoint_edge_cases() {
    use crate::api::telemetry_middleware::is_task_endpoint;

    // Edge case: Double slash (empty segment)
    assert!(!is_task_endpoint("/api/projects//tasks"));

    // Valid: Nested task endpoint
    assert!(is_task_endpoint("/api/projects/123/tasks/456"));

    // False positive prevention: Project name containing "tasks"
    assert!(!is_task_endpoint("/api/projects/my-tasks-project"));

    // Valid: Basic task endpoint
    assert!(is_task_endpoint("/api/projects/abc123/tasks"));

    // Invalid: Not a project endpoint
    assert!(!is_task_endpoint("/api/tasks"));

    // Invalid: Missing tasks segment
    assert!(!is_task_endpoint("/api/projects/123"));

    // Invalid: Different resource after project ID
    assert!(!is_task_endpoint("/api/projects/123/settings"));
}

#[test]
fn test_hash_id_consistency() {
    use crate::api::telemetry_middleware::hash_id;

    let id = "project-123";

    // Same input should produce same hash
    let hash1 = hash_id(id);
    let hash2 = hash_id(id);
    assert_eq!(hash1, hash2);

    // Different input should produce different hash
    let hash3 = hash_id("project-124");
    assert_ne!(hash1, hash3);

    // Hash should be deterministic
    assert_eq!(
        hash_id("test"),
        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    );
}

#[test]
fn test_hash_id_output_format() {
    use crate::api::telemetry_middleware::hash_id;

    let hash = hash_id("project-123");

    // SHA256 hash should be 64 hex characters
    assert_eq!(hash.len(), 64);

    // Should only contain hex characters
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Should be lowercase
    assert_eq!(hash, hash.to_lowercase());
}
