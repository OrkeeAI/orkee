#!/bin/bash
# ABOUTME: macOS post-install script for Orkee desktop app
# ABOUTME: Copies orkee binary to /usr/local/bin for system-wide CLI access

set -e

# This script runs after the .app bundle is installed
# The binary is located inside the app bundle at:
# /Applications/Orkee.app/Contents/MacOS/orkee

APP_BUNDLE="/Applications/Orkee.app"
BINARY_SOURCE="$APP_BUNDLE/Contents/MacOS/orkee"
BINARY_TARGET="/usr/local/bin/orkee"

echo "Installing orkee CLI binary..."

# Check if app bundle exists
if [ ! -d "$APP_BUNDLE" ]; then
    echo "Error: Orkee.app not found at $APP_BUNDLE"
    exit 1
fi

# Check if binary exists in bundle
if [ ! -f "$BINARY_SOURCE" ]; then
    echo "Error: orkee binary not found at $BINARY_SOURCE"
    exit 1
fi

# Create /usr/local/bin if it doesn't exist
mkdir -p /usr/local/bin

# Copy binary to /usr/local/bin (requires admin privileges)
cp "$BINARY_SOURCE" "$BINARY_TARGET"

# Make it executable
chmod +x "$BINARY_TARGET"

# Verify installation
if [ -f "$BINARY_TARGET" ]; then
    echo "✓ orkee binary installed to $BINARY_TARGET"
    echo "✓ You can now use 'orkee' commands in your terminal"

    # Show version
    "$BINARY_TARGET" --version || echo "Warning: Could not verify orkee version"
else
    echo "Error: Failed to install orkee binary"
    exit 1
fi

exit 0
