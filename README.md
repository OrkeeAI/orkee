# Orkee

A CLI and dashboard for AI agent orchestration

## Features

- 🤖 **Agent Management** - Deploy and manage AI agents across different environments
- 📊 **Real-time Dashboard** - Monitor agent performance and activity
- 🔧 **CLI Interface** - Command-line tools for agent configuration and control
- 🔗 **Orchestration** - Coordinate multiple agents for complex workflows

## Project Structure

This is a Turborepo monorepo containing:

```
orkee/
├── packages/
│   ├── dashboard/    # React/Vite dashboard application
│   └── cli/          # Rust CLI application
├── docs/             # Documentation site
└── README.md
```

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
# Initialize a new agent project
orkee init my-agent

# Start the dashboard
orkee dashboard

# Deploy an agent
orkee deploy ./my-agent
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

# Work on specific applications
turbo dev --filter=@orkee/dashboard  # Dashboard only
turbo dev --filter=@orkee/cli        # CLI development only
turbo build --filter=@orkee/dashboard # Build dashboard only
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