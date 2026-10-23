---
title: Crate Map
category: docs
---
# Crate Map

The routing table of the Rust workspace: which crate owns what and what to read first.
**If you are working on X, this tells you which crate owns it.** The bird's-eye view is the
root `ARCHITECTURE.md`; the file and crate rules are [code-structure.md](code-structure.md).

What the code owns is not repeated here: the members are the root `Cargo.toml`
(`[workspace] members`, `default-members` = everything but the desktop app), each crate's
dependencies are its own `Cargo.toml`, the allowed edges are `EDGES` in `lumacore-tidy`
(`crate_edges_point_down`), the folders are its `GROUPS` (`crates_sit_in_their_group`), and
every crate with a `README.md` says there what it owns, doesn't own and its edges. Every crate
is `lumacore-<name>` under `crates/<group>/`; the station is `apps/lumacore-station`.

## Folders

Crates live in folders by what they are about (archive §34 23). The layers are the
dependency direction, not the folders.

```
crates/
├── foundation/   lumacore-base · -registry · -sysmon · -trace · -wire · -syntax · -syntax-derive · -lanes
├── model/        lumacore-runtime · -protocols · -fixtures · -tree · -playback · -effects · -timecode
│                 · -tags · -patch · -state · -objects
├── formats/      lumacore-gdtf · -luma · -mvr · -poolxml · -drives
├── engine/       lumacore-render · lumacore-engines
├── network/      lumacore-net · lumacore-clocksync          (everything with sockets or threads)
│   └── protocols/                                       the implementations: runtime, plugins by Kind
│       ├── lumacore-lines
│       ├── dmx/      lumacore-artnet · lumacore-sacn
│       └── data/     lumacore-osc
├── commands/     lumacore-suggest
│   └── objects/  lumacore-cmd-treeshell · -keys · -tags · -showfile · -lumasession · -station · -devices · -protocols
│       └── protocols/    each protocol's object, mirroring network/protocols
│           ├── dmx/      lumacore-cmd-sacn · lumacore-cmd-artnet
│           └── data/     lumacore-cmd-osc
├── ui/           lumacore-views · -terminal · -shell · -http · -webui
├── server/       lumacore-server · lumacore-lumasession
└── dev/          lumacore-tidy · -headers · -sweep        (never a product dependency)
apps/             lumacore-station (the assembler) · lumacore-app · lumacore-cli · lumacore-scripts
tests/            systemtest · catalog · fuzz · e2e
vendor/           third-party crates LumaCore changed (`vendor/README.md`); path dependencies, never members
```

A new crate goes into the folder of what it is about; a new protocol into
`network/protocols/dmx/` or `data/` by its row's kind, its object into
`commands/objects/protocols/` beside it. Code that reads files beside the crates uses
`env!("LUMACORE_ROOT")` (`.cargo/config.toml`), never `../..`.

## The direction

Bottom to top, each layer only on the ones below it (checked by `crate_edges_point_down`):

1. **Vocabulary** — `lumacore-base`, `-wire`, `-trace`, `-headers`, `-syntax-derive`: no workspace crate below them.
2. **The catalog and the data** — `lumacore-registry`; `-syntax`; the data crates `-playback`, `-effects`, `-timecode`, `-runtime`, `-tags`, `-fixtures`, `-protocols`; `lumacore-tree` (ShowData, Root) on them.
3. **Formats and I/O below the kernel** — `-gdtf`, `-luma`, `-mvr`, `-poolxml`, `-drives`, `-patch`, `-net`, `-clocksync`, `-render`, the protocol crates.
4. **The kernel** — `lumacore-state` (AppState, change path, Oops, errors, dialogs, the outbox) and `lumacore-objects` on it (the registry tables, every object and verb, dispatch, resolve, rights, claims, lum, menus, settings, suggestions). The objects crate re-exports the state's modules, so callers write `lumacore_objects::…`; inside the two crates, `crate::…`.
5. **Commands, views, runtimes** — `lumacore-cmd-*`, `-suggest`, `-views`, `-lines`, `-engines`.
6. **Terminals, HTTP, LumaSession, the server** — `-terminal`, `-shell`, `-http`, `-lumasession`, `-server`.
7. **The station** — `apps/lumacore-station`: `src/catalog.rs` + a thin main per binary, nothing else (tidy `the_station_only_assembles`).

