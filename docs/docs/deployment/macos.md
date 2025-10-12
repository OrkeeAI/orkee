---
sidebar_position: 5
title: macOS Deployment
---

# macOS Deployment

Deploy Orkee on macOS for development, team servers, or production environments with launchd service management.

## Installation Methods

### Method 1: Homebrew (Recommended)

The easiest way to install Orkee on macOS:

```bash
# Install Orkee via Homebrew
brew tap OrkeeAI/orkee
brew install orkee

# Verify installation
orkee --version
orkee --help
```

### Method 2: npm Global Install

```bash
# Install globally via npm
npm install -g orkee

# Verify installation
orkee --version
```

### Method 3: Build from Source

For the latest development version or customization:

```bash
# Install prerequisites
brew install rust node pnpm

# Clone repository
git clone https://github.com/OrkeeAI/orkee.git
cd orkee

# Install dependencies
pnpm install

# Build all packages
turbo build

# Build Rust binary (optimized)
cd packages/cli
cargo build --release

# Optional: Build with cloud features
cargo build --release --features cloud

# Install binary
sudo cp target/release/orkee /usr/local/bin/
sudo chmod 755 /usr/local/bin/orkee

# Verify
orkee --version
```

## Quick Start

### Development Server

For local development or testing:

```bash
# Start dashboard with defaults (API: 4001, UI: 5173)
orkee dashboard

# Custom ports
orkee dashboard --api-port 8080 --ui-port 3000

# Development mode (use local source)
orkee dashboard --dev
```

### Production Server

For team or production deployments:

```bash
# Create configuration directory
mkdir -p ~/.orkee

# Create production environment file
cat > ~/.orkee/production.env << 'EOF'
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
ORKEE_HOST=0.0.0.0

CORS_ORIGIN=https://your-domain.local
RATE_LIMIT_ENABLED=true
SECURITY_HEADERS_ENABLED=true

RUST_LOG=info
EOF

# Start with production config
orkee dashboard
```

## Service Management with launchd

### Create Launch Agent (Current User)

For running Orkee as the current user:

```bash
# Create launch agent directory
mkdir -p ~/Library/LaunchAgents

# Create plist file
cat > ~/Library/LaunchAgents/com.orkee.dashboard.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.orkee.dashboard</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/orkee</string>
        <string>dashboard</string>
    </array>

    <key>EnvironmentVariables</key>
    <dict>
        <key>ORKEE_API_PORT</key>
        <string>4001</string>
        <key>ORKEE_UI_PORT</key>
        <string>5173</string>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/usr/local/var/log/orkee/stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/orkee/stderr.log</string>

    <key>WorkingDirectory</key>
    <string>/usr/local/var/lib/orkee</string>
</dict>
</plist>
EOF

# Create log directory
sudo mkdir -p /usr/local/var/log/orkee
sudo chown $USER /usr/local/var/log/orkee

# Create data directory
sudo mkdir -p /usr/local/var/lib/orkee
sudo chown $USER /usr/local/var/lib/orkee

# Load service
launchctl load ~/Library/LaunchAgents/com.orkee.dashboard.plist

# Check status
launchctl list | grep orkee
```

### Create Launch Daemon (System-Wide)

For system-wide deployment accessible to all users:

