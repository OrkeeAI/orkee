// ABOUTME: Main telemetry module for opt-in usage analytics and error reporting
// ABOUTME: Provides privacy-first telemetry with granular user controls

use sqlx::SqlitePool;
use std::path::PathBuf;

pub mod collector;
pub mod config;
pub mod events;
pub mod posthog;

pub use collector::{send_buffered_events, TelemetryCollector};
pub use config::{TelemetryConfig, TelemetryManager, TelemetrySettings};
pub use events::{track_error, track_event, EventType, TelemetryEvent};

/// Initialize the telemetry manager with the shared database connection
pub async fn init_telemetry_manager() -> Result<TelemetryManager, Box<dyn std::error::Error>> {
    let pool = get_database_pool().await?;
    TelemetryManager::new(pool).await
}

/// Get the shared database connection pool
async fn get_database_pool() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::Duration;

    let db_path = get_database_path()?;

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

    // Configure connection pool with limits to prevent resource exhaustion
    // Telemetry is a background feature and doesn't need many connections
    let pool = SqlitePoolOptions::new()
        .max_connections(3) // Limit connections: 1 for collector, 1 for API, 1 spare
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await?;

    // Run migrations from the projects package to ensure telemetry tables exist
    sqlx::migrate!("../projects/migrations").run(&pool).await?;

    Ok(pool)
}

/// Get the path to the Orkee database
fn get_database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Failed to find home directory")?;
    Ok(home.join(".orkee").join("orkee.db"))
}
