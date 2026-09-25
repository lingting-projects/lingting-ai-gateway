#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"

require_command git
require_command ssh-keygen
require_command awk
require_command sed

require_file "$PUBKEY_FILE"
require_file "$INFO_FILE"
require_file "$SIG_FILE"

if [[ -z "${RELEASE_PUBKEY_FINGERPRINT:-}" ]]; then
    fail "RELEASE_PUBKEY_FINGERPRINT is not set"
fi

info "Checking release public key"

ACTUAL_PUBKEY_FINGERPRINT="$(
    ssh-keygen -lf "$PUBKEY_FILE" -E sha256 |
        awk '{ print $2 }'
)"

echo "expected fingerprint:"
echo "  $RELEASE_PUBKEY_FINGERPRINT"

echo "actual fingerprint:"
echo "  $ACTUAL_PUBKEY_FINGERPRINT"

if [[ "$ACTUAL_PUBKEY_FINGERPRINT" != "$RELEASE_PUBKEY_FINGERPRINT" ]]; then
    fail "release public key fingerprint mismatch"
fi

info "Verifying release metadata signature"

bash "$SCRIPT_DIR/sign.sh" verify

info "Reading release metadata"

INFO_VERSION="$(read_info_value version)"
INFO_TAG="$(read_info_value tag)"
INFO_REPOSITORY="$(read_info_value repository)"
INFO_BRANCH="$(read_info_value branch)"
INFO_COMMIT="$(read_info_value commit)"
INFO_TIMESTAMP="$(read_info_value timestamp)"
INFO_TIMESTAMP_UNIX="$(read_info_value timestamp_unix)"

if [[ "$INFO_VERSION" != "1" ]]; then
    fail "unsupported release metadata version: $INFO_VERSION"
fi

if [[ -z "$INFO_TAG" ]]; then
    fail "release metadata tag is empty"
fi

if [[ -z "${GITHUB_REF_NAME:-}" ]]; then
    fail "GITHUB_REF_NAME is not set"
fi

if [[ "$INFO_TAG" != "$GITHUB_REF_NAME" ]]; then
    fail "release metadata tag does not match GitHub tag

metadata:
  $INFO_TAG

github:
  $GITHUB_REF_NAME"
fi

if [[ "$INFO_REPOSITORY" != "$FRAMEWORK_REPOSITORY" ]]; then
    fail "framework repository mismatch

expected:
  $FRAMEWORK_REPOSITORY

metadata:
  $INFO_REPOSITORY"
fi

if [[ -z "$INFO_BRANCH" ]]; then
    fail "framework branch is empty"
fi

require_sha1_commit "$INFO_COMMIT"

if [[ -z "$INFO_TIMESTAMP" ]]; then
    fail "release timestamp is empty"
fi

if [[ ! "$INFO_TIMESTAMP_UNIX" =~ ^[0-9]+$ ]]; then
    fail "invalid release timestamp_unix: $INFO_TIMESTAMP_UNIX"
fi

echo
echo "release metadata:"
echo "  version:        $INFO_VERSION"
echo "  tag:            $INFO_TAG"
echo "  repository:     $INFO_REPOSITORY"
echo "  branch:         $INFO_BRANCH"
echo "  commit:         $INFO_COMMIT"
echo "  timestamp:      $INFO_TIMESTAMP"
echo "  timestamp_unix: $INFO_TIMESTAMP_UNIX"

info "Checking framework branch on origin"

REMOTE_FRAMEWORK_COMMIT=""

REMOTE_FRAMEWORK_COMMIT="$(
    get_remote_branch_commit \
        "$FRAMEWORK_REPOSITORY" \
        "$INFO_BRANCH"
)"

if [[ -z "$REMOTE_FRAMEWORK_COMMIT" ]]; then
    fail "framework branch was not found on origin

branch:
  $INFO_BRANCH"
fi

if [[ "$REMOTE_FRAMEWORK_COMMIT" != "$INFO_COMMIT" ]]; then
    fail "framework branch does not point to release commit

branch:
  $INFO_BRANCH

expected:
  $INFO_COMMIT

actual:
  $REMOTE_FRAMEWORK_COMMIT"
fi

echo "framework branch confirmed:"
echo "  branch: $INFO_BRANCH"
echo "  commit: $REMOTE_FRAMEWORK_COMMIT"

info "Checking framework commit"

checkout_framework \
    "$INFO_COMMIT" \
    "$FRAMEWORK_DIR"

echo
echo "========================================"
echo "Release verification succeeded"
echo "========================================"
echo

echo "framework:"
echo "  $FRAMEWORK_DIR"

echo
echo "commit:"
echo "  $INFO_COMMIT"
