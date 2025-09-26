---
sidebar_position: 1
---

# Environment Variables

This document provides comprehensive information about all environment variables used to configure Orkee.

## Port Configuration

The core port configuration variables that control where Orkee services run:

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_API_PORT` | `4001` | API server port (can be overridden by `--api-port` flag) |
| `ORKEE_UI_PORT` | `5173` | Dashboard UI port (can be overridden by `--ui-port` flag) |
| `ORKEE_CORS_ORIGIN` | Auto-calculated | Allowed CORS origin (auto-set to `http://localhost:${ORKEE_UI_PORT}`) |

## Development Configuration

Variables that control development behavior:

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_DEV_MODE` | `false` | Use local dashboard from `packages/dashboard/` instead of `~/.orkee/dashboard/` |

### Legacy Variables (Deprecated)

| Variable | Replacement | Status |
|----------|-------------|--------|
| ~~`PORT`~~ | `ORKEE_API_PORT` | **Deprecated** |
| ~~`CORS_ORIGIN`~~ | `ORKEE_CORS_ORIGIN` | **Deprecated** |

## Server Configuration

Basic server behavior and CORS settings:

| Variable | Default | Description |
|----------|---------|-------------|
| `CORS_ALLOW_ANY_LOCALHOST` | `true` | Allow any localhost origin in development |

## Directory Browsing & Security

Configure path validation and sandboxing for directory browsing:

| Variable | Default | Description |
|----------|---------|-------------|
| `ALLOWED_BROWSE_PATHS` | `~/Documents,~/Projects,~/Desktop,~/Downloads` | Comma-separated list of allowed directory paths |
| `BROWSE_SANDBOX_MODE` | `relaxed` | Directory browsing security mode: `strict`/`relaxed`/`disabled` |

### Sandbox Modes

- **`strict`**: Only paths in `ALLOWED_BROWSE_PATHS` are accessible
- **`relaxed`**: Block dangerous system paths but allow most user paths (recommended)
- **`disabled`**: No restrictions (not recommended for security)

## Rate Limiting

Configure rate limiting to protect against abuse:

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_ENABLED` | `true` | Enable/disable rate limiting middleware |
| `RATE_LIMIT_HEALTH_RPM` | `60` | Rate limit for health endpoints (requests per minute) |
| `RATE_LIMIT_BROWSE_RPM` | `20` | Rate limit for directory browsing (requests per minute) |
| `RATE_LIMIT_PROJECTS_RPM` | `30` | Rate limit for project CRUD operations (requests per minute) |
| `RATE_LIMIT_PREVIEW_RPM` | `10` | Rate limit for preview server operations (requests per minute) |
| `RATE_LIMIT_GLOBAL_RPM` | `30` | Global rate limit for other endpoints (requests per minute) |
| `RATE_LIMIT_BURST_SIZE` | `5` | Burst size multiplier for rate limiting |

## Security Headers

Control security headers and request tracking:

| Variable | Default | Description |
|----------|---------|-------------|
| `SECURITY_HEADERS_ENABLED` | `true` | Enable/disable security headers middleware |
| `ENABLE_HSTS` | `false` | Enable HTTP Strict Transport Security (only for HTTPS) |
| `ENABLE_REQUEST_ID` | `true` | Enable request ID generation for audit logging |

## TLS/HTTPS Configuration

Configure HTTPS/TLS encryption:

| Variable | Default | Description |
|----------|---------|-------------|
| `TLS_ENABLED` | `false` | Enable/disable HTTPS/TLS support |
| `TLS_CERT_PATH` | `~/.orkee/certs/cert.pem` | Path to TLS certificate file |
| `TLS_KEY_PATH` | `~/.orkee/certs/key.pem` | Path to TLS private key file |
| `AUTO_GENERATE_CERT` | `true` | Auto-generate self-signed certificates for development |

## Dashboard Variables

Configure the React frontend dashboard:

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_ORKEE_API_PORT` | `4001` | API server port (passed from CLI via environment) |
| `VITE_API_URL` | Auto-constructed | Backend API URL (defaults to `http://localhost:${VITE_ORKEE_API_PORT}`) |

## Task Master AI Variables

Configure AI-powered task management features:

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | **Yes** | Claude API key (format: `sk-ant-api03-...`) |
| `PERPLEXITY_API_KEY` | Recommended | Perplexity API for research features (format: `pplx-...`) |
| `OPENAI_API_KEY` | Optional | OpenAI API key (format: `sk-proj-...`) |
| `GOOGLE_API_KEY` | Optional | Google Gemini API key |
| `MISTRAL_API_KEY` | Optional | Mistral AI API key |
| `XAI_API_KEY` | Optional | xAI API key |
| `GROQ_API_KEY` | Optional | Groq API key |
| `OPENROUTER_API_KEY` | Optional | OpenRouter API key |
| `AZURE_OPENAI_API_KEY` | Optional | Azure OpenAI API key |
| `OLLAMA_API_KEY` | Optional | Ollama API key (for remote servers) |
| `GITHUB_API_KEY` | Optional | GitHub API for import/export (format: `ghp_...` or `github_pat_...`) |

