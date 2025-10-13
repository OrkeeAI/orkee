---
sidebar_position: 1
title: Development Setup
---

# Development Setup

Set up your local development environment for contributing to Orkee.

## Prerequisites

Before you begin, ensure you have the following installed:

### Required Software

| Tool | Minimum Version | Purpose |
|------|----------------|---------|
| **Node.js** | v18+ | Dashboard and build tools |
| **pnpm** | v8+ | Package management |
| **Rust** | Latest stable (1.70+) | CLI, TUI, and core libraries |
| **Git** | Any recent version | Version control |

### Installation Instructions

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="macos" label="macOS" default>

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Node.js
brew install node

# Install pnpm
npm install -g pnpm

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installations
node --version
pnpm --version
rustc --version
cargo --version
```

</TabItem>
<TabItem value="linux" label="Linux (Ubuntu/Debian)">

```bash
# Update package list
sudo apt update

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs

# Install pnpm
npm install -g pnpm

# Install build tools
sudo apt install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installations
node --version
pnpm --version
rustc --version
cargo --version
```

</TabItem>
<TabItem value="windows" label="Windows">

```powershell
# Install Node.js from https://nodejs.org/
# Or use Chocolatey
choco install nodejs

# Install pnpm
npm install -g pnpm

# Install Rust
# Download from https://rustup.rs/
# Or use Chocolatey
choco install rust

# Verify installations
node --version
pnpm --version
rustc --version
cargo --version
```

</TabItem>
</Tabs>

## Getting the Code

### 1. Fork the Repository

1. Go to https://github.com/OrkeeAI/orkee
2. Click the **Fork** button in the top-right
3. Select your GitHub account

### 2. Clone Your Fork

```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/orkee.git
cd orkee

# Add upstream remote
git remote add upstream https://github.com/OrkeeAI/orkee.git

# Verify remotes
git remote -v
# Should show:
# origin    https://github.com/YOUR-USERNAME/orkee.git (fetch)
# origin    https://github.com/YOUR-USERNAME/orkee.git (push)
# upstream  https://github.com/OrkeeAI/orkee.git (fetch)
# upstream  https://github.com/OrkeeAI/orkee.git (push)
```

### 3. Install Dependencies

```bash
# Install Node.js dependencies
pnpm install

# This installs dependencies for:
# - Dashboard (React app)
# - CLI tools
# - Build system (Turborepo)
```

## Project Structure

Understanding the monorepo structure:

```
orkee/
├── packages/
│   ├── cli/              # Rust CLI server (Axum HTTP API)
│   │   ├── src/
│   │   │   ├── api/      # API endpoints
│   │   │   ├── bin/      # Binary entry points
│   │   │   └── config/   # Configuration
│   │   └── Cargo.toml
│   │
│   ├── dashboard/        # React SPA (Vite + TypeScript)
│   │   ├── src/
│   │   │   ├── components/  # Reusable components
│   │   │   ├── contexts/    # React contexts
│   │   │   ├── pages/       # Page components
│   │   │   └── services/    # API client
│   │   └── package.json
│   │
│   ├── tui/              # Terminal UI (Ratatui)
│   │   ├── src/
│   │   │   ├── app.rs       # Main TUI logic
│   │   │   ├── events.rs    # Event handling
│   │   │   └── state.rs     # Application state
│   │   └── Cargo.toml
│   │
│   ├── projects/         # Shared Rust library
│   │   ├── src/
│   │   │   ├── manager.rs   # Project management
│   │   │   ├── storage/     # Database layer
│   │   │   └── api/         # API handlers
│   │   └── Cargo.toml
│   │
│   └── cloud/            # Cloud sync (optional)
│       ├── src/
│       │   ├── sync/        # Sync engine
│       │   ├── auth/        # Authentication
│       │   └── providers/   # Cloud providers
│       └── Cargo.toml
│
├── docs/                 # Docusaurus documentation site
├── deployment/           # Deployment configurations
├── turbo.json           # Turborepo configuration
├── package.json         # Root package.json
└── Cargo.toml           # Workspace Cargo.toml
```

## Development Workflow

### Starting the Development Server

```bash
# Start all development servers
turbo dev

