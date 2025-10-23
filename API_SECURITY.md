# API Security Documentation

## Overview

Orkee implements API token authentication to secure API endpoints while maintaining a simple, local-first design. This document explains the authentication system, security model, and best practices.

## Authentication Model

### Design Philosophy

Orkee is designed as a **local-first desktop application** with optional cloud sync, not a traditional web service. The authentication system reflects this:

- **Local-first**: App runs on localhost, user owns the machine
- **Simple API tokens**: No complex OAuth flows or user login required
- **Automatic token generation**: Token created on first startup
- **Optional cloud authentication**: Separate OAuth for cloud sync features
- **Defense in depth**: Protects against localhost malware and cross-origin attacks

### Security Model

**Trust Boundaries**:
- **Trusted**: User's local machine and processes they control
- **Untrusted**: Cross-origin requests, malicious localhost services, exported databases

**What is Protected**:
- API endpoints (except health/status/csrf-token)
- Settings management
- Project operations
- Preview server control

**What is NOT Protected by Default**:
- Local file system access (controlled by OS permissions)
- Database file (`~/.orkee/orkee.db` - protected by file permissions)
- Configuration files

---

## API Token System

### Token Generation

Tokens are automatically generated on first startup:

1. **First Run Detection**: Server checks if any active tokens exist in database
2. **Token Generation**: If none exist, generates a 32-byte cryptographically secure token
3. **Token Display**: Shows token once in console output
4. **File Storage**: Saves token to `~/.orkee/api-token` with 0600 permissions (Unix)
5. **Database Storage**: Stores SHA-256 hash of token in `api_tokens` table

**Token Format**:
- 32 random bytes encoded as base64 (URL-safe, no padding)
- Example: `mK3tN9xQ8vR2jP7wL4yF6hS1dC5bA0zX8uI2oE9gT7r`
- Length: 43 characters

**Token Hash**:
- SHA-256 hash of token
- Stored in database for verification
- Never shown to user after initial generation

### Token Storage

#### File Storage: `~/.orkee/api-token`

**Location**: `~/.orkee/api-token`

**Permissions** (Unix):
- File mode: `0600` (owner read/write only)
- Prevents other users from reading token

**Windows**:
- Uses Windows file permissions
- Accessible only to current user

**Purpose**:
- Allows Tauri desktop app to read token automatically
- Enables manual API testing with curl
- Backup for token recovery

#### Database Storage: `api_tokens` Table

**Schema**:
```sql
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP NULL,
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_api_tokens_is_active ON api_tokens(is_active);
CREATE INDEX idx_api_tokens_token_hash ON api_tokens(token_hash);
```

**Fields**:
- `id`: UUID for token identification
- `token_hash`: SHA-256 hash for verification (never the plaintext token)
- `name`: Human-readable name (e.g., "default", "ci-token")
- `created_at`: Token creation timestamp
- `last_used_at`: Last successful authentication (updated on each use)
- `is_active`: Boolean flag (0 = revoked, 1 = active)

---

## Authentication Flow

### API Request Authentication

```
1. Client includes token in request
   ├─ Header: X-API-Token: <token>
   └─ Example: curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/projects

2. Middleware checks if path requires authentication
   ├─ Whitelisted paths bypass check: /api/health, /api/status, /api/csrf-token
   ├─ Development mode bypass: If ORKEE_DEV_MODE=true, skip authentication entirely
   └─ Protected paths continue to token validation

3. Token extraction
   ├─ Extract X-API-Token header value
   └─ Return 401 if missing

4. Token verification
   ├─ Hash provided token with SHA-256
   ├─ Query database for matching hash where is_active = 1
   ├─ Perform constant-time comparison (prevents timing attacks)
   └─ Return 401 if no match or inactive

5. Update last used timestamp
   ├─ Update last_used_at in database
   └─ Non-fatal error (logged but doesn't block request)

6. Request proceeds
   ├─ Store authentication status in request extensions
   └─ Pass to handler
```

### Desktop App Authentication

The Tauri desktop app handles authentication automatically:

```
1. App startup
   └─ Read token from ~/.orkee/api-token

2. API requests
   ├─ Dashboard detects platform (Tauri vs web)
   ├─ Tauri (production): Include token from file
   └─ Web (development): No token needed (ORKEE_DEV_MODE=true bypasses auth)

3. Token refresh
   ├─ Token cached in memory
   ├─ Re-read on 401 errors (handles token rotation)
   └─ Prompt user if file missing
```

