---
sidebar_position: 1
title: CLI Commands
---

# CLI Commands

Comprehensive reference for all Orkee command-line interface commands.

## Overview

Orkee provides a powerful CLI for managing AI agent projects, running development servers, and syncing with Orkee Cloud. All commands follow the pattern:

```bash
orkee <command> [subcommand] [options]
```

## Main Commands

### Dashboard Command

Launch the Orkee dashboard with both API server and web interface.

```bash
orkee dashboard [options]
```

**Options:**

| Flag | Environment Variable | Default | Description |
|------|---------------------|---------|-------------|
| `--api-port <port>` | `ORKEE_API_PORT` | `4001` | Port for the API server |
| `--ui-port <port>` | `ORKEE_UI_PORT` | `5173` | Port for the dashboard UI |
| `--restart` | - | - | Restart existing processes |
| `--dev` | `ORKEE_DEV_MODE` | `false` | Use local dashboard from packages/dashboard |

**Examples:**

```bash
# Start with default ports (API: 4001, UI: 5173)
orkee dashboard

# Custom ports via command line
orkee dashboard --api-port 8080 --ui-port 3000

# Custom ports via environment variables
ORKEE_API_PORT=8080 ORKEE_UI_PORT=3000 orkee dashboard

# Development mode (use local dashboard source)
orkee dashboard --dev

# Restart existing dashboard processes
orkee dashboard --restart
```

**What It Does:**

- Starts Rust API server on configured port (default: 4001)
- Launches web dashboard on configured port (default: 5173)
- Automatically opens browser to dashboard URL
- Provides real-time project monitoring and management

### TUI Command

Launch the Orkee Terminal User Interface for keyboard-driven project management.

```bash
orkee tui [options]
```

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--refresh-interval <seconds>` | `20` | Auto-refresh interval |
| `--theme <theme>` | `dark` | UI theme (dark/light) |

**Examples:**

```bash
# Launch TUI with defaults
orkee tui

# Custom refresh interval (30 seconds)
orkee tui --refresh-interval 30

# Light theme
orkee tui --theme light
```

**What It Does:**

- Runs entirely in the terminal (no browser required)
- Provides keyboard-driven interface for project management
- Works offline (no HTTP server required)
- Direct SQLite database access

**Key Bindings:**

- `↑/↓` or `j/k` - Navigate projects
- `Enter` - View project details
- `n` - Create new project
- `e` - Edit selected project
- `d` - Delete selected project
- `r` - Refresh project list
- `q` - Quit TUI

### Projects Command

Manage projects via the command line without launching the dashboard or TUI.

```bash
orkee projects <subcommand> [options]
```

#### List Projects

Display all projects in a formatted table.

```bash
orkee projects list
```

**Output:**

```
ID  Name                Status     Path
1   my-app              Building   /Users/user/projects/my-app
2   api-service         Launched   /Users/user/projects/api
3   website             Planning   /Users/user/projects/web
```

#### Show Project

Display detailed information about a specific project.

```bash
orkee projects show <id>
```

**Example:**

```bash
orkee projects show 1
```

**Output:**

```
Project Details
---------------
ID: 1
Name: my-app
Status: Building
Path: /Users/user/projects/my-app
Description: React application with TypeScript
Created: 2024-01-15 10:30:00
Updated: 2024-01-16 14:20:00

Git Info:
  Branch: main
  Remote: origin
  URL: https://github.com/user/my-app

Tags: react, typescript, frontend
```

#### Add Project

Create a new project interactively or with provided options.

```bash
orkee projects add [options]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--name <name>` | Project name |
| `--path <path>` | Project directory path |
| `--description <desc>` | Project description |

**Examples:**

```bash
# Interactive mode (prompts for all fields)
orkee projects add

# Specify all fields via flags
orkee projects add --name my-app --path ~/projects/my-app --description "React app"

# Mix of flags and interactive
orkee projects add --path ~/projects/my-app
# (will prompt for name and description)
```

#### Edit Project

Update an existing project interactively.

```bash
orkee projects edit <id>
```

**Example:**

```bash
orkee projects edit 1
```

**Interactive Prompts:**

- Update name? (y/n)
- Update description? (y/n)
- Update path? (y/n)
- Update status? (y/n)
- Update tags? (y/n)

#### Delete Project

Remove a project from Orkee.

```bash
orkee projects delete <id> [options]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--yes` or `-y` | Skip confirmation prompt |

**Examples:**

```bash
# With confirmation prompt
orkee projects delete 1

