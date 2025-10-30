# Tauri Desktop App Rebuild Guide

## The Issue

When running the **Orkee desktop app** (Tauri), you're seeing "Failed to update credentials" because:

1. The Tauri app bundles the Rust backend (CLI) inside it
2. When you make code changes to the Rust backend, you need to rebuild the CLI
3. Then rebuild the Tauri app to include the updated CLI

## Current Status

✅ **CLI rebuilt successfully!** (Release build completed in 1m 32s)

The CLI binary is now at:
```
/Users/danziger/code/orkee/orkee-oss/target/release/orkee
```

## Next Steps

### Option 1: Rebuild Tauri App (Recommended)

Since you're using the desktop app, you need to rebuild it to include the updated CLI:

```bash
cd /Users/danziger/code/orkee/orkee-oss

# Rebuild the Tauri app (this will include the updated CLI)
cd packages/desktop
bun run tauri build
# OR for dev mode:
bun run tauri dev
```

### Option 2: Use Web Version (Faster for Development)

For faster iteration during development, use the web version instead:

```bash
# Terminal 1: Start the CLI backend directly
cd /Users/danziger/code/orkee/orkee-oss
export ANTHROPIC_API_KEY="your-key-here"
./target/release/orkee serve

# Terminal 2: Start the web dashboard
cd packages/dashboard
bun run dev
```

Then open http://localhost:5173 in your browser.

## Why This Happens

### Tauri App Architecture
```
┌─────────────────────────────────────┐
│     Tauri Desktop App               │
│                                     │
│  ┌──────────────┐  ┌─────────────┐ │
│  │   Frontend   │  │   Backend   │ │
│  │  (React/TS)  │  │  (Rust CLI) │ │
│  │              │  │             │ │
│  │  Dashboard   │→ │  orkee-cli  │ │
│  │  (Vite)      │  │  (Bundled)  │ │
│  └──────────────┘  └─────────────┘ │
│                                     │
│  Port: Dynamic (e.g., 24156)        │
└─────────────────────────────────────┘
```

When you change Rust code:
1. ❌ Frontend hot-reloads automatically
2. ❌ Backend does NOT reload (it's compiled into the app)
3. ✅ You must rebuild the CLI
4. ✅ Then rebuild/restart the Tauri app

### Web Version Architecture
```
┌──────────────┐         ┌─────────────┐
│   Frontend   │         │   Backend   │
│  (React/TS)  │         │  (Rust CLI) │
│              │         │             │
│  Dashboard   │────────→│  orkee-cli  │
│  (Vite)      │  HTTP   │  (Separate) │
└──────────────┘         └─────────────┘
   Port: 5173              Port: 4001
   Auto-reload             Manual restart
```

When you change Rust code:
1. ❌ Frontend hot-reloads automatically
2. ✅ Restart the CLI backend manually
3. ✅ No Tauri rebuild needed!

## Commands Reference

### Build CLI Only (What We Just Did)
```bash
cargo build --package orkee-cli --bin orkee --release
```

### Build CLI (Debug - Faster)
```bash
cargo build --package orkee-cli --bin orkee
# Binary at: target/debug/orkee
```

### Build Tauri App (Release)
```bash
cd packages/desktop
bun run tauri build
# App at: packages/desktop/src-tauri/target/release/bundle/
```

### Build Tauri App (Dev Mode)
```bash
cd packages/desktop
bun run tauri dev
# Opens app with hot-reload for frontend only
```

### Run CLI Directly
```bash
# Debug build
./target/debug/orkee serve

# Release build (faster, optimized)
./target/release/orkee serve
```

## Troubleshooting

### "Failed to update credentials" Still Happening

**Cause**: Tauri app is still using old CLI binary

**Fix**:
```bash
# Option A: Rebuild Tauri app
cd packages/desktop
bun run tauri dev

# Option B: Use web version
./target/release/orkee serve
# Then open http://localhost:5173
```

### "Anthropic API key not configured"

**Cause**: Environment variable not set or not passed to the app

**Fix**:
```bash
# For CLI:
export ANTHROPIC_API_KEY="sk-ant-..."
./target/release/orkee serve

# For Tauri (set before building):
export ANTHROPIC_API_KEY="sk-ant-..."
cd packages/desktop
bun run tauri dev
```

### Changes Not Appearing

**Checklist**:
- [ ] Did you rebuild the CLI? (`cargo build --package orkee-cli --bin orkee --release`)
- [ ] Did you restart/rebuild the Tauri app? (`bun run tauri dev`)
- [ ] Are you using the correct binary? (Check port in logs)
- [ ] Did you clear browser cache? (Cmd+Shift+R)

## Development Workflow Recommendations

### For Backend Changes (Rust)
1. ✅ Use **web version** for faster iteration
2. ✅ Make changes to Rust code
3. ✅ Rebuild CLI: `cargo build --package orkee-cli --bin orkee`
4. ✅ Restart CLI: `./target/debug/orkee serve`
5. ✅ Test in browser at http://localhost:5173

### For Frontend Changes (TypeScript/React)
1. ✅ Either Tauri or web version works
2. ✅ Make changes to TypeScript/React code
3. ✅ Vite hot-reloads automatically
4. ✅ No rebuild needed!

### For Full Integration Testing
1. ✅ Build CLI: `cargo build --package orkee-cli --bin orkee --release`
2. ✅ Build Tauri: `cd packages/desktop && bun run tauri build`
3. ✅ Test the bundled app

## Current Build Status

✅ **CLI built successfully** (Release mode)
- Binary: `/Users/danziger/code/orkee/orkee-oss/target/release/orkee`
- Includes: Latest model selection code
- Includes: Updated AI service with environment variable support

⏳ **Tauri app needs rebuild**
- The desktop app is still using the old CLI
- Rebuild with: `cd packages/desktop && bun run tauri dev`

## Quick Fix for Your Current Issue

Since you're using the Tauri app and getting "Failed to update credentials":

```bash
# Option 1: Rebuild Tauri app (5-10 minutes)
cd /Users/danziger/code/orkee/orkee-oss/packages/desktop
bun run tauri dev

# Option 2: Switch to web version (30 seconds)
cd /Users/danziger/code/orkee/orkee-oss
export ANTHROPIC_API_KEY="your-key-here"
./target/release/orkee serve
# Then open http://localhost:5173 in browser
```

**Recommendation**: Use Option 2 (web version) for development - it's much faster!

## Files Modified (Included in New Build)

The CLI now includes these changes:
1. ✅ `packages/projects/src/ai_service.rs` - Model selection via env var
2. ✅ `packages/projects/src/api/ai_handlers.rs` - Logs actual model used
3. ✅ `packages/projects/src/users/storage.rs` - Credentials update logic

The frontend changes (ModelSelectionDialog, etc.) work with both Tauri and web versions.

## Performance Comparison

| Action | Tauri App | Web Version |
|--------|-----------|-------------|
| **Rust code change** | Rebuild CLI (1-2 min) + Rebuild Tauri (5-10 min) | Rebuild CLI (1-2 min) + Restart (5 sec) |
| **Frontend change** | Hot reload (instant) | Hot reload (instant) |
| **Full rebuild** | 10-15 minutes | 2-3 minutes |
| **Startup time** | 5-10 seconds | 2-3 seconds |

**Verdict**: Use web version for development, Tauri for final testing/distribution.
