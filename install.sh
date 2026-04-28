#!/usr/bin/env bash

set -e

VERSION="${NEXUS_VERSION:-0.1.0}"
REPO="GTgyani206/nexus"
INSTALL_DIR="${NEXUS_INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)" in
  Darwin) OS="macos" ;;
  Linux) OS="linux" ;;
  *)
    echo "Unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64) ARCH="x86_64" ;;
  aarch64 | arm64) ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $(uname -m)"
    exit 1
    ;;
esac

FILENAME="nexus_${VERSION}_${OS}_${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${FILENAME}"

echo "Installing nexus ${VERSION} for ${OS}/${ARCH}..."
mkdir -p "$INSTALL_DIR"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" | tar -xz -C "$INSTALL_DIR"
elif command -v wget >/dev/null 2>&1; then
  wget -qO- "$URL" | tar -xz -C "$INSTALL_DIR"
else
  echo "Either curl or wget is required."
  exit 1
fi

chmod +x "$INSTALL_DIR/nexus"
echo "Installed: $INSTALL_DIR/nexus"

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo "Add this to your shell profile:"
  echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo "Run: nexus --help"
