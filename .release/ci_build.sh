#!/usr/bin/env bash
set -euo pipefail

RELEASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$RELEASE_DIR/.." && pwd)"

cd "$ROOT_DIR"

usage() {
    cat <<'EOF'
Usage: ci_build.sh [-s] [-w] [-l] [-m]

  -s  Build slim/dynamically linked variant
  -w  Build Windows
  -l  Build Linux
  -m  Build macOS
EOF
}

slim=false
platform=""

while getopts ":swlm" opt; do
    case "$opt" in
        s)
            slim=true
            ;;
        w)
            platform="windows"
            ;;
        l)
            platform="linux"
            ;;
        m)
            platform="macos"
            ;;
        :)
            echo "error: option -$OPTARG requires no argument" >&2
            usage >&2
            exit 2
            ;;
        \?)
            echo "error: unknown option: -$OPTARG" >&2
            usage >&2
            exit 2
            ;;
    esac
done

shift $((OPTIND - 1))

if [[ $# -ne 0 ]]; then
    echo "error: unexpected arguments: $*" >&2
    usage >&2
    exit 2
fi

if [[ -z "$platform" ]]; then
    echo "error: exactly one platform is required" >&2
    usage >&2
    exit 2
fi

if [[ "$slim" == true && "$platform" == "macos" ]]; then
    echo "error: macOS does not support the slim build" >&2
    exit 2
fi

args=(build "$platform")

if [[ "$slim" == true ]]; then
    args+=(--slim)
fi

exec bash "$RELEASE_DIR/release.sh" "${args[@]}"