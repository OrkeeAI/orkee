---
sidebar_position: 3
title: First Run Guide
---

# First Run Guide

What to expect when running Orkee for the first time and how to get started quickly.

## Before You Start

### Prerequisites

Make sure you have Orkee installed:

```bash
# Verify installation
orkee --version

# Should output something like:
# orkee 0.1.0
```

If not installed, see the [Installation Guide](installation).

### Understanding Orkee's Architecture

Orkee consists of three main interfaces:

1. **Dashboard** (Web UI) - Visual interface for project management
2. **TUI** (Terminal UI) - Keyboard-driven terminal interface
3. **CLI** (Command Line) - Direct command-line project management

All interfaces share the same local SQLite database: `~/.orkee/orkee.db`

## First Launch

### Option 1: Dashboard (Recommended)

The dashboard provides the most user-friendly interface for first-time users.

```bash
# Start the dashboard
orkee dashboard
```

**What happens:**

1. **Database Initialization** (first run only)
   ```
   Initializing Orkee database at ~/.orkee/orkee.db...
   ✓ Database created successfully
   ✓ Schema migrations applied
   ✓ Full-text search enabled
   ```

2. **Server Startup**
   ```
   Starting Orkee API server on port 4001...
   ✓ API server running at http://localhost:4001

   Starting Orkee Dashboard on port 5173...
   ✓ Dashboard running at http://localhost:5173

   Opening browser...
   ```

3. **Browser Opens Automatically**
   - Dashboard loads at `http://localhost:5173`
   - Initial empty state with "Get Started" prompts
   - Health indicator shows green (connected)

### Option 2: TUI (Terminal Interface)

For terminal enthusiasts or headless environments:

```bash
# Start the TUI
orkee tui
```

**What happens:**

1. Database initializes (if first run)
2. Terminal interface appears with keyboard controls
3. Shows empty project list with help text

**Key bindings:**
- `↑/↓` or `j/k` - Navigate
- `n` - New project
- `Enter` - View details
- `q` - Quit

### Option 3: CLI Commands

For direct command-line management:

```bash
# List projects (will be empty)
orkee projects list

# Add your first project
orkee projects add
```

## Initial Setup

### 1. Configure Your Environment (Optional)

Orkee works out of the box, but you can customize:

```bash
# Create configuration directory
mkdir -p ~/.orkee

# Create environment file (optional)
cat > ~/.orkee/.env << 'EOF'
# Server Ports
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173

# Security
RATE_LIMIT_ENABLED=true
BROWSE_SANDBOX_MODE=relaxed
ALLOWED_BROWSE_PATHS=~/Documents,~/Projects,~/Code

# Logging
RUST_LOG=info
EOF
```

### 2. Add Your First Project

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="dashboard" label="Via Dashboard" default>

1. Click **"Add Project"** button or the **"+"** icon
2. Fill in the form:
   - **Name**: Project display name (e.g., "My App")
   - **Path**: Absolute path to project directory
   - **Description**: Brief description (optional)
   - **Tags**: Comma-separated tags (optional)
3. Click **"Create Project"**
4. Project appears in the list!

**Tip**: Click the folder icon next to the path field to browse directories.

</TabItem>
<TabItem value="tui" label="Via TUI">

1. Press `n` to create new project
2. Enter project details when prompted:
   ```
   Name: My App
   Path: /Users/yourname/projects/my-app
   Description: React application with TypeScript
   Tags: react,typescript,frontend
   ```
3. Press Enter to confirm
4. Project appears in the list!

**Tip**: Press `Tab` to auto-complete paths.

</TabItem>
<TabItem value="cli" label="Via CLI">

**Interactive mode:**
```bash
orkee projects add
# Follow the prompts
```

**Direct mode:**
```bash
orkee projects add \
  --name "My App" \
  --path ~/projects/my-app \
  --description "React application with TypeScript"
```

**Verify:**
```bash
orkee projects list
```

</TabItem>
</Tabs>

### 3. Understand Project Status

Orkee tracks projects through six status stages:

| Status | Description | When to Use |
|--------|-------------|-------------|
| **Planning** | Initial planning phase | New ideas, not started yet |
| **Building** | Active development | Currently working on it |
| **Review** | Ready for review/testing | Code complete, needs review |
| **Launched** | Live in production | Deployed and running |
| **On Hold** | Temporarily paused | Waiting on dependencies |
| **Archived** | Completed or cancelled | No longer active |

