#!/usr/bin/env sh
set -eu pipefail
( set -o pipefail 2>/dev/null ) || true

REPO="AcidBurnHen/croner"
BIN="croner"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERSION="${VERSION:-latest}"  # e.g. v0.1.2 or "latest"

# Resolve version tag
if [ "$VERSION" = "latest" ]; then
    VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    sed -n 's/.*"tag_name": *"\(.*\)".*/\1/p')"
fi

# Detect OS/Arch
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
    Linux)  OS=unknown-linux-musl ;;
    Darwin) OS=apple-darwin ;;
    *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac
case "$ARCH" in
    x86_64|amd64) ARCH=x86_64 ;;
    arm64|aarch64) ARCH=aarch64 ;;
    *) echo "Unsupported ARCH: $ARCH" >&2; exit 1 ;;
esac

EXT="tar.gz"
[ "$OS" = "x86_64-pc-windows-msvc" ] && EXT="zip" # not used on Unix

ASSET="${BIN}-${VERSION}-${ARCH}-${OS}.${EXT}"
BASE="https://github.com/${REPO}/releases/download/${VERSION}"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
echo "Downloading ${ASSET} ..."
curl -fsSL "${BASE}/${ASSET}" -o "${TMP}/${ASSET}"

# Optional checksum (per-file .sha256 is in your release)
if curl -fsSL "${BASE}/${ASSET}.sha256" -o "${TMP}/${ASSET}.sha256" 2>/dev/null; then
    echo "Verifying checksum..."
    SUM_EXPECTED="$(cut -d' ' -f1 < "${TMP}/${ASSET}.sha256")"
    if command -v sha256sum >/dev/null 2>&1; then
        SUM_ACTUAL="$(sha256sum "${TMP}/${ASSET}" | awk '{print $1}')"
    else
        SUM_ACTUAL="$(shasum -a 256 "${TMP}/${ASSET}" | awk '{print $1}')"
    fi
        [ "$SUM_ACTUAL" = "$SUM_EXPECTED" ] || { echo "Checksum mismatch"; exit 1; }
fi

# Extract & install
tar -xzf "${TMP}/${ASSET}" -C "$TMP"
chmod +x "${TMP}/${BIN}"

if install -m 0755 "${TMP}/${BIN}" "${INSTALL_DIR}/${BIN}" 2>/dev/null; then
    :
elif command -v sudo >/dev/null 2>&1; then
    sudo install -m 0755 "${TMP}/${BIN}" "${INSTALL_DIR}/${BIN}"
else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "${INSTALL_DIR}"
    install -m 0755 "${TMP}/${BIN}" "${INSTALL_DIR}/${BIN}"
    case ":$PATH:" in *":${INSTALL_DIR}:"*) : ;; *)
        echo "Add ${INSTALL_DIR} to PATH (e.g. export PATH=\$PATH:${INSTALL_DIR})"
    esac
fi

echo "Installed ${BIN} ${VERSION} to ${INSTALL_DIR}/${BIN}"