### Development Mode Bypass

When `ORKEE_DEV_MODE=true` is set (automatically enabled by `orkee dashboard --dev`):

- **Authentication is completely bypassed** for all API endpoints
- **Web dashboard works without tokens** - no need for the browser to access `~/.orkee/api-token`
- **Only works on localhost** - server binds to 127.0.0.1, not accessible from network
- **Production Tauri app** - Uses full authentication with tokens from file

**Security Model**:
- Development: Localhost-only, single-user, trusted environment = no auth needed
- Production: Desktop app with file-based tokens = full authentication required

---

## Endpoint Protection

### Whitelisted Endpoints (No Authentication Required)

These endpoints are accessible without a token:

- **`GET /api/health`** - Basic health check
- **`GET /api/status`** - Detailed service status
- **`GET /api/csrf-token`** - CSRF token retrieval

**Rationale**: Health/status endpoints needed for monitoring and liveness probes. CSRF tokens needed before authentication can occur.

### Protected Endpoints (Authentication Required)

All other API endpoints require authentication:

- **Projects API** - `/api/projects/*`
- **Settings API** - `/api/settings/*`
- **Preview Servers** - `/api/preview/*`
- **Directory Browsing** - `/api/browse-directories`
- **Tasks & Specs** - `/api/tasks/*`, `/api/specs/*`

---

## Security Features

### 1. Token Hashing (SHA-256)

**Why**: Protects against database export attacks

**Implementation**:
```rust
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
```

**Benefit**: If database is exported, attacker cannot recover plaintext tokens.

### 2. Constant-Time Comparison

**Why**: Prevents timing attacks that could leak token information

**Implementation**:
```rust
use subtle::ConstantTimeEq;

pub fn verify_token_hash(token: &str, stored_hash: &str) -> bool {
    let computed_hash = Self::hash_token(token);
    computed_hash.as_bytes().ct_eq(stored_hash.as_bytes()).into()
}
```

**Benefit**: Token verification takes same time regardless of how many characters match.

### 3. is_env_only Protection

**Why**: Prevents API modification of critical bootstrap settings

**Protected Settings**:
- `api_port` - API server port
- `ui_port` - Dashboard UI port
- `dev_mode` - Development mode flag

**Implementation**: Server-side enforcement before database update:
```rust
// Check if setting is environment-only
if setting.is_env_only {
    return Err(StorageError::Validation(
        format!("Cannot modify environment-only setting: {}", key)
    ));
}
```

**Benefit**: These settings cannot be changed via API, only via `.env` file or CLI flags.

### 4. Input Validation