```bash
# Create orkee user and group
sudo dscl . -create /Users/orkee
sudo dscl . -create /Users/orkee UserShell /usr/bin/false
sudo dscl . -create /Users/orkee RealName "Orkee Service"
sudo dscl . -create /Users/orkee UniqueID 520
sudo dscl . -create /Users/orkee PrimaryGroupID 520
sudo dscl . -create /Users/orkee NFSHomeDirectory /var/lib/orkee

# Create directories
sudo mkdir -p /var/lib/orkee/{data,certs,logs}
sudo mkdir -p /etc/orkee
sudo chown -R orkee:orkee /var/lib/orkee

# Create environment file
sudo tee /etc/orkee/production.env << 'EOF'
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
ORKEE_HOST=0.0.0.0

CORS_ORIGIN=https://your-domain.local
CORS_ALLOW_ANY_LOCALHOST=false

BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS=/var/lib/orkee/data

RATE_LIMIT_ENABLED=true
RATE_LIMIT_GLOBAL_RPM=30
SECURITY_HEADERS_ENABLED=true
ENABLE_REQUEST_ID=true

RUST_LOG=info
EOF

# Set permissions
sudo chmod 600 /etc/orkee/production.env
sudo chown orkee:orkee /etc/orkee/production.env

# Create launch daemon plist
sudo tee /Library/LaunchDaemons/com.orkee.dashboard.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.orkee.dashboard</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/orkee</string>
        <string>dashboard</string>
    </array>

    <key>UserName</key>
    <string>orkee</string>

    <key>GroupName</key>
    <string>orkee</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>HOME</key>
        <string>/var/lib/orkee</string>
    </dict>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>

    <key>ThrottleInterval</key>
    <integer>10</integer>

    <key>StandardOutPath</key>
    <string>/var/lib/orkee/logs/stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/var/lib/orkee/logs/stderr.log</string>

    <key>WorkingDirectory</key>
    <string>/var/lib/orkee</string>

    <key>SoftResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>65536</integer>
    </dict>

    <key>HardResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>65536</integer>
    </dict>
</dict>
</plist>
EOF

# Set permissions
sudo chmod 644 /Library/LaunchDaemons/com.orkee.dashboard.plist
sudo chown root:wheel /Library/LaunchDaemons/com.orkee.dashboard.plist

# Load daemon
sudo launchctl load /Library/LaunchDaemons/com.orkee.dashboard.plist

# Check status
sudo launchctl list | grep orkee
```

### Service Management Commands

```bash
# User service (LaunchAgent)
# -----------------------------------------
# Load service
launchctl load ~/Library/LaunchAgents/com.orkee.dashboard.plist

# Unload service
launchctl unload ~/Library/LaunchAgents/com.orkee.dashboard.plist

# Start service
launchctl start com.orkee.dashboard

# Stop service
launchctl stop com.orkee.dashboard

# Check if running
launchctl list | grep orkee

# View logs
tail -f /usr/local/var/log/orkee/stdout.log
tail -f /usr/local/var/log/orkee/stderr.log


# System service (LaunchDaemon)
# -----------------------------------------
# Load daemon
sudo launchctl load /Library/LaunchDaemons/com.orkee.dashboard.plist

# Unload daemon
sudo launchctl unload /Library/LaunchDaemons/com.orkee.dashboard.plist

# Start daemon
sudo launchctl start com.orkee.dashboard

# Stop daemon
sudo launchctl stop com.orkee.dashboard

# Check if running
sudo launchctl list | grep orkee

# View logs
sudo tail -f /var/lib/orkee/logs/stdout.log
sudo tail -f /var/lib/orkee/logs/stderr.log
```

## TLS/HTTPS Configuration

### Self-Signed Certificate (Development)

```bash
# Create certificate directory
mkdir -p ~/.orkee/certs

# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 \
  -keyout ~/.orkee/certs/key.pem \
  -out ~/.orkee/certs/cert.pem \
  -days 365 -nodes \
  -subj "/CN=localhost"

# Set permissions
chmod 600 ~/.orkee/certs/key.pem
chmod 644 ~/.orkee/certs/cert.pem

# Add to macOS Keychain
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain \
  ~/.orkee/certs/cert.pem

# Configure Orkee
cat >> ~/.orkee/production.env << 'EOF'
TLS_ENABLED=true
TLS_CERT_PATH=$HOME/.orkee/certs/cert.pem
TLS_KEY_PATH=$HOME/.orkee/certs/key.pem
AUTO_GENERATE_CERT=false
ORKEE_API_PORT=4443
EOF
```

### Commercial Certificate