Clients (`apps/lumacore-cli`, `-app`) compile only `lumacore-wire` and the WebUI's files.
The kernel never imports a terminal, LumaSession or the server: it asks for LumaSession work
through its outbox (`session_link`).

## Crates by group

Links to a crate's `README.md` where it has one; otherwise the doc that covers it.

### foundation

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-base` | Leaf: `Value`, identity (`Guid`, `NodeMeta`), `IdPool`, undo (`UndoStack`, `ShowSubtree`), keybindings, variables, replication types, error codes (`Coded`), `fingerprint`, the session clock (`clock`: a rate overlay on `Instant`) | [object-tree.md](../../user/object-tree/object-tree.md) |
| `lumacore-registry` | The catalog: pure-data rows every crate adds in its `register`, `freeze()` once, `get()` everywhere | [README](../../../crates/foundation/lumacore-registry/README.md) |
| `lumacore-sysmon` | The System Monitor: the ring, levels, subscribers, the log file, feedback events, panic reports | [README](../../../crates/foundation/lumacore-sysmon/README.md), [server-diagnostics.md](../build/server-diagnostics.md) |
| `lumacore-trace` | Every trace macro (`trace_zone!`, `held_zone!`, `tracy_plot!` …), the Perfetto collector and its JSON; its `tracy` / `perfetto` features decide what records | [README](../../../crates/foundation/lumacore-trace/README.md), [profiling.md](../build/profiling.md) |
| `lumacore-wire` | The client ↔ station contract: ports, discovery groups, `SESSION_PROTOCOL` / `CLIENT_PROTOCOL`, beacon, command-port request / reply, shell packets, key codes, `UiEvent`, `Status`. serde + socket2 only | `src/lib.rs` |
| `lumacore-syntax` | Lum: lexer, total parser, printer, cursor, highlighting, `bind` to signatures; `GRAMMAR` must equal lum.md's grammar block | [lum.md](../../user/syntax/lum.md) |
| `lumacore-syntax-derive` | `#[derive(Args)]`: a typed command's struct is its signature (proc-macro) | — |
| `lumacore-lanes` | A state thread's two job lanes: `StateHandle` (`call`, `call_within`, `send`), the shell and machine lanes, the reentrant refusal, the busy watchdog; generic over the `Host` | [README](../../../crates/foundation/lumacore-lanes/README.md) |

