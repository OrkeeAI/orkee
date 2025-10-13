# Tauri Installer Scripts

This directory contains platform-specific installer hooks that automatically add the `orkee` CLI binary to the system PATH when users install the Orkee Desktop app.

## Overview

When users install the desktop app, they get:
- ✅ Desktop GUI application
- ✅ CLI commands (`orkee projects list`, etc.)
- ✅ TUI interface (`orkee tui`)

The installer scripts handle copying/linking the bundled CLI binary to system PATH automatically.

## Files

### Windows (`windows/hooks.nsh`)
- **Type**: NSIS installer hooks
- **Location**: `%LOCALAPPDATA%\Orkee\bin\orkee.exe` (per-user) or `Program Files\Orkee\bin\orkee.exe` (per-machine)
- **PATH Setup**: Uses `setx` command to add to PATH
- **Hooks**:
  - `NSIS_HOOK_POSTINSTALL`: Copies binary and adds to PATH
  - `NSIS_HOOK_PREUNINSTALL`: Removes binary

### macOS (`macos/postinstall.sh`, `macos/preuninstall.sh`)
- **Type**: Bash scripts (NOT currently functional - see limitation below)
- **Location**: `/usr/local/bin/orkee`
- **PATH Setup**: `/usr/local/bin` is already in PATH on macOS
- **Permissions**: Requires admin privileges for `/usr/local/bin` access
- **Scripts**:
  - `postinstall.sh`: Copies binary from app bundle to `/usr/local/bin`
  - `preuninstall.sh`: Removes binary from `/usr/local/bin`
- **⚠️ Known Limitation**: Tauri creates DMG installers for macOS, which don't support post-install scripts. Scripts are included but won't execute automatically. See manual setup below.

### Linux (`linux/postinstall.sh`, `linux/preuninstall.sh`)
- **Type**: Bash scripts executed by package manager (.deb/.rpm only)
- **Location**: `/usr/local/bin/orkee` (symlink or copy)
- **PATH Setup**: `/usr/local/bin` is typically in PATH
- **Fallback**: If symlink fails, copies binary instead
- **Scripts**:
  - `postinstall.sh`: Creates symlink or copies binary
  - `preuninstall.sh`: Removes symlink/binary
- **Note**: AppImage format does not support post-install hooks - see manual setup below

## Testing

To test installer scripts locally:

### macOS
```bash
# Build the app
cd packages/dashboard
bash prepare-binaries.sh
bun run tauri:build

# Install the .dmg (drag app to Applications folder)
open src-tauri/target/release/bundle/dmg/Orkee_*.dmg

# ⚠️ IMPORTANT: CLI access requires manual setup on macOS
# DMG installers don't support automatic CLI installation
# See "macOS Manual CLI Setup" section below
```

### Windows
```bash
# Build the app
cd packages/dashboard
bash prepare-binaries.sh
bun run tauri:build

# Install the .msi (run as admin)
# Then in a NEW terminal (to pick up PATH changes):
orkee --version
orkee projects list
orkee tui
```

### Linux (.deb/.rpm)
```bash
# Build the app
cd packages/dashboard
bash prepare-binaries.sh
bun run tauri:build

# Install the .deb/.rpm
sudo dpkg -i src-tauri/target/release/bundle/deb/orkee_*.deb
# or
sudo rpm -i src-tauri/target/release/bundle/rpm/orkee-*.rpm

# Verify CLI access
orkee --version
orkee projects list
orkee tui
```

### Linux (AppImage)
AppImages don't support post-install hooks, so CLI access requires setup.

#### Option 1: Automated Setup (Recommended)
Download and run the helper script:

```bash
# Download the AppImage
chmod +x Orkee*.AppImage

# Download and run the helper script
curl -fsSL https://raw.githubusercontent.com/OrkeeAI/orkee/main/packages/dashboard/src-tauri/linux/install-cli-from-appimage.sh | bash

# Or if you have the repository cloned:
bash packages/dashboard/src-tauri/linux/install-cli-from-appimage.sh

# Follow the interactive prompts to choose installation location
```

The script will:
- Extract the orkee binary from the AppImage
- Install it to `/usr/local/bin` or `~/.local/bin` (your choice)
- Verify the installation
- Alert you if PATH updates are needed

