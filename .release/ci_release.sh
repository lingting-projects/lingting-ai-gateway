#!/usr/bin/env bash

set -e

cd dist/release

cat ./*.sha256 > SHA256SUMS
