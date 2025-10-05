# Orkee Desktop (Tauri) Setup Guide

This document explains the Tauri desktop application setup for Orkee.

## Overview

Orkee now supports both **web** and **desktop** platforms from a single codebase:

- **Web version**: Runs in any modern browser
- **Desktop version**: Native application for Windows, macOS, and Linux (powered by Tauri)

## Project Structure

```
packages/dashboard/
├── src/                      # Shared React application code
│   ├── lib/
│   │   └── platform.ts      # Platform detection utilities
│   ├── services/
│   │   └── api.ts           # Platform-aware API client
│   └── ...
├── src-tauri/               # Tauri-specific backend (Rust)
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   └── lib.rs           # App logic
│   ├── icons/               # App icons (needs generation)
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # Tauri configuration
│   └── build.rs             # Build script
├── dist/                    # Build output (web assets)
└── package.json             # Node.js dependencies & scripts
```

## Prerequisites

Before working with the desktop version, ensure you have:

1. **Rust** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **System dependencies** (varies by OS)
   - **macOS**: Xcode Command Line Tools
     ```bash
     xcode-select --install
     ```
   - **Linux**: See [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)
   - **Windows**: Microsoft Visual Studio C++ Build Tools

## Development Commands

### Web Development (existing workflow)

```bash
# Start web dev server (Vite on port 5173)
pnpm dev

# Build for web
pnpm build

# Preview web build
pnpm preview
```

### Desktop Development (new)

```bash
# Start desktop app in development mode
# This runs the web dev server AND opens Tauri window
pnpm tauri:dev

# Build desktop app for production
# Creates installers/bundles for your current platform
pnpm tauri:build

# Tauri CLI (for advanced usage)
pnpm tauri --help
```

## How It Works

### Platform Detection

The app uses `src/lib/platform.ts` to detect the runtime environment:

```typescript
import { isTauriApp, isWebApp, getPlatform } from '@/lib/platform';

if (isTauriApp()) {
  // Desktop-specific code
  console.log('Running in Tauri desktop app');
} else {
  // Web-specific code
  console.log('Running in web browser');
}
```

### API Communication

Both platforms connect to the Orkee CLI server (default: `http://localhost:4001`):

- **Web**: Uses Vite proxy configuration (configured in `vite.config.ts`)
- **Desktop**: Can connect directly to localhost or embedded server

The API client in `src/services/api.ts` automatically handles platform differences.

### Shared Codebase Benefits

- **95%+ code reuse**: Same React components, logic, and styling
- **Progressive enhancement**: Desktop features only activate when available
- **Consistent behavior**: Same API interactions across platforms
- **Easier maintenance**: Single codebase to update

## Desktop-Specific Features (Future)

The platform detection system enables:

- **File system access**: Native file dialogs and direct file operations
- **System tray**: Background operation with tray icon
- **Native menus**: Platform-native menu bars
- **Auto-updates**: Built-in update mechanism
- **Offline-first**: Better local data persistence
- **Native notifications**: System-level notifications

## Building & Distribution

### Development Build

```bash
pnpm tauri:dev
```

This:
1. Starts the Vite dev server
2. Compiles the Rust backend
3. Opens the Tauri window with hot-reload

### Production Build

```bash
pnpm tauri:build
```

This creates platform-specific installers in `src-tauri/target/release/bundle/`:

- **macOS**: `.dmg` and `.app`
- **Windows**: `.msi` and `.exe`
- **Linux**: `.deb`, `.AppImage`, or `.rpm`

### Cross-Platform Building

By default, Tauri builds for your current platform. For cross-platform builds, see the [Tauri documentation](https://tauri.app/v1/guides/building/cross-platform).

## Icons

Before building a production desktop app, generate proper icons:

1. Create a 1024x1024px PNG logo
2. Run: `pnpm tauri icon path/to/logo.png`

See `src-tauri/icons/README.md` for details.

## Configuration

### Tauri Config (`src-tauri/tauri.conf.json`)

Key settings:
- **Window size**: `width`, `height`, `minWidth`, `minHeight`
- **App metadata**: `productName`, `version`, `identifier`
- **Security**: CSP and permissions
- **Bundle**: Icons, copyright, descriptions

### Vite Config (`vite.config.ts`)

The existing Vite config works for both platforms. The web dev server runs on port 5173 (configurable via `ORKEE_UI_PORT`).

## CLI Server Dependency

Both web and desktop versions require the Orkee CLI server to be running:

```bash
# In packages/cli/
cargo run --bin orkee -- dashboard
```

### Future: Embedded Server

We could bundle the CLI server directly into the Tauri app, eliminating this dependency:

1. Package the Orkee binary as a Tauri resource
2. Start it on app launch via Tauri commands
3. Handle lifecycle (shutdown on app close)

## Troubleshooting

### Icons Missing

**Error**: "Error: unable to find icon file"

**Solution**: Generate icons or use placeholders:
```bash
pnpm tauri icon path/to/logo.png
```

### Rust Not Found

**Error**: "cargo: command not found"

**Solution**: Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build Errors on macOS

**Error**: "xcrun: error: unable to find utility"

**Solution**: Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### API Connection Issues

**Error**: "Failed to connect to localhost:4001"

**Solution**: Ensure the CLI server is running:
```bash
cd packages/cli && cargo run --bin orkee -- dashboard
```

## Testing Both Versions

### Test Web Version

```bash
# Terminal 1: Start CLI server
cd packages/cli
cargo run --bin orkee -- dashboard

# Terminal 2: Web browser opens automatically
# Or visit http://localhost:5173
```

### Test Desktop Version

```bash
# Terminal 1: Start CLI server (if not auto-started)
cd packages/cli
cargo run --bin orkee -- dashboard

# Terminal 2: Start desktop app
cd packages/dashboard
pnpm tauri:dev
```

## Migration Notes

### Existing Web Users

No changes required! The web version works exactly as before:

```bash
pnpm dev          # Web development
pnpm build        # Web production
```

### New Desktop Users

Install Rust, then:

```bash
pnpm tauri:dev    # Desktop development
pnpm tauri:build  # Desktop production
```

## Next Steps

1. **Generate proper icons** for production builds
2. **Test desktop features** like file dialogs
3. **Consider embedding** the CLI server
4. **Set up auto-updates** for desktop distribution
5. **Create installers** for each platform

## Resources

- [Tauri Documentation](https://tauri.app)
- [Tauri API Reference](https://tauri.app/v1/api/js/)
- [Orkee Project Documentation](../../CLAUDE.md)

---

**Note**: This is the initial Tauri setup. Desktop-specific features (file access, system tray, etc.) can be added progressively without affecting the web version.
