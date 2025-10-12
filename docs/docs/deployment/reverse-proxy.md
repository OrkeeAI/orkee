---
sidebar_position: 2
title: Reverse Proxy Setup
---

# Reverse Proxy Setup

Deploy Orkee behind a reverse proxy like Nginx or Apache for enhanced security, load balancing, and TLS termination.

## Overview

Running Orkee behind a reverse proxy provides several advantages:

- **TLS Termination**: Proxy handles HTTPS, Orkee runs on HTTP
- **Load Balancing**: Distribute traffic across multiple Orkee instances
- **Rate Limiting**: Additional protection layer
- **Static Content**: Serve assets without touching Orkee
- **DDoS Protection**: Proxy absorbs attacks before reaching Orkee
- **Centralized Logging**: Single point for access logs

## Architecture

```
Internet → Reverse Proxy (HTTPS:443) → Orkee (HTTP:4001)
           ├─ TLS/SSL
           ├─ Rate limiting
           ├─ Security headers
           └─ Request logging
```

## Nginx Configuration

### Quick Setup

The deployment includes pre-configured Nginx templates:

- **`nginx/orkee.conf`** - Basic HTTP/HTTPS configuration
- **`nginx/orkee-ssl.conf`** - Advanced SSL configuration with rate limiting
- **`nginx/snippets/ssl-params.conf`** - Reusable SSL security parameters
- **`nginx/snippets/proxy-params.conf`** - Common proxy headers

**Installation:**

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

### Basic Configuration

Create `/etc/nginx/sites-available/orkee`:

```nginx
# Redirect HTTP to HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name your-domain.com;

    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS server
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name your-domain.com;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Proxy configuration
    location / {
        proxy_pass http://127.0.0.1:4001;
        proxy_http_version 1.1;

        # Proxy headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check endpoint
    location /api/health {
        proxy_pass http://127.0.0.1:4001/api/health;
        access_log off;
    }
}
```

### Advanced Configuration with Rate Limiting

```nginx
# Rate limiting zones
limit_req_zone $binary_remote_addr zone=general:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=api:10m rate=30r/m;
limit_req_zone $binary_remote_addr zone=health:10m rate=60r/m;

# Upstream configuration (for load balancing)
upstream orkee_backend {
    server 127.0.0.1:4001;
    # Add more servers for load balancing:
    # server 127.0.0.1:4002;
    # server 127.0.0.1:4003;

    least_conn;      # Load balancing algorithm
    keepalive 32;    # Keep connections alive
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name your-domain.com;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    include /etc/nginx/snippets/ssl-params.conf;

    # Logging
    access_log /var/log/nginx/orkee-access.log combined;
    error_log /var/log/nginx/orkee-error.log warn;

    # General locations with rate limiting
    location / {
        limit_req zone=general burst=20 nodelay;

        proxy_pass http://orkee_backend;
        include /etc/nginx/snippets/proxy-params.conf;
    }

    # API endpoints with stricter rate limiting
    location /api/ {
        limit_req zone=api burst=10 nodelay;

        proxy_pass http://orkee_backend;
        include /etc/nginx/snippets/proxy-params.conf;
    }

    # Health check with relaxed rate limiting
    location /api/health {
        limit_req zone=health burst=20 nodelay;

        proxy_pass http://orkee_backend;
        access_log off;
    }

    # Static files (if serving from proxy)
    location /static/ {
        alias /var/www/orkee/static/;
        expires 7d;
        add_header Cache-Control "public, immutable";
    }

    # Favicon
    location = /favicon.ico {
        access_log off;
        log_not_found off;
    }
}
```

### SSL Parameters Snippet

Create `/etc/nginx/snippets/ssl-params.conf`:

```nginx
# SSL/TLS Configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_prefer_server_ciphers on;
ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;

# SSL session cache
ssl_session_cache shared:SSL:10m;
ssl_session_timeout 10m;
ssl_session_tickets off;

# OCSP stapling
ssl_stapling on;
ssl_stapling_verify on;
resolver 8.8.8.8 8.8.4.4 valid=300s;
resolver_timeout 5s;

# Security headers
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
add_header X-Frame-Options "DENY" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
```

### Proxy Parameters Snippet

Create `/etc/nginx/snippets/proxy-params.conf`:

```nginx
# Proxy headers
proxy_set_header Host $host;
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;
proxy_set_header X-Forwarded-Host $host;

# Proxy settings
proxy_http_version 1.1;
proxy_set_header Connection "";
proxy_buffering off;
proxy_request_buffering off;

# Timeouts
proxy_connect_timeout 60s;
proxy_send_timeout 60s;
proxy_read_timeout 60s;

# Keep-alive
proxy_set_header Connection "keep-alive";
```

## Orkee Configuration Behind Proxy

Update Orkee environment variables for proxy mode:

```bash
# /etc/orkee/production.env
TLS_ENABLED=false              # Nginx handles TLS
ORKEE_API_PORT=4001           # Backend port
ORKEE_HOST=127.0.0.1          # Bind to localhost only
CORS_ORIGIN=https://your-domain.com
SECURITY_HEADERS_ENABLED=false # Nginx provides headers
RATE_LIMIT_ENABLED=true       # Additional layer (optional)
```

## Load Balancing

For high-availability deployments with multiple Orkee instances:

### Nginx Upstream Configuration

