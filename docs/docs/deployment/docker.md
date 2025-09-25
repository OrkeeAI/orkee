# Docker Deployment

Deploy Orkee using Docker containers for consistent, portable deployments across any environment.

## Prerequisites

- **Docker**: Version 20.10 or higher
- **Docker Compose**: Version 2.0 or higher (optional but recommended)
- **Platform**: Any system supporting Docker (Linux, macOS, Windows)

## Quick Start

### Using Docker Compose (Recommended)

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  orkee:
    image: orkee/orkee:latest
    container_name: orkee
    ports:
      - "4001:4001"  # API server
      - "5173:5173"  # Dashboard
    volumes:
      - orkee_data:/root/.orkee
      - /var/run/docker.sock:/var/run/docker.sock:ro  # Optional: for Docker integration
    environment:
      - ORKEE_API_PORT=4001
      - ORKEE_UI_PORT=5173
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4001/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

volumes:
  orkee_data:
    driver: local
```

Start the service:

```bash
# Download and start
docker-compose up -d

# View logs
docker-compose logs -f

# Access the dashboard at http://localhost:5173
```

### Using Docker CLI

Run Orkee directly with Docker:

```bash
# Create a volume for persistent data
docker volume create orkee_data

# Run Orkee container
docker run -d \
  --name orkee \
  -p 4001:4001 \
  -p 5173:5173 \
  -v orkee_data:/root/.orkee \
  -e RUST_LOG=info \
  --restart unless-stopped \
  orkee/orkee:latest

# Access the dashboard at http://localhost:5173
```

## Configuration

### Environment Variables

Configure Orkee using environment variables in your `docker-compose.yml`:

```yaml
environment:
  # Port Configuration
  - ORKEE_API_PORT=4001
  - ORKEE_UI_PORT=5173
  
  # Logging
  - RUST_LOG=info  # debug, info, warn, error
  
  # Security
  - TLS_ENABLED=false
  - SECURITY_HEADERS_ENABLED=true
  
  # Path Validation
  - BROWSE_SANDBOX_MODE=relaxed  # strict, relaxed, disabled
  
  # Rate Limiting
  - RATE_LIMIT_ENABLED=true
  - RATE_LIMIT_GLOBAL_RPM=30
  
  # Cloud Sync (optional)
  - ORKEE_CLOUD_TOKEN=your_token_here
  - ORKEE_CLOUD_API_URL=https://api.orkee.ai
```

### Custom Ports

Change the default ports:

```yaml
services:
  orkee:
    ports:
      - "8080:8080"  # Custom API port
      - "3000:3000"  # Custom UI port
    environment:
      - ORKEE_API_PORT=8080
      - ORKEE_UI_PORT=3000
```

### Volume Mounts

Mount additional directories:

```yaml
services:
  orkee:
    volumes:
      - orkee_data:/root/.orkee
      - /home/user/projects:/workspace/projects:ro  # Read-only project access
      - ./config:/config:ro  # Custom configuration files
      - /var/log/orkee:/var/log/orkee  # Log directory
```

## Production Configuration

### Complete Production Setup

```yaml
version: '3.8'

services:
  orkee:
    image: orkee/orkee:latest
    container_name: orkee-prod
    ports:
      - "127.0.0.1:4001:4001"  # Bind to localhost only
      - "127.0.0.1:5173:5173"
    volumes:
      - orkee_data:/root/.orkee
      - ./logs:/var/log/orkee
      - /etc/ssl/certs:/etc/ssl/certs:ro  # System CA certificates
    environment:
      - ORKEE_API_PORT=4001
      - ORKEE_UI_PORT=5173
      - RUST_LOG=warn
      - TLS_ENABLED=true
      - TLS_CERT_PATH=/certs/cert.pem
      - TLS_KEY_PATH=/certs/key.pem
      - SECURITY_HEADERS_ENABLED=true
      - RATE_LIMIT_ENABLED=true
      - BROWSE_SANDBOX_MODE=strict
    secrets:
      - orkee_tls_cert
      - orkee_tls_key
      - orkee_cloud_token
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "-k", "https://localhost:4001/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  # Optional: Reverse proxy for production
  nginx:
    image: nginx:alpine
    container_name: orkee-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
    depends_on:
      - orkee
    restart: unless-stopped

volumes:
  orkee_data:
    driver: local

secrets:
  orkee_tls_cert:
    file: ./certs/cert.pem
  orkee_tls_key:
    file: ./certs/key.pem
  orkee_cloud_token:
    file: ./secrets/cloud_token.txt
