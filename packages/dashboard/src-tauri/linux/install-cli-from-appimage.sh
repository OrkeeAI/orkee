#!/usr/bin/env bash
# ABOUTME: Helper script to extract and install orkee CLI from AppImage
# ABOUTME: Automates manual AppImage CLI extraction for easier user setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Orkee CLI Installation Helper for AppImage"
echo "==========================================="
echo ""

# Find AppImage in current directory
APPIMAGE=$(find . -maxdepth 1 -name "Orkee*.AppImage" -o -name "orkee*.AppImage" | head -n1)

if [ -z "$APPIMAGE" ]; then
    echo -e "${RED}Error: No Orkee AppImage found in current directory${NC}"
    echo ""
    echo "Please download the Orkee AppImage and run this script from the same directory."
    echo "Or specify the AppImage path as an argument:"
    echo "  $0 /path/to/Orkee.AppImage"
    exit 1
fi

# Allow override via command line argument
if [ -n "$1" ]; then
    APPIMAGE="$1"
fi

# Verify AppImage exists and is executable
if [ ! -f "$APPIMAGE" ]; then
    echo -e "${RED}Error: AppImage not found: $APPIMAGE${NC}"
    exit 1
fi

echo -e "Found AppImage: ${GREEN}$APPIMAGE${NC}"
echo ""

# Make executable if needed
if [ ! -x "$APPIMAGE" ]; then
    echo "Making AppImage executable..."
    chmod +x "$APPIMAGE"
fi

# Extract AppImage
echo "Extracting AppImage..."
"$APPIMAGE" --appimage-extract > /dev/null 2>&1

# Verify binary was extracted
EXTRACTED_BINARY="squashfs-root/usr/bin/orkee"
if [ ! -f "$EXTRACTED_BINARY" ]; then
    echo -e "${RED}Error: orkee binary not found in AppImage${NC}"
    echo "Expected location: $EXTRACTED_BINARY"
    rm -rf squashfs-root
    exit 1
fi

echo -e "${GREEN}✓${NC} Binary extracted successfully"
echo ""

# Ask user where to install
echo "Where would you like to install the orkee CLI?"
echo ""
echo "  1) /usr/local/bin/orkee (recommended, requires sudo)"
echo "  2) ~/.local/bin/orkee (no sudo needed)"
echo "  3) Custom location"
echo ""
read -p "Enter choice [1-3] (default: 1): " choice
choice=${choice:-1}

case "$choice" in
    1)
        TARGET_DIR="/usr/local/bin"
        TARGET_PATH="$TARGET_DIR/orkee"
        NEEDS_SUDO=true
        ;;
    2)
        TARGET_DIR="$HOME/.local/bin"
        TARGET_PATH="$TARGET_DIR/orkee"
        NEEDS_SUDO=false
        ;;
    3)
        read -p "Enter full path (e.g., /opt/bin/orkee): " TARGET_PATH
        TARGET_DIR=$(dirname "$TARGET_PATH")
        # Check if target directory needs sudo
        if [ -w "$TARGET_DIR" ]; then
            NEEDS_SUDO=false
        else
            NEEDS_SUDO=true
        fi
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        rm -rf squashfs-root
        exit 1
        ;;
esac

echo ""
echo -e "Installing to: ${GREEN}$TARGET_PATH${NC}"

# Create target directory if needed
if [ ! -d "$TARGET_DIR" ]; then
    echo "Creating directory: $TARGET_DIR"
    if [ "$NEEDS_SUDO" = true ]; then
        if ! sudo mkdir -p "$TARGET_DIR"; then
            echo -e "${RED}Error: Failed to create directory $TARGET_DIR${NC}"
            rm -rf squashfs-root
            exit 1
        fi
    else
        if ! mkdir -p "$TARGET_DIR"; then
            echo -e "${RED}Error: Failed to create directory $TARGET_DIR${NC}"
            rm -rf squashfs-root
            exit 1
        fi
    fi
fi

# Copy binary
if [ "$NEEDS_SUDO" = true ]; then
    echo "Copying binary (requires sudo)..."
    if ! sudo cp "$EXTRACTED_BINARY" "$TARGET_PATH"; then
        echo -e "${RED}Error: Failed to copy binary to $TARGET_PATH${NC}"
        rm -rf squashfs-root
        exit 1
    fi
    if ! sudo chmod +x "$TARGET_PATH"; then
        echo -e "${RED}Error: Failed to make binary executable${NC}"
        rm -rf squashfs-root
        exit 1
    fi
else
    echo "Copying binary..."
    if ! cp "$EXTRACTED_BINARY" "$TARGET_PATH"; then
        echo -e "${RED}Error: Failed to copy binary to $TARGET_PATH${NC}"
        rm -rf squashfs-root
        exit 1
    fi
    if ! chmod +x "$TARGET_PATH"; then
        echo -e "${RED}Error: Failed to make binary executable${NC}"
        rm -rf squashfs-root
        exit 1
    fi
fi

# Clean up extracted files
echo "Cleaning up..."
rm -rf squashfs-root

echo ""
echo -e "${GREEN}✓ Installation complete!${NC}"
echo ""

# Verify installation
if [ -f "$TARGET_PATH" ] && [ -x "$TARGET_PATH" ]; then
    echo "Verifying installation..."
    if "$TARGET_PATH" --version > /dev/null 2>&1; then
        VERSION=$("$TARGET_PATH" --version)
        echo -e "${GREEN}✓${NC} orkee CLI is working: $VERSION"
    else
        echo -e "${YELLOW}Note: Binary installed but version check failed${NC}"
        echo "You can still try using: orkee --version"
    fi
else
    echo -e "${RED}Error: Binary not found at $TARGET_PATH${NC}"
    exit 1
fi

# Check if target directory is in PATH
if [[ ":$PATH:" != *":$TARGET_DIR:"* ]]; then
    echo ""
    echo -e "${YELLOW}Warning: $TARGET_DIR is not in your PATH${NC}"
    echo ""
    echo "Add it to your PATH by running:"

    # Detect shell
    if [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    else
        SHELL_RC="$HOME/.profile"
    fi

    echo -e "  ${GREEN}echo 'export PATH=\"$TARGET_DIR:\$PATH\"' >> $SHELL_RC${NC}"
    echo -e "  ${GREEN}source $SHELL_RC${NC}"
    echo ""
fi

echo ""
echo "You can now use orkee commands:"
echo "  orkee --version"
echo "  orkee projects list"
echo "  orkee tui"
echo ""
