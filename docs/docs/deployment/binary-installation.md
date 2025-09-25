# Binary Installation

Install Orkee by downloading pre-built binaries directly from GitHub releases. This method gives you full control over the installation process and doesn't require npm or Docker.

## Prerequisites

- **Operating System**: Linux, macOS, or Windows
- **Architecture**: x64 (AMD64) or ARM64
- **Network**: Internet access to download binaries

## Download Options

### Latest Release

Download the latest stable release:

```bash
# Linux x64
curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-linux-x64.tar.gz -o orkee.tar.gz

# macOS x64 (Intel)
curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-macos-x64.tar.gz -o orkee.tar.gz

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-macos-arm64.tar.gz -o orkee.tar.gz

# Windows x64
curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-windows-x64.zip -o orkee.zip
```

### Specific Version

Download a specific version (replace `v1.0.0` with desired version):

```bash
# Linux x64
curl -L https://github.com/orkee-ai/orkee/releases/download/v1.0.0/orkee-linux-x64.tar.gz -o orkee.tar.gz

# macOS x64
curl -L https://github.com/orkee-ai/orkee/releases/download/v1.0.0/orkee-macos-x64.tar.gz -o orkee.tar.gz

# Windows x64
curl -L https://github.com/orkee-ai/orkee/releases/download/v1.0.0/orkee-windows-x64.zip -o orkee.zip
```

## Installation by Platform

### Linux/macOS Installation

1. **Download and extract**:
   ```bash
   # Download (use appropriate URL from above)
   curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-linux-x64.tar.gz -o orkee.tar.gz
   
   # Extract
   tar -xzf orkee.tar.gz
   
   # Make executable (if not already)
   chmod +x orkee
   ```

2. **Install globally** (optional):
   ```bash
   # Move to system PATH
   sudo mv orkee /usr/local/bin/
   
   # Or for user-only installation
   mkdir -p ~/bin
   mv orkee ~/bin/
   echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

3. **Verify installation**:
   ```bash
   orkee --version
   ```

### Windows Installation

1. **Download and extract**:
   ```powershell
   # Download using PowerShell
   Invoke-WebRequest -Uri "https://github.com/orkee-ai/orkee/releases/latest/download/orkee-windows-x64.zip" -OutFile "orkee.zip"
   
   # Extract
   Expand-Archive -Path "orkee.zip" -DestinationPath "."
   ```

2. **Add to PATH** (optional):
   ```powershell
   # Create directory for binaries
   New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\bin"
   
   # Move binary
   Move-Item -Path "orkee.exe" -Destination "$env:USERPROFILE\bin\"
   
   # Add to PATH permanently
   $env:PATH += ";$env:USERPROFILE\bin"
   [Environment]::SetEnvironmentVariable("PATH", $env:PATH, [EnvironmentVariableTarget]::User)
   ```

3. **Verify installation**:
   ```cmd
   orkee.exe --version
   ```

## Starting Orkee

### Quick Start

Start both API server and dashboard:

```bash
# Linux/macOS
./orkee dashboard

# Windows
orkee.exe dashboard

# If installed globally
orkee dashboard
```

### Configuration

Use environment variables or command-line flags:

```bash
# Custom ports
orkee dashboard --api-port 8080 --ui-port 3000

# With environment variables
export ORKEE_API_PORT=8080
export ORKEE_UI_PORT=3000
orkee dashboard

# Enable debug logging
RUST_LOG=debug orkee dashboard
```

## Advanced Installation

### System Service (Linux)

Create a systemd service for automatic startup:

1. **Create service file**:
   ```bash
   sudo nano /etc/systemd/system/orkee.service
   ```

2. **Service configuration**:
   ```ini
   [Unit]
   Description=Orkee AI Agent Orchestration Platform
   After=network.target
   
   [Service]
   Type=simple
   User=orkee
   Group=orkee
   WorkingDirectory=/opt/orkee
   ExecStart=/usr/local/bin/orkee dashboard --api-port 4001 --ui-port 5173
   Restart=always
   RestartSec=10
   
   # Environment
   Environment=RUST_LOG=info
   Environment=ORKEE_API_PORT=4001
   Environment=ORKEE_UI_PORT=5173
   
   # Security
   NoNewPrivileges=yes
   PrivateTmp=yes
   ProtectSystem=strict
   ProtectHome=yes
   ReadWritePaths=/opt/orkee
   
   [Install]
   WantedBy=multi-user.target
   ```

3. **Setup and start**:
   ```bash
   # Create user
   sudo useradd -r -s /bin/false orkee
   sudo mkdir -p /opt/orkee
   sudo chown orkee:orkee /opt/orkee
   
   # Enable and start service
   sudo systemctl daemon-reload
   sudo systemctl enable orkee
   sudo systemctl start orkee
   
   # Check status
   sudo systemctl status orkee
   ```

### System Service (macOS)

Create a LaunchDaemon for automatic startup:

1. **Create plist file**:
   ```bash
   sudo nano /Library/LaunchDaemons/ai.orkee.daemon.plist
   ```

2. **LaunchDaemon configuration**:
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>Label</key>
       <string>ai.orkee.daemon</string>
       <key>ProgramArguments</key>
       <array>
           <string>/usr/local/bin/orkee</string>
           <string>dashboard</string>
           <string>--api-port</string>
           <string>4001</string>
           <string>--ui-port</string>
           <string>5173</string>
       </array>
       <key>RunAtLoad</key>
       <true/>
       <key>KeepAlive</key>
       <true/>
       <key>EnvironmentVariables</key>
       <dict>
           <key>RUST_LOG</key>
           <string>info</string>
       </dict>
   </dict>
   </plist>
   ```

