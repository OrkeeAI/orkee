#!/bin/bash

echo "Testing Orkee Dashboard Development Mode"
echo "========================================="
echo ""
echo "This script demonstrates using the local dashboard during development"
echo ""

# Kill any existing processes on the ports
echo "🧹 Cleaning up any existing processes..."
lsof -ti:4001 | xargs -r kill -9 2>/dev/null || true
lsof -ti:5173 | xargs -r kill -9 2>/dev/null || true

echo ""
echo "📦 Option 1: Using --dev flag"
echo "Run: cargo run --bin orkee -- dashboard --dev"
echo ""
echo "📦 Option 2: Using environment variable"
echo "Run: ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard"
echo ""
echo "📦 Option 3: Production mode (downloads from GitHub)"
echo "Run: cargo run --bin orkee -- dashboard"
echo ""
echo "The --dev flag or ORKEE_DEV_MODE=true will use the local dashboard at:"
echo "  packages/dashboard/"
echo ""
echo "Without --dev, it will use the downloaded dashboard at:"
echo "  ~/.orkee/dashboard/"