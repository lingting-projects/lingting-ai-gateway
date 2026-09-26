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
    sed -i '25c\framework-core = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }' "$ROOT_DIR/Cargo.toml"
    sed -i '26c\framework-datetime = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }' "$ROOT_DIR/Cargo.toml"
    sed -i '27c\framework-proc-auto = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }' "$ROOT_DIR/Cargo.toml"
    sed -i '28c\framework-proc-core = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }' "$ROOT_DIR/Cargo.toml"
    sed -i '29c\framework-proc-ts = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }' "$ROOT_DIR/Cargo.toml"
    sed -i '30c\framework-web = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2", features = ["collect"] }' "$ROOT_DIR/Cargo.toml"
    sed -i '31c\framework-web-axum = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2", features = ["collect"] }' "$ROOT_DIR/Cargo.toml"
    sed -i '32c\framework-region = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }' "$ROOT_DIR/Cargo.toml"
    cargo update -p framework-core -p framework-datetime -p framework-proc-auto -p framework-proc-core -p framework-proc-ts -p framework-region -p framework-web -p framework-web-axum
}

if [[ "${LINGTING_GATEWAY_CI:-}" == "1" ]]; then
    rsync_framework
fi

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
