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
- **Type**: Bash scripts executed by macOS installer
- **Location**: `/usr/local/bin/orkee`
- **PATH Setup**: `/usr/local/bin` is already in PATH on macOS
- **Permissions**: Requires admin privileges for `/usr/local/bin` access
- **Scripts**:
  - `postinstall.sh`: Copies binary from app bundle to `/usr/local/bin`
  - `preuninstall.sh`: Removes binary from `/usr/local/bin`

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

# Install the .dmg
open src-tauri/target/release/bundle/dmg/Orkee_*.dmg

# Verify CLI access
orkee --version
orkee projects list
orkee tui
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
      "files": {
        "Scripts/postinstall": "macos/postinstall.sh",
        "Scripts/preuninstall": "macos/preuninstall.sh"
      }
    },
    "linux": {
      "deb": {
        "files": {
          "usr/share/orkee/postinstall.sh": "linux/postinstall.sh",
          "usr/share/orkee/preuninstall.sh": "linux/preuninstall.sh"
        }
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

### AppImage First-Run CLI Setup Prompt
**Problem**: AppImage users must manually discover the CLI installation helper script since AppImages don't support post-install hooks.

**Solution**: Add first-run detection in the Tauri desktop app to prompt AppImage users:

**Implementation Approach:**
1. **Detection**: Check if running from AppImage (`APPIMAGE` env var or `squashfs` mount detection)
2. **CLI Check**: Test if `orkee` CLI is available in PATH (`which orkee` or equivalent)
3. **First-Run Dialog**: Show modal on first launch (if AppImage + CLI not installed):
   - Explain CLI/TUI features are available
   - Provide "Install Now" button to launch helper script
   - Include "Remind Me Later" and "Don't Show Again" options
4. **State Persistence**: Store user choice in app config (`~/.orkee/config.json`)
5. **Helper Integration**: Either:
   - Bundle the `install-cli-from-appimage.sh` script in the app
   - Open browser to script URL with instructions
   - Provide terminal commands to copy/paste

**Benefits:**
- Improves discoverability of CLI/TUI features
- Reduces user friction for AppImage installations
- Provides guidance without requiring documentation reading

**Complexity**: Medium - requires Rust backend detection + React frontend dialog + state management

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

## Additional Resources

- [Tauri Windows Installer Docs](https://v2.tauri.app/distribute/windows-installer/)
- [NSIS Documentation](https://nsis.sourceforge.io/Docs/)
- [macOS Package Scripts](https://developer.apple.com/library/archive/documentation/DeveloperTools/Reference/DistributionDefinitionRef/Chapters/Distribution_XML_Ref.html)
