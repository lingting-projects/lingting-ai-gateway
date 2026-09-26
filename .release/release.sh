#!/usr/bin/env bash
set -euo pipefail

RELEASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$RELEASE_DIR/.." && pwd)"

cd "$ROOT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo is required" >&2
    exit 1
fi

# === start
# 这里用于 release 的 Rust 代码进行替换
# === end
rsync_framework() {
    return 0
}

rsync_framework

cargo build \
    --locked \
    --release \
    --package bin-release

if [[ -x "$ROOT_DIR/target/release/bin-release" ]]; then
    BIN_RELEASE="$ROOT_DIR/target/release/bin-release"
elif [[ -x "$ROOT_DIR/target/release/bin-release.exe" ]]; then
    BIN_RELEASE="$ROOT_DIR/target/release/bin-release.exe"
else
    echo "error: release tool was not built" >&2
    exit 1
fi

exec "$BIN_RELEASE" "$@"