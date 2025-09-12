# Orkee CLI Package

The Orkee CLI package provides the core REST API server and command-line interface for the Orkee platform.

## Overview

This package serves as:
- **REST API Server** - Axum-based HTTP/HTTPS server on port 4001
- **CLI Interface** - Command-line tools for project management
- **Cloud Integration** - Optional cloud sync commands (with `--features cloud`)

## Architecture

The CLI package is structured as:
- `src/bin/orkee.rs` - Main CLI entry point
- `src/bin/cli/` - CLI command modules
- `src/api/` - REST API routes and handlers
- `src/middleware/` - Authentication, CORS, rate limiting
- `src/services/` - Business logic and integrations

## Features

### Project Management
```bash
# List all projects
orkee projects list

# Create a new project
orkee projects create --name "My Project" --path "/path/to/project"

# Update project
orkee projects update <id> --name "New Name"

# Delete project
orkee projects delete <id>
```

### Cloud Sync (Optional)
When built with `--features cloud`:
```bash
# Authenticate with Orkee Cloud
orkee cloud login

# Sync projects (full support for all fields)
orkee cloud sync

# Check for conflicts
orkee cloud conflicts --project <id>

# Push incremental changes
orkee cloud push --project <id>
```

### Dashboard Server
```bash
# Start the API server and dashboard with default ports
orkee dashboard
# API on port 4001, UI on port 5173

# With custom ports via CLI arguments
orkee dashboard --api-port 8080 --ui-port 3000

# With custom ports via environment variables
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 orkee dashboard

# CLI arguments override environment variables
ORKEE_API_PORT=9000 orkee dashboard --api-port 7777
# Uses API port 7777 (CLI) and UI port 9000 (env)

# With HTTPS enabled (auto-generates certs)
TLS_ENABLED=true orkee dashboard

# Restart services (kills existing processes first)
orkee dashboard --restart
```

## API Endpoints

The REST API provides:
- `/api/projects` - CRUD operations for projects
- `/api/projects/:id/preview` - Preview server management
- `/api/health` - Health check endpoint
- `/api/metrics` - System metrics

## Environment Variables

```bash
# Port Configuration (simple and clean - just two ports!)
ORKEE_API_PORT=4001       # API server port (default: 4001)
ORKEE_UI_PORT=5173        # Dashboard UI port (default: 5173)

# Server Configuration (advanced)
HOST=127.0.0.1
# ORKEE_CORS_ORIGIN is auto-calculated from UI port if not set
CORS_ALLOW_ANY_LOCALHOST=true  # Allow any localhost origin in dev

# TLS/HTTPS Configuration
TLS_ENABLED=false
TLS_CERT_PATH=certs/cert.pem
TLS_KEY_PATH=certs/key.pem

# Cloud Configuration (optional)
ORKEE_CLOUD_API_URL=https://api.orkee.ai
ORKEE_CLOUD_TOKEN=your-token
```

## Building

```bash
# Standard build (no cloud features)
cargo build --bin orkee

# Build with cloud sync features
cargo build --bin orkee --features cloud

# Release build
cargo build --release --bin orkee --features cloud
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test api::
cargo test middleware::
```

## License

Part of the Orkee project. See root LICENSE file for details.
