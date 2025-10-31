# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Orkee is an AI agent orchestration platform consisting of a Rust CLI server and React dashboard. The CLI provides a REST API backend while the dashboard offers a web interface for monitoring and managing AI agents. Orkee features a SQLite-first architecture with optional cloud sync capabilities via Orkee Cloud for backup, sync, and collaboration features.

## Prerequisites

- Node.js v18+
- bun v1.0+ (package manager)
- Rust (latest stable) with cargo

## Breaking Changes & Migration

### Recent Breaking Changes

#### Package Manager Migration (pnpm → bun)
**Impact**: All contributors must install bun and update their development workflow.

**Migration Steps**:
1. Install bun: `curl -fsSL https://bun.sh/install | bash` (or use npm: `npm install -g bun`)
2. Remove old dependencies: `rm -rf node_modules pnpm-lock.yaml`
3. Install with bun: `bun install`
4. Update muscle memory: Replace all `pnpm` commands with `bun`

**Why this change**: Bun provides significantly faster installation and execution times, improving developer experience.

#### Cargo Release Profile Optimizations
**Impact**: Release builds now use different optimization settings that may affect build times and binary size.

**What changed**:
- Added link-time optimization (LTO)
- Enabled codegen-units optimization
- Modified debug symbol settings
- Adjusted optimization levels for dependencies

**Migration**: No action required for most developers. Release builds will automatically use new settings via `cargo build --release`.

**Note**: First release build after this change may take longer due to additional optimizations, but subsequent builds will benefit from improved performance.

## Architecture

### Five-Package Monorepo Structure
- **CLI Server** (`packages/cli/`): Rust Axum HTTP server providing REST API endpoints
- **Dashboard** (`packages/dashboard/`): React SPA with Vite, Shadcn/ui, and Tailwind CSS
- **TUI** (`packages/tui/`): Ratatui-based standalone terminal interface using the projects library directly
- **Projects** (`packages/projects/`): Shared Rust library for project management (used by CLI and TUI)
- **Cloud** (`packages/cloud/`): Optional cloud sync functionality with Orkee Cloud integration, OAuth authentication, and subscription management

### Communication Architecture
- CLI Server runs on port 4001 by default (configurable via `--api-port` or `ORKEE_API_PORT`)
- Dashboard dev server runs on port 5173 by default (configurable via `--ui-port` or `ORKEE_UI_PORT`)
- TUI works directly with the projects library (no server connection)
- Dashboard polls health endpoints every 20 seconds
- API responses follow format: `{success: boolean, data: any, error: string | null}`

#### Real-Time Updates
- **SSE Endpoint**: `/api/preview/events` for real-time server state updates
- **Broadcast Channel**: Capacity of 200 events per subscriber (configurable via `ORKEE_EVENT_CHANNEL_SIZE` environment variable, range: 10-10000)
- **Automatic Fallback**: Falls back to 5-second polling if SSE fails after 3 retry attempts (2-second delays)
- **Connection Modes**:
  - `sse` (live): Active SSE connection with real-time updates
  - `polling` (fallback): HTTP polling every 5 seconds when SSE unavailable
  - `connecting` (initial/retry): Establishing or retrying SSE connection
- **Lag Handling**: When clients lag behind, sends sync event with current state instead of disconnecting

### CLI Server Details
- **API Port**: 4001 (configurable via `--api-port` flag or `ORKEE_API_PORT` env var)
- **UI Port**: 5173 (configurable via `--ui-port` flag or `ORKEE_UI_PORT` env var)
- **CORS**: Auto-configured based on UI port (or via `ORKEE_CORS_ORIGIN`)
- **Framework**: Axum with Tower middleware
- **API Endpoints**:
  - Health: `/api/health` and `/api/status`
  - Projects: Full CRUD at `/api/projects/*`
  - Directories: `/api/directories/list` for filesystem browsing
  - Preview Servers: `/api/preview/servers/*` for dev server management
    - Discovery: `GET /api/preview/servers/discover` - Scan for external dev servers
    - External Server Control: `POST /api/preview/servers/external/:id/restart` and `/stop`

