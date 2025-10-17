// ABOUTME: Telemetry collector for batching and sending events
// ABOUTME: Handles event collection, buffering, and transmission to telemetry endpoint

use super::config::TelemetryManager;
use super::events::{
    cleanup_old_events, cleanup_old_unsent_events, get_unsent_events, increment_retry_count,
    mark_events_as_sent, mark_failed_events_as_sent,
};
use super::posthog::create_posthog_batch;
use reqwest::Client;
use serde::Deserialize;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PostHogResponse {
    status: i32,
    #[serde(default)]
    status_text: String,
}

pub struct TelemetryCollector {
    manager: Arc<TelemetryManager>,
    pool: SqlitePool,
    client: Client,
    endpoint: String,
}

impl TelemetryCollector {
    pub fn new(manager: Arc<TelemetryManager>, pool: SqlitePool, endpoint: String) -> Self {
        Self {
            manager,
            pool,
            client: Client::new(),
            endpoint,
        }
    }

    /// Start the background task to periodically send telemetry events
    pub async fn start_background_task(self: Arc<Self>) {
        let collector = self.clone();
        let flush_interval_secs = collector.manager.get_flush_interval_secs();
        let retention_days = collector.manager.get_retention_days();
        let unsent_retention_days = collector.manager.get_unsent_retention_days();

        tokio::spawn(async move {
            let mut flush_interval = interval(Duration::from_secs(flush_interval_secs));

            loop {
                flush_interval.tick().await;

                if !collector.manager.is_telemetry_enabled() {
                    continue;
                }

                if !collector.manager.is_any_telemetry_enabled().await {
                    continue;
                }

                if let Err(e) = collector.send_buffered_events_internal().await {
                    error!("Failed to send telemetry events: {}", e);
                }

                // Mark failed events (retry_count >= 3) as sent to prevent accumulation
                // This prevents failed events from lingering in the database
                match sqlx::query(
                    r#"
                    SELECT id FROM telemetry_events
                    WHERE COALESCE(retry_count, 0) >= 3
                    AND sent_at IS NULL
                    "#,
                )
                .fetch_all(&collector.pool)
                .await
                {
                    Ok(rows) => {
                        if !rows.is_empty() {
                            let failed_event_ids: Vec<String> = rows
                                .iter()
                                .map(|row| row.get::<String, _>("id"))
                                .collect();

                            if let Err(e) =
                                mark_failed_events_as_sent(&collector.pool, &failed_event_ids).await
                            {
                                error!("Failed to mark failed telemetry events as sent: {}", e);
                            } else {
                                info!(
                                    "Marked {} failed telemetry events as sent",
                                    failed_event_ids.len()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to query for failed telemetry events: {}", e);
                    }
                }

                // Clean up old sent events based on configured retention
                if let Err(e) = cleanup_old_events(&collector.pool, retention_days).await {
                    error!("Failed to cleanup old sent telemetry events: {}", e);
                }

                // Clean up old unsent events to prevent unbounded growth
                // when PostHog is down or unreachable
                if let Err(e) =
                    cleanup_old_unsent_events(&collector.pool, unsent_retention_days).await
                {
                    error!("Failed to cleanup old unsent telemetry events: {}", e);
                }
            }
        });

        info!("Telemetry background task started");
    }

    async fn send_buffered_events_internal(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings = self.manager.get_settings().await;

        // Get unsent events (batch of 50)
        let events = get_unsent_events(&self.pool, 50).await?;

        if events.is_empty() {
            debug!("No telemetry events to send");
            return Ok(());
        }

        // Filter events based on user settings
        let mut filtered_events = Vec::new();
        for event in &events {
            let mut event = event.clone();
            match event.event_type {
                super::events::EventType::Error => {
                    if settings.error_reporting {
                        event.machine_id = settings.machine_id.clone();
                        if settings.non_anonymous_metrics {
                            event.user_id = settings.user_id.clone();
                            event.anonymous = false;
                        }
                        filtered_events.push(event);
                    }
                }
                super::events::EventType::Usage | super::events::EventType::Performance => {
                    if settings.usage_metrics {
                        event.machine_id = settings.machine_id.clone();
                        if settings.non_anonymous_metrics {
                            event.user_id = settings.user_id.clone();
                            event.anonymous = false;
                        }
                        filtered_events.push(event);
                    }
                }
            }
        }

        if filtered_events.is_empty() {
            // Mark all events as sent even if filtered out
            let event_ids: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
            mark_events_as_sent(&self.pool, &event_ids).await?;
            debug!("All telemetry events filtered out based on user settings");
            return Ok(());
        }

        // Create PostHog batch
        let batch = create_posthog_batch(filtered_events.clone());

        // Send to PostHog endpoint
        // PostHog uses /batch endpoint for batch events
        let endpoint = self.endpoint.trim_end_matches('/');
        let batch_endpoint = if endpoint.ends_with("/capture") {
            endpoint.replace("/capture", "/batch")
        } else {
            format!("{}/batch", endpoint)
        };

        let timeout_secs = self.manager.get_http_timeout_secs();
        let response = self
            .client
            .post(&batch_endpoint)
            .json(&batch)
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(timeout_secs))
            .send()
            .await;

        let event_ids: Vec<String> = filtered_events.iter().map(|e| e.id.clone()).collect();

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Mark events as sent on success
                    mark_events_as_sent(&self.pool, &event_ids).await?;
                    info!(
                        "Successfully sent {} telemetry events to PostHog",
                        filtered_events.len()
                    );
                } else {
                    // Increment retry count on HTTP error
                    error!("PostHog endpoint returned error: {}", resp.status());
                    increment_retry_count(&self.pool, &event_ids).await?;
                }
            }
            Err(e) => {
                // Increment retry count on network error
                warn!("Failed to send telemetry to PostHog: {}", e);
                increment_retry_count(&self.pool, &event_ids).await?;
            }
        }

