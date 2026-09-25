#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<'EOF'
Usage:
  bash .release/ci_build.sh [options]

Options:
  -s    Build slim version
  -l    Build Linux
  -w    Build Windows
  -m    Build macOS

Examples:
  bash .release/ci_build.sh -l
  bash .release/ci_build.sh -sl

  bash .release/ci_build.sh -w
  bash .release/ci_build.sh -sw

  bash .release/ci_build.sh -m
  bash .release/ci_build.sh -sm
EOF

    exit 1
}

SLIM=0
BUILD_LINUX=0
BUILD_WINDOWS=0
BUILD_MACOS=0

while getopts ":slwmh" opt; do
    case "$opt" in
        s)
            SLIM=1
            ;;
        l)
            BUILD_LINUX=1
            ;;
        w)
            BUILD_WINDOWS=1
            ;;
        m)
            BUILD_MACOS=1
            ;;
        h)
            usage
            ;;
        \?)
            fail "invalid option: -$OPTARG"
            ;;
    esac
done

shift $((OPTIND - 1))

if [[ $# -ne 0 ]]; then
    fail "unexpected arguments: $*"
fi

if [[ "$BUILD_LINUX" -eq 0 &&
      "$BUILD_WINDOWS" -eq 0 &&
      "$BUILD_MACOS" -eq 0 ]]; then
    fail "at least one platform is required: -l, -w or -m"
fi

require_command cargo
require_command rustc
require_command rustup

PACKAGE="bin-server"
BINARY_NAME="bin-server"

TARGET_DIR="$ROOT_DIR/target"
DIST_DIR="$ROOT_DIR/dist"

mkdir -p "$DIST_DIR"

if [[ "$SLIM" -eq 1 ]]; then
    BUILD_VARIANT="slim"
else
    BUILD_VARIANT="static"
fi

echo
echo "========================================"
echo "Rust build"
echo "========================================"
echo

echo "package:"
echo "  $PACKAGE"

echo "variant:"
echo "  $BUILD_VARIANT"

echo "rust:"
rustc --version

echo "cargo:"
cargo --version

echo "platforms:"

if [[ "$BUILD_LINUX" -eq 1 ]]; then
    echo "  linux"
fi

if [[ "$BUILD_WINDOWS" -eq 1 ]]; then
    echo "  windows"
fi

if [[ "$BUILD_MACOS" -eq 1 ]]; then
    echo "  macos"
fi

#
# Linux
#

build_linux() {
    local target
    local output

    info "Building Linux"

    if [[ "$SLIM" -eq 1 ]]; then
        target="x86_64-unknown-linux-gnu"
        output="$DIST_DIR/lingting-ai-gateway-linux-slim"

        echo "target:"
        echo "  $target"

        echo "link mode:"
        echo "  dynamic glibc"

    else
        target="x86_64-unknown-linux-musl"
        output="$DIST_DIR/lingting-ai-gateway-linux"

        echo "target:"
        echo "  $target"

        echo "link mode:"
        echo "  static musl"

        if ! command -v musl-gcc >/dev/null 2>&1; then
            info "Installing musl-tools"

            sudo apt-get update
            sudo apt-get install -y musl-tools
        fi
    fi

    rustup target add "$target"

    (
        cd "$ROOT_DIR"

        cargo build \
            --release \
            --locked \
            --package "$PACKAGE" \
            --target "$target"
    )

    cp \
        "$TARGET_DIR/$target/release/$BINARY_NAME" \
        "$output"

    chmod +x "$output"

    echo
    echo "Linux artifact:"
    echo "  $output"

    file "$output" || true
}

#
# Windows
#

build_windows() {
    local target
    local output
    local rustflags

    info "Building Windows"

    target="x86_64-pc-windows-msvc"

    if [[ "$SLIM" -eq 1 ]]; then
        output="$DIST_DIR/lingting-ai-gateway-windows-slim.exe"

        rustflags="${RUSTFLAGS:-} -C target-feature=-crt-static"

        echo "target:"
        echo "  $target"

        echo "CRT:"
        echo "  dynamic MSVC CRT"

    else
        output="$DIST_DIR/lingting-ai-gateway-windows.exe"

        rustflags="${RUSTFLAGS:-} -C target-feature=+crt-static"

        echo "target:"
        echo "  $target"

        echo "CRT:"
        echo "  static MSVC CRT"
    fi

    rustup target add "$target"

    (
        cd "$ROOT_DIR"

        RUSTFLAGS="$rustflags" \
            cargo build \
                --release \
                --locked \
                --package "$PACKAGE" \
                --target "$target"
    )

    cp \
        "$TARGET_DIR/$target/release/$BINARY_NAME.exe" \
        "$output"

    echo
    echo "Windows artifact:"
    echo "  $output"

    file "$output" || true
}

#
# macOS
#

build_macos() {
    local target
    local output

    info "Building macOS"

    target="x86_64-apple-darwin"

    if [[ "$SLIM" -eq 1 ]]; then
        output="$DIST_DIR/lingting-ai-gateway-macos-slim"

        echo "target:"
        echo "  $target"

        echo "link mode:"
        echo "  platform-dependent"

    else
        output="$DIST_DIR/lingting-ai-gateway-macos"

        echo "target:"
        echo "  $target"

        echo "link mode:"
        echo "  platform-dependent"
    fi

    rustup target add "$target"

    (
        cd "$ROOT_DIR"

        cargo build \
            --release \
            --locked \
            --package "$PACKAGE" \
            --target "$target"
    )

    cp \
        "$TARGET_DIR/$target/release/$BINARY_NAME" \
        "$output"

    chmod +x "$output"

    echo
    echo "macOS artifact:"
    echo "  $output"

    file "$output" || true
}

#
# Build
#

if [[ "$BUILD_LINUX" -eq 1 ]]; then
    build_linux
fi

if [[ "$BUILD_WINDOWS" -eq 1 ]]; then
    build_windows
fi

if [[ "$BUILD_MACOS" -eq 1 ]]; then
    build_macos
fi

echo
echo "========================================"
echo "Build completed"
echo "========================================"
echo

find "$DIST_DIR" \
    -maxdepth 1 \
    -type f \
    -print \
    | sort