### Dashboard Details
- **Framework**: React 18 with TypeScript
- **Build Tool**: Vite
- **Desktop Wrapper**: Tauri 2.0 (provides native desktop app with system tray)
- **Routing**: React Router v6 with pages: Usage, Projects, AIChat, MCPServers, Monitoring, Settings
- **UI Components**: Shadcn/ui with Tailwind CSS
- **State Management**: React Context (ConnectionContext for server connection)
- **API Client**: Generic fetch wrapper with health check polling
- **Tauri Configuration** (`packages/dashboard/src-tauri/tauri.conf.json`):
  - `macOSPrivateApi: true` - Enables private macOS APIs for controlling application activation policy (Dock icon behavior, Cmd+Tab visibility). Currently set to `Regular` policy (shows in Dock and Cmd+Tab). **Fallback behavior**: If this flag is disabled or the API call fails, the app defaults to macOS standard behavior (Regular policy) and continues to function normally. See `packages/dashboard/src-tauri/src/lib.rs:435-450` for implementation details.
  - `beforeDevCommand: "node ../dev-wrapper.js"` - Uses cross-platform wrapper script for Vite dev server process management
    - **Path Resolution Design**: Relative path `../dev-wrapper.js` assumes Tauri runs `beforeDevCommand` from `src-tauri/` directory
    - This is Tauri's documented behavior and matches other config paths (e.g., `frontendDist: "../dist"`)
    - The wrapper itself uses `cwd: __dirname` to ensure spawned processes run from correct directory regardless of invocation path
- **Dev Server Process Management** (`packages/dashboard/dev-wrapper.js`):
  - Cross-platform Node.js wrapper that prevents orphaned Vite processes when Tauri is killed
  - Uses `tree-kill` library for reliable process tree termination on Windows, macOS, and Linux
  - Graceful shutdown: Tries SIGTERM first, falls back to SIGKILL if process hangs
  - 5-second timeout for hanging processes before forcing shutdown (timeout keeps process alive to guarantee execution)
  - Preserves Vite's exit codes through cleanup process
  - Handles process cleanup on SIGINT/SIGTERM signals (Ctrl+C)
  - **Working Directory**: Uses `cwd: __dirname` in spawn() to ensure `bun run dev` executes from `packages/dashboard/` where `package.json` is located

### TUI Details
- **Framework**: Ratatui with crossterm backend
- **Event System**: EventHandler with sender/receiver channels
- **State Management**: AppState struct managing projects and screen navigation
- **Data Access**: Direct integration with orkee-projects library (no HTTP client)

### SQLx Query Strategy

Orkee uses two different SQLx query patterns based on the use case:

**Runtime Queries (`sqlx::query()`)** - Used in most packages:
- **Packages**: `ideate`, `projects`, `storage`, and most other Rust packages
- **Use Case**: Dynamic query construction where query structure is determined at runtime
- **Examples**:
  - Conditional column selection based on template category (`packages/ideate/src/templates.rs:26-40`)
  - Dynamic UPDATE queries that only modify provided fields (`packages/ideate/src/manager.rs:152-159`)
  - Flexible query building based on business logic
- **Trade-offs**:
  - ✅ Maximum flexibility for complex query patterns
  - ✅ Cleaner code when handling optional parameters
  - ✅ Reduced code duplication
  - ⚠️ Schema validation happens at runtime (caught by tests)
  - ⚠️ No compile-time query verification

**Compile-Time Queries (`sqlx::query!()`)** - Used selectively:
- **Packages**: `api` package (specifically `context_handlers.rs`)
- **Use Case**: Simple, static queries with fixed structure known at compile time
- **Examples**:
  - Basic SELECT queries with fixed columns (`packages/api/src/context_handlers.rs:36-38`)
  - Simple INSERT statements with known fields
- **Trade-offs**:
  - ✅ Compile-time SQL verification against database schema
  - ✅ Automatic type inference for result rows
  - ✅ Schema changes trigger compile errors
  - ❌ Cannot be used with dynamic query construction
  - ❌ Requires `.sqlx/` query cache for offline compilation

**Query Cache (`.sqlx/` directory)**:
- Contains 27 cached queries from the `api` package's compile-time macros
- Generated via `cargo sqlx prepare --workspace`
- Not needed for packages using runtime queries (ideate, projects, etc.)
- Only required for CI/offline builds when using `query!()` macros

**Why This Hybrid Approach**:
1. Runtime queries provide necessary flexibility for the ideate feature's dynamic templates and conditional updates
2. Compile-time queries add safety for simple, static queries in API handlers
3. Integration tests verify query correctness for both patterns
4. Type safety is maintained through Rust's type system and SQLx's Row trait regardless of query pattern

This is a deliberate architectural decision balancing flexibility, type safety, and maintainability.

## Development Commands

