---
sidebar_position: 3
title: Configuration Overview
---

# Configuration Overview

Orkee provides flexible configuration options to suit development, production, and team environments. This page provides a high-level overview of configuration methods and common scenarios.

## Configuration Methods

Orkee can be configured through three primary methods, listed in order of precedence:

1. **CLI Flags** - Highest precedence, overrides all other settings
2. **Environment Variables** - Middle precedence, overrides .env files
3. **.env Files** - Lowest precedence, provides default values

### CLI Flags

Command-line flags provide the most direct way to configure Orkee for a single session:

```bash
orkee dashboard --api-port 8080 --ui-port 3000
orkee tui --refresh-interval 20 --theme dark
```

### Environment Variables

Environment variables can be set in your shell or CI/CD environment:

```bash
export ORKEE_API_PORT=8080
export ORKEE_UI_PORT=3000
orkee dashboard
```

### .env Files

Create a `.env` file in your project root for persistent configuration:

```bash
# .env
ORKEE_API_PORT=8080
ORKEE_UI_PORT=3000
ORKEE_DEV_MODE=true
```

## Configuration Precedence

When the same setting is specified in multiple places, Orkee uses this precedence order:

```
CLI Flags > Environment Variables > .env Files > Defaults
```

**Example:**
```bash
# .env file
ORKEE_API_PORT=4001

# Shell environment
export ORKEE_API_PORT=5000

# CLI flag (wins)
orkee dashboard --api-port 8080  # Uses port 8080
```

## Configuration Categories

### Port Configuration

Control which ports Orkee uses for the API server and dashboard:

- `ORKEE_API_PORT` - API server port (default: 4001)
- `ORKEE_UI_PORT` - Dashboard UI port (default: 5173)

**CLI equivalents:**
```bash
orkee dashboard --api-port 4001 --ui-port 5173
```

### Development Configuration

Enable development mode to use the local dashboard source instead of the built version:

- `ORKEE_DEV_MODE` - Use local dashboard from `packages/dashboard/` (default: false)

**CLI equivalent:**
```bash
orkee dashboard --dev
```

### Security Configuration

Configure CORS, path validation, and TLS settings:

- `ORKEE_CORS_ORIGIN` - Allowed CORS origin (auto-calculated if not set)
- `CORS_ALLOW_ANY_LOCALHOST` - Allow any localhost origin in dev (default: true)
- `BROWSE_SANDBOX_MODE` - Path validation mode: `strict`, `relaxed`, or `disabled` (default: relaxed)
- `ALLOWED_BROWSE_PATHS` - Comma-separated allowed directories
- `TLS_ENABLED` - Enable HTTPS (default: false)
- `TLS_CERT_PATH` - Path to TLS certificate
- `TLS_KEY_PATH` - Path to TLS private key

See [Security Configuration](./configuration/security) for detailed security settings.

### Rate Limiting

Control API rate limits to protect against abuse:

- `RATE_LIMIT_ENABLED` - Enable rate limiting (default: true)
- `RATE_LIMIT_HEALTH_RPM` - Health endpoint limit (default: 60/min)
- `RATE_LIMIT_BROWSE_RPM` - Directory browsing limit (default: 20/min)
- `RATE_LIMIT_PROJECTS_RPM` - Projects API limit (default: 30/min)

See [Rate Limiting](./configuration/rate-limiting) for all rate limit settings.

### Cloud Sync Configuration

Configure Orkee Cloud integration for backup and sync:

- `ORKEE_CLOUD_TOKEN` - Authentication token for Orkee Cloud
- `ORKEE_CLOUD_API_URL` - API URL for Orkee Cloud (default: https://api.orkee.ai)

See [Cloud Configuration](./configuration/cloud) for cloud sync setup.

## Common Configuration Scenarios

### Development Environment

For local development with hot reloading:

```bash
# .env.development
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
ORKEE_DEV_MODE=true
CORS_ALLOW_ANY_LOCALHOST=true
RATE_LIMIT_ENABLED=false
```

```bash
orkee dashboard --dev
```

### Production Environment

For production deployments with security enabled:

```bash
# .env.production
ORKEE_API_PORT=4001
ORKEE_UI_PORT=3000
TLS_ENABLED=true
TLS_CERT_PATH=/etc/orkee/certs/cert.pem
TLS_KEY_PATH=/etc/orkee/certs/key.pem
RATE_LIMIT_ENABLED=true
SECURITY_HEADERS_ENABLED=true
BROWSE_SANDBOX_MODE=strict
```

See [Production Deployment](../deployment/production) for complete production setup.

### Team Environment

For shared team environments with cloud sync:

```bash
# .env.team
ORKEE_API_PORT=4001
ORKEE_CLOUD_TOKEN=your_team_token
ORKEE_CLOUD_API_URL=https://api.orkee.ai
ALLOWED_BROWSE_PATHS=/home/projects,/shared/workspace
```

### Docker Environment

When running Orkee in Docker:

```yaml
# docker-compose.yml
services:
  orkee:
    image: orkee/orkee:latest
    environment:
      - ORKEE_API_PORT=4001
      - ORKEE_UI_PORT=3000
      - TLS_ENABLED=false  # Let reverse proxy handle TLS
```

See [Docker Deployment](../deployment/docker) for complete Docker setup.

## Configuration Files Reference

### Environment Variable Files

- `.env` - Default environment variables for all environments
- `.env.local` - Local overrides (git-ignored)
- `.env.development` - Development-specific settings
- `.env.production` - Production-specific settings

### Configuration Best Practices

1. **Never commit sensitive tokens** - Use `.env.local` for secrets and add it to `.gitignore`
2. **Use environment-specific files** - Maintain separate configs for dev/prod
3. **Document custom settings** - Add comments explaining non-obvious configurations
4. **Validate on startup** - Check that required variables are set before running
5. **Use CLI flags sparingly** - Reserve for one-off overrides, not standard operation

## Legacy Configuration Variables

These variables are deprecated but still supported for backward compatibility:

- `PORT` - Use `ORKEE_API_PORT` instead
- `CORS_ORIGIN` - Use `ORKEE_CORS_ORIGIN` instead

## Next Steps

- **[Port Configuration](./configuration/ports)** - Detailed port configuration options
- **[Security Configuration](./configuration/security)** - Security and authentication settings
- **[Rate Limiting](./configuration/rate-limiting)** - API rate limit configuration
- **[Cloud Configuration](./configuration/cloud)** - Orkee Cloud sync setup
- **[Environment Variables Reference](./configuration/environment-variables)** - Complete variable listing

## Quick Reference

### Essential Variables

```bash
# Ports
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173

# Development
ORKEE_DEV_MODE=false

# Security
TLS_ENABLED=false
RATE_LIMIT_ENABLED=true
BROWSE_SANDBOX_MODE=relaxed

# Cloud
ORKEE_CLOUD_TOKEN=your_token
ORKEE_CLOUD_API_URL=https://api.orkee.ai
```

### Essential CLI Flags

```bash
orkee dashboard --api-port 4001 --ui-port 5173 --dev
orkee tui --refresh-interval 20 --theme dark
```

For a complete list of all configuration options, see the [Environment Variables Reference](./configuration/environment-variables).
