#!/bin/bash
# ABOUTME: Linux pre-uninstall script for Orkee desktop app
# ABOUTME: Removes orkee binary/symlink from /usr/local/bin

set -e

BINARY_TARGET="/usr/local/bin/orkee"

echo "Removing orkee CLI binary..."

# Remove binary or symlink if it exists
if [ -f "$BINARY_TARGET" ] || [ -L "$BINARY_TARGET" ]; then
    rm -f "$BINARY_TARGET" 2>/dev/null || {
        echo "Warning: Could not remove $BINARY_TARGET (insufficient permissions)"
        echo "You may need to manually remove it: sudo rm $BINARY_TARGET"
    }
    echo "✓ orkee CLI removed"
else
    echo "orkee binary not found (already removed or never installed)"
fi

exit 0
