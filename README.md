# Orkee

A CLI, TUI, dashboard, and native desktop app for AI agent orchestration

## Features

- 🤖 **AI Agent Orchestration** - Deploy and manage AI agents across different environments
- 🎯 **Customizable AI Models** - Configure different AI providers and models for each task type (chat, PRD generation, insight extraction, etc.)
- 📊 **Real-time Dashboard** - Web-based interface for monitoring and management
- 🖥️ **Terminal Interface** - Rich TUI for interactive command-line workflows
- 🖼️ **Native Desktop App** - Tauri-based desktop application with system tray integration
- 🔧 **CLI Tools** - Command-line interface for configuration and control
- 🔗 **Workflow Coordination** - Orchestrate complex multi-agent workflows
- 🔐 **Enterprise Security** - OAuth authentication, JWT validation, and Row Level Security
- 🔒 **HTTPS/TLS Support** - Secure connections with auto-generated or custom certificates
- 💾 **Local-First Architecture** - SQLite-based storage for fast, reliable data management

## Project Structure

This is a Turborepo monorepo containing:

```
orkee/
├── packages/
│   ├── cli/          # Rust Axum HTTP server providing REST API endpoints
│   ├── dashboard/    # React SPA with Vite, Shadcn/ui, and Tailwind CSS
│   │   └── src-tauri/    # Tauri desktop app wrapper with system tray
│   ├── tui/          # Ratatui-based standalone terminal interface
│   ├── projects/     # Shared Rust library for core functionality (used by CLI and TUI)
│   ├── preview/      # Development server management with registry
│   └── mcp-server/   # MCP (Model Context Protocol) server for Claude integration
├── deployment/       # Production deployment configurations
└── scripts/          # Build and release automation scripts
```

## Architecture

Orkee provides multiple interfaces for AI agent orchestration:

- **CLI Server** - REST API backend (default port 4001, configurable)
- **Dashboard** - React web interface (default port 5173, configurable)
- **Desktop App** - Native Tauri application with system tray (bundles CLI server as sidecar)
- **TUI** - Standalone terminal interface with rich interactive features
- **Projects Library** - Core SQLite-based project management (used by CLI and TUI)
- **Preview Library** - Development server management with central registry

The **Dashboard** and **Desktop App** require the CLI server to be running. The **TUI** works independently.

## Installation

### Option 1: Desktop App (Native GUI + CLI + TUI) - v0.0.9 (Recommended)

Download the native desktop application for your platform:

#### macOS
- **Apple Silicon**: [Orkee_0.0.9_aarch64.dmg](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/Orkee_0.0.9_aarch64.dmg) (12 MB)
- **Intel**: [Orkee_0.0.9_x64.dmg](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/Orkee_0.0.9_x64.dmg) (12 MB)

**Installation (IMPORTANT):**
1. Double-click the .dmg file and drag Orkee to your Applications folder
2. **Remove quarantine attributes (REQUIRED):**
   ```bash
   sudo xattr -cr /Applications/Orkee.app
   ```
   This command is necessary because the app is unsigned. macOS Gatekeeper will block the app without this step.
3. Launch Orkee from Applications folder or Spotlight

#### Windows
- **Installer (recommended)**: [Orkee_0.0.9_x64_en-US.msi](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/Orkee_0.0.9_x64_en-US.msi) (10 MB)
- **Setup EXE**: [Orkee_0.0.9_x64-setup.exe](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/Orkee_0.0.9_x64-setup.exe) (7 MB)

**Installation**: Download and run the installer. You may see a Windows SmartScreen warning - click "More info" and then "Run anyway" (app is unsigned).

#### Linux
- **Debian/Ubuntu**: [Orkee_0.0.9_amd64.deb](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/Orkee_0.0.9_amd64.deb) (12 MB)
  ```bash
  sudo dpkg -i Orkee_0.0.9_amd64.deb
  ```
- **Fedora/RHEL**: [Orkee-0.0.9-1.x86_64.rpm](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/Orkee-0.0.9-1.x86_64.rpm) (12 MB)
  ```bash
  sudo rpm -i Orkee-0.0.9-1.x86_64.rpm
  ```
- **Universal (AppImage)**: [Orkee_0.0.9_amd64.AppImage](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/Orkee_0.0.9_amd64.AppImage) (86 MB)
  ```bash
  chmod +x Orkee_0.0.9_amd64.AppImage
  ./Orkee_0.0.9_amd64.AppImage
  ```

The desktop app includes:
- 🖥️ Native desktop application with system tray
- 💻 Full CLI access (`orkee` command)
- 🎨 Terminal UI (`orkee tui`)
- 🌐 Web dashboard in native window

