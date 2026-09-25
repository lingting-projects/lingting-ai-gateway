#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"

require_command git
require_command ssh-keygen
require_command awk
require_command sed
require_command grep

require_file "$INFO_FILE"
require_file "$SIG_FILE"
require_file "$PUBKEY_FILE"

echo
info "Checking release metadata"

if [[ ! -s "$INFO_FILE" ]]; then
    fail "release metadata is empty: $INFO_FILE"
fi

if [[ ! -s "$SIG_FILE" ]]; then
    fail "release metadata signature is empty: $SIG_FILE"
fi

if [[ ! -s "$PUBKEY_FILE" ]]; then
    fail "release public key is empty: $PUBKEY_FILE"
fi

if LC_ALL=C grep -q $'\r' "$INFO_FILE"; then
    fail "release metadata contains CRLF line endings: $INFO_FILE"
fi

echo "metadata:"
cat "$INFO_FILE"

echo
info "Checking release public key fingerprint"

ACTUAL_FINGERPRINT="$(
    ssh-keygen \
        -lf "$PUBKEY_FILE" \
        -E sha256 |
        awk '{ print $2 }'
)"

if [[ "$ACTUAL_FINGERPRINT" != "${RELEASE_PUBKEY_FINGERPRINT:-}" ]]; then
    fail "release public key fingerprint mismatch

expected:
  ${RELEASE_PUBKEY_FINGERPRINT:-<not configured>}

actual:
  $ACTUAL_FINGERPRINT"
fi

echo "release public key fingerprint:"
echo "  $ACTUAL_FINGERPRINT"

echo
info "Verifying release metadata signature"

if ! ssh-keygen \
    -Y verify \
    -f "$PUBKEY_FILE" \
    -I "$SIGN_IDENTITY" \
    -n "$SIGN_NAMESPACE" \
    -s "$SIG_FILE" \
    < "$INFO_FILE"; then
    fail "release metadata signature verification failed"
fi

echo "release metadata signature verified"

echo
info "Checking metadata fields"

INFO_VERSION="$(read_info_value version)"
INFO_TAG="$(read_info_value tag)"

FRAMEWORK_REPOSITORY="$(read_info_value framework_repository)"
FRAMEWORK_BRANCH="$(read_info_value framework_branch)"
FRAMEWORK_COMMIT="$(read_info_value framework_commit)"

REACT_UI_REPOSITORY="$(read_info_value react_ui_repository)"
REACT_UI_BRANCH="$(read_info_value react_ui_branch)"
REACT_UI_COMMIT="$(read_info_value react_ui_commit)"

INFO_TIMESTAMP="$(read_info_value timestamp)"
INFO_TIMESTAMP_UNIX="$(read_info_value timestamp_unix)"

if [[ "$INFO_VERSION" != "1" ]]; then
    fail "unsupported release metadata version: $INFO_VERSION"
fi

if [[ -z "$INFO_TAG" ]]; then
    fail "release metadata tag is empty"
fi

if [[ "$INFO_TAG" != "${GITHUB_REF_NAME:-}" ]]; then
    fail "release metadata tag mismatch

expected:
  ${GITHUB_REF_NAME:-<empty>}

actual:
  $INFO_TAG"
fi

if [[ "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" != \
      "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" ]]; then
    fail "framework repository metadata is invalid"
fi

if [[ "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" != \
      "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" ]]; then
    fail "framework repository metadata is invalid"
fi

if [[ -z "$FRAMEWORK_BRANCH" ]]; then
    fail "framework branch is empty"
fi

require_sha1_commit "$FRAMEWORK_COMMIT"