# This starts:
# - CLI API server (port 4001)
# - Dashboard dev server (port 5173)
# - Hot reload for both
```

**What happens:**

1. **CLI Server**: Rust binary runs with hot reload (via cargo watch)
2. **Dashboard**: Vite dev server with HMR
3. **Automatic browser opening**: Dashboard opens at http://localhost:5173

### Working on Specific Packages

```bash
# Work on dashboard only
turbo dev --filter=@orkee/dashboard

# Work on CLI only
turbo dev --filter=@orkee/cli

# Build specific package
turbo build --filter=@orkee/dashboard

# Test specific package
turbo test --filter=@orkee/cli
```

### Running in Development Mode

For CLI development with local dashboard:

```bash
# Run CLI with --dev flag (uses local dashboard source)
cd packages/cli
cargo run --bin orkee -- dashboard --dev

# Or via environment variable
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard
```

### TUI Development

```bash
# Run TUI from source
cd packages/tui
cargo run

# With debug logging
RUST_LOG=debug cargo run

# Build and run optimized
cargo build --release
./target/release/orkee-tui
```

## Building

### Development Builds

```bash
# Build all packages (development mode)
turbo build

# Build specific package
turbo build --filter=@orkee/cli

# Rust development build (fast, unoptimized)
cd packages/cli
cargo build

# Binary location: target/debug/orkee
```

### Production Builds

```bash
# Build all packages (optimized)
turbo build

# Rust release build (slow, optimized)
cd packages/cli
cargo build --release

# With cloud features
cargo build --release --features cloud

# Binary location: target/release/orkee
```

### Build Times

| Build Type | Time | Size | Use Case |
|------------|------|------|----------|
| Dev (cargo build) | 2-5 min | ~100MB | Development, testing |
| Release (cargo build --release) | 5-15 min | ~15MB | Production, distribution |
| Release + cloud | 8-20 min | ~30MB | Production with cloud features |

## Testing

### Running Tests

```bash
# Run all tests across all packages
turbo test

# Run specific package tests
turbo test --filter=@orkee/cli
turbo test --filter=@orkee/dashboard

# Rust tests only
cargo test --all

# Specific Rust package
cd packages/cli && cargo test
cd packages/projects && cargo test
cd packages/tui && cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_project_creation

# Dashboard tests
cd packages/dashboard
pnpm test
```

### Writing Tests

**Rust Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project", "/path/to/project");
        assert_eq!(project.name, "Test Project");
    }

    #[tokio::test]
    async fn test_api_endpoint() {
        let response = api_client.get_projects().await.unwrap();
        assert!(!response.is_empty());
    }
}
```

**TypeScript Tests (Vitest):**

```typescript
import { describe, it, expect } from 'vitest'
import { ProjectCard } from './ProjectCard'

describe('ProjectCard', () => {
  it('renders project name', () => {
    const project = { id: 1, name: 'Test Project' }
    const { getByText } = render(<ProjectCard project={project} />)
    expect(getByText('Test Project')).toBeInTheDocument()
  })
})
```

## Linting and Formatting

### Running Linters

```bash
# Run all linters
turbo lint

# Fix automatically
turbo lint:fix

# Rust linting (clippy)
cargo clippy --all

# Fix Rust warnings
cargo clippy --fix

# TypeScript linting (ESLint)
cd packages/dashboard
pnpm lint

# Fix TypeScript issues
pnpm lint:fix
```

### Code Formatting

```bash
# Format all code
turbo format

# Rust formatting
cargo fmt --all

# Check formatting without changing
cargo fmt --all -- --check

# TypeScript formatting (Prettier)
cd packages/dashboard
pnpm format
```

