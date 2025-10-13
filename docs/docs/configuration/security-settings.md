---
sidebar_position: 3
---

# Security Settings

This guide covers Orkee's comprehensive security configuration options, including sandboxing, rate limiting, and security headers.

## Directory Browsing Security

Orkee implements a robust security system for directory browsing with three configurable modes.

### Security Modes

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="relaxed" label="Relaxed Mode (Default)" default>

**Configuration:**
```bash
BROWSE_SANDBOX_MODE=relaxed
ALLOWED_BROWSE_PATHS="~/Documents,~/Projects,~/Desktop,~/Downloads"
```

**Behavior:**
- ✅ Home directory access allowed
- ✅ Path traversal (`../`) permitted within safe boundaries
- ❌ System directories blocked
- ❌ Sensitive user directories blocked
- **Use case**: Local development with reasonable security

</TabItem>
<TabItem value="strict" label="Strict Mode">

**Configuration:**
```bash
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS="/home/user/projects,/home/user/documents"
```

**Behavior:**
- ✅ Only explicitly allowed paths accessible
- ❌ Path traversal completely blocked
- ❌ Root access blocked unless explicitly allowed
- ❌ All system directories blocked
- **Use case**: High-security environments, production deployments

</TabItem>
<TabItem value="disabled" label="Disabled Mode">

**Configuration:**
```bash
BROWSE_SANDBOX_MODE=disabled
```

**Behavior:**
- ⚠️ Access to any readable directory
- ⚠️ No security restrictions
- **Use case**: Debugging only, **NOT RECOMMENDED** for normal use

</TabItem>
</Tabs>

### Always Blocked Paths

The following paths are **always blocked** regardless of security mode:

#### System Directories
- `/etc`, `/private/etc` (system configuration)
- `/sys`, `/proc`, `/dev` (virtual filesystems)
- `/usr/bin`, `/usr/sbin`, `/bin`, `/sbin` (system binaries)
- `/var/log`, `/var/run`, `/var/lock` (system runtime)
- `/boot`, `/root`, `/mnt`, `/media`, `/opt`
- `/tmp`, `/var/tmp` (temporary directories)

#### Windows System Directories
- `C:\Windows`, `C:\System32`
- `C:\Program Files`, `C:\Program Files (x86)`
- `C:\ProgramData`

#### Sensitive User Directories
- `.ssh`, `.aws`, `.gnupg` (credentials)
- `.docker`, `.kube` (container configs)
- `.config/git`, `.gitconfig` (Git configuration)
- `.npm`, `.cargo/credentials` (package managers)
- `.env*` files (environment variables)
- `Library/Keychains` (macOS keychain)
- `AppData/Local/Microsoft` (Windows credentials)

### Configuration Examples

#### Development Environment
```bash
# Balanced security for local development
BROWSE_SANDBOX_MODE=relaxed
ALLOWED_BROWSE_PATHS="~/Documents,~/Projects,~/Code,~/Desktop"
```

#### Production Environment
```bash
# High security for production
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS="/var/app/projects,/var/app/uploads"
```

#### Team Development Server
```bash
# Shared server with multiple user directories
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS="/home/team/shared,/home/team/projects,/opt/team-tools"
```

## Rate Limiting

Protect your Orkee instance from abuse with configurable rate limiting.

### Rate Limiting Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_ENABLED` | `true` | Master enable/disable switch |
| `RATE_LIMIT_HEALTH_RPM` | `60` | Health check endpoints (per minute) |
| `RATE_LIMIT_BROWSE_RPM` | `20` | Directory browsing operations |
| `RATE_LIMIT_PROJECTS_RPM` | `30` | Project CRUD operations |
| `RATE_LIMIT_PREVIEW_RPM` | `10` | Preview server operations |
| `RATE_LIMIT_GLOBAL_RPM` | `30` | Default limit for other endpoints |
| `RATE_LIMIT_BURST_SIZE` | `5` | Burst multiplier |

### Rate Limiting Profiles

<Tabs>
<TabItem value="development" label="Development" default>

```bash
# Generous limits for development
RATE_LIMIT_ENABLED=true
RATE_LIMIT_HEALTH_RPM=120
RATE_LIMIT_BROWSE_RPM=40
RATE_LIMIT_PROJECTS_RPM=60
RATE_LIMIT_PREVIEW_RPM=20
RATE_LIMIT_GLOBAL_RPM=60
RATE_LIMIT_BURST_SIZE=10
```

**Characteristics:**
- High limits for local development
- Large burst sizes for testing
- Accommodates rapid development cycles

