#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"

if [[ $# -ne 0 ]]; then
    fail "this script does not accept arguments"
fi

require_command git
require_command ssh-keygen

require_directory "$ROOT_DIR"
require_directory "$FRAMEWORK_DIR"

require_file "$ROOT_DIR/Cargo.toml"

if [[ ! -x "$SCRIPT_DIR/sign.sh" ]]; then
    fail "sign script is not executable: $SCRIPT_DIR/sign.sh"
fi

if [[ ! -f "$SIGN_KEY" ]]; then
    fail "release private key not found: $SIGN_KEY"
fi

if [[ ! -f "${SIGN_KEY}.pub" ]]; then
    fail "release public key not found: ${SIGN_KEY}.pub"
fi

VERSION="$(get_cargo_version)"
TAG="v${VERSION}"

echo "version:"
echo "  $VERSION"

echo
echo "tag:"
echo "  $TAG"

echo
info "Checking framework repository"

FRAMEWORK_REMOTE="$(get_git_origin "$FRAMEWORK_DIR")"

if [[ "$(normalize_git_url "$FRAMEWORK_REMOTE")" != \
      "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" ]]; then
    fail "framework origin mismatch

expected:
  $FRAMEWORK_REPOSITORY

actual:
  $FRAMEWORK_REMOTE"
fi

require_clean_git_tree "$FRAMEWORK_DIR"

FRAMEWORK_BRANCH="$(get_git_branch "$FRAMEWORK_DIR")"
FRAMEWORK_COMMIT="$(get_git_commit "$FRAMEWORK_DIR")"

require_sha1_commit "$FRAMEWORK_COMMIT"

echo "framework repository:"
echo "  $FRAMEWORK_REMOTE"

echo "framework branch:"
echo "  $FRAMEWORK_BRANCH"

echo "framework commit:"
echo "  $FRAMEWORK_COMMIT"

info "Checking Cargo.lock"

require_command cargo

cargo metadata \
    --locked \
    --format-version 1 \
    >/dev/null

echo
info "Checking gateway repository"

GATEWAY_REMOTE="$(get_git_origin "$ROOT_DIR")"

if [[ "$(normalize_git_url "$GATEWAY_REMOTE")" != \
      "$(normalize_git_url "$GATEWAY_REPOSITORY")" ]]; then
    fail "gateway origin mismatch

expected:
  $GATEWAY_REPOSITORY

actual:
  $GATEWAY_REMOTE"
fi

require_clean_git_tree "$ROOT_DIR"

GATEWAY_BRANCH="$(get_git_branch "$ROOT_DIR")"
GATEWAY_COMMIT="$(get_git_commit "$ROOT_DIR")"

require_sha1_commit "$GATEWAY_COMMIT"

echo "gateway repository:"
echo "  $GATEWAY_REMOTE"

echo "gateway branch:"
echo "  $GATEWAY_BRANCH"

echo "gateway commit:"
echo "  $GATEWAY_COMMIT"

echo
info "Checking tag"

if git -C "$ROOT_DIR" rev-parse \
    --verify \
    --quiet \
    "refs/tags/$TAG" >/dev/null; then
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
info "Checking signing key"

ssh-keygen -lf "${SIGN_KEY}.pub" -E sha256

echo
info "All pre-flight checks passed"

echo
info "Pushing framework branch"

git -C "$FRAMEWORK_DIR" push origin "$FRAMEWORK_BRANCH"

echo
info "Verifying framework branch on origin"

REMOTE_FRAMEWORK_COMMIT=""

REMOTE_FRAMEWORK_COMMIT="$(
    get_remote_branch_commit \
        "$FRAMEWORK_REPOSITORY" \
        "$FRAMEWORK_BRANCH"
)"

if [[ -z "$REMOTE_FRAMEWORK_COMMIT" ]]; then
    fail "framework branch was not found on origin

branch:
  $FRAMEWORK_BRANCH"
fi

if [[ "$REMOTE_FRAMEWORK_COMMIT" != "$FRAMEWORK_COMMIT" ]]; then
    fail "framework branch does not point to expected commit

branch:
  $FRAMEWORK_BRANCH

expected:
  $FRAMEWORK_COMMIT

actual:
  $REMOTE_FRAMEWORK_COMMIT"
fi

echo "framework commit confirmed on origin:"
echo "  branch: $FRAMEWORK_BRANCH"
echo "  commit: $REMOTE_FRAMEWORK_COMMIT"

echo
info "Generating release metadata"

TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
TIMESTAMP_UNIX="$(date -u '+%s')"

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
info "Signing release metadata"

bash "$SCRIPT_DIR/sign.sh" sign

echo
info "Verifying generated signature"

bash "$SCRIPT_DIR/sign.sh" verify

echo
info "Checking generated release files"

require_file "$INFO_FILE"
require_file "$SIG_FILE"

if [[ ! -s "$INFO_FILE" ]]; then
    fail "generated info is empty"
fi

if [[ ! -s "$SIG_FILE" ]]; then
    fail "generated signature is empty"
fi

if [[ "$(read_info_value tag)" != "$TAG" ]]; then
    fail "info tag mismatch"
fi

if [[ "$(read_info_value repository)" != "$FRAMEWORK_REPOSITORY" ]]; then
    fail "info repository mismatch"
fi

if [[ "$(read_info_value commit)" != "$FRAMEWORK_COMMIT" ]]; then
    fail "info commit mismatch"
fi

echo
info "Preparing gateway release commit"

git -C "$ROOT_DIR" add \
    ".release/info" \
    ".release/info.sig"

git -C "$ROOT_DIR" diff --cached --check

git -C "$ROOT_DIR" commit \
    -m "release: $TAG"

RELEASE_COMMIT="$(get_git_commit "$ROOT_DIR")"

echo
echo "release commit:"
echo "  $RELEASE_COMMIT"

echo
info "Creating annotated tag"

git -C "$ROOT_DIR" tag \
    -a "$TAG" \
    -m "Release $TAG"

echo
info "Pushing gateway branch"

git -C "$ROOT_DIR" push origin "$GATEWAY_BRANCH"

echo
info "Pushing release tag"

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
