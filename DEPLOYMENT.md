# Production Deployment Guide

This guide covers deploying Orkee to production environments with proper security, performance, and reliability configurations.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Production Environment Setup](#production-environment-setup)
4. [TLS/SSL Configuration](#tlsssl-configuration)
5. [Reverse Proxy Setup](#reverse-proxy-setup)
6. [Docker Deployment](#docker-deployment)
7. [Security Hardening](#security-hardening)
8. [Monitoring & Logging](#monitoring--logging)
9. [Performance Optimization](#performance-optimization)
10. [Troubleshooting](#troubleshooting)

## Overview

Orkee can be deployed in several production configurations:

- **Direct HTTPS** - Orkee handles TLS directly with custom certificates
- **Behind Reverse Proxy** - Nginx/Apache handles TLS, proxies to Orkee HTTP
- **Container Deployment** - Docker/Kubernetes with orchestrated TLS
- **Cloud Deployment** - AWS/GCP/Azure with managed load balancers

## Prerequisites

### System Requirements
- **CPU**: 2+ cores recommended
- **RAM**: 2GB+ for standalone, 4GB+ with dashboard
- **Storage**: 1GB+ (depends on project data)
- **OS**: Linux (Ubuntu/CentOS), macOS, Windows Server

### Software Requirements
- **Rust**: Latest stable (1.70+)
- **Node.js**: v18+ (for dashboard)
- **pnpm**: v8+ (for dashboard builds)
- **systemd**: For service management (Linux)

## Production Environment Setup

### 1. Build Release Binaries

```bash
# Clone repository
git clone https://github.com/yourusername/orkee.git
cd orkee

# Install dependencies
pnpm install

# Build production binaries
turbo build

# Build optimized Rust binary
cd packages/cli
cargo build --release

# Binary location: target/release/orkee
```

### 2. Create Production User

```bash
# Create dedicated user (Linux)
sudo useradd --system --home /var/lib/orkee --shell /bin/false orkee
sudo mkdir -p /var/lib/orkee
sudo chown orkee:orkee /var/lib/orkee
```

### 3. Install Application

```bash
# Copy binary to system location
sudo cp target/release/orkee /usr/local/bin/
sudo chown root:root /usr/local/bin/orkee
sudo chmod 755 /usr/local/bin/orkee

# Create configuration directory
sudo mkdir -p /etc/orkee
sudo chown orkee:orkee /etc/orkee

# Create data directory
sudo mkdir -p /var/lib/orkee/{data,certs,logs}
sudo chown -R orkee:orkee /var/lib/orkee
```

## TLS/SSL Configuration

### Option A: Let's Encrypt (Recommended)

```bash
# Install certbot
sudo apt install certbot  # Ubuntu/Debian
sudo yum install certbot   # CentOS/RHEL

# Obtain certificate (replace your-domain.com)
sudo certbot certonly --standalone -d your-domain.com

# Certificates will be at:
# /etc/letsencrypt/live/your-domain.com/fullchain.pem
# /etc/letsencrypt/live/your-domain.com/privkey.pem
```

**Production Environment Variables:**
```bash
# /etc/orkee/production.env
TLS_ENABLED=true
TLS_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem
TLS_KEY_PATH=/etc/letsencrypt/live/your-domain.com/privkey.pem
AUTO_GENERATE_CERT=false

# Security settings
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
RATE_LIMIT_ENABLED=true

# Bind to production port
PORT=443

# Disable development features
CORS_ALLOW_ANY_LOCALHOST=false
CORS_ORIGIN=https://your-domain.com
```

### Option B: Commercial SSL Certificate

```bash
# Copy your certificates to secure location
sudo cp your-certificate.pem /var/lib/orkee/certs/cert.pem
sudo cp your-private-key.pem /var/lib/orkee/certs/key.pem
sudo chown orkee:orkee /var/lib/orkee/certs/*.pem
sudo chmod 600 /var/lib/orkee/certs/key.pem
sudo chmod 644 /var/lib/orkee/certs/cert.pem
```

**Environment Variables:**
```bash
TLS_ENABLED=true
TLS_CERT_PATH=/var/lib/orkee/certs/cert.pem
TLS_KEY_PATH=/var/lib/orkee/certs/key.pem
AUTO_GENERATE_CERT=false
```

### Certificate Renewal (Let's Encrypt)

```bash
# Test renewal
sudo certbot renew --dry-run

# Add to crontab for automatic renewal
sudo crontab -e
# Add line: 0 12 * * * /usr/bin/certbot renew --quiet --post-hook "systemctl reload orkee"
```

## Reverse Proxy Setup

### Nginx Configuration

Create `/etc/nginx/sites-available/orkee`:

```nginx
upstream orkee_backend {
    server 127.0.0.1:4001;
    # Add multiple servers for load balancing
    # server 127.0.0.1:4002;
    # server 127.0.0.1:4003;
}

# HTTP redirect to HTTPS
server {
    listen 80;
    server_name your-domain.com;
    
    # Let's Encrypt validation
    location /.well-known/acme-challenge/ {
        root /var/www/html;
    }
    
    # Redirect everything else to HTTPS
    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS server
server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    # SSL configuration
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_stapling on;
    ssl_stapling_verify on;
    
    # Modern SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    
    # Security headers (additional to Orkee's built-in headers)
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload";
    add_header X-Robots-Tag "noindex, nofollow, nosnippet, noarchive";
    
    # Proxy to Orkee
    location / {
        proxy_pass http://orkee_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # Buffer settings
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
        proxy_busy_buffers_size 256k;
    }
    
    # Health check endpoint (bypass proxy for monitoring)
    location /health {
        access_log off;
        proxy_pass http://orkee_backend/api/health;
    }
    
    # Rate limiting (additional to Orkee's built-in limiting)
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;
    
    # Logging
    access_log /var/log/nginx/orkee-access.log;
    error_log /var/log/nginx/orkee-error.log warn;
}
```

**Enable and test:**
```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/orkee /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx
```

**Orkee configuration behind proxy:**
```bash
# /etc/orkee/production.env (with Nginx proxy)
TLS_ENABLED=false  # Nginx handles TLS
PORT=4001
CORS_ORIGIN=https://your-domain.com
SECURITY_HEADERS_ENABLED=false  # Nginx provides headers
```

## Docker Deployment

### Dockerfile

Create `packages/cli/Dockerfile`:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/orkee /usr/local/bin/orkee

RUN useradd --system --home /var/lib/orkee --shell /bin/false orkee
RUN mkdir -p /var/lib/orkee/{data,certs} && chown -R orkee:orkee /var/lib/orkee

USER orkee
WORKDIR /var/lib/orkee

EXPOSE 4001 4000

ENV RUST_LOG=info

CMD ["orkee", "dashboard"]
```

### Docker Compose

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  orkee:
    build:
      context: ./packages/cli
      dockerfile: Dockerfile
    container_name: orkee-prod
    restart: unless-stopped
    
    ports:
      - "4001:4001"
      - "4000:4000"
    
    environment:
      - TLS_ENABLED=true
      - AUTO_GENERATE_CERT=true
      - PORT=4001
      - SECURITY_HEADERS_ENABLED=true
      - ENABLE_HSTS=true
      - RATE_LIMIT_ENABLED=true
      - RUST_LOG=info
      
    volumes:
      - ./data:/var/lib/orkee/data
      - ./certs:/var/lib/orkee/certs
      - ./logs:/var/lib/orkee/logs
    
    networks:
      - orkee-network
    
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4001/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3

networks:
  orkee-network:
    driver: bridge
```

**Deploy:**
```bash
# Build and start
docker-compose -f docker-compose.prod.yml up -d

# View logs
docker-compose -f docker-compose.prod.yml logs -f

# Update deployment
docker-compose -f docker-compose.prod.yml pull && \
docker-compose -f docker-compose.prod.yml up -d
```

## Security Hardening

### Systemd Service

Create `/etc/systemd/system/orkee.service`:

```ini
[Unit]
Description=Orkee AI Agent Orchestration
After=network.target
Wants=network.target

[Service]
Type=simple
User=orkee
Group=orkee
WorkingDirectory=/var/lib/orkee
ExecStart=/usr/local/bin/orkee dashboard
EnvironmentFile=/etc/orkee/production.env
Restart=always
RestartSec=10

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/orkee
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096
MemoryLimit=1G

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=orkee

[Install]
WantedBy=multi-user.target
```

**Enable and start:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable orkee
sudo systemctl start orkee
sudo systemctl status orkee
```

### Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP (for redirects)
sudo ufw allow 443/tcp   # HTTPS
sudo ufw enable

# iptables (CentOS/RHEL)
sudo firewall-cmd --permanent --add-service=ssh
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --reload
```

### File Permissions

```bash
# Secure configuration files
sudo chmod 600 /etc/orkee/production.env
sudo chown orkee:orkee /etc/orkee/production.env

# Secure certificate files
sudo chmod 600 /var/lib/orkee/certs/*.pem
sudo chown orkee:orkee /var/lib/orkee/certs/*.pem

# Log rotation
sudo tee /etc/logrotate.d/orkee << EOF
/var/lib/orkee/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    copytruncate
    postrotate
        systemctl reload orkee
    endscript
}
EOF
```

## Monitoring & Logging

### Log Configuration

Add to production environment:
```bash
# /etc/orkee/production.env
RUST_LOG=info
ENABLE_REQUEST_ID=true
```

### Health Check Monitoring

```bash
#!/bin/bash
# /usr/local/bin/orkee-healthcheck.sh

HEALTH_URL="https://your-domain.com/api/health"
MAX_RETRIES=3
RETRY_DELAY=5

for i in $(seq 1 $MAX_RETRIES); do
    if curl -f -s -o /dev/null --connect-timeout 10 "$HEALTH_URL"; then
        echo "$(date): Orkee health check passed"
        exit 0
    fi
    
    echo "$(date): Orkee health check failed (attempt $i/$MAX_RETRIES)"
    sleep $RETRY_DELAY
done

echo "$(date): Orkee health check failed after $MAX_RETRIES attempts"
systemctl restart orkee
exit 1
```

**Add to cron:**
```bash
# Check every 5 minutes
*/5 * * * * /usr/local/bin/orkee-healthcheck.sh >> /var/log/orkee-monitoring.log 2>&1
```

### Prometheus Metrics (Optional)

If you want to add metrics endpoint:

```bash
# Add to production.env
METRICS_ENABLED=true
METRICS_PORT=9090
```

## Performance Optimization

### Production Environment Variables

```bash
# /etc/orkee/production.env - Performance settings
RATE_LIMIT_GLOBAL_RPM=100
RATE_LIMIT_BURST_SIZE=10

# Directory browsing limits (if needed)
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS=/var/www,/opt/data

# Resource limits
RUST_LOG=warn  # Reduce log verbosity in production
```

### System Tuning

```bash
# Increase file descriptor limits
echo "orkee soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "orkee hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Network tuning for high concurrent connections
echo 'net.core.somaxconn = 1024' | sudo tee -a /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 1024' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Load Balancing

For high-availability setups, run multiple Orkee instances:

```bash
# Start multiple instances on different ports
PORT=4001 orkee dashboard &
PORT=4002 orkee dashboard &
PORT=4003 orkee dashboard &

# Update Nginx upstream configuration:
upstream orkee_backend {
    server 127.0.0.1:4001;
    server 127.0.0.1:4002;
    server 127.0.0.1:4003;
    
    # Health checks and load balancing
    least_conn;
    keepalive 32;
}
```

## Troubleshooting

### Common Issues

**Certificate Issues:**
```bash
# Check certificate validity
openssl x509 -in /path/to/cert.pem -text -noout

# Test TLS connection
openssl s_client -connect your-domain.com:443

# Check certificate permissions
ls -la /var/lib/orkee/certs/
```

**Service Issues:**
```bash
# Check service status
sudo systemctl status orkee

# View logs
sudo journalctl -u orkee -f

# Check file permissions
sudo -u orkee ls -la /var/lib/orkee/
```

**Network Issues:**
```bash
# Check port binding
sudo netstat -tlnp | grep 4001

# Test local connection
curl -k https://localhost:4001/api/health

# Check firewall
sudo ufw status
```

### Performance Issues

```bash
# Monitor resource usage
htop
iotop
sudo ss -tulpn | grep orkee

# Check file descriptor usage
lsof -u orkee | wc -l
```

### Security Audit

```bash
# Check running processes
ps aux | grep orkee

# Verify file permissions
find /var/lib/orkee -type f -exec ls -la {} \;

# Check open ports
nmap -sT localhost

# Review logs for security events
grep -i "rate limit\|security\|error" /var/log/syslog
```

---

## Additional Resources

- [DOCS.md](DOCS.md) - Complete configuration reference
- [SECURITY.md](SECURITY.md) - Security architecture and threat model
- [TESTING.md](TESTING.md) - Testing and validation procedures

For support with production deployments, please open an issue with:
- Deployment method (direct/proxy/container)
- Operating system and version
- Error messages and logs
- Configuration (with secrets redacted)