```bash
# Create secure certificate directory
sudo mkdir -p /etc/orkee/certs
sudo chmod 755 /etc/orkee/certs

# Copy certificates
sudo cp your-certificate.crt /etc/orkee/certs/cert.pem
sudo cp your-private-key.key /etc/orkee/certs/key.pem
sudo cp your-ca-bundle.crt /etc/orkee/certs/chain.pem

# Combine if needed
sudo cat /etc/orkee/certs/cert.pem /etc/orkee/certs/chain.pem \
  > /etc/orkee/certs/fullchain.pem

# Set permissions
sudo chmod 600 /etc/orkee/certs/key.pem
sudo chmod 644 /etc/orkee/certs/*.pem
sudo chown -R orkee:orkee /etc/orkee/certs

# Configure
TLS_ENABLED=true
TLS_CERT_PATH=/etc/orkee/certs/fullchain.pem
TLS_KEY_PATH=/etc/orkee/certs/key.pem
ORKEE_API_PORT=443
```

## Firewall Configuration

### Application Firewall

```bash
# Allow Orkee through firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/local/bin/orkee
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp /usr/local/bin/orkee

# Check firewall status
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# List applications
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps | grep orkee
```

### pf (Packet Filter)

For advanced firewall rules:

```bash
# Create pf rules file
sudo tee /etc/pf.anchors/com.orkee << 'EOF'
# Orkee firewall rules

# Allow SSH (be careful!)
pass in proto tcp from any to any port 22

# Allow HTTP/HTTPS
pass in proto tcp from any to any port 80
pass in proto tcp from any to any port 443

# Allow Orkee API (if needed)
pass in proto tcp from any to any port 4001

# Block direct access to backend (use only with reverse proxy)
# block in proto tcp from any to any port 4001
EOF

# Load rules
sudo pfctl -ef /etc/pf.anchors/com.orkee

# Check rules
sudo pfctl -sr
```

## Reverse Proxy with Nginx

### Install Nginx

```bash
# Install via Homebrew
brew install nginx

# Start Nginx
brew services start nginx

# Or start manually
nginx
```

### Configure Nginx

```bash
# Edit Nginx configuration
sudo vim /usr/local/etc/nginx/nginx.conf

# Or create site config
sudo mkdir -p /usr/local/etc/nginx/sites-enabled
sudo tee /usr/local/etc/nginx/sites-enabled/orkee.conf << 'EOF'
server {
    listen 80;
    server_name localhost your-domain.local;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name localhost your-domain.local;

    ssl_certificate /etc/orkee/certs/fullchain.pem;
    ssl_certificate_key /etc/orkee/certs/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;

    location / {
        proxy_pass http://127.0.0.1:4001;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
EOF

# Test configuration
nginx -t

# Reload Nginx
nginx -s reload

# Or restart service
brew services restart nginx
```

## Monitoring & Logging

### Log Management

```bash
# View logs (user service)
tail -f /usr/local/var/log/orkee/stdout.log
tail -f /usr/local/var/log/orkee/stderr.log

# View logs (system service)
sudo tail -f /var/lib/orkee/logs/stdout.log
sudo tail -f /var/lib/orkee/logs/stderr.log

# Search logs
grep ERROR /var/lib/orkee/logs/stderr.log

# Log rotation (using newsyslog)
sudo tee -a /etc/newsyslog.conf << 'EOF'
/var/lib/orkee/logs/*.log   orkee:orkee  644  7  *  $D0  J
EOF
```

### Health Check Script

```bash
# Create monitoring script
cat > /usr/local/bin/orkee-monitor.sh << 'EOF'
#!/bin/bash
HEALTH_URL="http://localhost:4001/api/health"
LOG_FILE="/var/log/orkee-monitor.log"

if ! curl -f -s -o /dev/null "$HEALTH_URL"; then
    echo "$(date): Health check failed - restarting service" >> "$LOG_FILE"
    launchctl stop com.orkee.dashboard
    sleep 2
    launchctl start com.orkee.dashboard
else
    echo "$(date): Health check passed" >> "$LOG_FILE"
fi
EOF

# Make executable
chmod +x /usr/local/bin/orkee-monitor.sh

# Create launch agent for monitoring
cat > ~/Library/LaunchAgents/com.orkee.monitor.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.orkee.monitor</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/orkee-monitor.sh</string>
    </array>

    <key>StartInterval</key>
    <integer>300</integer>

    <key>StandardOutPath</key>
    <string>/var/log/orkee-monitor.log</string>

    <key>StandardErrorPath</key>
    <string>/var/log/orkee-monitor.log</string>
</dict>
</plist>
EOF

# Load monitoring service
launchctl load ~/Library/LaunchAgents/com.orkee.monitor.plist
```

