# Orkee Cloud Testing Plan

## Overview

This document provides a comprehensive testing plan for the Orkee Cloud functionality. The testing is divided into phases, starting with CLI functionality and progressing to web features.

**Current Status**: Cloud implementation is ~60% complete. Core functionality is implemented but needs end-to-end testing with live Supabase.

## Prerequisites

### 1. Supabase Project Setup

Before testing, ensure your Supabase project has:
- [ ] Database schema applied (from `packages/cloud/migrations/001_supabase_schema.sql`)
- [ ] OAuth providers configured (GitHub and/or Google)
- [ ] Storage bucket `project-snapshots` created
- [ ] RLS policies enabled
- [ ] Environment variables set:
  ```bash
  export NEXT_PUBLIC_SUPABASE_URL=https://xxxxx.supabase.co
  export NEXT_PUBLIC_SUPABASE_ANON_KEY=xxxxx
  ```

### 2. Build Orkee with Cloud Features

```bash
cd packages/cli
cargo build --release --features cloud

# Verify cloud features are included
./target/release/orkee cloud --help
```

## Phase 1: CLI Cloud Testing

### Test 1.1: OAuth Authentication Flow

#### Steps:
1. Start fresh (remove any existing tokens):
   ```bash
   rm -f ~/.orkee/.cloud-token
   ```

2. Initiate authentication:
   ```bash
   ./target/release/orkee cloud enable
   ```

3. Browser should open to Supabase OAuth page
4. Login with GitHub or Google
5. After successful auth, browser redirects to `localhost:8899`
6. CLI captures token and displays success message

#### Expected Results:
- ✅ Browser opens automatically
- ✅ OAuth provider login page appears
- ✅ Successful redirect to localhost:8899
- ✅ CLI shows: "✅ Authenticated as: your-email@example.com"
- ✅ Token file created at `~/.orkee/.cloud-token`

#### Verification:
```bash
# Check token file exists
ls -la ~/.orkee/.cloud-token

# Verify token content (should be JSON with JWT)
cat ~/.orkee/.cloud-token | jq .

# Token should contain:
# - access_token (JWT string)
# - refresh_token (if provided)
# - expires_at (timestamp)
# - user_id
# - email
```

#### Error Cases to Test:
- [ ] Cancel OAuth login - should handle gracefully
- [ ] Close browser without completing - should timeout with clear message
- [ ] Port 8899 already in use - should detect and use alternative

### Test 1.2: Cloud Status Command

#### Steps:
```bash
./target/release/orkee cloud status
```

#### Expected Output:
```
Orkee Cloud Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Status:        Enabled
User:          your-email@example.com
Subscription:  Free
Projects:      0/2 (0% used)
Storage:       0MB/100MB (0% used)
Last Sync:     Never
Auto-Sync:     Disabled (upgrade for auto-sync)
```

#### Error Cases:
- [ ] Expired token - should prompt to re-authenticate
- [ ] No token - should show "Cloud not enabled" message
- [ ] Network error - should show connection error

### Test 1.3: Project Creation and Initial Sync

#### Setup:
```bash
# Create test projects
./target/release/orkee projects add \
  --name "Test Project 1" \
  --path "/tmp/test-project-1" \
  --description "First test project"

./target/release/orkee projects add \
  --name "Test Project 2" \
  --path "/tmp/test-project-2" \
  --description "Second test project"
```

#### Sync Test:
```bash
# Sync all projects
./target/release/orkee cloud sync

# Or sync specific project
./target/release/orkee cloud sync --project "Test Project 1"
```

#### Expected Results:
- ✅ Progress indicator shows during sync
- ✅ Success message: "✅ Synced 2 projects to cloud"
- ✅ Each project shows upload size and time

#### Verification in Supabase Dashboard:
1. Check `projects` table:
   - Should have 2 records
   - user_id matches authenticated user
   - sync_status = 'synced'
   - cloud_version = 1

2. Check `project_snapshots` table:
   - Should have 2 snapshot records
   - version = 1 for each
   - size_bytes populated
   - checksum present

