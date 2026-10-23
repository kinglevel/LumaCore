# Shared helpers for the Unix scripts — source it, don't run it.
#
#   source "$(dirname "$0")/lib.sh"
#
# Gives: REPO_ROOT, WEB_ROOT, lc_profile (--debug), the one server feature
# set, the build functions, server_pids / server_healthy / wait_healthy /
# stop_server / start_server_bg, has_display, open_terminal / open_url,
# log / die.
#
# Every run writes a log: target/script-logs/<script>_<time>.log (cargo's
# full output included, colours stripped); target/script-logs/latest.log
# points at the newest.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"
source "$HOME/.cargo/env" 2>/dev/null || true

# Building as two users (root and you) leaves files the other can't write:
# cargo and the logs then fail. Say so up front, with the fix.
for dir in "$REPO_ROOT/target" "$REPO_ROOT/target/script-logs" \
    "$REPO_ROOT/target/local" "$REPO_ROOT/target/debug" "$REPO_ROOT/target/release"; do
    if [ -e "$dir" ] && [ ! -w "$dir" ]; then
        echo "✘ $dir belongs to $(stat -c %U "$dir" 2>/dev/null || stat -f %Su "$dir") and $(id -un) can't write to it." >&2
        echo "  Run the script as that user, or give the files back:" >&2
        echo "  sudo chown -R $(id -un) \"$REPO_ROOT/target\"" >&2
        exit 1
    fi
done

LOG_DIR="$REPO_ROOT/target/script-logs"
mkdir -p "$LOG_DIR"
SCRIPT_NAME="$(basename "$0")"
# Read-only commands (logs, server status, connect) set LC_NO_LOG=1 before
# sourcing: they write no log, so latest.log stays on the run being watched.
if [ "${LC_NO_LOG:-0}" = 1 ]; then
    SCRIPT_LOG=/dev/null
