# lumacore-sweep

The command sweep and the fuzz entry points: every command, keyword, flag and writable property
run in process as every rights level, then random lines grown from completion, with checks after
every line — all generated from the registries at run time.

## Purpose

The overnight hardening run (decisions §25 "Step 10 hardening"). `World` is a master and an
in-process member with no server, no engine threads, no sockets and the disk shut; `World::run`
runs one line as one user and checks what must hold after it (no panic, coded errors, undo puts
the show back, replication gives the member the same show, save → load round trips, Show lines
never reach the disk, slow lines). `Vocab` reads what can be said from the catalog; `lines` builds
the exhaustive plan and the random lines; `run` is the resumable overnight run with its report in
`target/fuzz-report/`. `fuzzing` holds the entry points of the cargo-fuzz targets in `tests/fuzz/`,
so a fuzz build compiles the sweep, never the server. It sits on objects, LumaSession and a
protocol only (decisions §34 20.10, 24.2).

## Owns / Doesn't own

- **Owns:** `World`, `Finding`, `Outcome`, `Expect`, `ROLES` (`world`); `Vocab` (`vocab`); the
  exhaustive plan and the random generator (`lines`); the overnight run, resume, watchdog,
  replay, the report (`run`); the fuzz entry points (`fuzzing`); the property tests (`tests.rs`).
- **Doesn't own:** the `lumacore-sweep` binary and the whole catalog it runs with (the station,
  `apps/lumacore-station/src/bin/lumacore-sweep.rs`, decisions §34 24.1); the fuzz targets
  (`tests/fuzz/fuzz_targets/`); `scripts/unix/fuzz_long`; what it checks (the change path, Oops,
  replication, show files — their crates).

## Public surface

- `run::{long, replay, Options}` — what the binary calls.
- `world::World`, `lines::{exhaustive, random_line, Rng}`, `vocab::Vocab` — the world and the
  generator.
- `fuzzing::{osc, stream_frame, command_line, token, commands}` — the fuzz targets' entry points.
- `register` — the crates it builds on (the property tests' and fuzz targets' catalog).

## Rules

- Never a hand-written command list: every word, flag and property comes from the catalog. [unchecked]
- It never installs the station's catalog: the binary does; a fuzz target installs the sweep's own. [unchecked]
- It never depends on the server. [tidy crate_edges_point_down]
- Never a per-step test: the overnight run is started by hand; only the property tests are ordinary `cargo test`s. [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-tree, lumacore-objects, lumacore-lumasession, lumacore-osc, lumacore-cmd-osc, lumacore-cmd-sacn, lumacore-cmd-artnet, lumacore-sysmon, lumacore-luma, lumacore-base, lumacore-suggest, lumacore-cmd-treeshell, lumacore-cmd-keys, lumacore-cmd-tags, lumacore-cmd-showfile, lumacore-cmd-station, lumacore-cmd-devices, lumacore-cmd-protocols.
- **Used by:** lumacore-station (the `lumacore-sweep` binary), tests/fuzz.

## Decisions

- [§25 "Step 10 hardening", "Step 10 built"](../../../docs/archive/decisions-2026-10.md) — the sweep, the
  fuzz targets, the property tests
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (20.10, 24.2), its README (18)

## Testing

- `cargo test -p lumacore-sweep` — the property tests (fixed seeds, seconds).
- The overnight run: `./scripts/unix/fuzz_long [minutes] [--fresh] [--no-fuzz]` (by hand only).
