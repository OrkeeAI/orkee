#!/bin/bash
# Automated Release Script for Orkee
# This script automates the entire release process

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Parse command line arguments
VERSION=""
SKIP_TESTS=false
SKIP_NPM=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-npm)
            SKIP_NPM=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --version <version>  Specify version (e.g., 0.0.1)"
            echo "  --skip-tests        Skip running tests"
            echo "  --skip-npm          Skip NPM publishing"
            echo "  --dry-run           Run without actually releasing"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Get version from package.json if not specified
if [ -z "$VERSION" ]; then
    VERSION=$(cat packages/cli/package.json | grep '"version"' | cut -d'"' -f4)
    print_warning "No version specified, using version from package.json: $VERSION"
fi

print_step "Starting release process for version $VERSION"

if [ "$DRY_RUN" = true ]; then
    print_warning "DRY RUN MODE - No actual changes will be made"
fi

# 1. Pre-release checks
print_step "Running pre-release checks..."

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    print_error "Not on main branch. Current branch: $CURRENT_BRANCH"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    print_error "There are uncommitted changes. Please commit or stash them."
    exit 1
fi

# Pull latest changes
print_step "Pulling latest changes from origin..."
git pull origin main

print_success "Pre-release checks passed"

# 2. Run tests
if [ "$SKIP_TESTS" = false ]; then
    print_step "Running all tests..."
    
    # Run Rust tests
    print_step "Running Rust tests..."
    cargo test --all
    print_success "Rust tests passed"
    
    # Run cargo check
    print_step "Running cargo check..."
    cargo check --all
    print_success "Cargo check passed"
    
    # Run cargo clippy
    print_step "Running cargo clippy..."
    cargo clippy --all -- -D warnings || print_warning "Clippy warnings found (non-blocking)"
    
    # Run cargo fmt check
    print_step "Checking code formatting..."
    cargo fmt --all -- --check || print_warning "Code formatting issues found (non-blocking)"
else
    print_warning "Skipping tests (--skip-tests flag)"
fi

# 3. Build binaries for all platforms
print_step "Building binaries for all platforms..."

# Create release directory
rm -rf release-artifacts
mkdir -p release-artifacts

# Function to build for a target
build_target() {
    local TARGET=$1
    local ARCHIVE_NAME=$2
    local IS_WINDOWS=$3
    
    print_step "Building for $TARGET..."
    
    if [ "$DRY_RUN" = false ]; then
        # Try to build with cross first, fallback to cargo
        if command -v cross &> /dev/null; then
            cross build --release --target "$TARGET" 2>/dev/null || cargo build --release --target "$TARGET"
        else
            cargo build --release --target "$TARGET"
        fi
        
        if [ $? -eq 0 ]; then
            # Create archive
            if [ "$IS_WINDOWS" = "true" ]; then
                # For Windows, create a zip file
                cd "target/$TARGET/release"
                zip "../../../release-artifacts/$ARCHIVE_NAME" orkee.exe
                cd ../../..
            else
                # For Unix, create a tar.gz
                tar -czf "release-artifacts/$ARCHIVE_NAME" \
                    -C "target/$TARGET/release" orkee
            fi
            print_success "Built $TARGET"
        else
            print_warning "Failed to build $TARGET (skipping)"
        fi
    else
        print_step "[DRY RUN] Would build for $TARGET"
    fi
}

# Build for each platform
build_target "aarch64-apple-darwin" "orkee-aarch64-apple-darwin.tar.gz" "false"
build_target "x86_64-apple-darwin" "orkee-x86_64-apple-darwin.tar.gz" "false"
build_target "x86_64-unknown-linux-gnu" "orkee-x86_64-unknown-linux-gnu.tar.gz" "false"
build_target "aarch64-unknown-linux-gnu" "orkee-aarch64-unknown-linux-gnu.tar.gz" "false"
build_target "x86_64-pc-windows-msvc" "orkee-x86_64-pc-windows-msvc.zip" "true"

print_success "All binaries built successfully"

# 4. Create Git tag
print_step "Creating Git tag v$VERSION..."