### model

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-runtime` | `Root.Runtime` (never saved; the session-wide parts replicate): logged-in users' modes and undo, LumaSessions, network runtime, claims, `Journal`, `HealthBoard` (`RenderStats`, `LineStats`, `ClockSync`, `ThreadHealth`) | [metrics-and-health.md](../build/metrics-and-health.md), [lumasession.md](../networking/lumasession.md) |
| `lumacore-protocols` | The protocol contract, below the tree: `Line` (one protocol row with its run of universes), `Network`, `Nic`, the `ProtocolDef` catalog row, `DmxDriver` / `ControlListener`, the input merger, `LinePlan`, the E-DMX codes | [README](../../../crates/model/lumacore-protocols/README.md) |
| `lumacore-fixtures` | Fixture types, fixtures (`Fixture.Mvr`, `Fixture.Luma`, the `Slot` key, `FidIndex`), attributes, universes — the patch's data below the tree | [README](../../../crates/model/lumacore-fixtures/README.md), [fixtures-patch-output.md](../../user/fixtures/fixtures-patch-output.md) |
| `lumacore-tree` | The hub: `Root`, `ShowData`, `ShowSettings` (`FORMAT_VERSION`), `StationSettings`, `User` (owns its programmer and selection), `UserProfile`, `UserRights`, `UiKey`, `Walk` + `ShowData::identity()`, the 3-way `merge`, `media`; re-exports the data crates | [lumacore.md](../../user/identity/lumacore.md), [objects.md](../../user/reference/objects.md) (generated), [show-files.md](../../user/show-files/show-files.md) |
| `lumacore-playback` | Sequences, cues, parts, `CueValue`, executors, masters, timing / curve presets, `PlaybackStates` (`ShowData.Playbacks`) | [sequences.md](../../user/playback/sequences.md), [control.md](../../user/object-tree/control.md) |
| `lumacore-effects` | Effect engine configuration (authored data only; no runtime) | [visions/effects.md](../../../visions/effects.md) |
| `lumacore-timecode` | Timeline tracks, clips, regions, markers, tempo (data only) | [visions/timelines.md](../../../visions/timelines.md) |
| `lumacore-tags` | The tag tree: tags with child tags, paths, moves, the member index | [README](../../../crates/model/lumacore-tags/README.md) |
| `lumacore-patch` | Conflict-aware patching: address overlaps, footprints, universe capacity (`PatchEngine`, stateless) | [fixtures-patch-output.md](../../user/fixtures/fixtures-patch-output.md) |
| `lumacore-state` | `AppState`, the change path (`change.rs`), Oops, `props` types, errors, dialogs, the outbox (`session_link`), `patch_draft`, the registries' types | `src/lib.rs` |
| `lumacore-objects` | The registry tables and every object and verb (one folder per object, one file per verb), dispatch, `resolve`, rights, claims, `lum`, menus, settings, `demo::build` (the rig) | [lum.md](../../user/syntax/lum.md), [commands.md](../../user/reference/commands.md) (generated) |

### formats

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-gdtf` | GDTF import (the derived `FixtureType`), DIN SPEC 15800 validation (warn only), channel resolution; the vendored `gdtf` reader is `vendor/gdtf` | [fixtures-patch-output.md](../../user/fixtures/fixtures-patch-output.md) |
| `lumacore-luma` | The `.luma` show file (`show_file`: a zip of the ShowData tree), the disk door (`drive`), `backups`, `media`, station config, `background` work | [README](../../../crates/formats/lumacore-luma/README.md), [show-files.md](../../user/show-files/show-files.md) |
| `lumacore-mvr` | The MVR file as a format: the scene read from the zip, `head` for the file list; no lumacore dependency | [README](../../../crates/formats/lumacore-mvr/README.md) |
| `lumacore-poolxml` | A DataPool XML export as a format: the pool's groups, read streaming | [README](../../../crates/formats/lumacore-poolxml/README.md) |
| `lumacore-drives` | This station's drives: the OS's volumes probed, `LumaCore/library/` on a drive, the id rule, the table | [README](../../../crates/formats/lumacore-drives/README.md) |

