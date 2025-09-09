# Testing Guide

## Running Tests

### All Tests (Recommended)
```bash
# Run all tests single-threaded to avoid race conditions
cargo test --workspace -- --test-threads=1
```

### Individual Package Tests
```bash
# CLI tests
cargo test -p orkee-cli

# MCP Server tests (should run single-threaded)
cargo test -p orkee-mcp-server -- --test-threads=1

# Projects tests
cargo test -p orkee-projects

# Cloud tests (optional - requires cloud features)
cargo test -p orkee-cloud

# TUI tests
cargo test -p orkee-tui

# Preview tests
cargo test -p orkee-preview
```

### TLS-Specific Tests
```bash
# Run only TLS core tests
cargo test -p orkee-cli tls

# Run only HTTPS redirect tests
cargo test -p orkee-cli https_redirect

# Run all TLS/HTTPS related tests (quiet output)
cargo test -p orkee-cli tls https_redirect --quiet
```

### Cloud Sync Tests
```bash
# Run all cloud tests (requires --features cloud)
cargo test -p orkee-cloud

# Run integration tests specifically  
cargo test -p orkee-cloud integration

# Run unit tests specifically
cargo test -p orkee-cloud unit

# Run tests in client module (HTTP client functionality)
cargo test -p orkee-cloud client

# Run cloud tests with verbose output
cargo test -p orkee-cloud -- --nocapture
```

## Test Coverage Summary

- **CLI**: 62 tests - API endpoints, configuration, health checks, TLS/HTTPS functionality
- **MCP Server**: 28 tests - Protocol handling, tools, integration tests
- **Projects**: 18 tests - CRUD operations, validation, storage
- **Cloud**: 28 tests - OAuth authentication, HTTP client, API integration, error handling
- **TUI**: 63 tests - UI components, state management, input handling
- **Preview**: 2 tests - Basic functionality

**Cloud Test Coverage**:
- **Integration Tests**: 16 comprehensive scenarios - Authentication flows, API client operations, error handling
- **Unit Tests**: 12 component tests - Error types, API response parsing, token validation, project conversion

Total: 201 tests (includes 28 cloud sync tests)

### CLI Test Breakdown

- **API Tests**: 16 tests - Health endpoints, project CRUD operations, directory browsing
- **Configuration Tests**: 3 tests - Environment variable parsing, validation
- **Security Tests**: 14 tests - Rate limiting, security headers, path validation, input sanitization
- **TLS/HTTPS Tests**: 29 tests - Certificate management, HTTPS redirects, dual server mode
  - **TLS Core**: 12 tests - Certificate generation, loading, validation, error handling
    - Self-signed certificate generation with proper permissions
    - Certificate loading and Rustls configuration
    - Certificate validity checking and renewal logic
    - Error handling for missing/invalid certificates
    - TLS initialization workflow testing
  - **HTTPS Redirect**: 17 tests - URL building, host parsing, middleware behavior
    - HTTP-to-HTTPS redirect functionality
    - X-Forwarded-Proto header handling for reverse proxies
    - Host and port parsing with various configurations
    - Query parameter and path preservation
    - Configuration-based redirect behavior

## Known Issues

### Race Conditions
Some tests may fail when run in parallel due to file system race conditions in the projects storage layer. Always use `--test-threads=1` for reliable results.

### CI/CD
The GitHub Actions workflow is configured to run tests single-threaded to ensure consistent results.

## Dashboard Tests
Dashboard tests should be run separately using:
```bash
cd packages/dashboard
pnpm test
```