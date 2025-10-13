---
sidebar_position: 4
title: Linux Server Deployment
---

# Linux Server Deployment

Complete guide for deploying Orkee on Linux servers with systemd service management, security hardening, and production best practices.

## System Requirements

### Minimum Requirements
- **CPU**: 2 cores
- **RAM**: 2GB
- **Storage**: 1GB (more for project data)
- **OS**: Ubuntu 20.04+, Debian 10+, CentOS 7+, RHEL 7+

### Recommended Requirements
- **CPU**: 4+ cores
- **RAM**: 4GB+ (with dashboard)
- **Storage**: 10GB+ SSD
- **OS**: Ubuntu 22.04 LTS or Debian 11+

### Software Prerequisites
- **Rust**: Latest stable (1.70+)
- **Node.js**: v18+
- **pnpm**: v8+
- **systemd**: For service management

## Quick Start

### 1. Install Dependencies

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="ubuntu" label="Ubuntu/Debian" default>

```bash
# Update package list
sudo apt update

# Install build tools
sudo apt install -y build-essential pkg-config libssl-dev curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js and pnpm
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs
npm install -g pnpm

# Verify installations
rustc --version
node --version
pnpm --version
```

</TabItem>
<TabItem value="centos" label="CentOS/RHEL">

```bash
# Update packages
sudo yum update -y

# Install EPEL repository
sudo yum install -y epel-release

# Install build tools
sudo yum groupinstall -y "Development Tools"
sudo yum install -y openssl-devel curl git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js
curl -fsSL https://rpm.nodesource.com/setup_18.x | sudo bash -
sudo yum install -y nodejs
npm install -g pnpm

# Verify installations
rustc --version
node --version
pnpm --version
```

</TabItem>
</Tabs>

### 2. Build Orkee

```bash
# Clone repository
git clone https://github.com/OrkeeAI/orkee.git
cd orkee

# Install Node.js dependencies
pnpm install

# Build frontend packages
turbo build

# Build Rust binary (optimized release build)
cd packages/cli
cargo build --release

# Optional: Build with cloud sync features
cargo build --release --features cloud

# Binary location
ls -lh target/release/orkee
```

### 3. Install Binary

```bash
# Copy binary to system location
sudo cp target/release/orkee /usr/local/bin/
sudo chown root:root /usr/local/bin/orkee
sudo chmod 755 /usr/local/bin/orkee

# Verify installation
orkee --version
orkee --help
```

## Production Deployment

### 1. Create Production User

Create a dedicated system user for running Orkee:

```bash
# Create orkee user
sudo useradd --system --home /var/lib/orkee --shell /bin/false orkee

# Create required directories
sudo mkdir -p /var/lib/orkee/{data,certs,logs}
sudo mkdir -p /etc/orkee

# Set ownership
sudo chown -R orkee:orkee /var/lib/orkee
sudo chmod 755 /var/lib/orkee
```

### 2. Configure Environment

Create production environment file:

```bash
# Create production environment config
sudo tee /etc/orkee/production.env << 'EOF'
# Server Configuration
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
ORKEE_HOST=127.0.0.1

# CORS Configuration
CORS_ORIGIN=https://your-domain.com
CORS_ALLOW_ANY_LOCALHOST=false

# Security Settings
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS=/var/lib/orkee/data

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_GLOBAL_RPM=30
RATE_LIMIT_BURST_SIZE=5

# Security Headers
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
ENABLE_REQUEST_ID=true

# Logging
RUST_LOG=info
ENABLE_METRICS=true

# TLS (if handling directly, not behind proxy)
TLS_ENABLED=false
# TLS_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem
# TLS_KEY_PATH=/etc/letsencrypt/live/your-domain.com/privkey.pem

# Cloud Configuration (optional)
# ORKEE_CLOUD_TOKEN=your_token_here
# ORKEE_CLOUD_API_URL=https://api.orkee.ai
EOF

# Set secure permissions
sudo chown orkee:orkee /etc/orkee/production.env
sudo chmod 600 /etc/orkee/production.env
```

### 3. Create Systemd Service

Create `/etc/systemd/system/orkee.service`:

