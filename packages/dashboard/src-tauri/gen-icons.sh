#!/bin/bash
# Icon generation script for Tauri
# 
# Usage:
#   1. Create or obtain an Orkee logo PNG (1024x1024 recommended)
#   2. Run: pnpm tauri icon path/to/your-icon.png
#
# This will automatically generate all required icon sizes for:
#   - macOS (.icns)
#   - Windows (.ico)
#   - Linux (.png in various sizes)
#
# For now, we'll use a simple colored square as a placeholder
# TODO: Replace with actual Orkee logo

echo "To generate proper icons, run:"
echo "  pnpm tauri icon path/to/logo.png"
echo ""
echo "The icon should be:"
echo "  - Square (1:1 aspect ratio)"
echo "  - At least 1024x1024 pixels"
echo "  - PNG format with transparency"
