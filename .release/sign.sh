#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

INFO_FILE="$ROOT_DIR/.release/info"
SIG_FILE="$ROOT_DIR/.release/info.sig"
PUBKEY_FILE="$ROOT_DIR/.release/pubkey"

SIGN_KEY="${HOME}/.ssh/lingting_gateway_ed25519"
SIGN_IDENTITY="lingting-release"
SIGN_NAMESPACE="lingting-release"

usage() {
    echo "Usage:"
    echo "  $0 sign"
    echo "  $0 verify"
    exit 1
}

if [[ $# -ne 1 ]]; then
    usage
fi

ACTION="$1"

if ! command -v ssh-keygen >/dev/null 2>&1; then
    echo "error: ssh-keygen not found"
    exit 1
fi

case "$ACTION" in
    sign)
        if [[ ! -f "$SIGN_KEY" ]]; then
            echo "error: private key not found:"
            echo "  $SIGN_KEY"
            exit 1
        fi

        if [[ ! -f "$INFO_FILE" ]]; then
            echo "error: info file not found:"
            echo "  $INFO_FILE"
            exit 1
        fi

        if [[ ! -f "${SIGN_KEY}.pub" ]]; then
            echo "error: public key not found:"
            echo "  ${SIGN_KEY}.pub"
            exit 1
        fi

        rm -f "$SIG_FILE"

        ssh-keygen \
            -Y sign \
            -f "$SIGN_KEY" \
            -n "$SIGN_NAMESPACE" \
            "$INFO_FILE"

        mv "${INFO_FILE}.sig" "$SIG_FILE"

        echo "signature generated:"
        echo "  $SIG_FILE"
        ;;

    verify)
        if [[ ! -f "$INFO_FILE" ]]; then
            echo "error: info file not found:"
            echo "  $INFO_FILE"
            exit 1
        fi

        if [[ ! -f "$SIG_FILE" ]]; then
            echo "error: signature file not found:"
            echo "  $SIG_FILE"
            exit 1
        fi

        if [[ ! -f "$PUBKEY_FILE" ]]; then
            echo "error: public key file not found:"
            echo "  $PUBKEY_FILE"
            exit 1
        fi

        ssh-keygen \
            -Y verify \
            -f "$PUBKEY_FILE" \
            -I "$SIGN_IDENTITY" \
            -n "$SIGN_NAMESPACE" \
            -s "$SIG_FILE" \
            < "$INFO_FILE"

        echo "signature verification succeeded"
        ;;

    *)
        usage
        ;;
esac