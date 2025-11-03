# Orkee Telemetry Documentation

This document describes the telemetry system in Orkee, what data is collected, how it's used, and how users can control it.

## Overview

Orkee includes an **opt-in telemetry system** that helps us understand how users interact with the application and identify issues. All telemetry is:

- **Opt-in**: Users must explicitly enable telemetry during first-run onboarding
- **Privacy-first**: Anonymous by default with granular controls
- **Transparent**: All tracked events are documented here
- **Local-first**: Events are buffered locally and sent in batches every 5 minutes
- **Respects user settings**: Users can disable specific categories or all telemetry at any time

## User Controls

### Telemetry Settings

Users can control telemetry through the Settings page or via the onboarding dialog. Available options:

- **Enable/Disable Telemetry**: Master switch to turn all telemetry on/off
- **Error Reporting**: Track application errors and crashes
- **Usage Metrics**: Track feature usage and user interactions
- **Anonymous Mode**: When enabled, only machine ID is sent (no user identification)

### Data Deletion

Users can delete all telemetry data at any time:
- Settings → Telemetry → Delete All Data
- This removes all locally stored telemetry events (sent and unsent)

## Tracked Events

### Page Views (Frontend)

Automatically tracked when users navigate between pages:

| Event Name | Description | Properties |
|------------|-------------|------------|
| `page_view:projects` | Projects list page viewed | `page_name: "projects"` |
| `page_view:project_detail` | Individual project page viewed | `page_name: "project_detail"` |
| `page_view:settings` | Settings page viewed | `page_name: "settings"` |
| `page_view:templates` | Templates page viewed | `page_name: "templates"` |

### Project Operations (Backend)

Tracked via API middleware when projects are created, updated, or deleted:

| Event Name | Description | Properties |
|------------|-------------|------------|
| `project_created` | New project created | `action: "create"` |
| `project_updated` | Project updated | `action: "update"`, `project_id: "<id>"` |
| `project_deleted` | Project deleted | `action: "delete"`, `project_id: "<id>"` |

### Preview Server Operations (Backend)

Tracked via API middleware when development servers are managed:

| Event Name | Description | Properties |
|------------|-------------|------------|
| `preview_server_started` | Dev server started | `action: "start"` |
| `preview_server_stopped` | Dev server stopped | `action: "stop"` |
| `preview_server_restarted` | Dev server restarted | `action: "restart"` |
| `preview_servers_stopped_all` | All servers stopped at once | `action: "stop_all"` |

### Ideate Operations (Backend)

Tracked via API middleware for AI-powered ideation features:

| Event Name | Description | Properties |
|------------|-------------|------------|
| `ideate_operation` | Any ideate API endpoint called | `operation: "<endpoint_path>"` |

Examples:
- `operation: "research/start"` - Research operation started
- `operation: "generation/start"` - Generation operation started
- `operation: "chat/message"` - Chat message sent

### Error Tracking (Frontend)

Automatically tracked when React errors occur:

| Event Name | Description | Properties |
|------------|-------------|------------|
| `react_error_boundary` | React component error caught | `message: "<error_message>"`, `stack_trace: "<stack>"` |

### System Events (Backend)

Tracked during application lifecycle:

| Event Name | Description | Properties |
|------------|-------------|------------|
| `telemetry_onboarding_completed` | User completed onboarding | `enabled: <boolean>`, `error_reporting: <boolean>`, `usage_metrics: <boolean>`, `non_anonymous: <boolean>` |
| `telemetry_settings_updated` | User changed telemetry settings | (current settings) |

## Data Storage

### Local Storage

- **Location**: `~/.orkee/orkee.db` (SQLite database)
- **Tables**:
  - `telemetry_events` - Buffered events awaiting transmission
  - `telemetry_settings` - User's telemetry preferences
- **Retention**: Sent events are kept for 30 days, then automatically deleted
- **Cleanup**: Old unsent events (7+ days) are automatically removed to prevent unbounded growth

### Background Collection

Events are sent to PostHog in batches:
- **Frequency**: Every 5 minutes (300 seconds)
- **Batch Size**: Up to 50 events per transmission
- **Retry Logic**: Failed events are retried up to 3 times
- **Filtering**: Events are filtered based on user settings before transmission

## Implementation Details

### Backend (Rust)

