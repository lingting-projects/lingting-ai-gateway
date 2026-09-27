#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO="$ROOT/Cargo.toml"
FRAMEWORK_CARGO="$ROOT/../lingting-rust-framework/Cargo.toml"

# framework-* 包名
packages() {
    sed -nE 's/^[[:space:]]*(framework-[A-Za-z0-9_-]+)[[:space:]]*=.*/\1/p' "$CARGO" | sort -u
}

# framework 项目第一个 version
version() {
    sed -nE 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$FRAMEWORK_CARGO" | head -1
}

# 当前模式：path -> local，否则 version
mode() {
    grep -qE '^[[:space:]]*framework-[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*\{[^}]*path[[:space:]]*=' "$CARGO" \
        && echo local || echo version
}

# 替换 dependency 中的 path/version
switch() {
    local target="$1"
    local ver="${2:-}"

    if [[ "$target" == local ]]; then
        perl -0pi -e '
            s{
                ^([[:space:]]*)(framework-[\w-]+)(\s*=\s*\{\s*)
                version\s*=\s*"[^"]+"
            }{
                $1.$2.$3.qq{path = "../lingting-rust-framework/$2"}
            }gexm
        ' "$CARGO"
    else
        FRAMEWORK_VERSION="$ver" perl -0pi -e '
            s{
                ^([[:space:]]*)(framework-[\w-]+)(\s*=\s*\{\s*)
                path\s*=\s*"\.\./lingting-rust-framework/framework-[^"]+"
            }{
                $1.$2.$3.qq{version = "$ENV{FRAMEWORK_VERSION}"}
            }gexm
        ' "$CARGO"
    fi
}

# 更新 lock
check() {
    local args=()
    while read -r pkg; do
        [[ -n "$pkg" ]] && args+=(-p "$pkg")
    done < <(packages)

    (cd "$ROOT" && cargo check "${args[@]}")
}

main() {
    local target="${1:-}"
    local current

    case "$target" in
        local|version) ;;
        "") target="$(mode)"; [[ "$target" == local ]] && target=version || target=local ;;
        -h|--help)
            echo "Usage: $0 [local|version]"
            exit 0
            ;;
        *)
            echo "Usage: $0 [local|version]" >&2
            exit 1
            ;;
    esac

    echo "==> Switch framework: $(mode) -> $target"

    if [[ "$target" == version ]]; then
        local ver
        ver="$(version)"
        [[ -n "$ver" ]] || { echo "Error: version not found" >&2; exit 1; }
        echo "==> Version: $ver"
        switch version "$ver"
    else
        switch local
    fi

    check
}

main "$@"
