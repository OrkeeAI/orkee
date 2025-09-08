# Production Deployment Guide

This guide covers deploying Orkee to production environments with proper security, performance, and reliability configurations.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Quick Start](#quick-start)
4. [Production Environment Setup](#production-environment-setup)
5. [TLS/SSL Configuration](#tlsssl-configuration)
6. [Reverse Proxy Setup](#reverse-proxy-setup)
7. [Docker Deployment](#docker-deployment)
8. [Security Hardening](#security-hardening)
9. [Monitoring & Logging](#monitoring--logging)
10. [Performance Optimization](#performance-optimization)
11. [Troubleshooting](#troubleshooting)

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

## Quick Start

### Using Docker Compose (Recommended)

1. **Copy environment template:**
   ```bash
   cp deployment/examples/.env.production .env
   # Edit .env with your domain and settings
   ```

2. **Deploy with Docker Compose:**
   ```bash
   docker-compose -f deployment/docker/docker-compose.yml up -d
   ```

3. **Configure Nginx (optional):**
   ```bash
   # Copy nginx configuration
   sudo cp deployment/nginx/orkee-ssl.conf /etc/nginx/sites-available/orkee
   sudo ln -s /etc/nginx/sites-available/orkee /etc/nginx/sites-enabled/
   sudo nginx -t && sudo systemctl reload nginx
   ```

### Manual Installation

Follow the detailed [Production Environment Setup](#production-environment-setup) section below.

## Production Environment Setup

### 1. Build Release Binaries

```bash
# Clone repository
git clone https://github.com/OrkeeAI/orkee.git
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

# Copy production environment template
sudo cp deployment/examples/.env.production /etc/orkee/production.env
sudo chown orkee:orkee /etc/orkee/production.env
sudo chmod 600 /etc/orkee/production.env
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

**Update production environment variables:**
```bash
# Edit /etc/orkee/production.env
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

# Set your domain
CORS_ORIGIN=https://your-domain.com
DOMAIN=your-domain.com
EMAIL=admin@your-domain.com
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

The deployment includes several pre-configured Nginx templates:

- **`nginx/orkee.conf`** - Basic HTTP/HTTPS configuration
- **`nginx/orkee-ssl.conf`** - Advanced SSL configuration with rate limiting
- **`nginx/snippets/ssl-params.conf`** - Reusable SSL security parameters
- **`nginx/snippets/proxy-params.conf`** - Common proxy headers

#### Quick Setup

```bash
# Copy the advanced SSL configuration
sudo cp deployment/nginx/orkee-ssl.conf /etc/nginx/sites-available/orkee
sudo cp -r deployment/nginx/snippets /etc/nginx/

# Edit the configuration for your domain
sudo sed -i 's/your-domain.com/yourdomain.com/g' /etc/nginx/sites-available/orkee

# Enable site
sudo ln -s /etc/nginx/sites-available/orkee /etc/nginx/sites-enabled/

# Test and reload
sudo nginx -t
sudo systemctl reload nginx
```

#### Custom Configuration

For custom setups, see the template files:
- [`nginx/orkee-ssl.conf`](nginx/orkee-ssl.conf) - Full production configuration
- [`nginx/snippets/ssl-params.conf`](nginx/snippets/ssl-params.conf) - SSL security settings

**Orkee configuration behind proxy:**
```bash
# /etc/orkee/production.env (with Nginx proxy)
TLS_ENABLED=false  # Nginx handles TLS
PORT=4001
CORS_ORIGIN=https://your-domain.com
SECURITY_HEADERS_ENABLED=false  # Nginx provides headers
```

## Docker Deployment

### Using Pre-built Docker Configurations

The deployment includes production-ready Docker configurations:

- **`docker/Dockerfile`** - Multi-stage production build
- **`docker/Dockerfile.dev`** - Development container with hot reload
- **`docker/docker-compose.yml`** - Production orchestration
- **`docker/docker-compose.dev.yml`** - Development orchestration
- **`docker/docker-entrypoint.sh`** - Container initialization script

#### Production Deployment

```bash
# Copy and customize environment
cp deployment/examples/.env.docker .env
# Edit .env with your settings (domain, ports, etc.)

# Deploy production stack
docker-compose -f deployment/docker/docker-compose.yml up -d

# View logs
docker-compose -f deployment/docker/docker-compose.yml logs -f

# Update deployment
docker-compose -f deployment/docker/docker-compose.yml pull
docker-compose -f deployment/docker/docker-compose.yml up -d
```

#### Development Deployment

```bash
# Copy development environment
cp deployment/examples/.env.development .env

# Start development stack
docker-compose -f deployment/docker/docker-compose.dev.yml up -d

# Access development dashboard at http://localhost:5173
```

#### SSL with Let's Encrypt (Docker)

```bash
# Run certbot to obtain certificates
docker-compose -f deployment/docker/docker-compose.yml --profile tools run --rm certbot

# Update docker-compose.yml to use real certificates
# (Edit nginx service volume mounts)
```

### Manual Docker Build

If you need to customize the Docker build:

```bash
# Build production image
docker build -f deployment/docker/Dockerfile -t orkee:latest .

# Run production container
docker run -d \
  --name orkee-prod \
  --restart unless-stopped \
  -p 4001:4001 -p 4000:4000 \
  -v ./data:/var/lib/orkee/data \
  -v ./certs:/var/lib/orkee/certs \
  -v ./logs:/var/lib/orkee/logs \
  -e TLS_ENABLED=true \
  -e AUTO_GENERATE_CERT=true \
  orkee:latest
```

## Security Hardening

### Systemd Service

Use the systemd service template:

```bash
# Copy and customize the service file
sudo cp deployment/systemd/orkee.service /etc/systemd/system/
sudo systemctl daemon-reload

# Enable and start
sudo systemctl enable orkee
sudo systemctl start orkee
sudo systemctl status orkee
```

The service file includes security hardening:
- Runs as non-root user
- Restricted filesystem access
- Resource limits
- Automatic restart on failure

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

# Set up log rotation
sudo tee /etc/logrotate.d/orkee << 'EOF'
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

### Health Check Monitoring

Create a health check script:

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
chmod +x /usr/local/bin/orkee-healthcheck.sh
# Check every 5 minutes
*/5 * * * * /usr/local/bin/orkee-healthcheck.sh >> /var/log/orkee-monitoring.log 2>&1
```

### Log Configuration

Add to production environment:
```bash
# /etc/orkee/production.env
RUST_LOG=info
LOG_LEVEL=info
ENABLE_REQUEST_ID=true
ENABLE_METRICS=true
```

### Docker Monitoring

```bash
# Monitor container health
docker-compose -f deployment/docker/docker-compose.yml ps

# View container logs
docker-compose -f deployment/docker/docker-compose.yml logs -f orkee

# Monitor resource usage
docker stats orkee-app
```

## Performance Optimization

### Production Environment Variables

```bash
# /etc/orkee/production.env - Performance settings
RATE_LIMIT_GLOBAL_RPM=100
RATE_LIMIT_BURST_SIZE=10

# Directory browsing limits (if needed)
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS=/var/lib/orkee/data

# Resource limits
RUST_LOG=info  # Balanced logging
MAX_REQUEST_SIZE=10485760  # 10MB
REQUEST_TIMEOUT=30
KEEP_ALIVE_TIMEOUT=65
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

For high-availability setups, the Nginx configuration supports multiple backend servers:

```nginx
# In nginx/orkee-ssl.conf
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

**Docker Issues:**
```bash
# Check container status
docker-compose -f deployment/docker/docker-compose.yml ps

# View container logs
docker-compose -f deployment/docker/docker-compose.yml logs orkee

# Check health status
docker inspect --format='{{.State.Health.Status}}' orkee-app
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

# Docker resource usage
docker stats orkee-app
```

### Configuration Validation

```bash
# Test Orkee configuration
orkee --help

# Test Nginx configuration
sudo nginx -t

# Validate Docker Compose
docker-compose -f deployment/docker/docker-compose.yml config
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

## File Reference

### Configuration Templates
- [`examples/.env.production`](examples/.env.production) - Production environment variables
- [`examples/.env.development`](examples/.env.development) - Development environment variables  
- [`examples/.env.docker`](examples/.env.docker) - Docker-specific configuration

### Nginx Configuration
- [`nginx/orkee-ssl.conf`](nginx/orkee-ssl.conf) - Advanced SSL configuration with rate limiting
- [`nginx/orkee.conf`](nginx/orkee.conf) - Basic HTTP/HTTPS configuration
- [`nginx/snippets/ssl-params.conf`](nginx/snippets/ssl-params.conf) - SSL security parameters
- [`nginx/snippets/proxy-params.conf`](nginx/snippets/proxy-params.conf) - Proxy headers

### Docker Configuration
- [`docker/Dockerfile`](docker/Dockerfile) - Multi-stage production build
- [`docker/Dockerfile.dev`](docker/Dockerfile.dev) - Development container
- [`docker/docker-compose.yml`](docker/docker-compose.yml) - Production orchestration
- [`docker/docker-compose.dev.yml`](docker/docker-compose.dev.yml) - Development orchestration
- [`docker/docker-entrypoint.sh`](docker/docker-entrypoint.sh) - Container initialization

### System Service
- [`systemd/orkee.service`](systemd/orkee.service) - Systemd service definition

## Additional Resources

- [DOCS.md](../DOCS.md) - Complete configuration reference
- [SECURITY.md](../SECURITY.md) - Security architecture and threat model
- [TESTING.md](../TESTING.md) - Testing and validation procedures

For support with production deployments, please open an issue with:
- Deployment method (direct/proxy/container)
- Operating system and version
- Error messages and logs
- Configuration (with secrets redacted)