**Change status:**
- Dashboard: Click project → Edit → Select status
- TUI: Select project → Press `e` → Choose status
- CLI: `orkee projects edit <id>` → Select status

### 4. Explore Git Integration (Automatic)

If your project is in a Git repository, Orkee automatically detects:

- Current branch
- Remote origin
- Repository URL

**View Git info:**
- Dashboard: Project details → Git section
- TUI: Select project → Git info displayed
- CLI: `orkee projects show <id>`

## Understanding the Interface

### Dashboard Layout

```
┌─────────────────────────────────────────────────┐
│  Orkee Dashboard                        [User]   │
├─────────────────────────────────────────────────┤
│                                                  │
│  [Projects]  [AI Chat]  [Servers]  [Settings]   │
│                                                  │
│  ┌──────────────────────────────────────┐       │
│  │ Projects (3)                    [+]  │       │
│  ├──────────────────────────────────────┤       │
│  │ My App              Building   🟢   │       │
│  │ API Service         Launched   🟢   │       │
│  │ Website             Planning   🟡   │       │
│  └──────────────────────────────────────┘       │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Navigation:**
- **Projects**: Main project list and management
- **AI Chat**: AI-powered project assistance (coming soon)
- **Servers**: Preview server management (coming soon)
- **Settings**: Configuration and preferences

**Status Indicators:**
- 🟢 Green: Healthy/Active
- 🟡 Yellow: Warning/On Hold
- 🔴 Red: Error/Archived

### TUI Layout

```
┌────────────────────────────────────────────────┐
│  Orkee - Press ? for help         [q] Quit    │
├────────────────────────────────────────────────┤
│                                                 │
│  Projects (3)                                   │
│  ┌──────────────────────────────────────────┐  │
│  │ ▶ My App              [Building]         │  │
│  │   API Service         [Launched]         │  │
│  │   Website             [Planning]         │  │
│  └──────────────────────────────────────────┘  │
│                                                 │
│  [n] New  [e] Edit  [d] Delete  [r] Refresh    │
│                                                 │
└────────────────────────────────────────────────┘
```

**Key Commands:**
- `?` - Help
- `↑/↓` - Navigate
- `n` - New project
- `e` - Edit project
- `d` - Delete project
- `r` - Refresh
- `q` - Quit

## Common First-Run Tasks

### Importing Existing Projects

Quickly add all projects from a directory:

```bash
# Add all subdirectories as projects
for dir in ~/projects/*/; do
  orkee projects add --path "$dir" --name "$(basename "$dir")"
done

# Or use a script
find ~/projects -maxdepth 1 -type d -not -path "*/\.*" | while read dir; do
  [ "$dir" != "$HOME/projects" ] && \
  orkee projects add --path "$dir" --name "$(basename "$dir")"
done
```

### Organizing with Tags

Use tags to categorize projects:

```bash
# Add tags during creation
orkee projects add --name "My App" --path ~/projects/my-app

# Then add tags via edit
orkee projects edit 1
# Add tags: react,frontend,active
```

**Suggested tag categories:**
- **Language/Framework**: `rust`, `react`, `python`, `nodejs`
- **Type**: `frontend`, `backend`, `api`, `cli`, `library`
- **Client**: `acme-corp`, `personal`, `opensource`
- **Priority**: `urgent`, `low-priority`, `experimental`
- **Status**: `active`, `maintenance`, `legacy`

### Configuring Security Settings

For production or team environments:

```bash
# Set strict sandbox mode
export BROWSE_SANDBOX_MODE=strict
export ALLOWED_BROWSE_PATHS=/var/projects,/opt/projects

# Enable rate limiting
export RATE_LIMIT_ENABLED=true
export RATE_LIMIT_GLOBAL_RPM=30

# Enable security headers
export SECURITY_HEADERS_ENABLED=true

# Restart with new settings
orkee dashboard
```

See [Security Settings](../configuration/security-settings) for details.

## Next Steps

### Basic Workflow

1. **Add projects** to Orkee
2. **Update status** as work progresses
3. **Use tags** for organization
4. **Launch dashboard** when working
5. **View at a glance** what you're working on

### Recommended Configuration

```bash
# Add to ~/.bashrc or ~/.zshrc
export ORKEE_API_PORT=4001
export ORKEE_UI_PORT=5173
export RUST_LOG=info

# Alias for quick launch
alias orkee-start='orkee dashboard'
alias orkee-stop='pkill orkee'
```

### Enable Cloud Sync (Optional)

Backup and sync projects to Orkee Cloud:

```bash
# Sign up at https://cloud.orkee.ai
# Then authenticate
orkee cloud login

