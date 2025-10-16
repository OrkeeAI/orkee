// ABOUTME: Telemetry event types and tracking functions
// ABOUTME: Defines event structures for usage, error, and performance telemetry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Usage,
    Error,
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub id: String,
    pub event_type: EventType,
    pub event_name: String,
    pub event_data: Option<Value>,
    pub anonymous: bool,
    pub session_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub machine_id: Option<String>,
    pub user_id: Option<String>,
    pub version: String,
    pub platform: String,
}

impl TelemetryEvent {
    pub fn new(event_type: EventType, event_name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            event_name,
            event_data: None,
            anonymous: true,
            session_id: None,
            timestamp: Utc::now(),
            machine_id: None,
            user_id: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.event_data = Some(data);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_identity(mut self, machine_id: Option<String>, user_id: Option<String>) -> Self {
        self.machine_id = machine_id;
        self.anonymous = user_id.is_none();
        self.user_id = user_id;
        self
    }

    pub async fn save_to_db(&self, pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
        let event_data_json = self.event_data.as_ref().map(|v| v.to_string());
        let event_type_str = match self.event_type {
            EventType::Usage => "usage",
            EventType::Error => "error",
            EventType::Performance => "performance",
        };
        let timestamp_str = self.timestamp.to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO telemetry_events (
                id,
                event_type,
                event_name,
                event_data,
                anonymous,
                session_id,
                created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            self.id,
            event_type_str,
            self.event_name,
            event_data_json,
            self.anonymous,
            self.session_id,
            timestamp_str,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Track a usage event (e.g., feature used, button clicked)
pub async fn track_event(
    pool: &SqlitePool,
    event_name: &str,
    properties: Option<HashMap<String, Value>>,
    session_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut event = TelemetryEvent::new(EventType::Usage, event_name.to_string());

    if let Some(props) = properties {
        event = event.with_data(serde_json::to_value(props)?);
    }

    if let Some(sid) = session_id {
        event = event.with_session(sid);
    }

    event.save_to_db(pool).await?;
    Ok(())
}

/// Track an error event
pub async fn track_error(
    pool: &SqlitePool,
    error_name: &str,
    error_message: &str,
    stack_trace: Option<String>,
    session_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut event = TelemetryEvent::new(EventType::Error, error_name.to_string());

    let mut error_data = HashMap::new();
    error_data.insert(
        "message".to_string(),
        Value::String(error_message.to_string()),
    );
    if let Some(trace) = stack_trace {
        error_data.insert("stack_trace".to_string(), Value::String(trace));
    }

    event = event.with_data(serde_json::to_value(error_data)?);

    if let Some(sid) = session_id {
        event = event.with_session(sid);
    }

    event.save_to_db(pool).await?;
    Ok(())
}

/// Get unsent events from the database, excluding those that have exceeded max retry attempts
pub async fn get_unsent_events(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<TelemetryEvent>, Box<dyn std::error::Error>> {
    const MAX_RETRY_COUNT: i64 = 3;

    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            event_type,
            event_name,
            event_data,
            anonymous,
            session_id,
            created_at
        FROM telemetry_events
        WHERE sent_at IS NULL
        AND COALESCE(retry_count, 0) < ?
        ORDER BY created_at ASC
        LIMIT ?
        "#,
        MAX_RETRY_COUNT,
        limit
    )
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let event_type = match row.event_type.as_str() {
            "usage" => EventType::Usage,
            "error" => EventType::Error,
            "performance" => EventType::Performance,
            _ => EventType::Usage,
        };

        let event_data = row
            .event_data
            .as_deref()
            .and_then(|json_str| serde_json::from_str(json_str).ok());

        let timestamp = DateTime::parse_from_rfc3339(&row.created_at)
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc);

        events.push(TelemetryEvent {
            id: row.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            event_type,
            event_name: row.event_name,
            event_data,
            anonymous: row.anonymous,
            session_id: row.session_id,
            timestamp,
            machine_id: None,
            user_id: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
        });
    }

    Ok(events)
}

/// Mark events as sent
pub async fn mark_events_as_sent(
    pool: &SqlitePool,
    event_ids: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    // Use a transaction to batch updates for better performance
    let mut tx = pool.begin().await?;

    for event_id in event_ids {
        sqlx::query!(
            r#"
            UPDATE telemetry_events
            SET sent_at = datetime('now')
            WHERE id = ?
            "#,
            event_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Increment retry count for events
pub async fn increment_retry_count(
    pool: &SqlitePool,
    event_ids: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;

    for event_id in event_ids {
        sqlx::query!(
            r#"
            UPDATE telemetry_events
            SET retry_count = COALESCE(retry_count, 0) + 1
            WHERE id = ?
            "#,
            event_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Mark failed events as sent after max retries to prevent infinite accumulation
pub async fn mark_failed_events_as_sent(
    pool: &SqlitePool,
    event_ids: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;

    for event_id in event_ids {
        sqlx::query!(
            r#"
            UPDATE telemetry_events
            SET sent_at = datetime('now')
            WHERE id = ?
            "#,
            event_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Clean up old sent events
pub async fn cleanup_old_events(
    pool: &SqlitePool,
    days_to_keep: i64,
) -> Result<u64, Box<dyn std::error::Error>> {
    let result = sqlx::query!(
        r#"
        DELETE FROM telemetry_events
        WHERE sent_at IS NOT NULL
        AND datetime(sent_at) < datetime('now', '-' || ? || ' days')
        "#,
        days_to_keep
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Clean up old unsent events that have been stuck for too long
/// This prevents unbounded growth when PostHog is down or unreachable
pub async fn cleanup_old_unsent_events(
    pool: &SqlitePool,
    days_to_keep: i64,
) -> Result<u64, Box<dyn std::error::Error>> {
    let result = sqlx::query!(
        r#"
        DELETE FROM telemetry_events
        WHERE sent_at IS NULL
        AND datetime(created_at) < datetime('now', '-' || ? || ' days')
        "#,
        days_to_keep
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