**Middleware**: `packages/cli/src/api/telemetry_middleware.rs`
- Intercepts successful API calls (2xx status codes)
- Extracts event name and properties from HTTP method and path
- Saves events to SQLite asynchronously (doesn't block responses)

**Background Collector**: `packages/cli/src/telemetry/collector.rs`
- Runs as a Tokio background task
- Wakes every 5 minutes to send buffered events
- Applies user settings filters before transmission
- Handles retries and marks events as sent

**Event Types**: `packages/cli/src/telemetry/events.rs`
- `EventType::Usage` - Feature usage and user interactions
- `EventType::Error` - Application errors and crashes
- `EventType::Performance` - Performance metrics (not yet implemented)

### Frontend (React)

**Context Provider**: `packages/dashboard/src/contexts/TelemetryContext.tsx`
- Provides hooks for tracking events
- Manages telemetry settings state
- Includes `withPageTracking` HOC for automatic page view tracking

**Error Boundary**: `packages/dashboard/src/components/TelemetryErrorBoundary.tsx`
- Catches React errors automatically
- Tracks errors to telemetry with full stack traces
- Provides user-friendly error UI

**Service**: `packages/dashboard/src/services/telemetry.ts`
- Communicates with backend API
- Handles telemetry settings CRUD
- Tracks events via `/api/telemetry/track` endpoint

## Privacy & Security

### What We DON'T Collect

- **Personal Information**: No names, emails, or contact info
- **Project Content**: No code, file contents, or project data
- **Sensitive Data**: No passwords, API keys, or credentials
- **Identifying Information**: No IP addresses or precise location data (unless user opts out of anonymous mode)

### What We DO Collect

- **Machine ID**: Randomly generated UUID for your installation
- **User ID**: Only if you opt out of anonymous mode
- **Version Info**: Orkee version and platform (macOS, Linux, Windows)
- **Timestamps**: When events occurred
- **Event Properties**: Minimal metadata (action types, operation names)

### Data Usage

Telemetry data is used solely to:
- Identify and fix bugs
- Understand which features are used
- Prioritize development efforts
- Improve user experience

Data is **never**:
- Sold to third parties
- Used for advertising
- Shared outside the development team
- Used to identify individual users (in anonymous mode)

## Configuration

### Environment Variables

- `POSTHOG_API_KEY`: PostHog project API key (required for telemetry to work)
- `ORKEE_TELEMETRY_ENABLED`: Override telemetry enabled state (default: follows user settings)
- `ORKEE_TELEMETRY_ENDPOINT`: PostHog endpoint URL (default: `https://app.posthog.com/capture`)
- `ORKEE_TELEMETRY_DEBUG`: Enable debug logging for telemetry (default: `false`)

### Database Schema

```sql
CREATE TABLE telemetry_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL CHECK(event_type IN ('usage', 'error', 'performance')),
    event_name TEXT NOT NULL,
    event_data TEXT,  -- JSON
    anonymous BOOLEAN NOT NULL DEFAULT 1,
    session_id TEXT,
    created_at TEXT NOT NULL,
    sent_at TEXT,
    retry_count INTEGER DEFAULT 0
);

CREATE TABLE telemetry_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    enabled BOOLEAN NOT NULL DEFAULT 0,
    error_reporting BOOLEAN NOT NULL DEFAULT 1,
    usage_metrics BOOLEAN NOT NULL DEFAULT 1,
    non_anonymous_metrics BOOLEAN NOT NULL DEFAULT 0,
    machine_id TEXT,
    first_run BOOLEAN NOT NULL DEFAULT 1,
    onboarding_completed BOOLEAN NOT NULL DEFAULT 0
);
```

## Testing

The telemetry system includes comprehensive tests:

- **Unit Tests**: Event creation, storage, retrieval (`packages/cli/src/tests/telemetry_tests.rs`)
- **Integration Tests**: Background collector, retry logic, settings management
- **Total**: 30 telemetry-specific tests
- **Coverage**: Event filtering, retry limits, data cleanup, concurrency

Run tests:
```bash
cd packages/cli
cargo test telemetry
```

## Disabling Telemetry

Users can disable telemetry through:

1. **Onboarding Dialog**: Choose "Disable" during first run
2. **Settings Page**: Settings → Telemetry → Toggle off
3. **Environment Variable**: `ORKEE_TELEMETRY_ENABLED=false` (overrides user settings)

When telemetry is disabled:
- No events are tracked or stored
- Background collector stops
- API endpoints return success but don't record events
- Settings are preserved for re-enabling later

## Questions or Concerns?

If you have questions about telemetry or want to report an issue:
- Open an issue: https://github.com/OrkeeAI/orkee/issues
- Review the code: All telemetry code is open source

We're committed to transparency and user privacy. This documentation will be updated as the telemetry system evolves.