```bash
# Install dependencies
bun install

# Start both CLI server and dashboard in development
turbo dev

# Build all packages
turbo build

# Run tests
turbo test

# Lint
turbo lint

# Work on specific packages
turbo dev --filter=@orkee/dashboard    # Dashboard only
turbo dev --filter=@orkee/cli          # CLI only
turbo build --filter=@orkee/dashboard  # Build dashboard only

# CLI-specific commands (run from packages/cli/)
cargo run --bin orkee -- dashboard                        # Start API (4001) + UI (5173)
cargo run --bin orkee -- dashboard --dev                  # Use local dashboard from packages/dashboard
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000  # Custom ports
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 cargo run --bin orkee -- dashboard  # Via env vars
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard    # Use local dashboard via env var
cargo run --bin orkee -- tui                              # Launch TUI interface
cargo run --bin orkee -- projects list                    # List all projects
cargo run --bin orkee -- projects add                     # Add a new project interactively
cargo run --bin orkee -- projects show <id>               # Show project details
cargo run --bin orkee -- projects edit <id>               # Edit project interactively
cargo run --bin orkee -- projects delete <id>             # Delete a project
cargo test                                                 # Run Rust tests
cargo build --release                                     # Production build
cargo build --features cloud                              # Build with optional cloud sync features

# Dashboard-specific commands (run from packages/dashboard/)
bun run dev                             # Start Vite dev server (port from ORKEE_UI_PORT or 5173)
ORKEE_UI_PORT=3000 bun run dev         # Start on custom port
bun run build                          # Production build
bun run lint                           # Run ESLint

# Desktop app build (Tauri) - IMPORTANT: Always build CLI first!
./rebuild-desktop.sh                   # Automated: Build CLI + Tauri app in one command
# OR manually:
cargo build --release --package orkee-cli  # Step 1: Build CLI binary (REQUIRED FIRST)
bun run tauri build                    # Step 2: Build desktop app (auto-copies CLI binary)
bun run tauri dev                      # Development mode with hot reload
```

## Building the Desktop App

**CRITICAL**: The Tauri desktop app bundles the `orkee` CLI binary as a sidecar. You **must** build the CLI first:

```bash
# Recommended: Use the helper script (does everything automatically)
cd packages/dashboard
./rebuild-desktop.sh

# Manual method:
# 1. Build the CLI binary first
cd /path/to/orkee-oss
cargo build --release --package orkee-cli

# 2. Build the Tauri app (build.rs auto-copies the CLI binary)
cd packages/dashboard
bun run tauri build

# 3. Install the app
rm -rf /Applications/Orkee.app
cp -R src-tauri/target/release/bundle/macos/Orkee.app /Applications/
open /Applications/Orkee.app
```

**How it works**: The `packages/dashboard/src-tauri/build.rs` script automatically copies the CLI binary from `target/release/orkee` to `binaries/orkee-{arch}-{os}` before each Tauri build. This ensures the bundled sidecar is always up-to-date.

## CLI Command Reference

```bash
orkee dashboard [--api-port 4001] [--ui-port 5173] [--restart] [--dev]
orkee tui [--refresh-interval 20] [--theme dark|light]
orkee projects list
orkee projects show <id>
orkee projects add [--name <name>] [--path <path>] [--description <desc>]
orkee projects edit <id>
orkee projects delete <id> [--yes]
orkee cloud enable                    # Enable Orkee Cloud sync
orkee cloud disable                   # Disable cloud sync (local-only mode)
orkee cloud sync [--project <id>]     # Manually sync to cloud
orkee cloud restore [--project <id>]  # Restore from cloud backup
orkee cloud list [--limit <n>]        # List cloud snapshots
orkee cloud status                    # Show sync status
orkee cloud login                     # Authenticate with Orkee Cloud
orkee cloud logout                    # Sign out of Orkee Cloud
```

## Projects API

The CLI server provides a REST API for project management:

### Endpoints
- **GET `/api/projects`** - List all projects (returns `{success, data: Project[], error}`)
- **GET `/api/projects/:id`** - Get project by ID
- **GET `/api/projects/by-name/:name`** - Get project by name
- **POST `/api/projects/by-path`** - Get project by path (body: `{"projectRoot": "/path"}`)
- **POST `/api/projects`** - Create new project (body: ProjectCreateInput)
- **PUT `/api/projects/:id`** - Update project (body: ProjectUpdateInput)
- **DELETE `/api/projects/:id`** - Delete project

### Data Storage
**SQLite Database**: Projects stored in `~/.orkee/orkee.db` using SQLite with:
- **Local-First Architecture**: Full functionality works offline
- **ACID Transactions**: Data integrity and concurrent access support
- **Full-Text Search**: FTS5 search across project names, descriptions, and paths
- **WAL Mode**: Write-Ahead Logging for better performance
- **Cloud Sync Support**: Optional backup to Orkee Cloud with OAuth authentication

**Database Schema**:
- `projects` table: Core project data with Git info, scripts, and metadata
- `project_tags` table: Normalized tag storage with many-to-many relationship
- `projects_fts` virtual table: Full-text search index
- Cloud sync tables: Removed - cloud state now managed by Orkee Cloud

**Migration from Legacy**: Automatic migration from `~/.orkee/projects.json` to SQLite on first run

