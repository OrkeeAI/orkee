# Orkee

A CLI, TUI, dashboard, and native desktop app for AI agent orchestration

## Features

- 🤖 **AI Agent Orchestration** - Deploy and manage AI agents across different environments
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

## OpenSpec Integration

Orkee includes a comprehensive OpenSpec implementation for spec-driven development, providing end-to-end workflows from Product Requirements Documents (PRDs) to validated task execution.

### Current Status

✅ **Core implementation complete** - All major features implemented and ready for production use:
- Database schema with 9 tables for PRDs, specs, requirements, scenarios, and changes
- 46 Rust unit tests passing across parser, validator, and sync modules
- 28 REST API endpoints for full CRUD operations
- 11 frontend components for spec management
- Complete AI integration with cost tracking

### Architecture Overview

```
PRD (Product Requirements Document)
    ↓ AI Analysis
Spec Capabilities (functional areas)
    ↓ Break down
Requirements & Scenarios (WHEN/THEN)
    ↓ Generate
Tasks (implementation items)
    ↓ Manual additions
Orphan Tasks → Suggest Specs
    ↓ Sync back
Updated PRD (regenerated)
```

### Database Schema

**9 Tables** storing all spec-related data in SQLite:

1. **prds** - Product Requirements Documents with versioning
2. **spec_capabilities** - High-level functional capabilities (equivalent to spec folders)
3. **spec_requirements** - Individual requirements within capabilities
4. **spec_scenarios** - WHEN/THEN/AND test scenarios for requirements
5. **spec_changes** - Change proposals with approval workflow
6. **spec_deltas** - Capability changes (added/modified/removed)
7. **task_spec_links** - Links between tasks and spec requirements
8. **prd_spec_sync_history** - Audit trail for all sync operations
9. **ai_usage_logs** - AI cost tracking and usage monitoring

### API Endpoints

**28 REST endpoints** across 5 categories:

- **PRD Management** (6 endpoints) - Upload, list, update, delete, analyze, sync
- **Spec/Capability Management** (7 endpoints) - CRUD operations, validation, requirements
- **Change Management** (6 endpoints) - Change proposals, deltas, status updates
- **Task-Spec Integration** (6 endpoints) - Link tasks, validate, generate, find orphans
- **AI Proxy** (5 endpoints) - AI-powered analysis, generation, and validation

### Frontend Components

**11 Components** for complete spec management UI:

1. **PRDUploadDialog** - 3-tab interface for upload, preview, and AI analysis
2. **SpecBuilderWizard** - 4-step wizard for creating specs (Mode → Capability → Requirements → Validation)
3. **TaskSpecLinker** - Search and link tasks to spec requirements
4. **SyncDashboard** - 3 tabs for orphan tasks, PRD sync status, and spec overview
5. **TaskSpecIndicator** - Reusable badge showing spec link status on tasks
6. **SpecDetailsView** - Comprehensive spec viewer with requirements and scenarios
7. **ChangeProposalForm** - 3-tab markdown editor for proposals, tasks, and design
8. **ValidationResultsPanel** - Display task validation results against scenarios
9. **SpecDiffViewer** - Side-by-side version comparison
10. **ScenarioTestRunner** - Test task implementations against WHEN/THEN/AND scenarios
11. **CostDashboard** - AI usage tracking with cost monitoring

### Key Workflows

#### PRD → Spec → Task Flow
1. Upload PRD document (markdown format)
2. AI analyzes and extracts capabilities
3. Breaks down into requirements with WHEN/THEN scenarios
4. Generates linked tasks from requirements
5. Tasks validated against scenarios
6. Requirements marked complete when tasks finish

#### Task → Spec → PRD Flow
1. Developer manually creates task
2. System detects orphan task (no spec link)
3. AI suggests spec requirement or new capability
4. Creates change proposal for review
5. Approved changes update spec and regenerate PRD
6. Task now linked to requirement

### AI Integration

**Vercel AI SDK Integration** with production-ready features:

