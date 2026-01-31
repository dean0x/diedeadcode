#!/bin/sh
# Install script for diedeadcode (ddd)
# Usage: curl -fsSL https://raw.githubusercontent.com/dean0x/diedeadcode/main/install.sh | sh

set -e

REPO="dean0x/diedeadcode"
BINARY_NAME="ddd"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin) PLATFORM="darwin" ;;
  linux) PLATFORM="linux" ;;
  mingw*|msys*|cygwin*) PLATFORM="win32" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Detect musl (Alpine, etc.)
LIBC=""
if [ "$PLATFORM" = "linux" ]; then
  if ldd --version 2>&1 | grep -q musl || [ -f /etc/alpine-release ]; then
    LIBC="-musl"
  fi
fi

# Construct artifact name
if [ "$PLATFORM" = "win32" ]; then
  ARTIFACT="ddd-${PLATFORM}-${ARCH}.exe"
else
  ARTIFACT="ddd-${PLATFORM}-${ARCH}${LIBC}"
fi

# Get latest release version
VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
  echo "Failed to get latest version"
  exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}"

echo "Installing diedeadcode ${VERSION} (${PLATFORM}-${ARCH}${LIBC})..."
echo "Downloading from: ${DOWNLOAD_URL}"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download binary
curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${BINARY_NAME}"

# Make executable
chmod +x "${TMP_DIR}/${BINARY_NAME}"

# Install
if [ -w "$INSTALL_DIR" ]; then
  mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
else
  echo "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
fi

echo ""
echo "✓ Installed ddd to ${INSTALL_DIR}/${BINARY_NAME}"
echo ""
echo "Run 'ddd --help' to get started"
