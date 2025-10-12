# Building the Orkee Desktop App

This guide explains how to build the Orkee desktop application using Tauri.

## Prerequisites

- Rust toolchain (latest stable)
- Bun package manager
- Node.js v18+
- Platform-specific requirements:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: Build essentials and webkit2gtk
  - **Windows**: Visual Studio Build Tools

## Build Process

### 1. Prepare CLI Binary

Before building the Tauri app, you need to build the CLI binary and copy it to the Tauri binaries directory:

```bash
cd packages/dashboard
bun run prepare-binaries
```

This script will:
- Detect your platform (macOS/Linux/Windows)
- Build the CLI binary for your architecture
- Copy it to `src-tauri/binaries/` with the correct platform-specific name

### 2. Build the Dashboard

Build the React frontend:

```bash
bun run build
```

### 3. Build the Tauri App

Build the desktop application:

```bash
bun run tauri:build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Development Mode

For development, you can run the Tauri app in dev mode:

```bash
# First, prepare the CLI binary
bun run prepare-binaries

# Then start dev mode
bun run tauri:dev
```

This will start the Vite dev server and open the Tauri window.

## Platform-Specific Notes

### macOS

The app uses private macOS APIs for system tray functionality (`macOSPrivateApi: true` in tauri.conf.json). This is required for:
- Hiding the dock icon
- Menu bar-only operation
- Application activation policy management

### Cross-Platform Builds

To build for multiple platforms, use the build script from the project root:

```bash
# Build CLI for all platforms
./scripts/build-all-platforms.sh
```

This will create binaries for:
- macOS (x86_64 and arm64)
- Linux (x86_64 and arm64)
- Windows (x86_64)

## Binary Management

**Important**: CLI binaries should NOT be committed to git. They are:
- Excluded via `.gitignore`
- Built as needed using the `prepare-binaries.sh` script
- Platform and architecture specific

If you see binary files in git history, they should be removed using:
```bash
git rm --cached packages/dashboard/src-tauri/binaries/*
git rm --cached packages/cli/orkee
```

## Troubleshooting

### Missing Binary Error

If Tauri fails to build with "binary not found" error:
1. Run `bun run prepare-binaries` first
2. Verify the binary exists in `src-tauri/binaries/`
3. Check that the binary name matches your platform in `tauri.conf.json`

### Build Permission Errors

Ensure the prepare-binaries script is executable:
```bash
chmod +x prepare-binaries.sh
```

### Cross-Compilation Issues

For cross-compilation, you may need additional tools:
- **Linux/Windows from macOS**: Install `cross` via `cargo install cross`
- **macOS targets**: Install via `rustup target add <target-triple>`
