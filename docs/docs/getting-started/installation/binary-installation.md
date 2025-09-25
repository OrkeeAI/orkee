---
sidebar_position: 2
title: Binary Installation
---

# Binary Installation

Install Orkee using pre-compiled binaries for when you need direct control or npm is not available.

## Quick Download

Download the appropriate binary for your platform from [GitHub Releases](https://github.com/OrkeeAI/orkee/releases/latest).

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="macos" label="macOS" default>

```bash
# Intel Macs
curl -L https://github.com/OrkeeAI/orkee/releases/latest/download/orkee-x86_64-apple-darwin.tar.gz | tar xz

# Apple Silicon Macs  
curl -L https://github.com/OrkeeAI/orkee/releases/latest/download/orkee-aarch64-apple-darwin.tar.gz | tar xz

# Make executable and move to PATH
chmod +x orkee
sudo mv orkee /usr/local/bin/
```

</TabItem>
<TabItem value="linux" label="Linux">

```bash
# x86_64 Linux
curl -L https://github.com/OrkeeAI/orkee/releases/latest/download/orkee-x86_64-unknown-linux-gnu.tar.gz | tar xz

# ARM64 Linux
curl -L https://github.com/OrkeeAI/orkee/releases/latest/download/orkee-aarch64-unknown-linux-gnu.tar.gz | tar xz

# Make executable and move to PATH
chmod +x orkee
sudo mv orkee /usr/local/bin/
```

</TabItem>
<TabItem value="windows" label="Windows">

```powershell
# Download for Windows
Invoke-WebRequest -Uri "https://github.com/OrkeeAI/orkee/releases/latest/download/orkee-x86_64-pc-windows-msvc.zip" -OutFile "orkee.zip"

# Extract
Expand-Archive orkee.zip -DestinationPath .

# Move to a directory in your PATH
Move-Item orkee.exe "C:\Program Files\orkee\orkee.exe"
```

Add `C:\Program Files\orkee` to your system PATH.

</TabItem>
</Tabs>

## Platform Support

| Platform | Architecture | Status |
|----------|--------------|--------|
| macOS | Intel (x86_64) | ✅ Supported |
| macOS | Apple Silicon (ARM64) | ✅ Supported |
| Linux | x86_64 | ✅ Supported |
| Linux | ARM64 | ✅ Supported |
| Windows | x86_64 | ✅ Supported |
| Windows | ARM64 | 🚧 Experimental |

## Installation Steps

### 1. Detect Your Platform

```bash
# Check architecture
uname -m

# Check OS
uname -s
```

### 2. Download Binary

Choose the correct file for your platform:

- **macOS Intel**: `orkee-x86_64-apple-darwin.tar.gz`
- **macOS Apple Silicon**: `orkee-aarch64-apple-darwin.tar.gz`
- **Linux x86_64**: `orkee-x86_64-unknown-linux-gnu.tar.gz`
- **Linux ARM64**: `orkee-aarch64-unknown-linux-gnu.tar.gz`
- **Windows x86_64**: `orkee-x86_64-pc-windows-msvc.zip`

### 3. Extract and Install

<Tabs>
<TabItem value="unix" label="macOS/Linux" default>

```bash
# Extract
tar -xzf orkee-*.tar.gz

# Make executable
chmod +x orkee

# Move to PATH (choose one)
sudo mv orkee /usr/local/bin/           # System-wide
mkdir -p ~/.local/bin && mv orkee ~/.local/bin/  # User-only
```

</TabItem>
<TabItem value="windows" label="Windows">

```powershell
# Extract ZIP
Expand-Archive orkee-*.zip -DestinationPath .

# Create directory and move
New-Item -Path "C:\Program Files\orkee" -ItemType Directory -Force
Move-Item orkee.exe "C:\Program Files\orkee\"

# Add to PATH (restart terminal after)
$env:PATH += ";C:\Program Files\orkee"
```

</TabItem>
</Tabs>

### 4. Verify Installation

```bash
orkee --version
orkee --help
```

## Benefits

- **No dependencies**: Self-contained executable
- **Specific versions**: Pin to exact releases
- **CI/CD friendly**: Reproducible installations
- **Offline installation**: No internet needed after download

## Troubleshooting

### Permission Denied (macOS/Linux)

```bash
# Fix permissions
chmod +x orkee

# If still having issues
sudo chmod +x orkee
sudo chown $(whoami) orkee
```

### Command Not Found

Ensure the binary is in your PATH:

<Tabs>
<TabItem value="unix" label="macOS/Linux" default>

```bash
# Check PATH
echo $PATH

# Add directory to PATH permanently
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

</TabItem>
<TabItem value="windows" label="Windows">

1. Open System Properties → Advanced → Environment Variables
2. Edit the PATH variable
3. Add the directory containing `orkee.exe`
4. Restart your terminal

</TabItem>
</Tabs>

### Windows Security Warning

Windows may show a security warning for unsigned binaries:

1. Click "More info" when the warning appears
2. Click "Run anyway"
3. Or add an exclusion to Windows Defender for the orkee directory

### Verifying Downloads

All releases include checksums for verification:

```bash
# Download checksum file
curl -L https://github.com/OrkeeAI/orkee/releases/latest/download/checksums.txt

# Verify (macOS/Linux)
shasum -c checksums.txt

# Verify (Windows)
certUtil -hashfile orkee-x86_64-pc-windows-msvc.zip SHA256
```

## Updating

To update to a new version:

1. Download the new binary
2. Replace the existing binary
3. Verify the new version: `orkee --version`

## Uninstalling

```bash
# Remove binary
sudo rm /usr/local/bin/orkee

# Clean up configuration (optional)
rm -rf ~/.orkee
```

## Next Steps

- [Quick Start Guide](../quick-start) - Get up and running
- [First Project](../first-project) - Create your first project
- [Configuration](../../configuration/environment-variables) - Customize your setup