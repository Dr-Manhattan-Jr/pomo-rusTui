#!/bin/bash
set -e

REPO="Dr-Manhattan-Jr/pomo-rusTui"
BINARY="pomo-rustui"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64) TARGET="linux-x86_64" ;;
            aarch64|arm64) TARGET="linux-aarch64" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    darwin)
        case "$ARCH" in
            x86_64) TARGET="macos-x86_64" ;;
            aarch64|arm64) TARGET="macos-aarch64" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Get latest release
echo "Fetching latest release..."
LATEST=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Failed to fetch latest release"
    exit 1
fi

echo "Latest version: $LATEST"

# Download
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST/pomo-rustui-$TARGET.tar.gz"
echo "Downloading from $DOWNLOAD_URL..."

TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

curl -sL "$DOWNLOAD_URL" -o "$TMPDIR/pomo-rustui.tar.gz"

# Extract
echo "Extracting..."
tar -xzf "$TMPDIR/pomo-rustui.tar.gz" -C "$TMPDIR"

# Install
echo "Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMPDIR/pomo-rusTui" "$INSTALL_DIR/$BINARY"
else
    sudo mv "$TMPDIR/pomo-rusTui" "$INSTALL_DIR/$BINARY"
fi

chmod +x "$INSTALL_DIR/$BINARY"

echo "Installed $BINARY $LATEST to $INSTALL_DIR/$BINARY"
echo "Run '$BINARY' to start"
