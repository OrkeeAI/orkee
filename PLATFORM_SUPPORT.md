# Platform Support

## Current Platform Support

### v0.0.1 (Released)
- ✅ macOS ARM64 (Apple Silicon) - `aarch64-apple-darwin`

### v0.0.2 (In Progress)
Platform support is being expanded. The following platforms will be supported:

- ✅ macOS ARM64 (Apple Silicon) - `aarch64-apple-darwin`  
- 🚧 macOS x64 (Intel) - `x86_64-apple-darwin` (OpenSSL linking issues)
- 🚧 Linux x64 - `x86_64-unknown-linux-gnu` (requires cross-compilation)
- 🚧 Linux ARM64 - `aarch64-unknown-linux-gnu` (requires cross-compilation)
- 🚧 Windows x64 - `x86_64-pc-windows-msvc` (requires Windows build environment)

## Building from Source

If your platform is not yet supported, you can build from source:

### Prerequisites
- Rust toolchain (latest stable)
- Node.js 18+ and pnpm
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/OrkeeAI/orkee.git
cd orkee

# Install dependencies
pnpm install

# Build the CLI
cd packages/cli
cargo build --release

# The binary will be at:
# target/release/orkee-cli (Unix)
# target/release/orkee-cli.exe (Windows)

# Create a symlink or add to PATH
sudo ln -s $(pwd)/target/release/orkee-cli /usr/local/bin/orkee
```

## Cross-Platform Building

To build for all platforms from a single machine:

### Using Cross (Recommended for Linux targets)

```bash
# Install cross
cargo install cross

# Build for Linux x64
cross build --release --target x86_64-unknown-linux-gnu

# Build for Linux ARM64  
cross build --release --target aarch64-unknown-linux-gnu
```

### macOS Cross-Compilation

For macOS x86_64 on Apple Silicon:

```bash
# Install x86_64 target
rustup target add x86_64-apple-darwin

# Set OpenSSL environment (requires Homebrew OpenSSL)
export OPENSSL_DIR=$(brew --prefix openssl@3)
export PKG_CONFIG_PATH=$OPENSSL_DIR/lib/pkgconfig

# Build
cargo build --release --target x86_64-apple-darwin
```

### Windows Cross-Compilation

Windows cross-compilation from Unix requires additional setup:

```bash
# Install Windows target
rustup target add x86_64-pc-windows-msvc

# Install xwin for Windows libraries
cargo install xwin
xwin download

# Build (requires additional configuration)
cargo build --release --target x86_64-pc-windows-msvc
```

## Platform-Specific Issues

### macOS
- **OpenSSL**: x86_64 builds require OpenSSL 3. Install with `brew install openssl@3`
- **Code signing**: Binaries may need to be signed for distribution

### Linux
- **GLIBC version**: Binaries are built against specific GLIBC versions
- **Static linking**: Consider static linking for better compatibility

### Windows  
- **MSVC runtime**: Requires Visual C++ Redistributable
- **Path separators**: Ensure proper path handling

## Contributing Platform Support

We welcome contributions to expand platform support! 

### How to Help

1. **Test on your platform**: Build from source and report issues
2. **Submit binaries**: Build for your platform and submit a PR
3. **Improve build scripts**: Help automate cross-compilation
4. **Documentation**: Update this guide with platform-specific notes

### Testing Checklist

Before submitting platform support:

- [ ] Binary builds successfully
- [ ] All tests pass (`cargo test`)
- [ ] CLI commands work (`orkee --help`, `orkee dashboard`)
- [ ] Dashboard starts and connects
- [ ] TUI mode works (`orkee tui`)
- [ ] Project management works (create, list, delete)

## Roadmap

### Near Term (v0.0.2-v0.1.0)
- Fix OpenSSL issues for macOS x64
- Add Linux x64 and ARM64 support via cross
- Improve npm postinstall script reliability

### Medium Term (v0.2.0+)
- Windows native support
- FreeBSD support
- Additional architectures (RISC-V, etc.)
- Static binaries for Linux
- Homebrew formula for macOS
- Snap/Flatpak packages for Linux

### Long Term
- Package managers for all platforms
- Auto-updating mechanism
- Platform-specific optimizations

## Support Matrix

| Platform | Architecture | v0.0.1 | v0.0.2 | Future |
|----------|-------------|--------|---------|--------|
| macOS | ARM64 | ✅ | ✅ | ✅ |
| macOS | x64 | ❌ | 🚧 | ✅ |
| Linux | x64 | ❌ | 🚧 | ✅ |
| Linux | ARM64 | ❌ | 🚧 | ✅ |
| Windows | x64 | ❌ | ❌ | ✅ |
| Windows | ARM64 | ❌ | ❌ | 🤔 |
| FreeBSD | x64 | ❌ | ❌ | 🤔 |

Legend:
- ✅ Fully supported
- 🚧 In progress
- 🤔 Planned
- ❌ Not supported

## Getting Help

If your platform is not supported:

1. Check [GitHub Issues](https://github.com/OrkeeAI/orkee/issues) for existing reports
2. Try building from source (see above)
3. Open an issue with your platform details
4. Join our Discord for community support

## Binary Naming Convention

Released binaries follow this naming pattern:
- `orkee-{target}` for Unix systems
- `orkee-{target}.exe` for Windows

Where `{target}` is the Rust target triple, e.g.:
- `orkee-aarch64-apple-darwin`
- `orkee-x86_64-unknown-linux-gnu`
- `orkee-x86_64-pc-windows-msvc.exe`

Archives are distributed as:
- `.tar.gz` for Unix systems  
- `.zip` for Windows