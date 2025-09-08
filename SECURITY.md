# Security Overview

Orkee implements comprehensive security measures designed for both local development and production deployments. This document outlines our security architecture, threat model, and implemented protections.

## Security Philosophy

Orkee follows a **defense-in-depth** strategy with **zero-trust principles**, implementing multiple layers of security even for local development use. Our approach prioritizes:

- **Secure by default** - Safe configurations out of the box
- **Principle of least privilege** - Minimal access rights
- **Defense in depth** - Multiple security layers
- **Transparent security** - Clear security boundaries and controls

## Threat Model

### Attack Vectors Addressed

| Threat | Protection | Implementation |
|--------|------------|----------------|
| **Path Traversal** | Directory sandboxing | Configurable sandbox modes with path validation |
| **Command Injection** | Input validation | Regex patterns and dangerous command detection |
| **CSRF Attacks** | CORS restrictions | Strict localhost-only origins in development |
| **Rate Limit Bypass** | Per-IP rate limiting | Token bucket algorithm with configurable limits |
| **Information Disclosure** | Sanitized errors | Request ID tracking with safe error responses |
| **Clickjacking** | Security headers | X-Frame-Options, CSP, and comprehensive headers |
| **TLS Attacks** | Modern encryption | TLS 1.2/1.3 with secure cipher suites |
| **Privilege Escalation** | Process isolation | Non-privileged user, system call restrictions |

### Assumptions & Boundaries

**Trust Boundaries:**
- **Local Development**: Single-user development machine
- **Production**: Multi-user server environment with network access
- **Network**: Assumes hostile network in production

**Assumed Threats:**
- Malicious directory traversal attempts
- Automated vulnerability scanning
- Brute force and DoS attacks
- Man-in-the-middle attacks (production)
- Malicious input injection

**Out of Scope:**
- Physical access to the host machine
- Compromised host operating system
- Social engineering attacks on administrators

