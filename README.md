# Orkee

A CLI, TUI and dashboard for AI agent orchestration

## Features

- 🤖 **AI Agent Orchestration** - Deploy and manage AI agents across different environments
- 📊 **Real-time Dashboard** - Web-based interface for monitoring and management
- 🖥️ **Terminal Interface** - Rich TUI for interactive command-line workflows
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
│   ├── tui/          # Ratatui-based standalone terminal interface
│   ├── projects/     # Shared Rust library for core functionality (used by CLI and TUI)
│   ├── cloud/        # Cloud sync functionality (Orkee Cloud integration) - optional dependency
│   └── mcp-server/   # MCP (Model Context Protocol) server for Claude integration
├── deployment/       # Production deployment configurations
└── scripts/          # Build and release automation scripts
```

## Architecture

Orkee provides multiple interfaces for AI agent orchestration:

- **CLI Server** - REST API backend (default port 4001, configurable)
- **Dashboard** - React web interface (default port 5173, configurable)
- **TUI** - Standalone terminal interface with rich interactive features
- **Projects Library** - Core SQLite-based project management (used by CLI and TUI)
- **Cloud Library** - Optional cloud sync functionality with Orkee Cloud backend

The **Dashboard** requires the CLI server to be running. The **TUI** works independently. **Cloud features** are optional and can be enabled with the `--features cloud` flag during compilation.

## Installation

```bash
# Install from npm (coming soon)
npm install -g orkee

# Or install from source
git clone https://github.com/OrkeeAI/orkee.git
cd orkee
pnpm install
turbo build

# Build with cloud sync features (optional)
cargo build --features cloud
```

## Quick Start

```bash
# Install dependencies
pnpm install

# Start both CLI server and dashboard in development
turbo dev

# Or start components individually:

# Launch the web dashboard (requires CLI server)
cargo run --bin orkee -- dashboard                      # Default ports: API 4001, UI 5173
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000  # Custom ports
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 cargo run --bin orkee -- dashboard  # Via env vars

# Launch the terminal interface (standalone)
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

## Documentation

- [Configuration & Architecture](CLAUDE.md) - Complete development guide and architecture details
- [Environment Variables & Configuration](DOCS.md) - Environment variables, security, and operational configuration
- [Production Deployment](DEPLOYMENT.md) - Docker, Nginx, TLS/SSL, and security setup
- [Security Guidelines](SECURITY.md) - Security policies and vulnerability reporting

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or later)
- [pnpm](https://pnpm.io/) (v8 or later)
- [Rust](https://rustup.rs/) (latest stable)

### Development Setup

```bash
git clone https://github.com/OrkeeAI/orkee.git
cd orkee
pnpm install
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
pnpm dev                      # Start Vite dev server (uses ORKEE_UI_PORT or 5173)
ORKEE_UI_PORT=3000 pnpm dev  # Start on custom port
pnpm build                    # Production build
pnpm lint                     # Run ESLint
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