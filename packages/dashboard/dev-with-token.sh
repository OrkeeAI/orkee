#!/bin/bash
# Helper script to run the web dashboard with API token authentication
# This allows the web dashboard to connect to a Tauri-spawned CLI server

# Read the API token from ~/.orkee/api-token
TOKEN_FILE="$HOME/.orkee/api-token"

if [ ! -f "$TOKEN_FILE" ]; then
    echo "❌ Error: API token file not found at $TOKEN_FILE"
    echo "Please start the Orkee CLI server first to generate the token."
    exit 1
fi

TOKEN=$(cat "$TOKEN_FILE")

if [ -z "$TOKEN" ]; then
    echo "❌ Error: API token file is empty"
    exit 1
fi

echo "✓ Using API token from $TOKEN_FILE"
echo "✓ Starting web dashboard with authentication..."

VITE_ORKEE_API_TOKEN="$TOKEN" bun run dev
