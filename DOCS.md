# Orkee Documentation

This document provides comprehensive information about Orkee configuration, environment variables, security settings, and operational details.

## AI Architecture

Orkee uses a strict separation of concerns for AI operations following the **Chat Mode Pattern**:

- **Frontend (TypeScript)**: All AI calls via Vercel AI SDK (`generateObject()`, `streamText()`)
- **AI Proxy**: Frontend routes through `/api/ai/{provider}/*` for secure API key management
- **Backend (Rust)**: Pure CRUD operations only (save/retrieve data, NO AI calls)

**Frontend AI Services:**
- `chat-ai.ts` - Interactive PRD discovery
- `prd-ai.ts` - PRD document generation
- `research-ai.ts` - Competitor analysis
- `roundtable-ai.ts` - Expert discussions
- `dependency-ai.ts` - Dependency analysis

**Benefits:**
- Secure API key management (keys never leave backend)
- Streaming responses for better UX
- Type-safe AI schemas with Zod validation
- Consistent error handling and cost tracking
- Easy to add new AI providers

See `CLAUDE.md` and `rework-ai.md` for technical details and migration history.

## Table of Contents

1. [AI Architecture](#ai-architecture)
2. [Launch Modes](#launch-modes)
3. [Dashboard Distribution](#dashboard-distribution)
4. [Bundle Optimization](#bundle-optimization)
5. [Environment Variables](#environment-variables)
6. [API Authentication](#api-authentication)
7. [Cloud Sync Configuration](#cloud-sync-configuration)
8. [Security Configuration](#security-configuration)
9. [TLS/HTTPS Configuration](#tlshttps-configuration)
10. [File Locations & Data Storage](#file-locations--data-storage)
11. [CLI Commands Reference](#cli-commands-reference)
12. [API Reference](#api-reference)
13. [Default Ports & URLs](#default-ports--urls)
14. [Development vs Production](#development-vs-production)
15. [Troubleshooting](#troubleshooting)

## Launch Modes

Orkee provides two distinct ways to run the dashboard interface, each optimized for different use cases:

### 1. Desktop Application (Tauri)

The Tauri desktop application provides a native app experience with everything bundled together.

#### Quick Start
```bash
# From root directory
bun dev:tauri

# Or from dashboard directory
cd packages/dashboard
bun tauri dev

# Or for production build
cd packages/dashboard
bun tauri build
```

#### Features
- **Native Application**: Runs as a desktop app with system integration
- **Self-Contained**: Includes both frontend and backend (CLI binary as sidecar)
- **Fast Startup**: Everything is bundled, no download needed
- **Hot Reload**: Full development experience with Vite HMR
- **Cross-Platform**: Builds for macOS, Windows, and Linux

#### Architecture
- **Frontend**: React app served via Tauri's webview
- **Backend**: CLI binary included as sidecar process
- **Communication**: IPC bridge between frontend and backend
- **Bundle Size**: ~18 MB (includes 8.9 MB CLI binary + UI assets)

### 2. Web Dashboard (Browser)

The web dashboard runs in any modern browser with the backend API server.

#### Quick Start
```bash
# From root directory (recommended)
bun dev:web

# Or using CLI directly
cd packages/cli
cargo run --bin orkee -- dashboard

# With development mode (hot reload)
cargo run --bin orkee -- dashboard --dev

# With custom ports
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000
```

#### Features
- **Browser-Based**: Runs in any modern web browser
- **Remote Access**: Can be accessed from other devices on network
- **Two Modes**:
  - **Production (Dist)**: Pre-built static files, fast loading
  - **Development (Source)**: Live reload with Vite dev server
- **Flexible Deployment**: Can run behind reverse proxies

#### Architecture
- **Frontend**: React SPA served by Vite (dev) or static server (prod)
- **Backend**: Axum REST API server on separate port
- **Communication**: HTTP/HTTPS with CORS configuration
- **Download Size**:
  - Dist mode: ~2-5 MB (pre-built assets)
  - Source mode: ~100 MB (with dependencies)
  - CLI binary: 8.9 MB (one-time download)

### Comparison Table

| Feature | Desktop (Tauri) | Web Dashboard |
|---------|----------------|---------------|
| **Launch Command** | `bun dev:tauri` | `bun dev:web` |
| **Platform** | Desktop app | Web browser |
| **Installation** | Single app bundle | CLI + dashboard download |
| **Startup Time** | Faster (bundled) | Slower (may need download) |
| **Hot Reload** | ✅ Yes | ✅ Yes (dev mode) |
| **Remote Access** | ❌ No | ✅ Yes |
| **System Integration** | ✅ Native menus, tray | ❌ Browser limitations |
| **Bundle Size** | 18 MB total | 8.9 MB CLI + 2-5 MB dashboard |
| **Offline Work** | ✅ Fully offline | ✅ After initial download |
| **Update Method** | App update | CLI update + dashboard refresh |

### Which to Choose?

**Choose Desktop (Tauri) if you:**
- Want a native desktop application experience
- Prefer faster startup times
- Need system tray integration (future feature)
- Don't need remote access
- Want everything in a single package

**Choose Web Dashboard if you:**
- Prefer browser-based interfaces
- Need remote access capabilities
- Want smaller initial download
- Need to run behind reverse proxy
- Want to integrate with existing web infrastructure

## Dashboard Distribution

Orkee uses a hybrid distribution system that optimizes for both end-users and developers.

### Distribution Modes

#### 1. Pre-Built Distribution (Production)
Optimized for end-users with minimal download size and fast startup.

**Characteristics:**
- **Download Size**: ~2-5 MB
- **Content**: Minified, bundled JavaScript/CSS/HTML
- **Startup**: Instant (static file serving)
- **Requirements**: No Node.js/Bun needed
- **Hot Reload**: Not available

**How it works:**
1. User runs `orkee dashboard`
2. CLI checks for cached dashboard in `~/.orkee/dashboard`
3. If not found, downloads `orkee-dashboard-dist.tar.gz` from GitHub
4. Extracts pre-built files to cache directory
5. Serves static files on configured port

#### 2. Source Distribution (Development)
Full development environment with hot reload capabilities.

**Characteristics:**
- **Download Size**: ~100 MB (with dependencies)
- **Content**: Source TypeScript/React components
- **Startup**: Slower (needs compilation)
- **Requirements**: Bun/Node.js required
- **Hot Reload**: Full Vite HMR support

**How it works:**
1. User runs `orkee dashboard --dev`
2. CLI checks for local dashboard in `packages/dashboard`
3. If not found, downloads `orkee-dashboard-source.tar.gz`
4. Installs dependencies with `bun install`
5. Starts Vite dev server with hot reload

### Mode Selection

The CLI automatically selects the appropriate mode:

```bash
# End users: Pre-built distribution (fast, small)
orkee dashboard

# Developers: Source distribution (hot reload)
orkee dashboard --dev

# Force dev mode via environment
ORKEE_DEV_MODE=true orkee dashboard

# Explicit path override
ORKEE_DASHBOARD_PATH=/custom/path orkee dashboard
```

### Fallback Strategy

The CLI implements intelligent fallbacks:

1. **Production Mode** (`orkee dashboard`):
   - Try pre-built distribution first
   - Fall back to source if pre-built unavailable
   - Useful during development releases

2. **Development Mode** (`orkee dashboard --dev`):
   - Look for local `packages/dashboard` first
   - Fall back to downloaded source
   - Always uses source mode for hot reload

### Caching Strategy

Dashboard assets are cached to minimize downloads:

**Cache Location**: `~/.orkee/dashboard/`

**Version Tracking**:
- `.version` file contains installed version
- `.mode` file tracks distribution mode (dist/source)
- Automatic re-download when version changes

**Cache Management**:
```bash
# Clean dashboard cache
rm -rf ~/.orkee/dashboard

# Force re-download
orkee dashboard --restart
```

### GitHub Release Artifacts

Each release includes two dashboard packages:

1. **`orkee-dashboard-dist.tar.gz`** (~2-5 MB)
   - Pre-built production bundle
   - Minified and optimized
   - Ready to serve

2. **`orkee-dashboard-source.tar.gz`** (~600 KB)
   - Source code only
   - Requires dependency installation
   - Enables development features

### Build Process

The GitHub Actions workflow handles both distributions:

```yaml
# Build and package dashboard
- name: Build Dashboard
  run: |
    cd packages/dashboard
    bun install
    bun run build  # Creates dist/ folder

- name: Package Distributions
  run: |
    # Pre-built distribution
    tar czf orkee-dashboard-dist.tar.gz dist/

    # Source distribution
    tar czf orkee-dashboard-source.tar.gz \
      --exclude="node_modules" \
      --exclude="dist" \
      .
```

## Bundle Optimization

Orkee has been optimized to minimize bundle sizes across all components.

### Binary Size Optimization

#### Rust Compilation Profile
The workspace uses aggressive optimization settings:

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-Time Optimization
codegen-units = 1   # Better optimization
strip = true        # Strip debug symbols
panic = "abort"     # Smaller panic handler
```

#### Results
| Component | Before | After | Reduction |
|-----------|--------|-------|-----------|
| CLI Binary (debug) | 66 MB | - | - |
| CLI Binary (release) | 24 MB | 8.9 MB | **63%** |
| Tauri Bundle | 75 MB | 18 MB | **76%** |

### Dashboard Optimization

#### Production Build
- **Tree Shaking**: Removes unused code
- **Minification**: Compresses JavaScript/CSS
- **Code Splitting**: Lazy loads routes
- **Asset Optimization**: Compresses images

#### Dependency Management
- **Production Mode**: `bun install --production`
- **Excludes**: Dev dependencies (TypeScript, ESLint)
- **Result**: ~50% smaller node_modules

### Distribution Size Comparison

| Distribution Type | Size | Content |
|------------------|------|---------|
| **CLI Binary** | 8.9 MB | Standalone executable |
| **Dashboard Dist** | 2-5 MB | Minified production build |
| **Dashboard Source** | 600 KB | Source code (no deps) |
| **Dashboard + Deps** | 100 MB | Source + prod dependencies |
| **Dashboard + Dev Deps** | 215 MB | Source + all dependencies |
| **Tauri Desktop** | 18 MB | Complete app bundle |

### Optimization Strategies

#### 1. Conditional Features
```bash
# Build without cloud features (smaller)
cargo build --release

# Build with cloud features
cargo build --release --features cloud
```

#### 2. Dynamic Downloads
- Dashboard downloaded only when needed
- Version-based caching prevents re-downloads
- Dist mode for users, source for developers

#### 3. Shared Dependencies
- Workspace-level Cargo dependencies
- Centralized TypeScript configs
- Monorepo with Turborepo caching

#### 4. Asset Loading
- Lazy route loading in React
- Dynamic imports for heavy components
- CDN potential for static assets

### Monitoring Bundle Size

#### Build-Time Analysis
```bash
# Analyze Rust binary size
cargo bloat --release --bin orkee

# Analyze JavaScript bundle
cd packages/dashboard
bun run build --analyze
```

#### CI Size Limits
The GitHub workflow can enforce size limits:
```yaml
- name: Check Binary Size
  run: |
    SIZE=$(stat -f%z target/release/orkee)
    if [ $SIZE -gt 10485760 ]; then  # 10 MB limit
      echo "Binary too large: $SIZE bytes"
      exit 1
    fi
```

### Future Optimizations

**Planned improvements:**
- WebAssembly modules for compute-heavy tasks
- Service worker for offline caching
- Brotli compression for assets
- Native Axum static file serving (remove Python dependency)
- CDN distribution option

## Environment Variables

### Overview: Settings Management

Orkee uses a hybrid configuration approach:
- **Bootstrap settings** (ports, dev mode) must be in `.env` - they control how the application starts
- **Runtime settings** (security, rate limiting, TLS, etc.) are managed via the Settings UI in the dashboard
- Settings configured via the UI persist in the database and take effect after restart

### Bootstrap Variables (Required in .env)

These variables control application startup and cannot be changed at runtime. They appear as read-only in the Settings UI.

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_API_PORT` | `4001` | API server port (can be overridden by `--api-port` flag) |
| `ORKEE_UI_PORT` | `5173` | Dashboard UI port (can be overridden by `--ui-port` flag) |
| `ORKEE_DEV_MODE` | `false` | Enable development mode for dashboard (uses source with hot reload) |
| `ORKEE_DASHBOARD_PATH` | Auto-detected | Explicit path to dashboard directory (overrides auto-detection) |

### Database-Managed Settings (Configure via Settings UI)

These settings are managed through the dashboard's Settings page and stored in the database. Changes require a server restart to take effect.

**To configure these settings:**
1. Start Orkee: `orkee dashboard`
2. Navigate to the Settings tab
3. Configure via the UI - changes persist automatically

#### Settings > Security
- CORS configuration (allow any localhost)
- Directory browsing paths and sandbox mode
- Security headers (HSTS, request ID, etc.)

#### Settings > Advanced
- Rate limiting for all endpoints (health, browse, projects, preview, AI, global)
- Burst size configuration
- TLS/HTTPS settings
- Certificate paths and auto-generation

#### Settings > Cloud
- Cloud sync enabled/disabled
- Cloud API URL

### Example .env Configuration

**Minimal configuration (recommended):**
```bash
# Bootstrap Configuration (Required)
ORKEE_API_PORT=4001       # API server port
ORKEE_UI_PORT=5173        # Dashboard UI port
ORKEE_DEV_MODE=false      # Development mode

# Cloud Authentication (Optional - for cloud sync)
# ORKEE_CLOUD_TOKEN=ok_live_abc123...
```

**Note**: All security, rate limiting, and TLS settings are now configured via the Settings UI. See the Settings page in the dashboard for the complete list of available options.

### API Authentication

Orkee uses API token authentication to secure API endpoints. The system is designed for local-first desktop applications with automatic token management.

#### How It Works

1. **Automatic Token Generation**: On first startup, Orkee automatically generates a secure API token
2. **Token Storage**: Token saved to `~/.orkee/api-token` (file permissions: 0600)
3. **Database Storage**: Token hash (SHA-256) stored in SQLite for verification
4. **Automatic Authentication**: Desktop app automatically includes token in all API requests
5. **Whitelisted Endpoints**: Health and status endpoints don't require authentication
6. **Development Mode Bypass**: Authentication skipped when `ORKEE_DEV_MODE=true` (see below)

#### Development Mode

When running `orkee dashboard --dev` (or manually setting `ORKEE_DEV_MODE=true`):

- **Authentication is completely bypassed** for all API endpoints
- **No token required** - useful for web dashboard development
- **Localhost only** - server binds to 127.0.0.1, not accessible from network
- **Production mode** - Full authentication required for Tauri desktop app

**Use Cases**:
- Web dashboard development (`bun run dev` in `packages/dashboard/`)
- Local API testing without managing tokens
- Rapid prototyping and debugging

**Security**: Development mode should only be used on localhost for development. Production deployments must not set `ORKEE_DEV_MODE`.

#### Token File Location

| Platform | Token File Path |
|----------|----------------|
| macOS/Linux | `~/.orkee/api-token` |
| Windows | `%USERPROFILE%\.orkee\api-token` |

#### Authentication Headers

All protected API endpoints require the `X-API-Token` header:

```bash
# Read token from file
export TOKEN=$(cat ~/.orkee/api-token)

# Make authenticated request
curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/projects
```

#### Whitelisted Endpoints (No Auth Required)

The following endpoints are accessible without authentication:
- `GET /api/health` - Basic health check
- `GET /api/status` - Detailed service status
- `GET /api/csrf-token` - CSRF token retrieval

#### Protected Endpoints (Auth Required)

All other API endpoints require authentication:
- Projects API - `/api/projects/*`
- Settings API - `/api/settings/*`
- Preview Servers - `/api/preview/*`
- Directory Browsing - `/api/browse-directories`
- Tasks & Specs - `/api/tasks/*`, `/api/specs/*`

#### Desktop App Integration

The Tauri desktop app handles authentication automatically:
1. Reads token from `~/.orkee/api-token` on startup
2. Includes `X-API-Token` header in all API requests
3. No user configuration required

#### Manual API Testing

For development or scripting, you can manually authenticate:

```bash
# Example: List projects
curl -H "X-API-Token: $(cat ~/.orkee/api-token)" \
  http://localhost:4001/api/projects

# Example: Create project
curl -X POST \
  -H "X-API-Token: $(cat ~/.orkee/api-token)" \
  -H "Content-Type: application/json" \
  -d '{"name": "My Project", "projectRoot": "/path/to/project"}' \
  http://localhost:4001/api/projects
```

#### Security Features

- **SHA-256 Hashing**: Tokens stored as hashes in database
- **Constant-Time Comparison**: Prevents timing attacks during verification
- **File Permissions**: Token file readable only by owner (Unix)
- **Single-Use Display**: Token shown once during generation

#### Troubleshooting Authentication

**401 Unauthorized Errors**:
1. Verify token file exists: `cat ~/.orkee/api-token`
2. Check token is included in request headers
3. Verify server is running: `curl http://localhost:4001/api/health`

**Token File Missing**:
1. Stop Orkee server
2. Delete database: `rm ~/.orkee/orkee.db`
3. Restart server - new token will be generated

For complete authentication documentation, see [API_SECURITY.md](API_SECURITY.md).

### Dashboard Variables

These variables configure the React dashboard frontend:

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_ORKEE_API_PORT` | `4001` | API server port (passed from CLI via environment) |
| `VITE_API_URL` | Auto-constructed from port | Backend API URL (defaults to `http://localhost:${VITE_ORKEE_API_PORT}`) |

#### Example dashboard .env:
```bash
# Usually auto-configured by the CLI, but can be overridden if needed
# VITE_API_URL=http://localhost:4001  # Only set this if you need a custom URL
```

### Dashboard Tauri Configuration

These variables configure the Tauri desktop application:

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_TRAY_POLL_INTERVAL_SECS` | `5` | System tray polling interval in seconds (min: 1, max: 60) - controls how often the tray menu checks for server status updates |
| `ORKEE_API_HOST` | `localhost` | API host for tray connections - for security, only localhost is allowed unless `ORKEE_ALLOW_REMOTE_API` is enabled |
| `ORKEE_ALLOW_REMOTE_API` | `false` | Enable remote API access - allows connecting to non-localhost API hosts (not recommended for security) |

#### Security Note
The Tauri desktop app launches and manages its own local Orkee CLI server process as a sidecar. By default, it only connects to `localhost` for security. Remote API access should only be enabled in trusted environments.

### Preview Server Configuration

These variables control the preview server registry and process management:

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_STALE_TIMEOUT_MINUTES` | `5` | Timeout before server entries are considered stale (max: 240 minutes) - controls when inactive servers are cleaned up from the registry |
| `ORKEE_PROCESS_START_TIME_TOLERANCE_SECS` | `5` | Tolerance for process start time validation (max: 60 seconds) - helps detect PID reuse on systems under heavy load |

### Real-Time Updates (SSE) Configuration (Optional)

Configure Server-Sent Events (SSE) for real-time server status updates:

| Variable | Default | Range/Notes | Description |
|----------|---------|-------------|-------------|
| `ORKEE_EVENT_CHANNEL_SIZE` | `200` | 10-10000 | Broadcast channel capacity for SSE events. Increase for CI/CD environments with bulk server operations. |
| `VITE_SSE_MAX_RETRIES` | `3` | - | Maximum SSE connection retry attempts before falling back to polling |
| `VITE_SSE_RETRY_DELAY` | `2000` | milliseconds | Delay between SSE retry attempts |
| `VITE_SSE_POLLING_INTERVAL` | `5000` | milliseconds | Polling interval when SSE connection fails |

**Use Cases**:
- **CI/CD Environments**: Increase `ORKEE_EVENT_CHANNEL_SIZE` to 500-1000 for handling bulk server start/stop operations
- **Poor Network Conditions**: Increase `VITE_SSE_RETRY_DELAY` and `VITE_SSE_MAX_RETRIES` for more resilient connections
- **Low-Latency Networks**: Decrease `VITE_SSE_POLLING_INTERVAL` for faster polling fallback

### AI Model Configuration (Optional)

Orkee supports multiple AI providers for different tasks. Configure model preferences via **Settings > AI Models** in the dashboard to select which AI provider and model to use for each task type (chat, PRD generation, insight extraction, etc.).

**Supported AI Providers:**

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | **Recommended** | Claude API key (format: `sk-ant-api03-...`) - Default for most tasks |
| `OPENAI_API_KEY` | Optional | OpenAI API key (format: `sk-proj-...`) - GPT-4, GPT-3.5 models |
| `GOOGLE_API_KEY` | Optional | Google Gemini API key - Gemini Pro, Flash models |
| `XAI_API_KEY` | Optional | xAI API key - Grok models |
| `PERPLEXITY_API_KEY` | Optional | Perplexity API for research features (format: `pplx-...`) |
| `MISTRAL_API_KEY` | Optional | Mistral AI API key |
| `GROQ_API_KEY` | Optional | Groq API key |
| `OPENROUTER_API_KEY` | Optional | OpenRouter API key |

**Task-Specific Model Selection:**

Configure different models for different tasks in Settings > AI Models:
- **Chat (Ideate)** - Interactive PRD discovery conversations
- **PRD Generation** - Creating complete PRD documents
- **PRD Analysis** - Analyzing and improving PRDs
- **Insight Extraction** - Extracting key insights from chat
- **Spec Generation** - Creating technical specifications
- **Task Suggestions** - AI-powered task recommendations
- **Task Analysis** - Analyzing task complexity and requirements
- **Spec Refinement** - Improving technical specifications
- **Research Generation** - Competitive analysis and research
- **Markdown Generation** - Converting content to markdown format

**Example .env configuration:**
```bash
# Configure API keys for the providers you want to use
ANTHROPIC_API_KEY=sk-ant-api03-...
OPENAI_API_KEY=sk-proj-...
GOOGLE_API_KEY=AIza...
XAI_API_KEY=xai-...

# Then configure model preferences via Settings > AI Models in the dashboard
```

**Note**: API keys are required only for the providers you select in model preferences. The system will use sensible defaults (Claude Sonnet 4) if no preferences are configured.

### Cloud Sync Variables (Orkee Cloud)

Configure Orkee Cloud integration for backup and synchronization:

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_CLOUD_API_URL` | `https://api.orkee.ai` | Orkee Cloud API base URL |
| `ORKEE_CLOUD_TOKEN` | - | Authentication token (set via `orkee cloud login`) |

#### Example cloud sync .env:
```bash
# Orkee Cloud Configuration
ORKEE_CLOUD_API_URL=https://api.orkee.ai

# Note: ORKEE_CLOUD_TOKEN is set automatically via `orkee cloud login`
# Do not set manually - use the OAuth authentication flow
```

**Note**: Authentication is handled through OAuth. Use `orkee cloud login` to authenticate, which will securely store your token in `~/.orkee/auth.toml`.

#### Additional Task Master Variables:
| Variable | Required | Description |
|----------|----------|-------------|
| `AZURE_OPENAI_API_KEY` | Optional | Azure OpenAI API key |
| `OLLAMA_API_KEY` | Optional | Ollama API key (for remote servers) |
| `GITHUB_API_KEY` | Optional | GitHub API for import/export (format: `ghp_...` or `github_pat_...`) |

## Cloud Sync Configuration

Orkee features a SQLite-first architecture with optional cloud synchronization capabilities. Data is stored locally in SQLite with full offline functionality, and can optionally be backed up and synchronized to Orkee Cloud.

**⚠️ Cloud Sync Requirements**: Cloud sync functionality is provided by the `orkee-cloud` package and requires compilation with the `--features cloud` flag. This makes cloud functionality optional, keeping the binary smaller when cloud sync is not needed.

```bash
# Build with cloud sync features
cargo build --features cloud

# Or build without cloud features (smaller binary)
cargo build
```

### Orkee Cloud Integration

- **Direct API Integration**: Simple, clean integration with Orkee Cloud API
- **OAuth Authentication**: Secure browser-based authentication flow
- **Project Synchronization**: Seamless sync of your project data
- **Multi-device Access**: Access your projects from any device

### Getting Started with Cloud Sync

1. **Authenticate with Orkee Cloud**:
   ```bash
   orkee cloud login
   # This opens your browser for OAuth authentication
   ```

2. **Sync your projects**:
   ```bash
   orkee cloud sync
   ```

3. **Check sync status**:
   ```bash
   orkee cloud status
   ```

### Cloud Sync Features

- **🔐 OAuth Security**: Secure browser-based authentication
- **🔑 Token Management**: Secure token storage in `~/.orkee/auth.toml`
- **🔄 Project Sync**: Seamless synchronization of project data
- **📊 Multi-device**: Access your projects from anywhere
- **⚡ Fast Access**: Direct API integration for responsive sync

### Authentication Architecture

- **OAuth Flow**: Standard OAuth 2.0 with PKCE for security
- **Token Storage**: Local secure storage with automatic expiry handling
- **API Integration**: Direct REST API calls to Orkee Cloud
- **Error Handling**: Comprehensive error messages and recovery

### Configuration Files

Cloud authentication is stored in `~/.orkee/auth.toml`:

```toml
# This file is managed automatically by `orkee cloud login`
# Do not edit manually

token = "orkee_abc123..."
expires_at = "2025-01-01T12:00:00Z"
user_email = "user@example.com"
user_name = "User Name"
user_id = "user-123"
```

### Cloud CLI Commands Reference

**Note**: All cloud commands require the CLI to be built with `--features cloud`.

| Command | Description |
|---------|-------------|
| `orkee cloud login` | Authenticate with Orkee Cloud (OAuth flow) |
| `orkee cloud logout` | Sign out from Orkee Cloud |
| `orkee cloud status` | Show authentication and sync status |
| `orkee cloud enable` | Enable cloud features |
| `orkee cloud disable` | Disable cloud features |
| `orkee cloud sync [--project <id>]` | Sync projects to cloud (all or specific) |
| `orkee cloud list` | List cloud projects |
| `orkee cloud restore --project <id>` | Restore project from cloud |

## AI Usage Tracking

Orkee provides comprehensive tracking of all AI operations, including token usage, costs, and tool invocations.

### Features

- **Automatic Tracking**: All AI SDK calls tracked automatically (zero manual logging)
- **Cost Monitoring**: Real-time cost tracking across AI providers (Anthropic, OpenAI, Google, xAI)
- **Tool Analytics**: Track tool invocations, success rates, and performance metrics
- **Usage Dashboard**: Visual analytics with charts for tokens, costs, and tool usage over time
- **Per-Operation Metrics**: Track specific operations (PRD generation, chat, analysis, etc.)

### Tracked Metrics

For every AI operation, Orkee automatically tracks:
- **Tokens**: Input, output, and total token counts
- **Cost**: Estimated cost based on provider pricing
- **Duration**: Actual request duration with high-precision timing
- **Tool Calls**: Which tools were invoked, arguments, results, and success/failure status
- **Model/Provider**: Which AI model and provider was used
- **Metadata**: Finish reason, response ID, and provider-specific metadata

### Accessing Usage Data

View comprehensive usage analytics in the **Usage** tab of the Orkee dashboard:

**Overview Tab:**
- Key metrics cards (total requests, tokens, costs, tool calls)
- Model breakdown by token usage
- Provider breakdown by cost
- Most used tools with success rates

**Charts & Analytics Tab:**
- Time-series visualizations for requests, tokens, and costs
- Tool usage bar charts (call counts, success/failure rates)
- Tool performance metrics (average duration)
- Model distribution pie chart
- Provider cost distribution

### Database Storage

All usage data is stored locally in `~/.orkee/orkee.db` in the `ai_usage_logs` table:
- Full-text search support for querying historical data
- Tool call data stored as JSON with full arguments and results
- Response metadata for debugging and analysis
- No data sent externally - completely local tracking

### API Endpoints

Access usage data programmatically:
- `GET /api/ai-usage/stats` - Aggregate statistics
- `GET /api/ai-usage/tools` - Tool usage breakdown
- `GET /api/ai-usage/time-series` - Historical data for charts
- `POST /api/ai-usage` - Log telemetry (used automatically by frontend)

### For Developers

When adding new AI functionality, all tracking happens automatically via telemetry wrappers. See the [AI Usage Implementation Plan](ai-usage.md) for technical details and the developer guide in README.md.

## Telemetry

Orkee includes optional, privacy-first telemetry to help improve the product. All telemetry is **opt-in** and disabled by default.

### User Privacy

- **Completely Opt-In**: All telemetry is disabled until you explicitly enable it
- **Granular Controls**: Choose what to share (errors, usage, or nothing)
- **Anonymous by Default**: Only an anonymous machine ID is used
- **Local Storage**: Data buffered locally before transmission
- **Transparent**: Full source code available in `packages/cli/src/telemetry/`

### What is NOT Collected

We never collect:
- Personal information (name, email, address)
- File contents or source code
- Project names or file paths
- Credentials, API keys, or secrets
- Browsing history

### What IS Collected (When Opted-In)

**Error Reporting** (optional):
- Error messages and stack traces
- Application version and platform
- Anonymous machine ID

**Usage Metrics** (optional):
- Feature usage (e.g., "project created")
- Application version and platform
- Anonymous machine ID

### Disabling Telemetry

Telemetry can be disabled:

1. **During First Run**: Decline during the onboarding dialog
2. **In Settings**: Toggle telemetry options in application settings
3. **Via Environment**: `export ORKEE_TELEMETRY_ENABLED=false`
4. **Delete Data**: Remove all collected data:
   ```bash
   sqlite3 ~/.orkee/orkee.db "DELETE FROM telemetry_events; DELETE FROM telemetry_stats;"
   ```

For complete implementation details and maintainer documentation, see [`TELEMETRY.md`](./TELEMETRY.md).

## Security Configuration

### Directory Browsing Sandbox

Orkee implements a comprehensive directory browsing security system with three modes:

#### Strict Mode (`BROWSE_SANDBOX_MODE=strict`)
- **Allowlist only**: Only paths in `ALLOWED_BROWSE_PATHS` are accessible
- **Root access blocked**: No access to system root unless explicitly allowed
- **Path traversal blocked**: `../` navigation is completely blocked
- **Use case**: High-security environments

#### Relaxed Mode (`BROWSE_SANDBOX_MODE=relaxed`) - **Default**
- **Blocklist approach**: Block dangerous system paths but allow most user paths
- **Path traversal allowed**: `../` navigation permitted within safe boundaries
- **Home directory allowed**: Full access to user's home directory
- **Use case**: Local development with reasonable security

#### Disabled Mode (`BROWSE_SANDBOX_MODE=disabled`) - **NOT RECOMMENDED**
- **No restrictions**: Access to any readable directory
- **Security warning**: Only use in completely trusted environments
- **Use case**: Debugging or specialized use cases only

### Blocked Paths (All Modes)

The following system directories are **always** blocked for security:

**System Directories:**
- `/etc`, `/private/etc` (configuration files)
- `/sys`, `/proc`, `/dev` (system filesystems)
- `/usr/bin`, `/usr/sbin`, `/bin`, `/sbin` (system binaries)
- `/var/log`, `/var/run`, `/var/lock` (system runtime)
- `/boot`, `/root`, `/mnt`, `/media`, `/opt`
- `/tmp`, `/var/tmp` (temporary directories)

**Windows System Directories:**
- `C:\Windows`, `C:\Program Files`, `C:\Program Files (x86)`
- `C:\ProgramData`, `C:\System32`

**Sensitive User Directories (relative to home):**
- `.ssh`, `.aws`, `.gnupg`, `.docker`, `.kube`
- `.config/git`, `.npm`, `.cargo/credentials`
- `.gitconfig`, `.env*` files
- `Library/Keychains` (macOS)
- `AppData/Local/Microsoft` (Windows)

### CORS Configuration

CORS is configured for local development security:

- **Allowed Origins**: Only `localhost` origins are permitted
- **Port Whitelist**: Ports 3000-3099, 4000-4199, 5000-5999, 8000-8099
- **Headers**: Restricted to essential headers only
- **Methods**: GET, POST, PUT, DELETE, OPTIONS

### Input Validation

All user inputs are validated with:

- **Length Limits**: Project names (100 chars), descriptions (1000 chars)
- **Pattern Validation**: Alphanumeric with safe special characters
- **Command Filtering**: Detection of dangerous shell commands
- **Path Safety**: Prevention of path traversal attacks
- **Script Injection**: Protection against code injection in scripts

### API Key Encryption

Orkee encrypts API keys stored in the database using ChaCha20-Poly1305 AEAD with two available modes:

####  Machine-Based Encryption (Default)

- **Use Case**: Personal use, single-user environments
- **Key Derivation**: HKDF-SHA256 from machine ID + application salt
- **Security Model**: Transport encryption for backup/sync
  - Protects data during file transfer and synchronization
  - **Does NOT provide at-rest encryption** on the local machine
  - Anyone with local database file access can decrypt keys
- **Backward Compatible**: Existing installations continue working without changes

#### Password-Based Encryption (Opt-in)

- **Use Case**: Shared machines, sensitive environments
- **Key Derivation**: Argon2id with recommended security parameters
  - Memory: 64 MB (m_cost=65536)
  - Iterations: 3 (t_cost=3)
  - Parallelism: 4 threads (p_cost=4)
- **Security Model**: True at-rest encryption
  - Data cannot be decrypted without the user's password
  - Password required when accessing encrypted data
  - Separate verification hash prevents authentication bypass

#### Migration Commands (Future)

```bash
orkee security set-password       # Upgrade to password-based encryption
orkee security change-password    # Change existing password
orkee security remove-password    # Downgrade to machine-based
orkee security status             # Show current encryption mode
```

#### Implementation Details

- **File**: `packages/projects/src/security/encryption.rs`
- **Database**: `~/.orkee/orkee.db` - `encryption_settings` table
- **Encryption Algorithm**: ChaCha20-Poly1305 with 256-bit keys
- **Nonce Generation**: Cryptographically secure random (SystemRandom)
- **Verification**: Separate hash with context differentiation

### Security Middleware

Orkee implements production-grade security middleware:

#### Rate Limiting
- **Per-IP tracking**: Token bucket algorithm with configurable limits
- **Endpoint-specific limits**: Different rates for health, browsing, projects, and preview operations
- **429 responses**: Proper `Too Many Requests` with `Retry-After` headers
- **Burst protection**: Configurable burst sizes to handle legitimate traffic spikes
- **Audit logging**: All rate limit violations logged for monitoring

#### Security Headers
Automatically applied to all responses:
- **Content Security Policy (CSP)**: Restrictive policy allowing development workflows
- **X-Content-Type-Options**: `nosniff` to prevent MIME type sniffing
- **X-Frame-Options**: `DENY` to prevent clickjacking
- **X-XSS-Protection**: Legacy XSS protection
- **Referrer-Policy**: `strict-origin-when-cross-origin`
- **Permissions-Policy**: Disables dangerous browser APIs (geolocation, camera, etc.)
- **HSTS** (optional): HTTP Strict Transport Security for HTTPS deployments

#### Error Handling & Logging
- **Sanitized responses**: No internal details leaked to clients
- **Request ID tracking**: Unique IDs for audit trail correlation
- **Structured logging**: JSON-formatted logs with security event flagging
- **Panic recovery**: Graceful handling of server panics with safe responses
- **Consistent format**: All API errors follow standard `{success, error, request_id}` format

## TLS/HTTPS Configuration

Orkee supports TLS/HTTPS encryption for secure connections. When enabled, the server runs in dual mode with automatic HTTP-to-HTTPS redirect.

### TLS Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TLS_ENABLED` | `false` | Enable/disable HTTPS/TLS support |
| `TLS_CERT_PATH` | `~/.orkee/certs/cert.pem` | Path to TLS certificate file |
| `TLS_KEY_PATH` | `~/.orkee/certs/key.pem` | Path to TLS private key file |
| `AUTO_GENERATE_CERT` | `true` | Auto-generate self-signed certificates for development |

### TLS Configuration Modes

#### Development Mode (Default)
- **Auto-generated certificates**: Self-signed certificates created automatically
- **Browser warnings**: Browsers will show security warnings (expected behavior)
- **Certificate validity**: 365 days, auto-renewed when within 30 days of expiry
- **Common Names**: `localhost`, `127.0.0.1`, `::1`, `orkee.local`, `dev.orkee.local`

#### Production Mode
- **Custom certificates**: Provide your own certificates via `TLS_CERT_PATH`/`TLS_KEY_PATH`
- **Valid certificates**: Use certificates from a trusted CA (Let's Encrypt, commercial CA)
- **No browser warnings**: Properly validated certificates won't trigger warnings

### Dual Server Architecture

When TLS is enabled, Orkee runs two servers simultaneously:

- **HTTPS Server** (main): Runs on configured port (default 4001) serving the full application
- **HTTP Redirect Server**: Runs on port-1 (default 4000) that redirects all traffic to HTTPS

#### Port Configuration
- **HTTPS Port**: Uses `PORT` environment variable (default 4001)
- **HTTP Port**: Automatically uses `PORT - 1` (default 4000)
- **Custom Ports**: If using non-default HTTPS port, HTTP port will be HTTPS port - 1

### Example TLS Configuration

#### Basic HTTPS Setup
```bash
# Enable TLS with auto-generated certificates
TLS_ENABLED=true
AUTO_GENERATE_CERT=true
PORT=4001

# Security headers (recommended for HTTPS)
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
```

#### Custom Certificate Setup
```bash
# Enable TLS with custom certificates
TLS_ENABLED=true
AUTO_GENERATE_CERT=false
TLS_CERT_PATH="/path/to/your/certificate.pem"
TLS_KEY_PATH="/path/to/your/private-key.pem"
PORT=4001

# Security headers for production
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
```

### Certificate Management

#### Auto-Generated Certificates
- **Location**: `~/.orkee/certs/` directory
- **Filenames**: `cert.pem` (certificate), `key.pem` (private key)
- **Permissions**: Private key automatically set to 600 (owner read/write only)
- **Renewal**: Automatic renewal when certificate is within 30 days of expiry
- **Multiple domains**: Includes localhost, 127.0.0.1, IPv6 localhost, and orkee.local domains

#### Custom Certificates
- **Format**: PEM format for both certificate and private key
- **Chain Support**: Full certificate chains are supported
- **Key Types**: RSA, ECDSA, and Ed25519 keys supported
- **Validation**: Certificates validated on startup, server won't start with invalid certificates

### HTTPS Redirect Behavior

The HTTP redirect server provides:
- **301 Permanent Redirect**: All HTTP requests redirected with 301 status
- **Preserve Paths**: Original path and query parameters preserved in redirect
- **Host Preservation**: Original host header preserved (configurable)
- **Custom Ports**: Non-standard HTTPS ports properly handled in redirect URLs
- **Proxy Support**: X-Forwarded-Proto header detection for reverse proxy setups

### Dashboard Configuration

When using HTTPS, update the dashboard configuration:

```bash
# Dashboard .env for HTTPS
VITE_API_URL=https://localhost:4001

# Or for custom domain
VITE_API_URL=https://dev.orkee.local:4001
```

### Troubleshooting TLS

#### Certificate Generation Issues
- **Permission errors**: Ensure write access to certificate directory
- **Port conflicts**: Check that both HTTP and HTTPS ports are available
- **Certificate validation**: Check server logs for certificate validation errors

#### Browser Issues
- **Self-signed warnings**: Expected with auto-generated certificates, add security exception
- **HSTS conflicts**: Clear browser HSTS cache if switching between HTTP/HTTPS
- **Mixed content**: Ensure all resources load over HTTPS when using HTTPS

#### Common Solutions
```bash
# Check certificate status
ls -la ~/.orkee/certs/

# Regenerate certificates (delete existing ones)
rm ~/.orkee/certs/cert.pem ~/.orkee/certs/key.pem

# Test HTTPS connection
curl -k https://localhost:4001/api/health

# Check server logs for TLS issues
orkee dashboard  # Watch for TLS-related log messages
```

## File Locations & Data Storage

### Configuration Files

| File | Purpose | Format |
|------|---------|--------|
| `~/.orkee/orkee.db` | Primary SQLite database | Binary SQLite |
| `~/.orkee/projects.json` | Legacy JSON storage (if enabled) | JSON |
| `.env` | Environment variables | Key=Value pairs |
| `.taskmaster/config.json` | Task Master AI configuration | JSON |

### Database Schema

The SQLite database (`~/.orkee/orkee.db`) contains:

- **projects**: Project metadata, paths, Git info
- **project_tags**: Many-to-many relationship for tags
- **tags**: Tag definitions
- **schema_migrations**: Version tracking

### Log Files

Audit logs are written to:
- **Format**: Structured JSON with timestamps
- **Location**: Standard output (captured by systemd/Docker in production)
- **Content**: Directory access attempts, security violations, API calls

Example log entry:
```json
{
  "timestamp": "2025-01-01T12:00:00Z",
  "user": "anonymous",
  "action": "browse_directory",
  "requested_path": "/home/user/Documents",
  "resolved_path": "/home/user/Documents",
  "allowed": true,
  "entries_count": 15,
  "source": "directory_browser"
}
```

## CLI Commands Reference

### Main Commands

#### Dashboard
Start the full dashboard (backend + frontend):
```bash
orkee dashboard [OPTIONS]

Options:
  -p, --port <PORT>              Server port [default: 4001]
      --cors-origin <CORS_ORIGIN> CORS origin [default: http://localhost:5173]
      --restart                  Restart services (kill existing first)
```

#### TUI (Terminal User Interface)
Launch the terminal interface:
```bash
orkee tui [OPTIONS]

Options:
      --refresh-interval <SECONDS>  Refresh interval [default: 20]
      --theme <THEME>               Theme: light, dark [default: dark]
```

### Project Management

#### List Projects
```bash
orkee projects list
```

#### Show Project Details
```bash
orkee projects show <PROJECT_ID>
```

#### Add New Project
```bash
orkee projects add [OPTIONS]

Options:
      --name <NAME>               Project name
      --path <PATH>               Project root path
      --description <DESCRIPTION> Project description
```

#### Edit Project
```bash
orkee projects edit <PROJECT_ID>
```

#### Delete Project
```bash
orkee projects delete <PROJECT_ID> [OPTIONS]

Options:
      --yes  Skip confirmation prompt
```

### Preview Management

#### Stop All Preview Servers
```bash
orkee preview stop-all
```

## API Reference

The CLI server provides a REST API on port 4001 (configurable).

### Response Format

All API responses follow this format:
```json
{
  "success": boolean,
  "data": any | null,
  "error": string | null
}
```

### Health & Status Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/health` | Basic health check |
| GET | `/api/status` | Detailed service status |

### Project Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/projects` | List all projects |
| GET | `/api/projects/:id` | Get project by ID |
| GET | `/api/projects/by-name/:name` | Get project by name |
| POST | `/api/projects/by-path` | Get project by path |
| POST | `/api/projects` | Create new project |
| PUT | `/api/projects/:id` | Update project |
| DELETE | `/api/projects/:id` | Delete project |

#### Project Data Structure
```json
{
  "id": "uuid",
  "name": "Project Name",
  "projectRoot": "/path/to/project",
  "status": "active" | "archived" | "draft",
  "priority": "high" | "medium" | "low",
  "createdAt": "2025-01-01T12:00:00Z",
  "updatedAt": "2025-01-01T12:00:00Z",
  "tags": ["tag1", "tag2"],
  "description": "Optional description",
  "setupScript": "Optional setup command",
  "devScript": "Optional dev command",
  "cleanupScript": "Optional cleanup command",
  "gitRepository": {
    "owner": "username",
    "repo": "reponame", 
    "url": "https://github.com/username/repo.git",
    "branch": "main"
  }
}
```

### Model Preferences Endpoints

Configure AI models per task type (chat, PRD generation, insight extraction, etc.)

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/model-preferences/:user_id` | Get user's model preferences for all tasks |
| PUT | `/api/model-preferences/:user_id` | Update all model preferences at once |
| PUT | `/api/model-preferences/:user_id/chat` | Update chat model configuration |
| PUT | `/api/model-preferences/:user_id/prd-generation` | Update PRD generation model |
| PUT | `/api/model-preferences/:user_id/prd-analysis` | Update PRD analysis model |
| PUT | `/api/model-preferences/:user_id/insight-extraction` | Update insight extraction model |
| PUT | `/api/model-preferences/:user_id/spec-generation` | Update spec generation model |
| PUT | `/api/model-preferences/:user_id/task-suggestions` | Update task suggestions model |
| PUT | `/api/model-preferences/:user_id/task-analysis` | Update task analysis model |
| PUT | `/api/model-preferences/:user_id/spec-refinement` | Update spec refinement model |
| PUT | `/api/model-preferences/:user_id/research-generation` | Update research generation model |
| PUT | `/api/model-preferences/:user_id/markdown-generation` | Update markdown generation model |

#### Model Configuration Request
```json
{
  "provider": "anthropic",  // anthropic, openai, google, xai
  "model": "claude-sonnet-4-20250514"  // Provider-specific model ID
}
```

#### Model Preferences Response
```json
{
  "success": true,
  "data": {
    "user_id": "default",
    "chat_model": "claude-sonnet-4-20250514",
    "chat_provider": "anthropic",
    "prd_generation_model": "claude-sonnet-4-20250514",
    "prd_generation_provider": "anthropic",
    "insight_extraction_model": "claude-sonnet-4-20250514",
    "insight_extraction_provider": "anthropic",
    // ... other task configurations
    "updated_at": "2025-01-01T12:00:00Z"
  },
  "error": null
}
```

**Note**: Configure model preferences via Settings > AI Models in the dashboard UI. Requires valid API keys for the selected providers (see [Task Master AI Variables](#task-master-ai-variables-optional)).

### Directory Browsing Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/api/browse-directories` | Browse filesystem directories |

#### Browse Request/Response
```json
// Request
{
  "path": "/path/to/browse" // optional, uses safe default if omitted
}

// Response
{
  "success": true,
  "data": {
    "currentPath": "/resolved/path",
    "parentPath": "/parent/path", // null if at sandbox root
    "directories": [
      {
        "name": "dirname",
        "path": "/full/path/to/dirname",
        "isDirectory": true
      }
    ],
    "isRoot": false,
    "separator": "/"
  },
  "error": null
}
```

### Preview Server Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/preview/health` | Preview service health |
| GET | `/api/preview/servers` | List active preview servers |
| POST | `/api/preview/servers/:project_id/start` | Start preview server |
| POST | `/api/preview/servers/:project_id/stop` | Stop preview server |
| GET | `/api/preview/servers/:project_id/status` | Get server status |
| GET | `/api/preview/servers/:project_id/logs` | Get server logs |
| POST | `/api/preview/servers/:project_id/logs/clear` | Clear server logs |
| POST | `/api/preview/servers/:project_id/activity` | Update activity timestamp |

## Default Ports & URLs

### Development Environment

| Service | Default Port | Configurable Via | URL | Purpose |
|---------|-------------|------------------|-----|---------|
| CLI Server | 4001 | `--api-port` or `ORKEE_API_PORT` | http://localhost:4001 | REST API backend |
| Dashboard | 5173 | `--ui-port` or `ORKEE_UI_PORT` | http://localhost:5173 | React frontend (dev) |
| Preview Servers | 5000-5999 | N/A | http://localhost:50XX | Dynamic project previews |

**Port Configuration Examples:**
```bash
# Use default ports
orkee dashboard

# Custom ports via CLI flags
orkee dashboard --api-port 8080 --ui-port 3000

# Custom ports via environment variables
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 orkee dashboard

# CLI flags override environment variables
ORKEE_API_PORT=9000 orkee dashboard --api-port 7777  # Uses 7777
```

### Production Environment

| Service | Port | Purpose |
|---------|------|---------|
| CLI Server | 80/443 | HTTP/HTTPS API |
| Dashboard | Served by CLI | Static files served by backend |

## Development vs Production

### Development Configuration
- CORS allows any localhost origin
- Detailed error messages
- Hot reloading enabled
- SQLite database in `~/.orkee/`
- Relaxed security policies

### Production Configuration
- Strict CORS policy
- Sanitized error messages
- Static file serving
- Database in persistent volume
- Enhanced security features:
  - Rate limiting
  - HTTPS enforcement
  - Security headers
  - Input sanitization
  - Audit logging

### Security Checklist for Production

- [ ] Configure strict CORS origins
- [ ] Enable HTTPS/TLS
- [ ] Set up rate limiting
- [ ] Configure security headers
- [ ] Set up log aggregation
- [ ] Enable audit logging
- [ ] Use environment-specific secrets
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategy
- [ ] Set up health checks

## Troubleshooting

### Common Issues

#### "Permission denied" errors
- Check `ALLOWED_BROWSE_PATHS` configuration
- Verify directory permissions
- Check sandbox mode setting

#### CORS errors in dashboard
- Verify `CORS_ORIGIN` matches dashboard URL
- Check `CORS_ALLOW_ANY_LOCALHOST` setting
- Ensure CLI server is running on correct port

#### Database connection errors
- Check `~/.orkee/` directory exists and is writable
- Verify SQLite file permissions
- Check disk space

#### Port conflicts
- Use `lsof -i :4001` to check port usage
- Change `PORT` environment variable
- Use `--port` flag with dashboard command

#### Preview server failures
- Check project has valid dev script
- Verify port availability (5000-5999 range)
- Check project directory permissions

#### Rate limiting errors (HTTP 429)
- Check `RATE_LIMIT_ENABLED` setting
- Adjust endpoint-specific limits (`RATE_LIMIT_*_RPM` variables)
- Increase burst size with `RATE_LIMIT_BURST_SIZE`
- Wait for `Retry-After` period before retrying
- Monitor logs for rate limit violations

#### Security header issues
- Verify `SECURITY_HEADERS_ENABLED=true`
- Check CSP violations in browser console
- For HTTPS deployments, enable `ENABLE_HSTS=true`
- Some headers may be overridden by reverse proxies

#### Request ID missing in logs
- Ensure `ENABLE_REQUEST_ID=true`
- Request IDs appear in error responses and structured logs
- Use request IDs for correlating audit events

### Debug Mode

Enable verbose logging:
```bash
RUST_LOG=debug orkee dashboard
```

### Health Check Endpoints

Test service health:
```bash
# Basic health
curl http://localhost:4001/api/health

# Detailed status
curl http://localhost:4001/api/status
```

### Log Analysis

Monitor audit logs for security events:
```bash
# Filter security violations
journalctl -u orkee | grep "access denied"

# Monitor directory access
journalctl -u orkee | grep "directory_access"

# Monitor rate limit violations
journalctl -u orkee | grep "Rate limit exceeded"

# Filter by request ID for specific incident
journalctl -u orkee | grep "request_id.*abc123"

# Monitor all audit events
journalctl -u orkee | grep "audit.*true"

# Check middleware startup messages
journalctl -u orkee | grep "middleware configuration"
```

### Security Monitoring

Key audit events to monitor:
- **Rate Limiting**: `Rate limit exceeded` with IP and endpoint
- **Path Security**: `Path access denied`, `Path traversal attempt`
- **Request Tracking**: All errors include `request_id` for correlation
- **Server Panics**: `Server panic occurred` with sanitized details
- **Middleware Status**: Startup logs show enabled features

Example structured log entry:
```json
{
  "timestamp": "2025-01-01T12:00:00Z",
  "request_id": "abc123-def456",
  "audit": true,
  "action": "rate_limit_exceeded",
  "ip": "127.0.0.1",
  "path": "/api/projects",
  "retry_after": 60
}
```

---

For additional help, see:
- [README.md](./README.md) - Getting started guide
- [CLAUDE.md](./CLAUDE.md) - Development workflow
- [PRODUCTION_SECURITY_REVIEW.md](./PRODUCTION_SECURITY_REVIEW.md) - Security analysis