[View all releases](https://github.com/OrkeeAI/orkee/releases) | [Checksums](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.9/checksums.txt)

### Option 2: npm (CLI + TUI + Web Dashboard)

```bash
# Install globally via npm
npm install -g orkee

# Verify installation
orkee --version

# Start the dashboard
orkee dashboard

# Or use the terminal interface
orkee tui
```

The npm package automatically downloads the appropriate binary for your platform (macOS, Linux, Windows).

### Option 3: Build from Source

```bash
git clone https://github.com/OrkeeAI/orkee.git
cd orkee
bun install
turbo build
```

## Quick Start

```bash
# Install dependencies
bun install

# Choose your interface:

# 1. Native Desktop App with system tray (recommended)
turbo dev:tauri

# 2. Web-based dashboard
turbo dev                    # Start both CLI server and dashboard
turbo dev:web               # Alternative: web-only development

# 3. CLI + Dashboard (manual)
cargo run --bin orkee -- dashboard                      # Default ports: API 4001, UI 5173
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000  # Custom ports
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 cargo run --bin orkee -- dashboard  # Via env vars

# 4. Terminal interface (standalone, no server required)
cargo run --bin orkee -- tui

# Explore CLI capabilities
cargo run --bin orkee -- --help
```

### Enable HTTPS (Optional)

```bash
# Create .env file and enable TLS
echo "TLS_ENABLED=true" > .env

# Start with HTTPS (auto-generates development certificates)
cargo run --bin orkee -- dashboard

# Dashboard will be available at https://localhost:4001
# HTTP requests to port 4000 automatically redirect to HTTPS
```

## OAuth Authentication

Orkee supports OAuth authentication for AI providers (Claude, OpenAI, Google, xAI), enabling you to use your subscription accounts instead of API keys.

### Benefits

- ✅ **Use Claude Pro/Max subscriptions** - No API costs, leverage your existing subscription
- ✅ **Unified authentication** - One login for all services
- ✅ **Secure token management** - Encrypted storage with automatic refresh
- ✅ **Backward compatible** - Works alongside existing API key authentication

### Quick Setup

```bash
# Authenticate with a provider (opens browser for OAuth flow)
orkee login claude

# Check authentication status
orkee auth status

# Logout from a provider
orkee logout claude

# Logout from all providers
orkee logout all
```

### Supported Providers

| Provider | OAuth Support | Subscription Types |
|----------|--------------|-------------------|
| Claude (Anthropic) | ✅ Yes | Pro, Max |
| OpenAI | ✅ Yes | Plus, Team, Enterprise |
| Google (Vertex AI) | ✅ Yes | Cloud accounts |
| xAI (Grok) | ✅ Yes | Premium |

### How OAuth Works

1. **Login**: Run `orkee login <provider>` to start the OAuth flow
2. **Browser opens**: Authenticate with your provider account
3. **Token storage**: OAuth tokens are encrypted and stored in `~/.orkee/orkee.db`
4. **Automatic refresh**: Tokens are refreshed automatically before expiry (5-minute buffer)
5. **Use in apps**: Dashboard and CLI use OAuth tokens transparently

### Security Considerations

> **⚠️ IMPORTANT**: By default, Orkee uses **machine-based encryption** which provides transport encryption only.
>
> With machine-based encryption:
> - ✅ Tokens are protected during backup/sync operations
> - ❌ Anyone with access to `~/.orkee/orkee.db` on your machine can decrypt the tokens
> - ❌ This is **NOT** true at-rest encryption
>
> **For production use or shared machines**, upgrade to password-based encryption:
>
> ```bash
> orkee security set-password    # Enable password-based encryption
> orkee security status          # Check current encryption mode
> ```
>
> Password-based encryption provides:
> - ✅ True at-rest encryption - tokens cannot be decrypted without your password
> - ✅ Suitable for shared machines and production environments
> - ⚠️ Password cannot be recovered if lost

### Configuration

OAuth behavior can be configured via environment variables:

```bash
# OAuth client configuration (optional - defaults provided)
OAUTH_CLAUDE_CLIENT_ID=orkee-cli-oauth-client
OAUTH_CLAUDE_REDIRECT_URI=http://localhost:3737/callback
OAUTH_CLAUDE_SCOPES="model:claude account:read"

# OAuth server settings
OAUTH_CALLBACK_PORT=3737                  # Callback server port
OAUTH_STATE_TIMEOUT_SECS=600              # 10 minutes
OAUTH_TOKEN_REFRESH_BUFFER_SECS=300       # 5 minutes
```

### Authentication Preference

Set your preferred authentication method:

```bash
# Use OAuth only
orkee config set auth_preference oauth

# Use API keys only
orkee config set auth_preference api_key

# Try OAuth first, fall back to API keys (default)
orkee config set auth_preference hybrid
```

### Migration from API Keys

Orkee maintains full backward compatibility with API keys:

```bash
# Check current authentication
orkee auth status

# Add OAuth alongside existing API keys
orkee login claude

# Your API key continues to work as fallback
# Remove API key after successful OAuth (optional)
orkee config delete ANTHROPIC_API_KEY
```

### Dashboard Integration

The OAuth settings are accessible via the dashboard:

1. Navigate to **Settings** → **OAuth Authentication**
2. View authentication status for all providers
3. See token expiry times and subscription types
4. Logout from providers via UI buttons

For detailed OAuth setup and troubleshooting, see [OAUTH_SETUP.md](./OAUTH_SETUP.md).

## Desktop App (Tauri)

The Orkee Desktop App is a native application built with Tauri that provides:

### Features

- 🎯 **System Tray Integration** - Native menu bar icon with live server monitoring
- 🔄 **Automatic Server Management** - Launches and manages the CLI server automatically
- 🌐 **Quick Access** - Open servers in browser directly from tray menu
- 📋 **URL Copying** - Copy server URLs to clipboard with one click
- ⚡ **Server Controls** - Start, stop, and restart development servers from the tray
- 🎨 **Theme Adaptation** - macOS template icons automatically adapt to light/dark mode
- 💻 **Cross-Platform** - Supports macOS, Windows, and Linux

### System Tray Menu

The tray provides:
- **Show Orkee Dashboard** - Opens the main dashboard window
- **Dev Servers** - Lists all running development servers with:
  - Open in Browser
  - Copy URL
  - Restart Server
  - Stop Server
- **Refresh** - Manually refresh server list (also polls automatically every 5 seconds)
- **Quit Orkee** - Gracefully stops all servers and exits

### Running the Desktop App

#### Development Mode

```bash
# Start the Tauri dev app (from repository root)
turbo dev:tauri

# Or from the dashboard directory
cd packages/dashboard
pnpm tauri dev
```

#### Production Build

```bash
# Build the desktop app for your platform
cd packages/dashboard
pnpm tauri build

# The built app will be in:
# - macOS: src-tauri/target/release/bundle/macos/
# - Windows: src-tauri/target/release/bundle/msi/
# - Linux: src-tauri/target/release/bundle/appimage/
```

### Configuration

The desktop app supports the following environment variables:

```bash
# Customize tray polling interval (default: 5 seconds, min: 1, max: 60)
ORKEE_TRAY_POLL_INTERVAL_SECS=10

# UI port for the dashboard (default: 5173)
ORKEE_UI_PORT=3000
```

### Background Operation

The desktop app is designed to run in the background:
- Closing the window **hides** the app to the system tray (it doesn't quit)
- Access the app via the menu bar/system tray icon
- Quit from the tray menu to fully exit and stop all servers
- macOS: Runs as an Accessory app (menu bar only, no Dock icon by default)

**Note**: The Tauri app bundles the Orkee CLI binary as a sidecar process. It will automatically start the API server on an available port when launched.

## PRD Ideation & CCPM Workflow

Orkee provides AI-powered PRD (Product Requirements Document) ideation and Chat-Based Collaborative Project Management:

```
💡 Ideate → 📄 PRD → 📋 Epic → ✅ Tasks
```

**Ideation Modes:**
- 🚀 **Quick Mode** - Generate complete PRDs instantly from a description
- 🎯 **Guided Mode** - Step-by-step section building with AI assistance
- 📝 **Template-Based** - Use customizable templates for different project types

**PRD Features:**
- 📄 **Structured Sections** - Overview, UX, Technical, Roadmap, Dependencies, Risks, Research
- 🤖 **AI-Powered Generation** - Complete PRD generation or section-by-section expansion
- 💾 **Version Tracking** - Full PRD history with soft delete support
- 🎨 **Custom Templates** - Create and manage reusable PRD templates

**CCPM (Chat-Based Collaborative Project Management):**
- 💬 **Epic Generation** - Convert PRDs into actionable epics with AI
- 🔄 **Iterative Refinement** - Chat-based workflow for epic improvement
- 🎯 **Task Decomposition** - Break down epics into executable tasks
- 🔍 **Research Tools** - Competitor analysis, similar projects, and technical specs

## AI Usage Tracking & Telemetry

Orkee provides comprehensive tracking of all AI operations, including token usage, costs, and tool invocations:

### Features

- 📊 **Automatic Tracking** - All AI SDK calls are tracked automatically (zero manual logging required)
- 💰 **Cost Monitoring** - Real-time cost tracking across different AI providers and models
- 🔧 **Tool Call Analytics** - Track which tools are invoked, success rates, and performance
- 📈 **Usage Dashboard** - Visual analytics with charts for tokens, costs, and tool usage over time
- 🎯 **Per-Operation Metrics** - Track specific operations like PRD generation, chat responses, etc.

### Tracked Metrics

For every AI operation, Orkee tracks:
- **Tokens**: Input, output, and total token counts
- **Cost**: Estimated cost based on provider pricing
- **Duration**: Actual request duration (not 0ms!)
- **Tool Calls**: Which tools were invoked, arguments, results, and success/failure
- **Model/Provider**: Which AI model and provider was used
- **Metadata**: Finish reason, response ID, and provider-specific metadata

### Usage Dashboard

Access the Usage tab in the Orkee dashboard to view:
- **Overview** - Key metrics cards with totals for requests, tokens, costs, and tool calls
- **Model Breakdown** - Token usage distribution across different AI models
- **Provider Breakdown** - Cost distribution across providers (Anthropic, OpenAI, etc.)
- **Tool Analytics** - Most used tools with success rates and performance metrics
- **Charts & Analytics** - Time-series visualizations for requests, tokens, and costs

### For Developers

All AI operations are automatically wrapped with telemetry tracking. When adding new AI functionality:

**Using the telemetry wrapper:**
```typescript
import { trackAIOperation } from '@/lib/ai/telemetry';

// Wrap your AI SDK call
const result = await trackAIOperation(
  'operation_name',        // e.g., 'generate_prd', 'chat_response'
  projectId,               // Project ID or null for global operations
  () => generateText({     // Your AI SDK call
    model: anthropic('claude-3-opus'),
    prompt: 'Your prompt here',
    tools: { search, calculate }
  })
);
```

**The wrapper automatically:**
- Tracks token usage and costs
- Measures actual duration with high precision
- Extracts tool calls from responses
- Handles streaming responses via `onFinish` callback
- Sends telemetry to backend without blocking the operation
- Logs errors without breaking the AI operation

**Tool call data structure:**
```typescript
interface ToolCall {
  name: string;                    // Tool name (e.g., 'search', 'calculate')
  arguments: Record<string, any>;  // Tool arguments
  result?: any;                    // Tool result (if available)
  durationMs?: number;             // Tool execution time
  error?: string;                  // Error message if tool failed
}
```

### Architecture

- **Frontend Telemetry** - `packages/dashboard/src/lib/ai/telemetry.ts` wraps AI SDK calls
- **Backend Endpoint** - `POST /api/ai-usage` accepts telemetry data with validation
- **Database Storage** - SQLite table `ai_usage_logs` with full-text search support
- **Analytics Endpoints** - Stats, tool usage, and time-series data for charts

All telemetry is asynchronous to ensure zero performance impact on AI operations.

## Documentation

- [Configuration & Architecture](CLAUDE.md) - Complete development guide and architecture details
- [Environment Variables & Configuration](DOCS.md) - Environment variables, security, and operational configuration
- [Production Deployment](DEPLOYMENT.md) - Docker, Nginx, TLS/SSL, and security setup
- [Security Guidelines](SECURITY.md) - Security policies and vulnerability reporting
- [AI Usage Implementation Plan](ai-usage.md) - Detailed implementation plan for AI tracking features

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+) | [Bun](https://bun.sh/) (v1.0+) | [Rust](https://rustup.rs/) (latest stable)

### Quick Start

```bash
# Clone and install
git clone https://github.com/OrkeeAI/orkee.git
cd orkee && bun install

# Start development (all interfaces)
turbo dev                    # Web dashboard + CLI server
turbo dev:tauri              # Native desktop app

# Or start specific interfaces
cargo run --bin orkee -- dashboard --dev  # Web dashboard with hot reload
cargo run --bin orkee -- tui              # Terminal interface
```

### Common Commands

```bash
turbo build                  # Build all packages
turbo test                   # Run all tests
turbo lint                   # Lint all packages
cargo test                   # Run Rust tests
```

**For detailed development instructions, see [CLAUDE.md](CLAUDE.md)**

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

[MIT](LICENSE)

## Support

- 📖 [Documentation](https://orkee.ai/docs)
- 💬 [Discussions](https://github.com/OrkeeAI/orkee/discussions)
- 🐛 [Issues](https://github.com/OrkeeAI/orkee/issues)

---

Made with ❤️ by the Orkee team