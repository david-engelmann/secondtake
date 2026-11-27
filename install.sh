#!/bin/sh
# Install a prebuilt secondtake release binary with SHA-256 verification.
set -eu

REPO="david-engelmann/secondtake"
VERSION="${SECONDTAKE_VERSION:-}"
BIN_DIR="${SECONDTAKE_INSTALL_DIR:-$HOME/.local/bin}"

usage() {
    cat <<'EOF'
usage: install.sh [--version vX.Y.Z] [--bin-dir DIR]

Options:
  --version TAG   Release tag to install (defaults to latest GitHub release)
  --bin-dir DIR   Destination directory for the binary (default: $HOME/.local/bin)
  -h, --help      Show this help message and exit
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            [ "$#" -ge 2 ] || { echo "error: --version requires a value" >&2; exit 2; }
            VERSION="$2"
            shift 2
            ;;
        --bin-dir)
            [ "$#" -ge 2 ] || { echo "error: --bin-dir requires a value" >&2; exit 2; }
            BIN_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64) ARCH_TRIPLE="x86_64" ;;
    arm64|aarch64) ARCH_TRIPLE="aarch64" ;;
    *)
        echo "error: unsupported architecture: $ARCH" >&2
        exit 1
        ;;
esac

case "$OS" in
    Linux) TARGET="${ARCH_TRIPLE}-unknown-linux-gnu" ;;
    Darwin) TARGET="${ARCH_TRIPLE}-apple-darwin" ;;
    *)
        echo "error: unsupported operating system: $OS (use cargo install --git https://github.com/${REPO}.git on Windows)" >&2
        exit 1
        ;;
esac

fetch() {
    url="$1"
    dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$dest" "$url"
    else
        echo "error: curl or wget is required" >&2
        exit 1
    fi
}

if [ -z "$VERSION" ]; then
    TMP_META="$(mktemp)"
    fetch "https://api.github.com/repos/${REPO}/releases/latest" "$TMP_META"
    VERSION="$(sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' "$TMP_META" | head -n 1)"
    rm -f "$TMP_META"
    if [ -z "$VERSION" ]; then
        echo "error: could not determine latest release tag from GitHub" >&2
        exit 1
    fi
fi

STEM="secondtake-${VERSION}-${TARGET}"
ARCHIVE="${STEM}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT INT TERM

fetch "${BASE_URL}/${ARCHIVE}" "${WORK_DIR}/${ARCHIVE}"
fetch "${BASE_URL}/SHA256SUMS" "${WORK_DIR}/SHA256SUMS"

EXPECTED="$(awk -v file="$ARCHIVE" '$2 == file || $2 == "*" file { print $1; exit }' "${WORK_DIR}/SHA256SUMS")"
if [ -z "$EXPECTED" ]; then
    echo "error: ${ARCHIVE} not found in SHA256SUMS" >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL="$(sha256sum "${WORK_DIR}/${ARCHIVE}" | awk '{ print $1 }')"
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "${WORK_DIR}/${ARCHIVE}" | awk '{ print $1 }')"
else
    echo "error: sha256sum or shasum is required to verify release checksums" >&2
    exit 1
fi

if [ "$EXPECTED" != "$ACTUAL" ]; then
    echo "error: SHA-256 mismatch for ${ARCHIVE}: expected ${EXPECTED}, got ${ACTUAL}" >&2
    exit 1
fi

tar -xzf "${WORK_DIR}/${ARCHIVE}" -C "$WORK_DIR"
mkdir -p "$BIN_DIR"
cp "${WORK_DIR}/${STEM}/secondtake" "${BIN_DIR}/secondtake"
chmod 0755 "${BIN_DIR}/secondtake"

echo "Installed secondtake (${VERSION}) to ${BIN_DIR}/secondtake"
case ":${PATH}:" in
    *":${BIN_DIR}:"*) ;;
    *)
        echo "Note: add ${BIN_DIR} to your PATH to invoke secondtake directly."
        ;;
esac
