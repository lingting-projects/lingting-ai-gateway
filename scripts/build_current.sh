#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo is required." >&2
    exit 1
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

echo "==> Build system: ${OS} ${ARCH}"

case "${OS}:${ARCH}" in
    # macOS: use the native toolchain.
    Darwin:x86_64|Darwin:arm64)
        echo "==> Building macOS release version..."
        cargo build --release -p bin-server

        echo
        echo "==> Build complete:"
        echo "    target/release/bin-server"
        ;;

    # Linux: build a statically linked musl binary.
    Linux:x86_64)
        TARGET="x86_64-unknown-linux-musl"
        echo "==> Building Linux static version (${TARGET})..."

        if ! rustup target list --installed 2>/dev/null | grep -Fxq "${TARGET}"; then
            echo "==> Installing Rust target: ${TARGET}"
            rustup target add "${TARGET}"
        fi

        cargo build \
            --release \
            -p bin-server \
            --target "${TARGET}"

        echo
        echo "==> Build complete:"
        echo "    target/${TARGET}/release/bin-server"
        ;;

    Linux:aarch64|Linux:arm64)
        TARGET="aarch64-unknown-linux-musl"
        echo "==> Building Linux static version (${TARGET})..."

        if ! rustup target list --installed 2>/dev/null | grep -Fxq "${TARGET}"; then
            echo "==> Installing Rust target: ${TARGET}"
            rustup target add "${TARGET}"
        fi

        cargo build \
            --release \
            -p bin-server \
            --target "${TARGET}"

        echo
        echo "==> Build complete:"
        echo "    target/${TARGET}/release/bin-server"
        ;;

    Linux:armv7l)
        TARGET="armv7-unknown-linux-musleabihf"
        echo "==> Building Linux static version (${TARGET})..."

        if ! rustup target list --installed 2>/dev/null | grep -Fxq "${TARGET}"; then
            echo "==> Installing Rust target: ${TARGET}"
            rustup target add "${TARGET}"
        fi

        cargo build \
            --release \
            -p bin-server \
            --target "${TARGET}"

        echo
        echo "==> Build complete:"
        echo "    target/${TARGET}/release/bin-server"
        ;;

    # Windows: the normal release build is sufficient.
    MINGW*:x86_64|MSYS_NT*:x86_64|CYGWIN*:x86_64)
        echo "==> Building Windows release version..."
        cargo build --release -p bin-server

        echo
        echo "==> Build complete:"
        echo "    target/release/bin-server.exe"
        ;;

    MINGW*:aarch64|MSYS_NT*:aarch64)
        echo "==> Building Windows ARM64 release version..."
        cargo build --release -p bin-server

        echo
        echo "==> Build complete:"
        echo "    target/release/bin-server.exe"
        ;;

    *)
        echo "error: unsupported system: ${OS} ${ARCH}" >&2
        exit 1
        ;;
esac
