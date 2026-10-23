---
title: Developer Tools
category: docs
---
# Developer Tools

Pointers only; each tool's owner documents it.

## Utilities in `tools/`

[`tools/README.md`](../../../tools/README.md) owns them: `sacn_monitor.py` (a no-dependency
sACN multicast monitor), `sendCmd.py` (one command over the UDP command port, as a user,
through the normal rights gate) and the Obsidian snippets. The same from the Rust client:
`cargo run -p lumacore-cli -- --cmd "List Sacn"`; from inside the station the `dmx` window
(the `dmx` feed) and `DMX In <u>`.

## Scripts in `scripts/`

[`scripts/README.md`](../../../scripts/README.md) owns every script: `dev` (the plan runner),
`build`, `server start | stop | restart | status | log`, `app`, `connect.sh`, `start_server`,
`start_developer`, `stop_developer`, `test …`, `smoke_test`, `screenshots`, `fuzz_long`,
`logs`, `dev_init.sh`, `headers`, `build bench`, their Windows twins under `scripts/win/`, and
the TUI launcher (`cargo run -p lumacore-scripts`). First setup is
[getting-started.md](../../getting-started.md); what `dev_init` does is
[dev-init.md](dev-init.md).

## Test tools

Every suite and when to run it: [testing.md](../testing.md). `systemtest`
([`tests/systemtest/README.md`](../../../tests/systemtest/README.md)) rebuilds when the
registry, dispatch or the demo's sources change and writes its manifest and last run under
`target/systemtest/`.

## Everyday commands

```bash
cargo build                                                # every crate but the desktop app
cargo build -p lumacore-station --features perfetto,http   # the full server build
cargo run -p lumacore-cli                                  # terminal client (apps/lumacore-cli)
cargo run -p lumacore-app                                  # desktop app; starts target/<profile>/lumacore-server
cargo test                                                 # every crate but the desktop app
cargo fmt --all
cargo clippy --workspace --all-targets
```
