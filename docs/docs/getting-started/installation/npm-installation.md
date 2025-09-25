---
sidebar_position: 1
title: NPM Installation
---

# NPM Installation

The easiest and recommended way to install Orkee is through npm.

## Quick Installation

```bash
# Install globally
npm install -g orkee

# Verify installation
orkee --version
```

## Benefits

- **Automatic updates**: Easy to keep up to date
- **Platform detection**: Automatically downloads the right binary
- **No manual setup**: Works out of the box
- **Version management**: Easy to switch between versions

## Requirements

- **Node.js**: v18 or later
- **npm**: v8 or later (comes with Node.js)

## Installation Steps

### 1. Install Node.js

If you don't have Node.js installed:

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="official" label="Official Installer" default>

Download from [nodejs.org](https://nodejs.org/) and follow the installation wizard.

</TabItem>
<TabItem value="nvm" label="Node Version Manager">

```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install latest Node.js
nvm install node
nvm use node
```

</TabItem>
<TabItem value="package-manager" label="Package Manager">

```bash
# macOS with Homebrew
brew install node

# Ubuntu/Debian
sudo apt update && sudo apt install nodejs npm

# Windows with Chocolatey
choco install nodejs
```

</TabItem>
</Tabs>

### 2. Install Orkee

```bash
npm install -g orkee
```

### 3. Verify Installation

```bash
orkee --version
orkee --help
```

## Troubleshooting

### Permission Errors

If you get permission errors on macOS/Linux:

```bash
# Option 1: Use a Node version manager (recommended)
npm install -g orkee --unsafe-perm

# Option 2: Change npm's default directory
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

### Command Not Found

If `orkee` command is not found:

1. Check npm global bin directory:
   ```bash
   npm config get prefix
   ```

2. Add to PATH in your shell profile (`.bashrc`, `.zshrc`, etc.):
   ```bash
   export PATH="$(npm config get prefix)/bin:$PATH"
   ```

### Update Orkee

Keep Orkee up to date:

```bash
# Check for updates
npm outdated -g orkee

# Update to latest version
npm update -g orkee

# Install specific version
npm install -g orkee@1.0.0
```

## Local Installation

For project-specific installation:

```bash
# Install in current project
npm install orkee

# Run with npx
npx orkee --help

# Add to package.json scripts
# "scripts": { "orkee": "orkee" }
npm run orkee -- --help
```

## Next Steps

- [Quick Start Guide](../quick-start) - Get up and running
- [Troubleshooting](../troubleshooting) - Common issues
- [Configuration](../../configuration/environment-variables) - Customize your setup