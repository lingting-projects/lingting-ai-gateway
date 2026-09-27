#!/usr/bin/env bash

set -e

RELEASE_DIR="dist/release"

cd "$RELEASE_DIR"

cat ./*.sha256 > SHA256SUMS