#### Option 2: Manual Setup
```bash
# Download and make executable
chmod +x Orkee*.AppImage

# Extract the bundled orkee binary
./Orkee*.AppImage --appimage-extract
# This creates: squashfs-root/usr/bin/orkee

# Copy to system PATH (choose one method):

# Method A: Copy to /usr/local/bin (recommended, requires sudo)
sudo cp squashfs-root/usr/bin/orkee /usr/local/bin/orkee
sudo chmod +x /usr/local/bin/orkee

# Method B: Copy to ~/.local/bin (no sudo needed)
mkdir -p ~/.local/bin
cp squashfs-root/usr/bin/orkee ~/.local/bin/orkee
chmod +x ~/.local/bin/orkee
# Add to PATH if needed:
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify CLI access
orkee --version
orkee projects list
orkee tui

# Clean up extracted files (optional)
rm -rf squashfs-root
```

### macOS Manual CLI Setup

**Why Manual Setup?** Tauri creates DMG installers for macOS, which are disk images (not PKG installers). DMG files don't support post-install scripts - users simply drag the app to their Applications folder. This means the CLI binary must be installed manually.

**Option 1: Copy from App Bundle (Recommended)**
```bash
# After installing Orkee.app to /Applications
sudo cp /Applications/Orkee.app/Contents/MacOS/orkee /usr/local/bin/orkee
sudo chmod +x /usr/local/bin/orkee

# Verify CLI access
orkee --version
orkee projects list
orkee tui
```

**Option 2: Copy without sudo (per-user)**
```bash
# Create user bin directory if needed
mkdir -p ~/.local/bin

# Copy binary from app bundle
cp /Applications/Orkee.app/Contents/MacOS/orkee ~/.local/bin/orkee
chmod +x ~/.local/bin/orkee

# Add to PATH if needed
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Verify CLI access
orkee --version
```

**Option 3: Use npm instead**
If manual setup is inconvenient, consider using the npm package which automatically handles CLI installation:
```bash
npm install -g orkee
```

The npm package provides identical functionality (CLI, TUI, web dashboard) and automatically adds `orkee` to your PATH.

## Configuration

The installer hooks are referenced in `tauri.conf.json`:

```json
{
  "bundle": {
    "externalBin": ["binaries/orkee"],
    "windows": {
      "nsis": {
        "installerHooks": "windows/hooks.nsh"
      }
    },
    "macOS": {
      "minimumSystemVersion": "10.13"
      // Note: DMG installers don't support post-install scripts
      // CLI installation must be done manually (see docs above)
    },
    "linux": {
      "deb": {
        "postInstallScript": "linux/postinstall.sh",
        "preRemoveScript": "linux/preuninstall.sh"
      }
    }
  }
}
```

## Troubleshooting

### Windows: PATH not updated
- Close and reopen terminal (PATH changes require new session)
- Check `echo %PATH%` includes Orkee bin directory
- May need to restart Windows Explorer (`taskkill /f /im explorer.exe && start explorer.exe`)

### macOS: Permission denied
- The postinstall script requires admin privileges
- User will be prompted for password during installation
- If installation fails, try running with `sudo`

### Linux: Binary not found
- Check if `/usr/local/bin` is in PATH: `echo $PATH`
- Verify binary exists: `ls -l /usr/local/bin/orkee`
- Try running directly: `/usr/local/bin/orkee --version`
- Add to PATH manually if needed: `export PATH="/usr/local/bin:$PATH"`

### AppImage: CLI not available
- AppImages don't automatically install CLI access - follow manual setup instructions above
- Extract the binary: `./Orkee*.AppImage --appimage-extract`
- Copy to a directory in your PATH: `/usr/local/bin` or `~/.local/bin`
- Make executable: `chmod +x /usr/local/bin/orkee`

## Security Considerations

- **Windows**: Per-user installs don't require admin privileges (uses `%LOCALAPPDATA%`)
- **macOS**: Requires admin to write to `/usr/local/bin` (standard practice)
- **Linux**: Package managers typically run post-install scripts as root
- **Verification**: Users can verify the binary hash against GitHub releases

## CI/CD Integration

