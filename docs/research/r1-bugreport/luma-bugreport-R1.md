# Luma bug report R1 — published console bug fixes vs LumaCore

**Date:** 2026-10-13 · **Code:** `279e5de` (main) · **Source:** a private collection of 5250
bug fixes from an established console's release notes, sorted by area (not in the tree).

**How it was made:** nine read-only agents, one per area cluster, each read its bug-fix files, grouped
the fixes into bug classes and compared each class with our code, `docs/archive/decisions-2026-10.md`, the
roadmap and the docs. **Nothing was run** — every risk below comes from reading code and must be
confirmed with a failing test before it is fixed. Line numbers are as of `279e5de`. Nothing here is
fixed yet; every fix is asked before it is built (as in `issues/README.md`).

"(n reports)" = found independently by n agents. Overlaps with the sweep issues are noted
(`→ issues/NN`).

About half of those fixes are in areas LumaCore hasn't built (executors, timecode, macros, effects,
3D, media, RDM, MVR, worlds, filters, recipes). Many other classes are **prevented by design**: GUID
links, the automatic change path, one state thread, guarded commands, refused old formats, one
dialog path, per-user programmers.

## Status and how to continue (2026-10-13, read this first)

**Nothing is fixed or committed yet.** Every box below is still open.

**First attempt hung the machine.** It ran as root and started six agents, each building in its own git worktree
(`.claude/worktrees/agent-*`, ~33 GB of `target/`), on a box with 3 GB RAM, 16 cores and a 96 % full disk.
Their uncommitted, uncompiled diffs are saved in [r1-wip/](r1-wip/README.md) (by area, with the R1 ids each covers).

**Rules for the next attempt:**
- Run as the normal user; before starting: `sudo chown -R` the repo to that user, delete
  `.claude/worktrees/*/target` (then `git worktree remove --force` the six worktrees once their patches are merged),
  and make sure `cargo` exists for that user (the root toolchain isn't on that user's PATH).
- **One cargo process at a time**, `-j 2` (more only with more RAM), tests one crate / one test target at a time with
  `--test-threads=2`, `timeout` on long runs. No parallel building agents, no worktrees per agent;
  agents only read. No fuzz/sweep long runs.
- Apply the `r1-wip/` patches one area at a time: review against this report and the decisions, build, test,
  commit, then the next area. Then fix the items no patch covers.

**Decisions.** The open questions ([luma-bugreport-R1-questions.md](luma-bugreport-R1-questions.md), Q1–Q46) were
checked against `decisions.md` and the docs: [luma-bugreport-R1-docs-check.md](luma-bugreport-R1-docs-check.md).
- **Settled by the docs** (build as they say): Q1 ★b (+ fix the "never runs backwards" wording), **Q4 b**, **Q5 b**
  (no input caps; only the 1 GiB preallocation and the O(n²) shell scan), Q7 ★a, Q12 ★b (Off Sequence snaps),
  **Q17 b** (Blind hides the programmer only), Q19 ★a, Q27 ★, Q28 ★a, **Q29 b** (claim refused only by live holders),
  Q30 ★, **Q33 a** (shared profile = shared layout), **Q35 b** (station asks, `/NoConfirmation` skips, no setting),
  Q41 ★a, Q43 ★c, Q46 ★a. Bold = against the ★ in the questions file.
- **Docs contradict themselves — waiting for the user:** Q8, Q11, Q16, Q26, Q37, Q44 (and the format number: 10 in
  CLAUDE.md, 7 in crate-map.md / lumasession.md / decisions contents).
- **Not answered by the docs — waiting for the user:** Q2, Q3, Q9, Q10, Q14, Q15, Q21, Q22, Q24, Q25, Q32, Q34, Q36,
  Q38, Q39, Q40, Q42, Q45, and the open parts of Q6 (missing ids), Q13 (Goto/Go- timing), Q18, Q20, Q23, Q31 (a or b).
- Items that need none of these can be fixed first: R1-01 02 03 05 06 07 09 10 15 18 20 21 23 24 26 32 33 36 39 40
  41 44 46 48 51 53 59 63 64 65 68, D1–D4, D10.

## TODO — left open on branch `bugfixes` (2026-10-13)