## Performance Optimization

### Increase File Descriptor Limits

```bash
# Check current limits
ulimit -n

# Increase for current session
ulimit -n 65536

# Permanent increase (add to ~/.zshrc or ~/.bash_profile)
echo 'ulimit -n 65536' >> ~/.zshrc

# System-wide limits (requires reboot)
sudo tee -a /etc/sysctl.conf << 'EOF'
kern.maxfiles=65536
kern.maxfilesperproc=65536
EOF
```

### Optimize Database

```bash
# Vacuum database
sqlite3 ~/.orkee/orkee.db "VACUUM;"

# Analyze for query optimization
sqlite3 ~/.orkee/orkee.db "ANALYZE;"

# Enable WAL mode (if not already)
sqlite3 ~/.orkee/orkee.db "PRAGMA journal_mode=WAL;"
```

## Backup & Recovery

### Automated Backup Script

```bash
# Create backup script
cat > /usr/local/bin/orkee-backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="$HOME/Backups/orkee"
DB_PATH="$HOME/.orkee/orkee.db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/orkee_backup_$TIMESTAMP.db"

mkdir -p "$BACKUP_DIR"

if [ -f "$DB_PATH" ]; then
    cp "$DB_PATH" "$BACKUP_FILE"
    gzip "$BACKUP_FILE"
    echo "$(date): Backup created: $BACKUP_FILE.gz"

    # Keep last 30 days
    find "$BACKUP_DIR" -name "orkee_backup_*.db.gz" -mtime +30 -delete
else
    echo "$(date): Database not found: $DB_PATH"
fi
EOF

chmod +x /usr/local/bin/orkee-backup.sh

# Schedule daily backups
cat > ~/Library/LaunchAgents/com.orkee.backup.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.orkee.backup</string>

    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/orkee-backup.sh</string>
    </array>

    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>2</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
</dict>
</plist>
EOF

launchctl load ~/Library/LaunchAgents/com.orkee.backup.plist
```

### Time Machine Exclusions

```bash
# Exclude temporary/cache files from backups
tmutil addexclusion ~/.orkee/logs
tmutil addexclusion ~/.orkee/cache
tmutil addexclusion /usr/local/var/log/orkee
```

## Troubleshooting

### Service Won't Start

```bash
# Check service status
launchctl list | grep orkee

# View error logs
tail -50 /var/lib/orkee/logs/stderr.log

# Test binary manually
/usr/local/bin/orkee --version
/usr/local/bin/orkee dashboard --help

# Check port availability
lsof -i :4001

# Unload and reload service
launchctl unload ~/Library/LaunchAgents/com.orkee.dashboard.plist
launchctl load ~/Library/LaunchAgents/com.orkee.dashboard.plist
```

### Permission Issues

```bash
# Fix ownership (user service)
chown -R $USER ~/.orkee
chmod 755 ~/.orkee

# Fix ownership (system service)
sudo chown -R orkee:orkee /var/lib/orkee
sudo chmod 755 /var/lib/orkee
sudo chmod 600 /etc/orkee/production.env
```

### Network Issues

```bash
# Check if API is listening
lsof -i -P | grep orkee

# Test API endpoint
curl http://localhost:4001/api/health

# Check firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getappblocked /usr/local/bin/orkee
```

## Updating Orkee

```bash
# Homebrew installation
brew upgrade orkee

# npm installation
npm update -g orkee

# From source
cd ~/orkee
git pull origin main
pnpm install
turbo build
cd packages/cli
cargo build --release
sudo cp target/release/orkee /usr/local/bin/

# Restart service
launchctl stop com.orkee.dashboard
launchctl start com.orkee.dashboard
```

## Next Steps

- [TLS/HTTPS Configuration](tls-https) - Advanced SSL setup
- [Reverse Proxy Setup](reverse-proxy) - Nginx configuration
- [Security Settings](../configuration/security-settings) - Security hardening
- [Monitoring Guide](../operations/monitoring) - Advanced monitoring
