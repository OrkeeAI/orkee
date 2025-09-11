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
# Start the API server (required for web dashboard)
orkee dashboard

# With custom port
orkee dashboard --port 8080

# With HTTPS enabled (auto-generates certs)
TLS_ENABLED=true orkee dashboard
```

## API Endpoints

The REST API provides:
- `/api/projects` - CRUD operations for projects
- `/api/projects/:id/preview` - Preview server management
- `/api/health` - Health check endpoint
- `/api/metrics` - System metrics

## Environment Variables

```bash
# Server Configuration
HOST=127.0.0.1
PORT=4001
CORS_ORIGIN=http://localhost:5173

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
