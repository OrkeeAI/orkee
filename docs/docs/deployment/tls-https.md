---
sidebar_position: 3
title: TLS/HTTPS Configuration
---

# TLS/HTTPS Configuration

Configure TLS/SSL certificates for secure HTTPS connections to your Orkee deployment.

## Overview

Orkee supports multiple TLS configuration options:

- **Let's Encrypt** - Free automated certificates (recommended)
- **Commercial Certificates** - Purchased SSL certificates
- **Self-Signed Certificates** - Development/testing only
- **Auto-Generated Certificates** - Built-in development certificates

## TLS Configuration Options

### Option A: Let's Encrypt (Recommended for Production)

Let's Encrypt provides free, automated SSL certificates trusted by all major browsers.

#### Prerequisites

- Domain name pointing to your server
- Ports 80 and 443 accessible
- Server with root access

#### Installation

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install certbot

# CentOS/RHEL
sudo yum install certbot

# macOS
brew install certbot
```

#### Obtain Certificate

```bash
# Standalone mode (stops services on port 80)
sudo certbot certonly --standalone -d your-domain.com

# Or specify multiple domains
sudo certbot certonly --standalone \
  -d your-domain.com \
  -d www.your-domain.com
```

**Certificate locations:**
```
Certificate: /etc/letsencrypt/live/your-domain.com/fullchain.pem
Private Key: /etc/letsencrypt/live/your-domain.com/privkey.pem
Chain:       /etc/letsencrypt/live/your-domain.com/chain.pem
Full Chain:  /etc/letsencrypt/live/your-domain.com/fullchain.pem
```

#### Configure Orkee for Let's Encrypt

Update your environment configuration:

```bash
# /etc/orkee/production.env
TLS_ENABLED=true
TLS_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem
TLS_KEY_PATH=/etc/letsencrypt/live/your-domain.com/privkey.pem
AUTO_GENERATE_CERT=false

# Security settings
ENABLE_HSTS=true
SECURITY_HEADERS_ENABLED=true

# Bind to HTTPS port
ORKEE_API_PORT=443
ORKEE_HOST=0.0.0.0

# Set your domain
CORS_ORIGIN=https://your-domain.com
```

**Set file permissions:**

```bash
# Grant Orkee user access to certificates
sudo chown -R orkee:orkee /etc/letsencrypt/
sudo chmod -R 755 /etc/letsencrypt/live/
sudo chmod -R 755 /etc/letsencrypt/archive/
```

#### Automatic Certificate Renewal

Let's Encrypt certificates expire after 90 days. Set up automatic renewal:

```bash
# Test renewal (dry run)
sudo certbot renew --dry-run

# Add to crontab for automatic renewal
sudo crontab -e

# Add this line to renew twice daily and reload Orkee
0 0,12 * * * /usr/bin/certbot renew --quiet --post-hook "systemctl reload orkee"
```

**Alternative: Using systemd timer**

```bash
# Enable certbot renewal timer
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer

# Check timer status
sudo systemctl status certbot.timer
```

### Option B: Commercial SSL Certificate

Use purchased certificates from providers like DigiCert, Comodo, or GoDaddy.

#### Installation

```bash
# Create certificate directory
sudo mkdir -p /var/lib/orkee/certs
sudo chown orkee:orkee /var/lib/orkee/certs
sudo chmod 755 /var/lib/orkee/certs

# Copy your certificates
sudo cp your-certificate.crt /var/lib/orkee/certs/cert.pem
sudo cp your-private-key.key /var/lib/orkee/certs/key.pem
sudo cp your-ca-bundle.crt /var/lib/orkee/certs/chain.pem

# If you need to combine certificate with CA bundle
cat your-certificate.crt your-ca-bundle.crt > /var/lib/orkee/certs/fullchain.pem

