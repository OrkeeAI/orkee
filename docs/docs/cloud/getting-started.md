---
sidebar_position: 1
title: Getting Started with Cloud Sync
---

# Getting Started with Cloud Sync

Orkee Cloud provides secure backup, synchronization, and collaboration features for your AI agent projects. This guide covers everything you need to know to set up and use cloud sync.

## What is Orkee Cloud?

Orkee Cloud extends Orkee's local-first architecture with optional cloud capabilities:

- **Automatic Backup**: Your projects are safely backed up to the cloud
- **Multi-Device Sync**: Access your projects from any machine
- **Team Collaboration**: Share projects with team members (coming soon)
- **Version History**: Restore previous versions of your projects
- **Enterprise Security**: OAuth authentication with JWT validation and Row Level Security

Orkee maintains a **SQLite-first architecture** where all functionality works offline. Cloud sync enhances this with backup and synchronization without compromising local performance.

## Prerequisites

Before enabling cloud sync, ensure you have:

1. **Orkee CLI installed** - Install via npm or build from source
2. **Cloud-enabled build** - Compile with `--features cloud` flag
3. **Orkee Cloud account** - Sign up at [orkee.ai](https://orkee.ai)
4. **Internet connection** - Required for authentication and sync operations

### Building with Cloud Support

Cloud features are optional. To enable them, build Orkee with the `cloud` feature flag:

```bash
# Build CLI with cloud support
cd packages/cli
cargo build --features cloud --release

# Or during development
cargo run --features cloud --bin orkee -- --help
```

If you installed via npm, cloud features may already be included depending on your distribution.

## Setting Up Cloud Sync

### Step 1: Authenticate with Orkee Cloud

First, authenticate with your Orkee Cloud account:

```bash
orkee cloud login
```

This command will:
1. Open your default browser to the Orkee Cloud authentication page
2. Prompt you to sign in or create an account
3. Request authorization for the CLI to access your cloud projects
4. Store authentication tokens securely in `~/.orkee/`

**Example output:**
```
Opening browser for authentication...
Waiting for authorization...
✓ Successfully authenticated with Orkee Cloud
✓ Token saved to ~/.orkee/cloud-token
```

### Step 2: Check Authentication Status

Verify your authentication and check sync status:

```bash
orkee cloud status
```

**Example output:**
```
Cloud Sync Status
─────────────────────────────────────
Authentication: ✓ Authenticated
User: joe@example.com
Subscription: Pro Plan
Last Sync: 2 minutes ago
Sync Enabled: Yes
Projects Synced: 5 of 5

Connection: https://api.orkee.ai
Storage: 234 MB used / 10 GB available
```

### Step 3: Enable Cloud Sync

Enable automatic cloud synchronization:

```bash
orkee cloud enable
```

This activates automatic background sync for your projects. Changes will be synced to the cloud periodically.

**To disable cloud sync later:**
```bash
orkee cloud disable
```

Disabling cloud sync puts Orkee back into local-only mode. Your local data remains intact, and you can re-enable sync at any time.

## Using Cloud Sync

### Automatic Synchronization

Once enabled, Orkee automatically syncs your projects in the background when:
- You create, update, or delete a project
- The CLI server is running
- You're connected to the internet

No manual intervention is required for day-to-day operations.

### Manual Sync Operations

#### Sync All Projects

Force an immediate sync of all projects:

```bash
orkee cloud sync
```

**Example output:**
```
Syncing projects to Orkee Cloud...
✓ project-alpha (id: 1) - synced
✓ project-beta (id: 2) - synced
✓ project-gamma (id: 3) - synced

Successfully synced 3 projects
Last sync: just now
```

#### Sync a Specific Project

Sync only one project by ID:

```bash
orkee cloud sync --project 1
```

This is useful when you've made significant changes to a single project and want to ensure it's backed up immediately.

### Viewing Cloud Projects

List all projects stored in Orkee Cloud:

```bash
orkee cloud list
```

**Example output:**
```
Cloud Projects
─────────────────────────────────────
ID   Name              Last Synced        Size
1    project-alpha     2 minutes ago      1.2 MB
2    project-beta      1 hour ago         456 KB
3    project-gamma     5 minutes ago      2.1 MB
4    project-delta     1 day ago          890 KB
5    project-epsilon   3 hours ago        1.5 MB

Total: 5 projects (6.1 MB)
```

**Limit results:**
```bash
orkee cloud list --limit 10
```

### Restoring from Cloud Backups

#### Restore All Projects

Restore all projects from the cloud (useful when setting up a new machine):

```bash
orkee cloud restore
```

**Warning:** This will overwrite local projects with cloud versions. Orkee will prompt for confirmation before proceeding.

#### Restore a Specific Project

Restore a single project by ID:

```bash
orkee cloud restore --project 1
```

**Example output:**
```
Restoring project from Orkee Cloud...
Project: project-alpha (id: 1)
Last synced: 2 hours ago

✓ Downloaded project data
✓ Restored to local database
✓ Project files validated

Successfully restored project-alpha
```

## Common Use Cases

### Setting Up a New Development Machine

When setting up Orkee on a new machine:

1. Install Orkee CLI with cloud support
2. Authenticate with your account:
   ```bash
   orkee cloud login
   ```
3. Restore all projects:
   ```bash
   orkee cloud restore
   ```
4. Enable automatic sync:
   ```bash
   orkee cloud enable
   ```

### Backing Up Before Major Changes

Before making significant changes to your projects:

```bash
# Force immediate backup
orkee cloud sync

# Verify backup succeeded
orkee cloud status
```

### Working Offline

Orkee's local-first architecture means you can work completely offline:

1. Make changes to projects locally
2. Changes are saved to local SQLite database immediately
3. When back online, run `orkee cloud sync` to push changes
4. Or wait for automatic sync if cloud sync is enabled

### Recovering from Local Data Loss

If your local database is corrupted or lost:

```bash
# Remove corrupted database (if needed)
rm ~/.orkee/orkee.db

# Restore from cloud
orkee cloud restore

# Verify restoration
orkee projects list
```

## Troubleshooting

### Authentication Issues

**Problem:** `orkee cloud login` fails or browser doesn't open

**Solutions:**
1. Check internet connection
2. Verify firewall isn't blocking browser communication
3. Try manual authentication at [orkee.ai/login](https://orkee.ai/login)
4. Check that your browser isn't blocking pop-ups
5. Ensure `~/.orkee/` directory has correct permissions

**Problem:** "Token expired" error

**Solution:** Re-authenticate:
```bash
orkee cloud logout
orkee cloud login
```

### Sync Failures

**Problem:** Sync fails with connection error

**Solutions:**
1. Check internet connectivity
2. Verify Orkee Cloud API status at [status.orkee.ai](https://status.orkee.ai)
3. Check authentication status: `orkee cloud status`
4. Re-authenticate if token expired: `orkee cloud login`

**Problem:** Project won't sync or shows conflict

**Solutions:**
1. Check project details: `orkee projects show <id>`
2. Try manual sync: `orkee cloud sync --project <id>`
3. If conflicts persist, restore from cloud: `orkee cloud restore --project <id>`

### Storage Issues

**Problem:** Sync fails with "storage quota exceeded" error

**Solutions:**
1. Check storage usage: `orkee cloud status`
2. Delete unused projects: `orkee projects delete <id>`
3. Upgrade your Orkee Cloud plan at [orkee.ai/pricing](https://orkee.ai/pricing)

### Configuration Issues

**Problem:** Cloud commands not available

**Solution:** Ensure Orkee was built with cloud support:
```bash
# Rebuild with cloud features
cargo build --features cloud --release

# Verify cloud commands are available
orkee cloud --help
```

### Log Files

For detailed troubleshooting, check Orkee log files:

```bash
# View recent logs
tail -f ~/.orkee/logs/orkee.log

# Search for cloud-related errors
grep -i "cloud\|sync" ~/.orkee/logs/orkee.log
```

## Security and Authentication

### OAuth Authentication

Orkee Cloud uses industry-standard OAuth 2.0 for authentication:

- **Secure token storage**: Authentication tokens are stored securely in `~/.orkee/cloud-token`
- **Automatic token refresh**: Tokens are refreshed automatically before expiration
- **Revocable access**: Revoke access at any time from your Orkee Cloud account settings

### Data Security

Your project data is protected with multiple security layers:

- **Transport encryption**: All data transmitted via TLS/HTTPS
- **JWT validation**: API requests validated with cryptographic signatures
- **Row Level Security**: Database-level access controls
- **Secure storage**: Cloud storage with enterprise-grade security

### Token Management

**View current authentication:**
```bash
orkee cloud status
```

**Sign out and remove tokens:**
```bash
orkee cloud logout
```

**Revoke access from web:**
1. Visit [orkee.ai/account/security](https://orkee.ai/account/security)
2. Navigate to "Connected Applications"
3. Click "Revoke" next to Orkee CLI

### Best Practices

1. **Never share tokens**: Don't commit `~/.orkee/cloud-token` to version control
2. **Rotate regularly**: Log out and re-authenticate periodically
3. **Monitor access**: Review connected applications in your account settings
4. **Use secure networks**: Avoid syncing on untrusted public Wi-Fi
5. **Enable 2FA**: Enable two-factor authentication on your Orkee Cloud account

## Environment Variables

Cloud sync can be configured with environment variables:

```bash
# Set Orkee Cloud API URL (usually not needed)
export ORKEE_CLOUD_API_URL=https://api.orkee.ai

# Set authentication token manually (for CI/CD)
export ORKEE_CLOUD_TOKEN=your-token-here
```

**For CI/CD pipelines:**
```bash
# Authenticate non-interactively
ORKEE_CLOUD_TOKEN=your-token orkee cloud sync
```

## Next Steps

Now that you have cloud sync configured:

- **Learn more about projects**: See [Project Management Guide](../user-guide/projects.md)
- **Explore collaboration**: Visit [Team Collaboration Guide](./collaboration.md) (coming soon)
- **Optimize sync settings**: Review [Cloud Configuration](../configuration/cloud.md) (coming soon)
- **Set up automation**: Configure [Automated Backups](./automation.md) (coming soon)

## Getting Help

If you encounter issues not covered in this guide:

- **Documentation**: [orkee.ai/docs](https://orkee.ai/docs)
- **Community**: [GitHub Discussions](https://github.com/OrkeeAI/orkee/discussions)
- **Issues**: [GitHub Issues](https://github.com/OrkeeAI/orkee/issues)
- **Support**: support@orkee.ai

---

**Note**: Orkee Cloud features are fully implemented in the open-source client and require an Orkee Cloud account. Visit [orkee.ai](https://orkee.ai) for API access and subscription plans.
