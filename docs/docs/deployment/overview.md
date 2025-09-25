# Deployment Overview

Orkee is designed for flexible deployment in various environments, from local development to production servers. This section covers all deployment options with step-by-step guides.

## Deployment Options

### Quick Start Options
- **[npm Installation](./npm-installation.md)** - Fastest way to get started (recommended for most users)
- **[Docker](./docker.md)** - Containerized deployment with Docker Compose
- **[Direct Binary](./binary-installation.md)** - Manual installation with GitHub releases

### Platform-Specific Guides
- **[Linux Server](./linux-server.md)** - Ubuntu/Debian production deployment
- **[macOS](./macos.md)** - Local development and production on macOS
- **[Windows](./windows.md)** - Windows Server and local installation
- **[Cloud Platforms](./cloud-platforms.md)** - AWS, GCP, Azure deployment guides

### Advanced Deployment
- **[Reverse Proxy](./reverse-proxy.md)** - Nginx/Apache configuration for production
- **[TLS/HTTPS](./tls-https.md)** - SSL certificate setup and security
- **[Environment Variables](./environment.md)** - Complete configuration reference

## Architecture Overview

Orkee consists of two main components:

### CLI Server (Rust)
- **Default Port**: 4001 (configurable)
- **Purpose**: REST API backend
- **Database**: SQLite (`~/.orkee/orkee.db`)
- **Binary**: `orkee` executable

### Dashboard (React/Vite)
- **Default Port**: 5173 (configurable)
- **Purpose**: Web interface
- **Connects to**: CLI server API
- **Technology**: React SPA with Vite dev server

## System Requirements

### Minimum Requirements
- **OS**: Linux, macOS, or Windows
- **Memory**: 512MB RAM
- **Storage**: 100MB free space
- **Network**: HTTP/HTTPS access for web interface

### Recommended Requirements
- **OS**: Linux (Ubuntu 20.04+), macOS (11+), Windows (Server 2019+)
- **Memory**: 1GB+ RAM
- **Storage**: 1GB+ free space
- **Network**: Dedicated port access (4001, 5173)

## Quick Installation

### Option 1: npm (Recommended)

```bash
# Install globally
npm install -g orkee

# Start Orkee
orkee dashboard

# Access at http://localhost:5173
```

### Option 2: Docker

```bash
# Using Docker Compose
curl -O https://raw.githubusercontent.com/orkee-ai/orkee/main/deployment/docker/docker-compose.yml
docker-compose up -d

# Access at http://localhost:5173
```

### Option 3: Binary Download

```bash
# Download for your platform
curl -L https://github.com/orkee-ai/orkee/releases/latest/download/orkee-linux-x64.tar.gz | tar xz
./orkee dashboard
```

## Production Considerations

### Security
- Enable TLS/HTTPS for production deployments
- Configure firewall rules for required ports
- Set up proper authentication if exposing publicly
- Regular security updates

### Performance
- Consider resource limits for Docker deployments
- Monitor SQLite database size and performance
- Configure appropriate logging levels
- Set up health monitoring

### Backup & Recovery
- Regular backup of `~/.orkee/` directory
- Consider database replication for high availability
- Document recovery procedures
- Test backup restoration process

## Getting Help

For deployment issues:

1. Check the [troubleshooting guide](../help-support/troubleshooting.md)
2. Review platform-specific documentation
3. Check GitHub issues for known problems
4. Join our community Discord for support

## Next Steps

Choose your deployment method:
- New users: Start with [npm installation](./npm-installation.md)
- Docker users: Follow [Docker guide](./docker.md)
- Production: Review [Linux server guide](./linux-server.md)