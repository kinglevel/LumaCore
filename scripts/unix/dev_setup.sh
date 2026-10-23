#!/usr/bin/env bash
# One-time machine setup so an ordinary user (not root) can build, test and run LumaCore.
#
# What it does (each step is skipped when already done):
#   1. installs the system packages a build needs (apt, asks for sudo once)
#   2. gives the repo back to you: files a root build left behind (target/, test results,
#      logs) are chowned to your user, so cargo can overwrite them
#   3. installs Rust for your user with rustup (stable, with clippy + rustfmt)
#   4. puts ~/.cargo/bin on PATH for bash and fish
#   5. uses mold as the linker for your user only (~/.cargo/config.toml, not the repo)
#   6. installs cargo-machete (finds unused dependencies, decisions §31)
#   7. installs the Playwright browsers for tests/e2e
#   8. optional: nightly + cargo-fuzz for the overnight fuzz run (--nightly)
#      optional: real-time scheduling for the render / send threads (--realtime)
#   9. checks that a small crate builds
#
# Usage:
#   ./scripts/unix/dev_setup.sh              # everything above except nightly
#   ./scripts/unix/dev_setup.sh --nightly    # also nightly + cargo-fuzz
#   ./scripts/unix/dev_setup.sh --no-apt     # skip the system packages
#   ./scripts/unix/dev_setup.sh --clean      # also delete target/debug (rebuilt with the §31 profile)
#   ./scripts/unix/dev_setup.sh --realtime   # also install scripts/unix/realtime.conf (SCHED_FIFO)
#
# Run it as yourself, not with sudo: it calls sudo only where it must.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

NIGHTLY=0
APT=1
CLEAN=0
REALTIME=0
for arg in "$@"; do
    case "$arg" in
        --nightly) NIGHTLY=1 ;;
        --no-apt) APT=0 ;;
        --clean) CLEAN=1 ;;
        --realtime) REALTIME=1 ;;
        -h|--help) sed -n '2,23p' "$0"; exit 0 ;;
        *) printf 'error: unknown option %s (see --help)\n' "$arg" >&2; exit 2 ;;
    esac
done

say()  { printf '\n==> %s\n' "$1"; }
ok()   { printf '    ok: %s\n' "$1"; }
warn() { printf '    warning: %s\n' "$1" >&2; }
die()  { printf 'error: %s\n' "$1" >&2; exit 1; }

[ "$(id -u)" -ne 0 ] || die "run this as your own user, not root (it uses sudo where needed)"
ME="$(id -un)"
MY_GROUP="$(id -gn)"
CARGO_BIN="$HOME/.cargo/bin"