The GitHub Actions workflow (`.github/workflows/tauri-release.yml`) automatically builds installers with these scripts included. Triggered by tags starting with `desktop-v`.

```bash
git tag desktop-v0.0.2
git push origin desktop-v0.0.2
```

## npm vs Desktop Installation

Orkee provides two distribution methods with identical functionality:

### Desktop App (Native Installers)
- **Installation**: Platform-specific installers (.dmg, .msi, .deb, .rpm, .AppImage)
- **CLI Access**: Automatically added to system PATH during installation
- **Dashboard UI**: Opens in native Tauri window
- **Target Audience**: End users who prefer native app experience
- **Updates**: Manual download of new installers
- **Uninstall**: Standard OS uninstall procedures (removes CLI from PATH)

### npm Package
- **Installation**: `npm install -g orkee`
- **CLI Access**: Automatically added to PATH by npm
- **Dashboard UI**: Opens in default browser
- **Target Audience**: Developers, CI/CD environments, users comfortable with terminal
- **Updates**: `npm update -g orkee`
- **Uninstall**: `npm uninstall -g orkee`

### Key Differences

| Feature | Desktop App | npm Package |
|---------|-------------|-------------|
| GUI Container | Native window (Tauri) | Browser window |
| Installation | OS-specific installer | npm command |
| Auto-updates | Manual | Via npm |
| System Integration | Native (system tray, dock) | CLI only |
| Disk Space | ~50MB (includes Chromium) | ~25MB (uses system browser) |
| CLI/TUI Access | ✅ Automatic | ✅ Automatic |
| Web Dashboard | ✅ Included | ✅ Included |

### Recommendation
- **Desktop App**: Best for end users, provides native feel
- **npm Package**: Best for developers, lighter weight, easier updates
- **Both work identically**: Same CLI commands, TUI, and functionality

## Future Improvements

### Code Signing (Required for Production)
Current installers are **not code-signed**. For production releases, code signing is essential:

**macOS:**
- **Issue**: Users see "unidentified developer" warning, app won't run without right-click override
- **Solution**: Apple Developer Program membership ($99/year) + code signing certificate
- **Implementation**: Add `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, and `APPLE_ID` to GitHub secrets
- **Gatekeeper**: Signed apps bypass security warnings
- **Notarization**: Required for macOS 10.15+ (automated via Tauri action)

**Windows:**
- **Issue**: SmartScreen warnings, "Unknown publisher" messages
- **Solution**: Code signing certificate from trusted CA (Sectigo, DigiCert, etc.)
- **Implementation**: Add `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD` to GitHub secrets
- **Benefits**: Builds user trust, removes security warnings

**Linux:**
- **Status**: Not typically required, but GPG signatures recommended for repositories
- **Implementation**: Sign packages with GPG key for package manager distribution

**References:**
- [Tauri Code Signing Guide](https://v2.tauri.app/distribute/sign/)
- [Apple Developer Program](https://developer.apple.com/programs/)
- [Windows Code Signing](https://learn.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)

### Package Manager Distribution
For wider reach, consider distributing through platform package managers:

**Windows:**
- **WinGet**: Microsoft's official package manager
- **Chocolatey**: Popular community package manager
- **Scoop**: Command-line installer for Windows

**macOS:**
- **Homebrew**: De facto standard for macOS packages
- **MacPorts**: Alternative package manager

**Linux:**
- **Flathub**: Universal Flatpak repository
- **Snap Store**: Canonical's universal package format
- **AUR**: Arch User Repository (community-maintained)

These integrations would complement the existing npm distribution and native installers.

### First-Run CLI Setup Helper (macOS & AppImage)
**Problem**: macOS DMG and Linux AppImage installations don't support automatic CLI setup:
- **macOS**: DMG files are disk images, not installers - no post-install scripts
- **AppImage**: Portable format without installation hooks

Users must manually discover CLI installation steps, which hurts discoverability.

**Proposed Solution**: Add first-run detection in the Tauri desktop app to offer automatic CLI setup

**Implementation Approach:**
1. **Platform Detection**: Check if running on macOS or from AppImage (`APPIMAGE` env var)
2. **CLI Check**: Test if `orkee` CLI is available in PATH (`which orkee` or equivalent)
3. **First-Run Dialog**: Show modal on first launch (if unsupported platform + CLI not installed):
   - Explain CLI/TUI features are available
   - Provide "Install Now" button to trigger installation
   - Include "Remind Me Later" and "Don't Show Again" options
4. **Installation Logic**:
   - **macOS**: Use Tauri dialog to request admin password, copy binary to `/usr/local/bin`
   - **AppImage**: Bundle and execute `install-cli-from-appimage.sh` helper script
5. **State Persistence**: Store user choice in app config (`~/.orkee/config.json`)
6. **Error Handling**: Provide fallback manual instructions if automated install fails

**Benefits:**
- Improves discoverability of CLI/TUI features
- Reduces user friction for macOS and AppImage installations
- Provides guidance without requiring documentation reading
- Maintains security (user must approve admin actions)

**Complexity**: Medium-High
- Rust backend: Platform detection, CLI checking, binary copying with privileges
- Frontend: React dialog component, state management
- Security: Proper privilege escalation handling
- Testing: Manual testing across macOS versions and AppImage environments

**Status**: Tracked as future enhancement - to be implemented in separate PR

### Binary Signature Verification (Security Enhancement)
**Priority**: Medium - Enhances supply chain security and installation trust

**Problem**: Currently, installers verify binary existence but not authenticity. Users have no cryptographic assurance that binaries haven't been tampered with.

**Proposed Solution**: Add GPG signature verification to installer scripts

**Implementation Approach:**

**macOS/Linux:**
1. **Generate GPG Key**: Create project GPG key for signing binaries
2. **Sign Binaries**: Sign orkee binary during CI build (`gpg --detach-sign --armor orkee`)
3. **Bundle Signature**: Include `.sig` file alongside binary in installers
4. **Verify in Scripts**: Add verification step in post-install scripts:
   ```bash
   # Import public key (embedded or from keyserver)
   gpg --import orkee-public.key

   # Verify signature
   if gpg --verify orkee.sig orkee; then
       echo "✓ Binary signature verified"
   else
       echo "Warning: Binary signature verification failed"
       # Provide user choice: continue or abort
   fi
   ```

**Windows:**
- Already partially addressed by code signing (when implemented)
- GPG verification can complement Authenticode signatures

**Benefits:**
- Protects against supply chain attacks
- Verifies binary integrity (not just code signing)
- Works alongside (not instead of) platform code signing
- Open standard (GPG) provides transparency

**Challenges:**
- Adds complexity to CI/CD pipeline
- Requires secure key management
- User experience: how to handle verification failures?
- May need fallback for users without GPG installed

**References:**
- [GPG Binary Signing Best Practices](https://www.gnupg.org/)
- [Reproducible Builds Project](https://reproducible-builds.org/)

### Windows Registry Key Validation
**Priority**: Low - Defensive programming against edge cases

**Problem**: Windows installer assumes PATH registry values are well-formed strings. Corrupted or malformed PATH entries could cause unexpected behavior.

**Proposed Solution**: Add validation before PATH modification

**Implementation Approach:**
```nsis
; Read current PATH
ReadRegStr $1 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"

; Validate PATH format
${If} $1 == ""
  ; Empty PATH - safe to initialize
  StrCpy $1 "$0"
${ElseIf} ${StrContains} $1 ";" ","
  ; Malformed - contains both ; and , separators (should only be ;)
  MessageBox MB_OK|MB_ICONWARNING "Warning: PATH appears corrupted. Manual setup required."
  Goto skip_path_setup
${ElseIf} ${StrLen} $1 > 3000
  ; Suspiciously long PATH (beyond reasonable use)
  MessageBox MB_OK|MB_ICONWARNING "Warning: PATH is unusually long. Skipping automatic setup."
  Goto skip_path_setup
${Else}
  ; PATH looks valid - proceed with normal setup
  ; ... existing logic ...
${EndIf}