3. Check Storage bucket `project-snapshots`:
   - Files at path: `{user_id}/{project_id}/snapshot_{timestamp}.gz`
   - Files are compressed (gzip)
   - Files are encrypted (if encryption enabled)

### Test 1.4: List Cloud Snapshots

#### Steps:
```bash
# List all snapshots
./target/release/orkee cloud list

# List with limit
./target/release/orkee cloud list --limit 5

# List for specific project
./target/release/orkee cloud list --project "Test Project 1"
```

#### Expected Output:
```
Cloud Snapshots
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Project: Test Project 1
  Version 1 - 2025-09-09 10:30:45 - 256KB
  
Project: Test Project 2
  Version 1 - 2025-09-09 10:30:46 - 128KB

Total: 2 snapshots (384KB)
```

### Test 1.5: Restore from Cloud

#### Setup:
```bash
# First, note the project ID
./target/release/orkee projects list

# Delete local project
./target/release/orkee projects delete <project-id> --yes

# Verify it's gone
./target/release/orkee projects list
```

#### Restore Test:
```bash
# Restore all projects from cloud
./target/release/orkee cloud restore

# Or restore specific project
./target/release/orkee cloud restore --project "Test Project 1"
```

#### Expected Results:
- ✅ Progress indicator during download
- ✅ Success message: "✅ Restored 1 project from cloud"
- ✅ Project appears in local list
- ✅ All project data matches original (name, path, description)

### Test 1.6: Free Tier Limits

#### Test Project Limit:
```bash
# Already have 2 projects, try to add a 3rd
./target/release/orkee projects add \
  --name "Test Project 3" \
  --path "/tmp/test-project-3"

# Try to sync
./target/release/orkee cloud sync
```

#### Expected Result:
```
❌ Project limit exceeded
You have reached the free tier limit of 2 projects.
Upgrade to Starter ($9/mo) for 10 projects or Pro ($29/mo) for unlimited.

Visit https://app.orkee.ai/billing to upgrade.
```

#### Test Storage Limit:
```bash
# Create a large file in project directory
dd if=/dev/urandom of=/tmp/test-project-1/large-file.bin bs=1M count=50

# Try to sync (should approach 100MB limit)
./target/release/orkee cloud sync
```

#### Expected Warning at 80MB:
```
⚠️ Storage Warning
You're using 85MB of your 100MB free tier storage.
Upgrade to Starter for 5GB or Pro for 50GB storage.
```

#### Expected Error at 100MB:
```
❌ Storage limit exceeded
You have exceeded the free tier limit of 100MB.
Upgrade to Starter ($9/mo) for 5GB storage.
```

### Test 1.7: Sync Conflict Detection

#### Setup:
```bash
# Sync project from device A
./target/release/orkee cloud sync

# On device B, login and restore
./target/release/orkee cloud enable
./target/release/orkee cloud restore

# Make changes on device A
./target/release/orkee projects edit <id> --description "Changed on device A"
./target/release/orkee cloud sync

# Make different changes on device B
./target/release/orkee projects edit <id> --description "Changed on device B"
./target/release/orkee cloud sync
```

#### Expected Result:
```
⚠️ Sync Conflict Detected
Project "Test Project 1" has been modified on another device.

Local version:  Modified 2 minutes ago
Cloud version:  Modified 5 minutes ago

Choose resolution:
1. Keep local version (overwrite cloud)
2. Keep cloud version (overwrite local)
3. View differences
4. Cancel

Choice: 
```

### Test 1.8: Error Handling

#### Network Disconnection:
```bash
# Disconnect network
# Try to sync
./target/release/orkee cloud sync
```

Expected: "❌ Network error: Unable to connect to Orkee Cloud"

#### Invalid Token:
```bash
# Corrupt token file
echo "invalid-json" > ~/.orkee/.cloud-token
./target/release/orkee cloud status
```

Expected: "❌ Authentication error. Please run 'orkee cloud enable' to re-authenticate"

#### Interrupt During Sync:
```bash
# Start sync of large project
./target/release/orkee cloud sync

# Press Ctrl+C during upload
```

Expected: Clean cancellation with message "Sync cancelled by user"

### Test 1.9: Performance Testing

