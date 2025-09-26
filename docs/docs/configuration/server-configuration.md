---
sidebar_position: 2
---

# Server Configuration

This guide covers advanced server configuration options for Orkee, including port management, CORS settings, and performance tuning.

## Port Management

Orkee uses a simple two-port architecture for maximum flexibility:

### Default Ports

- **API Server**: `4001` (configurable via `ORKEE_API_PORT`)
- **Dashboard UI**: `5173` (configurable via `ORKEE_UI_PORT`)

### Port Configuration Methods

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="cli" label="Command Line Flags" default>

```bash
# Override ports with command line flags
orkee dashboard --api-port 8080 --ui-port 3000

# Enable development mode with custom ports
orkee dashboard --dev --api-port 8080 --ui-port 3000
```

</TabItem>
<TabItem value="env" label="Environment Variables">

```bash
# Set via environment variables
ORKEE_API_PORT=8080 ORKEE_UI_PORT=3000 orkee dashboard

# Enable development mode via environment
ORKEE_DEV_MODE=true ORKEE_API_PORT=8080 ORKEE_UI_PORT=3000 orkee dashboard
```

</TabItem>
<TabItem value="file" label=".env File">

```bash
# Create .env file
echo "ORKEE_API_PORT=8080" > .env
echo "ORKEE_UI_PORT=3000" >> .env
echo "ORKEE_DEV_MODE=true" >> .env
orkee dashboard
```

</TabItem>
</Tabs>

### Port Precedence

Configuration follows this priority order (highest to lowest):

1. **Command line flags** (highest)
2. **Environment variables**
3. **`.env` file**
4. **Default values** (lowest)

## CORS Configuration

Cross-Origin Resource Sharing (CORS) is automatically configured for security and development convenience.

### Automatic CORS Setup

Orkee automatically configures CORS based on your UI port:

```bash
# These configurations are equivalent:
ORKEE_UI_PORT=5173  # CORS automatically allows http://localhost:5173
ORKEE_CORS_ORIGIN=http://localhost:5173  # Manual override
```

### CORS Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_CORS_ORIGIN` | Auto-calculated | Specific allowed origin |
| `CORS_ALLOW_ANY_LOCALHOST` | `true` | Allow any localhost port in development |

### Development vs Production CORS

<Tabs>
<TabItem value="dev" label="Development" default>

```bash
# Relaxed CORS for development
CORS_ALLOW_ANY_LOCALHOST=true
ORKEE_UI_PORT=5173  # Auto-allows localhost:5173
```

**Allowed Origins:**
- Any `localhost` port between 3000-8999
- IPv6 localhost (`::1`)
- `127.0.0.1`

</TabItem>
<TabItem value="prod" label="Production">

```bash
# Strict CORS for production
CORS_ALLOW_ANY_LOCALHOST=false
ORKEE_CORS_ORIGIN=https://your-domain.com:5173
```

**Security Features:**
- Only explicitly allowed origins
- HTTPS enforcement recommended
- No wildcard origins

</TabItem>
</Tabs>

### CORS Security Features

- **Port Whitelist**: Only development ports (3000-8999) allowed
- **Method Restrictions**: Limited to `GET, POST, PUT, DELETE, OPTIONS`
- **Header Restrictions**: Only essential headers permitted
- **No Credentials**: Cookies/auth headers blocked by default

## Performance Configuration

### Connection Settings

Configure server performance based on your environment:

```bash
# High-traffic settings
ORKEE_API_PORT=4001
CORS_ALLOW_ANY_LOCALHOST=false

# Development settings (more permissive)
ORKEE_API_PORT=4001
CORS_ALLOW_ANY_LOCALHOST=true
```

### Resource Limits

Orkee implements several built-in resource limits:

- **Request Size**: 10MB maximum payload
- **Concurrent Connections**: OS-dependent (typically 1000+)
- **Request Timeout**: 30 seconds for most operations
- **Directory Listing**: 1000 files maximum per request

## Multi-Interface Architecture

Orkee supports multiple interfaces with different configuration needs:

### Dashboard Mode (Default)

Runs both API server and web interface:

```bash
orkee dashboard --api-port 4001 --ui-port 5173
```

**Services Started:**
- Rust API server (port 4001)
- Vite dev server (port 5173)
- Automatic browser opening

### API-Only Mode

Run only the API server without the web interface:

```bash
orkee api --port 4001
```

**Use Cases:**
- CI/CD integration
- Headless environments
- Custom frontend development

### TUI Mode

Terminal User Interface with no network ports:

```bash
orkee tui
```

**Features:**
- No HTTP server required
- Direct SQLite access
- Keyboard-driven interface

## Server Startup Options

### Restart Behavior

Control how Orkee handles process management:

