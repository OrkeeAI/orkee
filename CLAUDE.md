# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Orkee is an AI agent orchestration platform consisting of a Rust CLI server and React dashboard. The CLI provides a REST API backend while the dashboard offers a web interface for monitoring and managing AI agents. Orkee features a SQLite-first architecture with optional cloud sync capabilities via Orkee Cloud for backup, sync, and collaboration features.

## Prerequisites

- Node.js v18+ 
- pnpm v8+ (package manager: `pnpm@10.15.1`)
- Rust (latest stable) with cargo

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

### CLI Server Details
- **API Port**: 4001 (configurable via `--api-port` flag or `ORKEE_API_PORT` env var)
- **UI Port**: 5173 (configurable via `--ui-port` flag or `ORKEE_UI_PORT` env var)
- **CORS**: Auto-configured based on UI port (or via `ORKEE_CORS_ORIGIN`)
- **Framework**: Axum with Tower middleware
- **API Endpoints**: 
  - Health: `/api/health` and `/api/status`
  - Projects: Full CRUD at `/api/projects/*`
  - Directories: `/api/directories/list` for filesystem browsing

### Dashboard Details  
- **Framework**: React 18 with TypeScript
- **Build Tool**: Vite
- **Routing**: React Router v6 with pages: Usage, Projects, AIChat, MCPServers, Monitoring, Settings
- **UI Components**: Shadcn/ui with Tailwind CSS
- **State Management**: React Context (ConnectionContext for server connection)
- **API Client**: Generic fetch wrapper with health check polling

### TUI Details
- **Framework**: Ratatui with crossterm backend
- **Event System**: EventHandler with sender/receiver channels
- **State Management**: AppState struct managing projects and screen navigation
- **Data Access**: Direct integration with orkee-projects library (no HTTP client)

## Development Commands

```bash
# Install dependencies
pnpm install

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
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000  # Custom ports
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 cargo run --bin orkee -- dashboard  # Via env vars
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
pnpm dev                                # Start Vite dev server (port from ORKEE_UI_PORT or 5173)
ORKEE_UI_PORT=3000 pnpm dev           # Start on custom port
pnpm build                             # Production build
pnpm lint                              # Run ESLint
```

## CLI Command Reference

```bash
orkee dashboard [--api-port 4001] [--ui-port 5173] [--restart]
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
- ~~Cloud sync tables~~: Removed - cloud state now managed by Orkee Cloud

**Migration from Legacy**: Automatic migration from `~/.orkee/projects.json` to SQLite on first run

## Environment Variables

### Port Configuration
- `ORKEE_API_PORT`: API server port (default: 4001) - can be overridden by `--api-port` flag
- `ORKEE_UI_PORT`: Dashboard UI port (default: 5173) - can be overridden by `--ui-port` flag

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
- `RATE_LIMIT_GLOBAL_RPM`: Default limit for other endpoints (default: 30/min)
- `RATE_LIMIT_BURST_SIZE`: Burst multiplier (default: 5)

### Security Headers
- `SECURITY_HEADERS_ENABLED`: Enable security headers (default: true)
- `ENABLE_REQUEST_ID`: Enable request ID tracking (default: true)

### Cloud Sync Configuration (Orkee Cloud)
- `ORKEE_CLOUD_TOKEN`: Authentication token for Orkee Cloud (required for cloud features)
- `ORKEE_CLOUD_API_URL`: API URL for Orkee Cloud (defaults to https://api.orkee.ai)

## Key Files

### CLI Package
- `packages/cli/src/bin/orkee.rs`: Main CLI entry point and command routing
- `packages/cli/src/bin/cli/cloud.rs`: Cloud sync CLI commands
- `packages/cli/src/api/mod.rs`: API router configuration
- `packages/cli/src/config.rs`: Server configuration

### Projects Package
- `packages/projects/src/lib.rs`: Public API for project management
- `packages/projects/src/manager.rs`: Core CRUD operations
- `packages/projects/src/storage/sqlite.rs`: SQLite storage implementation
- `packages/projects/src/api/handlers.rs`: HTTP request handlers

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

### Cloud Security Architecture (Orkee Cloud)
- **Authentication**: Token-based authentication with Orkee Cloud API
- **Access Control**: Token-based authorization with subscription tier validation  
- **Data Integrity**: Orkee Cloud handles data consistency and backups
- **Encryption**: Transport layer security (TLS) for all API communications

## Production Deployment

### Deployment Configurations
The `deployment/` directory contains production-ready configurations:

- **Docker**: `deployment/docker/`
  - Multi-stage production Dockerfile
  - Docker Compose with Nginx reverse proxy
  - Development and production variants
- **Nginx**: `deployment/nginx/`
  - SSL-enabled reverse proxy configuration
  - Rate limiting and security headers
  - WebSocket support for real-time features
- **Systemd**: `deployment/systemd/`
  - Production service configuration
  - Security hardening settings

### Security Testing Commands
```bash
# Security audit checks
cargo audit                    # Check for known vulnerabilities
pnpm audit                     # Check Node.js dependencies

# Production readiness checks
cargo test --release           # Run all tests in release mode
turbo lint                     # Check code quality

# TLS certificate testing (if using custom certs)
openssl x509 -in cert.pem -text -noout    # Verify certificate details
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

## Other Notes
- You run in an environment where `ast-grep` is available; whenever a search requires syntax-aware or structural matching, default to `ast-grep --lang rust -p '<pattern>'` (or set `--lang` appropriately) and avoid falling back to text-only tools like `rg` or `grep` unless I explicitly request a plain-text search.

## Task Master AI Instructions
**Import Task Master's development workflow commands and guidelines, treat as if import is in the main CLAUDE.md file.**
@./.taskmaster/CLAUDE.md
