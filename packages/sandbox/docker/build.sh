#!/bin/bash

# Build script for Orkee sandbox Docker images

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building Orkee sandbox Docker images..."

# Build base image
echo "Building base image..."
docker build -t orkee/sandbox:base -f Dockerfile.base .

# Build agent-specific images
echo "Building Claude Code image..."
docker build -t orkee/sandbox:claude-code -f Dockerfile.claude-code .

echo "Building Aider image..."
docker build -t orkee/sandbox:aider -f Dockerfile.aider .

# Tag images with version if provided
if [ -n "$1" ]; then
    VERSION="$1"
    echo "Tagging images with version $VERSION..."

    docker tag orkee/sandbox:base "orkee/sandbox:base-$VERSION"
    docker tag orkee/sandbox:claude-code "orkee/sandbox:claude-code-$VERSION"
    docker tag orkee/sandbox:aider "orkee/sandbox:aider-$VERSION"
fi

echo "Build complete!"
echo ""
echo "Available images:"
docker images | grep "orkee/sandbox"

echo ""
echo "To run a sandbox:"
echo "  docker run -it --rm -v \$(pwd):/workspace orkee/sandbox:claude-code"
echo "  docker run -it --rm -v \$(pwd):/workspace orkee/sandbox:aider"
echo ""
echo "Or use docker-compose:"
echo "  docker-compose up claude-code"
echo "  docker-compose up aider"