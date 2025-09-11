# Orkee Cloud Package

This package provides cloud synchronization capabilities for Orkee, implementing a direct integration with the Orkee Cloud API.

## Overview

The cloud package enables:
- **OAuth Authentication**: Secure browser-based authentication flow
- **Project Synchronization**: Full bidirectional sync of all project data
- **Complete Project Support**: Scripts, tags, MCP servers, git info, tasks
- **Conflict Resolution**: Automatic detection and resolution strategies
- **Incremental Sync**: Efficient delta updates for large projects
- **Multi-device Access**: Access your projects from any authenticated device
- **Token Management**: Secure local storage of authentication tokens

## Implementation

This package contains a complete cloud client implementation:

- **`CloudClient`** - Main client for Orkee Cloud API integration
- **`HttpClient`** - HTTP wrapper with authentication and error handling
- **`AuthManager`** - OAuth flow and token management
- **API Models** - Full OSS-compatible project models with all fields
- **Sync Operations** - Full, incremental, and selective sync modes
- **Conflict Detection** - Automatic conflict detection and resolution
- **8-char IDs** - Consistent ID format across OSS and Cloud
- **Comprehensive Error Handling** - User-friendly error messages and recovery

## Architecture

- **Local-First**: Full functionality works offline, cloud sync is enhancement
- **Direct API Integration**: Clean integration with Orkee Cloud REST API
- **OAuth 2.0**: Standard authentication flow with token persistence
- **Secure Storage**: Tokens stored securely in `~/.orkee/auth.toml`

## Requirements

Cloud features require:
1. Compilation with `--features cloud` flag
2. An Orkee Cloud account for API access

## Building

```bash
# Build Orkee with cloud features enabled
cargo build --features cloud

# The cloud features are disabled by default (smaller binary)
cargo build  # Cloud features NOT included
```

## Usage

Authentication and basic usage:

```bash
# Authenticate with Orkee Cloud (opens browser)
orkee cloud login

# Check authentication status  
orkee cloud status

# Sync all projects to cloud (includes all project fields)
orkee cloud sync

# Sync specific project
orkee cloud sync --project <project-id>

# Check for sync conflicts
orkee cloud conflicts --project <project-id>

# Push incremental changes
orkee cloud push --project <project-id>

# List cloud projects
orkee cloud list

# Restore project from cloud
orkee cloud restore --project <project-id>
```

## Environment Variables

```bash
# Optional: Set API URL (defaults to https://api.orkee.ai)
export ORKEE_CLOUD_API_URL=https://api.orkee.ai

# Optional: Direct token authentication (instead of OAuth)
export ORKEE_CLOUD_TOKEN=your-api-token
```

## Testing

The package includes comprehensive tests:

```bash
# Run all cloud tests
cargo test -p orkee-cloud

# Run specific test suites
cargo test -p orkee-cloud integration  # Integration tests
cargo test -p orkee-cloud unit         # Unit tests
```

**Test Coverage**: 28 tests covering authentication, API operations, error handling, and integration scenarios.

## Status

**✅ Production Ready**: The OSS cloud client is fully implemented with complete support for all project fields, conflict detection, and incremental sync. 

**✅ API Server Complete**: The Orkee Cloud API server is fully implemented with all sync endpoints, task management, and full field support.

## License

Part of the Orkee project. See root LICENSE file for details.