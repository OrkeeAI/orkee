#!/bin/bash
set -e

# ABOUTME: Builds CLI binary and copies it to Tauri binaries directory for bundling
# ABOUTME: This ensures the Tauri app has the correct CLI binary for the current platform

echo "Preparing CLI binary for Tauri..."

# Detect current platform
OS=$(uname -s)
ARCH=$(uname -m)

# Determine target triple and binary extension
BINARY_EXT=""
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
        echo "Unsupported Linux architecture: $ARCH"
        exit 1
    fi
elif [[ "$OS" =~ ^(MINGW|MSYS|CYGWIN) ]]; then
    TARGET="x86_64-pc-windows-msvc"
    BINARY_EXT=".exe"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

echo "Building for target: $TARGET"

# Build CLI binary
cd "$(dirname "$0")/../cli"
cargo build --release --target "$TARGET"

# Create binaries directory if it doesn't exist
BINARIES_DIR="../dashboard/src-tauri/binaries"
mkdir -p "$BINARIES_DIR"

# Copy binary to Tauri binaries directory with platform-specific name
BINARY_NAME="orkee-$TARGET"
cp "target/$TARGET/release/orkee$BINARY_EXT" "$BINARIES_DIR/$BINARY_NAME$BINARY_EXT"

echo "✓ Binary prepared: $BINARIES_DIR/$BINARY_NAME"
echo ""
echo "You can now build the Tauri app with:"
echo "  cd packages/dashboard"
echo "  bun run tauri:build"
