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
    rm "$BINARY_TARGET"
    echo "✓ orkee binary removed from $BINARY_TARGET"
else
    echo "orkee binary not found at $BINARY_TARGET (already removed or never installed)"
fi

exit 0
