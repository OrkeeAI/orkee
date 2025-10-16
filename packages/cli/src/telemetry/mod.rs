// ABOUTME: Main telemetry module for opt-in usage analytics and error reporting
// ABOUTME: Provides privacy-first telemetry with granular user controls

pub mod config;
pub mod events;
pub mod collector;
pub mod posthog;

pub use config::{TelemetryConfig, TelemetrySettings, TelemetryManager};
pub use events::{TelemetryEvent, EventType, track_event, track_error};
pub use collector::{TelemetryCollector, send_buffered_events};