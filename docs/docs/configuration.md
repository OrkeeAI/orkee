---
sidebar_position: 3
title: Configuration Overview
---

# Configuration Overview

Orkee provides flexible configuration options to suit development, production, and team environments. This page provides a high-level overview of configuration methods and common scenarios.

## Configuration Methods

Orkee provides flexible configuration through multiple methods:

1. **Settings UI** (Recommended) - Configure runtime settings via the dashboard's Settings tab
2. **CLI Flags** - Override settings for a single session
3. **Environment Variables** - Bootstrap settings that control application startup
4. **.env Files** - Persistent bootstrap configuration

### Settings UI (Database-Managed)

Most settings are now configured via the Settings UI in the dashboard and stored in the database:
- Security settings (CORS, directory browsing, security headers)
- Rate limiting configuration
- TLS/HTTPS settings
- Cloud sync preferences

These settings persist across restarts and don't require .env configuration.

### Bootstrap Configuration (.env / Environment Variables)

Only essential bootstrap settings need to be in `.env`:
- `ORKEE_API_PORT` - API server port
- `ORKEE_UI_PORT` - Dashboard UI port  
- `ORKEE_DEV_MODE` - Development mode flag
- `ORKEE_CLOUD_TOKEN` - Cloud authentication token (optional)

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

**✨ Configured via Settings UI** (Settings > Security tab):

- CORS configuration (allow any localhost)
- Directory browsing paths and sandbox mode
- Security headers (HSTS, request ID, etc.)

See [Security Configuration](./configuration/security) for detailed security settings.

### Rate Limiting

**✨ Configured via Settings UI** (Settings > Advanced tab):

- Rate limiting for all endpoints (health, browse, projects, preview, AI, global)
- Burst size configuration
- Enable/disable rate limiting

See [Rate Limiting](./configuration/rate-limiting) for all rate limit settings.

### TLS/HTTPS Configuration

**✨ Configured via Settings UI** (Settings > Advanced tab):

- Enable/disable HTTPS
- Certificate and key paths
- Auto-generate certificates for development

See [Security Configuration](./configuration/security) for TLS setup.

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
```

```bash
orkee dashboard --dev
```

Then configure security/rate limiting via Settings UI as needed.

### Production Environment

For production deployments:

```bash
# .env.production
ORKEE_API_PORT=4001
ORKEE_UI_PORT=3000
ORKEE_DEV_MODE=false
```

Then configure via Settings UI:
- **Settings > Security**: Enable strict sandbox mode, configure CORS
- **Settings > Advanced**: Enable TLS/HTTPS, set certificate paths, configure rate limiting
- **Settings > Cloud**: Enable cloud sync if needed

See [Production Deployment](../deployment/production) for complete production setup.

### Team Environment

For shared team environments with cloud sync:

```bash
# .env.team
ORKEE_API_PORT=4001
ORKEE_CLOUD_TOKEN=your_team_token
```

Then configure via Settings UI:
- **Settings > Security**: Set allowed browse paths
- **Settings > Cloud**: Set cloud API URL

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
```

Configure TLS, rate limiting, and security via the Settings UI after startup.

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

### Minimal .env Configuration

```bash
# Bootstrap Settings (Required in .env)
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
ORKEE_DEV_MODE=false

# Cloud Authentication (Optional)
# ORKEE_CLOUD_TOKEN=your_token
```

### Settings UI Configuration

All other settings are configured via the dashboard:
- **Settings > General**: Editor integration, preferences
- **Settings > Security**: CORS, directory browsing, security headers
- **Settings > Database**: Import/export, data management  
- **Settings > Privacy**: Telemetry, error reporting
- **Settings > Cloud**: Cloud sync, API URL
- **Settings > Advanced**: Rate limiting, TLS/HTTPS, certificates

### Essential CLI Flags

```bash
orkee dashboard --api-port 4001 --ui-port 5173 --dev
orkee tui --refresh-interval 20 --theme dark
```

For legacy environment variable documentation, see the [Environment Variables Reference](./configuration/environment-variables).
