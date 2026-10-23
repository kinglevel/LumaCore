#!/usr/bin/env bash
# Developer onboarding helper for LumaCore.
#
# What it does:
# - resolves the current LumaCore repo root
# - checks for basic developer tooling
# - reads .env.local for developer path variables
# - validates the configured paths
# - copies Obsidian CSS snippets
# - builds the lumacore-scripts launcher
#
# Usage:
#   ./scripts/unix/dev_init.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env.local"
ENV_EXAMPLE="${REPO_ROOT}/.env.example"

say() {
    printf '%s\n' "$1"
}

warn() {
    printf 'warning: %s\n' "$1" >&2
}

fail() {
    printf 'error: %s\n' "$1" >&2
    exit 1
}

check_cmd() {
    local cmd="$1"
    if command -v "$cmd" >/dev/null 2>&1; then
        say "  - found ${cmd}: $(command -v "$cmd")"
    else
        warn "missing ${cmd}"
        return 1
    fi
}

say "LumaCore developer init"
say "  - repo root: ${REPO_ROOT}"

missing_tools=0

say ""
say "Checking developer tools..."
check_cmd git || missing_tools=1
check_cmd rustc || missing_tools=1
check_cmd cargo || missing_tools=1

if [ "$missing_tools" -ne 0 ]; then
    say ""
    warn "Some tools are missing. You can still continue, but you should finish setup before building."
    warn "See docs/getting-started.md for platform setup instructions."
fi

say ""
say "Checking environment configuration..."

missing_paths=0

if [ -f "$ENV_FILE" ]; then
    say "  - found .env.local"

    while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
            ""|\#*)
                continue
                ;;
        esac

        if [[ "$line" != *=* ]]; then
            warn "invalid entry (expected NAME=path): ${line}"
            missing_paths=1
            continue
        fi

        name="${line%%=*}"
        value="${line#*=}"

        if [ -z "$name" ] || [ -z "$value" ]; then
            warn "invalid entry (empty name or path): ${line}"
            missing_paths=1
            continue
        fi

        # Resolve relative paths against repo root
        if [[ "$value" != /* ]]; then
            value="${REPO_ROOT}/${value}"
        fi

        if [ -d "$value" ]; then
            say "  - ok: ${name}=${value}"
        else
            warn "missing path: ${name}=${value}"
            missing_paths=1
        fi
    done < "$ENV_FILE"
else
    say "  - no .env.local found (optional)"
    say "  - to configure companion repos, run:"
    say "      cp .env.example .env.local"
    say "      # then edit .env.local with your local paths"
fi

say ""
say "Checking Tracy profiler..."

# tracy-client-sys 0.24.3 bundles Tracy v0.11.1 (protocol 69).
# The Tracy GUI/capture tool MUST match this version exactly, otherwise you
# get "protocol mismatch" when connecting. Homebrew's tracy formula tracks
# the latest release (currently 0.13.x) which is incompatible.
TRACY_REQUIRED_VERSION="0.11.1"

tracy_version_ok=0
if command -v tracy-profiler >/dev/null 2>&1; then
    installed="$(tracy-profiler --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
    if [ "$installed" = "$TRACY_REQUIRED_VERSION" ]; then
        say "  - Tracy profiler ${installed} (ok)"
        tracy_version_ok=1
    else
        warn "Tracy profiler ${installed:-unknown} found — need ${TRACY_REQUIRED_VERSION}"
    fi
elif command -v tracy-capture >/dev/null 2>&1; then
    say "  - tracy-capture found (version check skipped)"
    tracy_version_ok=1
fi

if [ "$tracy_version_ok" -eq 0 ]; then
    say "  - Tracy v${TRACY_REQUIRED_VERSION} is required (matches bundled tracy-client-sys)"
    say "  - Do NOT use 'brew install tracy' — Homebrew ships a newer, incompatible version."

    if ! command -v cmake >/dev/null 2>&1; then
        warn "cmake is required to build Tracy from source"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            say "  - Installing cmake via Homebrew..."
            brew install cmake
        else
            warn "Install cmake manually (apt install cmake, dnf install cmake, etc.)"
        fi
    fi

    if command -v cmake >/dev/null 2>&1; then
        say "  - Building Tracy v${TRACY_REQUIRED_VERSION} from source..."
        TRACY_BUILD_DIR="/tmp/tracy-build"
        rm -rf "$TRACY_BUILD_DIR"
        git clone --depth 1 --branch "v${TRACY_REQUIRED_VERSION}" \
            https://github.com/wolfpld/tracy.git "$TRACY_BUILD_DIR" 2>&1 | tail -1

        cmake -B "$TRACY_BUILD_DIR/profiler/build" \
              -S "$TRACY_BUILD_DIR/profiler" \
              -DCMAKE_BUILD_TYPE=Release \
              -DCMAKE_OSX_ARCHITECTURES="$(uname -m)" \
              > /dev/null 2>&1

        cmake --build "$TRACY_BUILD_DIR/profiler/build" --parallel \
              2>&1 | tail -3

        TRACY_PROFILER_BIN="$TRACY_BUILD_DIR/profiler/build/tracy-profiler"
        if [ -x "$TRACY_PROFILER_BIN" ]; then
            say "  - Installing tracy-profiler to /usr/local/bin/ (requires sudo)..."
            sudo cp "$TRACY_PROFILER_BIN" /usr/local/bin/tracy-profiler
            say "  - Tracy profiler v${TRACY_REQUIRED_VERSION} installed"
        else
            warn "Tracy build failed — check build output above"
        fi

        rm -rf "$TRACY_BUILD_DIR"
    fi
fi

say ""
say "Copying Obsidian snippets..."
SNIPPETS_SRC="${REPO_ROOT}/tools/obsidian-snippets"
SNIPPETS_DST="${REPO_ROOT}/.obsidian/snippets"
mkdir -p "$SNIPPETS_DST"
for snippet in "$SNIPPETS_SRC"/*.css; do
    [ -f "$snippet" ] || continue
    base="$(basename "$snippet")"
    cp -f "$snippet" "$SNIPPETS_DST/$base"
    say "  - copied: ${base}"
done
say "  Enable snippets in Obsidian: Settings > Appearance > CSS snippets"

say ""
say "Building lumacore-scripts launcher..."
(cd "$REPO_ROOT" && cargo build -p lumacore-scripts \
    2>&1 | grep -E "^(   Compiling lumacore|    Finished|error)" || true)

scripts_bin="${REPO_ROOT}/target/debug/lumacore-scripts"
if [ -x "$scripts_bin" ]; then
    say "  - built: ${scripts_bin}"
else
    warn "lumacore-scripts build failed"
fi

say ""
say "Developer onboarding complete."
say "Next steps:"
say "  1. cp .env.example .env.local  (if you haven't already)"
say "  2. edit .env.local with your local companion repo paths"
say "  3. cargo build"
say "  4. cargo test --workspace"
say "  5. ./scripts/unix/start_server"

if [ "$missing_paths" -ne 0 ]; then
    exit 1
fi
