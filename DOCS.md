# Orkee Documentation

This document provides comprehensive information about Orkee configuration, environment variables, security settings, and operational details.

## Table of Contents

1. [Environment Variables](#environment-variables)
2. [Security Configuration](#security-configuration)
3. [TLS/HTTPS Configuration](#tlshttps-configuration)
4. [File Locations & Data Storage](#file-locations--data-storage)
5. [CLI Commands Reference](#cli-commands-reference)
6. [API Reference](#api-reference)
7. [Default Ports & URLs](#default-ports--urls)
8. [Development vs Production](#development-vs-production)
9. [Troubleshooting](#troubleshooting)

## Environment Variables

### CLI Server Variables

These variables configure the Orkee CLI server (Rust backend):

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `4001` | Port for the CLI server to listen on |
| `CORS_ORIGIN` | `http://localhost:5173` | Allowed CORS origin (must be localhost) |
| `CORS_ALLOW_ANY_LOCALHOST` | `true` | Allow any localhost origin in development |
| `ALLOWED_BROWSE_PATHS` | `~/Documents,~/Projects,~/Desktop,~/Downloads` | Comma-separated list of allowed directory paths |
| `BROWSE_SANDBOX_MODE` | `relaxed` | Directory browsing security mode: `strict`/`relaxed`/`disabled` |

### Security Middleware Variables

Configure rate limiting, security headers, and error handling:

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_ENABLED` | `true` | Enable/disable rate limiting middleware |
| `RATE_LIMIT_HEALTH_RPM` | `60` | Rate limit for health endpoints (requests per minute) |
| `RATE_LIMIT_BROWSE_RPM` | `20` | Rate limit for directory browsing (requests per minute) |
| `RATE_LIMIT_PROJECTS_RPM` | `30` | Rate limit for project CRUD operations (requests per minute) |
| `RATE_LIMIT_PREVIEW_RPM` | `10` | Rate limit for preview server operations (requests per minute) |
| `RATE_LIMIT_GLOBAL_RPM` | `30` | Global rate limit for other endpoints (requests per minute) |
| `RATE_LIMIT_BURST_SIZE` | `5` | Burst size multiplier for rate limiting |
| `SECURITY_HEADERS_ENABLED` | `true` | Enable/disable security headers middleware |
| `ENABLE_HSTS` | `false` | Enable HTTP Strict Transport Security (only for HTTPS) |
| `ENABLE_REQUEST_ID` | `true` | Enable request ID generation for audit logging |

#### Example .env configuration:
```bash
# Server Configuration
PORT=4001
CORS_ORIGIN="http://localhost:5173"
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
```

### Dashboard Variables

These variables configure the React dashboard frontend:

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_API_URL` | `http://localhost:4001` | Backend API URL for the dashboard to connect to |

#### Example dashboard .env:
```bash
VITE_API_URL=http://localhost:4001
```

### Task Master AI Variables (Optional)

For AI-powered task management features, configure these API keys:

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

## Security Configuration

### Directory Browsing Sandbox

Orkee implements a comprehensive directory browsing security system with three modes:

#### Strict Mode (`BROWSE_SANDBOX_MODE=strict`)
- **Allowlist only**: Only paths in `ALLOWED_BROWSE_PATHS` are accessible
- **Root access blocked**: No access to system root unless explicitly allowed
- **Path traversal blocked**: `../` navigation is completely blocked
- **Use case**: High-security environments

#### Relaxed Mode (`BROWSE_SANDBOX_MODE=relaxed`) - **Default**
- **Blocklist approach**: Block dangerous system paths but allow most user paths
- **Path traversal allowed**: `../` navigation permitted within safe boundaries
- **Home directory allowed**: Full access to user's home directory
- **Use case**: Local development with reasonable security

#### Disabled Mode (`BROWSE_SANDBOX_MODE=disabled`) - **NOT RECOMMENDED**
- **No restrictions**: Access to any readable directory
- **Security warning**: Only use in completely trusted environments
- **Use case**: Debugging or specialized use cases only

### Blocked Paths (All Modes)

The following system directories are **always** blocked for security:

**System Directories:**
- `/etc`, `/private/etc` (configuration files)
- `/sys`, `/proc`, `/dev` (system filesystems)
- `/usr/bin`, `/usr/sbin`, `/bin`, `/sbin` (system binaries)
- `/var/log`, `/var/run`, `/var/lock` (system runtime)
- `/boot`, `/root`, `/mnt`, `/media`, `/opt`
- `/tmp`, `/var/tmp` (temporary directories)

**Windows System Directories:**
- `C:\Windows`, `C:\Program Files`, `C:\Program Files (x86)`
- `C:\ProgramData`, `C:\System32`

**Sensitive User Directories (relative to home):**
- `.ssh`, `.aws`, `.gnupg`, `.docker`, `.kube`
- `.config/git`, `.npm`, `.cargo/credentials`
- `.gitconfig`, `.env*` files
- `Library/Keychains` (macOS)
- `AppData/Local/Microsoft` (Windows)

### CORS Configuration

CORS is configured for local development security:

- **Allowed Origins**: Only `localhost` origins are permitted
- **Port Whitelist**: Ports 3000-3099, 4000-4199, 5000-5999, 8000-8099
- **Headers**: Restricted to essential headers only
- **Methods**: GET, POST, PUT, DELETE, OPTIONS

### Input Validation

All user inputs are validated with:

- **Length Limits**: Project names (100 chars), descriptions (1000 chars)
- **Pattern Validation**: Alphanumeric with safe special characters
- **Command Filtering**: Detection of dangerous shell commands
- **Path Safety**: Prevention of path traversal attacks
- **Script Injection**: Protection against code injection in scripts

### Security Middleware

Orkee implements production-grade security middleware:

#### Rate Limiting
- **Per-IP tracking**: Token bucket algorithm with configurable limits
- **Endpoint-specific limits**: Different rates for health, browsing, projects, and preview operations
- **429 responses**: Proper `Too Many Requests` with `Retry-After` headers
- **Burst protection**: Configurable burst sizes to handle legitimate traffic spikes
- **Audit logging**: All rate limit violations logged for monitoring

#### Security Headers
Automatically applied to all responses:
- **Content Security Policy (CSP)**: Restrictive policy allowing development workflows
- **X-Content-Type-Options**: `nosniff` to prevent MIME type sniffing
- **X-Frame-Options**: `DENY` to prevent clickjacking
- **X-XSS-Protection**: Legacy XSS protection
- **Referrer-Policy**: `strict-origin-when-cross-origin`
- **Permissions-Policy**: Disables dangerous browser APIs (geolocation, camera, etc.)
- **HSTS** (optional): HTTP Strict Transport Security for HTTPS deployments

#### Error Handling & Logging
- **Sanitized responses**: No internal details leaked to clients
- **Request ID tracking**: Unique IDs for audit trail correlation
- **Structured logging**: JSON-formatted logs with security event flagging
- **Panic recovery**: Graceful handling of server panics with safe responses
- **Consistent format**: All API errors follow standard `{success, error, request_id}` format

## TLS/HTTPS Configuration

Orkee supports TLS/HTTPS encryption for secure connections. When enabled, the server runs in dual mode with automatic HTTP-to-HTTPS redirect.

### TLS Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TLS_ENABLED` | `false` | Enable/disable HTTPS/TLS support |
| `TLS_CERT_PATH` | `~/.orkee/certs/cert.pem` | Path to TLS certificate file |
| `TLS_KEY_PATH` | `~/.orkee/certs/key.pem` | Path to TLS private key file |
| `AUTO_GENERATE_CERT` | `true` | Auto-generate self-signed certificates for development |

### TLS Configuration Modes

#### Development Mode (Default)
- **Auto-generated certificates**: Self-signed certificates created automatically
- **Browser warnings**: Browsers will show security warnings (expected behavior)
- **Certificate validity**: 365 days, auto-renewed when within 30 days of expiry
- **Common Names**: `localhost`, `127.0.0.1`, `::1`, `orkee.local`, `dev.orkee.local`

#### Production Mode
- **Custom certificates**: Provide your own certificates via `TLS_CERT_PATH`/`TLS_KEY_PATH`
- **Valid certificates**: Use certificates from a trusted CA (Let's Encrypt, commercial CA)
- **No browser warnings**: Properly validated certificates won't trigger warnings

### Dual Server Architecture

When TLS is enabled, Orkee runs two servers simultaneously:

- **HTTPS Server** (main): Runs on configured port (default 4001) serving the full application
- **HTTP Redirect Server**: Runs on port-1 (default 4000) that redirects all traffic to HTTPS

#### Port Configuration
- **HTTPS Port**: Uses `PORT` environment variable (default 4001)
- **HTTP Port**: Automatically uses `PORT - 1` (default 4000)
- **Custom Ports**: If using non-default HTTPS port, HTTP port will be HTTPS port - 1

### Example TLS Configuration

#### Basic HTTPS Setup
```bash
# Enable TLS with auto-generated certificates
TLS_ENABLED=true
AUTO_GENERATE_CERT=true
PORT=4001

# Security headers (recommended for HTTPS)
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
```

#### Custom Certificate Setup
```bash
# Enable TLS with custom certificates
TLS_ENABLED=true
AUTO_GENERATE_CERT=false
TLS_CERT_PATH="/path/to/your/certificate.pem"
TLS_KEY_PATH="/path/to/your/private-key.pem"
PORT=4001

# Security headers for production
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
```

### Certificate Management

#### Auto-Generated Certificates
- **Location**: `~/.orkee/certs/` directory
- **Filenames**: `cert.pem` (certificate), `key.pem` (private key)
- **Permissions**: Private key automatically set to 600 (owner read/write only)
- **Renewal**: Automatic renewal when certificate is within 30 days of expiry
- **Multiple domains**: Includes localhost, 127.0.0.1, IPv6 localhost, and orkee.local domains

#### Custom Certificates
- **Format**: PEM format for both certificate and private key
- **Chain Support**: Full certificate chains are supported
- **Key Types**: RSA, ECDSA, and Ed25519 keys supported
- **Validation**: Certificates validated on startup, server won't start with invalid certificates

### HTTPS Redirect Behavior

The HTTP redirect server provides:
- **301 Permanent Redirect**: All HTTP requests redirected with 301 status
- **Preserve Paths**: Original path and query parameters preserved in redirect
- **Host Preservation**: Original host header preserved (configurable)
- **Custom Ports**: Non-standard HTTPS ports properly handled in redirect URLs
- **Proxy Support**: X-Forwarded-Proto header detection for reverse proxy setups

### Dashboard Configuration

When using HTTPS, update the dashboard configuration:

```bash
# Dashboard .env for HTTPS
VITE_API_URL=https://localhost:4001

# Or for custom domain
VITE_API_URL=https://dev.orkee.local:4001
```

### Troubleshooting TLS

#### Certificate Generation Issues
- **Permission errors**: Ensure write access to certificate directory
- **Port conflicts**: Check that both HTTP and HTTPS ports are available
- **Certificate validation**: Check server logs for certificate validation errors

#### Browser Issues
- **Self-signed warnings**: Expected with auto-generated certificates, add security exception
- **HSTS conflicts**: Clear browser HSTS cache if switching between HTTP/HTTPS
- **Mixed content**: Ensure all resources load over HTTPS when using HTTPS

#### Common Solutions
```bash
# Check certificate status
ls -la ~/.orkee/certs/

# Regenerate certificates (delete existing ones)
rm ~/.orkee/certs/cert.pem ~/.orkee/certs/key.pem

# Test HTTPS connection
curl -k https://localhost:4001/api/health

# Check server logs for TLS issues
orkee dashboard  # Watch for TLS-related log messages
```

## File Locations & Data Storage

### Configuration Files

| File | Purpose | Format |
|------|---------|--------|
| `~/.orkee/orkee.db` | Primary SQLite database | Binary SQLite |
| `~/.orkee/projects.json` | Legacy JSON storage (if enabled) | JSON |
| `.env` | Environment variables | Key=Value pairs |
| `.taskmaster/config.json` | Task Master AI configuration | JSON |

### Database Schema

The SQLite database (`~/.orkee/orkee.db`) contains:

- **projects**: Project metadata, paths, Git info
- **project_tags**: Many-to-many relationship for tags
- **tags**: Tag definitions
- **schema_migrations**: Version tracking

### Log Files

Audit logs are written to:
- **Format**: Structured JSON with timestamps
- **Location**: Standard output (captured by systemd/Docker in production)
- **Content**: Directory access attempts, security violations, API calls

Example log entry:
```json
{
  "timestamp": "2025-01-01T12:00:00Z",
  "user": "anonymous",
  "action": "browse_directory",
  "requested_path": "/home/user/Documents",
  "resolved_path": "/home/user/Documents",
  "allowed": true,
  "entries_count": 15,
  "source": "directory_browser"
}
```

## CLI Commands Reference

### Main Commands

#### Dashboard
Start the full dashboard (backend + frontend):
```bash
orkee dashboard [OPTIONS]

Options:
  -p, --port <PORT>              Server port [default: 4001]
      --cors-origin <CORS_ORIGIN> CORS origin [default: http://localhost:5173]
      --restart                  Restart services (kill existing first)
```

#### TUI (Terminal User Interface)
Launch the terminal interface:
```bash
orkee tui [OPTIONS]

Options:
      --refresh-interval <SECONDS>  Refresh interval [default: 20]
      --theme <THEME>               Theme: light, dark [default: dark]
```

### Project Management

#### List Projects
```bash
orkee projects list
```

#### Show Project Details
```bash
orkee projects show <PROJECT_ID>
```

#### Add New Project
```bash
orkee projects add [OPTIONS]

Options:
      --name <NAME>               Project name
      --path <PATH>               Project root path
      --description <DESCRIPTION> Project description
```

#### Edit Project
```bash
orkee projects edit <PROJECT_ID>
```

#### Delete Project
```bash
orkee projects delete <PROJECT_ID> [OPTIONS]

Options:
      --yes  Skip confirmation prompt
```

### Preview Management

#### Stop All Preview Servers
```bash
orkee preview stop-all
```

## API Reference

The CLI server provides a REST API on port 4001 (configurable).

### Response Format

All API responses follow this format:
```json
{
  "success": boolean,
  "data": any | null,
  "error": string | null
}
```

### Health & Status Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/health` | Basic health check |
| GET | `/api/status` | Detailed service status |

### Project Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/projects` | List all projects |
| GET | `/api/projects/:id` | Get project by ID |
| GET | `/api/projects/by-name/:name` | Get project by name |
| POST | `/api/projects/by-path` | Get project by path |
| POST | `/api/projects` | Create new project |
| PUT | `/api/projects/:id` | Update project |
| DELETE | `/api/projects/:id` | Delete project |

#### Project Data Structure
```json
{
  "id": "uuid",
  "name": "Project Name",
  "projectRoot": "/path/to/project",
  "status": "active" | "archived" | "draft",
  "priority": "high" | "medium" | "low",
  "createdAt": "2025-01-01T12:00:00Z",
  "updatedAt": "2025-01-01T12:00:00Z",
  "tags": ["tag1", "tag2"],
  "description": "Optional description",
  "setupScript": "Optional setup command",
  "devScript": "Optional dev command",
  "cleanupScript": "Optional cleanup command",
  "gitRepository": {
    "owner": "username",
    "repo": "reponame", 
    "url": "https://github.com/username/repo.git",
    "branch": "main"
  }
}
```

### Directory Browsing Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/api/browse-directories` | Browse filesystem directories |

#### Browse Request/Response
```json
// Request
{
  "path": "/path/to/browse" // optional, uses safe default if omitted
}

// Response
{
  "success": true,
  "data": {
    "currentPath": "/resolved/path",
    "parentPath": "/parent/path", // null if at sandbox root
    "directories": [
      {
        "name": "dirname",
        "path": "/full/path/to/dirname",
        "isDirectory": true
      }
    ],
    "isRoot": false,
    "separator": "/"
  },
  "error": null
}
```

### Preview Server Endpoints

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/api/preview/health` | Preview service health |
| GET | `/api/preview/servers` | List active preview servers |
| POST | `/api/preview/servers/:project_id/start` | Start preview server |
| POST | `/api/preview/servers/:project_id/stop` | Stop preview server |
| GET | `/api/preview/servers/:project_id/status` | Get server status |
| GET | `/api/preview/servers/:project_id/logs` | Get server logs |
| POST | `/api/preview/servers/:project_id/logs/clear` | Clear server logs |
| POST | `/api/preview/servers/:project_id/activity` | Update activity timestamp |

## Default Ports & URLs

### Development Environment

| Service | Port | URL | Purpose |
|---------|------|-----|---------|
| CLI Server | 4001 | http://localhost:4001 | REST API backend |
| Dashboard | 5173 | http://localhost:5173 | React frontend (dev) |
| Preview Servers | 5000-5999 | http://localhost:50XX | Dynamic project previews |

### Production Environment

| Service | Port | Purpose |
|---------|------|---------|
| CLI Server | 80/443 | HTTP/HTTPS API |
| Dashboard | Served by CLI | Static files served by backend |

## Development vs Production

### Development Configuration
- CORS allows any localhost origin
- Detailed error messages
- Hot reloading enabled
- SQLite database in `~/.orkee/`
- Relaxed security policies

### Production Configuration
- Strict CORS policy
- Sanitized error messages
- Static file serving
- Database in persistent volume
- Enhanced security features:
  - Rate limiting
  - HTTPS enforcement
  - Security headers
  - Input sanitization
  - Audit logging

### Security Checklist for Production

- [ ] Configure strict CORS origins
- [ ] Enable HTTPS/TLS
- [ ] Set up rate limiting
- [ ] Configure security headers
- [ ] Set up log aggregation
- [ ] Enable audit logging
- [ ] Use environment-specific secrets
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategy
- [ ] Set up health checks

## Troubleshooting

### Common Issues

#### "Permission denied" errors
- Check `ALLOWED_BROWSE_PATHS` configuration
- Verify directory permissions
- Check sandbox mode setting

#### CORS errors in dashboard
- Verify `CORS_ORIGIN` matches dashboard URL
- Check `CORS_ALLOW_ANY_LOCALHOST` setting
- Ensure CLI server is running on correct port

#### Database connection errors
- Check `~/.orkee/` directory exists and is writable
- Verify SQLite file permissions
- Check disk space

#### Port conflicts
- Use `lsof -i :4001` to check port usage
- Change `PORT` environment variable
- Use `--port` flag with dashboard command

#### Preview server failures
- Check project has valid dev script
- Verify port availability (5000-5999 range)
- Check project directory permissions

#### Rate limiting errors (HTTP 429)
- Check `RATE_LIMIT_ENABLED` setting
- Adjust endpoint-specific limits (`RATE_LIMIT_*_RPM` variables)
- Increase burst size with `RATE_LIMIT_BURST_SIZE`
- Wait for `Retry-After` period before retrying
- Monitor logs for rate limit violations

#### Security header issues
- Verify `SECURITY_HEADERS_ENABLED=true`
- Check CSP violations in browser console
- For HTTPS deployments, enable `ENABLE_HSTS=true`
- Some headers may be overridden by reverse proxies

#### Request ID missing in logs
- Ensure `ENABLE_REQUEST_ID=true`
- Request IDs appear in error responses and structured logs
- Use request IDs for correlating audit events

### Debug Mode

Enable verbose logging:
```bash
RUST_LOG=debug orkee dashboard
```

### Health Check Endpoints

Test service health:
```bash
# Basic health
curl http://localhost:4001/api/health

# Detailed status
curl http://localhost:4001/api/status
```

### Log Analysis

Monitor audit logs for security events:
```bash
# Filter security violations
journalctl -u orkee | grep "access denied"

# Monitor directory access
journalctl -u orkee | grep "directory_access"

# Monitor rate limit violations
journalctl -u orkee | grep "Rate limit exceeded"

# Filter by request ID for specific incident
journalctl -u orkee | grep "request_id.*abc123"

# Monitor all audit events
journalctl -u orkee | grep "audit.*true"

# Check middleware startup messages
journalctl -u orkee | grep "middleware configuration"
```

### Security Monitoring

Key audit events to monitor:
- **Rate Limiting**: `Rate limit exceeded` with IP and endpoint
- **Path Security**: `Path access denied`, `Path traversal attempt`
- **Request Tracking**: All errors include `request_id` for correlation
- **Server Panics**: `Server panic occurred` with sanitized details
- **Middleware Status**: Startup logs show enabled features

Example structured log entry:
```json
{
  "timestamp": "2025-01-01T12:00:00Z",
  "request_id": "abc123-def456",
  "audit": true,
  "action": "rate_limit_exceeded",
  "ip": "127.0.0.1",
  "path": "/api/projects",
  "retry_after": 60
}
```

---

For additional help, see:
- [README.md](./README.md) - Getting started guide
- [CLAUDE.md](./CLAUDE.md) - Development workflow
- [PRODUCTION_SECURITY_REVIEW.md](./PRODUCTION_SECURITY_REVIEW.md) - Security analysis