</TabItem>
<TabItem value="production" label="Production">

```bash
# Conservative limits for production
RATE_LIMIT_ENABLED=true
RATE_LIMIT_HEALTH_RPM=30
RATE_LIMIT_BROWSE_RPM=10
RATE_LIMIT_PROJECTS_RPM=15
RATE_LIMIT_PREVIEW_RPM=5
RATE_LIMIT_GLOBAL_RPM=15
RATE_LIMIT_BURST_SIZE=2
```

**Characteristics:**
- Strict limits to prevent abuse
- Small burst sizes
- Focus on stability and resource protection

</TabItem>
<TabItem value="high-traffic" label="High Traffic">

```bash
# Optimized for high-traffic environments
RATE_LIMIT_ENABLED=true
RATE_LIMIT_HEALTH_RPM=200
RATE_LIMIT_BROWSE_RPM=50
RATE_LIMIT_PROJECTS_RPM=100
RATE_LIMIT_PREVIEW_RPM=30
RATE_LIMIT_GLOBAL_RPM=80
RATE_LIMIT_BURST_SIZE=15
```

**Characteristics:**
- High limits for busy environments
- Larger burst sizes for traffic spikes
- Balanced protection vs accessibility

</TabItem>
</Tabs>

### Rate Limiting Behavior

#### Token Bucket Algorithm
- Each IP address gets its own token bucket
- Tokens replenish at the configured rate
- Burst size determines maximum tokens available
- Requests consume tokens; rejected when bucket is empty

#### HTTP Responses
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 30
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1642680000

{
  "success": false,
  "error": "Rate limit exceeded. Try again in 60 seconds.",
  "request_id": "req_abc123"
}
```

#### Monitoring and Logging
Rate limit violations are automatically logged:
```json
{
  "level": "warn",
  "timestamp": "2024-01-15T10:30:00Z",
  "message": "Rate limit exceeded",
  "ip": "192.168.1.100",
  "endpoint": "/api/projects",
  "limit": 30,
  "request_id": "req_abc123"
}
```

## Security Headers

Orkee implements comprehensive security headers to protect against common web vulnerabilities.

### Security Headers Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SECURITY_HEADERS_ENABLED` | `true` | Enable security headers middleware |
| `ENABLE_HSTS` | `false` | HTTP Strict Transport Security (HTTPS only) |
| `ENABLE_REQUEST_ID` | `true` | Request ID tracking for audit trails |

### Applied Security Headers

#### Content Security Policy (CSP)
```http
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' ws: wss:; font-src 'self' data:; media-src 'self' blob:; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'deny'
```

**Purpose**: Prevents XSS attacks by controlling resource loading

#### Additional Headers
- **X-Content-Type-Options**: `nosniff` - Prevents MIME type sniffing
- **X-Frame-Options**: `DENY` - Prevents clickjacking
- **X-XSS-Protection**: `1; mode=block` - Legacy XSS protection
- **Referrer-Policy**: `strict-origin-when-cross-origin` - Controls referrer information
- **Permissions-Policy**: Disables dangerous browser APIs (camera, microphone, geolocation)

#### HTTPS-Specific Headers
When `ENABLE_HSTS=true` and HTTPS is enabled:
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

### Security Header Profiles

<Tabs>
<TabItem value="development" label="Development" default>

```bash
# Development-friendly security
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=false  # HTTP allowed
ENABLE_REQUEST_ID=true
```

**Characteristics:**
- Relaxed CSP for development tools
- HTTP connections allowed
- Request tracking enabled

</TabItem>
<TabItem value="production" label="Production">

```bash
# Production security hardening
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true   # HTTPS enforced
ENABLE_REQUEST_ID=true
```

**Characteristics:**
- Strict CSP enforcement
- HSTS enforced for HTTPS
- Full audit trail logging

</TabItem>
<TabItem value="disabled" label="Disabled">

```bash
# Security headers disabled (not recommended)
SECURITY_HEADERS_ENABLED=false
ENABLE_HSTS=false
ENABLE_REQUEST_ID=false
```

**Use case**: Legacy compatibility only

</TabItem>
</Tabs>

## Input Validation

Orkee implements comprehensive input validation to prevent various attack vectors.

### Validation Rules

#### Project Data Validation
- **Names**: 100 character limit, alphanumeric + safe special chars
- **Descriptions**: 1000 character limit, HTML stripped
- **Paths**: Path traversal prevention, length limits
- **Tags**: Normalized, duplicate removal

