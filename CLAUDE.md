# LumaCore

Real-time DMX lighting console written in Rust. This file holds what every change needs: the commands, the invariants, and where everything else lives. Area rules load on demand from `.claude/rules/`; procedures are skills; decisions are records under `docs/decisions/`; the architecture is `ARCHITECTURE.md`.

## Build

```bash
cargo build                                                 # every crate but the desktop app (default-members; no HTTP, no perfetto)
cargo build -p lumacore-station --features perfetto,http    # the full server
cargo build -p lumacore-http                                # the HTTP engine crate only
cargo build -p lumacore-cli -p lumacore-app                 # the clients (wire only; the app starts target/<profile>/lumacore-server)
./scripts/unix/build bench [local|release|dev]              # a fresh timed build → a row in docs/dev/build-times.md
```

The scripts build `[profile.local]` (release + incremental); `--ship` is the full release, `--debug` the dev profile. The git commit is stamped only by the station: never add a `build.rs` or `env!` stamp lower in the graph. `vendor/` holds third-party crates LumaCore changed, pinned with `[patch.crates-io]`, never workspace members; every change is marked `// LumaCore:` and listed in `vendor/README.md`. Build servers and profiles: the `build-bench` skill.

## Test

```bash
cargo test                                   # every crate but the desktop app (--workspace for all)
cargo test -p lumacore-tidy                  # the rulebook, the ratchets and the docs (runs scripts/unix/docs_check)
cargo test -p lumacore-catalog-tests         # the reference, the goldens, the lum examples, the demo show
cd tests/e2e && npx playwright test          # the UI end to end; tests/e2e/README.md says which file drives what
scripts/unix/docs_check --index              # rebuild the decisions index after a record change (alone: the docs findings, no build)
```

While building a step run only what it touches; the gate (full e2e, tidy, catalog tests) runs once per step before the docs and the commit. Fuzzing and the command sweep are a long run only, by hand. Details: the `step-tests` skill. Where a test goes: a line needing words from more than one command crate → `tests/catalog`; the kernel's machinery → its stand-in words (`commands/test_words.rs`); else beside its code in `foo/tests.rs`.

| Flag | Effect |
|---|---|
| `http` | Enables `lumacore-http` (axum, WebSocket, REST, the embedded WebUI) |
| `perfetto` | Enables the Perfetto trace engine |
| `tracy` | Tracy profiler integration |

## Layout

Crates live under `crates/` in domain folders (`foundation`, `model`, `formats`, `engine`, `network`, `commands`, `ui`, `server`, `dev`), every one named `lumacore-<category>`; `apps/` holds the station (the one assembler: catalog + thin binaries) and the clients; `tests/` holds systemtest, catalog, fuzz and e2e. The map is `ARCHITECTURE.md`; the per-crate routing table is `docs/dev/architecture/crate-map.md`; the file rules are `docs/dev/architecture/code-structure.md` and `docs/meta/tidy.md`. Code that reads files beside the crates uses `env!("LUMACORE_ROOT")`, never `../..`.

## Invariants

Each names its door and the check that catches a violation. Area rules refine them; none weakens them.

