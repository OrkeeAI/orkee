# npm Installation

The fastest way to install Orkee is through npm. This method automatically downloads the appropriate binary for your platform and sets up global CLI access.

## Prerequisites

- **Node.js**: Version 16 or higher
- **npm**: Version 7 or higher (or pnpm/yarn equivalent)
- **Platform**: Windows, macOS, or Linux (x64/arm64)

## Installation

### Global Installation (Recommended)

Install Orkee globally to use it from anywhere:

```bash
# Using npm
npm install -g orkee

# Using pnpm
pnpm add -g orkee

# Using yarn
yarn global add orkee
```

### Local Installation

Install in a specific project:

```bash
# Using npm
npm install orkee

# Using pnpm
pnpm add orkee

# Using yarn
yarn add orkee
```

## Verification

Verify the installation:

```bash
# Check version
orkee --version

# View help
orkee --help

# Test installation
orkee dashboard --help
```

## Starting Orkee

### Quick Start

Start both the API server and dashboard:

```bash
orkee dashboard
```

This will:
- Start the API server on port 4001
- Launch the dashboard on port 5173
- Open your browser automatically

### Custom Ports

Specify custom ports if needed:

```bash
# Custom API port
orkee dashboard --api-port 8080

# Custom UI port
orkee dashboard --ui-port 3000

# Both custom ports
orkee dashboard --api-port 8080 --ui-port 3000
```

### Environment Variables

Alternative port configuration:

```bash
# Set environment variables
export ORKEE_API_PORT=8080
export ORKEE_UI_PORT=3000

# Start with environment settings
orkee dashboard
```

## Platform-Specific Notes

### macOS

- Installation requires macOS 10.15 (Catalina) or later
- First run may show a security warning - approve in System Preferences > Security & Privacy
- Both x64 and Apple Silicon (M1/M2) are supported

### Linux

- Supports Ubuntu 18.04+, Debian 10+, CentOS 7+, and other modern distributions
- Both x64 and ARM64 architectures supported
- May require `libc6` on some minimal distributions

### Windows

- Supports Windows 10/11 and Windows Server 2016+
- PowerShell or Command Prompt supported
- Both x64 and ARM64 architectures supported

## Troubleshooting

### Permission Issues

If you get permission errors during global installation:

```bash
# On macOS/Linux - use sudo
sudo npm install -g orkee

# Or configure npm to use a different directory
npm config set prefix ~/.npm-global
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
npm install -g orkee
```

### Binary Download Issues

If the binary download fails:

```bash
# Clear npm cache
npm cache clean --force

# Try installing again
npm install -g orkee

# Or use direct binary installation
curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-$(uname -s)-$(uname -m).tar.gz | tar xz
```

### Port Conflicts

If default ports are in use:

```bash
# Check what's using the ports
lsof -i :4001
lsof -i :5173

# Use different ports
orkee dashboard --api-port 4002 --ui-port 5174
```

### Node.js Version Issues

If you have an incompatible Node.js version:

```bash
# Check current version
node --version

# Install/update Node.js
# Visit: https://nodejs.org/
# Or use a version manager like nvm:
nvm install --lts
nvm use --lts
```

## Updating Orkee

### Check for Updates

```bash
# Check current version
orkee --version

# Check npm for latest version
npm view orkee version
```

### Update Global Installation

```bash
# Using npm
npm update -g orkee

# Using pnpm
pnpm update -g orkee

# Using yarn
yarn global upgrade orkee
```

### Update Local Installation

```bash
# Using npm
npm update orkee

# Using pnpm
pnpm update orkee

# Using yarn
yarn upgrade orkee
```

## Uninstallation

### Remove Global Installation

```bash
# Using npm
npm uninstall -g orkee

# Using pnpm
pnpm remove -g orkee

# Using yarn
yarn global remove orkee
```

### Clean Up Data

If you want to completely remove Orkee and its data:

```bash
# Remove the data directory (CAUTION: This deletes all projects!)
rm -rf ~/.orkee/

# On Windows:
# rmdir /s %USERPROFILE%\.orkee\
```

## Next Steps

After installation:

1. **[First Run](../getting-started/first-run.md)** - Complete initial setup
2. **[Configuration](../configuration/server-configuration.md)** - Customize settings
3. **[Projects Guide](../user-guide/projects.md)** - Learn project management
4. **[Dashboard Guide](../user-guide/dashboard.md)** - Explore the web interface

## Advanced Options

### Pre-release Versions

Install beta or development versions:

```bash
# Install specific version
npm install -g orkee@1.0.0-beta.1

# Install latest pre-release
npm install -g orkee@next
```

### Offline Installation

For air-gapped environments:

```bash
# Download the package
npm pack orkee

# Transfer orkee-1.0.0.tgz to target system
npm install -g orkee-1.0.0.tgz
```

### Custom Binary Location

Override the binary download location:

```bash
# Set custom binary path
export ORKEE_BINARY_PATH=/usr/local/bin/orkee

# Install with custom path
npm install -g orkee
```