#### Command Injection Prevention
Scripts and commands are validated for dangerous patterns:
- Shell metacharacters (`; && || |`)
- Command substitution (`` `command` ``, `$(command)`)
- File redirection (`> >> < <<`)
- Background execution (`&`)

#### Path Safety Validation
- Canonical path resolution
- Symlink following restrictions
- Hidden file access controls
- Extension-based filtering

### Custom Validation Rules

You can extend validation through environment variables:

```bash
# Custom path validation
ALLOWED_FILE_EXTENSIONS=".js,.ts,.py,.md,.txt"
MAX_FILE_SIZE=10485760  # 10MB

# Custom name validation
PROJECT_NAME_PATTERN="^[a-zA-Z0-9-_]+$"
MAX_PROJECT_NAME_LENGTH=50
```

## Error Handling & Audit Logging

Secure error handling prevents information disclosure while maintaining audit trails.

### Error Sanitization

**Client Response** (safe):
```json
{
  "success": false,
  "error": "Access denied",
  "request_id": "req_abc123"
}
```

**Server Log** (detailed):
```json
{
  "level": "error",
  "timestamp": "2024-01-15T10:30:00Z",
  "message": "Directory access denied: path traversal attempt",
  "ip": "192.168.1.100",
  "path": "/etc/passwd",
  "user_agent": "curl/7.68.0",
  "request_id": "req_abc123"
}
```

### Audit Events

Orkee logs security-relevant events:

#### Authentication Events
```json
{"event": "auth_failure", "ip": "1.2.3.4", "reason": "invalid_token"}
{"event": "auth_success", "ip": "1.2.3.4", "user": "john@example.com"}
```

#### Access Control Events
```json
{"event": "access_denied", "path": "/restricted", "reason": "sandbox_violation"}
{"event": "rate_limit_exceeded", "ip": "1.2.3.4", "endpoint": "/api/projects"}
```

#### System Events
```json
{"event": "server_start", "version": "0.1.0", "security_mode": "relaxed"}
{"event": "config_change", "setting": "rate_limit_enabled", "value": true}
```

## Security Best Practices

### Development Environment

1. **Use relaxed mode** for local development
2. **Enable rate limiting** to catch issues early
3. **Monitor logs** for unexpected behavior
4. **Regular security audits** of allowed paths

```bash
# Recommended development configuration
BROWSE_SANDBOX_MODE=relaxed
ALLOWED_BROWSE_PATHS="~/Documents,~/Projects,~/Code"
RATE_LIMIT_ENABLED=true
SECURITY_HEADERS_ENABLED=true
ENABLE_REQUEST_ID=true
```

### Production Environment

1. **Use strict mode** for directory browsing
2. **Enable all security features**
3. **Monitor and alert** on security events
4. **Regular security updates**
5. **Principle of least privilege**

```bash
# Recommended production configuration
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS="/var/app/safe,/opt/app/public"
RATE_LIMIT_ENABLED=true
SECURITY_HEADERS_ENABLED=true
ENABLE_HSTS=true
ENABLE_REQUEST_ID=true

# Strict rate limits
RATE_LIMIT_GLOBAL_RPM=15
RATE_LIMIT_BURST_SIZE=2
```

### Network Security

1. **Use HTTPS** in production
2. **Configure firewalls** to restrict access
3. **Use reverse proxy** for additional security
4. **Regular security scanning**

### Monitoring and Alerting

Set up monitoring for security events:

```bash
# Monitor rate limit violations
grep "Rate limit exceeded" /var/log/orkee/security.log

# Monitor access violations
grep "access_denied" /var/log/orkee/security.log | jq '.path'

# Monitor authentication failures
grep "auth_failure" /var/log/orkee/security.log
```

## Troubleshooting Security Issues

### Common Security Problems

#### Directory Access Denied
```bash
# Check sandbox configuration
echo $BROWSE_SANDBOX_MODE
echo $ALLOWED_BROWSE_PATHS

# Verify path is allowed
ls -la /requested/path

# Check logs for details
tail -f /var/log/orkee.log | grep access_denied
```

#### Rate Limit Issues
```bash
# Check current limits
curl -I http://localhost:4001/api/health

# Monitor rate limit headers
curl -v http://localhost:4001/api/projects

# Adjust limits if needed
RATE_LIMIT_PROJECTS_RPM=60 orkee dashboard
```

#### CSP Violations
```bash
# Check browser console for CSP errors
# Adjust CSP in production if needed (not recommended)

# Temporary CSP disable for debugging
SECURITY_HEADERS_ENABLED=false orkee dashboard
```

