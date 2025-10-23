# Dashboard Development Mode

This document explains how to use the local dashboard during development instead of the downloaded version.

## Problem
Previously, when developing the dashboard, you had to manually copy files to `~/.orkee/dashboard` to test changes. This was cumbersome and slowed down the development workflow.

## Solution
The Orkee CLI now supports a development mode that uses the local dashboard from `packages/dashboard/` instead of downloading from GitHub releases.

## Usage

### Option 1: Using the --dev flag
```bash
cd orkee-oss
cargo run --bin orkee -- dashboard --dev
```

### Option 2: Using environment variable
```bash
cd orkee-oss
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard
```

### Option 3: With custom ports
```bash
cargo run --bin orkee -- dashboard --dev --api-port 8080 --ui-port 3000
```

## How It Works

1. **Development Mode Detection**: The CLI checks for:
   - The `--dev` flag
   - The `ORKEE_DEV_MODE` environment variable
   
2. **Dashboard Path Resolution**:
   - In dev mode: Uses `packages/dashboard/` from the current working directory
   - In production mode: Uses `~/.orkee/dashboard/` (downloaded from GitHub releases)
   
3. **Fallback Behavior**: If dev mode is enabled but the local dashboard isn't found, it falls back to the downloaded version with a warning.

## Benefits

- **Faster Development**: No need to copy files to test changes
- **Live Reloading**: Vite's hot module replacement works directly with your source files
- **Easy Switching**: Toggle between dev and production dashboards with a simple flag
- **No Authentication Required**: API authentication is bypassed in dev mode (see below)
- **CI/CD Compatible**: Production builds still work as expected without the local dashboard

## Authentication Bypass in Dev Mode

**IMPORTANT**: When `ORKEE_DEV_MODE=true` or `--dev` flag is set, **API authentication is completely bypassed**.

### What This Means

- **No Token Required**: The web dashboard can access all API endpoints without tokens
- **Localhost Only**: Server binds to 127.0.0.1, not accessible from network
- **Development Only**: This bypass is for local development convenience

### Why This Matters

In development mode:
- ✅ Web dashboard works without needing to access `~/.orkee/api-token`
- ✅ No token management needed during development
- ✅ Browser can make API calls directly without authentication headers
- ✅ Faster iteration for dashboard development

In production mode (Tauri desktop app):
- ✅ Full authentication required
- ✅ Token automatically loaded from `~/.orkee/api-token`
- ✅ All API endpoints protected

### Security Model

**Development**: Localhost-only, single-user, trusted environment = no auth needed

**Production**: Desktop app with file-based tokens = full authentication required

**⚠️ WARNING**: Never set `ORKEE_DEV_MODE=true` in production deployments. This would disable all API authentication.

## File Changes

The implementation added:
1. `--dev` flag to the `dashboard` command
2. `ORKEE_DEV_MODE` environment variable support
3. Logic to detect and use local dashboard when available

## Testing

Run the test script to see available options:
```bash
./test-dev-mode.sh
```

Then start the dashboard in dev mode:
```bash
cargo run --bin orkee -- dashboard --dev
```

You should see output indicating it's using the local dashboard:
```
🔧 Using local development dashboard from /path/to/orkee-oss/packages/dashboard
```