All setting values validated before database update. See [Input Validation](#input-validation) section.

### 5. Atomic Bulk Updates

**Why**: Prevents partial configuration corruption

**Implementation**: SQLite transactions with rollback on any validation failure

**Benefit**: Bulk updates are all-or-nothing - either all settings update or none do.

---

## Input Validation

### Validation Rules

All settings undergo type-specific and setting-specific validation:

**Port Numbers** (1-65535):
```rust
validate_port("80")      // ✓ OK
validate_port("443")     // ✓ OK
validate_port("0")       // ✗ Error: Invalid port
validate_port("65536")   // ✗ Error: Invalid port
validate_port("abc")     // ✗ Error: Not an integer
```

**Boolean Values** (only "true" or "false"):
```rust
validate_boolean("true")   // ✓ OK
validate_boolean("false")  // ✓ OK
validate_boolean("yes")    // ✗ Error: Invalid boolean
validate_boolean("1")      // ✗ Error: Invalid boolean
validate_boolean("True")   // ✗ Error: Invalid boolean (case-sensitive)
```

**Enum Values** (allowed values only):
```rust
validate_enum("strict", &["strict", "relaxed", "disabled"])     // ✓ OK
validate_enum("relaxed", &["strict", "relaxed", "disabled"])    // ✓ OK
validate_enum("invalid", &["strict", "relaxed", "disabled"])    // ✗ Error
```

**Paths** (no path traversal):
```rust
validate_path("/home/user/file.txt")     // ✓ OK
validate_path("~/Documents")             // ✓ OK
validate_path("../../../etc/passwd")     // ✗ Error: Path traversal detected
```

**URLs** (http:// or https:// only):
```rust
validate_url("https://api.orkee.ai")                    // ✓ OK
validate_url("http://localhost:3000")                   // ✓ OK
validate_url("invalid-url")                             // ✗ Error: Must start with http:// or https://
validate_url("https://example.com/path with spaces")   // ✗ Error: Cannot contain spaces
```

**Rate Limits** (must be >= 1):
```rust
validate_integer("60", Some(1), None)    // ✓ OK
validate_integer("1", Some(1), None)     // ✓ OK
validate_integer("0", Some(1), None)     // ✗ Error: Must be >= 1
validate_integer("-1", Some(1), None)    // ✗ Error: Must be >= 1
```

### Error Responses

**400 Bad Request** - Validation errors:
```json
{
  "success": false,
  "error": "Validation failed: Invalid port number: 65536. Must be between 1 and 65535",
  "request_id": "abc123-def456"
}
```

**403 Forbidden** - is_env_only violations:
```json
{
  "success": false,
  "error": "Cannot modify environment-only setting: api_port",
  "request_id": "abc123-def456"
}
```

**401 Unauthorized** - Authentication failures:
```json
{
  "success": false,
  "error": "API token required. Please include X-API-Token header.",
  "request_id": "abc123-def456"
}
```

---

## Client Integration

### Tauri Desktop App

**Implementation** (`packages/dashboard/src-tauri/src/lib.rs`):

```rust
#[tauri::command]
async fn get_api_token() -> Result<String, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Unable to determine home directory".to_string())?;

    let token_path = home.join(".orkee").join("api-token");

    fs::read_to_string(&token_path)
        .map(|t| t.trim().to_string())
        .map_err(|e| format!("Failed to read API token: {}", e))
}
```

**Frontend Usage** (`packages/dashboard/src/lib/platform.ts`):

```typescript
export async function getApiToken(): Promise<string | null> {
  if (isDesktop()) {
    try {
      const token = await invoke<string>('get_api_token');
      return token;
    } catch (error) {
      console.error('Failed to get API token:', error);
      return null;
    }
  }
  return null; // Web mode doesn't use tokens in development
}
```

**API Client** (`packages/dashboard/src/services/api.ts`):

```typescript
async get<T>(endpoint: string, options?: RequestOptions): Promise<ApiResponse<T>> {
  const token = await getApiToken();
  const headers: HeadersInit = {
    ...options?.headers,
  };

  if (token) {
    headers['X-API-Token'] = token;
  }

  // ... rest of request
}
```

### Manual API Testing (curl)

```bash
# Read token from file
export TOKEN=$(cat ~/.orkee/api-token)

# Make authenticated request
curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/projects

# Alternative: inline token
curl -H "X-API-Token: $(cat ~/.orkee/api-token)" http://localhost:4001/api/projects
```

### Custom Clients

Any HTTP client can authenticate by including the token header:

**JavaScript (fetch)**:
```javascript
const token = await fs.promises.readFile(
  path.join(os.homedir(), '.orkee', 'api-token'),
  'utf8'
);

const response = await fetch('http://localhost:4001/api/projects', {
  headers: {
    'X-API-Token': token.trim()
  }
});
```

**Python (requests)**:
```python
import os
import requests

token_path = os.path.join(os.path.expanduser('~'), '.orkee', 'api-token')
with open(token_path, 'r') as f:
    token = f.read().strip()

response = requests.get(
    'http://localhost:4001/api/projects',
    headers={'X-API-Token': token}
)
```

**Rust (reqwest)**:
```rust
use std::fs;
use dirs;

let home = dirs::home_dir().unwrap();
let token_path = home.join(".orkee").join("api-token");
let token = fs::read_to_string(token_path)?.trim().to_string();

let response = reqwest::Client::new()
    .get("http://localhost:4001/api/projects")
    .header("X-API-Token", token)
    .send()
    .await?;
```

---

## Token Management

### Future Features

The following token management features are planned but not yet implemented:

**Token Rotation**:
```bash
orkee tokens regenerate        # Generate new token, invalidate old
orkee tokens list              # List all tokens
orkee tokens revoke <id>       # Revoke specific token
```

**Multiple Tokens**:
- Named tokens for different purposes (e.g., "ci", "testing", "admin")
- Separate tokens for different apps/scripts
- Token-specific permissions (future: role-based access control)

**Token Expiration**:
- Optional expiration dates
- Auto-renewal
- Expiration warnings

### Current Limitations

**Single Token Only**: Currently only one token supported per installation

**No Token Rotation**: Old token remains valid indefinitely

**No UI Management**: Token management must be done via CLI or database

**Workaround for Token Reset**:
If you need to reset the token:
1. Stop Orkee server
2. Delete token from database:
   ```sql
   sqlite3 ~/.orkee/orkee.db "DELETE FROM api_tokens;"
   ```
3. Delete token file:
   ```bash
   rm ~/.orkee/api-token
   ```
4. Restart server - new token will be generated

---

## Security Considerations

### Threats Mitigated

✅ **Cross-Origin Requests**: Tokens prevent unauthorized access from web browsers

✅ **Malicious Localhost Services**: Other services cannot access Orkee API without token

✅ **Database Export Attacks**: Tokens stored as SHA-256 hashes

✅ **Timing Attacks**: Constant-time comparison prevents token leakage

✅ **Configuration Corruption**: Input validation prevents invalid values

✅ **Partial Updates**: Atomic transactions prevent partial configuration changes

### Threats NOT Mitigated

⚠️ **Local Process Inspection**: Processes with debugging permissions can read token from memory

⚠️ **File System Access**: Users with file access can read `~/.orkee/api-token`

⚠️ **Network Sniffing**: Tokens sent over HTTP in local network (use HTTPS if concerned)

⚠️ **Compromised Desktop App**: Malicious code in Tauri app has full access

### Best Practices

**For Desktop Users**:
- Use standard OS file permissions (automatic with Orkee)
- Don't share `~/.orkee/` directory
- Don't commit `api-token` file to version control
- Use HTTPS if accessing over network (optional)

**For CI/CD**:
- Generate dedicated token for CI (future feature)
- Store token in CI environment variables
- Don't log token in CI output
- Rotate tokens periodically (when feature available)

**For Developers**:
- Don't hardcode tokens in source code
- Read token from `~/.orkee/api-token` at runtime
- Handle 401 errors gracefully (token may have changed)
- Include `X-API-Token` header in all API requests

---

## Troubleshooting

### 401 Unauthorized Errors

**Symptom**: API calls return "API token required" or "Invalid API token"

**Solutions**:
1. Verify token file exists:
   ```bash
   cat ~/.orkee/api-token
   ```

2. Check token is included in request:
   ```bash
   curl -v -H "X-API-Token: $(cat ~/.orkee/api-token)" http://localhost:4001/api/projects
   ```

3. Verify server is running:
   ```bash
   curl http://localhost:4001/api/health
   ```

4. Check server logs for authentication errors

### Token File Missing

**Symptom**: `~/.orkee/api-token` file doesn't exist

**Solution**:
1. Stop Orkee server
2. Delete database to reset:
   ```bash
   rm ~/.orkee/orkee.db
   ```
3. Restart server - new token will be generated

### Dashboard Can't Authenticate

**Symptom**: Dashboard shows authentication errors

**Solutions**:
1. Verify Tauri app can read token file:
   - Check file permissions: `ls -la ~/.orkee/api-token`
   - Should be readable by current user

2. Restart desktop app completely:
   - Quit from system tray
   - Relaunch application

3. Check browser console for errors

4. Verify API server is running on correct port

### Database Locked Errors

**Symptom**: "Database is locked" errors

**Solution**:
1. Stop all Orkee processes:
   ```bash
   pkill orkee
   ```

2. Wait a few seconds

3. Restart:
   ```bash
   orkee dashboard
   ```

---

## Related Documentation

- **[MANUAL_TESTING.md](MANUAL_TESTING.md)** - Manual testing procedures
- **[SECURITY.md](SECURITY.md)** - Overall security documentation
- **[DOCS.md](DOCS.md)** - Complete configuration reference
- **[fixpr9.md](fixpr9.md)** - Implementation details for PR #9 security fixes

---

## Support

For security issues or questions:
- **GitHub Issues**: https://github.com/OrkeeAI/orkee/issues
- **Discussions**: https://github.com/OrkeeAI/orkee/discussions
- **Email**: For security vulnerabilities, please report privately

---

**Last Updated**: 2025-10-23
**Version**: 1.0 (Phase 6 Complete)
