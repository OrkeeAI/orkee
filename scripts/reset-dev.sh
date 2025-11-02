#!/bin/bash
# ABOUTME: Development reset script - kills Orkee processes and removes database
# ABOUTME: Use when you need a clean slate for testing

set -e

echo "🔄 Resetting Orkee development environment..."

# Kill all orkee processes
echo "⏹️  Stopping all Orkee processes..."
pkill -f orkee || echo "   No Orkee processes found"

# Wait a moment for processes to fully terminate
sleep 1

# Remove database files
echo "🗑️  Removing database files..."
rm -f ~/.orkee/orkee.db
rm -f ~/.orkee/orkee.db-shm
rm -f ~/.orkee/orkee.db-wal

echo "✅ Reset complete! Database will be recreated on next startup."
