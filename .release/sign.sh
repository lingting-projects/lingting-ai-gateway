#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/common.sh"

usage() {
    echo "Usage:"
    echo "  bash .release/sign.sh sign"
    echo "  bash .release/sign.sh verify"
    exit 1
}

if [[ $# -ne 1 ]]; then
    usage
fi

ACTION="$1"

require_command ssh-keygen

case "$ACTION" in
    sign)
        require_file "$SIGN_KEY"
        require_file "${SIGN_KEY}.pub"
        require_file "$INFO_FILE"

        rm -f "$SIG_FILE"

        ssh-keygen \
            -Y sign \
            -f "$SIGN_KEY" \
            -n "$SIGN_NAMESPACE" \
            "$INFO_FILE"

        mv \
            "${INFO_FILE}.sig" \
            "$SIG_FILE"

        echo "signature generated:"
        echo "  $SIG_FILE"
        ;;

    verify)
        require_file "$INFO_FILE"
        require_file "$SIG_FILE"
        require_file "$PUBKEY_FILE"

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