# Skip confirmation
orkee projects delete 1 --yes
orkee projects delete 1 -y
```

### OAuth/Auth Commands

Manage authentication for AI providers (Claude, OpenAI, Google, xAI) and Docker Hub.

```bash
orkee login <provider>        # Authenticate with AI provider
orkee logout <provider>       # Logout from AI provider
orkee auth status             # Show authentication status
orkee auth refresh <provider> # Refresh authentication token
orkee auth login docker       # Authenticate with Docker Hub
```

#### Login Command

Authenticate with an AI provider using OAuth.

**Syntax:**
```bash
orkee login <provider> [options]
```

**Arguments:**
- `<provider>`: Provider name (claude, openai, google, xai)

**Options:**
- `--force`: Force re-authentication even if already logged in

**Examples:**
```bash
# Login to Claude
orkee login claude

# Force re-authentication
orkee login claude --force

# Login to other providers
orkee login openai
orkee login google
orkee login xai
```

**What It Does:**
1. Opens your browser to the provider's OAuth authorization page
2. Starts a local callback server on port 3737
3. Exchanges authorization code for access token
4. Encrypts and stores token in `~/.orkee/orkee.db`
5. Displays account information (email, subscription type)

**Output Example:**
```
🔐 Authenticating with claude...
📱 Opening browser for authentication...
✅ Successfully authenticated!
   Account: user@example.com
   Subscription: Pro
   Expires: 2025-12-04 15:30:00
```

#### Logout Command

Remove OAuth authentication for a provider.

**Syntax:**
```bash
orkee logout <provider>
```

**Arguments:**
- `<provider>`: Provider name (claude, openai, google, xai) or "all"

**Examples:**
```bash
# Logout from Claude
orkee logout claude

# Logout from all providers
orkee logout all
```

**What It Does:**
- Removes encrypted OAuth tokens from database
- Reverts to API key authentication (if configured)
- Does not revoke tokens with the provider (tokens remain valid until expiry)

#### Auth Status Command

Show authentication status for all providers.

**Syntax:**
```bash
orkee auth status
```

**Output Example:**
```
OAuth Authentication Status:

✅ claude (Authenticated)
   Account: user@example.com
   Subscription: Pro
   Expires: 2025-12-04 15:30:00 (29 days)

❌ openai (Not authenticated)
   Use 'orkee login openai' to authenticate

✅ google (Authenticated)
   Account: user@example.com
   Subscription: Standard
   Expires: 2025-11-15 10:00:00 (10 days)
   ⚠️  Token expires soon! Run 'orkee auth refresh google'

❌ xai (Not authenticated)
   Use 'orkee login xai' to authenticate
```

**Status Indicators:**
- ✅ Green checkmark: Authenticated and valid
- ⚠️  Yellow warning: Token expiring soon (< 7 days)
- ❌ Red X: Not authenticated

#### Auth Refresh Command

Manually refresh an OAuth token.

**Syntax:**
```bash
orkee auth refresh <provider>
```

**Arguments:**
- `<provider>`: Provider name (claude, openai, google, xai)

**Examples:**
```bash
# Refresh Claude token
orkee auth refresh claude
```

**What It Does:**
- Uses refresh token to obtain new access token
- Updates encrypted token in database
- Extends token expiration time

**Note:** Tokens automatically refresh 5 minutes before expiry. Manual refresh is useful for:
- Testing token refresh functionality
- Force-refreshing after provider changes
- Troubleshooting authentication issues

#### Authentication Preferences

Configure OAuth vs API key priority:

```bash
# Set authentication preference
orkee config set auth_preference <mode>
```

**Modes:**
- `hybrid` (default): Try OAuth first, fall back to API keys
- `oauth`: Use OAuth only, error if not authenticated
- `api_key`: Use API keys only, ignore OAuth tokens

**Examples:**
```bash
# Default: OAuth with API key fallback
orkee config set auth_preference hybrid

# OAuth only
orkee config set auth_preference oauth

# API keys only
orkee config set auth_preference api_key
```

#### OAuth Configuration

**Environment Variables:**

```bash
# OAuth callback port
OAUTH_CALLBACK_PORT=3737

# Security timeouts
OAUTH_STATE_TIMEOUT_SECS=600        # State timeout (10 minutes)
OAUTH_TOKEN_REFRESH_BUFFER_SECS=300 # Refresh buffer (5 minutes)

