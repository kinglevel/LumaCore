# lumacore-station

The station: the one assembler. It lists every crate the program is built from and starts them;
it holds no code that does work.

## Purpose

How the program is put together (decisions §34 24). `src/catalog.rs` is the one list of every
crate's `register` — the server and everything it builds on, the protocols this station runs, the
sweep — and `install_catalog` builds, checks and installs it once per process. Each binary in
`src/bin/` parses its arguments, installs the catalog and calls one library function:

| Binary | Calls |
|---|---|
| `lumacore` | the in-process REPL (`lumacore_server::app::build_state`, `lumacore_terminal::repl::run_repl`) |
| `lumacore-server` | `lumacore_server::serve` (the desktop app starts this one) |
| `lumacore-sweep` | `lumacore_sweep::run::{long, replay}` |
| `gen-demo` | `lumacore_objects::demo::build` (the rig from `library/mvr`, `library/poolxml`, `library/demo/manual.lum`) → `library/shows/demo.luma` (gitignored; `--if-stale` builds only when missing or stale) |
| `gen-docs` | `lumacore_objects::reference` → `docs/user/reference/` |

What the program is built with is a feature choice here: `http`, `perfetto` and `tracy` pass to
the server. Like rustc's `main.rs` over `rustc_driver` and Bevy's `DefaultPlugins`: one assembler,
the work in crates.

## Owns / Doesn't own

- **Owns:** `register`, `install_catalog`; the binaries' argument parsing and start; the features.
- **Doesn't own:** any catalog row (each crate adds its own); the server, the REPL, the sweep, the
  demo show and the reference (their crates).

## Public surface

- `install_catalog()` — what every binary, systemtest and any tool that runs the whole program
  calls first.
- `register` — the station's whole list.

## Rules

- Only `src/catalog.rs` and `src/bin/<binary>.rs`. [tidy the_station_only_assembles]
- Each binary installs the catalog and calls one library function. [unchecked]
- No other crate lists other crates' `register`s for the whole program. [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-objects, lumacore-terminal, lumacore-server, lumacore-sweep, lumacore-cmd-artnet, lumacore-cmd-sacn, lumacore-cmd-osc.
- **Used by:** systemtest (its catalog).

## Decisions

- [§34](../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — the station
  is the assembler (24), the catalog (21), each protocol a crate (22)
- [§23 "Step 8"](../../docs/archive/decisions-2026-10.md) — the binaries, the app starts `lumacore-server`

## Testing

- `cargo build -p lumacore-station --features http` — every binary.
- `cargo run -p systemtest` — a whole station in process.