# Enable sync
orkee cloud enable

# Verify
orkee cloud status
```

See [Cloud Sync Guide](../cloud/getting-started) for details.

### Run as a Service

For permanent installation:

**Linux (systemd):**
```bash
# Copy service file
sudo cp deployment/systemd/orkee.service /etc/systemd/system/

# Enable and start
sudo systemctl enable orkee
sudo systemctl start orkee
```

**macOS (launchd):**
```bash
# Copy plist file
cp deployment/launchd/com.orkee.dashboard.plist ~/Library/LaunchAgents/

# Load service
launchctl load ~/Library/LaunchAgents/com.orkee.dashboard.plist
```

See [Deployment Guide](../deployment/linux-server) for details.

## Verification Checklist

After first run, verify everything works:

- [ ] Dashboard opens in browser
- [ ] API health check passes: `curl http://localhost:4001/api/health`
- [ ] Can create a project
- [ ] Can edit project details
- [ ] Can delete a project
- [ ] Projects persist after restart
- [ ] TUI launches successfully
- [ ] CLI commands work

## Understanding Data Storage

### Database Location

```bash
# Default database location
~/.orkee/orkee.db

# View database size
ls -lh ~/.orkee/orkee.db

# Backup database
cp ~/.orkee/orkee.db ~/.orkee/orkee.db.backup
```

### SQLite Database

Orkee uses SQLite for local-first data storage:

```bash
# Query database directly (advanced)
sqlite3 ~/.orkee/orkee.db "SELECT * FROM projects;"

# Database info
sqlite3 ~/.orkee/orkee.db ".schema projects"

# Enable WAL mode (better performance)
sqlite3 ~/.orkee/orkee.db "PRAGMA journal_mode=WAL;"
```

### Directory Structure

```
~/.orkee/
├── orkee.db              # Main database
├── orkee.db-wal          # Write-ahead log
├── orkee.db-shm          # Shared memory
├── auth.toml             # Cloud authentication (if enabled)
├── certs/                # TLS certificates (if configured)
│   ├── cert.pem
│   └── key.pem
└── logs/                 # Application logs
    └── orkee.log
```

## Migration from Projects.json

If you previously used Orkee's JSON-based storage:

```bash
# Orkee automatically migrates on first run
# Old file: ~/.orkee/projects.json
# New database: ~/.orkee/orkee.db

# Verify migration
orkee projects list

# Backup old file (optional)
cp ~/.orkee/projects.json ~/.orkee/projects.json.backup
```

## Getting Help

### Built-in Help

```bash
# General help
orkee --help

# Command-specific help
orkee dashboard --help
orkee projects --help
orkee projects add --help

# Version information
orkee --version
```

### Documentation

- [CLI Commands](../user-guide/cli-commands) - Complete command reference
- [Configuration Guide](../configuration/environment-variables) - All settings
- [Troubleshooting](troubleshooting) - Common issues
- [API Reference](../api-reference/overview) - REST API docs

### Community Support

- **GitHub Discussions**: https://github.com/OrkeeAI/orkee/discussions
- **GitHub Issues**: https://github.com/OrkeeAI/orkee/issues
- **Email**: support@orkee.ai

## Quick Reference

### Essential Commands

```bash
# Start dashboard
orkee dashboard

# Start TUI
orkee tui

# List projects
orkee projects list

# Add project
orkee projects add

# View project
orkee projects show <id>

# Edit project
orkee projects edit <id>

# Delete project
orkee projects delete <id>

# Cloud status
orkee cloud status

# Check health
curl http://localhost:4001/api/health
```

### Essential Paths

```bash
~/.orkee/orkee.db          # Database
~/.orkee/.env              # Configuration
~/.orkee/auth.toml         # Cloud auth
/usr/local/bin/orkee       # Binary location
```

### Essential Environment Variables

```bash
ORKEE_API_PORT=4001        # API port
ORKEE_UI_PORT=5173         # Dashboard port
RUST_LOG=info              # Log level
BROWSE_SANDBOX_MODE=relaxed # Security mode
```

## What's Next?

Now that you're up and running:

1. **Explore the Dashboard** - Click around and try features
2. **Add Your Projects** - Import your existing work
3. **Customize Settings** - Adjust to your workflow
4. **Learn CLI Commands** - Increase productivity
5. **Enable Cloud Sync** - Backup your data
6. **Join Community** - Get help and share feedback

Welcome to Orkee! 🎉