1. **The catalog.** Every declaration is a pure-data row a crate adds in its `register`; readers use `lumacore_registry::get()`; never a static list of another crate's rows. Check: `freeze()`.
2. **One state thread, two lanes.** Everything reaches `ServerContext` through `StateHandle`; never `StateHandle::call` inside a job, never network I/O inside a job (queue an outbox request), never compile or render on the state thread. Check: E-INTERNAL-001, the lanes in `lumacore-lanes`.
3. **The change path is automatic.** Dispatch fingerprints every `ShowData` subtree around a mutating line; a command only declares `undo:`. A new `ShowData` field does not compile until it has a `ShowSubtree` variant, its `oops.rs` arms and a `show_file` entry; never a second walk of the show per line. Check: the compiler.
4. **Strict identity.** Every node has an `Identity`, every show object a `NodeMeta`; every stored link is a GUID, never a slot number or a name; every show type implements `Walk` exhaustively; copies get new GUIDs, moves keep them. Check: E-SHOW-003, the compiler.
5. **Errors have codes.** Every refusal carries `E-AREA-NNN` from its crate's catalog, rendered through `errors::render` / `errors::reply`; never `text::render_error` outside `errors`. Check: tidy `error_replies_carry_a_code`, the catalog test.
6. **One resolver, one property registry, one namespace.** Targets become objects only through `core::resolve`; properties are declared once on `ObjectDef::props` and `Set` is one typed handler; every name means one thing across commands and objects; never a keyword match outside the registry; new handlers are typed (`#[derive(Args)]`). Check: tidy `old_resolvers_only_go_down`, the keyword rule, `the_shim_count_only_goes_down`.
7. **Tree paths are typed.** Code names a `Node`, never a `"Root.…"` string; `cd` / `enter` / `ls` go through `AppState::resolve_target`. Check: tidy `tree_paths_are_typed`, `tree_paths.golden.txt`.
8. **Older shows upgrade step by step.** Bump `FORMAT_VERSION` on any change and ship its step in `show_file::upgrade::STEPS` (an empty one when nothing saved changed shape); a show older than `OLDEST_FORMAT` or newer than the build is refused; read and write only through `show_file`. Check: `every_format_from_the_oldest_has_its_step`, E-SHOW-001 / 002 / 004.
9. **Every test runs on the rig.** `demo::fresh_state()` gives a copy of the demo show; a test names the rig's facts or picks numbers past it, never a small show of its own. Check: `the_demo_show_builds_loads_and_matches_its_golden`.
10. **The render tick allocates, locks, logs and looks up nothing.** The tick and the lines' send walk the compiled `Plan`'s arrays; DMX bytes exist only in the tick; plans are compiled on the compiler thread. Check: `assert_no_alloc`, the tidy render regions, the budget test.
11. **One rights gate.** Rights are declared on the row (`CommandDef.rights`, `ObjectDef::rights`, the property's level); dispatch checks every statement before any runs; never a per-transport or `is_admin()` check in a handler. Check: E-RIGHTS-001.
12. **Code structure.** Headers only from `scripts/unix/headers`, Parts in the one order, one folder per object and one file per verb, no `mod.rs`, files under the limit, shared things through their one door, old files follow when their area is touched, ratchets only go down. Check: `cargo test -p lumacore-tidy`.
13. **Edges point down.** The station only assembles; clients compile only `lumacore-wire` and the WebUI's files; the model crates never depend on the terminals; any wire change bumps `SESSION_PROTOCOL` / `CLIENT_PROTOCOL`, no shims. Check: tidy `crate_edges_point_down`, `the_station_only_assembles`.
14. **The disk has one door.** `core` touches files only through `drive`; every `CommandDef` declares its `reach`; a filename on a line goes through `drive::library_file`. Check: tidy `core_reaches_the_disk_only_through_drive`.
15. **The language is `docs/user/syntax/lum.md`.** Its grammar block is the language and every example is a test; any change to the syntax is a hard discussion first, never on the side of another task; a word that is not built is `Planned` and replies so, never a stub that acknowledges. Check: the catalog's lum tests, `planned_words_reply_planned_and_change_nothing`.

Two more that are habits rather than doors: every question the station asks is one server-held dialog (never a client-only confirm); every UI gesture is a command line (never a client-only state or a pool-specific endpoint).

## Where things go

- A fact every change needs → this file, only if it is non-obvious from the code, has been hit repeatedly, is specific enough to act on, and names its door or check. This file stays under 150 lines.
- A rule for one area → `.claude/rules/<area>.md`, loaded when a matching file is touched.
- A decision and its reasons → a record in `docs/decisions/` (the `decision-record` skill); a reversal is a new record that supersedes the old one.
- A procedure → a skill under `.claude/skills/`.
- Status and progress → the plan in `docs/plans/active/` while open, `CHANGELOG.md` when built; `docs/meta/roadmap.md` is one line per open item with a link.
- A measurement → `docs/dev/benchmarks.md` or `docs/dev/build-times.md`, never a rule.
- Research → `docs/research/<topic>/`; a design for something not built → `visions/`.
- A value the code owns (format version, protocol numbers, ports, counts, durations) → not in prose; name where the code holds it.
- Nothing is deleted: superseded docs move to `docs/archive/` with a row in its README. The decisions log before the records is `docs/archive/decisions-2026-10.md`; `§N` cites it.

## Routing

| Area | Rules (load on demand) | Covers |
|---|---|---|
| Render and lines | `.claude/rules/render.md` | the tick, the plan, the merge, cue timing, masters, the session clock, health, panics |
| Protocols | `.claude/rules/protocols.md` | each protocol declares itself, a protocol is an object and a line one row, networks and NICs, DMX input |
| Patch | `.claude/rules/patch.md` | the Devices draft and live columns, a fixture's ids, folders, Import MVR, the MVR tree words |
| Commands | `.claude/rules/commands.md` | rows and objects, owner-first addressing, quoting, the range rule, default object, Oops, selections, groups, programmers, modes, reach |
| Show file | `.claude/rules/showfile.md` | the `.luma` zip, stores, backup and recovery, drives, Import Show |
| UI kit | `.claude/rules/ui-kit.md` | kit elements only, the pointer core, grids and slabs, dialogs, menus, window types, settings, feeds, phone, keys on screen |
| LumaSession | `.claude/rules/lumasession.md` | ports and services, replication, remote edits, merge on rejoin, elections, claims, shutdown, app and local server |
| Keys | `.claude/rules/keys.md` | UiKeys and gestures, core keys, Meow |

Skills: `step-tests`, `headers`, `regen-reference`, `build-bench`, `decision-record`, `doc-garden`. Docs: `ARCHITECTURE.md` · `CONTRIBUTING.md` · `docs/decisions/README.md` (the index) · `docs/meta/roadmap.md` · `CHANGELOG.md` · `docs/dev/testing.md` · `docs/meta/tidy.md` · `docs/user/reference/` (generated by `gen-docs`; never hand-write a syntax table). User guides under `docs/user/`, developer docs under `docs/dev/`, the index `docs/README.md`.

## Remote architecture in one breath

LumaSession is the station-to-station side (never "network session"); "Network" means only a data network. Services: the UDP command port, the TCP shell port, the discovery beacon, HTTP with `/ws/feeds` (everything live in a page, one socket, no SSE, no poll) and `/ws/shell`, the LumaSession stream, Art-Net and sACN on their standard ports, OSC per line. The port numbers are constants in `lumacore-wire`. Only the master changes `ShowData`; members forward lines and apply results in order; station-local state is never replicated. Two servers on one machine: `lumacore-server --station B --port-base <n>`. Everything else: the LumaSession rules file and `docs/dev/networking/lumasession.md`.
