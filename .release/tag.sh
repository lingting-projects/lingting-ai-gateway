#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

RELEASE_DIR="$ROOT_DIR/.release"
INFO_FILE="$RELEASE_DIR/info"
SIG_FILE="$RELEASE_DIR/info.sig"
SIGN_SCRIPT="$RELEASE_DIR/sign.sh"
CARGO_TOML="$ROOT_DIR/Cargo.toml"

FRAMEWORK_DIR="${FRAMEWORK_DIR:-$ROOT_DIR/../lingting-rust-framework}"

FRAMEWORK_REPOSITORY="https://github.com/lingting-projects/lingting-rust-framework.git"
GATEWAY_REPOSITORY="https://github.com/lingting-projects/lingting-ai-gateway.git"

SIGN_KEY="${HOME}/.ssh/lingting_gateway_ed25519"

fail() {
    echo "error: $*" >&2
    exit 1
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

if [[ $# -ne 0 ]]; then
    fail "this script does not accept arguments"
fi

if [[ ! -d "$ROOT_DIR/.git" ]]; then
    fail "gateway repository not found: $ROOT_DIR"
fi

if [[ ! -d "$FRAMEWORK_DIR/.git" ]]; then
    fail "framework repository not found: $FRAMEWORK_DIR"
fi

if [[ ! -f "$CARGO_TOML" ]]; then
    fail "Cargo.toml not found: $CARGO_TOML"
fi

if [[ ! -x "$SIGN_SCRIPT" ]]; then
    fail "sign script is not executable: $SIGN_SCRIPT"
fi

if [[ ! -f "$SIGN_KEY" ]]; then
    fail "release private key not found: $SIGN_KEY"
fi

if [[ ! -f "${SIGN_KEY}.pub" ]]; then
    fail "release public key not found: ${SIGN_KEY}.pub"
fi

echo "==> Reading gateway version"

VERSION="$(
    sed -nE '
        /^\[workspace\.package\]/,/^\[/ {
            s/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*$/\1/p
        }
    ' "$CARGO_TOML" |
    head -n 1
)"

if [[ -z "$VERSION" ]]; then
    VERSION="$(
        sed -nE '
            /^\[package\]/,/^\[/ {
                s/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*$/\1/p
            }
        ' "$CARGO_TOML" |
        head -n 1
    )"
fi

if [[ -z "$VERSION" ]]; then
    fail "failed to find version in [workspace.package] or [package]"
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
    fail "invalid Cargo.toml version: $VERSION"
fi

TAG="v${VERSION}"

echo "version:"
echo "  $VERSION"

echo "tag:"
echo "  $TAG"

echo
echo "==> Checking framework repository"

FRAMEWORK_REMOTE="$(
    git -C "$FRAMEWORK_DIR" remote get-url origin
)"

FRAMEWORK_REMOTE_NORMALIZED="$(
    normalize_git_url "$FRAMEWORK_REMOTE"
)"

FRAMEWORK_REPOSITORY_NORMALIZED="$(
    normalize_git_url "$FRAMEWORK_REPOSITORY"
)"

if [[ "$FRAMEWORK_REMOTE_NORMALIZED" != "$FRAMEWORK_REPOSITORY_NORMALIZED" ]]; then
    fail "framework origin mismatch

expected:
  $FRAMEWORK_REPOSITORY

actual:
  $FRAMEWORK_REMOTE"
fi

if [[ -n "$(git -C "$FRAMEWORK_DIR" status --porcelain)" ]]; then
    fail "framework working tree is not clean"
fi

FRAMEWORK_BRANCH="$(
    git -C "$FRAMEWORK_DIR" branch --show-current
)"

if [[ -z "$FRAMEWORK_BRANCH" ]]; then
    fail "framework is in detached HEAD state"
fi

FRAMEWORK_COMMIT="$(
    git -C "$FRAMEWORK_DIR" rev-parse HEAD
)"

if [[ ! "$FRAMEWORK_COMMIT" =~ ^[0-9a-f]{40}$ ]]; then
    fail "invalid framework commit: $FRAMEWORK_COMMIT"
fi

echo "framework repository:"
echo "  $FRAMEWORK_REMOTE"

echo "framework branch:"
echo "  $FRAMEWORK_BRANCH"

echo "framework commit:"
echo "  $FRAMEWORK_COMMIT"

echo
echo "==> Checking gateway repository"

GATEWAY_REMOTE="$(
    git -C "$ROOT_DIR" remote get-url origin
)"

GATEWAY_REMOTE_NORMALIZED="$(
    normalize_git_url "$GATEWAY_REMOTE"
)"

GATEWAY_REPOSITORY_NORMALIZED="$(
    normalize_git_url "$GATEWAY_REPOSITORY"
)"

if [[ "$GATEWAY_REMOTE_NORMALIZED" != "$GATEWAY_REPOSITORY_NORMALIZED" ]]; then
    fail "gateway origin mismatch

expected:
  $GATEWAY_REPOSITORY

actual:
  $GATEWAY_REMOTE"
fi

if [[ -n "$(git -C "$ROOT_DIR" status --porcelain)" ]]; then
    fail "gateway working tree is not clean"
fi

GATEWAY_BRANCH="$(
    git -C "$ROOT_DIR" branch --show-current
)"