```nginx
upstream orkee_backend {
    # Multiple backend servers
    server 127.0.0.1:4001 weight=3;
    server 127.0.0.1:4002 weight=3;
    server 127.0.0.1:4003 weight=1;  # Backup server

    # Load balancing method
    least_conn;           # Use least connections algorithm
    # Alternative methods:
    # ip_hash;            # Session persistence by IP
    # hash $request_uri;  # URL-based distribution

    # Health checks (requires nginx-plus or third-party module)
    # max_fails=3;
    # fail_timeout=30s;

    # Connection pool
    keepalive 32;
    keepalive_timeout 65s;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    location / {
        proxy_pass http://orkee_backend;
        include /etc/nginx/snippets/proxy-params.conf;
    }
}
```

### Running Multiple Orkee Instances

```bash
# Start multiple instances on different ports
ORKEE_API_PORT=4001 orkee dashboard &
ORKEE_API_PORT=4002 orkee dashboard &
ORKEE_API_PORT=4003 orkee dashboard &

# Or use systemd services
sudo systemctl start orkee@4001
sudo systemctl start orkee@4002
sudo systemctl start orkee@4003
```

## Apache Configuration

Alternative to Nginx using Apache as reverse proxy:

```apache
<VirtualHost *:80>
    ServerName your-domain.com
    Redirect permanent / https://your-domain.com/
</VirtualHost>

<VirtualHost *:443>
    ServerName your-domain.com

    # SSL Configuration
    SSLEngine on
    SSLCertificateFile /etc/letsencrypt/live/your-domain.com/fullchain.pem
    SSLCertificateKeyFile /etc/letsencrypt/live/your-domain.com/privkey.pem
    SSLProtocol all -SSLv3 -TLSv1 -TLSv1.1
    SSLCipherSuite HIGH:!aNULL:!MD5

    # Security headers
    Header always set Strict-Transport-Security "max-age=31536000"
    Header always set X-Frame-Options "DENY"
    Header always set X-Content-Type-Options "nosniff"

    # Reverse proxy
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:4001/
    ProxyPassReverse / http://127.0.0.1:4001/

    # Proxy headers
    RequestHeader set X-Forwarded-Proto "https"
    RequestHeader set X-Forwarded-Port "443"

    # Logging
    ErrorLog ${APACHE_LOG_DIR}/orkee-error.log
    CustomLog ${APACHE_LOG_DIR}/orkee-access.log combined
</VirtualHost>
```

**Enable required modules:**

```bash
sudo a2enmod proxy proxy_http ssl headers
sudo systemctl restart apache2
```

## Monitoring & Troubleshooting

### Test Proxy Configuration

```bash
# Test Nginx configuration
sudo nginx -t

# Reload without downtime
sudo systemctl reload nginx

# Test Apache configuration
sudo apachectl configtest
sudo systemctl reload apache2
```

### Check Backend Connectivity

```bash
# Test Orkee directly
curl http://localhost:4001/api/health

# Test through proxy
curl https://your-domain.com/api/health

# Check response headers
curl -I https://your-domain.com
```

### View Access Logs

```bash
# Nginx logs
sudo tail -f /var/log/nginx/orkee-access.log
sudo tail -f /var/log/nginx/orkee-error.log

# Apache logs
sudo tail -f /var/log/apache2/orkee-access.log
sudo tail -f /var/log/apache2/orkee-error.log
```

### Monitor Performance

```bash
# Check connection stats (Nginx)
sudo nginx -T | grep 'worker_connections'
sudo ss -ant | grep :443 | wc -l

# Check upstream health
curl http://127.0.0.1:4001/api/health
```

## Security Best Practices

### 1. Limit Proxy Access

Only allow proxy to access Orkee backend:

```bash
# Firewall rules (ufw)
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw deny 4001/tcp  # Block direct backend access

# iptables
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 4001 -s 127.0.0.1 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 4001 -j DROP
```

### 2. Rate Limiting

Configure aggressive rate limiting at proxy level:

```nginx
# Nginx rate limiting
limit_req_zone $binary_remote_addr zone=api:10m rate=30r/m;
limit_req_status 429;

location /api/ {
    limit_req zone=api burst=10 nodelay;
    limit_req_log_level warn;
    proxy_pass http://orkee_backend;
}
```

### 3. Request Size Limits

```nginx
# Nginx
client_max_body_size 10M;
client_body_buffer_size 128k;

# Apache
LimitRequestBody 10485760
```

### 4. IP Whitelisting (if applicable)

```nginx
# Nginx - Allow specific IPs only
geo $allowed_ip {
    default 0;
    192.168.1.0/24 1;
    10.0.0.0/8 1;
}

server {
    location / {
        if ($allowed_ip = 0) {
            return 403;
        }
        proxy_pass http://orkee_backend;
    }
}
```

## Common Issues

### 502 Bad Gateway

**Cause**: Proxy cannot reach Orkee backend

**Solution**:
```bash
# Check Orkee is running
systemctl status orkee
curl http://localhost:4001/api/health

# Check SELinux (CentOS/RHEL)
sudo setsebool -P httpd_can_network_connect 1

# Check firewall
sudo ufw status
```

### 504 Gateway Timeout

**Cause**: Backend took too long to respond

**Solution**: Increase timeouts in Nginx:
```nginx
proxy_connect_timeout 120s;
proxy_send_timeout 120s;
proxy_read_timeout 120s;
```

### SSL Certificate Errors

```bash
# Check certificate validity
openssl x509 -in /etc/letsencrypt/live/your-domain.com/fullchain.pem -text -noout

# Test SSL configuration
openssl s_client -connect your-domain.com:443 -servername your-domain.com
```

## Next Steps

- [TLS/HTTPS Configuration](tls-https) - SSL certificate setup
- [Linux Server Deployment](linux-server) - Complete Linux deployment
- [Docker Deployment](docker) - Container-based deployment
- [Security Settings](../configuration/security-settings) - Security hardening
