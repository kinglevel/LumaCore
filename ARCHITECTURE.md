# Architecture

A map of LumaCore for someone who has to change it: what a station is, where the code lives by
crate group, and the rules that hold across groups. It is revisited when the shape changes, not
synced line by line — the crate READMEs, `docs/dev/architecture/crate-map.md` (every crate, its
edges, the data flow) and `docs/dev/architecture/code-structure.md` (how a file and a crate are
laid out) carry the detail; the reasons are the records under `docs/decisions/` (frozen log:
`docs/archive/decisions-2026-10.md`). Numbers the code owns (ports, rates, format and protocol
versions, crate counts) are not repeated here; the constant that holds each one is named instead.

## Bird's-eye view

LumaCore is a real-time DMX lighting console written in Rust. One process, the **station**,
owns a show and renders it to DMX; clients (a terminal, the web UI, the desktop app) attach to it
over sockets and never hold show state of their own.

- **One state thread.** A station's show, runtime and station settings live in one
  `ServerContext`, owned by a single thread in `lumacore-server`. Every caller — a shell, the
  UDP command port, discovery, OSC, HTTP — reaches it only through a `StateHandle` with two lanes
  (`lumacore-lanes`): the shell lane, unbounded and served first, and the bounded machine lane,
  which refuses at once when full. After each change the thread publishes an `Arc<ShowData>`
  and a small `StationView`; readers never lock.
- **The catalog.** Every declaration — commands, objects, properties, errors, flags, feeds,
  window types, menus, protocols — is a pure-data row a crate adds in its `register` at startup
  (`lumacore-registry`). The station installs the frozen catalog once per process; dispatch,
  completion, help, rights, the generated reference and the UIs all read it. No crate keeps a
  static list of another crate's rows.
- **The change path.** Dispatch fingerprints every `ShowData` subtree around a mutating command;
  exactly the changed subtrees get the undo step, the replication op and the journal entry. A
  command declares only whether it is tracked; it never writes undo or sync code.
- **The render engine.** The show snapshot goes to a plan compiler thread, which produces a
  dense `Plan`; a render thread ticks at a fixed rate (`RENDER_HZ` in `lumacore-render`) over
  the plan's arrays — masters, Solo, park, DMX input merge — and the protocol lines sample its
  frames at their own rate and only send. DMX bytes exist only in the tick.
- **LumaSession.** Several stations join one session (`lumacore-lumasession`). The master runs
  every show-changing line as the sending user and pushes the resulting subtree snapshots to the
  members in order; members forward lines through an outbox. Election, heartbeat, merge on
  rejoin and a shared session clock keep the group one show.

## Entry points

- `apps/lumacore-station` is the one assembler: `src/catalog.rs` lists every crate's `register`,
  `install_catalog()` installs it, and each binary in `src/bin/` parses its arguments and calls
  one library function. Binaries: `lumacore-server` (the station; `lumacore_server::serve` from
  a `ServerBuilder`), `lumacore` (the in-process REPL, `lumacore_terminal::repl::run_repl` over
  `lumacore_server::app::build_state`), `gen-demo` (the demo rig, `lumacore_objects::demo::build`),
  `gen-docs` (the generated reference) and `lumacore-sweep` (the command sweep).
- The web UI is one page, `crates/ui/lumacore-webui/web/index.html`, embedded by
  `lumacore-webui` and served by `lumacore-http`; the desktop app's connect page is
  `crates/ui/lumacore-webui/web/connect.html`.
- Clients: `apps/lumacore-cli` attaches over the shell port; `apps/lumacore-app` spawns a
  `lumacore-server` beside it and opens one OS window per screen on the station's page.
- A command line enters through `lumacore_objects`' dispatch whatever the transport (shell key,
  HTTP, OSC, command port, a key binding); there is no second entry.

## Code map

The crates sit in `crates/` in folders by domain; the layers are the dependency graph, which
only points down (tidy `crate_edges_point_down`). Every crate is `lumacore-<name>`, folder =
package, and every crate's README says what it owns and does not own.

### `crates/foundation/`

`lumacore-base` (values, identity, the session clock, keybindings), `lumacore-registry` (the
catalog), `lumacore-trace` (the trace macros; the `tracy` / `perfetto` features decide what
records), `lumacore-wire` (the client ↔ station contract: ports, discovery beacon, shell packets,
`Status`, the protocol numbers `SESSION_PROTOCOL` and `CLIENT_PROTOCOL`), `lumacore-syntax` and
`lumacore-syntax-derive` (the Lum lexer, parser and `#[derive(Args)]`), `lumacore-lanes` (the
state thread's handle and lanes, generic over its host) and `lumacore-sysmon` (the System
Monitor every crate logs to).

