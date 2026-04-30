#!/bin/sh

# This script installs/updates Vectarine for Linux
# It automates the process of downloading and unzipping the latest release
# If you want to install it manually, you can download the latest release from https://github.com/vanyle/vectarine
# Running this script is just slightly more convenient, you do you <3 !

set -eu

echo "Installing Vectarine for Linux..."
cd $HOME

if ! command -v curl > /dev/null; then
    echo " Please install 'curl' on your system using your favorite package manager. "
    exit 1
fi

if command -v unzip > /dev/null; then
    :
elif command -v python3 > /dev/null; then
    :
elif command -v python > /dev/null; then
    :
else
    echo "Please install 'unzip' to extract the downloaded file."
    exit 1
fi

DOWNLOAD_URL=$(curl -L -s https://api.github.com/repos/vanyle/vectarine/releases/latest | grep "vectarine.linux.x86_64" | sed -n '2p' | grep -o -E 'https?://[^"]+') || true

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Failed to find download URL. GitHub API may be rate-limited."
    exit 1
fi

INSTALL_DIR="$HOME/.local/share/vectarine"
BIN_DIR="$HOME/.local/bin"
TEMP_ZIP="/tmp/vectarine.zip"

if [ -d "$INSTALL_DIR" ]; then
    echo "Updating existing installation..."
    # Safety: $INSTALL_DIR is defined above and always targets xxx/.local/share/vectarine, so we won't accidentally delete too many things.
    rm -rf "$INSTALL_DIR"
else
    echo "No existing installation found, installing fresh..."
fi

mkdir -p "$INSTALL_DIR"

echo ""
echo "Downloading $DOWNLOAD_URL ..."
# We install vectarine to ~/.local/share/vectarine and we symlink vectarine to ~/.local/bin so that it's in the path.
curl -L "$DOWNLOAD_URL" -o "$TEMP_ZIP"

if [ ! -f "$TEMP_ZIP" ]; then
    echo "Failed to download the file. Please check your internet connection and try again or download the latest release manually from https://github.com/vanyle/vectarine/releases"
    exit 1
fi

# Unzip
if command -v unzip > /dev/null; then
    unzip -q -o "$TEMP_ZIP" -d "$INSTALL_DIR"
elif command -v python3 > /dev/null; then
    python3 -m zipfile -e "$TEMP_ZIP" "$INSTALL_DIR"
elif command -v python > /dev/null; then
    python -m zipfile -e "$TEMP_ZIP" "$INSTALL_DIR"
else
    echo "Please install 'unzip' to extract the downloaded file."
    exit 1
fi

# Make the binary executable
if [ -f "$INSTALL_DIR/vecta" ]; then
    chmod +x "$INSTALL_DIR/vecta"
    
    # Update the symlink to ~/.local/bin
    mkdir -p "$BIN_DIR"
    ln -sf "$INSTALL_DIR/vecta" "$BIN_DIR/vecta"
else
    echo "Warning: Binary vecta not found in the extracted files."
fi

# Clean up the download
rm -f "$TEMP_ZIP"

echo ""
echo ""
echo "You're all set!"
echo "Vectarine has been installed to $INSTALL_DIR"
echo ""
echo "It can be ran using $BIN_DIR/vecta or simply 'vecta' if $BIN_DIR is in your PATH."
echo ""
echo "If you want to add Vectarine to your path, add the following to your .bashrc / .zshrc"
echo "export PATH=\"$BIN_DIR:\$PATH\""
echo "Note: You might not need to, ~/.local/bin might already be in your path"