### engine

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-render` | The one renderer: `merge::resolve`, the plan compiler (`plan`, `compiler`), the 60 Hz tick and its thread (`render`, `engine`), `RenderHub`, the bench show and the budget test | [README](../../../crates/engine/lumacore-render/README.md), [engines.md](../engines/engines.md) |
| `lumacore-engines` | `EngineRegistry`, the `Engine` trait, `EngineContext`; the `RenderEngine`, `PerfettoEngine` (feature `perfetto`) and `HttpUiEngine` (feature `http`) wrappers | [README](../../../crates/engine/lumacore-engines/README.md), [engines.md](../engines/engines.md) |

### network

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-net` | Leaf network I/O: NICs, the command-port and shell sockets, mDNS, discovery state, PTP read-out | [README](../../../crates/network/lumacore-net/README.md) |
| `lumacore-clocksync` | The session clock sync: `Time` pings, the filter, the discipline of `lumacore_base::clock`, holdover | [README](../../../crates/network/lumacore-clocksync/README.md) |
| `lumacore-lines` | The line runtime: `ProtocolRuntimes` — a driver per (protocol, NIC), a listener per Data line, reconciled to `protocol_plan::desired`; opens every protocol by its row, never names one | [README](../../../crates/network/protocols/lumacore-lines/README.md) |
| `lumacore-artnet` · `lumacore-sacn` | One DMX protocol each, declared in its `register` (`ARTNET`, `SACN` rows) | [artnet](../../../crates/network/protocols/dmx/lumacore-artnet/README.md), [sacn](../../../crates/network/protocols/dmx/lumacore-sacn/README.md) |
| `lumacore-osc` | OSC, a Data protocol: messages become command lines run as the line's user | [README](../../../crates/network/protocols/data/lumacore-osc/README.md) |

### commands

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-suggest` | The suggestion engine: grammar first, nucleo ranking, per-user Usage | [README](../../../crates/commands/lumacore-suggest/README.md) |
| `lumacore-cmd-treeshell` | The tree shell's words: cd, ls, cat, tree, find, grep, mv / cp / rm, Enter / Exit (the walk stays the kernel's) | [README](../../../crates/commands/objects/lumacore-cmd-treeshell/README.md) |
| `lumacore-cmd-keys` | Bind, Unbind, Shortcuts, UIKey, KeyBindings, the key lines' checks | [README](../../../crates/commands/objects/lumacore-cmd-keys/README.md) |
| `lumacore-cmd-tags` | The Tag object: Store, Label, Delete, Move, Assign / Unassign, List, Info, select | [README](../../../crates/commands/objects/lumacore-cmd-tags/README.md) |
| `lumacore-cmd-showfile` | SaveShow, LoadShow, NewShow, DeleteShow, CleanStart, Import Show; Show, Version, Recovery, Library; the Backup menu; the startup / autosave service | [README](../../../crates/commands/objects/lumacore-cmd-showfile/README.md) |
| `lumacore-cmd-lumasession` | Join, Leave, Invite, Dismiss rows, Claim / Pause LumaSession, the LumaSession object — on the kernel's outbox | [README](../../../crates/commands/objects/lumacore-cmd-lumasession/README.md) |
| `lumacore-cmd-station` | Server, Shell, Client, Clock, Journal, Health, UIEngine, Watch; Restart, Shutdown, Shutoff, Echo, Debug; Station, Interface, Network, Drive, their settings, the wiring, the Network / LumaSessions / Drives menus | [README](../../../crates/commands/objects/lumacore-cmd-station/README.md) |
| `lumacore-cmd-devices` | DeskLock and the lock gate, NormalTheme, Device, ScreenConfig | [README](../../../crates/commands/objects/lumacore-cmd-devices/README.md) |
| `lumacore-cmd-protocols` | The one set of line handlers every protocol's object plugs into (`line_object!`, the kinds' shared properties), DMX In, the DMX Protocols menu | [README](../../../crates/commands/objects/lumacore-cmd-protocols/README.md) |
| `lumacore-cmd-sacn` · `-artnet` · `-osc` | Each protocol's object (`Sacn`, `ArtNet`, `Osc`), registering its protocol's row first | the protocol crates' READMEs |

### ui

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-views` | What every UI shows, built on objects: menus as JSON and their rows, pools, the attribute board, the encoder bar, detail, the sequence sheet, window state. The HTTP feeds and the TUI read it; no client builds its own rows | [README](../../../crates/ui/lumacore-views/README.md) |
| `lumacore-terminal` | The terminals, one per Shell: line editing, history, suggestions, the TUI, dialogs as a centred window, the session registry | [README](../../../crates/ui/lumacore-terminal/README.md), [cli-terminal.md](../../user/terminal/cli-terminal.md) |
| `lumacore-shell` | The TCP shell port: a thread per client, keys queued in order and run as state jobs, the monitor client | [README](../../../crates/ui/lumacore-shell/README.md) |
| `lumacore-http` | The HTTP engine (feature `http`): REST reads, `/ws/feeds` and `/ws/shell`, the static WebUI, the `FEEDS` rows; reaches the station through `ConsoleBridge` | [http-api.md](../api/http-api.md) |
| `lumacore-webui` | The WebUI, files only: the station's page, the connect page, the kit (`web/kit/`), the vendored libraries; embedded by `build.rs` | [README](../../../crates/ui/lumacore-webui/README.md), [ui-kit/README.md](../ui-kit/README.md) |

