# Installer Scripts Tests

Automated test suite for validating Tauri installer scripts across all platforms.

## Running Tests

```bash
cd packages/dashboard/src-tauri/tests
./run-installer-tests.sh
```

## What's Tested

The test suite validates:

### prepare-binaries.sh
- ✅ Script exists and is executable
- ✅ Uses strict mode (`set -euo pipefail`)
- ✅ Has ABOUTME documentation
- ✅ Has DEBUG environment variable support
- ✅ Checks for cargo availability
- ✅ Verifies Rust toolchain targets
- ✅ Has comprehensive error messages

### Linux Scripts (postinstall.sh, preuninstall.sh, install-cli-from-appimage.sh)
- ✅ Use strict mode
- ✅ Have ABOUTME documentation

### macOS Scripts (postinstall.sh, preuninstall.sh)
- ✅ Use strict mode
- ✅ Have ABOUTME documentation

### Windows NSIS (hooks.nsh)
- ✅ File exists
- ✅ Includes required libraries (StrFunc.nsh)
- ✅ Has NSIS_HOOK_POSTINSTALL macro
- ✅ Has NSIS_HOOK_PREUNINSTALL macro

## Test Results

Example output:
```
Installer Scripts Test Suite
==============================

Testing prepare-binaries.sh...
✓ prepare-binaries.sh: strict mode
✓ prepare-binaries.sh: documented
✓ prepare-binaries.sh: executable
✓ prepare-binaries.sh: debug support
✓ prepare-binaries.sh: cargo check
✓ prepare-binaries.sh: target verify
✓ prepare-binaries.sh: error messages

Testing Linux installer scripts...
✓ install-cli-from-appimage.sh: strict mode
✓ install-cli-from-appimage.sh: documented
✓ postinstall.sh: strict mode
✓ postinstall.sh: documented
✓ preuninstall.sh: strict mode
✓ preuninstall.sh: documented

Testing macOS installer scripts...
✓ postinstall.sh: strict mode
✓ postinstall.sh: documented
✓ preuninstall.sh: strict mode
✓ preuninstall.sh: documented

Testing Windows NSIS hooks...
✓ hooks.nsh: exists
✓ hooks.nsh: includes StrFunc
✓ hooks.nsh: postinstall
✓ hooks.nsh: preuninstall

========================================
Results: 21 passed, 0 failed
========================================
```

## Adding New Tests

To add tests for new installer scripts:

1. Add the script path to the appropriate section in `run-installer-tests.sh`
2. Use the `test_file` function for bash scripts
3. Use custom checks for non-bash files (like NSIS)

Example:
```bash
# Test new Linux script
for script in "$TEST_DIR/../linux"/*.sh; do
    test_file "$script" || true
done
```

## CI Integration

These tests are automatically run by shellcheck CI workflow (`.github/workflows/tauri-release.yml`).

## Limitations

This test suite validates:
- ✅ Script structure and best practices
- ✅ Required error handling patterns
- ✅ Documentation presence

It does NOT test:
- ❌ Actual PATH modification (requires system-level testing)
- ❌ Binary installation (requires full build)
- ❌ Installer UI/UX
- ❌ Cross-platform behavior

For full end-to-end testing, use the manual testing instructions in `../INSTALLER_README.md`.
