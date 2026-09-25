#!/usr/bin/env bash

set -euo pipefail

RELEASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$RELEASE_DIR/.." && pwd)"

INFO_FILE="$RELEASE_DIR/info"
SIG_FILE="$RELEASE_DIR/info.sig"
PUBKEY_FILE="$RELEASE_DIR/pubkey"

FRAMEWORK_DIR="${FRAMEWORK_DIR:-$ROOT_DIR/../lingting-rust-framework}"

FRAMEWORK_REPOSITORY="https://github.com/lingting-projects/lingting-rust-framework.git"
GATEWAY_REPOSITORY="https://github.com/lingting-projects/lingting-ai-gateway.git"

SIGN_KEY="${HOME}/.ssh/lingting_gateway_ed25519"
SIGN_IDENTITY="lingting-release"
SIGN_NAMESPACE="lingting-release"

DIST_DIR="$ROOT_DIR/dist"

fail() {
    echo "error: $*" >&2
    exit 1
}

info() {
    echo
    echo "==> $*"
}

require_command() {
    local command_name="$1"

    if ! command -v "$command_name" >/dev/null 2>&1; then
        fail "required command not found: $command_name"
    fi
}

require_file() {
    local file="$1"

    if [[ ! -f "$file" ]]; then
        fail "file not found: $file"
    fi
}

require_directory() {
    local directory="$1"

    if [[ ! -d "$directory" ]]; then
        fail "directory not found: $directory"
    fi
}

normalize_git_url() {
    local url="$1"

    url="${url%.git}"

    case "$url" in
        git@github.com:*)
            printf '%s\n' "${url#git@github.com:}"
            ;;
        ssh://git@github.com/*)
            printf '%s\n' "${url#ssh://git@github.com/}"
            ;;
        https://github.com/*)
            printf '%s\n' "${url#https://github.com/}"
            ;;
        *)
            printf '%s\n' "$url"
            ;;
    esac
}

require_clean_git_tree() {
    local directory="$1"

    if [[ -n "$(git -C "$directory" status --porcelain)" ]]; then
        fail "git working tree is not clean: $directory"
    fi
}

get_git_origin() {
    local directory="$1"

    git -C "$directory" remote get-url origin
}

get_git_branch() {
    local directory="$1"

    local branch

    branch="$(
        git -C "$directory" branch --show-current
    )"

    if [[ -z "$branch" ]]; then
        fail "git repository is in detached HEAD state: $directory"
    fi

    printf '%s\n' "$branch"
}

get_git_commit() {
    local directory="$1"

    git -C "$directory" rev-parse HEAD
}

get_cargo_version() {
    local cargo_toml="$ROOT_DIR/Cargo.toml"

    require_file "$cargo_toml"

    local version

    version="$(
        sed -nE '
            /^\[workspace\.package\]/,/^\[/ {
                s/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*$/\1/p
            }
        ' "$cargo_toml" |
        head -n 1
    )"

    if [[ -z "$version" ]]; then
        version="$(
            sed -nE '
                /^\[package\]/,/^\[/ {
                    s/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*$/\1/p
                }
            ' "$cargo_toml" |
            head -n 1
        )"
    fi

    if [[ -z "$version" ]]; then
        fail "failed to find version in [workspace.package] or [package]"
    fi

    printf '%s\n' "$version"
}

get_release_tag() {
    local version

    version="$(get_cargo_version)"

    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
        fail "invalid Cargo.toml version: $version"
    fi

    printf 'v%s\n' "$version"
}

read_info_value() {
    local key="$1"

    if [[ ! -f "$INFO_FILE" ]]; then
        fail "release metadata not found: $INFO_FILE"
    fi

    sed -nE "s/^${key}=([^[:space:]].*)$/\1/p" "$INFO_FILE" |
        head -n 1
}

require_sha1_commit() {
    local commit="$1"

    if [[ ! "$commit" =~ ^[0-9a-fA-F]{40}$ ]]; then
        fail "invalid git commit: $commit"
    fi
}

get_remote_branch_commit() {
    local repository="$1"
    local branch="$2"

    git ls-remote \
        "$repository" \
        "refs/heads/$branch" |
        awk 'NR == 1 { print $1 }'
}

checkout_framework() {
    local commit="$1"
    local path="$2"

    require_sha1_commit "$commit"

    rm -rf "$path"

    git clone \
        --no-tags \
        "$FRAMEWORK_REPOSITORY" \
        "$path"

    git -C "$path" fetch \
        --no-tags \
        origin \
        "$commit"

    git -C "$path" checkout \
        --detach \
        "$commit"

    local actual_commit

    actual_commit="$(
        git -C "$path" rev-parse HEAD
    )"

    if [[ "$actual_commit" != "$commit" ]]; then
        fail "framework checkout commit mismatch

expected:
  $commit

actual:
  $actual_commit"
    fi
}

print_environment() {
    echo
    echo "root:"
    echo "  $ROOT_DIR"

    echo
    echo "release:"
    echo "  $RELEASE_DIR"

    echo
    echo "framework:"
    echo "  $FRAMEWORK_DIR"

    echo
    echo "rust:"
    rustc --version

    echo
    echo "cargo:"
    cargo --version
}
