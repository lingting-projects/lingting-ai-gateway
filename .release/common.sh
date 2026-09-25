#!/usr/bin/env bash
set -euo pipefail

RELEASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$RELEASE_DIR/.." && pwd)"

GATEWAY_REPOSITORY="https://github.com/lingting-projects/lingting-ai-gateway.git"
FRAMEWORK_REPOSITORY="https://github.com/lingting-projects/lingting-rust-framework.git"
REACT_UI_REPOSITORY="https://github.com/lingting/lingting-react-ui.git"

FRAMEWORK_DIR="${FRAMEWORK_DIR:-$ROOT_DIR/../lingting-rust-framework}"
REACT_UI_DIR="${REACT_UI_DIR:-$ROOT_DIR/../lingting-react-ui}"

SIGN_KEY="${LINGTING_RELEASE_SIGN_KEY:-${HOME}/.ssh/lingting_gateway_ed25519}"
SIGN_IDENTITY="lingting-release"
SIGN_NAMESPACE="lingting-release"

INFO_FILE="$RELEASE_DIR/info"
SIG_FILE="$RELEASE_DIR/info.sig"
PUBKEY_FILE="$RELEASE_DIR/pubkey"

DIST_DIR="$ROOT_DIR/dist"
TARGET_DIR="$ROOT_DIR/target"