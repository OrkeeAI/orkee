# OAuth Authentication

Orkee supports OAuth authentication for AI providers, allowing you to use your subscription accounts (Claude Pro/Max, OpenAI Plus, Google Cloud, xAI Premium) instead of API keys.

## Why Use OAuth?

### Benefits

- **Cost Savings**: Use your Claude Pro/Max subscription instead of paying for API usage
- **Unified Authentication**: Single sign-on for all AI services
- **Automatic Token Management**: Tokens refresh automatically before expiry
- **Secure Storage**: Encrypted token storage in local database
- **Backward Compatible**: Works alongside existing API key authentication

### Supported Providers

| Provider | OAuth Support | Subscription Types | Features |
|----------|--------------|-------------------|----------|
| **Claude** (Anthropic) | ✅ Yes | Pro, Max | Full API access with subscription |
| **OpenAI** | ✅ Yes | Plus, Team, Enterprise | GPT-4, GPT-3.5 access |
| **Google** (Vertex AI) | ✅ Yes | Cloud accounts | Gemini Pro, Flash models |
| **xAI** (Grok) | ✅ Yes | Premium | Grok model access |

## Quick Start

### 1. Authenticate with a Provider

```bash
# Authenticate with Claude
orkee login claude

# This will:
# 1. Open your browser for OAuth authorization
# 2. Start a local callback server
# 3. Securely store your encrypted token
# 4. Display your account information
```

### 2. Check Authentication Status

```bash
orkee auth status

# Output example:
# ✅ claude (Authenticated)
#    Account: user@example.com
#    Subscription: Pro
#    Expires: 2025-11-04 15:30:00 (29 days)
```

### 3. Use OAuth in Your Applications

OAuth tokens are used automatically! The dashboard and CLI will:
1. Check for OAuth tokens first
2. Fall back to API keys if no OAuth token exists
3. Automatically refresh tokens before they expire

## Authentication Commands

### Login to a Provider

```bash
# Login with provider selection
orkee login <provider>

# Examples:
orkee login claude
orkee login openai
orkee login google
orkee login xai

# Force re-authentication
orkee login claude --force
```

**What happens during login:**
1. Browser opens to provider's authorization page
2. You grant Orkee access to your account
3. Callback server receives authorization code
4. Token is exchanged and encrypted
5. Success message displays account details

### Check Authentication Status

```bash
# Show status for all providers
orkee auth status

# Output shows:
# - Authentication state (✅ authenticated, ❌ not authenticated)
# - Account email
# - Subscription type
# - Token expiration time
```

### Refresh a Token

```bash
# Manually refresh a token
orkee auth refresh claude

# Note: Tokens refresh automatically 5 minutes before expiry
# This command is useful for:
# - Testing token refresh
# - Force-refreshing after provider changes
# - Troubleshooting authentication issues
```

### Logout from a Provider

```bash
# Logout from specific provider
orkee logout claude

# Logout from all providers
orkee logout all

# This will:
# - Remove encrypted tokens from database
# - Fall back to API keys (if configured)
```

## Authentication Preferences

Configure how Orkee chooses between OAuth and API keys:

```bash
# Try OAuth first, fall back to API keys (default)
orkee config set auth_preference hybrid

# Use OAuth only (no API key fallback)
orkee config set auth_preference oauth

# Use API keys only (no OAuth)
orkee config set auth_preference api_key
```

### Preference Modes

| Mode | Behavior | Use Case |
|------|----------|----------|
| **hybrid** (default) | OAuth first → API key fallback | Best for most users |
| **oauth** | OAuth only, error if not authenticated | Force OAuth usage |
| **api_key** | API keys only, ignore OAuth tokens | Legacy/testing |

## Dashboard Integration

### Viewing OAuth Status

1. Navigate to **Settings** → **OAuth Authentication**
2. View authentication status for all providers:
   - ✅ Green checkmark: Authenticated
   - ❌ Red X: Not authenticated
   - ⚠️ Yellow warning: Token expiring soon
3. See account email and subscription type
4. View token expiration time

### Disconnecting Providers

Click the **Disconnect** button next to any authenticated provider to logout.

**Note:** CLI login is preferred as it handles the OAuth flow more reliably than browser-based flows.

## Security

### Token Storage

- **Location**: `~/.orkee/orkee.db` (oauth_tokens table)
- **Encryption**: ChaCha20-Poly1305 AEAD encryption
- **Unique Nonces**: Each token encrypted with unique nonce
- **File Permissions**: Database file is user-read/write only

### OAuth Security Features

- **PKCE**: Proof Key for Code Exchange (RFC 7636) prevents authorization code interception
- **State Parameter**: CSRF protection with cryptographically secure random state
- **Localhost Callback**: Callback server binds to 127.0.0.1 only (no network exposure)
- **Automatic Refresh**: Tokens refresh 5 minutes before expiry
- **Encrypted Storage**: All tokens encrypted at rest

### Best Practices

✅ **Do:**
- Use OAuth for Claude Pro/Max to avoid API costs
- Set `auth_preference=hybrid` for fallback protection
- Monitor token expiry with `orkee auth status`
- Logout when not using a provider: `orkee logout <provider>`

❌ **Don't:**
- Share your `~/.orkee/orkee.db` file
- Manually edit OAuth tokens in the database
- Use the same machine for multiple user accounts
- Ignore token expiration warnings

