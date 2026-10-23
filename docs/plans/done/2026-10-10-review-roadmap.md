# Handoff — code review → decisions → implementation (2026-10-10)

Read this first when continuing. Source of truth for every decision:
`docs/archive/decisions-2026-10.md` (§3, §10, §11, §15–§27). CLAUDE.md holds the short rules. Every
question the user answered, with their words: `docs/archive/docs/plans/done/2026-10-10-decisions-qa.md`.

## How the user works (keep doing this)

- Harden decisions by **asking** (AskUserQuestion, recommended option first) before building.
  Check older docs first (`docs/dev/networking/*.md`, decisions §2/§4) — the user often decided
  things in March already.
- **Implement only after the user says "go"** for a step. Commit + push only when asked.
- After code changes: `cargo build` + **only the tests for this step** (filtered by name). Never
  the full suite unless the user asks ("no full tests, just test per step").
- Cat naming (Meow Corp): modifier key **Meow**, the distribution tool = **Pawtern**.

## Work order (decisions.md §25) and status

| # | Step | Status |
|---|------|--------|
| 0 | Quick fixes + cleanup baseline | ✅ `533f7c4` |
| 1 | One registry / one namespace | ✅ `90bebb2` (+ systemtest fix `888b408`) |
| 2 | State thread + change path + typed errors (§26 "Step 2 details") | ✅ `c92c190` (systemtest 145/0/0) |
| 3 | Mode objects (On/Off/Toggle Blind/Highlight/Freeze/Solo, §16) + keys (§17) | ✅ commit "Step 3: mode objects, knock in/out, one key system" (step tests 35/35) |
| 4a | ✅ `9f10b69` (step tests pass; live: cues/Go/master/park/solo/resume on the engine frame and sACN on the wire) — hardened (decisions §27 "Step 4 hardening", §4, §7, §13): Value type, programmer LTP + transfer, Sequence → Cue → Part, Store Cue / Go / Go- / Goto / Off / Release, `ShowData.Playbacks`, grand master, Solo, Park as Values, one engine replacing 3 renderers, Clear Programmer User, PURR, generated list_tree | |
| 4b | ✅ `e4b128f` (step tests pass; live: unmapped silent, sACN from the mapped NIC incl. unicast, Art-Net ArtDmx + ArtPoll, clean shutdown) — (§11 "Step 4b hardening") — networks (show names, station mapping), output lines + routes, reconcile, who sends, ArtPoll + directed broadcast, sACN CID, OSC input line, ranges dropped, line status, Live Patch Set | |
| 4c | ✅ `5b48ea8` (step tests pass; live: guest sACN on lo0 merged into local 1–2 and re-sent as sACN on en0, priorities, 2.5 s loss, Park over input, MergeMode, `/ws/dmx`, clean shutdown) — (§11 "Step 4c hardening") — one Line type, DMX input + per-channel merge, sources, `DMX In`, `dmx` window | |
| 4d | ✅ `3b65944` (step tests pass; live with two servers: invite/dismiss/join/leave, services per NIC, LumaSession off) — menus LumaSessions / Network / DMX Protocols, LumaSession rename (no aliases, `Shell`), named sessions + invite/dismiss/on-off, services per NIC, staged patch (§11 "Menus, LumaSession and staged patch") | |
| 5 | ✅ (step tests pass; live: spoofed client_addr refused, Guest refused in a Shell, remote in-session Login refused, logout → 401, no passwords in sysmon/log) — Security (§19 "Step 5 hardening"): one rights gate in dispatch for every transport, OSC as line User, Remote Login holds (client_addr only from loopback, in-session Login), passwords masked at the source, logout revokes; UDP/HTTP stay open | |
| 6a | ✅ `d14f6a8` (step tests pass; live: user delete + Oops, saved file has format 1 + commit and no undo/profile programmers, newer format refused, Runtime.Users.1.Undo readable) — Groundwork (§21 "Step 6 hardening"): generated per-field subtrees (all undoable but Playbacks), programmer/selection onto User (old ones start empty), undo in `Runtime.Users.{User}.Undo` (replicated, not saved), format_version + commit (forward only, newer refused, old fix-ups removed), stale docs fixed | |
| 6b | ✅ `511655f` (step tests pass; live with two servers via `--station`/`--port-base`: 108 KB show joined over the stream, forwarded commands both ways, Oops from a member, modes replicated, NIC mapping stays local, master killed → command pending → master back → ran, Leave / Invite / Dismiss) — Link (§21 "6b hardening" / "6b built") | |
| 6c | ✅ `990cb2b` (step tests pass; live with three servers: master killed → election, master frozen and back → two masters, fenced + prompt, Use Mine, `Claim LumaSession` handover, IdleMaster alone (no DMX) → re-invited, diverged IdleMaster → prompt → Use Master with autosave, Alternative multicast group) — health, master loss, election (§21 "6c hardening" / "6c built") | |
| 6d | ✅ `3ae128f` (step tests pass; live with three servers: master frozen → B elected, both sides changed, A fenced with the Merge prompt listing what differs (refreshed after A's own change), Pause held the countdown, default Merge after 60 s and manual `/Merge`: both autosaved `before-merge`, mine moved to a free slot, members got the merged show, references clean, claims replicated) — identity (GUID on every object, GUID links, strict GUID distribution), format 3 + show_id, reference check, split base + 3-way merge + prompt, ownership (§21 "6d hardening" / "6d built") | |
| 7 | ✅ `c2b99bb` (step tests + e2e pool-window pass; live with two stations) — Groups (§22 "Step 7 hardening" / "Step 7 built"): DataPool Group only (members + grid + rotation/mirror, pawtern slot), FixtureGroup gone, probed console selection rule, grid cursor, Store /Merge /Overwrite /Remove, group masters (mode on group, level in Playbacks, fade), Master React, delete refused when referenced, unified pool window (PoolDef on ObjectDef, `/ws/pools`, gestures, full-screen selector, group editor, settings in views) | |
| 8 | ✅ `5e1e465` (§23 "Step 8 built": station lib 386/387 (known shutdown test), wire/trace/http/registry pass, e2e 53 pass + 3 known-stale failures, live CLI + E-WIRE-001 refusals) — hardened (§23 "Step 8 hardening") — `lumacore-core` → `lumacore-base`; `lumacore-wire` (client contract: ports, beacon + StationKind, command port, shell packets, keys, UiEvent, typed Status, SESSION/CLIENT protocol numbers, E-WIRE); clients under `apps/` on wire only; app spawns `lumacore-server`; `lumacore-trace`; build.rs fix; layers `server/` → `terminal/` · `net/` → `lumasession/` → `core/` (outbox) + tidy layering test; docs. One step. | |
| 9 | ✅ `7e0ec48` (§24 "Step 9 built": station lib 389/389 (shutdown test fixed), dmx 22 + release budget (35k < 3 ms), base/session/tree/wire/http/registry/tidy pass, e2e 53 + 3 known-stale, clippy clean; live: 60 Hz render + stages, line Rate/Changes/Sync on the wire, session clock Stable in 3 s, holdover on master loss) — hardened (§24 "Step 9 hardening" + "Session clock (researched)") — 60 Hz engine, plan compiler (two tiers), allocation-free tick + send (assert_no_alloc, tidy rule, bench), panic hold/restart, overload skip, Line Rate/Send/PacketGap/Sync, session clock (UDP Time pings, filter, rate overlay, holdover), Runtime.Health (render stages, lines, clock in PTP terms, linuxptp readout), Realtime window, leftovers removed. One step. | |
| 9b | ✅ `2f71191` (§23 "Step 9b built": station lib 389/389, app 2, http 31, wire 3, e2e 68/0 (the 3 stale fixed + 12 that didn't run), clippy clean; live: SIGTERM/SIGINT/SIGHUP clean stop + autosave, second signal exits, save guard copy + /Overwrite) — App ↔ local server (§23 "App ↔ local server hardening"): no silent attach to another version (protocol / build / rebuilt binary → Replace / Use / Cancel), server handles SIGTERM/SIGINT/SIGHUP (autosave + goodbye + deadline), hung stop → Force stop on click, save guard against overwriting a newer-format show (copy + overwrite prompt), demo-show test (generate + load + match); found + fixed: same-second re-login got the revoked token (v2 tokens with a nonce), gen-demo's LD user; scripts fixed | |
| 10 | ✅ `95cd6b0` (§25 "Step 10 built": property tests 14 pass + 1 ignored finding, tidy pass, clippy clean; full sweep + 5 min random = 474,831 lines, 10 findings recorded, fix later; fuzz short run no crash) — fuzz targets for every LAN decoder + lexer in an out-of-workspace nightly `fuzz/` crate, long run only (`scripts/unix/fuzz_long`, by hand overnight, logs to target/fuzz-report/); property tests (proptest, ~10 s) from random command sequences: .luma round-trip, replication, undo, merge, election/clock, identity/masking; command sweep (every verb/keyword/flag/property, generated dynamically from the registries; exhaustive + deep random via completion; checks per line incl. Oops, read-back, rights per role; resumable run.json; only `reach: Show` commands — tree shell + drive excluded via a required CommandDef field; in process, no sockets/disk); findings reported, fixed later; no size caps; stale docs fixed | |
| 11 | Hardening (§29): 11a docs restructure (one roadmap, archive, generated reference, LumaSession doc), 11b code fixes (step 10 findings, Planned, gaps, GDTF cells, playback/sACN leftovers), 11c Oops categories + UI Oops, 11d screenshots (client capture on server request, `Screenshot`, scripts/unix/screenshots) | |

Branch `termiham`, remote `origin`. Untracked user files to leave alone:
`library/shows/NewShow_2026.10.09_23.54.53UTC.luma`, `library/shows/testshow.luma`.

## Step 1 — what exists now (don't undo)

- `terminal/commands/registry.rs`: `CommandDef { verb, aliases, doc: Doc, run, undo: Undo,
  family, scope: Scope, status: Status, object_verb, args: Args, rights: Rights, class }`;
  `Word::{Command, Object}`; `resolve()` (exact → prefix with alias folding and
  shortest-prefix base rule → Levenshtein); `lookup_word`, `all_names`, `all_words`,
  `completion_keywords`, `verb_suggestions`. Undo shorthands `PROGRAMMER`, `SELECTION`,
  `PROGRAMMER_SHOWDATA`, `PATCH`, `STORE`, `EDIT`, `USER_SETTINGS`.
- `terminal/objects/`: `ObjectDef { keyword, aliases, doc, family, call_undo, call_class, verbs,
  complete, tree }`; `dispatch_verb`, `call` (no Call → List), `complete(state, Word, …)`.
  Macros `listed_object!` (verb => closure, optional `call:`, `extra:`), `playback_object!`
  (tree path, optional `extra:`), `preset_type!`.
- Handlers: `commands/run.rs`; per-object list fns in `commands/list/mod.rs`
  (`fixtures`, `tree(state, path, args)`, …), `info::info_*`, `show::dump_*`,
  `edit::label_with/edit_with`, `store::store_group/copy_group/update_group`,
  `move_cmd::move_*`.
- `commands/tokens.rs`: the one lexer (`lex`, `lex_lenient`, `statements`, `word_start`).
- `terminal/tidy.rs`: fails on match patterns with ≥2 registered words outside the registry
  (exempt: `// tidy: not-keywords (<reason>)` within 8 lines above), and on TUI hotkeys / core
  UI keys that don't resolve. `commands/word_tests.rs`: behaviour tests.
- `terminal::ShowDataScope`: the one snapshot-scope parser (wire values).
- Blind is a **planned** CommandDef (becomes a mode object in step 3).

## Known open items

- Pre-existing `lumacore` unit-test failures (fail on the pre-step-0 commit too):
  `remote::tests::shutdown_command_sets_should_exit` (Shutdown now asks to confirm),
  `terminal::commands::tests::fixture_selection_is_blocked_by_other_programmer_claim`.
- Undo quirks kept as-is in step 1: `set`, `oops`, `reset*`, `meow` are non-mutating (no undo
  frame); generated coverage comes in step 6.
- `universe.rs` Assign/Split/Merge grammar searches for the `universe` token (grammar, fine).

## Step 3 — what exists now (don't undo)

- **Runtime.Users** — `RuntimeState::users` / `RuntimeUser` (was `sessions` / `UserSession`),
  `user()` / `user_ref()`; `Modes` + `UserMode` in `lumacore-session`. Tree
  `Root.Runtime.Users.{no}.Modes|ViewSessions` (`vfs_runtime.rs`); `Info Server` shows `Users`.
- **Mode objects** `terminal/objects/modes.rs`: BLIND, HIGHLIGHT (alias `hi`), FREEZE, SOLO.
  Verbs `Verb::On/Off/Toggle` (new), Call = toggle, `Blind On` → E-CMD-005. Highlight/Solo still
  drive the old global overlay (`control::highlight_overlay/solo_overlay`); Blind/Freeze are
  state only. Old `highlight`/`solo`/`blind` CommandDefs removed; `toggle` CommandDef added.
  There are no Planned commands left.
- **On/Off** = knock in / knock out (`control::knock_in/knock_out`, `Programmer::knock_in/out`),
  on Fixture, Group and Selection objects too; Off also deselects.
- **Status**: per-user `modes` in the client status snapshot (`shell_session.rs`); station
  HI/SOLO statuses removed (`feedback.rs`). Frontend: mode icons, 4 mode keys + 🐾 Meow latch in
  the left sidebar, `.lit` LEDs from `UiKey.state`.
- **Keys**: `UiKey` has Release, ReleaseAfterHold and `meow: KeyCommands`; `Gesture` has 5;
  core keys for modes + every vanilla binding (`lumacore_core::keybindings::VANILLA`).
  `KeyBinding.ui_key` (legacy `command` read, migrated in `ShowData::after_load` via
  `UserProfile::migrate_keybindings` / `bind_command`). `terminal/keys.rs`: `bind`/`unbind`,
  `check_line`, `check_after_load` (sysmon "Keys"), `find`, `bindings`, `command_for`.
  TUI `KeyCache` (rebuilt on show_generation/user/debug change — Bind is immediate), Meow =
  profile `meow_key` (Set MeowKey), DoublePress by timing. Wire `KeyAction.meow`. Errors
  E-KEY-001..004.
- **Gaps**: mode state isn't replicated yet (Runtime replication = step 6); the old overlays are
  global, not per user (step 4 replaces them); keyboard Release/Hold through the web UI not
  forwarded (only on-screen keys); `Session` object still named for its old meaning (§18).

## Step 2 — what exists now (don't undo)

- **State thread** `crates/lumacore/src/server/state.rs`: owns `ServerContext`; `StateHandle::call`
  (wait for a result, panic → `CallError::Panicked(report)`) / `send` (fire and forget);
  `Periodic` tasks (housekeeping 50 ms, purr 30 s, HTTP diagnostics 1 s); OSC/inbound events via
  `select!`; publishes `Published { show: Arc<ArcSwap<ShowData>> (undo stacks stripped),
  station: StationView }` after every `show_generation` change / every 250 ms.
- **ServerBuilder** `server/mod.rs`: `ServerBuilder::start` → `Server::wait` → `Ended{restart}`;
  `serve(config) -> !` is the only entry of `lumacore-server` and `lumacore-app --server`.
  Shutdown: OSC → AppState sACN (`stop_sacn_output`) → engines `stop_all(deadline)` → network
  threads (`stop` flag) — one 5 s deadline, late ones logged.
- **Network threads** `remote/spawn.rs`: `spawn_udp_server` (new), `spawn_shell_server`,
  `spawn_session_discovery` — all via `StateHandle`, stop flag, health. `remote/shell.rs`:
  every lock is a job; socket writes after the call; `render_pass` runs on the state thread.
- **HTTP**: `engines/http.rs` `HttpUiEngine` (registry engine, graceful shutdown via
  `HttpHandle::stop` + join). `ServerConsoleBridge` = commands/logins via queue, reads from
  `Published` (`verify_token_with`, `screens::saved/default_config` take `&ShowData`).
  `HttpState.show` is the published ArcSwap. `POST/DELETE /api/groups` removed (wrote a boot
  copy; no client used them).
- **Change path** `terminal/change.rs` (`begin`/`finish`/`rollback`, cached images, `subtrees()`
  = the only subtree list). `dispatch.rs`: outer dispatch only (`dispatch_depth`); `Undo` is
  `ReadOnly | Untracked | Tracked(category)` (no subtree lists). Journal + undo frame +
  replication op from the diff. Datapool snapshot of a missing pool = `null` (restore removes).
  `Set` is now `USER_SETTINGS` (tracked) instead of read-only.
- **Panics** `panic_report.rs`: hook + `guarded()`; dispatch rolls back and replies with
  file:line/command/user/build/report # `[E-INTERNAL-001]`; `hissme` → `process::abort`.
- **Errors**: `lumacore_core::error_code` (`Coded`, `ErrorInfo`, `find`, `check_catalog`);
  catalogs `ERRORS` in patch/dmx/gdtf, `OUTPUT_ERRORS`/`UNIVERSE_ERRORS` in tree,
  `crate::errors::ERRORS` (E-CMD-001..004, E-SET-001..004, E-INTERNAL-001), `CATALOGS`.
  `Error` object: `List Error [area]`, `Info Error <code>`. One `renderer::strip_ansi`.
- **Runtime**: `lumacore-session` `journal.rs` (`Runtime.Journal`, 1000) and `health.rs`
  (`HealthBoard`, `ThreadHealth` atomics, `ThreadGuard`). Objects `Journal` (alias `events`:
  `List Journal`, `Watch Events`) and `Health`; tree nodes `Root.Runtime.Journal/Health`.
  Threads reporting: State, Udp, Shell, Discovery, DiscoverySend, Osc, Http, Render.sACN,
  Render.ArtNet, Render.Output (AppState's sACN).
- **`ShowData::after_load()`** (ensure_users + `rebuild_id_pools`, incl. the fixture pool, now
  `PatchData::fixture_id_pool`; `PatchEngine` takes no pool argument) — called by
  `replace_show_data`, `apply_show_data_scope`, `restore_subtrees`, `restore_snapshot`,
  replication.
- **Generic Set** `terminal/props.rs`: `TreeDef::write: Option<WriteDef{props, apply}>`;
  `props::META` (Name, Note, Locked, Hidden) on Fixture, Universe, Range, Output (+ Priority
  0–200, Enabled; Admin). `Set Fixture 3 Name "x"` or `Set <tree path> <prop> <value>`.

### Step 2 known gaps / follow-ups

- `Join` (`commands/session.rs`) still does blocking UDP requests inside a command (on the state
  thread) — moves to the station-to-station stream in step 6.
- Panics caught by dispatch are journaled (`internal error, report #n`) but not counted in
  `Health.State.panics` (only job-level panics are).
- Journal `epoch` is 0 until master election exists (§10).
- Change-path coverage is the existing `ShowSubtree` set (programmers, user settings, patch,
  datapools, protocols, playback, show settings, effect engines); Users, variables, icon pool
  etc. are not tracked yet → step 6 makes `change::subtrees()` exhaustive.
- Writable properties declared for patch objects only; more as areas are touched.
- Programmer snapshot for the playback engine: step 4 (§27).

## Step 4a — what exists now (don't undo)

See decisions §27 "4a built" for files and known gaps. Highlights: `DMX 1` / `DMX 1.17` reads
the engine's real frame; `Sequence` lists running playbacks; `List Root…` overviews are
generated from the tree (`list_tree::tree_overview`); `Runtime.RunningPlaybacks` became
`ShowData.Playbacks` (+ reserved `Runtime.Devices`); `Black` and the `release` alias of Unpark
are gone (`Off Master 1`, `At Release`); history button is PURR. sACN no longer binds the fixed
port 5569 (a second server on one machine silently sent nothing before).

## Step 4b — what exists now (don't undo)

See decisions §27 "4b built". Highlights: `List Network` shows the show's networks, this
station's mapping and its interfaces; `List Output`/`Info Output` show each line's live state;
`Root.Runtime.Network.Lines` / `ArtNetNodes`; status-bar icon per protocol ("sACN 1/1").
Unit tests never write `library/station_settings.json` (`LocalSettings::save` is a no-op under
`cfg(test)`).

## Step 6a — what exists now (don't undo)

See decisions §21 "6a built". One `ShowSubtree` per ShowData field (exhaustive destructure),
programmer/selection on `User`, undo in `Runtime.Users.{User}.Undo`, Oops/Redo replicate,
Clear Programmer User undo goes to the affected user, `format_version` + commit (forward only,
E-SHOW-001), old load fix-ups removed. Systemtest builds (undo domain now expects Unclaim Patch
to keep history) but was not run.

## Step 5 — what exists now (don't undo)

See decisions §19 "Built". One rights gate in dispatch (`terminal/rights.rs`, E-RIGHTS-001),
Remote Login holds (loopback-only `client_addr`, in-session Login, E-RIGHTS-002), passwords
masked once in dispatch, logout revokes tokens. UDP/HTTP/station trust deliberately stay open
(isolated rigs).

## Step 4d — what exists now (don't undo)

See decisions §27 "4d". Menu → Network / DMX Protocols / LumaSessions; LumaSession words
everywhere (no aliases, `Shell` for terminals); staged patch on the Patch claim; per-NIC
services; members known to their master.

## Step 4c — what exists now (don't undo)

See decisions §27 "4c built". Highlights: `Store Input 1 "Guest" /Protocol sACN /Network Guest`,
`Assign Universe 1 Thru 6 At Input 1 Universe 1`, `Set Input 1 InputPriority Super`,
`Set Input 1 MergeMode HTP`, `DMX In 1`, `Info Input 1` (sources), `Root.Runtime.Network.Sources`,
window type `dmx` (`/ws/dmx`). Output/Input share every handler — keep it that way.

## Step 4 — hardening notes

All answers are in decisions §27 "Step 4 hardening", §4 (programmers and fixture claims),
§7 (Freeze), §11 "Output routing", §13 (playback state), and `decisions-qa.md` "Step 4
hardening". The code survey (renderers, value types, cue model, protocols) found:
- three renderers: AppState sACN (the only one with programmer values), and `SacnEngine` /
  `ArtNetEngine` that render boot defaults forever and both send.
- programmer stores `u8` and renders only the effective user.
- `dimmer_offsets` is never computed, so Solo/Black do nothing on normal patches.
- 16-bit fine bytes are never written; Park reads `ch{n}` keys.
- OutputLine and range `sacn_assignments` are both ignored by the drivers.
- OSC runs as Admin.
- Go/Pause/Stop/Flash and the Sequence/Cue objects are stubs.

Ranges: dropped (4b hardening).