skip_path_setup:
; Continue installation without PATH setup
```

**Benefits:**
- Prevents installer from corrupting already-broken PATH
- Graceful handling of edge cases
- Better error messages for users with system issues

**Challenges:**
- Edge cases are rare (very low priority)
- Hard to test without intentionally corrupting registry
- May be overly defensive

**Complexity**: Low - simple string validation checks

## Known Limitations

### 1. Windows: PATH Mutation Race Condition
**File**: `windows/hooks.nsh:60, 82`
**Severity**: Low
**Type**: NSIS Framework Limitation

**Problem**: If another installer modifies the Windows PATH registry between reading and writing, those changes will be lost.

**Scenario**:
1. Orkee installer reads PATH from registry
2. Another installer (running simultaneously) modifies PATH
3. Orkee installer writes PATH back, overwriting the other installer's changes

**Impact**:
- Changes made by concurrent installers may be lost
- Unlikely in practice - most users don't run multiple installers simultaneously
- Worst case: User needs to reinstall the other application

**Root Cause**: NSIS framework limitation - no atomic read-modify-write operations for registry

**Mitigation Options Considered**:
- **Mutex/Lock**: NSIS doesn't provide cross-process synchronization primitives
- **Transaction API**: Windows offers `TxF` (Transactional NTFS) but deprecated since Windows Vista
- **Registry Transactions**: Not available through NSIS
- **Retry Logic**: Could add retry with delay, but doesn't eliminate race - only reduces probability

**Current Behavior**: Acceptable for typical use cases. Race window is small (milliseconds), and simultaneous installer execution is rare.

**Recommendation**: Document this limitation. Users experiencing PATH issues after installation should:
1. Close all installers before running Orkee installer
2. Manually verify PATH after installation: `echo %PATH%`
3. Reinstall affected applications if needed

**Future Consideration**: Consider migrating to WiX Toolset (if Tauri adds support), which provides better registry manipulation primitives.

### 2. Windows: PATH Duplication Edge Case
**File**: `windows/hooks.nsh:33`
**Severity**: Low

The `StrContains` check uses substring matching, which could theoretically create false positives:
- Example: `C:\Orkee\bin` would match inside `C:\Orkee\binary\tools`
- **Impact**: Worst case is a duplicate PATH entry, which is harmless
- **Likelihood**: Very low due to specific path patterns (`Program Files\Orkee\bin`, `LocalAppData\Orkee\bin`)
- **Mitigation**: Could implement exact path matching with delimiter checks, but adds complexity for minimal benefit

**Current behavior**: Acceptable for typical use cases

### 3. macOS: Multiple Installation Detection
**File**: `macos/postinstall.sh:23-29`
**Severity**: Low

If multiple versions of Orkee.app exist in different locations, the script picks the first match:
```bash
for location in "${POSSIBLE_LOCATIONS[@]}"; do
    if [ -d "$location" ]; then
        APP_BUNDLE="$location"
        break  # First match wins
    fi
done
```

**Scenarios**:
- User has Orkee in both `/Applications` and `~/Applications`
- Old versions not fully removed
- **Impact**: May link to wrong version
- **Likelihood**: Uncommon - users typically have one installation

**Potential improvements**:
- Accept installation path as installer argument
- Use macOS installer's built-in `$3` variable (target location)
- Check modification times and prefer newest
- Prompt user if multiple found

**Current behavior**: Works for standard single-installation scenarios

### 4. Linux: Version Verification Regex Limitation
**File**: `linux/postinstall.sh:13`
**Severity**: Low

The version verification regex only matches standard semver (X.Y.Z):
```bash
grep -oE '[0-9]+\.[0-9]+\.[0-9]+'
```

**Missing formats**:
- Pre-release: `0.0.2-rc1`, `1.0.0-beta.2`
- Build metadata: `1.0.0+20130313144700`
- Two-part versions: `1.0` (unusual but valid)

**Impact**: Version verification logs "unknown" for non-standard versions, but installation continues successfully. Does not affect functionality.

**Fix**: Use more comprehensive regex:
```bash
grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?'
```

**Current behavior**: Acceptable - verification is optional and only affects logging

## Additional Resources

- [Tauri Windows Installer Docs](https://v2.tauri.app/distribute/windows-installer/)
- [NSIS Documentation](https://nsis.sourceforge.io/Docs/)
- [macOS Package Scripts](https://developer.apple.com/library/archive/documentation/DeveloperTools/Reference/DistributionDefinitionRef/Chapters/Distribution_XML_Ref.html)