### server

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-server` | `ServerBuilder` / `Server` (start, the ordered stop against one deadline), `ServerContext`, the state thread (`state.rs`), the UDP command port, discovery, the HTTP console bridge, signals; the server's own log under `library/logs/server/` | [README](../../../crates/server/lumacore-server/README.md), [engines.md](../engines/engines.md) |
| `lumacore-lumasession` | Station ⇄ station: the link (forward, flush, ops in order, resync), election and fencing, the stream, replication, merge on rejoin, invites, per-NIC services | [README](../../../crates/server/lumacore-lumasession/README.md), [lumasession.md](../networking/lumasession.md) |

### dev

Never a product dependency (tidy `dev_tools_never_ship`).

| Crate | Owns | Read first |
|---|---|---|
| `lumacore-tidy` | The source rules of the whole workspace, tests only | [tidy.md](../../meta/tidy.md), [README](../../../crates/dev/lumacore-tidy/README.md) |
| `lumacore-headers` | The ANSI Shadow file headers' font and renderer (`./scripts/unix/headers`) | [code-structure.md](code-structure.md) |
| `lumacore-sweep` | The command sweep, the property tests, the fuzz targets' entry points; the binary is a thin main in the station | [README](../../../crates/dev/lumacore-sweep/README.md), [testing.md](../testing.md) |

### apps and tests

| Package | Owns | Read first |
|---|---|---|
| `apps/lumacore-station` | The assembler: `src/catalog.rs` (`register`, `install_catalog`, the git stamp) and a thin main per binary — `lumacore-server`, `lumacore` (the in-process REPL), `gen-demo`, `gen-docs`, `lumacore-sweep` | [README](../../../apps/lumacore-station/README.md) |
| `apps/lumacore-cli` | Terminal client: the shell stream, `--cmd`, `--health`, `--shutdown`, the Meownitor | [cli-terminal.md](../../user/terminal/cli-terminal.md) |
| `apps/lumacore-app` | The desktop app: the connect page from the WebUI, one OS window per screen on the station's page, starts `lumacore-server` next to it; OS code only in `src/platform/`; out of `default-members` | [software-init.md](../../user/identity/software-init.md) |
| `apps/lumacore-scripts` | The TUI launcher for `scripts/` | [scripts/README.md](../../../scripts/README.md) |
| `tests/systemtest` | Build-linked verification, in process | [README](../../../tests/systemtest/README.md) |
| `tests/catalog` | The whole catalog's tests: every row's rules, lum.md, the Lum words, the tree paths, the reference, the demo golden | [README](../../../tests/catalog/README.md) |
| `tests/fuzz` | cargo-fuzz targets (nightly, outside the workspace) | [testing.md](../testing.md) |
| `tests/e2e` | Playwright UI end-to-end | [README](../../../tests/e2e/README.md) |

External reference repos (the `SKILL` workspace) are per developer, named in the
gitignored `.env.local` — see [build-environment.md](../build/build-environment.md).

## Data flow

```
.luma (a zip: manifest + one JSON per ShowData field, folders per pool member)
    │  lumacore_luma::show_file
    ▼