> **Architecture Invariant.** No workspace crate sits below the vocabulary crates: they depend on
> nothing in the workspace. **Architecture Invariant.** `lumacore-wire` carries no show type;
> a wire change bumps a protocol number, never grows a shim.

### `crates/model/`

The show and its machinery, below any transport: `lumacore-protocols` (the protocol contract:
line data, `Nic`, the `ProtocolDef` row, driver traits, the input merger), `lumacore-fixtures`
(fixture types, fixtures, attributes, universes), `lumacore-tree` (`Root` → `ShowData` +
`RuntimeState`, `Walk`, merge, `FORMAT_VERSION`), `lumacore-runtime`, `lumacore-playback`,
`lumacore-effects`, `lumacore-timecode`, `lumacore-tags`, `lumacore-patch`, then `lumacore-state`
(`AppState`, the change path, Oops, props, errors, dialogs, the outbox, the disk door `drive`)
and `lumacore-objects` on top of it (the registry tables, every object with its verbs, dispatch,
resolve, rights, claims, the Lum vocabulary, menus, settings).

> **Architecture Invariant.** The model crates never depend on the terminals, the server or
> LumaSession; `lumacore-state` never names a protocol, a window or a socket.
> **Architecture Invariant.** Every stored link is a GUID — never a slot number or a name — and
> every show type's `Walk` destructures exhaustively, so an unclassified field does not compile.

### `crates/formats/`