if [ "$DRY_RUN" = false ]; then
    # Check if tag already exists
    if git rev-parse "v$VERSION" >/dev/null 2>&1; then
        print_error "Tag v$VERSION already exists"
        exit 1
    fi
    
    git tag -a "v$VERSION" -m "Release v$VERSION"
    git push origin "v$VERSION"
    print_success "Git tag created and pushed"
else
    print_step "[DRY RUN] Would create and push tag v$VERSION"
fi

# 5. Create GitHub release
print_step "Creating GitHub release..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    print_error "GitHub CLI (gh) is not installed. Please install it first."
    exit 1
fi

# Create release notes
RELEASE_NOTES="## Orkee v$VERSION

### Features
- 🤖 AI Agent Orchestration
- 📊 Real-time Dashboard
- 🖥️ Terminal Interface (TUI)
- 🔧 CLI Tools
- 🔗 Workflow Coordination
- ☁️ Cloud Sync (optional)
- 🔐 Enterprise Security
- 🔒 HTTPS/TLS Support

### Installation

#### Via npm (recommended)
\`\`\`bash
npm install -g orkee
\`\`\`

#### Direct Download
Download the appropriate binary for your platform from the assets below.

### Supported Platforms
- macOS (Apple Silicon & Intel)
- Linux (x64 & ARM64)
- Windows (x64)

### Getting Started
\`\`\`bash
orkee --help
orkee dashboard
orkee tui
\`\`\`

See [README](https://github.com/OrkeeAI/orkee#readme) for full documentation."

if [ "$DRY_RUN" = false ]; then
    # Create the release with all artifacts
    gh release create "v$VERSION" \
        --title "Orkee v$VERSION" \
        --notes "$RELEASE_NOTES" \
        release-artifacts/*.tar.gz \
        release-artifacts/*.zip
    
    print_success "GitHub release created"
else
    print_step "[DRY RUN] Would create GitHub release with:"
    echo "$RELEASE_NOTES"
    echo ""
    echo "Artifacts:"
    ls -la release-artifacts/ 2>/dev/null || echo "  (no artifacts in dry run)"
fi

# 6. Publish to NPM
if [ "$SKIP_NPM" = false ]; then
    print_step "Publishing to NPM..."
    
    # Navigate to cli package directory
    cd packages/cli/
    
    # Check if logged in to npm
    if ! npm whoami >/dev/null 2>&1; then
        print_error "Not logged in to npm. Please run 'npm login' first."
        exit 1
    fi
    
    if [ "$DRY_RUN" = false ]; then
        # Publish to npm
        npm publish --access public
        print_success "Published to NPM"
    else
        print_step "[DRY RUN] Would publish to NPM"
        npm pack
        print_step "Package created: $(ls *.tgz)"
    fi
    
    cd ../..
else
    print_warning "Skipping NPM publish (--skip-npm flag)"
fi

# 7. Post-release verification
print_step "Running post-release verification..."

if [ "$DRY_RUN" = false ]; then
    # Verify GitHub release
    if gh release view "v$VERSION" >/dev/null 2>&1; then
        print_success "GitHub release verified"
    else
        print_error "GitHub release not found"
    fi
    
    # Verify npm package (if published)
    if [ "$SKIP_NPM" = false ]; then
        if npm view orkee@$VERSION >/dev/null 2>&1; then
            print_success "NPM package verified"
        else
            print_warning "NPM package not yet available (may take a few minutes)"
        fi
    fi
fi

# 8. Cleanup
print_step "Cleaning up..."
if [ "$DRY_RUN" = false ]; then
    rm -rf release-artifacts/
    print_success "Cleanup complete"
fi

# 9. Prepare for next version
print_step "Next steps:"
echo "1. Update version in packages/cli/Cargo.toml for next release"
echo "2. Update version in packages/cli/package.json for next release"
echo "3. Commit version bump: git commit -am 'chore: bump version'"
echo "4. Push changes: git push origin main"

print_success "🎉 Release v$VERSION completed successfully!"

if [ "$DRY_RUN" = true ]; then
    print_warning "This was a dry run. To perform actual release, run without --dry-run flag"
fi