### Database Migrations
- **Initial schema**: `packages/projects/migrations/001_initial_schema.sql`
- **Migration system**: Uses SQLx migrations (tracked in `_sqlx_migrations` table)
- **To reset dev database**: `rm ~/.orkee/orkee.db && cargo run`
- **Integration tests**: `cargo test migration_integration_tests`
- **Schema validation**: All migrations tested automatically on every test run

#### Orphaned Reference Validation

**Problem**: Agent and model data is stored in JSON config files (`packages/agents/config/agents.json`, `packages/models/config/models.json`) but referenced by TEXT IDs in the database. When an agent/model is removed from config, database records become orphaned.

**Solution**: Automatic startup validation (runs during `SqliteStorage::initialize()`):

**Tables Validated**:
- `user_agents.agent_id` (NOT NULL) → **Deletes** orphaned records
- `user_agents.preferred_model_id` (nullable) → **Clears to NULL**
- `users.default_agent_id` (nullable) → **Clears to NULL**
- `tasks.assigned_agent_id` (nullable) → **Clears to NULL**
- `tasks.reviewed_by_agent_id` (nullable) → **Clears to NULL**
- `agent_executions.agent_id` (nullable) → **Clears to NULL**
- `agent_executions.model` (nullable) → **Clears to NULL**
- `ai_usage_logs.model` (NOT NULL) → **Logs warning only** (preserves historical data)

**Implementation**: `packages/projects/src/storage/sqlite.rs:217-435`

**Testing**: 7 comprehensive tests in `packages/projects/tests/migration_integration_tests.rs:848-1186`

**Logging**: Warnings logged to stderr on startup if orphaned references are found and cleaned up

#### Seed Data Strategy

**Approach**: All seed data uses `INSERT OR IGNORE` for idempotent migrations.

**Seed Data Included**:
- `storage_metadata` - Storage type and creation timestamp
- `users` - Default user (required for FK dependencies)
- `tags` - Default "main" tag (required for task FK dependencies)
- `encryption_settings` - Machine-based encryption by default
- `password_attempts` - Password attempt tracking initialization
- `telemetry_settings` - Telemetry configuration defaults
- `system_settings` - Default configuration (ports, security, TLS, rate limiting, etc.)

**Idempotency**: All INSERT statements use `OR IGNORE` to prevent UNIQUE constraint violations on migration reruns.

**Location**: `packages/projects/migrations/001_initial_schema.sql:1062-1128`

**Testing**: Comprehensive idempotency test verifies all seed data can be safely rerun (`test_migration_seed_data_is_idempotent`)

**Why in Migration**: Seed data is in the migration file (not separate `seeds.rs`) because:
1. Required for FK dependencies (default user, default tag)
2. Essential for application startup (encryption settings, system config)
3. Simple enough to maintain inline with schema
4. Uses `INSERT OR IGNORE` for safe reruns

#### Down Migration Strategy

**Purpose**: Rollback migration for development resets and test cleanup (not used in production).

**Approach**: Comprehensive cleanup with verification:

**Pre-Drop Verification** (commented out by default):
- Count records that will be deleted (projects, users, tasks)
- Check for orphaned data (data without proper FK relationships)
- Helps identify data integrity issues before cleanup

**Drop Order**:
1. **Triggers first** - Prevents trigger execution errors during table drops
2. **Views** - No dependencies on other objects
3. **Tables** - Reverse dependency order (child tables before parent tables)

**Clean State Verification** (commented out by default):
- List remaining tables (should only show `_sqlx_migrations` and SQLite internal tables)
- List remaining views (should be empty)
- List remaining triggers (should be empty)
- List remaining indexes (should only show indexes on `_sqlx_migrations`)

**Migration Metadata Cleanup** (optional, use with caution):
- Option to delete from `_sqlx_migrations` to reset migration state
- Option to drop `_sqlx_migrations` table entirely
- **WARNING**: Never do this in production - only for development resets

**Location**: `packages/projects/migrations/001_initial_schema.down.sql`

**Testing**: 3 comprehensive tests verify down migration completeness:
- `test_down_migration_removes_all_tables` - Verifies all tables/views/triggers/indexes are dropped
- `test_down_migration_drops_tables_in_correct_order` - Verifies FK cascade behavior with test data
- `test_down_migration_is_idempotent` - Verifies safe rerun with `DROP IF EXISTS`

**Idempotency**: All DROP statements use `IF EXISTS` for safe reruns.

**SQLx Note**: SQLx doesn't automatically run down migrations. They're for manual rollback via `sqlite3 < migrations/*.down.sql` or test cleanup.

## Preview Servers & External Server Discovery

Orkee provides comprehensive development server management with automatic discovery of servers started outside of Orkee.