# --- 1. system packages -----------------------------------------------------------------
if [ "$APT" -eq 1 ]; then
    say "System packages (sudo apt)"
    PACKAGES=(
        build-essential pkg-config curl git cmake   # compilers, C deps (zstd, …)
        clang mold                                  # fast linking
        libgtk-3-dev libwebkit2gtk-4.1-dev libxdo-dev  # the desktop app (wry / tao / muda)
        nodejs npm                                  # tests/e2e (Playwright)
    )
    missing=()
    for p in "${PACKAGES[@]}"; do
        dpkg -s "$p" >/dev/null 2>&1 || missing+=("$p")
    done
    if [ ${#missing[@]} -eq 0 ]; then
        ok "all installed"
    else
        printf '    installing: %s\n' "${missing[*]}"
        sudo apt-get update -qq
        sudo apt-get install -y "${missing[@]}"
    fi
fi

# --- 2. repo ownership ------------------------------------------------------------------
say "Files in the repo owned by someone else"
foreign="$(find "$REPO_ROOT" \! -user "$ME" -print -quit 2>/dev/null || true)"
if [ -z "$foreign" ]; then
    ok "everything belongs to $ME"
else
    count="$(find "$REPO_ROOT" \! -user "$ME" 2>/dev/null | wc -l)"
    printf '    %s files belong to another user (e.g. %s); chowning to %s\n' \
        "$count" "${foreign#"$REPO_ROOT"/}" "$ME"
    sudo chown -R "$ME:$MY_GROUP" "$REPO_ROOT"
    ok "done"
fi

if [ "$CLEAN" -eq 1 ] && [ -d "$REPO_ROOT/target/debug" ]; then
    say "Deleting target/debug ($(du -sh "$REPO_ROOT/target/debug" | cut -f1))"
    rm -rf "$REPO_ROOT/target/debug"
    ok "deleted (target/release, target/sweep and the reports stay)"
fi

# --- 3. Rust ----------------------------------------------------------------------------
say "Rust for $ME"
export PATH="$CARGO_BIN:$PATH"
if command -v rustup >/dev/null 2>&1; then
    ok "rustup present; updating stable"
    rustup update stable --no-self-update >/dev/null
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile default
fi
rustup component add clippy rustfmt >/dev/null
ok "$(rustc -V)"
ok "$(cargo -V)"

# --- 4. PATH for bash and fish ----------------------------------------------------------
say "PATH"
if ! grep -qs '.cargo/env' "$HOME/.bashrc" "$HOME/.profile"; then
    printf '\n. "$HOME/.cargo/env"\n' >> "$HOME/.bashrc"
    ok "added ~/.cargo/env to ~/.bashrc"
else
    ok "bash already loads ~/.cargo/env"
fi
if command -v fish >/dev/null 2>&1; then
    fish -c "fish_add_path -U '$CARGO_BIN'" && ok "fish: $CARGO_BIN on PATH (universal)"
fi

# --- 5. linker (user-level, so the repo keeps working on machines without mold) ----------
say "Linker"
USER_CARGO_CONFIG="$HOME/.cargo/config.toml"
if ! command -v mold >/dev/null 2>&1; then
    warn "mold not installed; keeping the default linker"
elif grep -qs 'fuse-ld=mold' "$USER_CARGO_CONFIG"; then
    ok "mold already set in $USER_CARGO_CONFIG"
elif grep -qs 'x86_64-unknown-linux-gnu' "$USER_CARGO_CONFIG"; then
    warn "$USER_CARGO_CONFIG already has a linux target section; add mold by hand"
elif [ "$(uname -m)" = "x86_64" ]; then
    mkdir -p "$HOME/.cargo"
    cat >> "$USER_CARGO_CONFIG" <<'EOF'

# Added by LumaCore scripts/unix/dev_setup.sh: link with mold (much faster than ld).
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF
    ok "mold set in $USER_CARGO_CONFIG"
fi

# --- 6. cargo-machete -------------------------------------------------------------------
say "cargo-machete"
if command -v cargo-machete >/dev/null 2>&1; then
    ok "present"
else
    cargo install --locked -j 2 cargo-machete
fi

# --- 7. Playwright browsers -------------------------------------------------------------
say "Playwright (tests/e2e)"
if [ -d "$REPO_ROOT/tests/e2e" ]; then
    (
        cd "$REPO_ROOT/tests/e2e"
        [ -d node_modules ] || npm ci
        npx --yes playwright install
    ) && ok "browsers installed for $ME"
fi

# --- 8. nightly + cargo-fuzz (optional) -------------------------------------------------
if [ "$NIGHTLY" -eq 1 ]; then
    say "Nightly + cargo-fuzz (overnight fuzz run)"
    rustup toolchain install nightly --profile minimal >/dev/null
    ok "$(rustc +nightly -V)"
    command -v cargo-fuzz >/dev/null 2>&1 || cargo install --locked -j 2 cargo-fuzz
    ok "cargo-fuzz present"
fi

# --- real-time scheduling (optional) ---------------------------------------------------
if [ "$REALTIME" -eq 1 ]; then
    say "Real-time scheduling (render and send threads, decisions §31)"
    LIMITS=/etc/security/limits.d/90-lumacore.conf
    if cmp -s "$REPO_ROOT/scripts/unix/realtime.conf" "$LIMITS"; then
        ok "$LIMITS is current"
    else
        sudo cp "$REPO_ROOT/scripts/unix/realtime.conf" "$LIMITS"
        ok "installed $LIMITS"
    fi
    sudo groupadd -f lumacore
    if id -nG "$ME" | tr ' ' '\n' | grep -qx lumacore; then
        ok "$ME is in group lumacore"
    else
        sudo usermod -aG lumacore "$ME"
        ok "added $ME to group lumacore — log out and in again (tmux too) for it to apply"
    fi
fi

# --- 9. check -------------------------------------------------------------------------
say "Test build (lumacore-base)"
(cd "$REPO_ROOT" && cargo build -q -p lumacore-base) && ok "builds"

say "Done"
cat <<EOF
    Open a new terminal (or run: source ~/.cargo/env) so cargo is on PATH.
    Builds use 2 jobs (.cargo/config.toml); override with: cargo build -j 4
    Next:
      cargo build -p lumacore-station --features http     # the station
      ./scripts/unix/test quick                   # every crate's tests
EOF
