#!/usr/bin/env bash

set -e

if [[ "$(uname -s)" == "Linux" ]]; then
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends musl-tools

    echo "CC_x86_64_unknown_linux_musl=musl-gcc" >> "$GITHUB_ENV"
    echo "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc" >> "$GITHUB_ENV"
fi
