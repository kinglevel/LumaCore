---
title: Scripts
category: docs
---
# Scripts

Helper scripts for local development, by platform:

```
scripts/
  unix/          # Bash scripts for macOS and Linux
  win/           # PowerShell scripts for Windows
```

The packages they build: `lumacore-station` (binaries `lumacore-server`, `lumacore-sweep`,
`gen-demo`, `gen-docs`), `lumacore-cli`, `lumacore-app` and `lumacore-scripts` — the clients
are their own packages under `apps/` (`cargo build -p lumacore-cli`). First setup (toolchain,
clone, `.env.local`) is [docs/getting-started.md](../docs/getting-started.md).

## Unix (`scripts/unix/`)

Every script takes `-h`. The default build is the `local` profile (`target/local/`: release
speed, incremental — decisions §34 "Build times hardened" 2); `--ship` builds full release
(`target/release/`, the smoke test's), `--debug` a debug build (`target/debug/`). Builds run on all
cores minus 2 (`.cargo/config.toml`), and each ends with one line: what was built, when it
finished, how long, how many crates. The station is always built with one feature set,
`perfetto,http` (`lib.sh`; `--tracy` adds Tracy), so switching scripts never
leaves a server without the web UI.

Every step prints a timestamped line, and its full output (cargo's progress bar,
each test as it finishes, warnings, errors) shows live. The same output goes, at
the same moment, to `target/script-logs/<script>_<time>_<pid>.log` (`latest.log`
is the newest run; the last 100 are kept), so a run can be watched from another
terminal with `logs`. Read-only commands (`logs`, `server status`, `connect.sh`,
`test fuzz-report`) write no log. The server's own output goes to
`target/<profile>/server.log`.

**Everyday**

| Script | What it does |
|---|---|
| `dev [--stop] [--build server,cli,app] [--tracy] [--run server,tracy,meownitor,web,app,cli-window,cli] [--show x] [--user x] [--clean-start] [--debug | --ship]` | The plan runner (the launcher's Plan pane writes this line): stop everything, build what's named, then run what's named in that order — server (restarted with the web UI, waits for health), Tracy, Meownitor window, web UI in the browser, app (background), CLI window, CLI here last. The first failure stops it; a binary a run step needs is built when missing; without a display the window steps are skipped (`cli-window` becomes `cli`). |
| `build [server\|cli\|app\|all] [--debug \| --ship] [--tracy]` | Build only; nothing is started. `app` is the app alone: it connects to any station, and needs the server built only to start a local one. |
| `build bench [local\|release\|dev] [--yes]` | A fresh, timed build to compare machines: asks, wipes that profile's target dir, builds server + CLI + app (every crate, dependencies too) and appends a row (date, machine, rustc, profile, jobs, crates, time, commit) to `docs/dev/build-times.md`. |
| `server start [--show demo] [--debug | --ship] [--no-build] [--fg] [--clean-start]` | Build, stop a running server, start it in the background (output in `target/<profile>/server.log`), wait for health. `--fg` runs it in this terminal. Every script-started server gets `--web-root crates/ui/lumacore-webui/web` (`lib.sh` `WEB_ROOT`, `LC_WEB_ROOT` overrides; live edits); without it the server serves the WebUI built into its binary. |
| `server stop [--force]` · `restart` · `status` · `log` | Stop cleanly (SIGTERM: autosave + ordered shutdown, up to 8 s; `--force` kills after that) · start again · PID, binary, uptime, health, HTTP + web UI, ports · follow the log. |
| `app [--debug | --ship] [--no-build] [--bg]` | Build and start the desktop app; its connect page reaches any station, "local" starts the server next to it or attaches to the running one. `--bg` starts it in the background (`target/<profile>/app.log`). Refuses with a hint when there is no display (SSH, console). |
| `connect.sh [--user x] [--target ip] [--cmd "…"] [--debug | --ship]` | Attach `lumacore-cli` to a running server; never starts or restarts one. |
| `start_server [--show x] [--user x] [--cmd "…"] [--debug | --ship] [--clean-start]` | Preset of `dev --build server,cli --run server,cli`: start the server, then attach the CLI here (or run one `--cmd`). |
| `stop_server [--force]` | Same as `server stop`. |

**Tests**

| Script | What it does |
|---|---|
| `test [quick]` | `cargo test`, every crate. |
| `test crate <name> [filter]` | `cargo test -p <name> [filter]`. |
| `test e2e [playwright args]` | UI end-to-end in `tests/e2e` (installs `node_modules` the first time). |
| `test smoke` | Same as `smoke_test`. |
| `test all` | quick + e2e + smoke, then a summary with each part's result and time. |
| `test regen` | `gen-docs` + `gen-demo` after a registry or demo show change. |
| `test screenshots` | Same as `screenshots`. |
| `test fuzz [minutes] [--fresh] [--no-fuzz]` | The long run (`fuzz_long`); without nightly + cargo-fuzz it adds `--no-fuzz` and says so. A progress line at least every 5 s: cases done / total, rate, time left, findings, the line it ran last (libFuzzer's status line per fuzz target). |
| `test fuzz-report` · `test replay <file>` | Print `target/fuzz-report/summary.md` · replay a finding from `target/fuzz-report/replays/`. |
| `smoke_test` | Builds full release binaries, starts the server with the demo show and `--RootMonitor`, waits for health, checks the startup sysmon lines, shuts it down and checks the shutdown line. Refuses while another server runs. |
| `screenshots` | Every window type, menu and overlay into `target/screenshots/` (Playwright). |
| `fuzz_long [minutes] [--fresh] [--no-fuzz]` | The overnight long run: command sweep, random sequences, every cargo-fuzz target, property tests with a random seed. Resumable; reports in `target/fuzz-report/`. Never part of per-step tests. |

**Watching a run** (from a second terminal; Ctrl-C stops watching, not the run)

| Script | What it does |
|---|---|
| `logs` | Follow the newest script run (`latest.log`); moves on when the next run starts. |
| `logs fuzz` · `logs server` | The newest fuzz run (also linked from `target/fuzz-report/logs/`) · the local server's output. |
| `logs list` · `logs <file>` | The recent runs, newest first · one log. |

**Developer and setup**

| Script | What it does |
|---|---|
| `start_developer [--show demo] [--user Admin] [--debug | --ship] [--clean-start]` | Preset of `dev --build server,cli --tracy --run server,tracy,meownitor,cli-window --show demo`: builds the server (+ Tracy) and the CLI, restarts the server with the show, opens terminal windows for the CLI, the Meownitor and Tracy (if installed). Without a display it attaches the CLI in this terminal. |
| `stop_developer` | Stops everything the dev plans start (server first, cleanly, then CLI, Meownitor, Tracy, app); leaves the `lumacore-scripts` launcher running. |
| `dev_init.sh` | Onboarding: checks tools, reads and validates `.env.local` (gitignored), copies the Obsidian CSS snippets, builds the `lumacore-scripts` launcher ([docs/dev/build/dev-init.md](../docs/dev/build/dev-init.md)). |
| `headers` | Renders a file's ANSI Shadow banner, a Part header or a Section box (`lumacore-headers`); tidy re-renders them. |
| `docs_check` | The docs' gardening checks and the decisions index (`python3 -I scripts/unix/docs_check`). |
| `lib.sh` | Shared helpers (paths, web root, profile, feature set, build, server PIDs, health, start, stop, terminal windows, browser). Sourced, not run. |

## Windows (`scripts/win/`)

`dev.ps1`, `start_developer.ps1`, `start_server.ps1`, `connect.ps1`, `stop_server.ps1`,
`stop_developer.ps1`, `dev_init.ps1` — the same jobs as their Unix twins, with
PowerShell parameters (`-Build server,cli`, `-Run server,cli`, `-Stop`, `-Tracy`,
`-Show`, `-User`, `-Cmd`, `-CleanStart`, `-DebugBuild`); `start_server.ps1` and
`start_developer.ps1` are presets of `dev.ps1`. Not on Windows yet: `build`,
`server`, `app`, `test`, `smoke_test`, `fuzz_long` (the launcher says so).

## TUI launcher

The `lumacore-scripts` binary has three panes; it picks the platform's folder by
itself. Tab moves between the Plan and the Scripts.

- **Plan** — checkboxes in a Build and a Run column: stop first, server, tracy
  (build = the server feature, run = the profiler), meownitor, web UI, app, cli
  (Space cycles off → here → window). Enter (or `r` / F5 anywhere) runs them as
  one `dev` line; the footer shows it. `n` clears the ticks.
- **Status** (every 2 s, all local): the server process(es) and health
  (`lumacore-cli --health`), `GET /api/station` (name, role, uptime, show,
  unsaved, busy, logged-in users), the running build against git HEAD and
  whether its binary was rebuilt since it started, what `/` serves (the
  WebUI, or nothing from an older server), the ports (cmd, shell, http,
  perfetto, stream), the app / CLI / Meownitor processes, and the binaries of
  the chosen profile with their age.
- **Scripts** — every script above, grouped Build / Server / App / Test / Fuzz /
  Logs / Developer / Setup; Enter runs the highlighted one.

The options apply to both: `d` debug / local, `s` show, `u` CLI user, `c`
clean start. The ticks and options are kept in `target/lumacore-scripts.json`.
Ctrl-C stops the running script, not the menu, and every run waits for Enter so
its output stays readable.

```bash
cargo run -p lumacore-scripts
```