## Debugging

### Rust Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin orkee -- dashboard

# Trace level logging
RUST_LOG=trace cargo run --bin orkee -- dashboard

# Module-specific logging
RUST_LOG=orkee::api=debug cargo run --bin orkee -- dashboard

# Use lldb (macOS/Linux)
rust-lldb target/debug/orkee

# Use gdb (Linux)
rust-gdb target/debug/orkee
```

### Dashboard Debugging

```bash
# Run with source maps
pnpm dev

# Browser DevTools:
# - Network tab: Inspect API calls
# - Console: Check for errors
# - React DevTools: Inspect component state
# - Redux DevTools: (if using Redux)
```

### Database Debugging

```bash
# Inspect database directly
sqlite3 ~/.orkee/orkee.db

# Common queries
.schema                            # View schema
SELECT * FROM projects;           # View all projects
SELECT * FROM project_tags;       # View tags
PRAGMA table_info(projects);     # Table structure

# Check database integrity
PRAGMA integrity_check;

# Optimize database
VACUUM;
ANALYZE;
```

## Hot Reload

### Dashboard Hot Reload

Vite provides instant hot module replacement:

```bash
cd packages/dashboard
pnpm dev

# Changes to .tsx, .ts, .css files reload instantly
```

### Rust Hot Reload

Use `cargo-watch` for Rust hot reload:

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch and rebuild on changes
cargo watch -x 'run --bin orkee -- dashboard'

# Watch and test
cargo watch -x test

# Watch specific package
cd packages/cli
cargo watch -x run
```

## Troubleshooting Development Issues

### Cargo Build Errors

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild from scratch
cargo clean && cargo build
```

### pnpm Issues

```bash
# Clear cache
pnpm store prune

# Reinstall dependencies
rm -rf node_modules pnpm-lock.yaml
pnpm install

# Rebuild specific package
turbo build --filter=@orkee/dashboard --force
```

### Port Conflicts

```bash
# Kill processes on port 4001 (API)
lsof -ti:4001 | xargs kill -9

# Kill processes on port 5173 (Dashboard)
lsof -ti:5173 | xargs kill -9

# Use different ports
ORKEE_API_PORT=8080 ORKEE_UI_PORT=3000 turbo dev
```

## Performance Profiling

### Rust Profiling

```bash
# Build with profiling enabled
cargo build --release --profile profiling

# macOS: Use Instruments
instruments -t "Time Profiler" target/release/orkee dashboard

# Linux: Use perf
perf record target/release/orkee dashboard
perf report

# Flamegraph
cargo install flamegraph
cargo flamegraph --bin orkee -- dashboard
```

### Dashboard Profiling

```bash
# Build with profiling
pnpm build

# Analyze bundle size
pnpm run analyze

# Chrome DevTools:
# Performance tab → Record → Stop → Analyze
```

## Useful Commands Reference

```bash
# Development
turbo dev                          # Start all dev servers
turbo dev --filter=<package>      # Start specific package
turbo build                        # Build all packages
turbo build --force               # Force rebuild
turbo test                        # Run all tests
turbo lint                        # Run all linters
turbo format                      # Format all code

# Rust-specific
cargo build                       # Dev build
cargo build --release            # Prod build
cargo test                       # Run tests
cargo clippy                     # Lint
cargo fmt                        # Format
cargo clean                      # Clean artifacts
cargo update                     # Update dependencies
cargo tree                       # Show dependency tree

# Dashboard-specific
pnpm dev                         # Start dev server
pnpm build                       # Production build
pnpm test                        # Run tests
pnpm lint                        # Lint code
pnpm format                      # Format code
pnpm preview                     # Preview production build
```

## Next Steps

- [Contributing Guidelines](contributing) - Learn how to contribute
- [Code Style Guide](../style-guide) - Code formatting standards
- [Testing Guide](../testing) - Writing and running tests
- [Architecture Overview](../../architecture) - System design