ShowData                                   lumacore-tree (authored, replicated, saved)
    ├── Devices (Dmx.Patch: fixtures by Slot, universes; Metadata.MVR)
    ├── DataPools (groups, presets, sequences, timing / curve presets …)
    ├── Playback · Playbacks (running state: saved, replicated, never undone)
    ├── Users (programmer, selection, usage) · UserProfiles · Stations · Session
    ├── Protocols (networks, each protocol's lines) · Tags · Variables · …
    └── ShowSettings (format_version, saved_with_commit, show_id)

Root (in memory)
    ├── ShowData                           authored truth
    ├── StationSettings                    station-local, never replicated
    ├── Runtime                            lumacore-runtime: users' modes and undo, LumaSessions,
    │                                      claims, network, journal, health — never saved
    └── Drives · UIEngines · Resources · Diagnostics

DMX
    state thread → EngineInput → RenderHub::publish
      → plan compiler thread (channel tier when the patch moved, value tier per change)
      → render thread (60 Hz: fades, masters, Solo, park, input merge) → frames (ArcSwap)
      → one driver per (protocol, NIC) samples at each line's Rate and only sends
    input lines → InputMerger → RenderHub::inputs → merged per channel in the tick

Data protocols
    an Osc line's listener → a command line run as the line's user → dispatch
```

## Quick routing

| Task | Crate | Read |
|---|---|---|
| Fixture types, modes, channels | `lumacore-fixtures`, `lumacore-gdtf` | [fixtures-patch-output.md](../../user/fixtures/fixtures-patch-output.md) |
| Patching, the Devices draft, MVR import | `lumacore-objects` (`objects/patch/`), `lumacore-state` (`patch_draft`), `lumacore-patch`, `lumacore-mvr` | [fixtures-patch-output.md](../../user/fixtures/fixtures-patch-output.md) |
| DMX output, input, line rates | `lumacore-render`, `lumacore-lines`, `lumacore-protocols`, the protocol crates | [engines.md](../engines/engines.md) |
| Session clock, health | `lumacore-base` (`clock`), `lumacore-clocksync`, `lumacore-runtime` | [metrics-and-health.md](../build/metrics-and-health.md) |
| Sequences, cues, masters | `lumacore-playback`, `lumacore-objects` | [sequences.md](../../user/playback/sequences.md) |
| LumaSessions, replication, election, merge | `lumacore-lumasession`, `lumacore-state` (`session_link`, `change`), `lumacore-cmd-lumasession` | [lumasession.md](../networking/lumasession.md), [replication.md](../networking/replication.md) |
| Discovery, NICs, the command port | `lumacore-net`, `lumacore-server` | [lumasession.md](../networking/lumasession.md) |
| The command language, a new verb or object | `lumacore-syntax`, `lumacore-objects`, the `lumacore-cmd-*` crate of the area | [lum.md](../../user/syntax/lum.md), [commands.md](../../user/reference/commands.md) |
| Suggestions | `lumacore-suggest` | its README |
| Object tree, typed paths | `lumacore-tree`, `lumacore-state` (`tree_path`) | [object-tree.md](../../user/object-tree/object-tree.md) |
| Show save / load, backups, drives | `lumacore-luma`, `lumacore-drives`, `lumacore-cmd-showfile` | [show-files.md](../../user/show-files/show-files.md) |
| HTTP, feeds, the WebUI | `lumacore-http`, `lumacore-views`, `lumacore-webui` | [http-api.md](../api/http-api.md), [ui-kit/README.md](../ui-kit/README.md) |
| Terminals, the shell port | `lumacore-terminal`, `lumacore-shell` | [cli-terminal.md](../../user/terminal/cli-terminal.md) |
| Keys | `lumacore-cmd-keys`, `lumacore-tree` (`ui_keys`) | [keys.md](../../user/keys/keys.md) |
| Tags | `lumacore-tags`, `lumacore-cmd-tags` | its README |
| Tests of every row | `tests/catalog` | [testing.md](../testing.md) |
