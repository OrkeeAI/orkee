// ABOUTME: Main telemetry module for opt-in usage analytics and error reporting
// ABOUTME: Provides privacy-first telemetry with granular user controls

use sqlx::SqlitePool;
use std::path::PathBuf;

pub mod config;
pub mod events;
pub mod collector;
pub mod posthog;

pub use config::{TelemetryConfig, TelemetrySettings, TelemetryManager};
pub use events::{TelemetryEvent, EventType, track_event, track_error};
pub use collector::{TelemetryCollector, send_buffered_events};

/// Initialize the telemetry manager with the shared database connection
pub async fn init_telemetry_manager() -> Result<TelemetryManager, Box<dyn std::error::Error>> {
    let pool = get_database_pool().await?;
    TelemetryManager::new(pool).await
}

/// Get the shared database connection pool
async fn get_database_pool() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let db_path = get_database_path()?;

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&database_url).await?;

    // Run migrations from the projects package to ensure telemetry tables exist
    sqlx::migrate!("../projects/migrations").run(&pool).await?;

    Ok(pool)
}

/// Get the path to the Orkee database
fn get_database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Failed to find home directory")?;
    Ok(home.join(".orkee").join("orkee.db"))
}