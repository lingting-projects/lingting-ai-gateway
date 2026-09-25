#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"

require_command sha256sum

require_directory "$DIST_DIR"

EXPECTED_FILES=(
    "lingting-ai-gateway-linux"
    "lingting-ai-gateway-linux-slim"
    "lingting-ai-gateway-windows.exe"
    "lingting-ai-gateway-windows-slim.exe"
    "lingting-ai-gateway-macos"
    "lingting-ai-gateway-macos-slim"
)

info "Checking release artifacts"

for file_name in "${EXPECTED_FILES[@]}"; do
    file_path="$DIST_DIR/$file_name"

    if [[ ! -f "$file_path" ]]; then
        fail "release artifact not found: $file_path"
    fi

    if [[ ! -s "$file_path" ]]; then
        fail "release artifact is empty: $file_path"
    fi
done

info "Generating SHA256SUMS"

CHECKSUM_FILE="$DIST_DIR/SHA256SUMS"

rm -f "$CHECKSUM_FILE"

(
    cd "$DIST_DIR"

    sha256sum \
        "${EXPECTED_FILES[@]}" \
        > "$CHECKSUM_FILE"
)

echo
cat "$CHECKSUM_FILE"

echo
echo "========================================"
echo "Release artifacts verified"
echo "========================================"
