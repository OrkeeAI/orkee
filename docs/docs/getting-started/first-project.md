---
sidebar_position: 3
---

# Create Your First Project

This guide walks you through creating and configuring your first AI project with Orkee step-by-step.

## Overview

Projects in Orkee are the primary organizational unit for your AI agents and workflows. Each project includes:

- **Metadata**: Name, description, tags, and priority
- **Location**: File system path and Git repository info  
- **Scripts**: Setup, development, and cleanup commands
- **Status**: Current project state (active, draft, archived)

## Prerequisites

Before creating your first project, ensure you have:

- [Orkee installed](installation) and verified
- A directory for your AI project
- (Optional) Git repository for version control

## Method 1: Interactive CLI Creation

The easiest way to create a project is using the interactive CLI:

```bash
orkee projects add
```

Follow the prompts:

```
? Project name: my-first-agent
? Project path: /Users/yourname/projects/my-first-agent  
? Description: My first AI agent using Orkee
? Tags (comma-separated): ai, agent, tutorial
? Priority: medium
? Status: active
```

## Method 2: Command Line Arguments

For automation or scripting, specify all options directly:

```bash
orkee projects add \
  --name "my-first-agent" \
  --path "/Users/yourname/projects/my-first-agent" \
  --description "My first AI agent using Orkee" \
  --tags "ai,agent,tutorial" \
  --priority medium \
  --status active
```

## Method 3: Web Dashboard

Using the web dashboard provides a visual interface:

1. **Start the dashboard**:
   ```bash
   orkee dashboard
   ```

2. **Open in browser**: Navigate to http://localhost:5173

3. **Add project**: Click "Add New Project" button

4. **Fill the form**:
   - Name: `my-first-agent`
   - Path: Browse or type the project directory
   - Description: Add a meaningful description
   - Tags: Space or comma-separated tags
   - Priority: Select from dropdown
   - Status: Choose project status

5. **Create**: Click "Create Project"

## Project Structure Setup

Once created, set up your project directory structure:

### Basic AI Agent Structure

```bash
mkdir -p my-first-agent/{src,tests,docs,config}
cd my-first-agent

# Create basic files
touch README.md
touch src/agent.py
touch requirements.txt
touch .env.example
```

### Example Project Files

**`README.md`**:
```markdown
# My First Agent

This is my first AI agent built with Orkee.

## Setup

```bash
pip install -r requirements.txt
```

## Usage

```bash
python src/agent.py
```
```

**`src/agent.py`**:
```python
#!/usr/bin/env python3
"""
My First AI Agent
"""

class MyFirstAgent:
    def __init__(self):
        self.name = "MyFirstAgent"
    
    def run(self):
        print(f"Hello from {self.name}!")
        
if __name__ == "__main__":
    agent = MyFirstAgent()
    agent.run()
```

**`requirements.txt`**:
```
openai>=1.0.0
python-dotenv>=1.0.0
```

## Configure Project Scripts

Add custom scripts to automate common tasks:

### Using CLI

```bash
# Edit project to add scripts
orkee projects edit <project-id>
```

When prompted, set:
- **Setup Script**: `pip install -r requirements.txt`
- **Dev Script**: `python src/agent.py`
- **Cleanup Script**: `rm -rf __pycache__ *.pyc`

### Using Dashboard

1. Open project details in the dashboard
2. Click "Edit Project"  
3. Add scripts in the respective fields
4. Save changes

## Git Repository Integration

If your project is in a Git repository, Orkee automatically detects and stores repository information:

```bash
cd my-first-agent
git init
git remote add origin https://github.com/yourusername/my-first-agent.git
git add .
git commit -m "Initial commit"
git push -u origin main
```

Orkee will automatically populate:
- Repository URL
- Current branch
- Owner/repository name

## Verify Your Project

Check that your project was created successfully:

### List Projects

```bash
orkee projects list
```

Output:
```
┌──────────┬─────────────────┬────────────────────────────┬──────────┬────────────┐
│ ID       │ Name            │ Path                       │ Status   │ Priority   │
├──────────┼─────────────────┼────────────────────────────┼──────────┼────────────┤
│ abc123   │ my-first-agent  │ /Users/.../my-first-agent  │ active   │ medium     │
└──────────┴─────────────────┴────────────────────────────┴──────────┴────────────┘
```