# Set proper permissions
sudo chown orkee:orkee /var/lib/orkee/certs/*.pem
sudo chmod 600 /var/lib/orkee/certs/key.pem
sudo chmod 644 /var/lib/orkee/certs/*.pem
```

#### Configure Orkee

```bash
# /etc/orkee/production.env
TLS_ENABLED=true
TLS_CERT_PATH=/var/lib/orkee/certs/fullchain.pem
TLS_KEY_PATH=/var/lib/orkee/certs/key.pem
AUTO_GENERATE_CERT=false

ENABLE_HSTS=true
SECURITY_HEADERS_ENABLED=true
ORKEE_API_PORT=443
```

### Option C: Self-Signed Certificate (Development Only)

For local development or testing. **Not recommended for production.**

#### Generate Self-Signed Certificate

```bash
# Create certificate directory
mkdir -p ~/.orkee/certs

# Generate certificate (valid for 365 days)
openssl req -x509 -newkey rsa:4096 \
  -keyout ~/.orkee/certs/key.pem \
  -out ~/.orkee/certs/cert.pem \
  -days 365 -nodes \
  -subj "/CN=localhost"

# Or with interactive prompts
openssl req -x509 -newkey rsa:4096 \
  -keyout ~/.orkee/certs/key.pem \
  -out ~/.orkee/certs/cert.pem \
  -days 365 -nodes

# Set permissions
chmod 600 ~/.orkee/certs/key.pem
chmod 644 ~/.orkee/certs/cert.pem
```

#### Configure Orkee

```bash
# Development environment
TLS_ENABLED=true
TLS_CERT_PATH=~/.orkee/certs/cert.pem
TLS_KEY_PATH=~/.orkee/certs/key.pem
AUTO_GENERATE_CERT=false

# Development ports
ORKEE_API_PORT=4443  # Use non-privileged port
ORKEE_HOST=127.0.0.1
```

#### Trust Self-Signed Certificate

**macOS:**
```bash
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain \
  ~/.orkee/certs/cert.pem
```

**Linux (Ubuntu/Debian):**
```bash
sudo cp ~/.orkee/certs/cert.pem /usr/local/share/ca-certificates/orkee.crt
sudo update-ca-certificates
```

**Browser:** Accept the security warning or import the certificate.

### Option D: Auto-Generated Certificate (Development Only)

Orkee can automatically generate self-signed certificates for development.

```bash
# Enable auto-generation
TLS_ENABLED=true
AUTO_GENERATE_CERT=true
TLS_CERT_PATH=~/.orkee/certs/cert.pem
TLS_KEY_PATH=~/.orkee/certs/key.pem

# Orkee will generate certificates on first run
orkee dashboard
```

## TLS Configuration Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TLS_ENABLED` | `false` | Enable HTTPS |
| `TLS_CERT_PATH` | `~/.orkee/certs/cert.pem` | Certificate file path |
| `TLS_KEY_PATH` | `~/.orkee/certs/key.pem` | Private key file path |
| `AUTO_GENERATE_CERT` | `true` | Auto-generate dev certificates |
| `ENABLE_HSTS` | `false` | Enable HTTP Strict Transport Security |
| `ORKEE_API_PORT` | `4001` | Port to bind (443 for HTTPS) |
| `ORKEE_HOST` | `localhost` | Interface to bind |

## Security Best Practices

### 1. Strong TLS Configuration

Orkee uses secure TLS defaults:

- **Protocols**: TLS 1.2 and TLS 1.3 only
- **Ciphers**: Modern, secure cipher suites
- **Key Exchange**: ECDHE for forward secrecy

**TLS configuration is handled automatically by rustls.**

### 2. Enable HSTS

HTTP Strict Transport Security forces browsers to use HTTPS:

```bash
ENABLE_HSTS=true
```

This adds the header:
```
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

### 3. Certificate Permissions

```bash
# Certificate should be readable by orkee user
sudo chown orkee:orkee /path/to/cert.pem
sudo chmod 644 /path/to/cert.pem

# Private key should be secure
sudo chown orkee:orkee /path/to/key.pem
sudo chmod 600 /path/to/key.pem
```

### 4. Redirect HTTP to HTTPS

When using direct TLS (not behind proxy), Orkee automatically redirects HTTP to HTTPS if both ports are configured.

### 5. Regular Certificate Updates

```bash
# Monitor certificate expiration
openssl x509 -in /path/to/cert.pem -noout -dates

# Check days until expiration
openssl x509 -in /path/to/cert.pem -noout -checkend 2592000

# Set up monitoring alerts
echo '0 0 * * * openssl x509 -in /path/to/cert.pem -noout -checkend 2592000 || echo "Certificate expires soon!" | mail -s "SSL Alert" admin@example.com"' | sudo crontab -e
```

## Testing TLS Configuration

### Verify Certificate

```bash
# Check certificate details
openssl x509 -in /path/to/cert.pem -text -noout

# Verify certificate dates
openssl x509 -in /path/to/cert.pem -noout -dates

# Check certificate chain
openssl verify -CAfile /path/to/chain.pem /path/to/cert.pem
```

### Test HTTPS Connection

```bash
# Test local HTTPS
curl https://localhost:443/api/health --insecure

# Test with certificate validation
curl https://your-domain.com/api/health

# Test TLS handshake
openssl s_client -connect your-domain.com:443 -servername your-domain.com

# Check specific TLS version
openssl s_client -connect your-domain.com:443 -tls1_2
openssl s_client -connect your-domain.com:443 -tls1_3
```

### Online SSL Testing

Use online tools to verify your configuration:

- **SSL Labs**: https://www.ssllabs.com/ssltest/
- **SSL Checker**: https://www.sslshopper.com/ssl-checker.html
- **Security Headers**: https://securityheaders.com/

## Troubleshooting

### Certificate Permission Errors

```bash
Error: Permission denied reading certificate

# Fix permissions
sudo chown orkee:orkee /path/to/cert.pem /path/to/key.pem
sudo chmod 644 /path/to/cert.pem
sudo chmod 600 /path/to/key.pem

# Verify orkee user can read files
sudo -u orkee cat /path/to/cert.pem
sudo -u orkee cat /path/to/key.pem
```

### Certificate Chain Issues

```bash
Error: Certificate verify failed

# Ensure fullchain is used, not just certificate
TLS_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem

# Or manually create fullchain
cat cert.pem chain.pem > fullchain.pem
```

### Port 443 Binding Issues

```bash
Error: Permission denied binding to port 443

# Option 1: Run as root (not recommended)
sudo orkee dashboard

# Option 2: Use capabilities (recommended)
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/orkee

# Option 3: Use higher port with reverse proxy
ORKEE_API_PORT=4443 orkee dashboard
```

### Mixed Content Warnings

```bash
# Ensure CORS_ORIGIN uses https://
CORS_ORIGIN=https://your-domain.com

# Check all resources load via HTTPS
# Fix any http:// URLs in frontend code
```

### Certificate Expiration

```bash
# Check expiration date
openssl x509 -in /path/to/cert.pem -noout -enddate

# Test renewal (Let's Encrypt)
sudo certbot renew --dry-run

# Force renewal
sudo certbot renew --force-renewal
```

### TLS Handshake Failures

```bash
# Check certificate and key match
openssl x509 -noout -modulus -in cert.pem | openssl md5
openssl rsa -noout -modulus -in key.pem | openssl md5
# MD5 sums should match

# Verify certificate chain order
openssl crl2pkcs7 -nocrl -certfile fullchain.pem | \
  openssl pkcs7 -print_certs -noout
```

## Certificate Management Scripts

### Certificate Expiration Monitor

Create `/usr/local/bin/check-orkee-cert.sh`:

```bash
#!/bin/bash
CERT_PATH="/etc/letsencrypt/live/your-domain.com/fullchain.pem"
DAYS_WARN=30
EMAIL="admin@your-domain.com"

# Check days until expiration
EXPIRY_DATE=$(openssl x509 -in "$CERT_PATH" -noout -enddate | cut -d= -f2)
EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s)
NOW_EPOCH=$(date +%s)
DAYS_LEFT=$(( ($EXPIRY_EPOCH - $NOW_EPOCH) / 86400 ))

if [ $DAYS_LEFT -lt $DAYS_WARN ]; then
    echo "Certificate expires in $DAYS_LEFT days!" | \
        mail -s "SSL Certificate Warning" "$EMAIL"
fi

echo "Certificate expires in $DAYS_LEFT days"
```

**Add to crontab:**
```bash
chmod +x /usr/local/bin/check-orkee-cert.sh
echo "0 9 * * * /usr/local/bin/check-orkee-cert.sh" | sudo crontab -e
```

### Automatic Renewal Script

Create `/usr/local/bin/renew-orkee-cert.sh`:

```bash
#!/bin/bash
certbot renew --quiet

if [ $? -eq 0 ]; then
    systemctl reload orkee
    echo "$(date): Certificate renewed and Orkee reloaded" >> /var/log/orkee-cert-renewal.log
else
    echo "$(date): Certificate renewal failed" >> /var/log/orkee-cert-renewal.log
    mail -s "Certificate Renewal Failed" admin@your-domain.com < /var/log/orkee-cert-renewal.log
fi
```

## Docker TLS Configuration

### Volume Mounts

```yaml
services:
  orkee:
    image: orkee:latest
    volumes:
      - ./certs:/var/lib/orkee/certs:ro
      - ./data:/var/lib/orkee/data
    environment:
      - TLS_ENABLED=true
      - TLS_CERT_PATH=/var/lib/orkee/certs/fullchain.pem
      - TLS_KEY_PATH=/var/lib/orkee/certs/key.pem
      - AUTO_GENERATE_CERT=false
    ports:
      - "443:443"
```

### Let's Encrypt with Docker

```yaml
services:
  orkee:
    image: orkee:latest
    volumes:
      - /etc/letsencrypt:/etc/letsencrypt:ro
      - ./data:/var/lib/orkee/data
    environment:
      - TLS_ENABLED=true
      - TLS_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem
      - TLS_KEY_PATH=/etc/letsencrypt/live/your-domain.com/privkey.pem
    ports:
      - "443:443"

  certbot:
    image: certbot/certbot
    volumes:
      - /etc/letsencrypt:/etc/letsencrypt
      - /var/lib/letsencrypt:/var/lib/letsencrypt
    entrypoint: "/bin/sh -c 'trap exit TERM; while :; do certbot renew; sleep 12h & wait $${!}; done;'"
```

## Next Steps

- [Reverse Proxy Setup](reverse-proxy) - Deploy behind Nginx/Apache
- [Linux Server Deployment](linux-server) - Complete Linux setup
- [Security Settings](../configuration/security-settings) - Additional security hardening
- [Docker Deployment](docker) - Container-based deployment