else
    SCRIPT_LOG="$LOG_DIR/${SCRIPT_NAME}_$(date +%Y%m%d_%H%M%S)_$$.log"
    : > "$SCRIPT_LOG"
    ln -sfn "$(basename "$SCRIPT_LOG")" "$LOG_DIR/latest.log"
    # Keep the newest 100 runs.
    ls -1t "$LOG_DIR"/*_*.log 2>/dev/null | tail -n +101 | xargs -r rm -f
fi

# Colours only on a terminal.
if [ -t 1 ]; then
    C_STEP=$'\e[1;36m' C_OK=$'\e[1;32m' C_ERR=$'\e[1;31m' C_DIM=$'\e[2m' C_OFF=$'\e[0m'
else
    C_STEP="" C_OK="" C_ERR="" C_DIM="" C_OFF=""
fi

# Plain text for the log file: no colour codes, progress-bar redraws dropped.
strip_term() {
    sed -u -e 's/\x1b\[[0-9;?]*[a-zA-Z]//g' -e 's/.*\r//'
}

# log <text…> — a timestamped step line, on screen and in the log.
log() {
    local t
    t="$(date +%H:%M:%S)"
    printf '%s[%s]%s %s\n' "$C_DIM" "$t" "$C_OFF" "$*"
    printf '[%s] %s\n' "$t" "$*" | strip_term >> "$SCRIPT_LOG"
}
log_ok() { log "${C_OK}✔${C_OFF} $*"; }
log_err() { log "${C_ERR}✘${C_OFF} $*" >&2; }

die() {
    log_err "$*"
    log "Log: $SCRIPT_LOG"
    exit 1
}

if [ "$SCRIPT_LOG" != /dev/null ]; then
    log "${C_STEP}▶ $SCRIPT_NAME${*:+ $*}${C_OFF}  (log: $SCRIPT_LOG)"
fi

# One feature set for every build of the station, so switching scripts never
# relinks a server without the web UI. Tracy only when asked (--tracy).
SERVER_FEATURES="perfetto,http"

# The WebUI folder every script-started server serves from disk (`--web-root`),
# so a JS / CSS edit is live without a rebuild; without it the server serves the
# copy built into its binary (decisions §36 2). LC_WEB_ROOT overrides it.
WEB_ROOT="${LC_WEB_ROOT:-$REPO_ROOT/crates/ui/lumacore-webui/web}"

# Default ports (lumacore-server --help).
CMD_PORT=47653
SHELL_PORT=47654
HTTP_PORT=47656

# The server stops within its 5 s shutdown deadline; give it some slack.
STOP_WAIT_S=8

# lc_profile 0|1 — sets PROFILE, TARGET_DIR, BUILD_ARGS, SERVER_BIN, CLI_BIN, APP_BIN.
# 1 = dev (--debug); otherwise the `local` profile (decisions §34 "Build times hardened" 2:
# release speed, incremental), or full release when SHIP=1 (--ship).
SHIP="${SHIP:-0}"
lc_profile() {
    if [ "${1:-0}" -eq 1 ]; then
        PROFILE="dev"
        TARGET_DIR="$REPO_ROOT/target/debug"
        BUILD_ARGS=()
    elif [ "$SHIP" -eq 1 ]; then
        PROFILE="release"
        TARGET_DIR="$REPO_ROOT/target/release"
        BUILD_ARGS=(--release)
    else
        PROFILE="local"
        TARGET_DIR="$REPO_ROOT/target/local"
        BUILD_ARGS=(--profile local)
    fi
    SERVER_BIN="$TARGET_DIR/lumacore-server"
    CLI_BIN="$TARGET_DIR/lumacore-cli"
    APP_BIN="$TARGET_DIR/lumacore-app"
    SERVER_LOG="$TARGET_DIR/server.log"
}

have_cargo() {
    command -v cargo >/dev/null 2>&1
}

# reuse_without_cargo <binary> — false when it has to build; true (with a warning)
# when there is no cargo for this user but a build exists to run.
reuse_without_cargo() {
    have_cargo && return 1
    [ -x "$1" ] || die "cargo not found for $(id -un) and $1 isn't built. Install Rust for $(id -un) (https://rustup.rs) or build as a user who has it."
    log "${C_ERR}!${C_OFF} cargo not found for $(id -un) — not building, using the existing $(basename "$1") built $(date -r "$1" '+%Y-%m-%d %H:%M')"
    return 0
}

# Cargo draws its progress bar and colours on a terminal even through tee.
if [ -t 1 ]; then
    export CARGO_TERM_COLOR=always CARGO_TERM_PROGRESS_WHEN=always
    CARGO_TERM_PROGRESS_WIDTH="$(tput cols 2>/dev/null || echo 100)"
    export CARGO_TERM_PROGRESS_WIDTH
fi

# run_logged <command…> — run it with its output live on screen and, at the
# same moment, in $SCRIPT_LOG (plain text, so `logs` / tail -F can follow a
# run from another terminal). Sets RUN_CODE (the command's own exit code,
# whatever the caller's errexit / pipefail), RUN_SECONDS and RUN_OUT (a copy
# of the output; the caller removes it). Returns RUN_CODE.
run_logged() {
    local fifo strip_pid start=$SECONDS
    RUN_OUT="$(mktemp)"
    fifo="$RUN_OUT.fifo"
    mkfifo "$fifo"
    # The log writer outlives Ctrl-C (a caller's INT trap would otherwise
    # give it the default action) so a run's last lines still reach the log.
    { trap '' INT; strip_term < "$fifo" >> "$SCRIPT_LOG"; } &
    strip_pid=$!
    { "$@" 2>&1 && echo 0 > "$RUN_OUT.code" || echo $? > "$RUN_OUT.code"; } | tee -i "$RUN_OUT" "$fifo"
    wait "$strip_pid" 2>/dev/null || true
    RUN_CODE="$(cat "$RUN_OUT.code")"
    RUN_SECONDS=$((SECONDS - start))
    rm -f "$fifo" "$RUN_OUT.code"
    return "$RUN_CODE"
}

# step <label> <command…> — a logged step with a ✔ / ✘ line and its time.
step() {
    local label="$1"
    shift
    log "${C_STEP}▶ $label${C_OFF}  ($*)"
    run_logged "$@" || true
    rm -f "$RUN_OUT"
    if [ "$RUN_CODE" -eq 0 ]; then
        log_ok "$label passed ($(fmt_secs "$RUN_SECONDS"))"
    else
        log_err "$label FAILED (exit $RUN_CODE, $(fmt_secs "$RUN_SECONDS")) — output above and in $SCRIPT_LOG"
    fi
    return "$RUN_CODE"
}

fmt_secs() {
    local s="$1"
    if [ "$s" -ge 3600 ]; then
        printf '%dh%02dm' $((s / 3600)) $((s % 3600 / 60))
    elif [ "$s" -ge 60 ]; then
        printf '%dm%02ds' $((s / 60)) $((s % 60))
    else
        printf '%ds' "$s"
    fi
}

# cargo_build <cargo args…> — cargo's full output live (progress bar and
# colours kept) and in the log; judged by cargo's exit status (a stale
# binary from an older build must not pass).
cargo_build() {
    local args=(build ${BUILD_ARGS[@]+"${BUILD_ARGS[@]}"} "$@")
    local compiled
    log "${C_STEP}cargo ${args[*]}${C_OFF}"
    if ! have_cargo; then
        log_err "cargo not found for $(id -un) (PATH: $PATH). Install Rust for $(id -un) (https://rustup.rs) or run as a user who has it."
        return 127
    fi
    run_logged cargo "${args[@]}" || true
    compiled="$(strip_term < "$RUN_OUT" | grep -c '^ *Compiling ' || true)"
    rm -f "$RUN_OUT"
    BUILT_CRATES="$compiled"
    if [ "$RUN_CODE" -ne 0 ]; then
        log_err "cargo failed (exit $RUN_CODE) after $(fmt_secs "$RUN_SECONDS") — the errors are above and in $SCRIPT_LOG"
        return "$RUN_CODE"
    fi
    # The build's stamp (decisions §34 "Build times hardened" 4): when it finished, how long,
    # how many crates.
    log_ok "Built $(built_what "$@") · finished $(date '+%Y-%m-%d %H:%M:%S') · $(fmt_secs "$RUN_SECONDS") · $compiled crate$([ "$compiled" -eq 1 ] || echo s) · $PROFILE"
}

# built_what <cargo args…> — the binary or package a build names (`--bin x`, else `-p x`).
built_what() {
    local prev="" what=""
    for a in "$@"; do
        case "$prev" in
            --bin) what="$a" ;;
            -p) [ -z "$what" ] && what="$a" ;;
        esac
        prev="$a"
    done
    echo "${what:-workspace}"
}

# build_server [extra features] — e.g. build_server tracy
build_server() {
    local features="$SERVER_FEATURES"
    [ -n "${1:-}" ] && features="$features,$1"
    log "Building lumacore-server ($PROFILE, features $features)"
    cargo_build -p lumacore-station --bin lumacore-server --features "$features"
}

build_cli() {
    log "Building lumacore-cli ($PROFILE)"
    cargo_build -p lumacore-cli
}

# The app alone: it connects to any station. Starting a local station from
# it needs the lumacore-server next to it too (build server).
build_app() {
    log "Building lumacore-app ($PROFILE)"
    cargo_build -p lumacore-app
}

# PIDs of any lumacore-server: by name and by the default ports.
server_pids() {
    {
        pgrep -x lumacore-server || true
        lsof -tiTCP:"$SHELL_PORT" -sTCP:LISTEN 2>/dev/null || true
        lsof -tiUDP:"$CMD_PORT" 2>/dev/null || true
    } | sed '/^$/d' | sort -u
}

server_healthy() {
    [ -x "$CLI_BIN" ] && "$CLI_BIN" --target 127.0.0.1 --health >/dev/null 2>&1
}

# wait_healthy <pid> [seconds]
wait_healthy() {
    local pid="$1" tries=$(( ${2:-10} * 2 )) i
    for ((i = 0; i < tries; i++)); do
        if ! kill -0 "$pid" 2>/dev/null; then
            log_err "lumacore-server (PID $pid) exited before becoming healthy — its output ($SERVER_LOG):"
            tail -n 20 "$SERVER_LOG" 2>/dev/null | tee -a "$SCRIPT_LOG" >&2 || true
            return 1
        fi
        server_healthy && return 0
        sleep 0.5
    done
    log_err "lumacore-server did not become healthy after ${2:-10} s. Its output: $SERVER_LOG"
    return 1
}

# stop_server — SIGTERM (the server autosaves unsaved changes and shuts down
# in order), wait for it, SIGKILL only with --force.
stop_server() {
    local force="${1:-}" pids i
    pids="$(server_pids)"
    if [ -z "$pids" ]; then
        log "No lumacore-server running."
        return 0
    fi
    log "Stopping lumacore-server (SIGTERM): $(echo $pids)"
    for pid in $pids; do kill "$pid" 2>/dev/null || true; done
    for ((i = 0; i < STOP_WAIT_S * 2; i++)); do
        [ -z "$(server_pids)" ] && { log_ok "lumacore-server stopped."; return 0; }
        sleep 0.5
    done
    pids="$(server_pids)"
    if [ "$force" = "--force" ]; then
        log "Still running after ${STOP_WAIT_S} s — killing (SIGKILL): $(echo $pids)"
        for pid in $pids; do kill -9 "$pid" 2>/dev/null || true; done
        sleep 0.5
        [ -z "$(server_pids)" ] && { log_ok "lumacore-server killed."; return 0; }
    fi
    log_err "lumacore-server still running after ${STOP_WAIT_S} s: $(echo $pids) (use --force)"
    return 1
}

# server_cmd [server args…] — sets SERVER_CMD to the full
# command (binary, the web root, then the args) for start_server_bg / --fg.
server_cmd() {
    SERVER_CMD=("$SERVER_BIN")
    if [ -d "$WEB_ROOT" ]; then
        SERVER_CMD+=(--web-root "$WEB_ROOT")
    else
        log_err "Web root $WEB_ROOT not found — the server serves its built-in WebUI"
    fi
    SERVER_CMD+=("$@")
}

# start_server_bg [server args…] — background, output to $SERVER_LOG; sets SERVER_PID.
start_server_bg() {
    [ -x "$SERVER_BIN" ] || die "$SERVER_BIN isn't built (./scripts/unix/build server)"
    server_cmd "$@"
    : > "$SERVER_LOG"
    log "Starting ${SERVER_CMD[*]}"
    nohup "${SERVER_CMD[@]}" >>"$SERVER_LOG" 2>&1 &
    SERVER_PID=$!
}

# A graphical session to open windows on (the app, terminal windows).
has_display() {
    [[ "$OSTYPE" == "darwin"* ]] || [ -n "${DISPLAY:-}" ] || [ -n "${WAYLAND_DISPLAY:-}" ]
}

# open_terminal <command> [title] — run it in a new terminal window.
open_terminal() {
    local cmd="$1" title="${2:-LumaCore}"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        case "${TERM_PROGRAM:-Terminal}" in
            iTerm.app|iTerm2)
                osascript <<APPLE
tell application "iTerm"
    activate
    set newWindow to (create window with default profile)
    tell current session of newWindow
        write text "cd '$REPO_ROOT' && $cmd"
    end tell
end tell
APPLE
                ;;
            *)
                osascript <<APPLE
tell application "Terminal"
    activate
    do script "cd '$REPO_ROOT' && $cmd"
end tell
APPLE
                ;;
        esac
    elif command -v gnome-terminal &>/dev/null; then
        gnome-terminal --title="$title" -- bash -c "$cmd; exec bash" &
    elif command -v konsole &>/dev/null; then
        konsole --new-tab -e bash -c "$cmd; exec bash" &
    elif command -v x-terminal-emulator &>/dev/null; then
        x-terminal-emulator -e bash -c "$cmd; exec bash" &
    elif [ -n "${TERMINAL:-}" ]; then
        $TERMINAL -e bash -c "$cmd; exec bash" &
    else
        log_err "No terminal emulator found (set \$TERMINAL) — not opening: $cmd"
        return 1
    fi
}

# open_url <url> — the default browser.
open_url() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open "$1"
    elif command -v xdg-open &>/dev/null; then
        xdg-open "$1" >/dev/null 2>&1 &
    else
        log_err "No xdg-open — open $1 yourself"
        return 1
    fi
}

# nightly + cargo-fuzz, needed by the cargo-fuzz targets in tests/fuzz/.
has_cargo_fuzz() {
    cargo +nightly fuzz --version >/dev/null 2>&1
}
