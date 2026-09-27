#!/usr/bin/env bash

set -e

TARGET="${RELEASE_TARGET:?RELEASE_TARGET is required}"
VERSION="${RELEASE_VERSION:?RELEASE_VERSION is required}"

SOURCE="dist/bin/bin-server"
NAME="lingting-ai-gateway-${VERSION}-${TARGET}"

if [[ "$TARGET" == windows-* ]]; then
    SOURCE="${SOURCE}.exe"
    NAME="${NAME}.exe"
fi

OUTPUT="dist/release/${NAME}"

mkdir -p dist/release
cp "$SOURCE" "$OUTPUT"

if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$OUTPUT" > "${OUTPUT}.sha256"
else
    shasum -a 256 "$OUTPUT" > "${OUTPUT}.sha256"
fi