#### Large Project Sync:
```bash
# Create project with many files
mkdir -p /tmp/large-project
for i in {1..100}; do
  echo "File content $i" > /tmp/large-project/file-$i.txt
done

./target/release/orkee projects add --name "Large Project" --path "/tmp/large-project"
time ./target/release/orkee cloud sync
```

#### Metrics to Track:
- [ ] Sync time for 10MB project: _______ seconds
- [ ] Sync time for 50MB project: _______ seconds
- [ ] Memory usage during sync: _______ MB
- [ ] CPU usage during sync: _______ %

### Test 1.10: Logout and Re-authentication

#### Steps:
```bash
# Logout
./target/release/orkee cloud logout

# Verify logged out
./target/release/orkee cloud status

# Re-authenticate
./target/release/orkee cloud enable

# Verify projects still accessible
./target/release/orkee cloud list
```

## Phase 2: Dashboard Testing

### Test 2.1: Login Flow
1. Navigate to app.orkee.ai
2. Click "Sign in with GitHub"
3. Complete OAuth flow
4. Verify redirect to dashboard

### Test 2.2: Project Display
- [ ] All synced projects visible
- [ ] Project metadata correct
- [ ] Last sync time displayed
- [ ] Storage usage shown

### Test 2.3: Subscription Management
- [ ] Current tier displayed
- [ ] Usage metrics accurate
- [ ] Upgrade CTA visible for free tier
- [ ] Billing page accessible

## Phase 3: Integration Testing

### Test 3.1: CLI + Dashboard Sync
1. Add project via CLI
2. Sync to cloud
3. Verify appears in dashboard immediately
4. Edit project in dashboard
5. Restore in CLI
6. Verify changes reflected

### Test 3.2: Multi-Device Sync
1. Install Orkee on 2 devices
2. Login with same account
3. Create project on device A
4. Sync and verify on device B
5. Edit on device B
6. Sync and verify on device A

## Phase 4: Billing Testing (When Implemented)

### Test 4.1: Upgrade Flow
1. Click upgrade from free tier
2. Complete Stripe checkout
3. Verify subscription updated
4. Verify new limits applied

### Test 4.2: Downgrade Flow
1. Cancel subscription
2. Verify downgrade at period end
3. Verify limits enforced after downgrade

## Regression Testing Checklist

After any code changes, verify:

### Core Functionality
- [ ] Local-only mode still works without cloud
- [ ] All CRUD operations work locally
- [ ] TUI functions without cloud features
- [ ] Dashboard works with local data

### Cloud Features
- [ ] Authentication flow works
- [ ] Sync uploads successfully
- [ ] Restore downloads successfully
- [ ] Limits enforced correctly
- [ ] Conflicts detected

### Performance
- [ ] No memory leaks during sync
- [ ] Reasonable sync times
- [ ] UI remains responsive

## Bug Reporting Template

When reporting issues, please include:

```markdown
### Issue Description
Brief description of the problem

### Steps to Reproduce
1. Step one
2. Step two
3. Step three

### Expected Behavior
What should happen

### Actual Behavior
What actually happened

### Environment
- OS: [e.g., macOS 14.5]
- Orkee Version: [from `orkee --version`]
- Rust Version: [from `rustc --version`]
- Node Version: [from `node --version`]

### Logs
```
Paste any error messages or logs here
```

### Screenshots
If applicable, add screenshots

### Additional Context
Any other relevant information
```

## Success Criteria

The cloud feature is considered ready for beta when:

1. **Authentication**: ✅ All OAuth flows work reliably
2. **Sync**: ✅ Projects sync up and down without data loss
3. **Limits**: ✅ Free tier limits enforced correctly
4. **Errors**: ✅ All error cases handled gracefully
5. **Performance**: ✅ Sync completes in reasonable time
6. **Security**: ✅ Tokens stored securely, data encrypted
7. **UX**: ✅ Clear messages and progress indicators

## Notes

- Test with both GitHub and Google OAuth if possible
- Test on different operating systems (macOS, Linux, Windows)
- Test with various network conditions (fast, slow, intermittent)
- Keep Supabase dashboard open to monitor data changes
- Document any unexpected behaviors or improvements needed

---

*Last Updated: 2025-09-09*