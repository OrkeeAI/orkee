---
sidebar_position: 3
title: Source Installation
---

# Source Installation

Build Orkee from source for development, customization, or to access the latest features.

## Prerequisites

Before building from source, ensure you have:

- **Rust**: Latest stable toolchain
- **Node.js**: v18+ 
- **pnpm**: v8+ package manager
- **Git**: For cloning the repository

## Quick Start

```bash
# Clone the repository
git clone https://github.com/OrkeeAI/orkee.git
cd orkee

# Install dependencies
pnpm install

# Build the project
turbo build

# Run Orkee
cargo run --bin orkee -- --help
```

## Detailed Installation

### 1. Install Prerequisites

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="rust" label="Install Rust" default>

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

</TabItem>
<TabItem value="nodejs" label="Install Node.js & pnpm">

```bash
# Install Node.js (https://nodejs.org)
# Then install pnpm
npm install -g pnpm@latest

# Verify installation
node --version
pnpm --version
```

</TabItem>
</Tabs>

### 2. Clone Repository

```bash
git clone https://github.com/OrkeeAI/orkee.git
cd orkee
```

### 3. Install Dependencies

```bash
# Install Node.js dependencies
pnpm install

# Optional: Install additional Rust targets
rustup target add x86_64-unknown-linux-gnu
```

### 4. Build Project

<Tabs>
<TabItem value="development" label="Development Build" default>

```bash
# Build all packages in development mode
turbo build

# Or build specific packages
turbo build --filter=@orkee/cli
turbo build --filter=@orkee/dashboard
```

</TabItem>
<TabItem value="release" label="Release Build">

```bash
# Build optimized release binaries
cargo build --release

# Or with cloud features
cargo build --release --features cloud
```

</TabItem>
</Tabs>

### 5. Run Orkee

```bash
# Development: Run from source
cargo run --bin orkee -- dashboard

# Release: Use built binary
./target/release/orkee dashboard
```

## Build Options

### Feature Flags

Orkee supports optional features that can be enabled during build:

```bash
# Build with cloud sync features
cargo build --features cloud

# Build without cloud features (smaller binary)
cargo build

# List all available features
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "orkee-cli") | .features'
```

### Target Platforms

Build for different platforms:

```bash
# Install cross-compilation tool
cargo install cross

# Build for Linux
cross build --release --target x86_64-unknown-linux-gnu

# Build for Windows
cross build --release --target x86_64-pc-windows-msvc

# Build for macOS (on macOS)
cargo build --release --target x86_64-apple-darwin
```

### Development vs Release

<Tabs>
<TabItem value="development" label="Development Build" default>

**Command**: `cargo build`

**Characteristics**:
- Faster compilation time
- Debug symbols included
- No optimizations
- Larger binary size
- Better for testing and debugging

</TabItem>
<TabItem value="release" label="Release Build">

**Command**: `cargo build --release`

**Characteristics**:
- Slower compilation time
- Optimized performance
- Smaller binary size
- Production ready
- No debug information

</TabItem>
</Tabs>

## Development Workflow

### Running in Development

```bash
# Start the full development stack
turbo dev

# Run CLI in development mode
cargo run --bin orkee -- dashboard

# Run with custom ports
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000

# Run with debug logging
RUST_LOG=debug cargo run --bin orkee -- dashboard
```

### Testing

```bash
# Run all tests
turbo test

# Run Rust tests only
cargo test

# Run with output
cargo test -- --nocapture

# Test specific package
cargo test -p orkee-cli
```

### Code Quality

```bash
# Format code
turbo format

# Run linter
turbo lint

# Run Rust-specific tools
cargo fmt
cargo clippy
```

## Troubleshooting

### Common Build Issues

#### Rust Version Too Old

```bash
# Update Rust
rustup update stable
rustc --version
```

#### Missing Dependencies (Linux)

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL/Fedora
sudo dnf install gcc gcc-c++ openssl-devel
```

#### Node.js Issues

```bash
# Clear npm cache
npm cache clean --force

# Reinstall dependencies
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

#### Permission Errors

```bash
# Fix permissions
chmod +x ./target/release/orkee

# Run without installing
cargo run --bin orkee -- --help
```

### Build Optimization

#### Faster Builds

```bash
# Use more CPU cores
export MAKEFLAGS="-j$(nproc)"

# Use faster linker (Linux)
sudo apt install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Use Rust cache
cargo install sccache
export RUSTC_WRAPPER=sccache
```

#### Smaller Binaries

```bash
# Strip symbols
cargo build --release
strip target/release/orkee

# Enable LTO (Link Time Optimization)
# Add to Cargo.toml [profile.release] section:
# lto = true
# codegen-units = 1
```

## Installation

After building, you can install the binary:

```bash
# Install to ~/.cargo/bin
cargo install --path packages/cli

# Or copy to system location
sudo cp target/release/orkee /usr/local/bin/

# Verify installation
orkee --version
```

## Keeping Updated

```bash
# Update source code
git pull origin main

# Update dependencies
pnpm install

# Rebuild
turbo build
```

## Contributing

If you're building from source to contribute:

1. **Fork the repository** on GitHub
2. **Create a feature branch**: `git checkout -b feature-name`
3. **Make your changes** and test thoroughly
4. **Run quality checks**: `turbo lint && turbo test`
5. **Submit a pull request** with a clear description

See our [Contributing Guide](../../development/contributing) for detailed guidelines.

## Next Steps

- [Quick Start Guide](../quick-start) - Get up and running
- [Development Guide](../../development/setup) - Set up for development
- [Configuration](../../configuration/environment-variables) - Customize your setup