## Cloud Sync Variables

Configure Orkee Cloud integration for backup and synchronization:

| Variable | Default | Description |
|----------|---------|-------------|
| `ORKEE_CLOUD_API_URL` | `https://api.orkee.ai` | Orkee Cloud API base URL |
| `ORKEE_CLOUD_TOKEN` | - | Authentication token (set via `orkee cloud login`) |

:::info Authentication
Authentication is handled through OAuth. Use `orkee cloud login` to authenticate, which will securely store your token in `~/.orkee/auth.toml`. Do not set `ORKEE_CLOUD_TOKEN` manually.
:::

## Example Configuration Files

### Complete .env File

```bash
# Port Configuration (simple and clean - just two ports!)
ORKEE_API_PORT=4001       # API server port
ORKEE_UI_PORT=5173        # Dashboard UI port
# ORKEE_CORS_ORIGIN is auto-calculated from UI port if not set

# Server Configuration
CORS_ALLOW_ANY_LOCALHOST=true

# Directory Browsing Security
ALLOWED_BROWSE_PATHS="~/Documents,~/Projects,~/Code,~/Desktop"
BROWSE_SANDBOX_MODE=relaxed

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_HEALTH_RPM=60
RATE_LIMIT_BROWSE_RPM=20
RATE_LIMIT_PROJECTS_RPM=30
RATE_LIMIT_PREVIEW_RPM=10
RATE_LIMIT_GLOBAL_RPM=30
RATE_LIMIT_BURST_SIZE=5

# Security Headers
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=false  # Set to true only when using HTTPS
ENABLE_REQUEST_ID=true

# TLS/HTTPS Configuration
TLS_ENABLED=false
AUTO_GENERATE_CERT=true
```

### Development Configuration

```bash
# Development-focused configuration
ORKEE_API_PORT=4001
ORKEE_UI_PORT=5173
CORS_ALLOW_ANY_LOCALHOST=true
BROWSE_SANDBOX_MODE=relaxed
RATE_LIMIT_ENABLED=true
SECURITY_HEADERS_ENABLED=true
TLS_ENABLED=false
```

### Production Configuration

```bash
# Production-ready configuration
ORKEE_API_PORT=443
ORKEE_UI_PORT=5173
CORS_ALLOW_ANY_LOCALHOST=false
ORKEE_CORS_ORIGIN=https://your-domain.com:5173
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS="/var/orkee/allowed,/home/orkee/projects"

# Enhanced security for production
RATE_LIMIT_ENABLED=true
RATE_LIMIT_GLOBAL_RPM=15
RATE_LIMIT_BURST_SIZE=3
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
ENABLE_REQUEST_ID=true

# TLS configuration
TLS_ENABLED=true
AUTO_GENERATE_CERT=false
TLS_CERT_PATH="/etc/ssl/certs/orkee.crt"
TLS_KEY_PATH="/etc/ssl/private/orkee.key"
```

### Dashboard-only Configuration

```bash
# Usually auto-configured by the CLI, but can be overridden if needed
# VITE_API_URL=http://localhost:4001  # Only set this if you need a custom URL

# For HTTPS setups
VITE_API_URL=https://localhost:4001
# Or for custom domain
VITE_API_URL=https://dev.orkee.local:4001
```

### Cloud Sync Configuration

```bash
# Orkee Cloud Configuration
ORKEE_CLOUD_API_URL=https://api.orkee.ai

# Note: ORKEE_CLOUD_TOKEN is set automatically via `orkee cloud login`
# Do not set manually - use the OAuth authentication flow
```

## Loading Environment Variables

Environment variables can be loaded in several ways:

1. **System environment**: Export variables in your shell
2. **`.env` file**: Create a `.env` file in your working directory
3. **Command line flags**: Override specific settings with CLI flags

### Precedence Order

1. Command line flags (highest priority)
2. Environment variables
3. `.env` file
4. Default values (lowest priority)

### Example Usage

```bash
# Using environment variables directly
ORKEE_API_PORT=8080 orkee dashboard

# Using .env file
echo "ORKEE_API_PORT=8080" > .env
orkee dashboard

# Using command line flags (overrides environment)
ORKEE_API_PORT=8080 orkee dashboard --api-port 9000  # Uses port 9000
```

## Troubleshooting

### Common Issues

#### Port Already in Use
```bash
# Check which process is using the port
lsof -i :4001

# Use a different port
ORKEE_API_PORT=4002 orkee dashboard
```

#### Permission Denied
```bash
# Check directory permissions
ls -la ~/.orkee/

# Fix permissions
chmod 755 ~/.orkee/
chmod 600 ~/.orkee/certs/key.pem  # For TLS private keys
```

#### Environment Variables Not Loading
```bash
# Verify .env file location and contents
cat .env
pwd

# Test environment loading
env | grep ORKEE

# Debug with verbose output
RUST_LOG=debug orkee dashboard
```