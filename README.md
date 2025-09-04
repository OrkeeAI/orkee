# Orkee

A CLI and dashboard for AI agent orchestration

## Features

- 🤖 **AI Agent Orchestration** - Deploy and manage AI agents across different environments
- 📊 **Real-time Dashboard** - Web-based interface for monitoring and management
- 🖥️ **Terminal Interface** - Rich TUI for interactive command-line workflows
- 🔧 **CLI Tools** - Command-line interface for configuration and control
- 🔗 **Workflow Coordination** - Orchestrate complex multi-agent workflows

## Project Structure

This is a Turborepo monorepo containing:

```
orkee/
├── packages/
│   ├── cli/          # Rust Axum HTTP server providing REST API endpoints  
│   ├── dashboard/    # React SPA with Vite, Shadcn/ui, and Tailwind CSS
│   ├── tui/          # Ratatui-based standalone terminal interface
│   └── projects/     # Shared Rust library for core functionality (used by CLI and TUI)
├── docs/             # Documentation site
└── README.md
```

## Architecture

Orkee provides multiple interfaces for AI agent orchestration:

- **CLI Server** - REST API backend running on port 4001
- **Dashboard** - React web interface on port 5173 (connects to CLI server)
- **TUI** - Standalone terminal interface with rich interactive features
- **Shared Libraries** - Common functionality across all interfaces

The **Dashboard** requires the CLI server to be running. The **TUI** works independently.

## Installation

```bash
# Install from npm (coming soon)
npm install -g orkee

# Or install from source
git clone https://github.com/yourusername/orkee.git
cd orkee
pnpm install
turbo build
```

## Quick Start

```bash
# Install dependencies
pnpm install

# Start both CLI server and dashboard in development
turbo dev

# Or start components individually:

# Launch the web dashboard (requires CLI server)
cargo run --bin orkee -- dashboard

# Launch the terminal interface (standalone)
cargo run --bin orkee -- tui

# Explore CLI capabilities
cargo run --bin orkee -- --help
```

## Documentation

- [Getting Started Guide](docs/getting-started.md)
- [CLI Reference](docs/cli-reference.md)
- [API Documentation](docs/api.md)
- [Examples](examples/)

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or later)
- [pnpm](https://pnpm.io/) (v8 or later)
- [Rust](https://rustup.rs/) (latest stable)

### Development Setup

```bash
git clone https://github.com/yourusername/orkee.git
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
cargo run --bin orkee -- dashboard           # Start API server (port 4001)
cargo run --bin orkee -- tui                 # Launch TUI interface
cargo run --bin orkee -- projects list       # Example command (see --help for all)
cargo run --bin orkee -- --help              # See all available commands
cargo test                                   # Run Rust tests

# Dashboard-specific commands (run from packages/dashboard/)
pnpm dev          # Start Vite dev server (port 5173)
pnpm build        # Production build
pnpm lint         # Run ESLint
```

### Project Commands

- **`turbo build`** - Build all applications and packages
- **`turbo dev`** - Start development servers for all apps
- **`turbo lint`** - Run linting across the monorepo
- **`turbo test`** - Execute tests for all packages

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

[MIT](LICENSE)

## Support

- 📖 [Documentation](https://orkee.dev/docs)
- 💬 [Discussions](https://github.com/yourusername/orkee/discussions)
- 🐛 [Issues](https://github.com/yourusername/orkee/issues)

---

Made with ❤️ by the Orkee team