if [[ "$(normalize_git_url "$REACT_UI_REPOSITORY")" != \
      "$(normalize_git_url "https://github.com/lingting/lingting-react-ui.git")" ]]; then
    fail "react-ui repository mismatch

expected:
  https://github.com/lingting/lingting-react-ui.git

actual:
  $REACT_UI_REPOSITORY"
fi

if [[ -z "$REACT_UI_BRANCH" ]]; then
    fail "react-ui branch is empty"
fi

require_sha1_commit "$REACT_UI_COMMIT"

if [[ -z "$INFO_TIMESTAMP" ]]; then
    fail "release metadata timestamp is empty"
fi

if [[ ! "$INFO_TIMESTAMP_UNIX" =~ ^[0-9]+$ ]]; then
    fail "release metadata timestamp_unix is invalid: $INFO_TIMESTAMP_UNIX"
fi

if [[ -z "$FRAMEWORK_REPOSITORY" ]]; then
    fail "framework repository is empty"
fi

if [[ "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" != \
      "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" ]]; then
    fail "framework repository metadata is invalid"
fi

echo "metadata version:"
echo "  $INFO_VERSION"

echo "tag:"
echo "  $INFO_TAG"

echo "framework repository:"
echo "  $FRAMEWORK_REPOSITORY"

echo "framework branch:"
echo "  $FRAMEWORK_BRANCH"

echo "framework commit:"
echo "  $FRAMEWORK_COMMIT"

echo "react-ui repository:"
echo "  $REACT_UI_REPOSITORY"

echo "react-ui branch:"
echo "  $REACT_UI_BRANCH"

echo "react-ui commit:"
echo "  $REACT_UI_COMMIT"

echo "timestamp:"
echo "  $INFO_TIMESTAMP"

echo "timestamp_unix:"
echo "  $INFO_TIMESTAMP_UNIX"

echo
info "Checking framework origin"

if [[ "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" != \
      "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" ]]; then
    fail "framework repository mismatch"
fi

if [[ "$(normalize_git_url "$FRAMEWORK_REPOSITORY")" != \
      "$(normalize_git_url "https://github.com/lingting-projects/lingting-rust-framework.git")" ]]; then
    fail "unexpected framework repository

expected:
  https://github.com/lingting-projects/lingting-rust-framework.git

actual:
  $FRAMEWORK_REPOSITORY"
fi

echo "framework repository confirmed"

echo
info "Checking framework branch on origin"

REMOTE_FRAMEWORK_COMMIT="$(
    get_remote_branch_commit \
        "$FRAMEWORK_REPOSITORY" \
        "$FRAMEWORK_BRANCH"
)"

if [[ -z "$REMOTE_FRAMEWORK_COMMIT" ]]; then
    fail "framework branch was not found on origin

repository:
  $FRAMEWORK_REPOSITORY

branch:
  $FRAMEWORK_BRANCH"
fi

if [[ "$REMOTE_FRAMEWORK_COMMIT" != "$FRAMEWORK_COMMIT" ]]; then
    fail "framework branch does not point to expected commit

repository:
  $FRAMEWORK_REPOSITORY

branch:
  $FRAMEWORK_BRANCH

expected:
  $FRAMEWORK_COMMIT

actual:
  $REMOTE_FRAMEWORK_COMMIT"
fi

echo "framework origin confirmed:"
echo "  branch: $FRAMEWORK_BRANCH"
echo "  commit: $REMOTE_FRAMEWORK_COMMIT"

echo
info "Checking lingting-react-ui repository"

if [[ "$(normalize_git_url "$REACT_UI_REPOSITORY")" != \
      "$(normalize_git_url "https://github.com/lingting/lingting-react-ui.git")" ]]; then
    fail "unexpected react-ui repository

expected:
  https://github.com/lingting/lingting-react-ui.git

actual:
  $REACT_UI_REPOSITORY"
fi

echo "react-ui repository confirmed"

echo
info "Checking lingting-react-ui branch on origin"

REMOTE_REACT_UI_COMMIT="$(
    get_remote_branch_commit \
        "$REACT_UI_REPOSITORY" \
        "$REACT_UI_BRANCH"
)"

if [[ -z "$REMOTE_REACT_UI_COMMIT" ]]; then
    fail "react-ui branch was not found on origin

repository:
  $REACT_UI_REPOSITORY

branch:
  $REACT_UI_BRANCH"
fi

if [[ "$REMOTE_REACT_UI_COMMIT" != "$REACT_UI_COMMIT" ]]; then
    fail "react-ui branch does not point to expected commit

repository:
  $REACT_UI_REPOSITORY

branch:
  $REACT_UI_BRANCH

expected:
  $REACT_UI_COMMIT

actual:
  $REMOTE_REACT_UI_COMMIT"
fi

echo "react-ui origin confirmed:"
echo "  branch: $REACT_UI_BRANCH"
echo "  commit: $REMOTE_REACT_UI_COMMIT"

echo
info "Checking release tag"

if [[ -n "${GITHUB_REF_NAME:-}" && "$INFO_TAG" != "$GITHUB_REF_NAME" ]]; then
    fail "release tag does not match GitHub ref"
fi

echo "release tag confirmed:"
echo "  $INFO_TAG"

echo
info "Checking framework checkout"

CHECKOUT_FRAMEWORK_DIR="$ROOT_DIR/../.release-framework-verify"

rm -rf "$CHECKOUT_FRAMEWORK_DIR"

checkout_framework \
    "$FRAMEWORK_COMMIT" \
    "$CHECKOUT_FRAMEWORK_DIR"

CHECKED_FRAMEWORK_COMMIT="$(
    git -C "$CHECKOUT_FRAMEWORK_DIR" rev-parse HEAD
)"

if [[ "$CHECKED_FRAMEWORK_COMMIT" != "$FRAMEWORK_COMMIT" ]]; then
    rm -rf "$CHECKOUT_FRAMEWORK_DIR"

    fail "framework checkout commit mismatch

expected:
  $FRAMEWORK_COMMIT

actual:
  $CHECKED_FRAMEWORK_COMMIT"
fi

rm -rf "$CHECKOUT_FRAMEWORK_DIR"

echo "framework checkout confirmed:"
echo "  $FRAMEWORK_COMMIT"

echo
info "Release metadata verification passed"