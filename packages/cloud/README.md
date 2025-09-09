# Orkee Cloud Package

This package provides cloud synchronization capabilities for Orkee, implementing a direct integration with the Orkee Cloud API.

## Overview

The cloud package enables:
- **OAuth Authentication**: Secure browser-based authentication flow
- **Project Synchronization**: Seamless sync of project data to Orkee Cloud
- **Multi-device Access**: Access your projects from any authenticated device
- **Token Management**: Secure local storage of authentication tokens

## Implementation

This package contains a complete cloud client implementation:

- **`CloudClient`** - Main client for Orkee Cloud API integration
- **`HttpClient`** - HTTP wrapper with authentication and error handling
- **`AuthManager`** - OAuth flow and token management
- **API Models** - Request/response structures for cloud operations
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

# Sync all projects to cloud
orkee cloud sync

# Sync specific project
orkee cloud sync --project <project-id>

# List cloud projects
orkee cloud list

# Restore project from cloud
orkee cloud restore --project <project-id>
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

**✅ Phase 3 Complete**: The OSS cloud client is fully implemented and ready. The client can authenticate, make API calls, handle errors, and integrate with CLI commands. 

**⏸️ Waiting for API Server**: The client is ready to connect to the Orkee Cloud API server when it becomes available (Phases 4-10 of the implementation plan).

## License

Part of the Orkee project. See root LICENSE file for details.