---
sidebar_position: 1
---

# Dashboard Development Mode

Orkee supports a development mode that allows you to use the local dashboard source code instead of the downloaded production version, making dashboard development much more efficient.

## Overview

By default, Orkee downloads and uses a pre-built dashboard from GitHub releases stored in `~/.orkee/dashboard/`. During development, this means you would need to manually copy your changes to test them. Development mode eliminates this friction by using your local dashboard source directly.

## Enabling Development Mode

### Method 1: Command Line Flag (Recommended)

Use the `--dev` flag when starting the dashboard:

```bash
cargo run --bin orkee -- dashboard --dev
```

### Method 2: Environment Variable

Set the `ORKEE_DEV_MODE` environment variable:

```bash
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard
```

### Method 3: In .env File

Add to your `.env` file:

```bash
ORKEE_DEV_MODE=true
```

Then start normally:

```bash
cargo run --bin orkee -- dashboard
```

## How It Works

### Dashboard Path Resolution

1. **Development Mode Enabled**: 
   - Checks for local dashboard at `packages/dashboard/` relative to current working directory
   - If found, uses the local dashboard directly
   - If not found, falls back to downloaded version with a warning

2. **Development Mode Disabled** (default):
   - Always uses downloaded dashboard from `~/.orkee/dashboard/`
   - Downloads if not present or version mismatch

### Visual Indicators

When development mode is active, you'll see:

```bash
🔧 Using local development dashboard from /path/to/orkee-oss/packages/dashboard
```

When using downloaded dashboard:

```bash
📂 Using cached dashboard from /Users/username/.orkee/dashboard
```

## Benefits

- **🚀 No File Copying**: Work directly with source files
- **🔄 Live Reloading**: Vite's hot module replacement works immediately  
- **⚡ Faster Iteration**: Instant feedback on changes
- **🔒 Safe Fallback**: Automatically falls back if local dashboard isn't found

## Examples

### Basic Development

```bash
# Start in development mode
cargo run --bin orkee -- dashboard --dev

# Dashboard available at http://localhost:5173
# API available at http://localhost:4001
```

### With Custom Ports

```bash
# Development mode with custom ports
cargo run --bin orkee -- dashboard --dev --api-port 8080 --ui-port 3000
```

### Environment Variable Method

```bash
# Set environment variable for persistent dev mode
export ORKEE_DEV_MODE=true

# Start dashboard (will automatically use dev mode)
cargo run --bin orkee -- dashboard
```

## Requirements

For development mode to work, you need:

1. **Local Dashboard Source**: The `packages/dashboard/` directory must exist
2. **Dependencies Installed**: Run `pnpm install` in the dashboard directory
3. **Valid Package.json**: Dashboard must have proper Vite configuration

## Troubleshooting

### "Local dashboard not found" Warning

If you see this warning:

```
⚠️ Local dashboard not found, falling back to downloaded version
```

**Solutions**:
1. Ensure you're running from the project root directory
2. Check that `packages/dashboard/` exists
3. Verify the dashboard has been built with `pnpm install`

### Port Conflicts

If ports are already in use:

```bash
# Use different ports
cargo run --bin orkee -- dashboard --dev --api-port 4002 --ui-port 5174
```

### Dependencies Issues

If the local dashboard won't start:

```bash
# Install/update dependencies
cd packages/dashboard
pnpm install

# Return to project root
cd ../..
cargo run --bin orkee -- dashboard --dev
```

## Best Practices

1. **Use --dev flag**: More explicit than environment variables
2. **Check console output**: Look for the dashboard path indicator
3. **Keep dependencies updated**: Run `pnpm install` periodically
4. **Test both modes**: Ensure your changes work in production mode too

## Integration with Build Tools

Development mode works seamlessly with:

- **Turborepo**: `turbo dev --filter=@orkee/dashboard`
- **Vite HMR**: Hot module replacement for React components
- **TypeScript**: Live type checking
- **ESLint**: Real-time linting feedback

This makes development mode the ideal choice for dashboard development workflows.