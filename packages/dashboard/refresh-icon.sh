#!/bin/bash
# Script to clear macOS icon cache and refresh app icon

echo "Clearing icon caches..."

# Clear system icon cache (requires sudo)
sudo rm -rf /Library/Caches/com.apple.iconservices.store

# Clear user icon cache
rm -rf ~/Library/Caches/com.apple.iconservices

# Kill and restart services
killall Dock
killall Finder

echo "Icon caches cleared! Dock and Finder restarted."
echo "Now run: bun tauri:dev"