if [[ -z "$GATEWAY_BRANCH" ]]; then
    fail "gateway is in detached HEAD state"
fi

GATEWAY_COMMIT="$(
    git -C "$ROOT_DIR" rev-parse HEAD
)"

if [[ ! "$GATEWAY_COMMIT" =~ ^[0-9a-f]{40}$ ]]; then
    fail "invalid gateway commit: $GATEWAY_COMMIT"
fi

echo "gateway repository:"
echo "  $GATEWAY_REMOTE"

echo "gateway branch:"
echo "  $GATEWAY_BRANCH"

echo "gateway commit:"
echo "  $GATEWAY_COMMIT"

echo
echo "==> Checking tag"

if git -C "$ROOT_DIR" rev-parse --verify --quiet "refs/tags/$TAG" >/dev/null; then
    fail "tag already exists locally: $TAG"
fi

if git -C "$ROOT_DIR" ls-remote \
    --exit-code \
    --refs \
    origin \
    "refs/tags/$TAG" >/dev/null 2>&1; then

    fail "tag already exists on origin: $TAG"
fi

echo "tag is available:"
echo "  $TAG"

echo
echo "==> Checking signing key"

ssh-keygen -lf "${SIGN_KEY}.pub" -E sha256

echo
echo "==> All pre-flight checks passed"

echo
echo "==> Pushing framework branch"

git -C "$FRAMEWORK_DIR" push origin "$FRAMEWORK_BRANCH"

echo
echo "==> Verifying framework branch on origin"

REMOTE_FRAMEWORK_COMMIT="$(
    git ls-remote "$FRAMEWORK_REPOSITORY" \
        "refs/heads/$FRAMEWORK_BRANCH" |
        awk 'NR == 1 { print $1 }'
)

if [[ "$REMOTE_FRAMEWORK_COMMIT" != "$FRAMEWORK_COMMIT" ]]; then
    fail "framework branch does not point to expected commit

branch:
  $FRAMEWORK_BRANCH

expected:
  $FRAMEWORK_COMMIT

actual:
  ${REMOTE_FRAMEWORK_COMMIT:-<none>}"
fi

echo "framework commit confirmed on origin:"
echo "  branch: $FRAMEWORK_BRANCH"
echo "  commit: $REMOTE_FRAMEWORK_COMMIT"

echo
echo "==> Generating release metadata"

TIMESTAMP="$(
    date -u '+%Y-%m-%dT%H:%M:%SZ'
)"

TIMESTAMP_UNIX="$(
    date -u '+%s'
)"

cat > "$INFO_FILE" <<EOF
version=1
tag=$TAG
repository=$FRAMEWORK_REPOSITORY
branch=$FRAMEWORK_BRANCH
commit=$FRAMEWORK_COMMIT
timestamp=$TIMESTAMP
timestamp_unix=$TIMESTAMP_UNIX
EOF

echo
echo "==> Signing release metadata"

"$SIGN_SCRIPT" sign

echo
echo "==> Verifying generated signature"

"$SIGN_SCRIPT" verify

echo
echo "==> Checking generated release files"

if [[ ! -s "$INFO_FILE" ]]; then
    fail "generated info is empty"
fi

if [[ ! -s "$SIG_FILE" ]]; then
    fail "generated signature is empty"
fi

INFO_TAG="$(
    sed -n 's/^tag=//p' "$INFO_FILE"
)"

INFO_REPOSITORY="$(
    sed -n 's/^repository=//p' "$INFO_FILE"
)"

INFO_COMMIT="$(
    sed -n 's/^commit=//p' "$INFO_FILE"
)"

if [[ "$INFO_TAG" != "$TAG" ]]; then
    fail "info tag mismatch"
fi

if [[ "$INFO_REPOSITORY" != "$FRAMEWORK_REPOSITORY" ]]; then
    fail "info repository mismatch"
fi

if [[ "$INFO_COMMIT" != "$FRAMEWORK_COMMIT" ]]; then
    fail "info commit mismatch"
fi

echo
echo "==> Preparing gateway release commit"

git -C "$ROOT_DIR" add \
    ".release/info" \
    ".release/info.sig"

git -C "$ROOT_DIR" diff --cached --check

git -C "$ROOT_DIR" commit \
    -m "release: $TAG"

RELEASE_COMMIT="$(
    git -C "$ROOT_DIR" rev-parse HEAD
)"

echo
echo "release commit:"
echo "  $RELEASE_COMMIT"

echo
echo "==> Creating annotated tag"

git -C "$ROOT_DIR" tag \
    -a "$TAG" \
    -m "Release $TAG"

echo
echo "==> Pushing gateway branch"

git -C "$ROOT_DIR" push origin "$GATEWAY_BRANCH"

echo
echo "==> Pushing release tag"

git -C "$ROOT_DIR" push origin "$TAG"

echo
echo "========================================"
echo "Release created successfully"
echo "========================================"
echo

echo "version:"
echo "  $VERSION"

echo
echo "tag:"
echo "  $TAG"

echo
echo "gateway commit:"
echo "  $RELEASE_COMMIT"

echo
echo "framework branch:"
echo "  $FRAMEWORK_BRANCH"

echo
echo "framework commit:"
echo "  $FRAMEWORK_COMMIT"

echo
echo "metadata:"
echo "  $INFO_FILE"

echo
echo "signature:"
echo "  $SIG_FILE"