# Provider-specific (optional)
OAUTH_CLAUDE_CLIENT_ID=your-client-id
OAUTH_CLAUDE_REDIRECT_URI=http://localhost:3737/callback
OAUTH_CLAUDE_SCOPES="model:claude account:read"
```

**Database Storage:**
- **Location**: `~/.orkee/orkee.db`
- **Tables**: `oauth_tokens`, `oauth_providers`
- **Encryption**: ChaCha20-Poly1305 AEAD
- **Security**: Tokens encrypted with unique nonces

For detailed OAuth documentation, see the [OAuth Authentication Guide](./oauth-authentication.md).

### Docker Authentication

Authenticate with Docker Hub for building and pushing container images.

#### Docker Login Command

**Syntax:**
```bash
orkee auth login docker
```

**What It Does:**
1. Executes `docker login` as a subprocess
2. Docker CLI opens browser for authentication (or prompts for credentials)
3. Docker stores credentials in system keychain (`~/.docker/config.json`)
4. Future `docker build` and `docker push` commands use stored credentials automatically

**Output Example:**
```
🐳 Authenticating with Docker Hub...
Opening browser for Docker authentication...

Login Succeeded
✅ Successfully authenticated with Docker Hub
   Username: your-username
```

**Benefits:**
- ✅ **Simple**: No custom credential management
- ✅ **Secure**: Leverages Docker's system keychain integration
- ✅ **Standard**: Uses Docker's native authentication flow
- ✅ **Rate limits**: Authenticated users get 200 pulls/6 hours (vs 100 unauthenticated)

**Prerequisites:**
- Docker CLI must be installed and in PATH
- Docker daemon must be running

**Verification:**
```bash
# Verify authentication succeeded
docker info

# Should display your Docker Hub username in the output
```

**Troubleshooting:**
- **Error: "docker: command not found"**: Install Docker CLI
- **Error: "Cannot connect to Docker daemon"**: Start Docker Desktop or Docker daemon
- **Authentication fails**: Try running `docker login` directly to see detailed error messages
- **Rate limiting**: Authenticated users get higher rate limits; run `orkee auth login docker` to increase limits

**Storage:**
- **Location**: Credentials stored by Docker CLI (not in Orkee database)
- **macOS**: System Keychain
- **Linux**: `~/.docker/config.json` (or configured credential helper)
- **Windows**: Windows Credential Manager

For complete Docker authentication details and sandbox image management workflow, see the project's [docker.md](../../docker.md) documentation.

### Cloud Command

Manage Orkee Cloud synchronization and backups.

```bash
orkee cloud <subcommand> [options]
```

#### Enable Cloud Sync

Enable cloud synchronization for your projects.

```bash
orkee cloud enable
```

**What It Does:**

- Enables cloud sync features
- Requires authentication with Orkee Cloud
- Projects will automatically sync to cloud

#### Disable Cloud Sync

Switch to local-only mode (no cloud sync).

```bash
orkee cloud disable
```

**What It Does:**

- Disables cloud synchronization
- Projects remain local only
- Previously synced data remains in cloud

#### Manual Sync

Manually trigger synchronization to Orkee Cloud.

```bash
orkee cloud sync [options]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--project <id>` | Sync specific project only |

**Examples:**

```bash
# Sync all projects
orkee cloud sync

# Sync specific project
orkee cloud sync --project 1
```

#### Restore from Cloud

Restore projects from cloud backup.

```bash
orkee cloud restore [options]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--project <id>` | Restore specific project only |

**Examples:**

```bash
# Restore all projects
orkee cloud restore

# Restore specific project
orkee cloud restore --project 1
```

#### List Cloud Snapshots

View available cloud backup snapshots.

```bash
orkee cloud list [options]
```

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--limit <n>` | `10` | Number of snapshots to show |

**Example:**

```bash
# List last 10 snapshots
orkee cloud list

# List last 50 snapshots
orkee cloud list --limit 50
```

**Output:**

```
Cloud Snapshots
---------------
1. 2024-01-16 14:20:00 - my-app (Building)
2. 2024-01-16 10:30:00 - api-service (Launched)
3. 2024-01-15 18:45:00 - website (Planning)
```

#### Cloud Status

Show current cloud sync status.

```bash
orkee cloud status
```

**Output:**

```
Cloud Sync Status
-----------------
Status: Enabled
Last Sync: 2024-01-16 14:20:00
Projects Synced: 3/5
Pending Changes: 2 projects
Storage Used: 45 MB / 10 GB

Account: user@example.com
Plan: Pro
```

#### Cloud Login

Authenticate with Orkee Cloud.

```bash
orkee cloud login
```

**What It Does:**

- Opens browser for OAuth authentication
- Stores authentication token locally
- Enables cloud features

#### Cloud Logout

Sign out of Orkee Cloud.

```bash
orkee cloud logout
```

**What It Does:**

- Removes authentication token
- Disables cloud features
- Local data remains intact