## Security Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Layers                         │
├─────────────────────────────────────────────────────────────┤
│ 1. TLS/HTTPS Encryption (Transport Layer)                  │
│    • TLS 1.2/1.3 with secure ciphers                     │
│    • Certificate validation and auto-renewal              │
│    • HTTPS redirect for all HTTP traffic                  │
├─────────────────────────────────────────────────────────────┤
│ 2. Network Security (Application Layer)                    │
│    • CORS origin validation                               │
│    • Rate limiting per IP address                         │
│    • Security headers (HSTS, CSP, X-Frame-Options)       │
├─────────────────────────────────────────────────────────────┤
│ 3. Input Validation (Request Layer)                        │
│    • Path traversal detection and prevention              │
│    • Command injection filtering                          │
│    • Input sanitization and length limits                 │
├─────────────────────────────────────────────────────────────┤
│ 4. Access Control (Resource Layer)                         │
│    • Directory sandboxing with configurable modes         │
│    • File system boundary enforcement                     │
│    • Sensitive directory blocking                         │
├─────────────────────────────────────────────────────────────┤
│ 5. Error Handling (Response Layer)                         │
│    • Information disclosure prevention                    │
│    • Request ID tracking for audit trails                 │
│    • Sanitized error responses                           │
└─────────────────────────────────────────────────────────────┘
```

## Security Features

### 1. Transport Layer Security (TLS/HTTPS)

**Implementation:**
- Native Rust TLS using rustls library
- Automatic certificate generation for development
- Support for custom certificates in production
- Dual server mode with HTTP-to-HTTPS redirect

**Configuration:**
```bash
TLS_ENABLED=true                 # Enable HTTPS
AUTO_GENERATE_CERT=true          # Auto-generate dev certificates
ENABLE_HSTS=true                 # HTTP Strict Transport Security
```

**Security Benefits:**
- Encrypts all data in transit
- Prevents man-in-the-middle attacks
- Provides authentication via certificates
- Enables secure browser features (HSTS, secure cookies)

### 2. Cross-Origin Resource Sharing (CORS)

**Implementation:**
- Strict allowlist of permitted origins
- Only localhost domains allowed in development
- Configurable origin validation
- Explicit credential and method restrictions

**Configuration:**
```bash
CORS_ORIGIN="http://localhost:5173"     # Specific allowed origin
CORS_ALLOW_ANY_LOCALHOST=true           # Dev mode flexibility
```

**Blocked Origins:**
- Any non-localhost origin in development mode
- Invalid or malformed origins
- Origins not matching the configured allowlist

### 3. Rate Limiting

**Implementation:**
- Per-IP address tracking using token bucket algorithm
- Endpoint-specific rate limits
- Configurable burst sizes for legitimate traffic spikes
- Redis-compatible for distributed deployments

**Default Limits:**
```bash
RATE_LIMIT_HEALTH_RPM=60        # Health endpoints
RATE_LIMIT_BROWSE_RPM=20        # Directory browsing
RATE_LIMIT_PROJECTS_RPM=30      # Project operations
RATE_LIMIT_GLOBAL_RPM=30        # Default for other endpoints
```

**Protection Against:**
- Brute force attacks
- Automated vulnerability scanners
- Resource exhaustion (DoS)
- API abuse

### 4. Directory Sandboxing

**Three Security Modes:**

#### Strict Mode (`BROWSE_SANDBOX_MODE=strict`)
- **Allowlist only**: Access restricted to explicitly configured paths
- **Zero path traversal**: All `../` navigation blocked
- **Maximum security**: Suitable for production environments

#### Relaxed Mode (`BROWSE_SANDBOX_MODE=relaxed`) - Default
- **Blocklist approach**: Block dangerous system paths
- **Controlled traversal**: `../` allowed within safe boundaries  
- **Home directory access**: Full user home directory access
- **Development friendly**: Balance of security and usability

#### Disabled Mode (`BROWSE_SANDBOX_MODE=disabled`) - Not Recommended
- **No restrictions**: Access to any readable directory
- **Debug only**: Use only in completely trusted environments

**Always Blocked Paths:**
```
/etc, /sys, /proc, /dev, /boot, /root
/usr/bin, /usr/sbin, /bin, /sbin
/var/log, /var/run, /tmp
~/.ssh, ~/.aws, ~/.gnupg, ~/.docker
C:\Windows, C:\System32 (Windows)
```

### 5. Input Validation

**Path Validation:**
- Canonical path resolution to prevent bypasses
- Null byte injection prevention
- Unicode normalization to prevent encoding attacks
- Length limits to prevent buffer overflows

**Command Injection Prevention:**
```rust
// Dangerous patterns detected:
- Shell metacharacters: |, &, ;, `, $, etc.
- Command separators: &&, ||, ;
- Redirection operators: >, >>, <
- Process substitution: $(), ``
- Malicious commands: rm, dd, curl, wget
```

**Input Sanitization:**
- Project names: 100 character limit, alphanumeric + safe specials
- Descriptions: 1000 character limit
- Script content: Dangerous command detection
- File paths: Canonical resolution and sandbox validation

### 6. Security Headers

**Implemented Headers:**
```http
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY  
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), camera=(), microphone=()
Strict-Transport-Security: max-age=31536000; includeSubDomains (when HSTS enabled)
```

**Protection Against:**
- Content injection attacks
- Clickjacking
- MIME type confusion
- Information leakage via referrers
- Dangerous browser API access

### 7. Error Handling & Logging

**Sanitized Responses:**
- No internal error details exposed to clients
- Consistent error format: `{success: false, error: {...}, request_id: "..."}`
- Stack traces logged server-side only
- Request ID correlation for audit trails

**Audit Logging:**
```rust
// Security events logged:
- Rate limit violations (with IP addresses)
- Path traversal attempts
- Invalid authentication attempts  
- Configuration errors
- Certificate validation failures
```

## Security Best Practices

### Development Environment

