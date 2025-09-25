#!/bin/bash
set -e

# Script to build Orkee for all platforms
echo "Building Orkee for all platforms..."

# Ensure we're in the CLI package directory
cd "$(dirname "$0")/../packages/cli"

# Create dist directory at project root
DIST_DIR="../../dist"
mkdir -p "$DIST_DIR"

# Build for each platform
platforms=(
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-pc-windows-msvc"
)

for target in "${platforms[@]}"; do
  echo "Building for $target..."
  
  # Check if target is installed
  if ! rustup target list --installed | grep -q "$target"; then
    echo "Installing target $target..."
    rustup target add "$target"
  fi
  
  # Set OpenSSL env vars for macOS x86_64
  if [[ "$target" == "x86_64-apple-darwin" ]]; then
    export OPENSSL_DIR=$(brew --prefix openssl@3)
    export PKG_CONFIG_PATH=$OPENSSL_DIR/lib/pkgconfig
  fi
  
  # Determine build command based on platform
  BUILD_CMD="cargo"
  
  # Use cross for Linux and Windows targets (cross-compilation)
  if [[ "$target" == *"linux"* ]] || [[ "$target" == *"windows"* ]]; then
    if command -v cross &> /dev/null; then
      BUILD_CMD="cross"
    else
      echo "Installing cross for $target..."
      cargo install cross --locked
      BUILD_CMD="cross"
    fi
  fi
  
  # Build
  echo "Building with $BUILD_CMD..."
  if $BUILD_CMD build --release --target "$target"; then
    # Copy binary to dist
    if [[ "$target" == *"windows"* ]]; then
      cp "target/$target/release/orkee.exe" "$DIST_DIR/orkee-$target.exe" || true
    else
      cp "target/$target/release/orkee" "$DIST_DIR/orkee-$target" || true
    fi
    echo "✓ Built $target with $BUILD_CMD"
  else
    echo "✗ Failed to build $target with $BUILD_CMD"
  fi
done

# Create tarballs for each platform
cd "$DIST_DIR"
for file in orkee-*; do
  if [[ -f "$file" && "$file" != *.tar.gz && "$file" != *.zip ]]; then
    if [[ "$file" == *.exe ]]; then
      # Create zip for Windows
      zip "${file%.exe}.zip" "$file"
      echo "Created ${file%.exe}.zip"
    else
      # Create tar.gz for Unix
      tar -czf "$file.tar.gz" "$file"
      echo "Created $file.tar.gz"
    fi
  fi
done

echo "Build complete! Binaries are in $DIST_DIR"
ls -la "$DIST_DIR"