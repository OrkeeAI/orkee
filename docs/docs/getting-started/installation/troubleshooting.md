---
sidebar_position: 4
title: Installation Troubleshooting
---

# Installation Troubleshooting

Common installation issues and their solutions.

## NPM Installation Issues

### Command Not Found

**Problem**: `orkee: command not found` after installing via npm.

**Solution**:

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="check-path" label="Check PATH" default>

```bash
# Check if npm global bin is in PATH
npm config get prefix
echo $PATH

# Add npm global bin to PATH
echo 'export PATH="$(npm config get prefix)/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

</TabItem>
<TabItem value="reinstall" label="Reinstall">

```bash
# Uninstall and reinstall
npm uninstall -g orkee
npm install -g orkee

# Verify installation
orkee --version
```

</TabItem>
</Tabs>

### Permission Errors

**Problem**: `EACCES: permission denied` during npm install.

**Solutions**:

<Tabs>
<TabItem value="nvm" label="Use Node Version Manager (Recommended)" default>

```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install and use Node.js
nvm install node
nvm use node

# Install Orkee
npm install -g orkee
```

</TabItem>
<TabItem value="change-directory" label="Change npm Directory">

```bash
# Create directory for global packages
mkdir ~/.npm-global

# Configure npm to use new directory
npm config set prefix '~/.npm-global'

# Add to PATH
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.profile
source ~/.profile

# Install Orkee
npm install -g orkee
```

</TabItem>
</Tabs>

## Binary Installation Issues

### Permission Denied (macOS/Linux)

**Problem**: `Permission denied` when trying to run orkee binary.

**Solution**:

```bash
# Make executable
chmod +x orkee

# If still having issues
sudo chmod +x orkee
sudo chown $(whoami) orkee
```

### macOS Gatekeeper Warning

**Problem**: macOS blocks execution of unsigned binary.

**Solution**:

1. **Right-click** the orkee binary → **Open**
2. Click **Open** in the security dialog
3. Or use terminal:
   ```bash
   xattr -d com.apple.quarantine orkee
   ```

### Windows Security Warning

**Problem**: Windows SmartScreen blocks execution.

**Solution**:

1. Click **More info** in the warning dialog
2. Click **Run anyway**
3. Or add an exclusion in Windows Defender:
   - Open **Windows Security** → **Virus & threat protection**
   - Click **Manage settings** under Virus & threat protection settings
   - Add exclusion for the orkee directory

## Source Installation Issues

### Rust Not Found

**Problem**: `cargo: command not found` when building from source.

**Solution**:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Build Dependencies Missing

**Problem**: Build fails with missing system dependencies.

**Solutions by platform**:

<Tabs>
<TabItem value="ubuntu" label="Ubuntu/Debian" default>

```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev
```

</TabItem>
<TabItem value="centos" label="CentOS/RHEL/Fedora">

```bash
sudo dnf install gcc gcc-c++ openssl-devel pkg-config
```

</TabItem>
<TabItem value="macos" label="macOS">

```bash
# Install Xcode command line tools
xcode-select --install
```

</TabItem>
</Tabs>

### Node.js Version Issues

**Problem**: Build fails due to Node.js version mismatch.

**Solution**:

```bash
# Check Node.js version
node --version

# Update to supported version (18+)
# Using nvm
nvm install 18
nvm use 18

# Or using package manager
# macOS with Homebrew
brew upgrade node

# Clear cache and reinstall
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### Out of Memory During Build

**Problem**: Build fails with "out of memory" error.

**Solution**:

```bash
# Increase swap space (Linux)
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Or build with fewer parallel jobs
MAKEFLAGS="-j2" cargo build --release
```

## General Issues

### Antivirus Interference

**Problem**: Antivirus software blocks or quarantines orkee.

**Solution**:

1. **Add exception** for orkee binary/installation directory
2. **Temporarily disable** real-time protection during installation
3. **Download binary directly** instead of using package managers

### Network/Proxy Issues

**Problem**: Download fails due to network restrictions.

**Solution**:

<Tabs>
<TabItem value="proxy" label="Configure Proxy" default>

```bash
# For npm
npm config set proxy http://proxy:8080
npm config set https-proxy http://proxy:8080

# For cargo
export HTTPS_PROXY=http://proxy:8080
export HTTP_PROXY=http://proxy:8080
```

</TabItem>
<TabItem value="offline" label="Offline Installation">

```bash
# Download binary on a connected machine
# Transfer to target machine
# Make executable and install
chmod +x orkee
sudo mv orkee /usr/local/bin/
```

</TabItem>
</Tabs>

### Version Mismatch

**Problem**: Different versions of orkee components causing issues.

**Solution**:

```bash
# Check all components
orkee --version
npm list -g orkee

# Clean install
npm uninstall -g orkee
rm -rf ~/.orkee  # Optional: remove config
npm install -g orkee@latest
```

## Platform-Specific Issues

### Linux: GLIBC Version

**Problem**: Binary requires newer GLIBC than available.

**Solution**:

```bash
# Check GLIBC version
ldd --version

# If too old, build from source or use different binary
# Or upgrade system (if possible)
```

### macOS: Rosetta on M1/M2

**Problem**: x86_64 binary on Apple Silicon Mac.

**Solution**:

```bash
# Install Rosetta 2 if not already installed
sudo softwareupdate --install-rosetta

# Or download ARM64 binary instead
curl -L https://github.com/OrkeeAI/orkee/releases/latest/download/orkee-aarch64-apple-darwin.tar.gz | tar xz
```

### Windows: Missing Visual C++ Runtime

**Problem**: Binary requires Visual C++ runtime.

**Solution**:

Download and install [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)

## Getting Help

If none of these solutions work:

### Diagnostic Information

Collect this information when reporting issues:

```bash
# System information
uname -a                    # OS and architecture
orkee --version            # Orkee version (if working)
cargo --version           # Rust version
node --version             # Node.js version
npm --version              # npm version

# Installation method used
# Error messages (full output)
# Steps that led to the issue
```

### Where to Get Help

1. **[GitHub Issues](https://github.com/OrkeeAI/orkee/issues)** - Report bugs and request features
2. **[GitHub Discussions](https://github.com/OrkeeAI/orkee/discussions)** - Community help and questions
3. **[Discord Community](https://discord.gg/orkee)** - Real-time chat support

### Before Reporting

1. **Search existing issues** for your problem
2. **Try the latest version** - your issue may already be fixed
3. **Include diagnostic information** and error messages
4. **Specify your platform** and installation method used

## Quick Fix Checklist

Try these steps in order:

1. ✅ **Verify system requirements** (OS, Node.js, Rust versions)
2. ✅ **Check PATH** contains the directory where orkee was installed
3. ✅ **Restart terminal** after installation
4. ✅ **Run with full path** to confirm binary works: `/usr/local/bin/orkee --help`
5. ✅ **Try different installation method** (npm → binary → source)
6. ✅ **Check permissions** on the binary and installation directory
7. ✅ **Disable antivirus temporarily** during installation
8. ✅ **Clear caches** (npm, cargo) and reinstall
9. ✅ **Check for system updates** that might provide missing dependencies

## Next Steps

Once installation issues are resolved:

- [Quick Start Guide](../quick-start) - Get up and running
- [First Project](../first-project) - Create your first project
- [Configuration](../../configuration/environment-variables) - Customize your setup