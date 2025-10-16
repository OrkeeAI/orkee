// ABOUTME: Telemetry collector for batching and sending events
// ABOUTME: Handles event collection, buffering, and transmission to telemetry endpoint

use super::config::TelemetryManager;
use super::events::{get_unsent_events, mark_events_as_sent, cleanup_old_events};
use super::posthog::create_posthog_batch;
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

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

        tokio::spawn(async move {
            let mut flush_interval = interval(Duration::from_secs(300)); // 5 minutes

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

                // Clean up old events (keep for 30 days)
                if let Err(e) = cleanup_old_events(&collector.pool, 30).await {
                    error!("Failed to cleanup old telemetry events: {}", e);
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
                },
                super::events::EventType::Usage | super::events::EventType::Performance => {
                    if settings.usage_metrics {
                        event.machine_id = settings.machine_id.clone();
                        if settings.non_anonymous_metrics {
                            event.user_id = settings.user_id.clone();
                            event.anonymous = false;
                        }
                        filtered_events.push(event);
                    }
                },
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
        let batch_endpoint = if self.endpoint.ends_with("/capture") {
            self.endpoint.replace("/capture", "/batch")
        } else {
            format!("{}/batch", self.endpoint)
        };

        let response = self.client
            .post(&batch_endpoint)
            .json(&batch)
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(30))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Mark events as sent
                    let event_ids: Vec<String> = filtered_events.iter().map(|e| e.id.clone()).collect();
                    mark_events_as_sent(&self.pool, &event_ids).await?;
                    info!("Successfully sent {} telemetry events to PostHog", filtered_events.len());
                } else {
                    error!("PostHog endpoint returned error: {}", resp.status());
                }
            },
            Err(e) => {
                // Don't fail if telemetry endpoint is unreachable
                debug!("Failed to send telemetry to PostHog: {}", e);
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