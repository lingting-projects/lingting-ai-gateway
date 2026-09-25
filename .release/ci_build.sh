#!/usr/bin/env bash
set -euo pipefail

RELEASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$RELEASE_DIR/.." && pwd)"

# shellcheck source=/dev/null
source "$RELEASE_DIR/common.sh"

SLIM=0
BUILD_LINUX=0
BUILD_WINDOWS=0
BUILD_MACOS=0

usage() {
    cat <<'EOF'
Usage:
  ci_build.sh -l       Build Linux static
  ci_build.sh -sl      Build Linux slim
  ci_build.sh -w       Build Windows static
  ci_build.sh -sw      Build Windows slim
  ci_build.sh -m       Build macOS
  ci_build.sh -sm      Build macOS slim
EOF
}

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
            exit 0
            ;;
        \?)
            fail "invalid option: -$OPTARG"
            ;;
    esac
done

if [[ "$OPTIND" -le "$#" ]]; then
    fail "unexpected argument: ${!OPTIND}"
fi

BUILD_COUNT=$((BUILD_LINUX + BUILD_WINDOWS + BUILD_MACOS))

if [[ "$BUILD_COUNT" -ne 1 ]]; then
    fail "exactly one platform must be specified: -l, -w, or -m"
fi

require_command cargo
require_command rustc
require_command rustup
require_command node
require_command pnpm

UI_DIR="$ROOT_DIR/lingting-ai-gateway-ui"

require_directory "$UI_DIR"
require_file "$UI_DIR/package.json"
require_file "$UI_DIR/pnpm-lock.yaml"

mkdir -p "$DIST_DIR"

info "Build environment"
info "  cargo: $(cargo --version)"
info "  rustc: $(rustc --version)"
info "  rustup: $(rustup --version | head -n 1)"
info "  node: $(node --version)"
info "  pnpm: $(pnpm --version)"

info "Installing frontend dependencies"

(
    cd "$UI_DIR"
    pnpm install --frozen-lockfile
)

build_cargo() {
    local target="$1"
    local output="$2"

    info "Building target: $target"

    rustup target add "$target"

    cargo build \
        --release \
        --locked \
        --package bin-server \
        --target "$target"

    local binary="$ROOT_DIR/target/$target/release/bin-server"

    if [[ ! -f "$binary" ]]; then
        fail "build output not found: $binary"
    fi

    cp "$binary" "$DIST_DIR/$output"

    chmod +x "$DIST_DIR/$output"

    info "Created: $DIST_DIR/$output"
}

build_cargo_windows() {
    local output="$1"
    local crt_static="$2"

    info "Building Windows target with CRT static=$crt_static"

    rustup target add x86_64-pc-windows-msvc

    local existing_rustflags="${RUSTFLAGS:-}"

    if [[ "$crt_static" == "true" ]]; then
        export RUSTFLAGS="${existing_rustflags:+$existing_rustflags }-C target-feature=+crt-static"
    else
        export RUSTFLAGS="${existing_rustflags:+$existing_rustflags }-C target-feature=-crt-static"
    fi

    cargo build \
        --release \
        --locked \
        --package bin-server \
        --target x86_64-pc-windows-msvc

    local binary="$ROOT_DIR/target/x86_64-pc-windows-msvc/release/bin-server.exe"

    if [[ ! -f "$binary" ]]; then
        fail "build output not found: $binary"
    fi

    cp "$binary" "$DIST_DIR/$output"

    info "Created: $DIST_DIR/$output"
}

if [[ "$BUILD_LINUX" -eq 1 ]]; then
    if [[ "$SLIM" -eq 1 ]]; then
        build_cargo \
            "x86_64-unknown-linux-gnu" \
            "lingting-ai-gateway-linux-slim"
    else
        if ! command -v musl-gcc >/dev/null 2>&1; then
            info "musl-gcc not found; installing musl-tools"

            sudo apt-get update
            sudo apt-get install -y musl-tools
        fi

        build_cargo \
            "x86_64-unknown-linux-musl" \
            "lingting-ai-gateway-linux"
    fi
fi

if [[ "$BUILD_WINDOWS" -eq 1 ]]; then
    if [[ "$SLIM" -eq 1 ]]; then
        build_cargo_windows \
            "lingting-ai-gateway-windows-slim.exe" \
            "false"
    else
        build_cargo_windows \
            "lingting-ai-gateway-windows.exe" \
            "true"
    fi
fi

if [[ "$BUILD_MACOS" -eq 1 ]]; then
    if [[ "$SLIM" -eq 1 ]]; then
        build_cargo \
            "x86_64-apple-darwin" \
            "lingting-ai-gateway-macos-slim"
    else
        build_cargo \
            "x86_64-apple-darwin" \
            "lingting-ai-gateway-macos"
    fi
fi

info "Build completed successfully"
