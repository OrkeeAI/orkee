#!/bin/bash
# Start Tauri dev with dynamic port handling

# Find an available port for UI (default 5173, fallback to next available)
export ORKEE_UI_PORT=${ORKEE_UI_PORT:-5173}

# Set the Tauri dev URL
export TAURI_DEV_URL="http://localhost:${ORKEE_UI_PORT}"

echo "Starting Orkee Dashboard..."
echo "UI Port: ${ORKEE_UI_PORT}"
echo "Dev URL: ${TAURI_DEV_URL}"

# Start Tauri dev
bun tauri dev
