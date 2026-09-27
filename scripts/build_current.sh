#!/usr/bin/env bash

set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

TARGET="${RUST_TARGET:-}"
PROFILE="${CARGO_PROFILE:-release}"

cargo run -p lib-web --example build_ts --features ts-export

if [ -n "$TARGET" ]; then
    cargo build \
        --locked \
        --profile "$PROFILE" \
        --package bin-server \
        --target "$TARGET"

    BINARY="target/$TARGET/$PROFILE/bin-server"
else
    cargo build \
        --locked \
        --profile "$PROFILE" \
        --package bin-server

    BINARY="target/$PROFILE/bin-server"
fi

if [[ "$TARGET" == *"-windows-"* ]]; then
    BINARY="${BINARY}.exe"
fi

OUTPUT_DIR="${BUILD_OUTPUT_DIR:-$(dirname "$BINARY")}"
mkdir -p "$OUTPUT_DIR"

cp "$BINARY" "$OUTPUT_DIR/"

echo "Build complete: $OUTPUT_DIR/$(basename "$BINARY")"
