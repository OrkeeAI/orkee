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
- **Type**: Bash scripts executed by package manager
- **Location**: `/usr/local/bin/orkee` (symlink or copy)
- **PATH Setup**: `/usr/local/bin` is typically in PATH
- **Fallback**: If symlink fails, copies binary instead
- **Scripts**:
  - `postinstall.sh`: Creates symlink or copies binary
  - `preuninstall.sh`: Removes symlink/binary

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

### Linux
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

## Additional Resources

- [Tauri Windows Installer Docs](https://v2.tauri.app/distribute/windows-installer/)
- [NSIS Documentation](https://nsis.sourceforge.io/Docs/)
- [macOS Package Scripts](https://developer.apple.com/library/archive/documentation/DeveloperTools/Reference/DistributionDefinitionRef/Chapters/Distribution_XML_Ref.html)
