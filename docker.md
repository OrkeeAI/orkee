# Docker Authentication

## Overview

**Status**: ✅ COMPLETE

**Implementation**: Simple wrapper around `docker login` CLI command

**Why**: Docker CLI handles credential storage in system keychain automatically, eliminating need for custom credential management.

---

## What Was Implemented

Added `orkee auth login docker` command that wraps `docker login`:

```bash
orkee auth login docker
```

### How It Works

1. User runs `orkee auth login docker`
2. Command executes `docker login` as a subprocess
3. Docker handles authentication (browser-based device code flow or username/password)
4. Docker stores credentials in system keychain (`~/.docker/config.json` or native keychain)
5. Future `docker build`, `docker push`, etc. automatically use stored credentials

### Implementation Details

- **File**: `packages/cli/src/bin/cli/auth.rs`
- **Function**: `import_docker_credentials()`
- **Approach**: Execute `docker login` via `Command::new("docker").arg("login")`
- **Credential Storage**: Handled by Docker (not Orkee database)

### Usage

```bash
# Authenticate with Docker Hub
orkee auth login docker

# Then build and push images normally
docker build -t username/image:tag .
docker push username/image:tag
```

### Why This Approach?

- **Simpler**: No database schema changes needed
- **Secure**: Leverages Docker's existing credential management (system keychain)
- **Standard**: Uses Docker's native authentication flow
- **Maintainable**: No custom credential encryption/decryption logic

---

## Implementation Status

### ✅ Completed
- [x] `orkee auth login docker` - Wrapper around `docker login` CLI

### 🚧 TODO: Sandbox Image Management

We still need CLI commands to build and push sandbox images:

#### Required Commands
```bash
# Build a sandbox image
orkee sandbox build [--name <name>] [--tag <tag>] [--dockerfile <path>]

# Push image to Docker Hub
orkee sandbox push <image:tag>

# List available sandbox images
orkee sandbox images

# Set default sandbox image
orkee sandbox config set-image <image:tag>
```

#### Implementation Plan

**1. Add `sandbox build` command** (`packages/cli/src/bin/cli/sandbox.rs`)
- Wrapper around `docker build`
- Optionally use Dockerfile from project or default template
- Tag image with convention: `username/orkee-sandbox:tag`

**2. Add `sandbox push` command** (`packages/cli/src/bin/cli/sandbox.rs`)
- Wrapper around `docker push`
- Verify authentication before pushing (check if logged in)
- Use stored Docker credentials automatically

**3. Add `sandbox images` command** (`packages/cli/src/bin/cli/sandbox.rs`)
- Wrapper around `docker images --filter label=orkee.sandbox=true`
- List all sandbox images built by Orkee

**4. Update `sandbox config` command**
- Already planned in original Phase 3
- Add ability to set default sandbox image for new sandboxes

#### Usage Flow
```bash
# 1. Login to Docker Hub (one-time)
orkee auth login docker

# 2. Build your sandbox image
orkee sandbox build --name my-sandbox --tag v1.0

# 3. Push to Docker Hub
orkee sandbox push username/orkee-sandbox:v1.0

# 4. Set as default
orkee sandbox config set-image username/orkee-sandbox:v1.0

# 5. Create sandboxes using your image
# (Now automatically uses your default image)
```

---

## Troubleshooting

### Authentication Issues
If you encounter authentication errors:

1. Run `orkee auth login docker`
2. Follow the Docker login prompts
3. Verify credentials are stored: `docker info` should show your username

### Rate Limiting
Docker Hub imposes rate limits on image pulls:
- **Unauthenticated**: 100 pulls per 6 hours
- **Authenticated**: 200 pulls per 6 hours
- **Solution**: Run `orkee auth login docker` to increase your limit
