#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"

usage() {
    cat <<EOF
Usage:
  bash .release/ci_build.sh [options]

Options:
  -s    Build slim/dynamic variant
  -l    Build Linux
  -w    Build Windows
  -m    Build macOS
  -h    Show this help

Examples:
  bash .release/ci_build.sh -l
  bash .release/ci_build.sh -sl
  bash .release/ci_build.sh -w
  bash .release/ci_build.sh -sw
  bash .release/ci_build.sh -m
  bash .release/ci_build.sh -sm
EOF
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
            exit 0
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

TARGET_COUNT=$((BUILD_LINUX + BUILD_WINDOWS + BUILD_MACOS))

if [[ "$TARGET_COUNT" -ne 1 ]]; then
    fail "exactly one target must be selected: -l, -w, or -m"
fi

require_command cargo
require_command rustc
require_command rustup
require_command git
require_command node
require_command pnpm
require_command readlink

require_file "$INFO_FILE"

UI_DIR="$ROOT_DIR/lingting-ai-gateway-ui"
LRI_DIR="$UI_DIR/lri"

require_directory "$UI_DIR"
require_file "$UI_DIR/package.json"
require_file "$UI_DIR/pnpm-lock.yaml"
require_directory "$LRI_DIR"

mkdir -p "$DIST_DIR"

echo
info "Build environment"

echo "  cargo: $(cargo --version)"
echo "  rustc: $(rustc --version)"
echo "  rustup: $(rustup --version | head -n 1)"
echo "  node: $(node --version)"
echo "  pnpm: $(pnpm --version)"

FRAMEWORK_COMMIT="$(read_info_value framework_commit)"
REACT_UI_REPOSITORY="$(read_info_value react_ui_repository)"
REACT_UI_BRANCH="$(read_info_value react_ui_branch)"
REACT_UI_COMMIT="$(read_info_value react_ui_commit)"

require_sha1_commit "$FRAMEWORK_COMMIT"
require_sha1_commit "$REACT_UI_COMMIT"

if [[ -z "$REACT_UI_BRANCH" ]]; then
    fail "react-ui branch is empty in release metadata"
fi

EXPECTED_REACT_UI_REPOSITORY="https://github.com/lingting/lingting-react-ui.git"

if [[ "$(normalize_git_url "$REACT_UI_REPOSITORY")" != \
      "$(normalize_git_url "$EXPECTED_REACT_UI_REPOSITORY")" ]]; then
    fail "unexpected react-ui repository

expected:
  $EXPECTED_REACT_UI_REPOSITORY

actual:
  $REACT_UI_REPOSITORY"
fi

checkout_react_ui() {
    local lri_dir="$1"
    local expected_commit="$2"

    local lri_real_dir
    local react_ui_dir
    local actual_commit

    lri_real_dir="$(readlink -f "$lri_dir")"

    if [[ -z "$lri_real_dir" ]]; then
        fail "failed to resolve lri directory: $lri_dir"
    fi

    if [[ ! -d "$lri_real_dir" ]]; then
        fail "resolved lri path is not a directory:

lri:
  $lri_dir

resolved:
  $lri_real_dir"
    fi

    react_ui_dir="$(dirname "$lri_real_dir")"

    if [[ ! -d "$react_ui_dir/.git" ]]; then
        fail "lingting-react-ui repository was not found at the parent directory of lri:

lri:
  $lri_dir

resolved lri:
  $lri_real_dir

expected repository:
  $react_ui_dir"
    fi

    local remote
    remote="$(get_git_origin "$react_ui_dir")"

    if [[ "$(normalize_git_url "$remote")" != \
          "$(normalize_git_url "$EXPECTED_REACT_UI_REPOSITORY")" ]]; then
        fail "lingting-react-ui origin mismatch

expected:
  $EXPECTED_REACT_UI_REPOSITORY

actual:
  $remote"
    fi

    info "Checking lingting-react-ui checkout"

    git -C "$react_ui_dir" fetch \
        --no-tags \
        origin \
        "$expected_commit"

    git -C "$react_ui_dir" checkout \
        --detach \
        "$expected_commit"

    actual_commit="$(git -C "$react_ui_dir" rev-parse HEAD)"

    if [[ "$actual_commit" != "$expected_commit" ]]; then
        fail "lingting-react-ui checkout commit mismatch

expected:
  $expected_commit

actual:
  $actual_commit"
    fi

    echo "lingting-react-ui:"
    echo "  directory: $react_ui_dir"
    echo "  commit: $actual_commit"
}

echo
info "Checking release metadata"

if [[ -z "$FRAMEWORK_COMMIT" ]]; then
    fail "framework commit is missing from release metadata"
fi

if [[ -z "$REACT_UI_COMMIT" ]]; then
    fail "react-ui commit is missing from release metadata"
fi

echo "framework commit:"
echo "  $FRAMEWORK_COMMIT"

echo "react-ui branch:"
echo "  $REACT_UI_BRANCH"

echo "react-ui commit:"
echo "  $REACT_UI_COMMIT"

echo
info "Checking framework checkout"

checkout_framework \
    "$FRAMEWORK_COMMIT" \
    "$FRAMEWORK_DIR"

echo
info "Checking React UI checkout"

checkout_react_ui \
    "$LRI_DIR" \
    "$REACT_UI_COMMIT"

echo
info "Installing frontend dependencies"

(
    cd "$UI_DIR"

    pnpm install --frozen-lockfile
)

build_ts_sdk() {
    echo
    info "Exporting TypeScript SDK"

    (
        cd "$ROOT_DIR"

        cargo run \
            --locked \
            --package lib-web \
            --example build_ts \
            --features ts-export
    )
}

build_linux() {
    local target
    local output

    if [[ "$SLIM" -eq 1 ]]; then
        target="x86_64-unknown-linux-gnu"
        output="$DIST_DIR/lingting-ai-gateway-linux-slim"

        echo
        info "Building Linux slim"

        cargo build \
            --release \
            --locked \
            --package bin-server \
            --target "$target"
    else
        target="x86_64-unknown-linux-musl"
        output="$DIST_DIR/lingting-ai-gateway-linux"

        if ! command -v musl-gcc >/dev/null 2>&1; then
            info "musl-gcc not found; installing musl-tools"

            sudo apt-get update
            sudo apt-get install -y musl-tools
        fi

        echo
        info "Building Linux static"

        cargo build \
            --release \
            --locked \
            --package bin-server \
            --target "$target"
    fi

    cp \
        "$TARGET_DIR/$target/release/bin-server" \
        "$output"

    chmod +x "$output"

    echo
    echo "artifact:"
    echo "  $output"
}

build_windows() {
    local target="x86_64-pc-windows-msvc"
    local output
    local existing_rustflags="${RUSTFLAGS:-}"

    if [[ "$SLIM" -eq 1 ]]; then
        output="$DIST_DIR/lingting-ai-gateway-windows-slim.exe"

        export RUSTFLAGS="${existing_rustflags:+$existing_rustflags }-C target-feature=-crt-static"

        echo
        info "Building Windows slim"
    else
        output="$DIST_DIR/lingting-ai-gateway-windows.exe"

        export RUSTFLAGS="${existing_rustflags:+$existing_rustflags }-C target-feature=+crt-static"

        echo
        info "Building Windows static CRT"
    fi

    cargo build \
        --release \
        --locked \
        --package bin-server \
        --target "$target"

    cp \
        "$TARGET_DIR/$target/release/bin-server.exe" \
        "$output"

    echo
    echo "artifact:"
    echo "  $output"
}

build_macos() {
    local target="x86_64-apple-darwin"
    local output

    if [[ "$SLIM" -eq 1 ]]; then
        output="$DIST_DIR/lingting-ai-gateway-macos-slim"

        echo
        info "Building macOS slim"
    else
        output="$DIST_DIR/lingting-ai-gateway-macos"

        echo
        info "Building macOS"
    fi

    cargo build \
        --release \
        --locked \
        --package bin-server \
        --target "$target"

    cp \
        "$TARGET_DIR/$target/release/bin-server" \
        "$output"

    chmod +x "$output"

    echo
    echo "artifact:"
    echo "  $output"
}

build_ts_sdk

if [[ "$BUILD_LINUX" -eq 1 ]]; then
    build_linux
elif [[ "$BUILD_WINDOWS" -eq 1 ]]; then
    build_windows
elif [[ "$BUILD_MACOS" -eq 1 ]]; then
    build_macos
fi

echo
info "Build completed"