- [ ] **R1-56 / Q31 — the page reports a window's scroll** (saved as the *first visible row*, decided).
  Nothing in the web page records scroll today. Open design choice: (a) the page sends
  `Set Window <n> Scroll <row>` when scrolling stops (a command line: synced, but it echoes) or
  (b) a quiet station-local message (silent, but an exception to "every gesture is a command line").
  Ask the user, then build the page side and restore on recall.
- [ ] **R1-60 / R1-11 / Q34 — pool window stop-gap** (decided: redraw only changed cells, keep scroll,
  draw only visible cells; `<luma-pool>` mockup in 12e). UI work: must be checked in Chrome and with
  `./scripts/unix/screenshots` — not possible on the 3 GB build box, so build it where a browser runs.

## Findings

Tick a box when the item is confirmed or fixed; strike it through if it turns out wrong.

### T1. Time: wall clock where the session clock belongs (4 reports)
- [x] R1-01 LTP stamps use the wall clock and are never raised to the highest stamp seen in a loaded show, a replicated op or a merge (`crates/lumacore-base/src/value.rs:179`). After failover / load / merge on a station whose clock is behind, new touches lose to old values.
- [x] R1-02 Heartbeat loss and the merge countdown use the wall clock (`core/session_link.rs:80,342-350`, `lumasession/election.rs:473,682`): an NTP step > 5 s triggers a false election; a step back hides a dead master.
- [x] R1-03 Saved group-master fades store an absolute start (`MasterLevel.start_ms`, `crates/lumacore-playback/src/lib.rs:519-520`); loaded where the clock is behind, the master stays at its old level.
- [x] R1-04 (Q1 b: the code already steps only at the first lock or beyond 1 s and slows down otherwise; the "never runs backwards" wording fixed everywhere) The session clock can step backwards (`lumacore_base clock.rs:132`, `lumasession/clock.rs:450,460`), against §24 "never runs backwards".
- [x] R1-05 `Group 1 At 50 Fade inf` → `fade_ms = u64::MAX`, `MasterLevel::at` overflows in the render tick (`lumacore-playback/src/lib.rs:520`): engine down in debug/sweep, silent wrap in release (§26 "checked arithmetic").

### T2. Numbers that aren't checked (3 reports)
- [x] R1-06 NaN/Inf get stored: `At + nan`, `At * inf`, glued `At /0` (`core/commands/control.rs:270-279`), `Master 1 At nan` (`core/commands/playback.rs:502-512`). serde_json saves NaN as `null` → the show won't load, member resyncs fail.
- [x] R1-07 `Patch 4294967295x` aborts the server: `Vec::with_capacity(count)` before any check (`crates/lumacore-patch/src/patch.rs:330`). OOM aborts aren't caught by the panic guard.
- [x] R1-08 Huge `Thru` ranges collected into Vecs (up to ~16 GB): `groups.rs:69-79`, `move_cmd.rs:76-90`, `universe.rs:70-90`, `programmers.rs:211-231`, `edit.rs:352`.
- [x] R1-09 `Label … 1 Thru 4294967295` loops ~4·10⁹ times on the state thread (`edit.rs:343-352`).
- [x] R1-10 A long `1 + 1 + …` selection builds a left-nested Box tree resolved recursively (`core/parser/mod.rs:112-131`, `resolve.rs:62-66`) → stack overflow.
- [ ] R1-11 (part fixed: group numbers 1–9999 on Store / Copy / Move (Q3) — drawing only the visible cells waits for the pool window rebuild (Q34)) `Store Group 1000000` hangs every client viewing that pool: no pool size cap, auto mode builds cells up to the highest number (`apps/lumacore-app/frontend/index.html:~2644`).
- [x] R1-12 Overflows: `Move … At 0` allowed; `dest_start + i` near u32::MAX (`move_cmd.rs:121,214,276`); `Grid 2147483647` + selection (`crates/lumacore-tree/src/programmer.rs:336`). `At 150` stored unclamped.
- [ ] R1-13 Sweep never tries > 999, NaN or inf (`sweep/lines.rs:161`, `vocab.rs:208`); random lines ≤ 7 words; cargo-fuzz never actually ran (only `--no-fuzz`), passes `-ignore_ooms=1` (`run.rs:620`), `command_line` skips dispatch above 4096 bytes.