```

### Resource Limits

Set memory and CPU limits:

```yaml
services:
  orkee:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
```

## Building Custom Images

### Dockerfile

Create a custom image with additional tools:

```dockerfile
FROM orkee/orkee:latest

# Install additional tools
RUN apt-get update && apt-get install -y \
    git \
    curl \
    wget \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Add custom configuration
COPY config/ /config/
COPY scripts/ /scripts/

# Set custom environment
ENV RUST_LOG=info
ENV ORKEE_API_PORT=4001
ENV ORKEE_UI_PORT=5173

# Custom entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["dashboard"]
```

Build and run:

```bash
# Build custom image
docker build -t my-orkee:latest .

# Run custom image
docker run -d \
  --name my-orkee \
  -p 4001:4001 -p 5173:5173 \
  -v orkee_data:/root/.orkee \
  my-orkee:latest
```

## Multi-Stage Deployment

### Development and Production

Use Docker Compose overrides for different environments:

**docker-compose.yml** (base):
```yaml
version: '3.8'

services:
  orkee:
    image: orkee/orkee:latest
    volumes:
      - orkee_data:/root/.orkee
    environment:
      - ORKEE_API_PORT=4001
      - ORKEE_UI_PORT=5173

volumes:
  orkee_data:
```

**docker-compose.override.yml** (development):
```yaml
version: '3.8'

services:
  orkee:
    ports:
      - "4001:4001"
      - "5173:5173"
    environment:
      - RUST_LOG=debug
    volumes:
      - ./dev-projects:/workspace:rw
```

**docker-compose.prod.yml** (production):
```yaml
version: '3.8'

services:
  orkee:
    ports:
      - "127.0.0.1:4001:4001"
      - "127.0.0.1:5173:5173"
    environment:
      - RUST_LOG=warn
      - TLS_ENABLED=true
    restart: unless-stopped
```

Deploy with specific configuration:

```bash
# Development
docker-compose up -d

# Production
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

## Container Management

### Basic Operations

```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# Restart services
docker-compose restart

# View logs
docker-compose logs -f orkee

# Execute commands in container
docker-compose exec orkee bash
docker-compose exec orkee orkee projects list
```

### Monitoring

```bash
# Check container status
docker-compose ps

# View resource usage
docker stats orkee

# Check health status
docker-compose exec orkee curl -f http://localhost:4001/api/health
```

### Data Management

```bash
# Backup data volume
docker run --rm -v orkee_data:/source -v $(pwd):/backup alpine tar czf /backup/orkee-backup.tar.gz -C /source .

# Restore data volume
docker run --rm -v orkee_data:/target -v $(pwd):/backup alpine tar xzf /backup/orkee-backup.tar.gz -C /target

# Clean up unused resources
docker system prune -f
docker volume prune -f
```

## Troubleshooting

### Port Conflicts

If ports are already in use:

```bash
# Check what's using the ports
lsof -i :4001
lsof -i :5173

# Use different ports in docker-compose.yml
ports:
  - "4002:4001"
  - "5174:5173"
```

### Permission Issues

Fix file permission issues:

```bash
# Set correct ownership
docker-compose exec orkee chown -R $(id -u):$(id -g) /root/.orkee

# Or run as current user
user: "${UID}:${GID}"
```

### Container Won't Start

Debug startup issues:

```bash
# Check logs
docker-compose logs orkee

# Run interactively
docker-compose run --rm orkee bash

# Test health endpoint
docker-compose exec orkee curl http://localhost:4001/api/health
```

### Network Issues

Resolve networking problems:

```bash
# Check Docker networks
docker network ls

# Inspect network configuration
docker network inspect $(docker-compose ps -q orkee)

# Test container connectivity
docker-compose exec orkee ping google.com
```

## Security Considerations

### Production Security

- Run containers as non-root user when possible
- Use Docker secrets for sensitive configuration
- Enable TLS/HTTPS for external access
- Restrict network access with firewall rules
- Regularly update base images for security patches

### Example Secure Configuration

```yaml
services:
  orkee:
    user: "1000:1000"  # Run as non-root
    read_only: true    # Read-only filesystem
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE
    tmpfs:
      - /tmp
    security_opt:
      - no-new-privileges:true
```

## Next Steps

After Docker deployment:

1. **[Reverse Proxy Setup](./reverse-proxy.md)** - Configure Nginx/Apache
2. **[TLS Configuration](./tls-https.md)** - Enable HTTPS
3. **[Environment Variables](./environment.md)** - Advanced configuration
4. **[Production Guide](../configuration/production.md)** - Production best practices