3. **Load and start**:
   ```bash
   # Load daemon
   sudo launchctl load /Library/LaunchDaemons/ai.orkee.daemon.plist
   
   # Start daemon
   sudo launchctl start ai.orkee.daemon
   
   # Check status
   sudo launchctl list | grep orkee
   ```

### Windows Service

Install as a Windows service using NSSM (Non-Sucking Service Manager):

1. **Download NSSM**:
   - Download from https://nssm.cc/download
   - Extract `nssm.exe` to a folder in PATH

2. **Install service**:
   ```cmd
   # Install service
   nssm install Orkee "C:\Program Files\Orkee\orkee.exe"
   nssm set Orkee Arguments "dashboard --api-port 4001 --ui-port 5173"
   nssm set Orkee DisplayName "Orkee AI Platform"
   nssm set Orkee Description "AI Agent Orchestration Platform"
   nssm set Orkee Start SERVICE_AUTO_START
   
   # Set environment
   nssm set Orkee AppEnvironmentExtra RUST_LOG=info
   
   # Start service
   nssm start Orkee
   ```

## Security Considerations

### File Permissions

Set appropriate file permissions:

```bash
# Linux/macOS - restrict binary access
chmod 755 orkee
chown root:root orkee  # If installed system-wide

# Make data directory secure
chmod 700 ~/.orkee/
```

### Firewall Configuration

Configure firewall rules:

```bash
# Linux (ufw)
sudo ufw allow 4001/tcp  # API port
sudo ufw allow 5173/tcp  # Dashboard port (if external access needed)

# Linux (iptables)
sudo iptables -A INPUT -p tcp --dport 4001 -j ACCEPT

# macOS
sudo pfctl -e
echo "pass in proto tcp from any to any port 4001" | sudo pfctl -f -
```

## Troubleshooting

### Binary Won't Execute

Check architecture and dependencies:

```bash
# Check binary architecture
file orkee

# Check for missing libraries (Linux)
ldd orkee

# Check for missing libraries (macOS)
otool -L orkee
```

### Permission Denied

Fix execution permissions:

```bash
# Make executable
chmod +x orkee

# Check if directory is in PATH
echo $PATH

# Run with full path
./orkee dashboard
```

### Port Already in Use

Find and stop conflicting processes:

```bash
# Find process using port
lsof -i :4001
netstat -tulpn | grep 4001

# Kill process (if safe to do so)
sudo kill -9 <PID>

# Or use different ports
orkee dashboard --api-port 4002 --ui-port 5174
```

## Updating

### Manual Update

Replace the binary with a newer version:

```bash
# Download new version
curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-linux-x64.tar.gz -o orkee-new.tar.gz

# Stop current instance
pkill orkee

# Backup current binary
mv orkee orkee-backup

# Extract and install new version
tar -xzf orkee-new.tar.gz
chmod +x orkee
sudo mv orkee /usr/local/bin/  # If installed globally

# Start new version
orkee dashboard
```

### Automated Update Script

Create an update script:

```bash
#!/bin/bash
# save as update-orkee.sh

set -e

INSTALL_DIR="/usr/local/bin"
BACKUP_DIR="$HOME/.orkee/backups"
PLATFORM="linux-x64"  # Change as needed

echo "Stopping Orkee..."
pkill orkee || true

echo "Creating backup..."
mkdir -p "$BACKUP_DIR"
cp "$INSTALL_DIR/orkee" "$BACKUP_DIR/orkee-$(date +%Y%m%d-%H%M%S)" || true

echo "Downloading latest version..."
curl -L "https://github.com/orkee-ai/orkee/releases/latest/download/orkee-$PLATFORM.tar.gz" -o /tmp/orkee.tar.gz

echo "Installing..."
tar -xzf /tmp/orkee.tar.gz -C /tmp/
chmod +x /tmp/orkee
sudo mv /tmp/orkee "$INSTALL_DIR/"

echo "Starting Orkee..."
orkee dashboard &

echo "Update complete!"
```

Make executable and run:

```bash
chmod +x update-orkee.sh
./update-orkee.sh
```

## Next Steps

After binary installation:

1. **[Configuration](../configuration/server-configuration.md)** - Customize settings
2. **[First Run](../getting-started/first-run.md)** - Complete initial setup
3. **[Production Setup](../configuration/production.md)** - Production considerations
4. **[TLS Configuration](./tls-https.md)** - Enable HTTPS