### T3. Files that don't survive a power cut (3 reports)
- [x] R1-14 (written atomically; an unreadable file is never written over — defaults in memory, sysmon; Q36: kept as built) Station settings written in place (`station_config.rs:173-184`); any parse error loads defaults and saves them over the file (`:94-123`): new station id, sACN CID and token secret, network→NIC map lost → **dark rig**.
- [x] R1-15 Media and GDTF stores, autosave copies and profile export use plain `drive::write`; a store file is trusted by existence only (`core/media.rs:53-85`, `show_file.rs:78-95,196-211`) → a truncated file is served and packed into the next save with a fresh hash.
- [x] R1-16 (a show is never replaced when its safety copy failed; every save keeps a backup, newest 10) Timed autosave overwrites the named show, no backup generations; a failed safety copy is ignored even when Invite goes ahead. (§9 says `.autosave/`.)
- [x] R1-17 (the startup show load is panic-guarded; a `.starting` marker skips it once after a hard death, E-SHOW-006 — Q38) Startup show load is not panic-guarded (`server/app.rs:120-147`) → a bad show crashes every start.
- [x] R1-18 (the round trip now goes through `show_file`; its property test is `#[ignore]`d with issue 09) The sweep's save/load round trip goes through serde JSON only (`sweep/world.rs:445`), not `show_file::encode/decode` — the layer where the biggest class ("field lost after save/load") lives.
- [x] R1-19 (fixed as §25 / Q5 b says: the 1 GiB preallocation and the O(n²) shell scan; no input caps) No size caps on `.luma` entries, zstd decode in the stream, a 1 GiB preallocation from a 4-byte header (`show_file.rs:419`, `stream.rs:302-304`), the shell read buffer (`net/shell.rs:768`). Accepted by §25 (decisions.md ~2318) — worth revisiting.

### T4. LumaSession: commands running twice, rollback
- [x] R1-20 Double run after a reconnect: `peer_gone` deletes the master's record of what already ran (`lumasession/link.rs:961`), the reconnect restarts from 0 (`:813-817`), resent commands run again (`:888`). Test `a_command_resent_after_a_reconnect_runs_once` only resends on the same connection.
- [x] R1-21 Double run across an election: `ReplicationOp` has no command id (`crates/lumacore-base/src/replication.rs:42`); `run_pending_locally` (`election.rs:236`) and the resend can re-run an applied line whose reply was lost.
- [x] R1-22 (Q26: a station leading the session asks Use Mine (default) / Use Master / Cancel; the split base is noted on disk and restored at start for the same show) A rebooted high-priority master re-invites the IdleMaster that kept going (`election.rs:913-960`); `handle_invite` (`lumasession/mod.rs:138`) never compares which show is newer; the split base is memory-only → session rolls back to the old show on disk.
- [x] R1-23 A big stream frame stops the heartbeat: `heard()` runs after a whole frame (`stream.rs:406`), Pings can't go out while writing → > 5 s frame cuts the link, resync loop.
- [x] R1-24 Members keyed by station name: a second member with the same name takes the first one's record (`link.rs:813-826,888`).
- [x] R1-25 OSC input listens on every station (`core/protocol_plan.rs:305-334`), unlike DMX input → a broadcast OSC message runs once per station. (2 reports)

