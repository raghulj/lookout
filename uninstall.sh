#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
AUTOSTART_DIR="$HOME/.config/autostart"
CONFIG_DIR="$HOME/.config/lookout"

echo -e "${BLUE}🗑️  Lookout Uninstaller${NC}"
echo ""

# Check if lookout is installed
if [ ! -f "$INSTALL_DIR/lookout" ]; then
    echo -e "${YELLOW}⚠️  Lookout doesn't appear to be installed in $INSTALL_DIR${NC}"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Uninstall cancelled."
        exit 0
    fi
fi

# Confirm uninstall
echo "This will remove:"
echo "  - Binary: $INSTALL_DIR/lookout"
echo "  - Desktop entry: $DESKTOP_DIR/lookout.desktop"
echo "  - Autostart entry: $AUTOSTART_DIR/lookout.desktop (if exists)"
echo ""

read -p "Do you also want to remove settings? (y/n) " -n 1 -r
echo
REMOVE_SETTINGS=$REPLY

echo ""
read -p "Are you sure you want to uninstall Lookout? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstall cancelled."
    exit 0
fi

echo ""
echo -e "${BLUE}Removing Lookout...${NC}"

# Stop lookout if running
if pgrep -x "lookout" > /dev/null; then
    echo -e "   ${YELLOW}Stopping running instance...${NC}"
    pkill -x "lookout" || true
    sleep 1
fi

# Remove binary
if [ -f "$INSTALL_DIR/lookout" ]; then
    rm -f "$INSTALL_DIR/lookout"
    echo -e "   ${GREEN}✓${NC} Removed binary"
else
    echo -e "   ${YELLOW}⊘${NC} Binary not found"
fi

# Remove desktop entry
if [ -f "$DESKTOP_DIR/lookout.desktop" ]; then
    rm -f "$DESKTOP_DIR/lookout.desktop"
    echo -e "   ${GREEN}✓${NC} Removed desktop entry"
else
    echo -e "   ${YELLOW}⊘${NC} Desktop entry not found"
fi

# Remove autostart entry
if [ -f "$AUTOSTART_DIR/lookout.desktop" ]; then
    rm -f "$AUTOSTART_DIR/lookout.desktop"
    echo -e "   ${GREEN}✓${NC} Removed autostart entry"
else
    echo -e "   ${YELLOW}⊘${NC} Autostart entry not found"
fi

# Update desktop database (if available)
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

# Remove settings if requested
if [[ $REMOVE_SETTINGS =~ ^[Yy]$ ]]; then
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo -e "   ${GREEN}✓${NC} Removed settings directory"
    else
        echo -e "   ${YELLOW}⊘${NC} Settings directory not found"
    fi
else
    echo -e "   ${BLUE}ℹ${NC} Kept settings in $CONFIG_DIR"
fi

echo ""
echo -e "${GREEN}✅ Lookout has been uninstalled${NC}"

if [[ ! $REMOVE_SETTINGS =~ ^[Yy]$ ]]; then
    echo ""
    echo "Your settings are preserved in: $CONFIG_DIR"
    echo "To remove them manually, run:"
    echo -e "  ${BLUE}rm -rf $CONFIG_DIR${NC}"
fi

echo ""
echo "Sorry to see you go! If you have feedback, please share it at:"
echo "  https://github.com/raghulj/lookout/issues"
echo ""
