#!/usr/bin/env bash
# ABOUTME: Linux post-install script for Orkee desktop app
# ABOUTME: Creates symlink to orkee binary in /usr/local/bin for CLI access

set -euo pipefail

# Enable debug mode if DEBUG env var is set
if [ "${DEBUG:-}" = "1" ]; then
    set -x
fi

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
if ! mkdir -p /usr/local/bin 2>/dev/null; then
    echo "Warning: Could not create /usr/local/bin (insufficient permissions)"
    echo "Desktop app will still work. For CLI access, run with sudo:"
    echo "  sudo mkdir -p /usr/local/bin"
    echo "  sudo ln -s $BINARY_SOURCE /usr/local/bin/orkee"
    exit 0  # Don't fail installation
fi

# Re-verify binary exists right before use (prevents TOCTOU race condition)
if [ ! -f "$BINARY_SOURCE" ]; then
    echo "Warning: Binary disappeared between detection and installation"
    echo "Desktop app will still work. CLI access may not be available."
    exit 0  # Don't fail installation
fi

# Create symlink (or copy if symlink fails)
if ln -sf "$BINARY_SOURCE" "$BINARY_TARGET" 2>/dev/null; then
    echo "✓ Created symlink: $BINARY_TARGET -> $BINARY_SOURCE"
elif cp "$BINARY_SOURCE" "$BINARY_TARGET" 2>/dev/null; then
    if ! chmod +x "$BINARY_TARGET" 2>/dev/null; then
        echo "Warning: Binary copied but could not make it executable"
        echo "Desktop app will still work. For CLI access, run: sudo chmod +x $BINARY_TARGET"
        exit 0  # Don't fail installation
    fi
    echo "✓ Copied binary to $BINARY_TARGET"
else
    echo "Warning: Could not install CLI binary (insufficient permissions)"
    echo "Desktop app will still work. For CLI access, run: sudo ln -s $BINARY_SOURCE $BINARY_TARGET"
    exit 0  # Don't fail installation
fi

# Verify installation
if [ -f "$BINARY_TARGET" ] || [ -L "$BINARY_TARGET" ]; then
    echo "✓ orkee CLI is now available"

    # Show version
    "$BINARY_TARGET" --version 2>/dev/null || echo "Desktop app installed successfully"
fi

exit 0
