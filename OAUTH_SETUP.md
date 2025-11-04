# OAuth Authentication Setup Guide

This guide provides detailed instructions for setting up OAuth authentication with AI providers in Orkee.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Provider Setup](#provider-setup)
  - [Claude (Anthropic)](#claude-anthropic)
  - [OpenAI](#openai)
  - [Google (Vertex AI)](#google-vertex-ai)
  - [xAI (Grok)](#xai-grok)
- [Configuration](#configuration)
- [Security](#security)
- [Troubleshooting](#troubleshooting)
- [Advanced Usage](#advanced-usage)

## Overview

OAuth authentication allows you to use your AI provider subscription accounts (Claude Pro/Max, OpenAI Plus, etc.) instead of API keys. Benefits include:

- **Cost savings**: Use your existing subscriptions instead of paying for API access
- **Simplified management**: One authentication per provider
- **Automatic refresh**: Tokens refresh automatically before expiry
- **Secure storage**: Encrypted token storage in local SQLite database
- **Backward compatible**: Works alongside API key authentication

## Quick Start

### 1. Authenticate with a Provider

```bash
# Authenticate with Claude
orkee login claude

# Or specify provider interactively
orkee login
```

This will:
1. Generate a secure PKCE challenge
2. Open your browser to the provider's OAuth page
3. Start a local callback server on port 3737
4. Exchange the authorization code for an access token
5. Store the encrypted token in `~/.orkee/orkee.db`

### 2. Check Authentication Status

```bash
orkee auth status
```

Expected output:
```
OAuth Authentication Status:

Claude (Anthropic)
├─ Status: ✅ Authenticated
├─ Account: user@example.com
├─ Subscription: Pro
└─ Expires: 2024-01-15 10:30:45 UTC (59 minutes)

OpenAI
└─ Status: ❌ Not authenticated
```

### 3. Use OAuth Tokens

Once authenticated, Orkee automatically uses OAuth tokens:

```bash
# Dashboard automatically uses OAuth tokens
orkee dashboard

# CLI commands use OAuth tokens
orkee projects list

# TUI uses OAuth tokens
orkee tui
```

## Provider Setup

### Claude (Anthropic)

**Supported Subscriptions:**
- Claude Pro ($20/month)
- Claude Max (coming soon)

**Setup Steps:**

1. **Authenticate:**
   ```bash
   orkee login claude
   ```

2. **Browser Opens:** Log in with your Claude account
3. **Authorize:** Grant Orkee access to:
   - `model:claude` - Use Claude models
   - `account:read` - Read account information

4. **Verify:**
   ```bash
   orkee auth status
   ```

**Scopes:**
- `model:claude` - Required for API access
- `account:read` - Optional, for subscription type detection

### OpenAI

**Supported Subscriptions:**
- ChatGPT Plus ($20/month)
- ChatGPT Team ($30/user/month)
- ChatGPT Enterprise (custom pricing)

**Setup Steps:**

1. **Authenticate:**
   ```bash
   orkee login openai
   ```

2. **Browser Opens:** Log in with your OpenAI account
3. **Authorize:** Grant Orkee access to:
   - `model.read` - Read available models
   - `model.request` - Make API requests

4. **Verify:**
   ```bash
   orkee auth status
   ```

**Note:** OpenAI OAuth is currently in beta. API key fallback is recommended.

### Google (Vertex AI)

**Supported Accounts:**
- Google Cloud Platform accounts with Vertex AI enabled
- Google Workspace accounts with AI features

**Setup Steps:**

1. **Enable Vertex AI:**
   - Go to [Google Cloud Console](https://console.cloud.google.com)
   - Enable Vertex AI API
   - Note your project ID

2. **Authenticate:**
   ```bash
   orkee login google
   ```

3. **Browser Opens:** Log in with your Google account
4. **Authorize:** Grant Orkee access to:
   - `https://www.googleapis.com/auth/cloud-platform` - Access Vertex AI

5. **Configure Project:**
   ```bash
   orkee config set GOOGLE_PROJECT_ID your-project-id
   ```

### xAI (Grok)

**Supported Subscriptions:**
- Grok Premium (pricing TBA)

**Setup Steps:**

1. **Authenticate:**
   ```bash
   orkee login xai
   ```

2. **Browser Opens:** Log in with your X (Twitter) account
3. **Authorize:** Grant Orkee access to:
   - `models:read` - Read available models
   - `models:write` - Make API requests

**Note:** xAI OAuth is in development. Check back for updates.

## Configuration

### Environment Variables

Configure OAuth behavior via environment variables:

```bash
# OAuth Client Configuration
OAUTH_CLAUDE_CLIENT_ID=orkee-cli-oauth-client
OAUTH_CLAUDE_CLIENT_SECRET=                    # Optional, for confidential clients
OAUTH_CLAUDE_REDIRECT_URI=http://localhost:3737/callback
OAUTH_CLAUDE_SCOPES="model:claude account:read"

OAUTH_OPENAI_CLIENT_ID=orkee-cli-oauth-client
OAUTH_OPENAI_REDIRECT_URI=http://localhost:3737/callback
OAUTH_OPENAI_SCOPES="model.read model.request"

OAUTH_GOOGLE_CLIENT_ID=orkee-cli-oauth-client
OAUTH_GOOGLE_REDIRECT_URI=http://localhost:3737/callback
OAUTH_GOOGLE_SCOPES="https://www.googleapis.com/auth/cloud-platform"

OAUTH_XAI_CLIENT_ID=orkee-cli-oauth-client
OAUTH_XAI_REDIRECT_URI=http://localhost:3737/callback
OAUTH_XAI_SCOPES="models:read models:write"

# OAuth Server Settings
OAUTH_CALLBACK_PORT=3737                       # Local callback server port
OAUTH_STATE_TIMEOUT_SECS=600                   # 10 minutes (max time for user to complete OAuth)
OAUTH_TOKEN_REFRESH_BUFFER_SECS=300            # 5 minutes (refresh tokens this early)
```

### Authentication Preference

Control which authentication method to use:

```bash
# OAuth only (fail if no OAuth token available)
orkee config set auth_preference oauth

# API keys only (never use OAuth)
orkee config set auth_preference api_key

# Hybrid (try OAuth first, fall back to API keys) - DEFAULT
orkee config set auth_preference hybrid
```

### Database Configuration

OAuth tokens are stored in the same SQLite database as projects:

```bash
# Default location
~/.orkee/orkee.db

# Tables used
# - oauth_tokens: Encrypted access/refresh tokens
# - oauth_providers: Provider configurations
# - users: User-level auth_preference setting
```

## Security

### Token Storage

- **Encryption**: All tokens encrypted at rest using ChaCha20-Poly1305
- **Key Derivation**: Machine-based (HKDF-SHA256) or password-based (Argon2id)
- **Isolation**: Database file protected by OS permissions
- **Rotation**: Tokens refreshed automatically before expiry

### PKCE Flow

Orkee implements OAuth 2.0 PKCE (Proof Key for Code Exchange) for enhanced security:

1. **Code Verifier**: Random 64-character string (43-128 characters per RFC 7636)
2. **Code Challenge**: SHA256 hash of verifier, base64url-encoded
3. **Challenge Method**: S256 (required by most providers)

This prevents authorization code interception attacks.

### State Parameter

Each OAuth flow includes a cryptographically random state parameter:

- **Purpose**: CSRF protection
- **Length**: 21 characters (nanoid)
- **Validation**: State verified on callback
- **Timeout**: 10 minutes (configurable)

### Browser Security

- **Localhost Only**: Callback server binds to 127.0.0.1 (not 0.0.0.0)
- **Single Use**: Server accepts one connection and exits
- **Timeout**: Callback expires after state timeout
- **No Persistence**: No cookies or session data

## Troubleshooting

### Common Issues

#### 1. Browser Doesn't Open

**Symptom:** `orkee login` completes but browser doesn't open

**Solutions:**

```bash
# Check if browser is configured
echo $BROWSER

# Manually open the URL printed by Orkee
# Copy and paste the URL into your browser

# Or use a different browser
BROWSER=firefox orkee login claude
```

#### 2. Port Already in Use

**Symptom:** `Failed to bind to 127.0.0.1:3737: Address already in use`

**Solutions:**

```bash
# Find what's using the port
lsof -i :3737

# Kill the process or use a different port
OAUTH_CALLBACK_PORT=8737 orkee login claude
```

#### 3. Token Expired

**Symptom:** API calls fail with "Unauthorized" or "Token expired"

**Solutions:**

```bash
# Check token status
orkee auth status

# Refresh token manually
orkee auth refresh claude

# Or re-authenticate
orkee login claude --force
```

#### 4. Tokens Not Refreshing

**Symptom:** Tokens expire and don't auto-refresh

**Possible Causes:**
- No refresh token (some providers don't provide one)
- Refresh token expired (usually 30-90 days)
- Provider changed OAuth configuration

**Solutions:**

```bash
# Re-authenticate to get new tokens
orkee login claude

# Check provider OAuth settings
# Ensure offline access scope is included
```

#### 5. Database Locked

**Symptom:** `database is locked` error during OAuth operations

**Solutions:**

```bash
# Close other Orkee processes
pkill orkee

# Check for orphaned connections
lsof ~/.orkee/orkee.db

# Restart and try again
orkee login claude
```

#### 6. Certificate Verification Failed

**Symptom:** `SSL certificate verify failed` during token exchange

**Solutions:**

```bash
# Update CA certificates (Linux)
sudo apt-get update && sudo apt-get install ca-certificates

# Or (macOS)
brew install ca-certificates

# Temporary workaround (NOT RECOMMENDED for production)
REQUESTS_CA_BUNDLE="" orkee login claude
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# Set RUST_LOG for detailed logs
RUST_LOG=orkee_auth=debug orkee login claude

# Full debug output
RUST_LOG=debug orkee login claude
```

Expected log output:
```
[DEBUG] orkee_auth::oauth::pkce: Generated PKCE challenge
[DEBUG] orkee_auth::oauth::manager: Building authorization URL
[INFO] orkee_auth::oauth::manager: Opening browser for authentication
[DEBUG] orkee_auth::oauth::server: Starting OAuth callback server on port 3737
[INFO] orkee_auth::oauth::server: 📡 Waiting for OAuth callback
[DEBUG] orkee_auth::oauth::server: Received connection from 127.0.0.1:51234
[DEBUG] orkee_auth::oauth::manager: Exchanging authorization code for token
[DEBUG] orkee_auth::oauth::storage: Storing OAuth token for provider: claude
[INFO] orkee_auth::oauth::manager: ✅ Successfully authenticated with claude
```

### Rate Limiting

OAuth endpoints are rate-limited to prevent abuse:

```bash
# OAuth endpoints rate limit
RATE_LIMIT_OAUTH_RPM=10  # 10 requests per minute

# If you hit the limit
Error: Rate limit exceeded. Try again in X seconds.

# Wait and retry, or increase limit (development only)
RATE_LIMIT_OAUTH_RPM=20 orkee login claude
```

## Advanced Usage

### Manual Token Management

While Orkee handles tokens automatically, you can manage them manually:

```bash
# Get current token
orkee auth token claude

# Refresh token before expiry
orkee auth refresh claude

# Logout (delete token)
orkee logout claude

# Logout from all providers
orkee logout all

# Force re-authentication (even if token valid)
orkee login claude --force
```

### Custom OAuth Configuration

For enterprise deployments or custom OAuth setups:

```bash
# Configure custom OAuth endpoints
orkee config set OAUTH_CLAUDE_AUTH_URL https://custom-auth.example.com/oauth/authorize
orkee config set OAUTH_CLAUDE_TOKEN_URL https://custom-auth.example.com/oauth/token

# Use custom client credentials
orkee config set OAUTH_CLAUDE_CLIENT_ID your-custom-client-id
orkee config set OAUTH_CLAUDE_CLIENT_SECRET your-custom-client-secret

# Configure custom scopes
orkee config set OAUTH_CLAUDE_SCOPES "model:claude account:read custom:scope"
```

### Dashboard Integration

The dashboard provides a visual interface for OAuth management:

1. **Navigate to Settings:** Click the gear icon in the top-right
2. **Select OAuth Tab:** View all provider statuses
3. **Provider Cards:** Each provider shows:
   - Authentication status
   - Account email
   - Subscription type
   - Token expiry time (with warnings for < 1 hour)
4. **Logout Buttons:** Disconnect from providers via UI

**Automatic Token Refresh:**
- Dashboard checks token status every 5 minutes
- Tokens are refreshed automatically when within 5-minute expiry buffer
- Visual warnings when tokens are about to expire

### Using OAuth in Code

If you're building integrations with Orkee:

```rust
// Rust example
use orkee_auth::oauth::{OAuthManager, OAuthProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = OAuthManager::new_default().await?;

    // Authenticate
    let token = manager.authenticate(OAuthProvider::Claude, "user-id").await?;
    println!("Access token: {}", token.access_token);

    // Check if token needs refresh
    if token.needs_refresh() {
        let refreshed = manager.refresh_token("user-id", OAuthProvider::Claude).await?;
        println!("Refreshed token");
    }

    // Logout
    manager.logout("user-id", OAuthProvider::Claude).await?;

    Ok(())
}
```

```typescript
// TypeScript/Dashboard example
import { useAuth } from '@/contexts/AuthContext';

function MyComponent() {
  const { authStatus, getToken, logout } = useAuth();

  // Get OAuth token for Claude
  const token = await getToken('claude');

  // Use token with AI SDK
  const provider = createClaudeCode({ authToken: token });
  const { text } = await generateText({
    model: provider('claude-3-5-sonnet-20241022'),
    prompt: 'Hello, Claude!',
  });

  // Logout
  await logout('claude');
}
```

### Migration Strategies

#### Gradual Migration

Migrate from API keys to OAuth progressively:

```bash
# Week 1: Add OAuth for primary provider
orkee login claude
orkee config set auth_preference hybrid

# Week 2: Test and monitor
# Use dashboard to verify OAuth is working

# Week 3: Add other providers
orkee login openai
orkee login google

# Week 4: Switch to OAuth-only (optional)
orkee config set auth_preference oauth
orkee config delete ANTHROPIC_API_KEY
```

#### Rollback Plan

If OAuth causes issues, revert to API keys:

```bash
# Switch back to API keys
orkee config set auth_preference api_key

# Logout from OAuth
orkee logout all

# Verify API keys still work
orkee projects list
```

## Support

For additional help:

- **GitHub Issues**: https://github.com/OrkeeAI/orkee/issues
- **Documentation**: https://docs.orkee.ai
- **Community**: https://discord.gg/orkee

## Related Documentation

- [README.md](./README.md) - General Orkee documentation
- [oauth.md](./oauth.md) - OAuth implementation plan and technical details
- [CLAUDE.md](./CLAUDE.md) - AI assistant context for development
