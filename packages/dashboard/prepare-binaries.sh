#!/bin/bash
set -e

# ABOUTME: Builds CLI binary and copies it to Tauri binaries directory for bundling
# ABOUTME: This ensures the Tauri app has the correct CLI binary for the current platform

echo "Preparing CLI binary for Tauri..."

# Allow target override via environment variable (for CI cross-compilation)
if [ -n "$TAURI_TARGET" ]; then
    TARGET="$TAURI_TARGET"
    echo "Using target from TAURI_TARGET environment variable: $TARGET"
else
    # Detect current platform
    OS=$(uname -s)
    ARCH=$(uname -m)

    # Determine target triple
    if [ "$OS" = "Darwin" ]; then
        if [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        else
            TARGET="x86_64-apple-darwin"
        fi
    elif [ "$OS" = "Linux" ]; then
        if [ "$ARCH" = "x86_64" ]; then
            TARGET="x86_64-unknown-linux-gnu"
        elif [ "$ARCH" = "aarch64" ]; then
            TARGET="aarch64-unknown-linux-gnu"
        else
            echo "Error: Unsupported Linux architecture: $ARCH"
            exit 1
        fi
    elif [[ "$OS" =~ ^(MINGW|MSYS|CYGWIN) ]]; then
        TARGET="x86_64-pc-windows-msvc"
    else
        echo "Error: Unsupported OS: $OS"
        exit 1
    fi
fi

# Determine binary extension based on target
BINARY_EXT=""
if [[ "$TARGET" =~ windows ]]; then
    BINARY_EXT=".exe"
fi

echo "Building for target: $TARGET"

# Build CLI binary
cd "$(dirname "$0")/../cli"
cargo build --release --target "$TARGET"

# Verify binary was created
BINARY_PATH="../../target/$TARGET/release/orkee$BINARY_EXT"
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Build may have failed. Check cargo output above."
    exit 1
fi

echo "✓ Binary built successfully: $BINARY_PATH"

# Create binaries directory if it doesn't exist
BINARIES_DIR="../dashboard/src-tauri/binaries"
mkdir -p "$BINARIES_DIR"

# Copy binary to Tauri binaries directory with platform-specific name
BINARY_NAME="orkee-$TARGET"
cp "$BINARY_PATH" "$BINARIES_DIR/$BINARY_NAME$BINARY_EXT"

# Verify copy succeeded
if [ ! -f "$BINARIES_DIR/$BINARY_NAME$BINARY_EXT" ]; then
    echo "Error: Failed to copy binary to $BINARIES_DIR/$BINARY_NAME$BINARY_EXT"
    exit 1
fi

echo "✓ Binary prepared: $BINARIES_DIR/$BINARY_NAME$BINARY_EXT"
echo ""
echo "You can now build the Tauri app with:"
echo "  cd packages/dashboard"
echo "  bun run tauri:build"
