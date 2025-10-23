#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="raghulj/lookout"
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
AUTOSTART_DIR="$HOME/.config/autostart"

echo -e "${BLUE}🔍 Detecting system...${NC}"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    armv7l)
        TARGET="armv7-unknown-linux-gnueabihf"
        ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: $ARCH${NC}"
        echo "Supported: x86_64, aarch64, armv7l"
        exit 1
        ;;
esac

echo -e "   Architecture: ${GREEN}$ARCH${NC}"
echo -e "   Target: ${GREEN}$TARGET${NC}"

# Detect OS (for helpful messages)
if [ -f /etc/os-release ]; then
    . /etc/os-release
    echo -e "   OS: ${GREEN}$NAME${NC}"
fi

echo ""

# Check dependencies
echo -e "${BLUE}✅ Checking dependencies...${NC}"

GTK4_FOUND=false
LIBADWAITA_FOUND=false

if command -v pkg-config &> /dev/null; then
    if pkg-config --exists gtk4 2>/dev/null; then
        GTK4_FOUND=true
        echo -e "   GTK4: ${GREEN}found${NC}"
    fi

    if pkg-config --exists libadwaita-1 2>/dev/null; then
        LIBADWAITA_FOUND=true
        echo -e "   libadwaita: ${GREEN}found${NC}"
    fi
fi

if [ "$GTK4_FOUND" = false ] || [ "$LIBADWAITA_FOUND" = false ]; then
    echo ""
    echo -e "${YELLOW}⚠️  Missing dependencies detected!${NC}"
    echo ""
    echo "Lookout requires GTK4 and libadwaita. Please install them:"
    echo ""
    if [ -f /etc/debian_version ]; then
        echo -e "  ${GREEN}sudo apt install libgtk-4-1 libadwaita-1-0${NC}"
    elif [ -f /etc/fedora-release ]; then
        echo -e "  ${GREEN}sudo dnf install gtk4 libadwaita${NC}"
    elif [ -f /etc/arch-release ]; then
        echo -e "  ${GREEN}sudo pacman -S gtk4 libadwaita${NC}"
    else
        echo "  Install gtk4 and libadwaita using your distribution's package manager"
    fi
    echo ""
    read -p "Continue installation anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 1
    fi
fi

echo ""

# Get latest release version
echo -e "${BLUE}⬇️  Fetching latest release...${NC}"

LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest")
VERSION=$(echo "$LATEST_RELEASE" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo -e "${RED}❌ Failed to fetch latest release${NC}"
    echo "Please check your internet connection or try again later."
    exit 1
fi

echo -e "   Latest version: ${GREEN}$VERSION${NC}"

# Construct download URL
BINARY_NAME="lookout-$TARGET"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$BINARY_NAME"

echo -e "   Downloading from: ${BLUE}$DOWNLOAD_URL${NC}"
echo ""

# Download binary
TEMP_FILE="/tmp/lookout-download-$$"
if ! curl -L --progress-bar "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
    echo -e "${RED}❌ Download failed${NC}"
    echo "URL: $DOWNLOAD_URL"
    echo ""
    echo "This might happen if:"
    echo "  - The release doesn't have a binary for your architecture ($TARGET)"
    echo "  - There's a network issue"
    echo "  - The release is still being built"
    rm -f "$TEMP_FILE"
    exit 1
fi

# Verify download
if [ ! -s "$TEMP_FILE" ]; then
    echo -e "${RED}❌ Downloaded file is empty${NC}"
    rm -f "$TEMP_FILE"
    exit 1
fi

echo ""
echo -e "${BLUE}📦 Installing...${NC}"

# Create directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"

# Install binary
mv "$TEMP_FILE" "$INSTALL_DIR/lookout"
chmod +x "$INSTALL_DIR/lookout"
echo -e "   Binary: ${GREEN}$INSTALL_DIR/lookout${NC}"

# Create desktop entry
DESKTOP_FILE="$DESKTOP_DIR/lookout.desktop"
cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Name=Lookout
Comment=AI-Powered Break Reminder for Linux
Exec=$INSTALL_DIR/lookout
Icon=preferences-desktop-display
Terminal=false
Type=Application
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
EOF

echo -e "   Desktop entry: ${GREEN}$DESKTOP_FILE${NC}"

# Update desktop database (if available)
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo ""

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo -e "${YELLOW}🛤️  ~/.local/bin is not in your PATH${NC}"
    echo ""

    # Detect shell and add to appropriate config
    SHELL_CONFIG=""
    if [ -n "$BASH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [ -f "$HOME/.profile" ]; then
        SHELL_CONFIG="$HOME/.profile"
    fi

    if [ -n "$SHELL_CONFIG" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_CONFIG"
        echo -e "   Added to PATH in ${GREEN}$SHELL_CONFIG${NC}"
        echo ""
        echo -e "${YELLOW}   Please restart your terminal or run:${NC}"
        echo -e "   ${GREEN}source $SHELL_CONFIG${NC}"
    else
        echo "   Please add this to your shell configuration:"
        echo -e "   ${GREEN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    fi
    echo ""
else
    echo -e "${GREEN}✅ ~/.local/bin is already in PATH${NC}"
    echo ""
fi

# Success message
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "Run 'lookout' to start the application, or find it in your app launcher."
echo ""
echo "Optional setup:"
echo -e "  ${BLUE}lookout --enable-autostart${NC}   Enable autostart on login"
echo ""
echo "Other commands:"
echo -e "  ${BLUE}lookout --version${NC}            Show version"
echo -e "  ${BLUE}lookout --help${NC}               Show help"
echo ""
echo "To update in the future:"
echo -e "  ${GREEN}curl -fsSL https://raw.githubusercontent.com/$REPO/main/install.sh | bash${NC}"
echo ""
echo "To uninstall:"
echo -e "  ${GREEN}curl -fsSL https://raw.githubusercontent.com/$REPO/main/uninstall.sh | bash${NC}"
echo ""

# Ask about autostart
read -p "Would you like to enable autostart? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    mkdir -p "$AUTOSTART_DIR"
    cp "$DESKTOP_FILE" "$AUTOSTART_DIR/lookout.desktop"
    echo -e "${GREEN}✅ Autostart enabled${NC}"
    echo ""
fi

echo -e "${BLUE}Thank you for installing Lookout!${NC}"
echo "Star the project: https://github.com/$REPO"
