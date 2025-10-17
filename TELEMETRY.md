# Telemetry System - Maintainer Documentation

This document provides complete technical documentation for Orkee's telemetry system. This is intended for project maintainers and contributors working with telemetry features.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Environment Variables](#environment-variables)
4. [PostHog Integration](#posthog-integration)
5. [Event Schema](#event-schema)
6. [Database Schema](#database-schema)
7. [Privacy & Security](#privacy--security)
8. [Testing](#testing)
9. [Troubleshooting](#troubleshooting)

## Overview

Orkee implements an optional, privacy-first telemetry system using PostHog for analytics. The system is designed with the following principles:

- **Opt-In by Default**: All telemetry disabled until user explicitly enables it
- **Granular Controls**: Separate toggles for error reporting, usage metrics, and anonymity
- **Local Buffering**: Events stored in SQLite before transmission
- **Graceful Degradation**: Telemetry failures never affect application functionality
- **Transparent**: Full source code available for review

## Architecture

### Component Overview

The telemetry system consists of four main modules:

```
packages/cli/src/telemetry/
├── mod.rs         # Public API and initialization
├── config.rs      # Settings management and configuration
├── events.rs      # Event types and database operations
├── collector.rs   # Event batching and transmission
└── posthog.rs     # PostHog API integration
```

### Data Flow

```
Application Code
    ↓
track_event() / track_error()
    ↓
TelemetryEvent::new()
    ↓
event.save_to_db(pool)
    ↓
SQLite: telemetry_events (sent_at = NULL)
    ↓
[Background Collector - Every 5 minutes]
    ↓
get_unsent_events(pool, 50)
    ↓
Filter by user settings
    ↓
create_posthog_batch(events)
    ↓
HTTP POST to PostHog /batch
    ↓
mark_events_as_sent(pool, event_ids)
    ↓
SQLite: telemetry_events (sent_at = NOW)
    ↓
[Cleanup Job - Same background task]
    ↓
cleanup_old_events(pool, 30)
```

### Module Details

#### `mod.rs` - Public API

Exports the main telemetry types and provides initialization:

```rust
pub use config::{TelemetryConfig, TelemetryManager, TelemetrySettings};
pub use events::{TelemetryEvent, EventType, track_event, track_error};
pub use collector::{TelemetryCollector, send_buffered_events};

pub async fn init_telemetry_manager() -> Result<TelemetryManager, Error>
```

**Key Functions:**
- `init_telemetry_manager()`: Initializes telemetry with database pool
- `get_database_pool()`: Creates/connects to SQLite database
- `get_database_path()`: Returns `~/.orkee/orkee.db`

#### `config.rs` - Settings Management

Manages user preferences and configuration:

```rust
pub struct TelemetrySettings {
    pub first_run: bool,
    pub onboarding_completed: bool,
    pub error_reporting: bool,
    pub usage_metrics: bool,
    pub non_anonymous_metrics: bool,
    pub machine_id: Option<String>,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TelemetryConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub debug_mode: bool,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
    pub retention_days: i64,
    pub http_timeout_secs: u64,
}

pub struct TelemetryManager {
    settings: Arc<RwLock<TelemetrySettings>>,
    config: TelemetryConfig,
    pool: SqlitePool,
}
```

**Key Methods:**
- `TelemetryManager::new(pool)`: Creates manager with database pool
- `get_settings()`: Returns current user settings
- `update_settings(settings)`: Persists new settings to database
- `complete_onboarding(error, usage, anon)`: First-run opt-in flow
- `generate_machine_id()`: Creates UUID v4 for device identification
- `is_any_telemetry_enabled()`: Checks if any telemetry is active
- `delete_all_data()`: GDPR compliance - removes all telemetry data

#### `events.rs` - Event Types

Defines event types and database operations:

```rust
pub enum EventType {
    Usage,
    Error,
    Performance,
}

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
```

**Key Functions:**
- `track_event(pool, name, props, session)`: Track usage event
- `track_error(pool, name, msg, trace, session)`: Track error event
- `get_unsent_events(pool, limit)`: Retrieve unsent events from database
- `mark_events_as_sent(pool, ids)`: Update sent_at timestamp
- `cleanup_old_events(pool, days)`: Remove events older than retention period

#### `collector.rs` - Batching & Transmission

Handles background collection and transmission:

```rust
pub struct TelemetryCollector {
    manager: Arc<TelemetryManager>,
    pool: SqlitePool,
    client: Client,
    endpoint: String,
}
```

**Key Methods:**
- `start_background_task()`: Spawns async task for periodic collection
- `send_buffered_events_internal()`: Collects and sends events to PostHog
- **Background Task**:
  - Runs every `flush_interval_secs` (default: 300s = 5min)
  - Fetches up to 50 unsent events
  - Filters based on user settings
  - Batches and sends to PostHog
  - Cleans up old events (30-day retention)

#### `posthog.rs` - PostHog Integration

Formats events for PostHog API:

```rust
pub struct PostHogEvent {
    pub api_key: String,
    pub event: String,
    pub properties: PostHogProperties,
    pub timestamp: String,
    pub distinct_id: Option<String>,
}

pub struct PostHogBatch {
    pub api_key: String,
    pub batch: Vec<PostHogEvent>,
}
```

**Key Functions:**
- `get_posthog_api_key()`: Retrieves API key from compile-time or runtime env
- `create_posthog_batch(events)`: Converts Orkee events to PostHog format
- Event name transformation:
  - Usage: `event_name` (as-is)
  - Error: `error_{event_name}`
  - Performance: `perf_{event_name}`

## Environment Variables

### Required Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTHOG_API_KEY` | - | PostHog project API key (format: `phc_...`). This is the **public client key**, safe to expose in compiled binaries. If not set, telemetry is completely disabled. Can be set at compile-time or runtime. |

### Optional Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_TELEMETRY_ENABLED` | `true` (if API key present) | Global kill switch for telemetry. Set to `false` to disable even when API key is present. |
| `ORKEE_TELEMETRY_ENDPOINT` | `https://app.posthog.com/capture` | PostHog endpoint URL. Override for self-hosted PostHog instances. |
| `ORKEE_TELEMETRY_DEBUG` | `false` | Enable debug logging for telemetry operations. Useful for troubleshooting. |

### Configuration Priority

The system determines if telemetry is enabled using this logic:

```rust
let has_api_key = get_posthog_api_key().is_some();
let env_enabled = env::var("ORKEE_TELEMETRY_ENABLED")
    .unwrap_or("true".to_string())
    .parse::<bool>()
    .unwrap_or(true);

let enabled = env_enabled && has_api_key;
```

**Result**: Telemetry is only enabled if **BOTH**:
1. `ORKEE_TELEMETRY_ENABLED` is true (or unset)
2. `POSTHOG_API_KEY` is available

### API Key Priority

The `POSTHOG_API_KEY` can be set in two ways with the following priority order:

1. **Compile-time**: `POSTHOG_API_KEY` environment variable set during `cargo build`
   ```bash
   POSTHOG_API_KEY="phc_your_key" cargo build --release
   ```

2. **Runtime**: `POSTHOG_API_KEY` environment variable at execution
   ```bash
   export POSTHOG_API_KEY="phc_your_key"
   orkee dashboard
   ```

The compile-time value takes priority if both are set. If neither is available, telemetry is disabled gracefully and the application continues to function normally.

**Validation**: The key must start with `phc_` (PostHog project key) and be at least 11 characters long. Keys with invalid format are rejected and telemetry is disabled.

## PostHog Integration

### API Key Types

PostHog has two types of API keys:

1. **Project API Key** (`phc_...`):
   - Used by clients to send events
   - Safe to expose in client-side code
   - Write-only (can send events, cannot read data)
   - **This is what Orkee uses**

2. **Personal API Key** (`phx_...`):
   - Used for admin operations
   - Can read and modify project data
   - **Never use this in Orkee**

### Endpoints

- **Event Capture**: `POST https://app.posthog.com/capture`
- **Batch Events**: `POST https://app.posthog.com/batch`

Orkee uses the `/batch` endpoint for efficiency.

### Request Format

```json
{
  "api_key": "phc_...",
  "batch": [
    {
      "api_key": "phc_...",
      "event": "event_name",
      "properties": {
        "distinct_id": "machine-id",
        "machine_id": "uuid",
        "session_id": "optional-uuid",
        "app_version": "0.0.3",
        "platform": "macos",
        "error_message": "optional",
        "stack_trace": "optional",
        // ... custom properties
      },
      "timestamp": "2025-01-16T12:00:00Z",
      "distinct_id": "machine-id"
    }
  ]
}
```

### Self-Hosted PostHog

To use a self-hosted PostHog instance:

```bash
export POSTHOG_API_KEY="phc_your_key"
export ORKEE_TELEMETRY_ENDPOINT="https://posthog.yourcompany.com/capture"
```

The collector automatically converts `/capture` to `/batch` for batch requests.

## Event Schema

### Event Structure

Every telemetry event has these core fields:

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | UUID v4 for event |
| `event_type` | EventType | Usage, Error, or Performance |
| `event_name` | String | Human-readable event name |
| `event_data` | JSON | Optional custom properties |
| `anonymous` | Boolean | Whether user_id is included |
| `session_id` | String | Optional session UUID |
| `timestamp` | DateTime | Event creation time (UTC) |
| `machine_id` | String | Optional device identifier |
| `user_id` | String | Optional user identifier |
| `version` | String | Application version (from Cargo.toml) |
| `platform` | String | OS platform (macos, linux, windows) |

### Event Types

#### Usage Events

Track feature usage and user interactions:

```rust
track_event(
    &pool,
    "project_created",
    Some(HashMap::from([
        ("project_type", json!("web")),
        ("has_git", json!(true)),
    ])),
    Some(session_id),
).await?;
```

**PostHog Event**:
- Event name: `project_created` (unchanged)
- Properties: Custom + standard fields

#### Error Events

Track errors and exceptions:

```rust
track_error(
    &pool,
    "api_request_failed",
    "Failed to connect to server",
    Some("Error at line 42\n  in function foo"),
    Some(session_id),
).await?;
```

**PostHog Event**:
- Event name: `error_api_request_failed` (prefixed)
- Properties:
  - `error_message`: Error message
  - `stack_trace`: Optional stack trace
  - Standard fields

#### Performance Events

Track performance metrics:

```rust
let mut event = TelemetryEvent::new(EventType::Performance, "api_latency".to_string());
event = event.with_data(json!({
    "duration_ms": 150,
    "endpoint": "/api/projects",
}));
event.save_to_db(&pool).await?;
```

**PostHog Event**:
- Event name: `perf_api_latency` (prefixed)
- Properties: Custom metrics + standard fields

### Anonymity

Events are filtered based on user settings before transmission:

**Anonymous Mode** (default):
```json
{
  "distinct_id": "550e8400-e29b-41d4-a716-446655440000",  // machine_id
  "machine_id": "550e8400-e29b-41d4-a716-446655440000",
  "anonymous": true
  // No user_id
}
```

**Non-Anonymous Mode** (opted-in):
```json
{
  "distinct_id": "user-123",  // user_id if set, else machine_id
  "machine_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "user-123",
  "anonymous": false
}
```

## Database Schema

### Tables

#### `telemetry_settings`

Stores user preferences (single row, `id = 1`):

```sql
CREATE TABLE telemetry_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Enforce single row
    first_run BOOLEAN NOT NULL DEFAULT TRUE,
    onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE,
    error_reporting BOOLEAN NOT NULL DEFAULT FALSE,
    usage_metrics BOOLEAN NOT NULL DEFAULT FALSE,
    non_anonymous_metrics BOOLEAN NOT NULL DEFAULT FALSE,
    machine_id TEXT,
    user_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Constraints**:
- `CHECK (id = 1)`: Only one settings row allowed
- Defaults: All telemetry disabled by default

**Trigger**:
```sql
CREATE TRIGGER update_telemetry_settings_timestamp
AFTER UPDATE ON telemetry_settings
BEGIN
    UPDATE telemetry_settings
    SET updated_at = datetime('now')
    WHERE id = NEW.id;
END;
```

#### `telemetry_events`

Buffers events before transmission:

```sql
CREATE TABLE telemetry_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,  -- 'usage', 'error', 'performance'
    event_name TEXT NOT NULL,
    event_data TEXT,  -- JSON
    anonymous BOOLEAN NOT NULL DEFAULT TRUE,
    session_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    sent_at TEXT,  -- NULL until sent
    retry_count INTEGER DEFAULT 0
);
```

**Indexes**:
```sql
-- Efficient unsent event queries
CREATE INDEX idx_telemetry_events_unsent
ON telemetry_events(sent_at, created_at)
WHERE sent_at IS NULL;

-- Event type filtering
CREATE INDEX idx_telemetry_events_type
ON telemetry_events(event_type, created_at);
```

#### `telemetry_stats` (Optional)

Daily statistics for monitoring:

```sql
CREATE TABLE telemetry_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stat_date TEXT NOT NULL,  -- YYYY-MM-DD
    total_events INTEGER DEFAULT 0,
    error_events INTEGER DEFAULT 0,
    usage_events INTEGER DEFAULT 0,
    performance_events INTEGER DEFAULT 0,
    events_sent INTEGER DEFAULT 0,
    events_pending INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(stat_date)
);
```

### Database Operations

#### Insert Event

```rust
sqlx::query!(
    r#"
    INSERT INTO telemetry_events (
        id, event_type, event_name, event_data,
        anonymous, session_id, created_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
    "#,
    event.id,
    event_type_str,
    event.event_name,
    event_data_json,
    event.anonymous,
    event.session_id,
    timestamp_str,
)
.execute(pool)
.await?;
```

#### Query Unsent Events

```rust
sqlx::query!(
    r#"
    SELECT id, event_type, event_name, event_data,
           anonymous, session_id, created_at
    FROM telemetry_events
    WHERE sent_at IS NULL
    ORDER BY created_at ASC
    LIMIT ?
    "#,
    limit
)
.fetch_all(pool)
.await?;
```

#### Mark as Sent

```rust
sqlx::query!(
    r#"
    UPDATE telemetry_events
    SET sent_at = datetime('now')
    WHERE id = ?
    "#,
    event_id
)
.execute(pool)
.await?;
```

#### Cleanup Old Events

```rust
sqlx::query!(
    r#"
    DELETE FROM telemetry_events
    WHERE sent_at IS NOT NULL
    AND datetime(sent_at) < datetime('now', '-' || ? || ' days')
    "#,
    days_to_keep
)
.execute(pool)
.await?;
```

## Privacy & Security

### Data Minimization

The system implements strict data minimization:

**Never Collected**:
- Personal information (name, email, phone, address)
- File contents or source code
- Actual project names or file paths
- Credentials, API keys, or secrets
- Network traffic or browsing history
- Precise location data

**Collected (When Opted-In)**:
- Error messages and stack traces (sanitized)
- Feature usage events (e.g., "button clicked")
- Performance metrics (e.g., API latency)
- Application version and OS platform
- Anonymous machine ID (UUID)
- User ID (only if explicitly opted into non-anonymous mode)

### Security Measures

#### SQL Injection Protection

All database queries use parameterized queries via `sqlx::query!`:

```rust
// SAFE: Parameterized query
sqlx::query!("INSERT INTO telemetry_events (id, event_name) VALUES (?, ?)", id, name)
    .execute(pool)
    .await?;

// UNSAFE: String interpolation (never do this)
// sqlx::query(&format!("INSERT INTO telemetry_events (id) VALUES ('{}')", id))
```

**Test Coverage**: `test_event_storage_handles_special_characters` validates SQL injection protection.

#### Transport Security

- All data transmitted over HTTPS/TLS
- PostHog endpoint uses `https://` by default
- No sensitive data in URLs (everything in POST body)

#### Error Handling

- Telemetry failures logged but never crash the application
- Network errors silently ignored (app continues normally)
- Database errors logged for debugging but don't block user operations

#### Rate Limiting

- Batch size limited to 50 events per transmission
- Background task runs every 5 minutes (not on every event)
- Prevents accidental DoS of telemetry endpoint

### GDPR Compliance

#### User Rights

1. **Right to be Informed**: Clear disclosure in onboarding dialog
2. **Right of Access**: Users can query SQLite database directly
3. **Right to Erasure**: `delete_all_data()` removes all telemetry
4. **Right to Data Portability**: SQLite database can be exported
5. **Right to Object**: Telemetry can be disabled at any time

#### Implementation

**Delete All Data**:
```rust
pub async fn delete_all_data(&self) -> Result<u64, Error> {
    // Delete all telemetry events
    let result = sqlx::query!("DELETE FROM telemetry_events")
        .execute(&self.pool)
        .await?;

    // Reset statistics if table exists
    let _ = sqlx::query!("DELETE FROM telemetry_stats")
        .execute(&self.pool)
        .await;

    Ok(result.rows_affected())
}
```

**Test Coverage**: `test_delete_all_data_removes_events` validates data deletion.

## Testing

### Unit Tests

Location: `packages/cli/src/tests/telemetry_tests.rs`

**Test Coverage** (26 tests):

1. **Settings Persistence** (5 tests)
   - Default values
   - Persistence across loads
   - Onboarding flows
   - Single-row constraint

2. **Machine ID Generation** (2 tests)
   - UUID format validation
   - Stability across sessions

3. **Event Filtering** (4 tests)
   - Error event filtering
   - Usage event filtering
   - Anonymous mode
   - Non-anonymous mode

4. **SQL Query Safety** (4 tests)
   - SQL injection protection
   - Unicode handling
   - JSON injection protection
   - Bulk operations

5. **Privacy Controls** (3 tests)
   - Delete all data
   - Retention cleanup
   - Opt-out functionality

6. **Configuration** (5 tests)
   - Environment variable handling
   - API key requirements
   - Custom endpoints
   - Debug mode

7. **Event Structure** (3 tests)
   - Session ID tracking
   - Error events with stack traces
   - Timestamp accuracy

### Running Tests

```bash
# Run all telemetry tests
cargo test --package orkee-cli telemetry_tests

# Run specific test
cargo test --package orkee-cli telemetry_tests::test_machine_id_is_valid_uuid

# Run with debug output
cargo test --package orkee-cli telemetry_tests -- --nocapture
```

### Test Database

Tests use temporary SQLite databases:

```rust
async fn setup_test_db() -> (SqlitePool, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&database_url).await.unwrap();

    // Run migrations
    sqlx::migrate!("../projects/migrations")
        .run(&pool)
        .await
        .unwrap();

    (pool, temp_dir)
}
```

## Troubleshooting

### Debug Mode

Enable debug logging:

```bash
export ORKEE_TELEMETRY_DEBUG=true
export RUST_LOG=debug
cargo run --bin orkee -- dashboard
```

**Debug Output**:
- Event creation
- Database operations
- Filter decisions
- Network requests
- Background task activity

### Common Issues

#### Events Not Being Sent

**Symptom**: Events accumulate in database but never transmitted.

**Diagnosis**:
```bash
# Check settings
sqlite3 ~/.orkee/orkee.db "SELECT * FROM telemetry_settings WHERE id = 1;"

# Check unsent events
sqlite3 ~/.orkee/orkee.db "SELECT COUNT(*) FROM telemetry_events WHERE sent_at IS NULL;"

# Check API key
echo $POSTHOG_API_KEY

# Test PostHog connectivity
curl -v https://app.posthog.com/capture
```

**Solutions**:
1. Verify `POSTHOG_API_KEY` is set
2. Verify `ORKEE_TELEMETRY_ENABLED=true` (or unset)
3. Enable telemetry in user settings
4. Check network connectivity

#### Background Task Not Running

**Symptom**: Events never leave database even when enabled.

**Diagnosis**:
```bash
# Enable debug logging
RUST_LOG=debug orkee dashboard 2>&1 | grep telemetry

# Look for: "Telemetry background task started"
```

**Solutions**:
1. Verify `TelemetryManager` is initialized
2. Verify `start_background_task()` is called
3. Check for runtime errors in logs

#### API Key Not Found

**Symptom**: "Telemetry disabled: no API key" in logs.

**Solutions**:
```bash
# Set at runtime
export POSTHOG_API_KEY="phc_your_key"

# Or set at compile time
POSTHOG_API_KEY="phc_your_key" cargo build --release
```

### Manual Database Operations

#### View Current Settings

```bash
sqlite3 ~/.orkee/orkee.db <<EOF
.mode column
.headers on
SELECT * FROM telemetry_settings;
EOF
```

#### View Unsent Events

```bash
sqlite3 ~/.orkee/orkee.db <<EOF
SELECT
    event_type,
    event_name,
    created_at,
    anonymous
FROM telemetry_events
WHERE sent_at IS NULL
ORDER BY created_at DESC
LIMIT 10;
EOF
```

#### Manually Trigger Send

While the application is running, events are sent automatically. To force immediate send:

1. Reduce `flush_interval_secs` to 10 seconds (requires code change)
2. Or wait for the next 5-minute interval
3. Or restart the application (triggers send on shutdown in future versions)

#### Reset Telemetry

```bash
sqlite3 ~/.orkee/orkee.db <<EOF
DELETE FROM telemetry_events;
DELETE FROM telemetry_stats;
UPDATE telemetry_settings
SET first_run = 1,
    onboarding_completed = 0,
    error_reporting = 0,
    usage_metrics = 0,
    non_anonymous_metrics = 0,
    machine_id = NULL,
    user_id = NULL,
    updated_at = datetime('now')
WHERE id = 1;
EOF
```

---

## Additional Resources

- [PostHog Documentation](https://posthog.com/docs)
- [PostHog API Reference](https://posthog.com/docs/api)
- [SQLx Documentation](https://docs.rs/sqlx)
- [GDPR Compliance Guide](https://gdpr.eu/)

For questions or issues, see the main [CLAUDE.md](./CLAUDE.md) development guide.
