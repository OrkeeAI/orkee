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

Orkee includes comprehensive OpenSpec support for spec-driven development with a 5-tab workflow:

```
📄 PRDs → 📝 Changes → ✅ Specs → 📦 Archive → 📊 Coverage
```

**Key Features:**
- 🔄 **Change Management** - Proposal-based workflow with approval (Draft → Review → Approved → Implementing → Completed → Archived)
- 📝 **Delta Operations** - Add, modify, remove, or rename capabilities with structured proposals
- ✅ **Task Integration** - Link tasks to requirements with WHEN/THEN scenario validation
- 🤖 **AI-Powered** - PRD analysis, task generation, spec suggestions, and validation
- 📊 **Cost Tracking** - Monitor AI usage with detailed analytics
- 💾 **SQLite-Based** - 9 tables storing PRDs, specs, requirements, scenarios, and change history

**Documentation:**
- **[DOCS.md - OpenSpec Integration](DOCS.md#openspec-integration)** - Complete API reference with mermaid diagrams
- **[docs/docs/openspec/](docs/docs/openspec/)** - Detailed guides for workflows, changes, PRDs, specs, and tasks
- **[SPEC_TASK.md](SPEC_TASK.md)** - Technical specifications

## Documentation

- [Configuration & Architecture](CLAUDE.md) - Complete development guide and architecture details
- [Environment Variables & Configuration](DOCS.md) - Environment variables, security, and operational configuration
- [Production Deployment](DEPLOYMENT.md) - Docker, Nginx, TLS/SSL, and security setup
- [Security Guidelines](SECURITY.md) - Security policies and vulnerability reporting

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