#!/bin/bash
# ABOUTME: Linux post-install script for Orkee desktop app
# ABOUTME: Creates symlink to orkee binary in /usr/local/bin for CLI access

set -e

# Detect the actual install location (varies by package format)
# AppImage: typically in /opt or user's home
# .deb: /usr/bin or /opt
# .rpm: /usr/bin or /opt

# Try common install locations
POSSIBLE_LOCATIONS=(
    "/usr/bin/orkee"
    "/opt/Orkee/orkee"
    "/opt/orkee/orkee"
    "/usr/local/bin/orkee-desktop"
)

BINARY_SOURCE=""
for location in "${POSSIBLE_LOCATIONS[@]}"; do
    if [ -f "$location" ]; then
        BINARY_SOURCE="$location"
        break
    fi
done

if [ -z "$BINARY_SOURCE" ]; then
    echo "Warning: Could not locate orkee binary in standard locations"
    echo "CLI access may not be available. Desktop app will still work."
    exit 0  # Don't fail installation
fi

BINARY_TARGET="/usr/local/bin/orkee"

echo "Installing orkee CLI binary..."

# Create /usr/local/bin if it doesn't exist
mkdir -p /usr/local/bin

# Create symlink (or copy if symlink fails)
if ln -sf "$BINARY_SOURCE" "$BINARY_TARGET" 2>/dev/null; then
    echo "✓ Created symlink: $BINARY_TARGET -> $BINARY_SOURCE"
elif cp "$BINARY_SOURCE" "$BINARY_TARGET" 2>/dev/null; then
    chmod +x "$BINARY_TARGET"
    echo "✓ Copied binary to $BINARY_TARGET"
else
    echo "Warning: Could not install CLI binary (insufficient permissions)"
    echo "Desktop app will still work. For CLI access, run: sudo ln -s $BINARY_SOURCE $BINARY_TARGET"
    exit 0  # Don't fail installation
fi

# Verify installation
if [ -f "$BINARY_TARGET" ] || [ -L "$BINARY_TARGET" ]; then
    echo "✓ orkee CLI is now available"
    "$BINARY_TARGET" --version 2>/dev/null || echo "Desktop app installed successfully"
fi

exit 0
