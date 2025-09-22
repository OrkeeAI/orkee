# Orkee

A CLI, TUI and dashboard for AI agent orchestration.

## Installation

```bash
npm install -g orkee
```

## Usage

```bash
# Launch the web dashboard (default ports: API 4001, UI 5173)
orkee dashboard

# Launch with custom ports
orkee dashboard --api-port 8080 --ui-port 3000

# Launch the terminal interface
orkee tui

# Project management
orkee projects list
orkee projects add --name "My Project" --path "/path/to/project"
orkee projects show <id>
orkee projects delete <id>

# Get help
orkee --help
```

## Features

- 🤖 **AI Agent Orchestration** - Deploy and manage AI agents across different environments
- 📊 **Real-time Dashboard** - Web-based interface for monitoring and management
- 🖥️ **Terminal Interface** - Rich TUI for interactive command-line workflows
- 🔧 **CLI Tools** - Command-line interface for configuration and control
- 🔗 **Workflow Coordination** - Orchestrate complex multi-agent workflows
- ☁️ **Cloud Sync** - Optional backup and sync with Orkee Cloud
- 🔐 **Enterprise Security** - OAuth authentication, JWT validation, and Row Level Security
- 🔒 **HTTPS/TLS Support** - Secure connections with auto-generated or custom certificates

## Documentation

For full documentation, visit [https://github.com/OrkeeAI/orkee](https://github.com/OrkeeAI/orkee)

## License

MIT