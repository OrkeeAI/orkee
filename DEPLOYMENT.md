# Production Deployment

**⚠️ This file has been moved to [`deployment/README.md`](deployment/README.md)**

The comprehensive deployment guide with all configuration templates, examples, and instructions is now located in the `deployment/` directory for better organization.

## Quick Links

- **[Complete Deployment Guide](deployment/README.md)** - Full production deployment instructions
- **[Docker Configuration](deployment/docker/)** - Container deployment with Docker Compose
- **[Nginx Configuration](deployment/nginx/)** - Reverse proxy setup and SSL templates
- **[Environment Templates](deployment/examples/)** - Production, development, and Docker environment files
- **[System Service](deployment/systemd/)** - Systemd service configuration

## Quick Start

1. **Docker Deployment (Recommended):**
   ```bash
   cp deployment/examples/.env.production .env
   # Edit .env with your settings
   docker-compose -f deployment/docker/docker-compose.yml up -d
   ```

2. **Manual Installation:**
   See [deployment/README.md](deployment/README.md) for detailed instructions.

For complete deployment instructions, troubleshooting, and all configuration options, please refer to [**deployment/README.md**](deployment/README.md).