## Global Options

These options work with any command:

| Flag | Description |
|------|-------------|
| `--help` or `-h` | Show help information |
| `--version` or `-V` | Show version number |
| `--verbose` or `-v` | Enable verbose logging |
| `--quiet` or `-q` | Suppress non-error output |

**Examples:**

```bash
# Show help for any command
orkee --help
orkee dashboard --help
orkee projects --help
orkee projects add --help

# Check version
orkee --version

# Verbose logging
orkee dashboard --verbose
orkee projects list -v

# Quiet mode
orkee cloud sync --quiet
```

## Environment Variables

Configure Orkee behavior via environment variables:

### Port Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_API_PORT` | `4001` | API server port |
| `ORKEE_UI_PORT` | `5173` | Dashboard UI port |

### Development Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_DEV_MODE` | `false` | Use local dashboard source |

### CORS Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_CORS_ORIGIN` | auto-calculated | Allowed CORS origin |
| `CORS_ALLOW_ANY_LOCALHOST` | `true` | Allow any localhost port (dev) |

### Cloud Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_CLOUD_TOKEN` | - | Orkee Cloud auth token |
| `ORKEE_CLOUD_API_URL` | `https://api.orkee.ai` | Cloud API endpoint |

### Security Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `BROWSE_SANDBOX_MODE` | `relaxed` | Path validation (strict/relaxed/disabled) |
| `ALLOWED_BROWSE_PATHS` | See below | Comma-separated allowed directories |
| `RATE_LIMIT_ENABLED` | `true` | Enable rate limiting |
| `SECURITY_HEADERS_ENABLED` | `true` | Enable security headers |

**Default Allowed Browse Paths:**
```
~/Documents,~/Projects,~/Desktop,~/Downloads
```

### TLS/HTTPS Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `TLS_ENABLED` | `false` | Enable HTTPS |
| `TLS_CERT_PATH` | `~/.orkee/certs/cert.pem` | TLS certificate path |
| `TLS_KEY_PATH` | `~/.orkee/certs/key.pem` | TLS private key path |
| `AUTO_GENERATE_CERT` | `true` | Auto-generate dev certs |

### Rate Limiting Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_HEALTH_RPM` | `60` | Health endpoint limit (per minute) |
| `RATE_LIMIT_BROWSE_RPM` | `20` | Directory browsing limit |
| `RATE_LIMIT_PROJECTS_RPM` | `30` | Projects API limit |
| `RATE_LIMIT_PREVIEW_RPM` | `10` | Preview operations limit |
| `RATE_LIMIT_GLOBAL_RPM` | `30` | Default limit for other endpoints |
| `RATE_LIMIT_BURST_SIZE` | `5` | Burst multiplier |

## Configuration Files

Orkee stores configuration and data in `~/.orkee/`:

```
~/.orkee/
├── orkee.db          # SQLite database (projects, settings)
├── auth.toml         # Cloud authentication credentials
├── certs/            # TLS certificates (if using HTTPS)
│   ├── cert.pem
│   └── key.pem
└── logs/             # Application logs
```

## Exit Codes

Orkee uses standard exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid arguments |
| `3` | Configuration error |
| `4` | Network error |
| `5` | Database error |

## Tips & Best Practices

### Port Management

```bash
# Check if ports are in use
lsof -i :4001
lsof -i :5173

# Use custom ports if defaults are taken
orkee dashboard --api-port 8080 --ui-port 3000

# Set ports permanently via environment
echo 'export ORKEE_API_PORT=8080' >> ~/.bashrc
echo 'export ORKEE_UI_PORT=3000' >> ~/.bashrc
```

### Process Management

```bash
# Check running Orkee processes
ps aux | grep orkee

# Kill all Orkee processes
pkill orkee

# Restart dashboard cleanly
orkee dashboard --restart
```

### Cloud Sync Workflow

```bash
# Initial setup
orkee cloud login
orkee cloud enable

# Daily workflow
orkee cloud sync        # Manual sync
orkee cloud status      # Check sync status

# Recovery
orkee cloud restore     # Restore from backup
```

### Debugging

```bash
# Enable verbose logging
RUST_LOG=debug orkee dashboard --verbose

# Check logs
tail -f ~/.orkee/logs/orkee.log

# Test API connectivity
curl http://localhost:4001/api/health
```

## Next Steps

- [Configuration Guide](../configuration/environment-variables) - Detailed configuration options
- [Security Settings](../configuration/security-settings) - Security configuration
- [Troubleshooting](../getting-started/troubleshooting) - Common issues and solutions
- [API Reference](../api-reference/overview) - REST API documentation
