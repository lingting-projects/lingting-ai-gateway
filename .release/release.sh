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
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-core = { path = "../lingting-rust-framework/framework-core" }'" "'framework-core = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-datetime = { path = "../lingting-rust-framework/framework-datetime" }'" "'framework-datetime = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-proc-auto = { path = "../lingting-rust-framework/framework-proc-auto" }'" "'framework-proc-auto = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-proc-core = { path = "../lingting-rust-framework/framework-proc-core" }'" "'framework-proc-core = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-proc-ts = { path = "../lingting-rust-framework/framework-proc-ts" }'" "'framework-proc-ts = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-web = { path = "../lingting-rust-framework/framework-web", features = ["collect"] }'" "'framework-web = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2", features = ["collect"] }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-web-axum = { path = "../lingting-rust-framework/framework-web-axum", features = ["collect"] }'" "'framework-web-axum = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2", features = ["collect"] }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    python3 - "$ROOT_DIR/Cargo.toml" "'framework-region = { path = "../lingting-rust-framework/framework-region" }'" "'framework-region = { git = "https://github.com/lingting-projects/lingting-rust-framework.git", branch = "main", rev = "accfc05b1767cacdeadc0218fa9f69f8f108a3f2" }'" <<'PY'
    import pathlib
    import sys
    cargo_toml = pathlib.Path(sys.argv[1])
    old_line = sys.argv[2]
    new_line = sys.argv[3]
    content = cargo_toml.read_text()
    lines = content.splitlines(keepends=True)
    replaced = False
    for index, line in enumerate(lines):
    if line.rstrip("\r\n") == old_line:
    newline = "\r\n" if line.endswith("\r\n") else "\n" if line.endswith("\n") else ""
    lines[index] = new_line + newline
    replaced = True
    break
    if not replaced:
    raise SystemExit("framework dependency line not found: " + old_line)
    cargo_toml.write_text("".join(lines))
    PY
    cargo update -p framework-core -p framework-datetime -p framework-proc-auto -p framework-proc-core -p framework-proc-ts -p framework-region -p framework-web -p framework-web-axum
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