```bash
# Minimal secure development setup
TLS_ENABLED=false                    # HTTPS not required for localhost
BROWSE_SANDBOX_MODE=relaxed          # Balanced security/usability
RATE_LIMIT_ENABLED=true              # Protect against accidental DoS
SECURITY_HEADERS_ENABLED=true        # Practice secure defaults
CORS_ALLOW_ANY_LOCALHOST=true        # Development flexibility
```

### Production Environment

```bash
# Production security configuration
TLS_ENABLED=true                     # Always use HTTPS
ENABLE_HSTS=true                     # Enforce HTTPS in browsers
BROWSE_SANDBOX_MODE=strict           # Maximum directory protection
RATE_LIMIT_ENABLED=true              # Essential for public access
SECURITY_HEADERS_ENABLED=true        # Full header protection
CORS_ALLOW_ANY_LOCALHOST=false       # Explicit origin control
AUTO_GENERATE_CERT=false             # Use proper CA certificates
```

### Infrastructure Security

**Reverse Proxy (Recommended):**
- Use Nginx/Apache for TLS termination
- Implement additional rate limiting
- Add WAF (Web Application Firewall) protection
- Configure proper SSL/TLS settings

**Container Security:**
- Run as non-privileged user
- Use read-only root filesystem where possible
- Limit system capabilities
- Implement resource limits

**Network Security:**
- Firewall rules limiting access to necessary ports only
- VPN or private networks for administrative access
- Load balancer with DDoS protection
- Network segmentation

## Vulnerability Reporting

### Security Contact

**For security vulnerabilities, please DO NOT open public issues.**

Instead, report security issues privately to:
- **Email**: security@orkee.dev (if available)
- **GitHub**: Use private vulnerability reporting feature
- **Response Time**: We aim to respond within 48 hours

### What to Include

1. **Description**: Clear description of the vulnerability
2. **Impact**: Potential security impact and affected systems
3. **Reproduction**: Step-by-step reproduction instructions
4. **Environment**: Affected versions and configurations
5. **Suggested Fix**: If you have recommendations

### Response Process

1. **Acknowledgment** (48 hours): We confirm receipt and begin investigation
2. **Assessment** (5 days): Severity assessment and impact analysis
3. **Development** (varies): Fix development and testing
4. **Disclosure** (coordinated): Public disclosure after fix is available

## Security Auditing

### Self-Assessment Checklist

**Configuration Security:**
- [ ] TLS enabled for production deployments
- [ ] Strong certificates from trusted CA
- [ ] Rate limiting enabled and properly configured
- [ ] Directory sandbox set to appropriate mode
- [ ] CORS origins properly restricted
- [ ] Security headers enabled
- [ ] Error logging configured

**Infrastructure Security:**
- [ ] Application running as non-privileged user
- [ ] Firewall configured to block unnecessary ports
- [ ] Regular security updates applied
- [ ] Log monitoring and alerting configured
- [ ] Backup and recovery procedures tested

**Operational Security:**
- [ ] Certificate renewal automated
- [ ] Health monitoring configured
- [ ] Incident response plan documented
- [ ] Regular security assessments scheduled

### Testing Security Controls

```bash
# Test rate limiting
for i in {1..100}; do curl https://your-domain.com/api/health; done

# Test path traversal protection
curl "https://your-domain.com/api/directories/list" \
  -d '{"path": "../../../etc/passwd"}'

# Test HTTPS redirect
curl -I http://your-domain.com/

# Test security headers
curl -I https://your-domain.com/

# Test CORS restrictions
curl -H "Origin: https://malicious-site.com" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: X-Requested-With" \
  -X OPTIONS https://your-domain.com/api/health
```

## Related Documentation

- **[DOCS.md](DOCS.md)** - Complete configuration reference including all security settings
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Production deployment guide with security hardening
- **[TESTING.md](TESTING.md)** - Security test coverage and validation procedures

## Security Updates

Stay informed about security updates:
- Subscribe to repository releases for security patches
- Monitor security advisories in our documentation
- Follow security best practices for your deployment environment

---

**Last Updated**: December 2024  
**Security Review**: Complete - All critical security controls implemented  
**Next Review**: Scheduled with major version releases