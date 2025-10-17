# Orkee

A CLI, TUI, dashboard, and native desktop app for AI agent orchestration

## Features

- 🤖 **AI Agent Orchestration** - Deploy and manage AI agents across different environments
- 📊 **Real-time Dashboard** - Web-based interface for monitoring and management
- 🖥️ **Terminal Interface** - Rich TUI for interactive command-line workflows
- 🖼️ **Native Desktop App** - Tauri-based desktop application with system tray integration
- 🔧 **CLI Tools** - Command-line interface for configuration and control
- 🔗 **Workflow Coordination** - Orchestrate complex multi-agent workflows
- ☁️ **Cloud Sync** - Optional backup and sync with Orkee Cloud (fully implemented)
- 🔐 **Enterprise Security** - OAuth authentication, JWT validation, and Row Level Security
- 🔒 **HTTPS/TLS Support** - Secure connections with auto-generated or custom certificates
- 💾 **Local-First Architecture** - SQLite-based storage with optional cloud enhancement

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
│   ├── cloud/        # Cloud sync functionality (Orkee Cloud integration) - optional dependency
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
- **Cloud Library** - Optional cloud sync functionality with Orkee Cloud backend

The **Dashboard** and **Desktop App** require the CLI server to be running. The **TUI** works independently. **Cloud features** are optional and can be enabled with the `--features cloud` flag during compilation.

## Installation

### Option 1: npm (CLI + TUI + Web Dashboard)

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

### Option 2: Desktop App (Native GUI + CLI + TUI) - v0.0.5

Download the native desktop application for your platform:

#### macOS
- **Apple Silicon**: [Orkee_0.0.5_aarch64.dmg](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/Orkee_0.0.5_aarch64.dmg) (12 MB)
- **Intel**: [Orkee_0.0.5_x64.dmg](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/Orkee_0.0.5_x64.dmg) (12 MB)

**Installation**: Double-click the .dmg file and drag Orkee to your Applications folder. On first launch, you may need to right-click and select "Open" to bypass Gatekeeper (app is unsigned).

#### Windows
- **Installer (recommended)**: [Orkee_0.0.5_x64_en-US.msi](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/Orkee_0.0.5_x64_en-US.msi) (10 MB)
- **Setup EXE**: [Orkee_0.0.5_x64-setup.exe](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/Orkee_0.0.5_x64-setup.exe) (7 MB)

**Installation**: Download and run the installer. You may see a Windows SmartScreen warning - click "More info" and then "Run anyway" (app is unsigned).

#### Linux
- **Debian/Ubuntu**: [Orkee_0.0.5_amd64.deb](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/Orkee_0.0.5_amd64.deb) (12 MB)
  ```bash
  sudo dpkg -i Orkee_0.0.5_amd64.deb
  ```
- **Fedora/RHEL**: [Orkee-0.0.5-1.x86_64.rpm](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/Orkee-0.0.5-1.x86_64.rpm) (12 MB)
  ```bash
  sudo rpm -i Orkee-0.0.5-1.x86_64.rpm
  ```
- **Universal (AppImage)**: [Orkee_0.0.5_amd64.AppImage](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/Orkee_0.0.5_amd64.AppImage) (86 MB)
  ```bash
  chmod +x Orkee_0.0.5_amd64.AppImage
  ./Orkee_0.0.5_amd64.AppImage
  ```

The desktop app includes:
- 🖥️ Native desktop application with system tray
- 💻 Full CLI access (`orkee` command)
- 🎨 Terminal UI (`orkee tui`)
- 🌐 Web dashboard in native window

[View all releases](https://github.com/OrkeeAI/orkee/releases) | [Checksums](https://github.com/OrkeeAI/orkee/releases/download/desktop-v0.0.5/checksums.txt)

### Option 3: Build from Source

```bash
git clone https://github.com/OrkeeAI/orkee.git
cd orkee
bun install
turbo build

# Build with cloud sync features (optional)
cargo build --features cloud
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

### Cloud Sync Setup (Optional)

```bash
# Authenticate with Orkee Cloud (opens browser for OAuth authentication)
cargo run --features cloud --bin orkee -- cloud login

# Check authentication and sync status
cargo run --features cloud --bin orkee -- cloud status

# Manual sync to cloud
cargo run --features cloud --bin orkee -- cloud sync

# List cloud projects
cargo run --features cloud --bin orkee -- cloud list

# Restore project from cloud
cargo run --features cloud --bin orkee -- cloud restore --project <id>

# Enable/disable cloud features
cargo run --features cloud --bin orkee -- cloud enable
cargo run --features cloud --bin orkee -- cloud disable
```

**Note**: Cloud features require compilation with `--features cloud` and an Orkee Cloud account. The OSS client is fully implemented and ready - visit https://orkee.ai for API access.

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
cargo run --features cloud --bin orkee -- cloud login    # Authenticate with Orkee Cloud
cargo run --features cloud --bin orkee -- cloud sync     # Sync projects to cloud
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

See [DEV_MODE_README.md](DEV_MODE_README.md) for detailed usage instructions.

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