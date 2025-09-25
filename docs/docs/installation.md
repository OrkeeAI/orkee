# Installation

This guide walks you through installing and setting up Orkee on your system.

## Prerequisites

Before installing Orkee, ensure you have the following prerequisites:

- **Node.js** (v18 or later) - [Download here](https://nodejs.org/)
- **pnpm** (v8 or later) - [Installation guide](https://pnpm.io/installation)
- **Rust** (latest stable) - [Install via rustup](https://rustup.rs/)

## Installation Methods

### Method 1: npm Installation (Recommended)

The simplest way to install Orkee is via npm:

```bash
# Install globally
npm install -g orkee

# Verify installation
orkee --help
```

:::info
npm installation is coming soon. For now, please use the source installation method below.
:::

### Method 2: Install from Source

For development or to get the latest features:

```bash
# Clone the repository
git clone https://github.com/OrkeeAI/orkee.git
cd orkee

# Install dependencies
pnpm install

# Build all packages
turbo build

# Build with cloud sync features (optional)
cargo build --features cloud
```

## Quick Start

Once installed, you can start using Orkee immediately:

```bash
# Install dependencies (if building from source)
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

## Interface Options

Orkee provides three different interfaces:

### Web Dashboard
The dashboard provides a modern web interface for managing AI agents:

```bash
# Default ports: API 4001, UI 5173
cargo run --bin orkee -- dashboard

# Custom ports
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000

# Using environment variables
ORKEE_API_PORT=9000 ORKEE_UI_PORT=3333 cargo run --bin orkee -- dashboard
```

The dashboard will be available at `http://localhost:5173` (or your configured UI port).

### Terminal Interface (TUI)
The TUI provides a rich terminal interface that works independently:

```bash
# Launch the terminal interface
cargo run --bin orkee -- tui
```

### Command Line Interface (CLI)
Direct command-line access to all Orkee functionality:

```bash
# View available commands
cargo run --bin orkee -- --help

# Project management
cargo run --bin orkee -- projects list
cargo run --bin orkee -- projects add
```

## HTTPS Setup (Optional)

To enable HTTPS for secure connections:

```bash
# Create .env file and enable TLS
echo "TLS_ENABLED=true" > .env

# Start with HTTPS (auto-generates development certificates)
cargo run --bin orkee -- dashboard
```

- Dashboard will be available at `https://localhost:4001`
- HTTP requests to port 4000 automatically redirect to HTTPS
- Development certificates are auto-generated for convenience

## Cloud Sync Setup (Optional)

Orkee supports optional cloud sync for backup and collaboration:

```bash
# Authenticate with Orkee Cloud (opens browser for OAuth)
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

:::note
Cloud features require:
1. Compilation with `--features cloud`
2. An Orkee Cloud account (visit [orkee.ai](https://orkee.ai) for access)
3. OAuth authentication setup
:::

## Development Setup

For developers wanting to contribute or customize Orkee:

```bash
# Clone and setup development environment
git clone https://github.com/OrkeeAI/orkee.git
cd orkee
pnpm install

# Start development servers
turbo dev

# Run tests
turbo test

# Lint code
turbo lint
```

## Verification

To verify your installation is working correctly:

```bash
# Check Orkee version
cargo run --bin orkee -- --version

# Start the dashboard and verify web interface
cargo run --bin orkee -- dashboard

# In another terminal, test the CLI
cargo run --bin orkee -- projects list
```

## Troubleshooting

### Common Issues

**Port conflicts**: If default ports are already in use, specify custom ports:
```bash
cargo run --bin orkee -- dashboard --api-port 4002 --ui-port 5174
```

**Permission errors**: Ensure you have proper permissions for the installation directory.

**Build failures**: Make sure all prerequisites are installed and up to date.

### Getting Help

- **GitHub Issues**: [Report bugs or request features](https://github.com/OrkeeAI/orkee/issues)
- **GitHub Discussions**: [Ask questions and share ideas](https://github.com/OrkeeAI/orkee/discussions)
- **Documentation**: Check the [Configuration](./configuration) and [Architecture](./architecture) guides

## Next Steps

Once Orkee is installed:

1. Read the [Configuration Guide](./configuration) to customize your setup
2. Review the [Architecture Overview](./architecture) to understand how Orkee works
3. Check the [Deployment Guide](./deployment) for production setups
4. Explore the [Security Guidelines](./security) for security best practices