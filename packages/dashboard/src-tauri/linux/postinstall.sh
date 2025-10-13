#!/usr/bin/env bash
# ABOUTME: Linux post-install script for Orkee desktop app
# ABOUTME: Creates symlink to orkee binary in /usr/local/bin for CLI access

set -e

# Verify binary version matches expected version (if VERSION env var is set)
verify_binary_version() {
    local binary_path="$1"

    if [ -n "$ORKEE_VERSION" ]; then
        local actual_version
        actual_version=$("$binary_path" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1 || echo "unknown")

        if [ "$actual_version" != "$ORKEE_VERSION" ] && [ "$actual_version" != "unknown" ]; then
            echo "Warning: Binary version ($actual_version) doesn't match expected version ($ORKEE_VERSION)"
            return 1
        fi

        echo "✓ Binary version verified: $actual_version"
    else
        echo "Note: Version verification skipped (ORKEE_VERSION not set)"
    fi

    return 0
}

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

    # Verify binary version if specified
    verify_binary_version "$BINARY_TARGET" || echo "Note: Version mismatch detected but installation will continue"

    # Show version
    "$BINARY_TARGET" --version 2>/dev/null || echo "Desktop app installed successfully"
fi

exit 0
