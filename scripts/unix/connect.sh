#!/usr/bin/env bash
# Attach lumacore-cli to a running server; never starts or restarts one.
#
# Usage:
#   ./scripts/unix/connect.sh [--user LD] [--target 192.168.1.20] [--cmd "list fixture"] [--debug | --ship]
set -euo pipefail
export LC_NO_LOG=1
source "$(dirname "$0")/lib.sh"

DEBUG=0
TARGET="127.0.0.1"
CLI_ARGS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --debug) DEBUG=1 ;;
        --ship) SHIP=1 ;;
        --target) TARGET="${2:?--target needs a host}"; shift ;;
        --user) CLI_ARGS+=(--user "${2:?--user needs a name}"); shift ;;
        --cmd|-C) CLI_ARGS+=(--cmd "${2:?--cmd needs a line}"); shift ;;
        -h|--help) sed -n '2,5p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "Unknown argument: $1" ;;
    esac
    shift
done
lc_profile "$DEBUG"

if [ ! -x "$CLI_BIN" ]; then
    build_cli || die "Build failed!"
fi
exec "$CLI_BIN" --target "$TARGET" ${CLI_ARGS[@]+"${CLI_ARGS[@]}"}
