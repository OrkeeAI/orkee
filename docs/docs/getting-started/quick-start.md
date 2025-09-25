---
sidebar_position: 2
---

# Quick Start

Get up and running with Orkee in under 5 minutes! This guide will walk you through the basics of creating and managing your first AI project.

## 1. Installation

First, install Orkee globally using npm:

```bash
npm install -g orkee
```

Verify the installation:

```bash
orkee --version
```

## 2. Choose Your Interface

Orkee offers three interfaces to work with:

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="dashboard" label="🌐 Web Dashboard" default>

Launch the web dashboard for a visual interface:

```bash
orkee dashboard
```

This starts:
- **API Server**: http://localhost:4001
- **Web Dashboard**: http://localhost:5173

The dashboard provides a rich web interface for managing projects, monitoring system health, and configuring settings.

</TabItem>
<TabItem value="tui" label="💻 Terminal Interface">

Launch the Terminal User Interface (TUI) for keyboard-driven workflows:

```bash
orkee tui
```

The TUI provides:
- Interactive project management
- Real-time system monitoring  
- Keyboard shortcuts for fast navigation
- No web browser required

</TabItem>
<TabItem value="cli" label="⌨️ Command Line">

Use CLI commands for automation and scripting:

```bash
# List projects
orkee projects list

# Create a project
orkee projects add

# Show project details
orkee projects show <project-id>
```

Perfect for CI/CD pipelines and automation scripts.

</TabItem>
</Tabs>

## 3. Create Your First Project

Let's create your first AI project:

<Tabs>
<TabItem value="interactive" label="Interactive Mode" default>

```bash
orkee projects add
```

Follow the interactive prompts:
- **Project Name**: `my-ai-agent`
- **Project Path**: `/path/to/your/project` 
- **Description**: `My first AI agent project`
- **Tags**: `ai, agent, demo` (optional)

</TabItem>
<TabItem value="command" label="Command Mode">

```bash
orkee projects add \
  --name "my-ai-agent" \
  --path "/path/to/your/project" \
  --description "My first AI agent project"
```

</TabItem>
<TabItem value="dashboard" label="Web Dashboard">

1. Open http://localhost:5173 in your browser
2. Click **"Add New Project"**
3. Fill in the project details
4. Click **"Create Project"**

</TabItem>
</Tabs>

## 4. Explore Project Management

Once created, explore your project:

### View Project Details

```bash
orkee projects show <project-id>
```

### List All Projects

```bash
orkee projects list
```

### Edit Project

```bash
orkee projects edit <project-id>
```

## 5. Configure Orkee (Optional)

### Environment Variables

Create a `.env` file to customize Orkee:

```bash
# Server Configuration
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173

# Security
BROWSE_SANDBOX_MODE=relaxed
RATE_LIMIT_ENABLED=true

# HTTPS (optional)
TLS_ENABLED=false
```

### Cloud Sync (Optional)

Enable cloud synchronization:

```bash
# Build with cloud features
cargo build --features cloud

# Authenticate with Orkee Cloud
orkee cloud login

# Sync projects
orkee cloud sync
```

:::info
Cloud sync requires an [Orkee Cloud account](https://orkee.ai). Sign up to enable backup and multi-device synchronization.
:::

## 6. Explore Advanced Features

### Project Scripts

Add custom scripts to your projects:

```bash
# Add setup script
orkee projects edit <project-id>
# Set setupScript: "npm install && pip install -r requirements.txt"
# Set devScript: "npm run dev"
```

### Directory Browsing

Browse project directories through the API:

```bash
curl http://localhost:4001/api/browse-directories \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/your/project"}'
```

### Health Monitoring

Check system health:

```bash
curl http://localhost:4001/api/health
curl http://localhost:4001/api/status
```

## Common Workflows

### Development Workflow

```bash
# 1. Start the dashboard
orkee dashboard

# 2. Create a new project
orkee projects add --name "new-agent" --path "."

# 3. Open in browser
open http://localhost:5173

# 4. Start developing your AI agent
```

### Team Collaboration

```bash
# 1. Enable cloud sync
orkee cloud login

# 2. Create shared project
orkee projects add --name "team-agent"

# 3. Sync to cloud
orkee cloud sync --project <project-id>

# Team members can then restore:
orkee cloud restore --project <project-id>
```

### CI/CD Integration

```bash
#!/bin/bash
# .github/workflows/deploy.yml

# Install Orkee
npm install -g orkee

# Create deployment project
orkee projects add \
  --name "production-agent" \
  --path "$GITHUB_WORKSPACE" \
  --description "Production deployment"

# Run deployment scripts
orkee projects show <project-id>
```

## Next Steps

Now that you have Orkee running:

1. **[Create Your First Project](first-project)** - Detailed project creation guide
2. **[User Guide](../user-guide/cli-commands)** - Complete command reference
3. **[Configuration](../configuration/environment-variables)** - Customize your setup
4. **[Security Guide](../security/overview)** - Secure your deployment

## Troubleshooting

### Port Conflicts

If ports 4001 or 5173 are in use:

```bash
# Use custom ports
orkee dashboard --api-port 8080 --ui-port 3000

# Or set environment variables
ORKEE_API_PORT=8080 ORKEE_UI_PORT=3000 orkee dashboard
```

### Permission Issues

If you encounter permission errors:

```bash
# Check Orkee directory permissions
ls -la ~/.orkee/

# Fix permissions
chmod 755 ~/.orkee/
chmod 644 ~/.orkee/orkee.db
```

### Getting Help

- [Documentation](https://docs.orkee.ai)
- [GitHub Issues](https://github.com/OrkeeAI/orkee/issues)  
- [Community Discussions](https://github.com/OrkeeAI/orkee/discussions)

:::tip Success!
You now have Orkee running! The web dashboard provides the most user-friendly experience for managing projects, while the TUI and CLI are perfect for terminal-based workflows.
:::