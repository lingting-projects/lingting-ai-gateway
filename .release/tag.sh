#!/usr/bin/env bash
set -euo pipefail

RELEASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$RELEASE_DIR/.." && pwd)"

cd "$ROOT_DIR"

exec bash "$RELEASE_DIR/release.sh" tag