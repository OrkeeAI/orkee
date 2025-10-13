#!/usr/bin/env bash
# ABOUTME: macOS pre-uninstall script for Orkee desktop app
# ABOUTME: Removes orkee binary from /usr/local/bin

set -euo pipefail

# Enable debug mode if DEBUG env var is set
if [ "${DEBUG:-}" = "1" ]; then
    set -x
fi

BINARY_TARGET="/usr/local/bin/orkee"

echo "Removing orkee CLI binary..."

# Remove binary if it exists
if [ -f "$BINARY_TARGET" ]; then
    if rm "$BINARY_TARGET" 2>/dev/null; then
        echo "✓ orkee binary removed from $BINARY_TARGET"
    else
        echo "Warning: Could not remove $BINARY_TARGET (insufficient permissions)"
        echo "You may need to manually remove it: sudo rm $BINARY_TARGET"
    fi
else
    echo "orkee binary not found at $BINARY_TARGET (already removed or never installed)"
fi

exit 0