### Server Sources

Orkee tracks three types of dev servers:
- **Orkee**: Servers launched directly through Orkee (via API or UI)
- **External**: Manually registered servers that were started externally
- **Discovered**: Automatically detected external servers matched to known projects

### Automatic Discovery

**Background Discovery Task**: Runs every 30 seconds (configurable via `ORKEE_DISCOVERY_INTERVAL_SECS`) to:
1. Scan common dev server ports (3000-5173 by default, configurable via `ORKEE_DISCOVERY_PORTS`)
2. Detect processes listening on these ports (user-owned only for security)
3. Identify framework from command line (Next.js, Vite, React, Django, Flask, etc.)
4. Match servers to existing projects by working directory
5. Auto-register discovered servers in the global registry
6. Load environment variables from `.env` files in project directories

**Security**: Discovery only tracks processes owned by the current user (UID validation).

### Server Registry

**Global Registry**: `~/.orkee/server-registry.json` persists all server state:
- Server ID, project association, PID, port, status
- Framework detection, command line, start time
- Source type (Orkee/External/Discovered)
- Matched project ID for discovered servers

**Crash Resistance**: Servers survive Orkee restarts via registry persistence and recovery on startup.

**Cleanup**: Stale entries (servers no longer running) are automatically removed every 2 minutes (configurable via `ORKEE_CLEANUP_INTERVAL_MINUTES`).

### Visual Indicators

System tray menu shows all servers with source indicators:
- No indicator: Orkee-launched servers
- "(External)" suffix: Manually registered servers
- "(Auto-detected)" suffix: Automatically discovered servers

### API Endpoints

- **GET `/api/preview/servers/discover`** - Manually trigger discovery scan, returns discovered server IDs
- **POST `/api/preview/servers/external/:id/restart`** - Restart external server using project configuration
- **POST `/api/preview/servers/external/:id/stop`** - Stop and unregister external server

## Environment Variables

### Port Configuration
- `ORKEE_API_PORT`: API server port (default: 4001) - can be overridden by `--api-port` flag
- `ORKEE_UI_PORT`: Dashboard UI port (default: 5173) - can be overridden by `--ui-port` flag

### Development Configuration
- `ORKEE_DEV_MODE`: Enable development mode to use local dashboard from `packages/dashboard/` instead of downloaded version (default: false)

### Basic Configuration
- `ORKEE_CORS_ORIGIN`: Allowed CORS origin (auto-calculated from UI port if not set)
- `CORS_ALLOW_ANY_LOCALHOST`: Allow any localhost origin in dev (default: true)
- `PORT`: Legacy API port variable (use `ORKEE_API_PORT` instead)
- `CORS_ORIGIN`: Legacy CORS variable (use `ORKEE_CORS_ORIGIN` instead)

### Security & Path Validation
- `BROWSE_SANDBOX_MODE`: Path validation mode - `strict`/`relaxed`/`disabled` (default: relaxed)
- `ALLOWED_BROWSE_PATHS`: Comma-separated allowed directories (default: ~/Documents,~/Projects,~/Desktop,~/Downloads)

### TLS/HTTPS Configuration
- `TLS_ENABLED`: Enable HTTPS (default: false)
- `TLS_CERT_PATH`: Path to TLS certificate (default: ~/.orkee/certs/cert.pem)
- `TLS_KEY_PATH`: Path to TLS private key (default: ~/.orkee/certs/key.pem)
- `AUTO_GENERATE_CERT`: Auto-generate dev certificates (default: true)
- `ENABLE_HSTS`: Enable HTTP Strict Transport Security (default: false)

### Rate Limiting
- `RATE_LIMIT_ENABLED`: Enable rate limiting (default: true)
- `RATE_LIMIT_HEALTH_RPM`: Health endpoint limit (default: 60/min)
- `RATE_LIMIT_BROWSE_RPM`: Directory browsing limit (default: 20/min)
- `RATE_LIMIT_PROJECTS_RPM`: Projects API limit (default: 30/min)
- `RATE_LIMIT_PREVIEW_RPM`: Preview operations limit (default: 10/min)
- `RATE_LIMIT_TELEMETRY_RPM`: Telemetry tracking limit (default: 15/min)
- `RATE_LIMIT_AI_RPM`: AI-powered operations limit (default: 10/min) - applies to `/ai/*` and `/ideate/*` endpoints
- `RATE_LIMIT_USERS_RPM`: User management operations limit (default: 10/min)
- `RATE_LIMIT_SECURITY_RPM`: Security operations limit (default: 10/min)
- `RATE_LIMIT_GLOBAL_RPM`: Default limit for other endpoints (default: 30/min)
- `RATE_LIMIT_BURST_SIZE`: Burst multiplier (default: 5)

