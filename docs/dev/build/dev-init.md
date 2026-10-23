---
title: Developer Init
category: docs
---
# Developer Init

What the `dev_init` scripts do, step by step. First setup itself (toolchain, clone, build,
run) is [getting-started.md](../../getting-started.md); run `dev_init` once after cloning, and
again after a toolchain change.

## What It Does

The script runs these steps in order:

### 1. Check Developer Tools

Verifies that `git`, `rustc`, and `cargo` are installed and on `PATH`. Warns
(does not fail) if any are missing.

### 2. Check Environment Configuration

Reads `.env.local` (if present) and validates that each `NAME=path` entry
points to an existing directory. Relative paths are resolved against the repo
root. Missing paths produce warnings, not errors — companion repos are
optional.

If `.env.local` does not exist, the script prints setup instructions:

```bash
cp .env.example .env.local
# Edit .env.local with your local paths
```

See [Build Environment](build-environment.md) for the full `.env.local`
specification and variable reference.

### 3. Install Tracy Profiler

The Tracy client library LumaCore links is the `tracy-client` version pinned in the root
`Cargo.toml` (its `tracy-client-sys` bundles one Tracy release). The Tracy GUI **must be that
exact release** or it refuses the connection with a protocol mismatch.

The script checks for `tracy-profiler` on `PATH` and validates its version. If Tracy is
missing or the wrong version, dev_init **builds and installs the matching release from
source**:

- Clones that Tracy release to a temp directory
- Builds the profiler GUI with cmake
- Installs `tracy-profiler` to `/usr/local/bin/` (macOS/Linux, requires sudo)
  or `%LOCALAPPDATA%\Programs\tracy\` (Windows, added to user PATH)
- Cleans up the build directory

If cmake is not installed, dev_init installs it via Homebrew on macOS or
prompts you to install it manually on other platforms.

> **Do not use `brew install tracy`** — Homebrew ships whatever is newest, which is
> protocol-incompatible with the bundled client library.

See [Profiling](profiling.md) for Tracy and Perfetto usage details.

### 4. Copy Obsidian Snippets

Copies CSS snippets from `tools/obsidian-snippets/` into
`.obsidian/snippets/` so the documentation vault renders correctly. Enable
them in Obsidian under Settings → Appearance → CSS snippets.

### 5. Build lumacore-scripts Launcher

Builds the `lumacore-scripts` TUI launcher binary so developers can use
the interactive script menu:

```bash
cargo run -p lumacore-scripts   # any platform (the launcher lives in apps/lumacore-scripts)
```

## After Dev Init

```bash
cargo build                          # verify the build works
cargo test                           # every crate but the desktop app (../testing.md)
./scripts/unix/start_developer       # start the full dev environment
```

`start_developer` builds `lumacore-server` with the Tracy feature on top of the scripts'
`perfetto,http`, so profiling is enabled. See [Profiling](profiling.md) and
[Metrics & Health](metrics-and-health.md).

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `missing rustc` / `missing cargo` | Install Rust: see [Getting Started](../../getting-started.md) |
| `.env.local` path warnings | Edit `.env.local` to point to your local companion repos, or ignore if you don't have them |
| Tracy "protocol mismatch" | Build the pinned Tracy release from source (see step 3 above) |
| `lumacore-scripts build failed` | `cargo build -p lumacore-scripts` and read its error |
| dev_init exits 1 | An `.env.local` entry is missing or malformed; fix or remove it |
| Obsidian snippets not showing | Enable them in Settings → Appearance → CSS snippets |

## Script Locations

| Platform | Path |
|----------|------|
| macOS / Linux | `scripts/unix/dev_init.sh` |
| Windows | `scripts/win/dev_init.ps1` |
