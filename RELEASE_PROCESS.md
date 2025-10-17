# Release Process

This document describes the release process for Orkee CLI and Desktop applications.

## Release Types

Orkee has two separate release tracks:

1. **CLI Release** (`v*` tags) - Rust CLI binary distributed via npm
2. **Desktop Release** (`desktop-v*` tags) - Tauri desktop application (macOS, Linux, Windows)

## Version Synchronization

Both releases should maintain the same version number, but are tagged separately:
- CLI: `v0.0.4`
- Desktop: `desktop-v0.0.4`

## Release Checklist

### 1. Update Version Numbers

Update version in the following files:

```bash
# Workspace version (inherited by all Rust packages)
Cargo.toml                                    # [workspace.package] version

# Desktop-specific versions
packages/dashboard/src-tauri/Cargo.toml       # [package] version
packages/dashboard/src-tauri/tauri.conf.json  # version field
```

### 2. Update Lock Files

```bash
# Update Cargo.lock with new versions
cargo update --workspace
```

### 3. Build and Test Locally

```bash
# Build CLI binary
cargo build --release --bin orkee

# Test CLI
cargo test

# Build Tauri desktop app locally (optional verification)
cd packages/dashboard
./prepare-binaries.sh
bun run tauri:build
```

### 4. Commit Changes

```bash
git add Cargo.toml packages/dashboard/src-tauri/Cargo.toml packages/dashboard/src-tauri/tauri.conf.json Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git push origin main
```

### 5. Create CLI Release

```bash
# Create and push CLI tag
git tag vX.Y.Z
git push origin vX.Y.Z
```

This triggers:
- **Workflow**: `Build Multi-Platform Binaries` (`.github/workflows/build-binaries.yml`)
- **Builds**: CLI binaries for macOS (x86_64, aarch64), Linux (x86_64), Windows (x86_64)
- **Publishes**: npm package with platform-specific binaries
- **Creates**: GitHub release with binary assets

### 6. Create Desktop Release

```bash
# Create and push desktop tag
git tag desktop-vX.Y.Z
git push origin desktop-vX.Y.Z
```

This triggers:
- **Workflow**: `Build and Release Tauri Desktop App` (`.github/workflows/tauri-release.yml`)
- **Builds**: Desktop applications for macOS (Intel + Apple Silicon), Linux, Windows
- **Creates**: GitHub release with installers (.dmg, .AppImage, .msi)

## Release Artifacts

### CLI Release (`v*`)
- **npm Package**: `orkee@X.Y.Z` with platform-specific binaries
- **GitHub Assets**:
  - `orkee-x86_64-apple-darwin.tar.gz`
  - `orkee-aarch64-apple-darwin.tar.gz`
  - `orkee-x86_64-unknown-linux-gnu.tar.gz`
  - `orkee-x86_64-pc-windows-msvc.zip`

### Desktop Release (`desktop-v*`)
- **macOS**:
  - `Orkee_X.Y.Z_aarch64.dmg` (Apple Silicon)
  - `Orkee_X.Y.Z_x64.dmg` (Intel)
- **Linux**:
  - `orkee_X.Y.Z_amd64.AppImage`
  - `orkee_X.Y.Z_amd64.deb`
- **Windows**:
  - `Orkee_X.Y.Z_x64-setup.exe`
  - `Orkee_X.Y.Z_x64_en-US.msi`

## Example: Release 0.0.4

```bash
# 1. Update versions
# Edit Cargo.toml, packages/dashboard/src-tauri/Cargo.toml, packages/dashboard/src-tauri/tauri.conf.json

# 2. Update lock files
cargo update --workspace

# 3. Test
cargo test
bun lint

# 4. Commit
git add Cargo.toml packages/dashboard/src-tauri/Cargo.toml packages/dashboard/src-tauri/tauri.conf.json Cargo.lock
git commit -m "chore: bump version to 0.0.4"
git push origin main

# 5. CLI release
git tag v0.0.4
git push origin v0.0.4

# 6. Desktop release
git tag desktop-v0.0.4
git push origin desktop-v0.0.4
```

## Hotfix Releases

For urgent fixes that need to be released immediately:

1. Create a hotfix branch from the release tag: `git checkout -b hotfix/X.Y.Z vX.Y.Z`
2. Make the fix and bump patch version
3. Follow normal release process
4. Merge hotfix branch back to main

## Rollback

To rollback a release:

1. Delete the GitHub release (preserves git tag)
2. Delete the git tag: `git tag -d vX.Y.Z && git push origin :vX.Y.Z`
3. For npm: `npm unpublish orkee@X.Y.Z` (only works within 72 hours)

## Manual Intervention

Both workflows support manual triggering via `workflow_dispatch` in the GitHub Actions UI if automatic tag-based releases fail.

## Troubleshooting

### CLI build fails
- Check Rust version compatibility
- Verify all platforms compile locally
- Review GitHub Actions logs for platform-specific errors

### Desktop build fails
- Ensure `prepare-binaries.sh` script is executable
- Verify Tauri configuration is valid
- Check platform-specific dependencies (see workflow for details)

### Version mismatch
- Ensure all version fields are updated (4 files total)
- Run `cargo update --workspace` after version changes
- Verify `Cargo.lock` is committed
