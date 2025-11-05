#!/bin/bash

# Build script for Orkee sandbox Docker image

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building Orkee sandbox Docker image..."

# Build the image
docker build -t orkee/sandbox:latest -f Dockerfile .

# Tag with version if provided
if [ -n "$1" ]; then
    VERSION="$1"
    echo "Tagging image with version $VERSION..."
    docker tag orkee/sandbox:latest "orkee/sandbox:$VERSION"
fi

echo "Build complete!"
echo ""
echo "Available image:"
docker images | grep "orkee/sandbox"

echo ""
echo "To run the sandbox:"
echo "  docker run -it --rm -v \$(pwd):/workspace orkee/sandbox:latest"