```bash
# Restart existing processes
orkee dashboard --restart

# Kill existing processes first
pkill -f orkee && orkee dashboard
```

### Background Operation

Run Orkee services in the background:

```bash
# Run as background process
nohup orkee dashboard &

# Using systemd (production)
systemctl start orkee
```

## Configuration Validation

Orkee validates configuration on startup:

### Startup Checks

- **Port availability**: Ensures ports are not in use
- **Permission validation**: Checks file system permissions
- **Network binding**: Validates host/port combinations
- **Certificate validation**: For HTTPS setups

### Common Validation Errors

#### Port Already in Use

```bash
Error: Port 4001 is already in use

# Solution: Find and kill the process
lsof -i :4001
kill <PID>

# Or use a different port
orkee dashboard --api-port 4002
```

#### Permission Denied

```bash
Error: Permission denied binding to port 443

# Solution: Use sudo or higher port
sudo orkee dashboard --api-port 443
# or
orkee dashboard --api-port 4443
```

#### Invalid CORS Origin

```bash
Error: Invalid CORS origin format

# Solution: Use valid URL format
ORKEE_CORS_ORIGIN=https://example.com:3000 orkee dashboard
```

## Advanced Configuration

### Custom Host Binding

By default, Orkee binds to `localhost`. For network access:

```bash
# Bind to all interfaces (production use)
ORKEE_HOST=0.0.0.0 orkee dashboard

# Bind to specific interface
ORKEE_HOST=192.168.1.100 orkee dashboard
```

:::warning Network Security
Binding to `0.0.0.0` makes Orkee accessible from any network interface. Only use in trusted environments or behind a firewall.
:::

### Service Discovery

For containerized or distributed deployments:

```bash
# Docker container setup
ORKEE_API_PORT=4001
ORKEE_HOST=0.0.0.0
ORKEE_CORS_ORIGIN=http://frontend-service:5173
```

### Health Check Configuration

Configure health check endpoints for monitoring:

```bash
# Health checks available at:
curl http://localhost:4001/api/health      # Basic health
curl http://localhost:4001/api/status      # Detailed status
```

**Health Check Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime": 3600,
    "database": "connected"
  }
}
```

## Configuration Examples

### Local Development

```bash
# .env for local development
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
CORS_ALLOW_ANY_LOCALHOST=true

# Start development server
orkee dashboard
```

### Team Development

```bash
# Shared development server
ORKEE_HOST=0.0.0.0
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
ORKEE_CORS_ORIGIN=http://team-dev-server:5173
CORS_ALLOW_ANY_LOCALHOST=false
```

### Production Setup

```bash
# Production configuration
ORKEE_HOST=127.0.0.1  # Behind reverse proxy
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
ORKEE_CORS_ORIGIN=https://app.company.com
CORS_ALLOW_ANY_LOCALHOST=false

# Security headers enabled
SECURITY_HEADERS_ENABLED=true
RATE_LIMIT_ENABLED=true
```

### Docker Compose

```yaml
version: '3.8'
services:
  orkee:
    image: orkee:latest
    environment:
      - ORKEE_API_PORT=4001
      - ORKEE_UI_PORT=5173
      - ORKEE_HOST=0.0.0.0
      - CORS_ALLOW_ANY_LOCALHOST=false
      - ORKEE_CORS_ORIGIN=http://localhost:5173
    ports:
      - "4001:4001"
      - "5173:5173"
```

## Troubleshooting

### Debug Configuration

Enable detailed logging to diagnose configuration issues:

```bash
# Enable debug logging
RUST_LOG=debug orkee dashboard

# Specific module logging
RUST_LOG=orkee::config=debug orkee dashboard

# Network-specific debugging
RUST_LOG=orkee::server=trace orkee dashboard
```

### Configuration Verification

Verify your configuration is loaded correctly:

```bash
# Check environment variables
env | grep ORKEE

# Test API connectivity
curl http://localhost:4001/api/health

# Test CORS headers
curl -H "Origin: http://localhost:5173" http://localhost:4001/api/health
```

### Common Issues and Solutions

#### Dashboard Can't Connect to API

```bash
# Check API server is running
curl http://localhost:4001/api/health

# Verify CORS configuration
curl -H "Origin: http://localhost:5173" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS http://localhost:4001/api/health

# Check dashboard configuration
cat packages/dashboard/.env
```

#### Performance Issues

```bash
# Monitor resource usage
top -p $(pgrep orkee)

# Check connection limits
netstat -an | grep 4001 | wc -l

# Analyze request patterns
RUST_LOG=info orkee dashboard | grep "request_id"
```

For more specific configuration scenarios, see:

- [Security Configuration](security-settings) for security-focused settings
- [Cloud Sync](cloud-sync) for cloud integration configuration
- [TLS/HTTPS Configuration](../security/tls-https) for HTTPS setup