One crate per file format, with no dependency on the station: `lumacore-gdtf` (fixture
descriptions; the vendored parser is under `vendor/`), `lumacore-mvr` (the scene as a file),
`lumacore-poolxml` (a DataPool XML export, read streaming), `lumacore-luma` (the `.luma` show
file — a zip of the `ShowData` tree —, backups, media stores, station config) and
`lumacore-drives` (this station's volumes).

> **Architecture Invariant.** Older shows upgrade step by step: a format change bumps
> `FORMAT_VERSION` and ships its step (`show_file::upgrade`); a file older than `OLDEST_FORMAT`
> or newer than the build is refused.

### `crates/engine/`

`lumacore-render` (the one merge, the plan and its compiler, the render thread) and
`lumacore-engines` (the capability-gated `EngineRegistry` with the render, Perfetto and HTTP
engines behind their features).

> **Architecture Invariant.** The render tick allocates, locks, logs and looks up nothing: it
> walks the compiled plan's arrays only, and retired plans go back to the compiler thread to be
> freed. Checked by `assert_no_alloc` in debug, the tidy render regions and the budget test.
> **Architecture Invariant.** Nothing compiles or renders on the state thread; the engine never
> logs.

### `crates/network/`

Everything with a socket or a thread of its own: `lumacore-net` (NICs, the command and shell
sockets, mDNS, PTP, discovery state), `lumacore-clocksync` (the session clock's discipline), and
under `crates/network/protocols/` the line runtime `lumacore-lines` (one driver per protocol and
NIC, started from the show) beside the implementations, each declaring its own catalog row:
`lumacore-artnet`, `lumacore-sacn`, `lumacore-osc`.

> **Architecture Invariant.** The line runtime never names a protocol and the protocol crates
> never see the renderer; a protocol this build lacks is refused at show load, not guessed.
> **Architecture Invariant.** Protocols are not engines, and a line on an unmapped network never
> sends — there is no "auto" NIC fallback.

### `crates/commands/`

`lumacore-suggest` (the one suggestion engine) and, under `crates/commands/objects/`, one crate
per domain's words: `lumacore-cmd-treeshell`, `-keys`, `-tags`, `-showfile`, `-lumasession`,
`-station`, `-devices`, `-protocols`, and each protocol's object under
`crates/commands/objects/protocols/` (`lumacore-cmd-sacn`, `-artnet`, `-osc`). Each is a handler
beside its catalog row; the kernel (dispatch, resolve, the walk) stays in `lumacore-objects`.

> **Architecture Invariant.** A word means one thing across the whole catalog; nothing matches on
> keyword strings outside the registry, and no command crate keeps a list of another's words.

### `crates/ui/`

`lumacore-views` (what any UI shows: menu rows, pools, attribute board, encoder, window state —
built on objects, read by HTTP and the TUI alike), `lumacore-terminal` (one terminal per shell:
line editing, suggestions, the TUI), `lumacore-shell` (the TCP shell port), `lumacore-http` (REST,
the two WebSocket routes, static files) and `lumacore-webui` (the page, the connect page and the
kit — files only, embedded).

> **Architecture Invariant.** No client builds its own rows: every list, menu and pool is a
> server descriptor, every gesture a command line, every question a server-held dialog. The
> frontend loads nothing from the internet and polls nothing on a timer.

### `crates/server/`

`lumacore-server` (startup and stop, `ServerContext`, the state thread, the command port,
discovery, the HTTP bridge; `SHUTDOWN_DEADLINE` for the ordered stop) and `lumacore-lumasession`
(link, stream, election, merge, replication).

> **Architecture Invariant.** The model asks for LumaSession work only through its outbox and
> never calls into the session crate; a job on the state thread does no network I/O and never
> waits on the state thread.

### `crates/dev/`

`lumacore-tidy` (the source rules as tests; every rule has an entry in `docs/meta/tidy.md`),
`lumacore-headers` (the generated file banners) and `lumacore-sweep` (the command sweep, the
property tests, the fuzz entry points).

> **Architecture Invariant.** Nothing that ships depends on a dev crate, and the sweep never
> compiles the server.

### `apps/`

`lumacore-station` (the assembler above), `lumacore-cli`, `lumacore-app` and
`lumacore-scripts` (a launcher for `scripts/`).

> **Architecture Invariant.** Nothing under `apps/` but the station depends on `lumacore`; the
> clients compile only `lumacore-wire` and the WebUI's files. The station holds no code that does
> work (tidy `the_station_only_assembles`).

### `tests/`

`tests/catalog` (the whole catalog's tests: every row's rules, the Lum examples, the generated
reference, the demo golden), `tests/systemtest` (a whole station in process), `tests/fuzz`
(cargo-fuzz targets, outside the workspace) and `tests/e2e` (Playwright against real servers).

> **Architecture Invariant.** Every test runs on the rig: the demo show built from the sources
> in `library/`, copied per test — never a small show of a test's own.

## Cross-cutting concerns

**The catalog.** A crate's `register` first calls the `register` of the crates it builds on,
enters its own name, then adds rows; `freeze()` checks the whole once and holds one namespace
across commands and objects. A row names another row by its static; what is not built is
published as `Planned` and replies so, never a stub that acknowledges. Test binaries get their
crate's catalog from a test-only hook in `lib.rs`.

**Identity.** Every node under `Root` has a GUID and a name; every show object a `NodeMeta`.
Numbers are only a pool's key, names are display names, and stable nodes outside the show derive
their GUID from their name. Copies get new GUIDs, moves keep them; load refuses missing or
duplicate ones; replication, undo and merge address objects by GUID.

**The change path and Oops.** A new `ShowData` field does not compile until it has a
`ShowSubtree` variant, its restore arms and a show-file entry. Every statement of a line is its
own Oops step; station-local settings ride in a step and are put back only on their station;
running playback and the modes are never undone. Writes outside commands (load, sync, restore)
rebuild the indexes through one door.

**Rights.** One rights level per user (`UserRights` in `lumacore-tree`), a ladder from Guest to
Admin. What a line needs is declared on its row — or by the object for a verb, or by the property
for `Set` — and one gate in dispatch checks it whatever the transport. Handlers never ask who is
admin.

**Errors with codes.** Every refusal carries `E-AREA-NNN` from its crate's catalog; the codes
are rendered through `errors` and nowhere else, so a reply never stacks two "Error:" prefixes.
`Info Error <code>` reads the catalog.

**The render plan.** Two tiers: the channel tier is the patch as dense slots, rebuilt only when
the patch generation moves; the value tier is the merge — park over frozen programmers over the
LTP of programmers and running sequences over defaults — compiled per change. Fades, group
masters and effect anchors read the session clock, never wall clock; a compiler panic keeps the
last plan rather than a blackout.

**Replication.** A member forwards every line whose scope is Master; the master runs it and
`flush` sends `(epoch, seq)` subtree snapshots plus undo steps to every member, applied in
order. Another station's own settings go straight to its command port, never through the show.
There is no second sync path.

**Tree paths and the disk.** Code names a typed `Node`, never a path string; a path change shows
in `tests/catalog/testdata/tree_paths.golden.txt`. The model touches files only through `drive`,
and every command declares its reach (show, tree shell or drive), so the sweep can run with the
disk door shut.

**The demo rig as the fixture.** `lumacore_objects::demo::build` replays the station's own words
over `library/mvr`, `library/poolxml` and `library/demo/manual.lum`; the result is the show every
test, the e2e setup and the station's start use, with a count-and-hash golden in `tests/catalog`.
A line's cost on it is the change path's two fingerprints — never add a second walk of the show
per line.

**Code structure.** Every `.rs` file opens with a generated banner and a module doc naming what
it owns and does not own, then Parts in one fixed order; one folder per object, one file per
verb, tests beside their module; shared things through their one door. The rules are tests
(`cargo test -p lumacore-tidy`); the ratchets they carry only go down.
