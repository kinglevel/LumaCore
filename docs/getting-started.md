---
title: Getting Started
category: docs
---
# Getting Started

## Prerequisites

- Rust toolchain with Cargo (2024 edition)
- Git
- A C/C++ linker (platform-specific, see below)

## Platform Setup

### Windows

Install the Rust toolchain and MSVC build tools:

```powershell
winget install Rustlang.Rustup
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Verify:

```powershell
rustc --version
cargo --version
```

### macOS

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y build-essential pkg-config curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Fedora

```bash
sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Verify Rust

```bash
rustc --version
cargo --version
rustup show active-toolchain
```

## Clone and Build

```bash
git clone <your-fork-or-repo-url>
cd LumaCore
cargo build                        # every crate but the desktop app (default-members)
cargo build -p lumacore-app        # the desktop app, when you want it
```

The packages you run:

| Package | Binary | What |
|---|---|---|
| `lumacore-station` | `lumacore-server` | The server. Add `--features http` for the web UI (`perfetto,http` is the full build) |
| `lumacore-cli` | `lumacore-cli` | Terminal client |
| `lumacore-app` | `lumacore-app` | Desktop app (starts `lumacore-server` next to it) |
| `lumacore-scripts` | `lumacore-scripts` | TUI launcher for the scripts |

The scripts build the `local` profile (`target/local/`: release speed, incremental); plain
cargo builds `target/debug/` unless you add `--release`.

## Developer Init

Run the onboarding helper to check your tools and the paths in `.env.local`
(what it does, step by step: [dev/build/dev-init.md](dev/build/dev-init.md)).

**Windows (PowerShell):**

```powershell
.\scripts\win\dev_init.ps1
```

**macOS / Linux:**

```bash
./scripts/unix/dev_init.sh
```

## Run LumaCore

Recommended local developer flow (macOS / Linux):

```bash
./scripts/unix/start_developer
```

This builds `lumacore-server` (features `perfetto,http`, plus `tracy`) and `lumacore-cli`,
restarts the server with the `demo` show, and opens new terminal windows for the CLI (as
`Admin`), the Meownitor (sysmon viewer) and Tracy if installed.
`./scripts/unix/stop_developer` stops them all.

Just the server and one client in this terminal:

```bash
./scripts/unix/start_server --show demo
```

The web UI is on the HTTP port (`REMOTE_HTTP_PORT` in `lumacore-wire`; `server status` prints
it): open `http://127.0.0.1:<port>`.

### TUI Launcher

A menu for all scripts:

```bash
cargo run -p lumacore-scripts
```

### Other entrypoints

| Action        | macOS / Linux                    |
|---------------|----------------------------------|
| Start server + CLI | `./scripts/unix/start_server [--show demo] [--user LD] [--debug]` |
| Stop server   | `./scripts/unix/stop_server`     |
| Stop developer| `./scripts/unix/stop_developer`  |
| Smoke test    | `./scripts/unix/smoke_test`      |

See [scripts/README.md](../scripts/README.md) for every script, including the Windows ones.

Using Cargo directly (any platform):

```bash
cargo run -p lumacore-station --bin lumacore-server --features http -- --show demo
cargo run -p lumacore-cli -- --target 127.0.0.1 --user Admin
cargo run -p lumacore-cli -- --target 127.0.0.1 --user LD
cargo run -p lumacore-cli -- --meownitor          # sysmon viewer
cargo run -p lumacore-cli -- --cmd "List Health"  # one command, then exit
cargo run -p lumacore-app                          # desktop app
```

The desktop app starts `target/<profile>/lumacore-server`, so build that first with
`--features http`.

## Multi-User Quick Start

Terminal 1 — start the server:

```bash
cargo run -p lumacore-station --bin lumacore-server --features http -- --show demo
```

Terminal 2 — connect as Admin:

```bash
cargo run -p lumacore-cli -- --user Admin
```

Terminal 3 — connect as LD:

```bash
cargo run -p lumacore-cli -- --user LD
```

Two servers on one machine (for LumaSession tests):

```bash
cargo run -p lumacore-station --bin lumacore-server --features http -- --station B --port-base 48660
```

## Verify Your Environment

```bash
cargo test
cargo fmt --all --check
cargo clippy --workspace --all-targets
```

If all three pass, you're good. See [dev/testing.md](dev/testing.md) for every suite, what to
run per step and the gate (Playwright e2e, the catalog's tests, tidy, the long run).

## Build Environment

External companion repos (reference repos, research notes) are optional and configured per
developer in `.env.local` — gitignored, copied from `.env.example`:

```bash
cp .env.example .env.local
# Edit .env.local with your local paths (absolute, or relative to the repo root)
```

The variable (`SKILL`), the link rules and what `dev_init` checks are in
[build-environment.md](dev/build/build-environment.md). LumaCore builds and runs without them.