        Ok(())
    }
}

/// Public function to manually trigger sending of buffered events
pub async fn send_buffered_events(
    manager: Arc<TelemetryManager>,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = manager.get_endpoint();
    let collector = TelemetryCollector::new(manager, pool, endpoint);
    collector.send_buffered_events_internal().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::events::{EventType, TelemetryEvent};
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Row;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Run migrations
        sqlx::migrate!("../projects/migrations")
            .run(&pool)
            .await
            .unwrap();

        pool
    }

    async fn insert_event_with_retry_count(
        pool: &SqlitePool,
        event: &TelemetryEvent,
        retry_count: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event_data_json = event.event_data.as_ref().map(|v| v.to_string());
        let event_type_str = match event.event_type {
            EventType::Usage => "usage",
            EventType::Error => "error",
            EventType::Performance => "performance",
        };
        let timestamp_str = event.timestamp.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO telemetry_events (
                id, event_type, event_name, event_data, anonymous, session_id, created_at, retry_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.id)
        .bind(event_type_str)
        .bind(&event.event_name)
        .bind(event_data_json)
        .bind(event.anonymous)
        .bind(&event.session_id)
        .bind(timestamp_str)
        .bind(retry_count)
        .execute(pool)
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_events_accumulate_on_failure_without_retry_logic() {
        let pool = setup_test_db().await;

        // Create test events
        let event1 = TelemetryEvent::new(EventType::Usage, "test_event".to_string());
        let event2 = TelemetryEvent::new(EventType::Usage, "test_event".to_string());

        // Insert with retry count simulating failed attempts
        insert_event_with_retry_count(&pool, &event1, 5)
            .await
            .unwrap();
        insert_event_with_retry_count(&pool, &event2, 10)
            .await
            .unwrap();

        // Query for unsent events
        let unsent =
            sqlx::query("SELECT id, retry_count FROM telemetry_events WHERE sent_at IS NULL")
                .fetch_all(&pool)
                .await
                .unwrap();

        // Without proper retry logic, events accumulate indefinitely
        assert_eq!(unsent.len(), 2);

        let retry_counts: Vec<i64> = unsent
            .iter()
            .map(|row| row.get::<i64, _>("retry_count"))
            .collect();

        assert!(retry_counts.contains(&5));
        assert!(retry_counts.contains(&10));
    }

    #[tokio::test]
    async fn test_events_excluded_after_max_retries() {
        let pool = setup_test_db().await;

        // Create test events with different retry counts
        let event1 = TelemetryEvent::new(EventType::Usage, "test_event_1".to_string());
        let event2 = TelemetryEvent::new(EventType::Usage, "test_event_2".to_string());
        let event3 = TelemetryEvent::new(EventType::Usage, "test_event_3".to_string());
        let event4 = TelemetryEvent::new(EventType::Usage, "test_event_4".to_string());

        // Insert events with various retry counts
        insert_event_with_retry_count(&pool, &event1, 0)
            .await
            .unwrap(); // Should be included
        insert_event_with_retry_count(&pool, &event2, 2)
            .await
            .unwrap(); // Should be included
        insert_event_with_retry_count(&pool, &event3, 3)
            .await
            .unwrap(); // Should be excluded (reached max)
        insert_event_with_retry_count(&pool, &event4, 5)
            .await
            .unwrap(); // Should be excluded (exceeded max)

        // Use the same function that collector uses
        let events = get_unsent_events(&pool, 50).await.unwrap();

        // Only events with retry_count < 3 should be returned
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|e| e.event_name == "test_event_1"));
        assert!(events.iter().any(|e| e.event_name == "test_event_2"));
        assert!(!events.iter().any(|e| e.event_name == "test_event_3"));
        assert!(!events.iter().any(|e| e.event_name == "test_event_4"));
    }

    #[tokio::test]
    async fn test_failed_events_marked_as_sent() {
        let pool = setup_test_db().await;

        // Create events that have exceeded max retries
        let event1 = TelemetryEvent::new(EventType::Usage, "failed_event_1".to_string());
        let event2 = TelemetryEvent::new(EventType::Error, "failed_event_2".to_string());
        let event3 = TelemetryEvent::new(EventType::Usage, "active_event".to_string());

        // Insert events: two with retry_count >= 3, one with retry_count < 3
        insert_event_with_retry_count(&pool, &event1, 3)
            .await
            .unwrap();
        insert_event_with_retry_count(&pool, &event2, 5)
            .await
            .unwrap();
        insert_event_with_retry_count(&pool, &event3, 2)
            .await
            .unwrap();

        // Verify all events are unsent initially
        let unsent_before =
            sqlx::query("SELECT COUNT(*) as count FROM telemetry_events WHERE sent_at IS NULL")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(unsent_before.get::<i64, _>("count"), 3);

        // Get failed event IDs (retry_count >= 3 and not sent)
        let failed_event_rows = sqlx::query(
            r#"
            SELECT id FROM telemetry_events
            WHERE COALESCE(retry_count, 0) >= 3
            AND sent_at IS NULL
            "#,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let failed_event_ids: Vec<String> = failed_event_rows
            .iter()
            .map(|row| row.get::<String, _>("id"))
            .collect();

        assert_eq!(failed_event_ids.len(), 2);

        // Mark failed events as sent (simulating cleanup)
        use crate::telemetry::events::mark_failed_events_as_sent;
        mark_failed_events_as_sent(&pool, &failed_event_ids)
            .await
            .unwrap();

        // Verify failed events are now marked as sent
        let unsent_after =
            sqlx::query("SELECT COUNT(*) as count FROM telemetry_events WHERE sent_at IS NULL")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(unsent_after.get::<i64, _>("count"), 1); // Only active_event remains unsent

        // Verify the sent events have retry_count >= 3
        let sent_failed =
            sqlx::query("SELECT COUNT(*) as count FROM telemetry_events WHERE sent_at IS NOT NULL AND COALESCE(retry_count, 0) >= 3")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(sent_failed.get::<i64, _>("count"), 2);
    }
}