```bash
sudo tee /etc/systemd/system/orkee.service << 'EOF'
[Unit]
Description=Orkee - AI Agent Orchestration Platform
Documentation=https://docs.orkee.ai
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=orkee
Group=orkee

# Environment
EnvironmentFile=/etc/orkee/production.env
WorkingDirectory=/var/lib/orkee

# Command
ExecStart=/usr/local/bin/orkee dashboard
ExecReload=/bin/kill -HUP $MAINPID

# Restart policy
Restart=always
RestartSec=10
StartLimitBurst=5
StartLimitInterval=60

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=orkee

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/orkee
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true
LockPersonality=true
MemoryDenyWriteExecute=false
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6

# Resource limits
LimitNOFILE=65536
TasksMax=4096

[Install]
WantedBy=multi-user.target
EOF
```

### 4. Enable and Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable orkee

# Start service
sudo systemctl start orkee

# Check status
sudo systemctl status orkee

# View logs
sudo journalctl -u orkee -f
```

## Service Management

### Basic Commands

```bash
# Start service
sudo systemctl start orkee

# Stop service
sudo systemctl stop orkee

# Restart service
sudo systemctl restart orkee

# Reload configuration (graceful)
sudo systemctl reload orkee

# Check status
sudo systemctl status orkee

# Enable auto-start on boot
sudo systemctl enable orkee

# Disable auto-start
sudo systemctl disable orkee
```

### View Logs

```bash
# Follow live logs
sudo journalctl -u orkee -f

# Last 100 lines
sudo journalctl -u orkee -n 100

# Today's logs
sudo journalctl -u orkee --since today

# Logs since specific time
sudo journalctl -u orkee --since "2024-01-15 10:00:00"

# Logs with priority (errors only)
sudo journalctl -u orkee -p err

# Export logs
sudo journalctl -u orkee > orkee.log
```

### Health Checks

```bash
# Check if service is running
systemctl is-active orkee

# Check if service is enabled
systemctl is-enabled orkee

# Test API endpoint
curl http://localhost:4001/api/health

