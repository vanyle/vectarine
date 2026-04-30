#!/bin/sh

# This script installs/updates Vectarine for macOS (Apple Silicon)
# It automates the process of downloading and unzipping the latest release
# If you want to install it manually, you can download the latest release from https://github.com/vanyle/vectarine
# Running this script is just slightly more convenient, you do you <3 !

set -eu

echo "Installing Vectarine for macOS..."

if ! command -v curl > /dev/null; then
    echo "Please install 'curl' on your system."
    exit 1
fi

if ! command -v unzip > /dev/null; then
    echo "Please install 'unzip' on your system."
    exit 1
fi

DOWNLOAD_URL=$(curl -L -s https://api.github.com/repos/vanyle/vectarine/releases/latest | grep "vectarine.macos.arm64.zip" | sed -n '2p' | grep -o -E 'https?://[^"]+')

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Failed to find the latest macOS release. Please check https://github.com/vanyle/vectarine/releases"
    exit 1
fi

TEMP_ZIP="/tmp/vectarine_macos.zip"
TEMP_DIR="/tmp/vectarine_macos_extract"

echo "Downloading $DOWNLOAD_URL ..."
curl -L "$DOWNLOAD_URL" -o "$TEMP_ZIP"

# Clean up any previous extraction
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

echo "Extracting..."
unzip -o "$TEMP_ZIP" -d "$TEMP_DIR"

# Remove existing installation if present
if [ -d "/Applications/vecta.app" ]; then
    echo "Updating existing installation..."
    rm -rf "/Applications/vecta.app"
fi

# Move to /Applications
mv "$TEMP_DIR/vecta.app" "/Applications/vecta.app"

# Remove the quarantine attribute so macOS doesn't block the app
xattr -d com.apple.quarantine "/Applications/vecta.app" 2>/dev/null || true

# Clean up temp files
rm -f "$TEMP_ZIP"
rm -rf "$TEMP_DIR"

echo "You're all set! Vectarine has been installed to /Applications/vecta.app"
echo "You can launch it from your Applications folder or Spotlight."