### Security Headers
- `SECURITY_HEADERS_ENABLED`: Enable security headers (default: true)
- `ENABLE_REQUEST_ID`: Enable request ID tracking (default: true)

### Cloud Sync Configuration (Orkee Cloud)
- `ORKEE_CLOUD_TOKEN`: Authentication token for Orkee Cloud (required for cloud features)
- `ORKEE_CLOUD_API_URL`: API URL for Orkee Cloud (defaults to https://api.orkee.ai)

### Dashboard Tauri Configuration
- `ORKEE_TRAY_POLL_INTERVAL_SECS`: Interval for tray menu polling (default: 5, min: 1, max: 60) - controls how often the system tray checks for server status updates
- `ORKEE_API_HOST`: API host for tray connections (default: localhost) - for security, only localhost is allowed unless `ORKEE_ALLOW_REMOTE_API` is set
- `ORKEE_ALLOW_REMOTE_API`: Enable remote API access (default: false) - allows connecting to non-localhost API hosts (not recommended)

### Preview Server Configuration
- `ORKEE_STALE_TIMEOUT_MINUTES`: Timeout before server entries are considered stale (default: 5, max: 240) - controls when inactive servers are cleaned up from the registry
- `ORKEE_PROCESS_START_TIME_TOLERANCE_SECS`: Tolerance for process start time validation (default: 5, max: 60) - helps detect PID reuse on systems under heavy load
- `ORKEE_CLEANUP_INTERVAL_MINUTES`: Interval for registry cleanup task (default: 2, range: 1-60) - controls how often stale servers are removed from the registry

### External Server Discovery Configuration
- `ORKEE_DISCOVERY_ENABLED`: Enable automatic discovery of external servers (default: true) - when enabled, Orkee automatically finds and tracks dev servers started outside of Orkee
- `ORKEE_DISCOVERY_INTERVAL_SECS`: Interval for discovery scans (default: 30, range: 5-300) - controls how often Orkee scans for new external servers
- `ORKEE_DISCOVERY_PORTS`: Custom ports to scan for external servers (default: 3000-5173) - comma-separated list or ranges (e.g., "3000,8080,9000-9100")

### Telemetry Configuration
- `POSTHOG_API_KEY`: PostHog project API key for telemetry (compile-time or runtime) - telemetry is disabled if not set
- `ORKEE_TELEMETRY_ENABLED`: Enable/disable telemetry globally (default: true if API key present, false otherwise)
- `ORKEE_TELEMETRY_ENDPOINT`: PostHog endpoint URL (default: https://app.posthog.com/capture) - can be overridden for self-hosted instances
- `ORKEE_TELEMETRY_DEBUG`: Enable debug logging for telemetry (default: false)

**Telemetry Features**:
- **Privacy-First**: All telemetry is opt-in with granular user controls
- **Anonymous by Default**: Machine ID only, no user identification unless explicitly enabled
- **Local Buffering**: Events stored in SQLite (`~/.orkee/orkee.db`) before sending
- **User Controls**: Settings in database control error reporting, usage metrics, and anonymity
- **Data Retention**: 30-day retention for sent events, configurable cleanup
- **Graceful Degradation**: System works normally if telemetry is disabled or unavailable

## Key Files

### CLI Package
- `packages/cli/src/bin/orkee.rs`: Main CLI entry point and command routing
- `packages/cli/src/bin/cli/cloud.rs`: Cloud sync CLI commands
- `packages/cli/src/api/mod.rs`: API router configuration
- `packages/cli/src/config.rs`: Server configuration
- `packages/cli/src/telemetry/mod.rs`: Telemetry system entry point
- `packages/cli/src/telemetry/config.rs`: Telemetry configuration and settings management
- `packages/cli/src/telemetry/events.rs`: Event types, tracking, and database operations
- `packages/cli/src/telemetry/collector.rs`: Event batching and sending to PostHog
- `packages/cli/src/telemetry/posthog.rs`: PostHog API integration and event formatting

### Projects Package
- `packages/projects/src/lib.rs`: Public API for project management
- `packages/projects/src/manager.rs`: Core CRUD operations
- `packages/projects/src/storage/sqlite.rs`: SQLite storage implementation
- `packages/projects/src/api/handlers.rs`: HTTP request handlers

### Preview Package
- `packages/preview/src/lib.rs`: Public API for preview server management and initialization
- `packages/preview/src/manager.rs`: Preview server lifecycle management (start/stop/restart)
- `packages/preview/src/registry.rs`: Global server registry with persistence and cleanup
- `packages/preview/src/discovery.rs`: External server discovery (port scanning, process detection, framework identification)
- `packages/preview/src/types.rs`: Type definitions (ServerSource, DevServerStatus, Framework, etc.)

### Cloud Package (Optional)
- `packages/cloud/src/lib.rs`: Public API for cloud sync functionality
- `packages/cloud/src/providers/`: Cloud provider abstractions and implementations (S3, R2)
- `packages/cloud/src/sync/`: Sync engine for cloud coordination
- `packages/cloud/src/auth/`: Authentication and credential management
- `packages/cloud/src/encryption/`: Data encryption and security
- `packages/cloud/src/config/`: Configuration management
- `packages/cloud/src/state.rs`: Cloud sync state management
- `packages/cloud/migrations/`: Database schema for cloud sync state
- `packages/projects/migrations/`: Database schema migrations

### Dashboard Package
- `packages/dashboard/src/App.tsx`: Main application component
- `packages/dashboard/src/contexts/ConnectionContext.tsx`: Server connection management
- `packages/dashboard/src/services/api.ts`: API client wrapper
- `packages/dashboard/src/services/projects.ts`: Project-specific API calls

### TUI Package
- `packages/tui/src/app.rs`: Main TUI application logic
- `packages/tui/src/events.rs`: Event handler with keyboard/tick events
- `packages/tui/src/state.rs`: Application state management

## Security Architecture

### Path Validation & Sandboxing
- **Implementation**: `packages/cli/src/api/path_validator.rs`
- **Three security modes**:
  - `strict`: Only explicitly allowed paths, no path traversal
  - `relaxed`: Home + allowed paths, blocks dangerous system directories (default)
  - `disabled`: No restrictions (not recommended)
- **Always blocked paths**: System directories (`/etc`, `/sys`, `/usr/bin`), sensitive user dirs (`.ssh`, `.aws`, `.gnupg`)

### Security Features
- **TLS/HTTPS**: Native Rust implementation with rustls, auto-certificate generation
- **Rate Limiting**: Per-endpoint limits with burst protection using Governor crate
- **Security Headers**: CSP, HSTS, X-Frame-Options, X-Content-Type-Options
- **CORS Protection**: Origin validation with configurable allowlist
- **Error Sanitization**: No internal details exposed, request ID tracking
- **Input Validation**: Path traversal protection, command injection prevention

### ⚠️ API Key Encryption - CRITICAL SECURITY INFORMATION

**IMPORTANT**: By default, Orkee uses machine-based encryption which provides **transport encryption only**. This means:
- ✅ API keys are protected during backup/sync operations
- ❌ API keys can be decrypted by anyone with local database file access on the same machine
- ❌ This is NOT true at-rest encryption

**RECOMMENDATION**: For production use or shared machines, upgrade to password-based encryption immediately:
```bash
orkee security set-password    # Enable password-based encryption
orkee security status          # Check current encryption mode
```

#### Implementation Details
- **Location**: `packages/projects/src/security/encryption.rs`
- **Encryption Algorithm**: ChaCha20-Poly1305 AEAD with unique nonces per encryption
- **Key Derivation**: HKDF-SHA256 (machine) or Argon2id (password)

#### Encryption Modes

**Machine-Based Encryption (Default - ⚠️ LIMITED SECURITY)**
- Derives key from machine ID + username + hostname + application salt using HKDF-SHA256
- Username and hostname provide additional entropy, especially important on VMs/containers where machine IDs may have low entropy
- **Security Model**: Transport encryption only
  - ✅ Protects data during backup/sync
  - ❌ Does NOT protect at-rest on local machine
  - ❌ Anyone with `~/.orkee/orkee.db` file access can decrypt keys
- **Use Case**: Personal use, single-user, trusted environment only

**Password-Based Encryption (Recommended - ✅ STRONG SECURITY)**
- Derives key from user password using Argon2id (64MB memory, 3 iterations, 4 threads)
- **Security Model**: True at-rest encryption
  - ✅ Data cannot be decrypted without password
  - ✅ Suitable for shared machines
  - ✅ Production-ready security
  - ⚠️ Password cannot be recovered if lost
- **Use Case**: Production, shared machines, sensitive environments

#### Security Management Commands

```bash
# Check current encryption mode
orkee security status

# Upgrade to password-based encryption (RECOMMENDED)
orkee security set-password

# Change existing password
orkee security change-password

# Downgrade to machine-based encryption (NOT RECOMMENDED)
orkee security remove-password
```

**Password Requirements**:
- Minimum 8 characters (longer is better)
- Password is hashed separately for verification (never stored in plain text)
- Account lockout protection after failed password attempts

**Brute-Force Protection**:
- Maximum attempts before lockout: 5 failed password attempts
- Lockout duration: 15 minutes
- Configuration constants: `PASSWORD_MAX_ATTEMPTS` and `PASSWORD_LOCKOUT_DURATION_MINUTES` in `packages/projects/src/storage/sqlite.rs`
- Exponential backoff strategy prevents rapid brute-force attacks

**Database Schema**: `~/.orkee/orkee.db` encryption_settings table stores mode, salt, and verification hash

### Cloud Security Architecture (Orkee Cloud)
- **Authentication**: Token-based authentication with Orkee Cloud API
- **Access Control**: Token-based authorization with subscription tier validation
- **Data Integrity**: Orkee Cloud handles data consistency and backups
- **Encryption**: Transport layer security (TLS) for all API communications

## Installation & Distribution

### npm Installation
Orkee is distributed as an npm package that downloads the appropriate binary for your platform:

```bash
# Install globally via npm
npm install -g orkee

# Or install locally in your project
npm install orkee

# Verify installation
orkee --help
```

### Manual Installation
For advanced users or CI environments, binaries are available from GitHub releases:

```bash
# Download from GitHub releases
curl -L https://github.com/OrkeeAI/orkee/releases/latest/download/orkee-x86_64-apple-darwin.tar.gz | tar xz
./orkee --help
```

## Deployment

### Production Deployment
For production deployments, see the comprehensive deployment guide and configuration files in the `deployment/` folder:

- **`deployment/README.md`** - Complete production deployment guide
- **`deployment/docker/`** - Docker configurations (Dockerfile, docker-compose.yml)
- **`deployment/nginx/`** - Nginx reverse proxy configurations
- **`deployment/systemd/`** - SystemD service files
- **`deployment/examples/`** - Environment variable templates

Key deployment options:
- **Docker Compose**: `docker-compose -f deployment/docker/docker-compose.yml up -d`
- **Direct HTTPS**: Orkee handles TLS with auto-generated certificates
- **Behind Proxy**: Nginx/Apache handles TLS, proxies to Orkee
- **Cloud Deployment**: AWS/GCP/Azure with managed load balancers

### Security & Quality Assurance
```bash
# Security audit checks
cargo audit                    # Check for known vulnerabilities
bun audit                      # Check Node.js dependencies

# Production readiness checks
cargo test --release           # Run all tests in release mode
turbo lint                     # Check code quality
```

## Important Notes

- The Dashboard is a **client** that requires the CLI server to be running
- TUI works standalone using the projects library directly (no server dependency)
- API responses always follow the `{success, data, error}` format
- Projects are automatically assigned Git repository info if in a Git repo
- The `turbo.json` configures Turborepo task orchestration
- **Production Status**: Cloud features in development - Orkee Cloud integration in progress
- **Architecture Update**: Simplified architecture using Orkee Cloud API for all cloud functionality
- **Current State**: Local SQLite fully functional, cloud sync being implemented with Orkee Cloud

## Code Signing (macOS)

To avoid users needing to run `sudo xattr -cr /Applications/Orkee.app`, the app needs to be code signed and notarized:

### Requirements
1. **Apple Developer Account** ($99/year) - https://developer.apple.com
2. **Developer ID Certificate** - Create at https://developer.apple.com/account/resources/certificates/add
3. **App-Specific Password** - Create at appleid.apple.com for notarization

### Configuration

Update `packages/dashboard/src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAMID)",
      "providerShortName": "TEAMID"
    }
  }
}
```

### Environment Variables

```bash
export APPLE_ID="your-email@example.com"
export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"  # App-specific password
export APPLE_TEAM_ID="YOUR10CHAR"
```

### Build Process

```bash
# With signing identity and env vars set:
bun run tauri build

# The build will automatically:
# 1. Sign the app bundle
# 2. Create a DMG installer
# 3. Sign the DMG
# 4. Submit to Apple for notarization
# 5. Staple the notarization ticket
```

## Release Process

### Desktop App Release (Tauri)

```bash
# 1. Update version in tauri.conf.json
# 2. Commit version bump
# 3. Create and push tag
git tag desktop-v0.0.X
git push origin desktop-v0.0.X

# CI will automatically:
# - Build for all platforms (macOS Intel, macOS ARM, Windows, Linux)
# - Create GitHub release with binaries
# - Generate checksums
```

### CLI Release (npm)

```bash
# 1. Update version in Cargo.toml
# 2. Commit version bump
# 3. Create and push tag
git tag v0.0.X
git push origin v0.0.X

# CI will automatically:
# - Build CLI binaries for all platforms
# - Publish to npm registry
```

## Other Notes
- You run in an environment where `ast-grep` is available; whenever a search requires syntax-aware or structural matching, default to `ast-grep --lang rust -p '<pattern>'` (or set `--lang` appropriately) and avoid falling back to text-only tools like `rg` or `grep` unless explicitly requested.
