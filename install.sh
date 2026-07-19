#!/bin/sh

set -eu

REPOSITORY="Nakanishi123/vibe-doc"
BINARY_NAME="vibe-doc"
INSTALL_DIR="${VIBE_DOC_INSTALL_DIR:-${HOME:-}/.local/bin}"
VERSION="${VIBE_DOC_VERSION:-latest}"

say() {
    printf '%s\n' "$*"
}

fail() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

require_install_dir() {
    if [ -z "$INSTALL_DIR" ] || [ "$INSTALL_DIR" = "/.local/bin" ]; then
        fail 'HOME is not set; set VIBE_DOC_INSTALL_DIR to the installation directory'
    fi
}

detect_archive() {
    os=$(uname -s)
    arch=$(uname -m)

    case "$os:$arch" in
        Linux:x86_64|Linux:amd64)
            platform="linux-x86_64"
            ;;
        Darwin:x86_64|Darwin:amd64)
            platform="macos-x86_64"
            ;;
        Darwin:arm64|Darwin:aarch64)
            platform="macos-aarch64"
            ;;
        *)
            fail "unsupported platform: $os $arch"
            ;;
    esac

    archive="vibe-doc-$platform.tar.gz"
}

release_url() {
    asset=$1

    if [ "$VERSION" = "latest" ]; then
        printf 'https://github.com/%s/releases/latest/download/%s\n' "$REPOSITORY" "$asset"
    else
        case "$VERSION" in
            v*) tag=$VERSION ;;
            *) tag="v$VERSION" ;;
        esac
        printf 'https://github.com/%s/releases/download/%s/%s\n' "$REPOSITORY" "$tag" "$asset"
    fi
}

download() {
    url=$1
    destination=$2

    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 --location --fail --silent --show-error \
            --output "$destination" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only --quiet --output-document="$destination" "$url"
    else
        fail 'curl or wget is required'
    fi
}

sha256() {
    file=$1

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{ print $1 }'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{ print $1 }'
    else
        fail 'sha256sum or shasum is required to verify the download'
    fi
}

verify_archive() {
    expected=$(awk -v name="$archive" '$2 == name || $2 == "*" name { print $1; exit }' "$checksum_file")
    [ -n "$expected" ] || fail "checksum not found for $archive"

    actual=$(sha256 "$archive_file")
    [ "$actual" = "$expected" ] || fail "checksum verification failed for $archive"
}

install_binary() {
    extracted_dir="$temporary_dir/vibe-doc-${platform}"
    tar -xzf "$archive_file" -C "$temporary_dir"
    [ -f "$extracted_dir/$BINARY_NAME" ] || fail "archive does not contain $BINARY_NAME"

    mkdir -p "$INSTALL_DIR"
    install -m 0755 "$extracted_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
}

require_install_dir
detect_archive

temporary_dir=$(mktemp -d 2>/dev/null || mktemp -d -t vibe-doc-install)
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM

archive_file="$temporary_dir/$archive"
checksum_file="$temporary_dir/SHA256SUMS"

say "Downloading vibe-doc ${VERSION} for ${platform}..."
download "$(release_url "$archive")" "$archive_file"
download "$(release_url SHA256SUMS)" "$checksum_file"
verify_archive
install_binary

say "Installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
case ":${PATH:-}:" in
    *:"$INSTALL_DIR":*) ;;
    *) say "Add $INSTALL_DIR to PATH to run $BINARY_NAME from any directory." ;;
esac