- **PRD Analysis** - Extract capabilities and requirements from documents
- **Task Generation** - Generate implementation tasks from specs
- **Spec Suggestions** - AI-powered recommendations for orphan tasks
- **Validation** - Verify task completion against WHEN/THEN scenarios
- **Cost Tracking** - Monitor AI usage with detailed analytics
- **Rate Limiting** - Protect against runaway costs
- **Caching** - Reduce redundant AI calls

### AI Usage Dashboard

Track and monitor AI costs with comprehensive analytics:

- **Summary Cards** - Total cost, tokens, requests, and average duration
- **By Operation** - Breakdown by analysis type (PRD, spec, validation)
- **By Model** - Compare costs across GPT-4, Claude, and other models
- **By Provider** - OpenAI vs Anthropic usage and costs
- **Recent Logs** - Detailed log viewer with error tracking

All AI usage data stored locally in SQLite with retention policies and export capabilities.

### Implementation Details

For complete implementation details, technical specifications, and development timeline, see [SPEC_TASK.md](SPEC_TASK.md).

## Documentation

- [Configuration & Architecture](CLAUDE.md) - Complete development guide and architecture details
- [Environment Variables & Configuration](DOCS.md) - Environment variables, security, and operational configuration
- [Production Deployment](DEPLOYMENT.md) - Docker, Nginx, TLS/SSL, and security setup
- [Security Guidelines](SECURITY.md) - Security policies and vulnerability reporting

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or later)
- [Bun](https://bun.sh/) (v1.0 or later)
- [Rust](https://rustup.rs/) (latest stable)

### Development Setup

```bash
git clone https://github.com/OrkeeAI/orkee.git
cd orkee
bun install
```

### Available Commands

```bash
# Build all apps and packages
turbo build

# Start all development servers
turbo dev

# Run tests across all packages
turbo test

# Lint all packages
turbo lint

# Work on specific packages
turbo dev --filter=@orkee/dashboard    # Dashboard only
turbo dev --filter=@orkee/cli          # CLI only
turbo build --filter=@orkee/dashboard  # Build dashboard only

# CLI-specific commands (run from packages/cli/)
cargo run --bin orkee -- dashboard           # Start API (4001) + UI (5173)
cargo run --bin orkee -- dashboard --dev     # Use local dashboard from packages/dashboard/
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000  # Custom ports
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 cargo run --bin orkee -- dashboard  # Via env
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard    # Use local dashboard via env
cargo run --bin orkee -- tui                 # Launch TUI interface
cargo run --bin orkee -- projects list       # List all projects
cargo run --bin orkee -- --help              # See all available commands
cargo test                                   # Run Rust tests

# Dashboard-specific commands (run from packages/dashboard/)
bun run dev                   # Start Vite dev server (uses ORKEE_UI_PORT or 5173)
ORKEE_UI_PORT=3000 bun run dev  # Start on custom port
bun run build                 # Production build
bun run lint                  # Run ESLint

# Tauri Desktop App commands (run from repository root or packages/dashboard/)
turbo dev:tauri              # Start Tauri dev app (from root)
bun tauri dev                # Start Tauri dev app (from packages/dashboard/)
bun tauri build              # Build production desktop app
bun tauri build --debug      # Build with debug symbols
bun tauri icon               # Generate app icons from source image
```

### Dashboard Development Mode

For dashboard development, you can use the local copy instead of the downloaded version:

```bash
# Method 1: Use --dev flag (recommended)
cargo run --bin orkee -- dashboard --dev

# Method 2: Use environment variable
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard

# Method 3: With custom ports in dev mode
cargo run --bin orkee -- dashboard --dev --api-port 8080 --ui-port 3000
```

**Benefits:**
- 🚀 **No file copying** - Uses `packages/dashboard/` directly
- 🔄 **Live reloading** - Vite HMR works with your source files
- ⚡ **Faster iteration** - Immediate feedback on changes

**How it works:**
- `--dev` or `ORKEE_DEV_MODE=true` enables development mode
- Uses local dashboard from `packages/dashboard/` instead of `~/.orkee/dashboard/`
- Falls back to downloaded version if local dashboard isn't found
- **Bypasses API authentication** for easier web dashboard development (localhost only)

See [DEV_MODE_README.md](DEV_MODE_README.md) for detailed usage instructions and [API_SECURITY.md](API_SECURITY.md) for authentication details.

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