# Check listening ports
sudo ss -tlnp | grep orkee
sudo netstat -tlnp | grep orkee
```

## Security Hardening

### 1. Firewall Configuration

<Tabs>
<TabItem value="ufw" label="UFW (Ubuntu/Debian)" default>

```bash
# Install UFW
sudo apt install ufw

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (important - don't lock yourself out!)
sudo ufw allow 22/tcp

# Allow HTTP and HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow specific port (if not using reverse proxy)
# sudo ufw allow 4001/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status verbose
```

</TabItem>
<TabItem value="firewalld" label="firewalld (CentOS/RHEL)">

```bash
# Install firewalld
sudo yum install firewalld

# Start and enable
sudo systemctl start firewalld
sudo systemctl enable firewalld

# Allow services
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https

# Allow specific port (if needed)
# sudo firewall-cmd --permanent --add-port=4001/tcp

# Reload firewall
sudo firewall-cmd --reload

# Check status
sudo firewall-cmd --list-all
```

</TabItem>
</Tabs>

### 2. File Permissions

```bash
# Secure configuration files
sudo chmod 600 /etc/orkee/production.env
sudo chown orkee:orkee /etc/orkee/production.env

# Secure data directory
sudo chmod 750 /var/lib/orkee/data
sudo chown -R orkee:orkee /var/lib/orkee/data

# Secure certificate files (if applicable)
sudo chmod 600 /var/lib/orkee/certs/*.pem
sudo chown orkee:orkee /var/lib/orkee/certs/*.pem

# Secure log directory
sudo chmod 750 /var/lib/orkee/logs
sudo chown orkee:orkee /var/lib/orkee/logs

# Binary permissions
sudo chmod 755 /usr/local/bin/orkee
sudo chown root:root /usr/local/bin/orkee
```

### 3. Log Rotation

Create `/etc/logrotate.d/orkee`:

```bash
sudo tee /etc/logrotate.d/orkee << 'EOF'
/var/lib/orkee/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    copytruncate
    create 0640 orkee orkee
    sharedscripts
    postrotate
        systemctl reload orkee > /dev/null 2>&1 || true
    endscript
}
EOF

# Test log rotation
sudo logrotate -d /etc/logrotate.d/orkee
```

### 4. System Resource Limits

Edit `/etc/security/limits.conf`:

```bash
# Add limits for orkee user
sudo tee -a /etc/security/limits.conf << 'EOF'
orkee soft nofile 65536
orkee hard nofile 65536
orkee soft nproc 4096
orkee hard nproc 4096
EOF
```

### 5. SELinux Configuration (CentOS/RHEL)

```bash
# Check SELinux status
sestatus

# Allow network connections (if using reverse proxy)
sudo setsebool -P httpd_can_network_connect 1

# Create custom SELinux policy (if needed)
# This example allows Orkee to bind to port 4001
sudo semanage port -a -t http_port_t -p tcp 4001

# Restore context
sudo restorecon -Rv /var/lib/orkee
sudo restorecon -Rv /etc/orkee
```

## Performance Optimization

### System Tuning

```bash
# Edit sysctl configuration
sudo tee -a /etc/sysctl.conf << 'EOF'
# Network tuning for high concurrent connections
net.core.somaxconn = 1024
net.ipv4.tcp_max_syn_backlog = 1024
net.ipv4.ip_local_port_range = 10000 65000
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 600

# File descriptor limits
fs.file-max = 2097152
EOF

# Apply changes
sudo sysctl -p
```

### Environment Configuration

```bash
# High-traffic production settings
RATE_LIMIT_GLOBAL_RPM=100
RATE_LIMIT_BURST_SIZE=20

# Resource limits
MAX_REQUEST_SIZE=10485760  # 10MB
REQUEST_TIMEOUT=30
KEEP_ALIVE_TIMEOUT=65

# Logging level (reduce for performance)
RUST_LOG=info  # or 'warn' for less logging
```

## Monitoring & Alerting

### Health Check Script

Create `/usr/local/bin/orkee-healthcheck.sh`:

```bash
#!/bin/bash
# Orkee health check and auto-restart script

HEALTH_URL="http://localhost:4001/api/health"
MAX_RETRIES=3
RETRY_DELAY=5
LOG_FILE="/var/log/orkee-monitoring.log"

log_message() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$LOG_FILE"
}

for i in $(seq 1 $MAX_RETRIES); do
    if curl -f -s -o /dev/null --connect-timeout 10 "$HEALTH_URL"; then
        log_message "Health check passed"
        exit 0
    fi

    log_message "Health check failed (attempt $i/$MAX_RETRIES)"

    if [ $i -lt $MAX_RETRIES ]; then
        sleep $RETRY_DELAY
    fi
done

# All retries failed - restart service
log_message "Health check failed after $MAX_RETRIES attempts - restarting service"
systemctl restart orkee

# Alert admin
mail -s "Orkee service restarted automatically" admin@example.com << EOF
Orkee health check failed after $MAX_RETRIES attempts.
Service has been automatically restarted.

Check logs: journalctl -u orkee -n 100
EOF

exit 1
```

**Set up cron job:**

```bash
# Make script executable
sudo chmod +x /usr/local/bin/orkee-healthcheck.sh

# Add to crontab (check every 5 minutes)
sudo crontab -e

# Add this line:
*/5 * * * * /usr/local/bin/orkee-healthcheck.sh
```

### Resource Monitoring

Create `/usr/local/bin/orkee-monitor.sh`:

```bash
#!/bin/bash
# Monitor Orkee resource usage

# Get process stats
PID=$(systemctl show --property MainPID --value orkee)

if [ "$PID" -gt 0 ]; then
    # CPU and memory usage
    ps -p $PID -o %cpu,%mem,rss,vsz,cmd --no-headers

    # Connection count
    CONNECTIONS=$(ss -ant | grep :4001 | wc -l)
    echo "Active connections: $CONNECTIONS"

    # Open files
    OPEN_FILES=$(lsof -p $PID 2>/dev/null | wc -l)
    echo "Open file descriptors: $OPEN_FILES"
else
    echo "Orkee is not running"
fi
```

```bash
# Make executable
sudo chmod +x /usr/local/bin/orkee-monitor.sh

# Run manually
sudo /usr/local/bin/orkee-monitor.sh

# Or add to cron for logging
*/10 * * * * /usr/local/bin/orkee-monitor.sh >> /var/log/orkee-stats.log 2>&1
```

## Backup & Recovery

### Database Backup

```bash
# Create backup directory
sudo mkdir -p /var/backups/orkee
sudo chown orkee:orkee /var/backups/orkee

# Backup script
sudo tee /usr/local/bin/orkee-backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/var/backups/orkee"
DB_PATH="/var/lib/orkee/data/orkee.db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/orkee_backup_$TIMESTAMP.db"
RETENTION_DAYS=30

# Create backup
if [ -f "$DB_PATH" ]; then
    cp "$DB_PATH" "$BACKUP_FILE"
    gzip "$BACKUP_FILE"
    echo "$(date): Backup created: $BACKUP_FILE.gz"

    # Remove old backups
    find "$BACKUP_DIR" -name "orkee_backup_*.db.gz" -mtime +$RETENTION_DAYS -delete
    echo "$(date): Old backups cleaned (retention: $RETENTION_DAYS days)"
else
    echo "$(date): Database file not found: $DB_PATH"
    exit 1
fi
EOF

# Make executable
sudo chmod +x /usr/local/bin/orkee-backup.sh

# Add to crontab (daily at 2 AM)
sudo crontab -e
# Add: 0 2 * * * /usr/local/bin/orkee-backup.sh >> /var/log/orkee-backup.log 2>&1
```

### Restore from Backup

```bash
# Stop service
sudo systemctl stop orkee

# Restore database
sudo -u orkee cp /var/backups/orkee/orkee_backup_YYYYMMDD_HHMMSS.db.gz /tmp/
sudo -u orkee gunzip /tmp/orkee_backup_YYYYMMDD_HHMMSS.db.gz
sudo -u orkee cp /tmp/orkee_backup_YYYYMMDD_HHMMSS.db /var/lib/orkee/data/orkee.db

# Start service
sudo systemctl start orkee

# Verify
curl http://localhost:4001/api/health
```

## Troubleshooting

### Service Won't Start

```bash
# Check service status
sudo systemctl status orkee

# View detailed logs
sudo journalctl -u orkee -n 100 --no-pager

# Check file permissions
sudo ls -la /var/lib/orkee/
sudo ls -la /etc/orkee/

# Test binary manually
sudo -u orkee /usr/local/bin/orkee --version
sudo -u orkee /usr/local/bin/orkee dashboard --help

# Check port availability
sudo ss -tlnp | grep 4001
```

### Port Already in Use

```bash
# Find process using port
sudo lsof -i :4001
sudo fuser 4001/tcp

# Kill process
sudo kill <PID>

# Or change port in config
sudo vim /etc/orkee/production.env
# Change ORKEE_API_PORT=4001 to another port
```

### Permission Denied Errors

```bash
# Fix ownership
sudo chown -R orkee:orkee /var/lib/orkee
sudo chown orkee:orkee /etc/orkee/production.env

# Fix permissions
sudo chmod 750 /var/lib/orkee
sudo chmod 600 /etc/orkee/production.env

# Restart service
sudo systemctl restart orkee
```

### Database Issues

```bash
# Check database file
sudo -u orkee ls -lh /var/lib/orkee/data/orkee.db

# Test database
sudo -u orkee sqlite3 /var/lib/orkee/data/orkee.db "SELECT COUNT(*) FROM projects;"

# Restore from backup if corrupted
sudo systemctl stop orkee
sudo -u orkee cp /var/backups/orkee/orkee_backup_LATEST.db.gz /tmp/
sudo -u orkee gunzip /tmp/orkee_backup_LATEST.db.gz
sudo -u orkee mv /tmp/orkee_backup_LATEST.db /var/lib/orkee/data/orkee.db
sudo systemctl start orkee
```

## Updating Orkee

```bash
# Stop service
sudo systemctl stop orkee

# Backup current binary
sudo cp /usr/local/bin/orkee /usr/local/bin/orkee.backup

# Build new version
cd ~/orkee
git pull origin main
pnpm install
turbo build
cd packages/cli
cargo build --release

# Install new binary
sudo cp target/release/orkee /usr/local/bin/
sudo chmod 755 /usr/local/bin/orkee

# Restart service
sudo systemctl start orkee

# Verify
orkee --version
sudo systemctl status orkee
curl http://localhost:4001/api/health
```

## Next Steps

- [TLS/HTTPS Configuration](tls-https) - Set up SSL certificates
- [Reverse Proxy Setup](reverse-proxy) - Deploy behind Nginx
- [Docker Deployment](docker) - Container-based deployment
- [Security Settings](../configuration/security-settings) - Additional security hardening
- [Monitoring Guide](../operations/monitoring) - Advanced monitoring setup
