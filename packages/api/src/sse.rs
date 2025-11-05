// ABOUTME: Reusable Server-Sent Events (SSE) infrastructure
// ABOUTME: Provides connection tracking, rate limiting, and stream helpers for real-time updates

use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Default maximum concurrent SSE connections per IP address
/// This prevents a single client from exhausting server resources by opening unlimited connections
const DEFAULT_MAX_SSE_CONNECTIONS_PER_IP: usize = 3;

/// Maximum size for individual SSE events (64KB)
/// Events exceeding this size will be replaced with a summary event to prevent
/// excessive memory usage and network bandwidth consumption
pub const MAX_SSE_EVENT_SIZE: usize = 64 * 1024;

/// Error returned when SSE connection limit is exceeded
#[derive(Debug)]
pub struct SseConnectionLimitExceeded;

/// Tracks concurrent SSE connections per IP address
#[derive(Clone)]
pub struct SseConnectionTracker {
    connections: Arc<Mutex<HashMap<IpAddr, usize>>>,
    max_connections_per_ip: usize,
}

impl Default for SseConnectionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SseConnectionTracker {
    pub fn new() -> Self {
        // Read from environment variable with validation
        let max_connections_per_ip = std::env::var("ORKEE_SSE_MAX_CONNECTIONS_PER_IP")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&v| v > 0 && v <= 100)
            .unwrap_or(DEFAULT_MAX_SSE_CONNECTIONS_PER_IP);

        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            max_connections_per_ip,
        }
    }

    /// Try to acquire a connection slot for the given IP
    /// Returns Ok(guard) if successful, Err if limit exceeded
    pub fn try_acquire(
        &self,
        ip: IpAddr,
    ) -> Result<SseConnectionGuard, SseConnectionLimitExceeded> {
        let mut connections = self.connections.lock().unwrap_or_else(|poisoned| {
            warn!(
                audit = true,
                "SSE connection tracker mutex poisoned, recovering - this indicates a panic occurred while holding the lock"
            );
            poisoned.into_inner()
        });
        let count = connections.entry(ip).or_insert(0);

        if *count >= self.max_connections_per_ip {
            warn!(
                ip = %ip,
                current = %count,
                max = self.max_connections_per_ip,
                audit = true,
                "SSE connection limit exceeded"
            );
            return Err(SseConnectionLimitExceeded);
        }

        *count += 1;
        info!(
            ip = %ip,
            count = %count,
            max = self.max_connections_per_ip,
            "SSE connection acquired"
        );

        Ok(SseConnectionGuard {
            ip,
            tracker: self.clone(),
        })
    }

    /// Release a connection slot for the given IP
    fn release(&self, ip: IpAddr) {
        let mut connections = self.connections.lock().unwrap_or_else(|poisoned| {
            warn!(
                audit = true,
                "SSE connection tracker mutex poisoned, recovering - this indicates a panic occurred while holding the lock"
            );
            poisoned.into_inner()
        });
        if let Some(count) = connections.get_mut(&ip) {
            *count = count.saturating_sub(1);
            info!(
                ip = %ip,
                remaining = %count,
                "SSE connection released"
            );

            // Clean up entry if count reaches zero
            if *count == 0 {
                connections.remove(&ip);
            }
        }
    }
}

/// RAII guard that automatically releases an SSE connection slot when dropped
pub struct SseConnectionGuard {
    ip: IpAddr,
    tracker: SseConnectionTracker,
}

impl Drop for SseConnectionGuard {
    fn drop(&mut self) {
        self.tracker.release(self.ip);
    }
}

/// Wrapper that guarantees guard cleanup even if stream is dropped without being consumed
pub struct GuardedSseStream<S> {
    stream: std::pin::Pin<Box<S>>,
    _guard: SseConnectionGuard,
}

impl<S> GuardedSseStream<S> {
    pub fn new(stream: S, guard: SseConnectionGuard) -> Self {
        Self {
            stream: Box::pin(stream),
            _guard: guard,
        }
    }
}

impl<S, T, E> Stream for GuardedSseStream<S>
where
    S: Stream<Item = Result<T, E>>,
{
    type Item = Result<T, E>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

/// Helper to create SSE response with standard keep-alive settings
pub fn create_sse_response<S>(stream: S) -> Sse<impl Stream<Item = Result<Event, Infallible>>>
where
    S: Stream<Item = Result<Event, Infallible>> + Send + 'static,
{
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Helper to create an SSE event from JSON-serializable data
pub fn create_sse_event<T: serde::Serialize>(
    event_type: &str,
    data: &T,
) -> Result<Event, serde_json::Error> {
    let json_data = serde_json::to_string(data)?;

    Ok(Event::default().event(event_type).data(json_data))
}

/// Helper to create an error SSE event
pub fn create_error_event(error_message: &str) -> Event {
    Event::default()
        .event("error")
        .data(format!("{{\"error\":\"{}\"}}", error_message))
}

/// Helper to create a heartbeat SSE event
pub fn create_heartbeat_event() -> Event {
    Event::default().event("heartbeat").data("{}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_tracker_basic() {
        let tracker = SseConnectionTracker::new();
        let ip = "127.0.0.1".parse().unwrap();

        // Should be able to acquire connections up to the limit
        let mut guards = Vec::new();
        for _ in 0..DEFAULT_MAX_SSE_CONNECTIONS_PER_IP {
            let guard = tracker.try_acquire(ip);
            assert!(guard.is_ok());
            guards.push(guard.unwrap());
        }

        // Next connection should fail
        let result = tracker.try_acquire(ip);
        assert!(result.is_err());

        // Drop one guard and try again
        guards.pop();
        let guard = tracker.try_acquire(ip);
        assert!(guard.is_ok());
    }

    #[test]
    fn test_connection_guard_drop() {
        let tracker = SseConnectionTracker::new();
        let ip = "127.0.0.1".parse().unwrap();

        {
            let _guard = tracker.try_acquire(ip).unwrap();
            // Guard is active
        }
        // Guard dropped, should be able to acquire again

        let guard = tracker.try_acquire(ip);
        assert!(guard.is_ok());
    }

    #[test]
    fn test_multiple_ips() {
        let tracker = SseConnectionTracker::new();
        let ip1 = "127.0.0.1".parse().unwrap();
        let ip2 = "127.0.0.2".parse().unwrap();

        // Each IP should have its own limit
        let _guard1 = tracker.try_acquire(ip1).unwrap();
        let _guard2 = tracker.try_acquire(ip2).unwrap();

        // Both should still be able to acquire more connections
        let _guard3 = tracker.try_acquire(ip1).unwrap();
        let _guard4 = tracker.try_acquire(ip2).unwrap();
    }

    #[test]
    fn test_create_sse_event() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct TestData {
            message: String,
        }

        let data = TestData {
            message: "test".to_string(),
        };

        let event = create_sse_event("test", &data).unwrap();
        // Event should be created successfully
        // We can't easily test the internal structure, but at least verify it doesn't panic
        drop(event);
    }
}