## Troubleshooting

### Browser Doesn't Open

**Symptoms**: Login command completes but no browser window opens.

**Solution:**
```bash
# The login command will display a URL
# Copy and paste it into your browser manually
orkee login claude

# Look for output like:
# If browser doesn't open, visit: https://console.anthropic.com/oauth/authorize?...
```

### Token Expired

**Symptoms**: API requests fail with authentication error.

**Solution:**
```bash
# Check token status
orkee auth status

# Refresh the token
orkee auth refresh claude

# Or re-authenticate
orkee login claude --force
```

### Callback Server Port Conflict

**Symptoms**: Error about port 3737 already in use.

**Solution:**
```bash
# Check what's using the port
lsof -i :3737

# Change the OAuth callback port
export OAUTH_CALLBACK_PORT=3738
orkee login claude
```

### Authentication Stuck

**Symptoms**: Login process hangs after browser authorization.

**Solution:**
1. Cancel the login process (Ctrl+C)
2. Check for callback server issues:
   ```bash
   # Ensure no orphaned servers
   pkill -f orkee
   ```
3. Try again with explicit port:
   ```bash
   OAUTH_CALLBACK_PORT=3737 orkee login claude
   ```

### Token Not Found

**Symptoms**: `orkee auth status` shows "Not authenticated" after successful login.

**Solution:**
```bash
# Check database integrity
sqlite3 ~/.orkee/orkee.db "SELECT COUNT(*) FROM oauth_tokens;"

# If count is 0, re-authenticate
orkee login claude

# If count > 0 but status shows not authenticated:
# Check encryption settings
orkee security status
```

### Multiple Users on Same Machine

**Symptoms**: OAuth tokens conflict between users.

**Issue**: Orkee stores tokens per-user in `~/.orkee/`. Each user has separate tokens.

**Solution:**
```bash
# Each user should authenticate separately
orkee login claude

# Tokens are stored in user's home directory
# ~/.orkee/orkee.db
```

## Migration from API Keys

### Step 1: Check Current Setup

```bash
# Check if API keys are configured
env | grep API_KEY

# Check OAuth status
orkee auth status
```

### Step 2: Add OAuth

```bash
# Login with your subscription account
orkee login claude

# API keys remain as fallback (hybrid mode)
```

### Step 3: Test OAuth

```bash
# Make a test request
# Dashboard/CLI will use OAuth automatically

# Verify OAuth is being used in logs
# Look for OAuth token usage messages
```

### Step 4: Remove API Keys (Optional)

```bash
# Once OAuth is working, you can remove API keys
orkee config delete ANTHROPIC_API_KEY

# Warning: Only do this if you're confident OAuth is working
# Keep hybrid mode enabled for safety
```

## Advanced Configuration

### Custom OAuth Client

If you have custom OAuth client credentials:

```bash
# Set custom client ID and redirect URI
export OAUTH_CLAUDE_CLIENT_ID=your-client-id
export OAUTH_CLAUDE_REDIRECT_URI=http://localhost:3737/callback
export OAUTH_CLAUDE_SCOPES="model:claude account:read"

# Login with custom configuration
orkee login claude
```

### Adjust Token Refresh Buffer

```bash
# Default: refresh 5 minutes before expiry
# Change to 10 minutes:
export OAUTH_TOKEN_REFRESH_BUFFER_SECS=600

orkee auth refresh claude
```

### Custom Callback Port

```bash
# Change OAuth callback server port
export OAUTH_CALLBACK_PORT=8080

orkee login claude
```

## Frequently Asked Questions

### Does OAuth work offline?

No, OAuth requires internet access for:
- Initial authentication
- Token refresh

However, once authenticated, tokens are valid for the provider's token lifetime (typically 30-90 days) and can be used offline until they expire.

### Can I use OAuth and API keys together?

Yes! The default `hybrid` mode uses OAuth first and falls back to API keys if OAuth tokens are unavailable or expired.

### How often do tokens need to be refreshed?

Tokens refresh automatically 5 minutes before expiry. You'll rarely need to manually refresh tokens.

### What data does Orkee collect during OAuth?

Orkee only collects:
- Access token (encrypted)
- Refresh token (encrypted)
- Token expiry time
- Account email (for display)
- Subscription type (for display)

No other data is collected or transmitted.

### Can I revoke OAuth access?

Yes, in two ways:

1. **From Orkee:**
   ```bash
   orkee logout claude
   ```

2. **From Provider:**
   Visit your provider's account settings and revoke Orkee's OAuth access.

## Related Documentation

- **[OAUTH_SETUP.md](../../OAUTH_SETUP.md)** - Complete OAuth setup guide
- **[SECURITY_AUDIT.md](../../SECURITY_AUDIT.md)** - OAuth security audit
- **[oauth.md](../../oauth.md)** - Implementation plan and status
- **[SECURITY.md](../../SECURITY.md)** - OAuth security architecture

## Support

For OAuth-related issues:

1. Check troubleshooting section above
2. Review [OAUTH_SETUP.md](../../OAUTH_SETUP.md)
3. Open an issue on GitHub with:
   - OAuth provider (claude, openai, google, xai)
   - Command you ran
   - Error message (sanitized - no tokens!)
   - Output of `orkee auth status`
