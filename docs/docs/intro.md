---
sidebar_position: 1
---

# Welcome to Orkee

A CLI, TUI and dashboard for AI agent orchestration

## What is Orkee?

Orkee is a comprehensive platform for orchestrating AI agents, providing multiple interfaces to manage complex multi-agent workflows. Whether you prefer command-line tools, terminal interfaces, or web dashboards, Orkee has you covered.

## Key Features

- 🤖 **AI Agent Orchestration** - Deploy and manage AI agents across different environments
- 📊 **Real-time Dashboard** - Web-based interface for monitoring and management
- 🖥️ **Terminal Interface** - Rich TUI for interactive command-line workflows
- 🔧 **CLI Tools** - Command-line interface for configuration and control
- 🔗 **Workflow Coordination** - Orchestrate complex multi-agent workflows
- ☁️ **Cloud Sync** - Optional backup and sync with Orkee Cloud (fully implemented)
- 🔐 **Enterprise Security** - OAuth authentication, JWT validation, and Row Level Security
- 🔒 **HTTPS/TLS Support** - Secure connections with auto-generated or custom certificates
- 💾 **Local-First Architecture** - SQLite-based storage with optional cloud enhancement

## Architecture Overview

Orkee provides multiple interfaces for AI agent orchestration:

- **CLI Server** - REST API backend (default port 4001, configurable)
- **Dashboard** - React web interface (default port 5173, configurable)
- **TUI** - Standalone terminal interface with rich interactive features
- **Projects Library** - Core SQLite-based project management (used by CLI and TUI)
- **Cloud Library** - Optional cloud sync functionality with Orkee Cloud backend

The **Dashboard** requires the CLI server to be running. The **TUI** works independently. **Cloud features** are optional and can be enabled with the `--features cloud` flag during compilation.

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

## Getting Started

Get started with Orkee by following our installation guide, then explore the different interfaces available to you.

### Quick Installation

```bash
# Install from npm (recommended)
npm install -g orkee

# Verify installation
orkee --help
```

### What's Next?

- 📖 **[Installation Guide](./installation.md)** - Detailed installation instructions
- ⚙️ **[Configuration](./configuration.md)** - Environment variables and settings
- 🚀 **[Deployment](./deployment.md)** - Production deployment guide
- 🔐 **[Security](./security.md)** - Security guidelines and best practices
- 🏗️ **[Architecture](./architecture.md)** - Technical architecture details

---

Made with ❤️ by the Orkee team