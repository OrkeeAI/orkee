#!/bin/bash
# ABOUTME: Helper script to rebuild the Orkee desktop app with the latest CLI binary
# ABOUTME: Ensures the bundled orkee sidecar is always up-to-date

set -e  # Exit on error

echo "🔨 Building Orkee Desktop App"
echo "=============================="
echo ""

# Step 1: Build the CLI binary
echo "📦 Step 1/3: Building orkee CLI (release mode)..."
cd "$(dirname "$0")/../.."  # Go to workspace root
cargo build --release --package orkee-cli
echo "✅ CLI built successfully"
echo ""

# Step 2: Build the Tauri desktop app
echo "📦 Step 2/3: Building Tauri desktop app..."
cd packages/dashboard
bun run tauri build
echo "✅ Desktop app built successfully"
echo ""

# Step 3: Show the output
echo "📦 Step 3/3: Build artifacts:"
echo ""
echo "  macOS App:"
echo "    src-tauri/target/release/bundle/macos/Orkee.app"
echo ""
echo "  macOS DMG:"
echo "    src-tauri/target/release/bundle/dmg/Orkee_0.0.7_aarch64.dmg"
echo ""
echo "✅ Build complete!"
echo ""
echo "To install, run:"
echo "  rm -rf /Applications/Orkee.app"
echo "  cp -R packages/dashboard/src-tauri/target/release/bundle/macos/Orkee.app /Applications/"
echo "  open /Applications/Orkee.app"
