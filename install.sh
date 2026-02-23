#!/bin/sh
set -e

REPO="madLinux7/dead-simple-cli-timer"
BIN="dstimer"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux)  os="linux" ;;
  Darwin) os="macos" ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64 | amd64) arch="x86_64" ;;
  arm64 | aarch64) arch="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

ASSET="${BIN}-${os}-${arch}"

# Resolve latest release tag
TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\(.*\)".*/\1/')"

if [ -z "$TAG" ]; then
  echo "Could not determine latest release tag."
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"

echo "Installing ${BIN} ${TAG} (${os}/${arch})..."

# Download
TMP="$(mktemp)"
curl -fsSL "$URL" -o "$TMP"
chmod +x "$TMP"

# Install
mkdir -p "$INSTALL_DIR"
mv "$TMP" "${INSTALL_DIR}/${BIN}"

echo "Installed to ${INSTALL_DIR}/${BIN}"

# PATH hint
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "  Add ${INSTALL_DIR} to your PATH:"
    echo "    export PATH=\"\$PATH:${INSTALL_DIR}\""
    ;;
esac
