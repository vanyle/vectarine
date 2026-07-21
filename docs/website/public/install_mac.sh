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

API_RESPONSE=$(curl -L -s -w "\n%{http_code}" https://api.github.com/repos/vanyle/vectarine/releases/latest) || true
HTTP_CODE=$(echo "$API_RESPONSE" | tail -n1)
API_BODY=$(echo "$API_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" != "200" ]; then
    echo "GitHub API request failed with status $HTTP_CODE"
    echo "Please retry or visit https://github.com/vanyle/vectarine/releases/latest for a manual download."
    exit 1
fi

DOWNLOAD_URL=$(echo "$API_BODY" | grep "vectarine.macos.arm64" | sed -n '2p' | grep -o -E 'https?://[^"]+') || true

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Failed to find the latest macOS release. Retry or visit https://github.com/vanyle/vectarine/releases/latest for a manual download."
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
if [ -d "/Applications/VectarineEditor.app" ]; then
    echo "Updating existing installation..."
    rm -rf "/Applications/VectarineEditor.app"
fi

# Move to /Applications
mv "$TEMP_DIR/VectarineEditor.app" "/Applications/VectarineEditor.app"

# Remove the quarantine attribute so macOS doesn't block the app
xattr -d com.apple.quarantine "/Applications/VectarineEditor.app" 2>/dev/null || true

# Clean up temp files
rm -f "$TEMP_ZIP"
rm -rf "$TEMP_DIR"

echo "You're all set! Vectarine has been installed to /Applications/VectarineEditor.app"
echo "The CLI tool was not installed. You can install it manually by putting it in your PATH."
echo "You can launch it from your Applications folder or Spotlight."
