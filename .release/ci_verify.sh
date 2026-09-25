#!/usr/bin/env bash
set -euo pipefail

RELEASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$RELEASE_DIR/.." && pwd)"

# shellcheck source=/dev/null
source "$RELEASE_DIR/common.sh"

require_command git
require_command ssh-keygen
require_command awk
require_command sed
require_command grep

require_file "$PUBKEY_FILE"
require_file "$INFO_FILE"
require_file "$SIG_FILE"

if [[ -z "${RELEASE_PUBKEY_FINGERPRINT:-}" ]]; then
    fail "RELEASE_PUBKEY_FINGERPRINT is not set"
fi

info "Checking release public key"

EXPECTED_FINGERPRINT="$RELEASE_PUBKEY_FINGERPRINT"

ACTUAL_FINGERPRINT="$(
    ssh-keygen -lf "$PUBKEY_FILE" -E sha256 |
        awk '{print $2}'
)"

printf 'expected fingerprint:\n  %s\n' "$EXPECTED_FINGERPRINT"
printf 'actual fingerprint:\n  %s\n' "$ACTUAL_FINGERPRINT"

if [[ "$ACTUAL_FINGERPRINT" != "$EXPECTED_FINGERPRINT" ]]; then
    fail "release public key fingerprint mismatch"
fi

if LC_ALL=C grep -q $'\r' "$INFO_FILE"; then
    fail "release metadata contains CRLF line endings: $INFO_FILE"
fi

info "Verifying release metadata signature"

bash "$RELEASE_DIR/sign.sh" verify

read_info() {
    local key="$1"

    read_info_value "$INFO_FILE" "$key"
}

INFO_VERSION="$(read_info version)"
INFO_TAG="$(read_info tag)"
INFO_REPOSITORY="$(read_info repository)"
INFO_BRANCH="$(read_info branch)"
INFO_COMMIT="$(read_info commit)"
INFO_TIMESTAMP="$(read_info timestamp)"
INFO_TIMESTAMP_UNIX="$(read_info timestamp_unix)"

if [[ "$INFO_VERSION" != "1" ]]; then
    fail "unsupported release metadata version: $INFO_VERSION"
fi

if [[ "$INFO_TAG" != "${GITHUB_REF_NAME:-}" ]]; then
    fail "metadata tag mismatch: metadata=$INFO_TAG github=${GITHUB_REF_NAME:-}"
fi

if [[ "$INFO_REPOSITORY" != "$FRAMEWORK_REPOSITORY" ]]; then
    fail "framework repository mismatch: $INFO_REPOSITORY"
fi

if [[ -z "$INFO_BRANCH" ]]; then
    fail "metadata branch is empty"
fi

if [[ ! "$INFO_COMMIT" =~ ^[0-9a-f]{40}$ ]]; then
    fail "metadata commit is not a SHA-1 commit: $INFO_COMMIT"
fi

if [[ -z "$INFO_TIMESTAMP" ]]; then
    fail "metadata timestamp is empty"
fi

if [[ ! "$INFO_TIMESTAMP_UNIX" =~ ^[0-9]+$ ]]; then
    fail "metadata timestamp_unix is invalid: $INFO_TIMESTAMP_UNIX"
fi

info "Checking framework remote branch"

REMOTE_FRAMEWORK_COMMIT=""

REMOTE_FRAMEWORK_COMMIT="$(
    get_remote_branch_commit \
        "$FRAMEWORK_REPOSITORY" \
        "$INFO_BRANCH"
)"

if [[ -z "$REMOTE_FRAMEWORK_COMMIT" ]]; then
    fail "framework branch not found remotely: $INFO_BRANCH"
fi

if [[ "$REMOTE_FRAMEWORK_COMMIT" != "$INFO_COMMIT" ]]; then
    fail "framework commit mismatch: metadata=$INFO_COMMIT remote=$REMOTE_FRAMEWORK_COMMIT"
fi

info "Checking framework commit"

CHECKOUT_DIR="$ROOT_DIR/../lingting-rust-framework"

checkout_framework \
    "$INFO_COMMIT" \
    "$CHECKOUT_DIR"

ACTUAL_FRAMEWORK_COMMIT="$(
    git -C "$CHECKOUT_DIR" rev-parse HEAD
)"

if [[ "$ACTUAL_FRAMEWORK_COMMIT" != "$INFO_COMMIT" ]]; then
    fail "framework checkout commit mismatch: expected=$INFO_COMMIT actual=$ACTUAL_FRAMEWORK_COMMIT"
fi

info "Release verification completed successfully"