### Show Project Details

```bash
orkee projects show abc123
```

Output:
```yaml
Project Details:
  ID: abc123
  Name: my-first-agent
  Description: My first AI agent using Orkee
  Path: /Users/yourname/projects/my-first-agent
  Status: active
  Priority: medium
  Tags: [ai, agent, tutorial]
  
Git Repository:
  URL: https://github.com/yourusername/my-first-agent.git
  Branch: main
  Owner: yourusername
  
Scripts:
  Setup: pip install -r requirements.txt
  Dev: python src/agent.py
  Cleanup: rm -rf __pycache__ *.pyc
  
Created: 2024-01-15 10:30:00 UTC
Updated: 2024-01-15 10:30:00 UTC
```

## Working with Your Project

### Running Scripts

Execute project scripts via Orkee:

```bash
# Run setup script
orkee projects run-script abc123 setup

# Run development script  
orkee projects run-script abc123 dev

# Run cleanup script
orkee projects run-script abc123 cleanup
```

### Updating Project

Modify project details as needed:

```bash
# Edit interactively
orkee projects edit abc123

# Update specific fields
orkee projects update abc123 --description "Updated description"
orkee projects update abc123 --priority high
orkee projects update abc123 --status archived
```

### Directory Navigation

Browse project files using the directory API:

```bash
curl http://localhost:4001/api/browse-directories \
  -H "Content-Type: application/json" \
  -d '{"path": "/Users/yourname/projects/my-first-agent"}'
```

## Best Practices

### Naming Conventions

- Use descriptive, lowercase names with hyphens
- Include project type in the name (e.g., `chatbot-agent`, `data-processor`)
- Keep names under 50 characters

### Organization

- **Tags**: Use consistent tagging scheme (`ai`, `agent`, `production`, `experimental`)
- **Descriptions**: Write clear, concise descriptions
- **Paths**: Use absolute paths to avoid confusion
- **Scripts**: Add setup, development, and cleanup scripts

### Security

- Never store sensitive data in project descriptions or scripts
- Use `.env` files for environment variables
- Add `.env` to `.gitignore`
- Use Orkee's sandbox mode for directory browsing security

## Troubleshooting

### Project Creation Fails

**Issue**: Permission denied when creating project

**Solution**:
```bash
# Check directory permissions
ls -la /path/to/parent/directory

# Fix permissions
chmod 755 /path/to/parent/directory
```

### Git Detection Issues

**Issue**: Git repository not detected

**Solution**:
```bash
# Verify git repository
cd /path/to/project
git remote -v

# Re-add remote if needed  
git remote add origin https://github.com/user/repo.git
```

### Script Execution Fails

**Issue**: Project scripts fail to execute

**Solution**:
```bash
# Test scripts manually first
cd /path/to/project
pip install -r requirements.txt

# Check script permissions
ls -la setup.sh
chmod +x setup.sh
```

## Next Steps

With your first project created:

1. **[Explore User Interfaces](../user-guide/cli-commands)** - Learn CLI, TUI, and Dashboard features
2. **[Configure Environment](../configuration/environment-variables)** - Customize Orkee settings  
3. **[Set Up Cloud Sync](../cloud/getting-started)** - Enable backup and collaboration
4. **[Deploy to Production](../deployment/docker)** - Scale your setup

## Advanced Project Features

### Project Templates

Create project templates for consistent setup:

```bash
# Create template project
orkee projects add-template \
  --name "ai-agent-template" \
  --structure "src/,tests/,docs/,config/" \
  --scripts "pip install -r requirements.txt"
```

### Bulk Operations

Manage multiple projects efficiently:

```bash
# List projects by tag
orkee projects list --tag ai

# Archive all draft projects
orkee projects bulk-update --status draft --set-status archived

# Export project configurations
orkee projects export --format json > projects-backup.json
```

### Integration with IDEs

Configure your IDE to work with Orkee projects:

```bash
# Generate VS Code workspace
orkee projects generate-workspace abc123

# Open in preferred editor
orkee projects open abc123 --editor vscode
```

Congratulations! You've successfully created and configured your first Orkee project. You're now ready to build and manage AI agents with Orkee's powerful project management features.