### T5. Fades and handovers (2 reports)
- [x] R1-26 Any Devices change (renaming a fixture, MasterReact, routes, MergeMode, Set Address, even the owner's draft edits) bumps `patch_generation` (`core/change.rs:195-197`); the channel tier rebuild drops the previous plan (`lumacore-dmx/src/plan.rs:251`) → every running fade snaps.
- [x] R1-27 (only a Go / Goto / Go- starts a cue fade, handovers snap, Highlight no longer the fade start, Store into a running cue retargets, a release fades by its cue (Q12)) Any change of who owns a channel fades by the cue's time (`plan.rs:296-311`): Clear, Unpark, Off Freeze, Blind, Off Sequence can fade (Clear → 10 s). Releases always snap (`plan.rs:300`), even with an out-fade. A fade started under Highlight starts from the highlighted value (`plan.rs:295`). Tests miss it: without an engine there is no previous plan.
- [ ] R1-28 Fade timing comes from the cue that stored the value (`lumacore-dmx/src/playback.rs:191-204`), not the cue you go to → Go-/Goto mix fade times.
- [ ] R1-29 Running state names cues by number, not GUID (`lumacore-playback/src/lib.rs:556,564`); the reference check skips it → after `Store Cue 3`, `Go`, `Oops` a sequence sits in a cue that doesn't exist.
- [ ] R1-30 (part fixed: Goto keeps fader/rate/paused/starter, > 3 decimals refused, `Go Cue 5` refused (Q15) — merge renumbering open) Goto rebuilds the running state (fader, rate, paused, who started it) (`core/commands/playback.rs:190-204`); `Go Cue 5` parses the cue and ignores it; `cue_key` rounds (`Store Cue 1.9999` → Cue 2, `1.0004` overwrites Cue 1); a merge can renumber a clashing cue to 5.001 and change tracking.

### T6. Values silently lost or ignored
- [x] R1-31 (Q16 b: an equal value is stored as a blocked value; a stored value replaces the cue's own in every part) `Store Cue` drops a programmer value equal to what tracks in, leaving the cue's own different value (`core/commands/playback.rs:140-156`). Cue 1 Dim 50, Cue 2 Dim 80, `At 50`, `Store Cue 2` → still 80. With tracking off it drops needed values; only part 0 is cleaned. `CueObject.blocked` is never read; §27 "blocked" means something else. (2 reports)
- [x] R1-32 `If` parsed then dropped on value lines (`control.rs:741-786`, `groups.rs:278-289`) and on On/Off/Full/Zero (`target_fixtures`, `control.rs:355`). (2 reports)
- [x] R1-33 (issue 07 built: quoted words reach handlers with their quotes; test `quoting_tests`) Quoted names don't work: `tokens::words` strips quotes and the quoted flag (`core/commands/tokens.rs:133`); parser/groups only take a name when it starts with `"` (`core/parser/mod.rs:169,195,234`, `groups.rs:177,525`). `Group "Wash"`, `Fixture "Spot 1"`, `Store Group "Front"` fail; quoted `"All"`, `"/v"`, `"/watch"` still act as keyword/flag. Parser tests use `test_tokens` (keeps quotes) and pass. → issues/07
- [x] R1-34 Four separate range grammars (parser, `groups.rs`, `universe.rs` + `programmers.rs`, `move_cmd.rs` + `edit.rs`) disagree on reversed ranges, gaps, names, aliases (`Fixture 1 + G 3` fails) and `-`. `parse_selection` doesn't check all tokens were used.
- [x] R1-35 (Label Thru needs a name or /Clear; Delete Fixture Thru deletes the range; Group Thru skips missing groups — Q6) `Delete Fixture 1 Thru 5` deletes only fixture 1 (`core/commands/patch.rs:282`); `Label Fixture 1 Thru 5` (no name) names fixture 1 "Thru 5"; `Group 1 Thru 3` with group 2 missing selects 1, errors, and still files an Oops step.
- [x] R1-36 Values for attributes a fixture lacks are accepted (`At 50` on fixtures without Dimmer; `Attribute "pan"` is case-sensitive) and stored into cues, never output (`control.rs:224-256,362-417`).
- [x] R1-37 `At + 10` adds to the user's own programmer value or 0, not the fixture's output; a physical value counts as 0 (`control.rs:240-253`). `Attribute X At + 10` fails (one token passed, `:381`).
- [x] R1-38 Deleted/unpatched fixtures stay in every user's `selected_fixtures` and programmer (`clear_dangling_selections` covers only DataPool/Sequence/Timecode); `Store Cue` copies them into cues. (2 reports)
- [ ] R1-39 (part fixed: a number beats a name in Select — empty group recall and `Please` are spec gaps) Selection: recalling an empty group after a value was set clears the selection (`lumacore-tree/src/programmer.rs:307`); `Select Sequence 3` picks sequence 1 if it is named "3" (`core/commands/selections.rs:63`); `Please` toggles the whole programmer.
- [x] R1-40 `Copy Group a Thru b At c /Overwrite|/Merge` with overlapping ranges copies the same group down the line (`groups.rs:699-720`). A reference console fixed exactly this.

### T7. Access control
- [x] R1-41 DeskLock bypass: `allowed_while_locked` passes any line whose first two words include DeskLock/NormalTheme (`core/devices.rs:61-64`) → `Shutdown DeskLock /NoConfirmation` runs from a locked device.
- [x] R1-42 (part fixed: who locked is stored, only they or Admin unlock; REST view lines and the layout PUT name their device; a device that connects while the station is locked starts locked; a REST line with no device runs — decided) New browser tabs are new, unlocked devices; `On DeskLock Station` locks only known devices (`devices.rs:51-99`). `/api/unlock` unlocks any device for any user whose password checks (no password → always); who locked isn't stored; REST commands and the layout PUT carry no device; locks are memory-only (restart unlocks).
- [x] R1-43 Claims bypassed: leaving Blind (`core/objects/modes.rs:42`) and `On` (`control.rs:322-325`) call `programmers::take_over` without checking `Claim Fixture`; `live_holder` ignores Blind users.
- [x] R1-44 `held_elsewhere` (`core/objects/mod.rs:405-426`) only catches `<Object> <number>`: misses Thru, names, tree paths, owner-first forms, Copy/Move targets, imports.
  Found while fixing: `Delete DataPool 1 Group 3` (owner-first) was refused — built with R1-55.
- [ ] R1-45 Two role sources: the command gate reads `User.role`, the `Set` gate (`core/props.rs:192-205`) the profile's role, and a user with no profile counts as Admin. `UserProfile.role` still exists and can arrive via `Import UserProfile /Overwrite`. `ensure_admin` doesn't restore an existing Admin's role. → issues/10
- [x] ~~R1-46 Guest can probably read password hashes with `Cat` (`core/vfs.rs:879-887`) — **unverified**, try as Guest. → issues/01~~ — checked 2026-10-13: no word shows a hash (`cat`, `Info`, `Dump`, `List`, `grep` as Guest and Admin; test `security_no_word_shows_a_password_hash`). The host disk through `Drives` stays issues/01.
- [x] ~~R1-47 A Blind user's Highlight and Solo still reach the stage (`core/app_state.rs:459-467`).~~ — decided (Q17 b, §27): Blind hides only the programmer; Highlight and Solo still reach the stage.

### T8. Patch and DMX
- [ ] R1-48 Non-contiguous GDTF offsets (`Offset="1,3"`, common on GDTF Share): `lumacore-gdtf/src/importer.rs:495` keeps the first offset + byte count; `lumacore-dmx/src/render.rs:157-165` writes bytes side by side → fine byte on the wrong channel; footprint too small (`importer.rs:446`) → next fixture overlaps. No sample covers it.
- [ ] R1-49 Staged patch half live: the engine renders the live patch, but `reconcile` → `desired()` reads routes from the draft (`core/protocol_plan.rs:231`) → deleting a routed universe in a draft stops it on the rig. Routes/merge modes also blocked or drafted during a draft (`dispatch.rs:140-155`), against §11. → issues/11
- [x] R1-50 `Patch … At` on a missing universe reads stdin on the state thread (`core/commands/patch.rs:70-92`) → server started from a terminal freezes; breaks the one-dialog rule; the user doc says the universe is created.
- [x] R1-51 (the tier keeps per-slot channel spans; the tick merges whole 16/24-bit channels) DMX input merges per byte (`render.rs:307-340`) → a 16/24-bit attribute can take coarse and fine bytes from different sources.
- [x] R1-52 (+ Q20: a changed footprint is taken where the slots are free, else listed) Load doesn't check the patch: overlap lets the last fixture win silently (`lumacore-tree tree.rs:594`); a break at address 0 panics in debug (`universe.rs:100`); `readdress_fixture` (`patch.rs:180`) can then free another fixture's slots; the re-derived GDTF footprint isn't compared with `channel_count` (`show_file.rs:609`).
- [x] R1-53 Art-Net nodes keyed by IP only (`artnet/source.rs:192,254`) → a node with > 4 ports keeps only its last reply; one failing unicast target ends the send loop (`?` at `:324`).
- [x] R1-54 (wire-universe clash refused, priority clamped to 200, `*.GDTF`, "not pinned" (Q22), PacketGap capped (Q21), 0-channel modes (Q24), /Mode /Revision (Q25); virtual dimmer → roadmap (Q23)) Smaller: two lines can send the same wire universe on one network; received sACN priority not clamped to 200 (`input.rs:234`); PacketGap (≤ 20 ms/universe) stalls the driver thread; `SO_REUSEPORT` sockets not pinned to a NIC can lose unicast; RGB-only fixtures ignore grand master, blackout and Solo; `Import library` misses `*.GDTF` (`import.rs:256`); footprint-0 mode → "start address 513" error; `Patch` always uses revision 0 / mode 0.

### T9. UI
- [x] R1-55 (owner-first `<verb> DataPool n Group m`; pinned windows send it; a gone pinned DataPool shows missing, Q30) A pool window pinned to DataPool 2 sends bare `Group n` for tap/Store/Move/Delete/drag (`index.html:2709-2748`, `terminal/tui_input.rs:270-290`), resolved in the selected datapool (`app_state.rs:732`) → deletes DataPool 1's object. A reference console fixed this twice.
- [ ] R1-56 (part fixed: the layout PUT keeps what existing windows hold (colour, scroll, settings, preset) — the web client capturing scroll (Q31) open) Window colour and scroll are wiped by any split, resize or tab move (`index.html:1587-1609`, layout PUT `crates/lumacore-http/src/endpoints/live_screens.rs:130-140`); the web client never captures scroll → `WindowPreset` doesn't hold everything (§30 "12c").
- [x] R1-57 (new windows store only changed settings; a refused save is shown; a saved window of an unknown type is dropped with a notice — Q32) The client PUTs back resolved settings (`live_screens.rs:181`) → every type default is frozen into the show; new defaults never reach touched windows. Unknown window type → PUT fails or the window is dropped; `saveLayout` swallows errors.
- [x] R1-58 (the layout PUT checks the revision; a member follows a profile op only where the saved layout changed — Q33 a) Layout races: the whole-layout PUT has no revision check (can overwrite `Set Self`/`Label Self`); on a member `follow_profiles` (`core/screens.rs:128-151`) restores the saved layout on any profile op and drops unsynced local edits.
- [ ] R1-59 (part fixed: the view bar refreshes after load, Oops, replication and profile import (change path) — views keyed by live screen number open) Multi-screen views keyed by live screen number (`core/views.rs:169,233`), which depends on connect order and differs per station. The view bar isn't refreshed after show load, Oops, replication or profile import.
- [ ] R1-60 (part fixed: the pool title escapes user text — the full grid rebuild (Q34) open) Pool windows (hand-built, 12e open) rebuild the whole grid on every feed change (`index.html:2644`), against §30 "never re-rendered under a finger"; the label goes into `innerHTML` unescaped (`:2642`).
- [x] R1-61 (Store View, Rename and Delete are station asks; `/NoConfirmation` skips; client confirms gone — was: (Q35 b: station asks, `/NoConfirmation` skips)) Client-only asks remain: Store View overlay, `confirmAction` / `promptText` for Delete and Rename — against the one-ask-path rule; a terminal tap deletes with no confirm.

### T10. Oops
- [x] R1-62 (OopsConfirmation checks every statement; Q10 b: a line with Oops/Redo runs statement by statement, each its own step) `X; Oops` on one line: Oops undoes the previous step, then the line files a step containing that undo and clears redo. OopsConfirmation only checks the first word (`Clear; Oops` skips it).
- [x] R1-63 Redo (`Meow`) has no conflict check, unlike Oops (`core/oops.rs:513`).
- [x] R1-64 `restore_subtrees` can fail half-way after the step was popped (`oops.rs:250`) → show half undone.
- [x] R1-65 Watches re-run any line, including show-changing ones, every ≥ 50 ms, filing an undo step and a replication op each time (`core/watch.rs`).

### T11. Robustness, other
- [x] R1-66 (Q39: last plan stays, stale_since latched, 3 panics in 60 s stop the compiler until a show load or restart) The plan compiler thread has no panic guard and isn't on the health board (`lumacore-dmx compiler.rs:68`) → a panic freezes the plan silently.
- [x] R1-67 (a call from a job is refused; HTTP on blocking threads; call_within timeouts — HTTP 10 s, shell 30 s, a late job never runs; watchdog after 5 s; `busy` in /api/station — Q40) `StateHandle::call` has no timeout and no guard against being called from a job; HTTP `execute`, `login`, `logout` block tokio workers (`console_bridge.rs:33,57,127`).
- [x] R1-68 HTTP screen endpoints `.expect()` the `live_screens` RwLock (`live_screens.rs:40-151`) → after one poisoning every request panics.
- [x] R1-69 Asset roots keyed by name: `Label AssetRoot` runs on the master and only moves the master's mapping (`core/commands/media.rs:935-948,994-1000`) → other stations lose media.
- [x] R1-70 (Q42: hashing runs on `core::background`; Import Asset stores via a follow-up line, /Verify reports to the System Monitor) `Import Asset` / `List Asset /Verify` hash multi-GB files on the state thread (`media.rs:1058-1161`) → every command stalls.
- [x] R1-71 Show names: a trailing apostrophe is lost, Windows-illegal characters pass; `keys::check_line` splits on a raw `;` (not the quote-aware splitter); saved key lines keep abbreviations, so a new registry word can change what they mean; selection paths scale quadratically at 35k fixtures (`Vec::contains`, linear `fixture_id_for_guid`).
- [ ] R1-72 The sweep reports ~2,900 member-vs-master divergences still open. → issues/20

## Docs and decisions out of step with the code
- [x] R1-D1 `Go` help (`core/commands/registry.rs:1737`, so the generated reference) still says "Without a sequence: Sequence 1. Cues snap for now".
- [x] R1-D2 (§8, §9, show-files.md, crate-map.md fixed) §8 opens with "plain JSON, zip never built"; `docs/user/…/show-files.md` says format 7 (it is 10) and omits `Usage.json`; §9 says autosave goes to `.autosave/` but timed autosave saves in place.
- [x] R1-D3 `roadmap.md:76` ticks preset pools as done (code and presets.md: Planned); `roadmap.md:389` "Blind suppresses programmer output" unticked but built and tested.
- [x] R1-D4 `presets.md` and `docs/user/macros/macros.md:22` describe the old "acknowledged" reply; `macros.md:227` example `Fixture Thru - Group 999` doesn't parse; it names a World object that doesn't exist. `presets.md:228-240` plans recipes with Pawterns and speed/phase.
- [x] R1-D5 (Q43 c: old forms no longer read; phaser fields stay until Effects) `value.rs:1-55,122-176` keeps phaser fields and parses old formats "so shows need no migration" — against §20 / no backward compat. (2 reports)
- [ ] R1-D6 (part fixed, Q44: Lo/Hi are effect-line properties; Stomp/Unstomp/IfStomp/Disable/Enable/Shuffle published as Planned — Effect → DataPool and DistributePresets → Pawtern wait for Effects) `Effect` lives under `Playback.Effects` (`lumacore-playback/src/lib.rs:20,420`, `object-tree.md:111`); §28 says DataPool. The Pawtern pool is still `DistributePresets` with `context: String` (`lumacore-tree/src/datapool.rs:151,265`). §27 still says 44 fps and contradicts itself on where playback state lives. Planned effect words (`Stomp`, `Form`, `Lo`, `Hi`, `Shuffle`) aren't published as Planned.
- [x] R1-D7 (Q11 a written into §15) §14 (shortest unambiguous abbreviation) vs §15 (short forms are aliases only); the code does both.
- [x] R1-D8 (Q45 a: fields kept, marked Planned in the code, roadmap items for them, triggers, OffCue, Cue Only, Delete/Move Cue) About 10 `SequenceSettings` / `CueTrigger` / `MibPreference` fields are saved but do nothing and aren't marked Planned; no roadmap items for triggers, MIB, OffCue, Cue Only, Delete/Move Cue. `docs/user/playback/sequences.md` talks about deleting a sequence; no Delete Sequence/Cue exists.
- [x] R1-D9 (colour/scroll now kept by the station as the docs say (patch 4); rights in descriptors → roadmap 12d (Q46 a); what "scroll" is waits for Q31) `docs/dev/ui-framework.md:102` and §30 contradict the code on scroll and colour; §30 says settings/bar descriptors carry rights, `SettingField` has none.
- [x] R1-D10 `core/parser/resolve.rs:123` says raw DMX, code compares percent; OSC module doc lists `/lumacore/fixture/…` as supported, the server says planned; holdover doc says 2 s, code ~3 s; §11 lists Pan/Tilt invert/offset as live-settable (not built, not on the roadmap).

## Spec gaps (nothing decided yet)
- **Numbers:** a cap on counts, ranges and pool numbers; non-finite numbers refused (E-CMD-008) and never stored; value validation after load; revisit "no input caps" (§25).
- **Time:** which clock LTP stamps use and a floor after load/join/merge; "never wall clock" to cover heartbeats and countdowns too.
- **Fades:** whose timing applies on Goto, Go-, releases and handovers; whether Go- asserts; what bumps `patch_generation` and whether fades survive a tier rebuild; what happens when the running cue disappears.
- **Session:** how a command is recognised as already run after a reconnect or election; Invite comparing which show is newer and a split base that survives a restart; frame size / chunking vs the 5 s timeout; inputs (DMX, OSC) on a member; election ignoring whether the winner can send output.
- **Patch:** what the patch draft owns vs what stays live (§11); GDTF offset lists, multiple logical channels, virtual dimmers; output/input continuity on a master change; patch mode/revision choice; multipatch; "merge per channel" is really per byte.
- **Robustness:** durability of station settings and stores; a guarded startup load falling back to CleanStart; a `dispatch_depth` cap and loop budgets before macros and follow cues; a state-thread stall watchdog; restart/report rules for the compiler thread; a way back for the engine after "down".
- **Command line:** quoting rules; one range syntax and the meaning of `-`; whether a line that errors may still change the show; Oops in compound lines; a redo conflict check; cleaning selections after a delete; empty group recall; what `If` reads.
- **Users / lock:** who may unlock a device and whether a lock survives a restart; which commands are Admin-only; how a member's exported profile reaches the master; autosave copy retention.
- **Programmer / presets:** engine reads a preset link or the copied value; what deleting a preset does to links (presets.md vs §22); channel functions / channel sets / physical values; activation groups; store options (/Merge, /Overwrite, /Remove, /CueOnly); Blind vs Highlight/Solo.
- **UI:** what a view holds and what "scroll" means; stored settings only as changes from defaults; a saved window whose type is gone; what a pinned pool window's gestures address; silent fallback of a gone link; screen numbering of views; layouts when users share a profile; layout write concurrency; a maximum pool size.
- **Effects (§28, §3):** §28's position formula `(clock − anchor) × BPM × Rate` jumps on every speed change, against its own "rate changes don't jump" (the most repeated phaser fix in the source); effects must run in the 60 Hz tick, not `resolve`; effect band vs priority bands (§7/§27); speed master units, BPM range, Tap/Learn/Double/Half; Go-/Assert/break cues with effect calls; NaN guards. §3 Pawtern (~20 lines) needs parameters, order, precedence, reset, Next/Previous, seeded Shuffle, Store naming.
- **Timecode (not built):** fps/drop-frame, signed offset, after-roll/freewheel, backup source, which station generates in a session, inclusive edges, GUID links from events, claims.

## Test ideas
- Property tests: Goto n == n × Go; store-then-play reproduces the programmer; an attribute lives in at most one live programmer; fades bounded and monotonic; a handover never fades unless a Go asked for it; range equivalence across every Thru parser; quoting round trip; Oops/redo on compound lines; "never two fixtures on one slot"; LTP order after load/handover; phase continuity for effects.
- Unit / systemtests: double run after reconnect (should fail today) and across an election; rebooted master rollback with two servers; wall-clock steps injected; throttled big frame; OSC broadcast; failover with clock skew; GDTF `Offset="1,3"`; 16-bit input merge; fades surviving a patch edit; draft-vs-live routes; Art-Net multi-bind; load with an overlapping patch; Copy with overlapping ranges; asset-root rename in a session; truncated-write / crash-during-save.
- Sweep: literals u32::MAX, nan, inf, `/0`, `1.0004`, quoted keywords and flags; findings for "error reply but changed the show", unused words, non-finite values in ShowData, fixture GUIDs that no longer exist, a running cue that doesn't exist, a locked device that ran something, a line that blocks; round trip through the real `.luma` layout; a multi-station mode.
- Fuzz: `.luma` bytes, GDTF import, profile import, the input merger; run cargo-fuzz for real without `-ignore_ooms`.
- e2e: views spec; pinned-datapool pool taps; scroll kept across feed updates; preset colour kept after a drag; a dialog on every window of a device.
- Budget: selection timing on a 35k-fixture show; render budget with effects running.

