#!/usr/bin/env bash
# ABOUTME: Builds CLI binary and copies it to Tauri binaries directory for bundling
# ABOUTME: This ensures the Tauri app has the correct CLI binary for the current platform

set -euo pipefail

# Enable debug mode if DEBUG env var is set
if [ "${DEBUG:-}" = "1" ]; then
    set -x
fi

echo "Preparing CLI binary for Tauri..."

# Verify cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found in PATH"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✓ Cargo found: $(cargo --version)"

# Allow target override via environment variable (for CI cross-compilation)
if [ -n "${TAURI_TARGET:-}" ]; then
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

# Verify Rust toolchain has the target installed
echo "Verifying Rust toolchain has target $TARGET..."
if ! rustup target list --installed | grep -q "^${TARGET}$"; then
    echo "Error: Rust target $TARGET is not installed"
    echo ""
    echo "Install the target with:"
    echo "  rustup target add $TARGET"
    echo ""
    echo "Or install all common targets:"
    echo "  rustup target add aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu x86_64-pc-windows-msvc"
    exit 1
fi
echo "✓ Target $TARGET is installed"

# Build CLI binary
cd "$(dirname "$0")/../cli"

# Use release-ci profile in CI, regular release profile otherwise
BUILD_PROFILE="${CARGO_BUILD_PROFILE:-release}"
echo "Running: cargo build --profile $BUILD_PROFILE --target $TARGET"
if ! cargo build --profile "$BUILD_PROFILE" --target "$TARGET" 2>&1; then
    echo ""
    echo "=========================================="
    echo "Error: Cargo build failed for target $TARGET"
    echo "=========================================="
    echo ""
    echo "Common causes:"
    echo "  1. Missing system dependencies (e.g., libssl-dev, pkg-config)"
    echo "  2. Compiler not available for target platform"
    echo "  3. Cargo.toml dependencies incompatible with target"
    echo "  4. Insufficient disk space or memory"
    echo ""
    echo "Troubleshooting:"
    echo "  - Check cargo output above for specific error messages"
    echo "  - Verify system dependencies: https://www.rust-lang.org/tools/install"
    echo "  - Try building without --release flag for more detailed errors"
    echo "  - Check available targets: rustup target list --installed"
    echo ""
    exit 1
fi

# Verify binary was created
# Note: Cargo uses the profile name as the directory name, except "release" stays as "release"
if [ "$BUILD_PROFILE" = "release-ci" ]; then
    PROFILE_DIR="release-ci"
else
    PROFILE_DIR="release"
fi
BINARY_PATH="../../target/$TARGET/$PROFILE_DIR/orkee$BINARY_EXT"
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Build may have failed. Check cargo output above."
    exit 1
fi

echo "✓ Binary built successfully: $BINARY_PATH"

# Verify binary is executable and functional (smoke test)
echo "Running smoke test..."
if [ -x "$BINARY_PATH" ]; then
    echo "✓ Binary is executable"

    # Try to run --version (should work without any server setup)
    if "$BINARY_PATH" --version > /dev/null 2>&1; then
        VERSION=$("$BINARY_PATH" --version 2>/dev/null || echo "unknown")
        echo "✓ Binary smoke test passed: $VERSION"
    else
        echo "Warning: Binary executes but --version failed"
        echo "Binary may not be fully functional, but continuing..."
    fi
else
    echo "Error: Binary exists but is not executable"
    echo "Binary path: $BINARY_PATH"
    echo "Permissions: $(ls -l "$BINARY_PATH" 2>/dev/null || echo 'cannot read')"
    exit 1
fi

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
