#!/usr/bin/env bash
# ABOUTME: macOS post-install script for Orkee desktop app
# ABOUTME: Copies orkee binary to /usr/local/bin for system-wide CLI access

set -e

# This script runs after the .app bundle is installed
# The binary is located inside the app bundle at:
# <install_location>/Orkee.app/Contents/MacOS/orkee

BINARY_TARGET="/usr/local/bin/orkee"

echo "Installing orkee CLI binary..."

# Try common install locations
POSSIBLE_LOCATIONS=(
    "/Applications/Orkee.app"
    "$HOME/Applications/Orkee.app"
    "/opt/Orkee.app"
    "/usr/local/Orkee.app"
)

APP_BUNDLE=""
for location in "${POSSIBLE_LOCATIONS[@]}"; do
    if [ -d "$location" ]; then
        APP_BUNDLE="$location"
        break
    fi
done

# Check if app bundle was found
if [ -z "$APP_BUNDLE" ]; then
    echo "Warning: Could not locate Orkee.app in standard locations"
    echo "Searched: ${POSSIBLE_LOCATIONS[*]}"
    echo "Desktop app will still work. For CLI access, manually copy:"
    echo "  <Orkee.app>/Contents/MacOS/orkee -> /usr/local/bin/orkee"
    exit 0  # Don't fail installation
fi

BINARY_SOURCE="$APP_BUNDLE/Contents/MacOS/orkee"
echo "Found Orkee.app at: $APP_BUNDLE"

# Check if binary exists in bundle
if [ ! -f "$BINARY_SOURCE" ]; then
    echo "Error: orkee binary not found at $BINARY_SOURCE"
    exit 1
fi

# Create /usr/local/bin if it doesn't exist
if ! mkdir -p /usr/local/bin 2>/dev/null; then
    echo "Error: Could not create /usr/local/bin (insufficient permissions)"
    echo "Desktop app will still work. For CLI access, run with sudo:"
    echo "  sudo mkdir -p /usr/local/bin"
    echo "  sudo cp \"$BINARY_SOURCE\" \"$BINARY_TARGET\""
    exit 0  # Don't fail installation
fi

# Copy binary to /usr/local/bin (requires admin privileges)
if ! cp "$BINARY_SOURCE" "$BINARY_TARGET" 2>/dev/null; then
    echo "Error: Could not copy binary to $BINARY_TARGET (insufficient permissions)"
    echo "Desktop app will still work. For CLI access, run:"
    echo "  sudo cp \"$BINARY_SOURCE\" \"$BINARY_TARGET\""
    exit 0  # Don't fail installation
fi

# Make it executable
if ! chmod +x "$BINARY_TARGET" 2>/dev/null; then
    echo "Warning: Could not make binary executable"
    echo "Desktop app will still work. For CLI access, run:"
    echo "  sudo chmod +x \"$BINARY_TARGET\""
    exit 0  # Don't fail installation
fi

# Verify installation
if [ -f "$BINARY_TARGET" ]; then
    echo "✓ orkee binary installed to $BINARY_TARGET"
    echo "✓ You can now use 'orkee' commands in your terminal"

    # Show version (non-fatal if it fails)
    if ! "$BINARY_TARGET" --version 2>/dev/null; then
        echo "Note: Could not verify orkee version (binary may need first-run initialization)"
    fi
else
    echo "Error: Failed to install orkee binary"
    exit 1
fi

exit 0
