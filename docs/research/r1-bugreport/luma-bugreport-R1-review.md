# R1 fixes — review against the docs (2026-10-13)

A read-only review of every commit on `bugfixes` (`main..2195b71`) against `decisions.md`,
the roadmap, CLAUDE.md, user/dev docs and the issues. Tick an item when it is fixed.

## Decisions taken after the review
- Delete in a macro or key line (nested): **never asks** (documented).
- `Delete ViewButton` doesn't ask; `Delete Window` asks; show objects ask.
- **Every statement of a line is its own step** (Q10), not only lines with Oops/Redo.
- Groups above 9999 already in a show: only **new** numbers are capped; old ones can be called,
  labelled, deleted and moved down.
- Copy / Move / Assign over a gap in the source **keep the gaps** (same spacing).
- Stored key lines: **fully expanded** — every unquoted unique keyword prefix is expanded (probe:
  `Lab Gro 9998 Sto` → `Label Group 9998 Store`); quoted text stays. A name that is a keyword
  prefix must be quoted.
- A background asset import that finishes after the show changed: **import anyway**.
- `List Asset /Verify` on a member runs **on the member** (asset folders are per station).
- Knock-in with claimed fixtures: **skip them and say how many**.
- A master with members invited by another station: **ask**; Use Mine tells the inviter to join;
  the dialog always closes.
- `Delete Fixture … Thru` (Patch claimed first): the question shows the **count**.
- Keys typed while the station is busy: **queued, never dropped** (only Enter waits).
- Q36 / Q45 letters corrected (Q36: kept as built; Q45 a: keep + Planned).

## Must-fix
- [x] 1 Delete on a LumaSession member: the stripped line is forwarded, the master asks again (one-shot → refused). Strip only just before the handler; forward with the flag. Link test.
- [x] 2 `Delete …; Oops` asks forever (split re-dispatches stripped statements at depth 0); a split part that asks drops the rest.
- [x] 3 `Delete Show` asks twice (its own /nc); e2e `pool-window.spec.ts:59-61` and `systemtest groups.rs:35` send Delete without /nc.
- [x] 4 OopsConfirmation lists steps before earlier statements run (`Clear; Oops`).
- [x] 5 Oops on a claimed object: `claim_changed_by` rolls back after the frame moved to redo and local state was restored.
- [x] 6 Owner-first DataPool lost in asks (Store /Overwrite buttons), in the Edit context, and left set after a panic.
- [x] 7 Input merge per channel: Park on a fine byte is overwritten (only the first byte's owner checked).
- [x] 8 Id pools: `allocate()` still overflows (`Move Fixture 1 At 4294967294` then Patch).
- [x] 9 Assign skips missing universes but numbers wires `start + i` (gaps → wrong wire).
- [x] 10 One range rule not everywhere: Copy/Delete Group (`parse_ids`), `Delete Universe 1 Thru 4`, `Thru - 3`; Copy can leave a partial copy.
- [x] 11 Key-line expansion: decided "fully expanded" — keep; document that names which are keyword prefixes need quotes.
- [x] 12 Relative values: attribute lookup case-sensitive before the mode spelling (`Attribute "pan" At + 10` → 10); physical values count as 0.
- [x] 13 A Patch that fails leaves the universe it created.
- [x] 14 Groups above 9999 in old shows can't be deleted/labelled (cap new numbers only).
- [x] 15 Shared fixture-type name refusal points to `Label FixtureType` (no such verb) and reuses E-PATCH-005.
- [x] 16 `split_base.json` kept after Use Master / Use Mine / LoadShow / CleanStart / snapshot; restore by show_id brings back a resolved split.
- [x] 17 Asset follow-up line built by string formatting (`;`/`"` in a file name); `/Hash` `/Size` unvalidated options.
- [x] 18 Keys dropped while busy change the line (`Delete Fixture 12` → `Delete Fixture 2`): queue keys.
- [x] 19 Master with members: the invite decision can't be closed (Use Mine / Cancel don't clear it).

## Should-fix
- [x] 6fee40e: logout revoke dropped when busy; Command/Tap/DialogAnswer/render still unbounded; watchdog only covers jobs; busy → HTTP 503; layout PUT lock check fails open on timeout.
- [x] 3699658: Verify from a member reports on the master (decided: runs on the member); one thread per job; sweep never drains; sandbox refresh while hashing; lost "verified" assertion.
- [x] 2bc08d3: role compare hides changes to other users; issue 20 f asked for `live_view`; relative skip also for text props.
- [x] fce792a: Keep Patch skips `claim_changed_by` (a draft can delete a claimed fixture).
- [x] c02e5b5: E-OUTPUT-003 only checked in `assign` (Set Output Target/Network can clash).
- [x] e9acbb1: `clear_dangling_selections` runs after read-only lines (no change pass).
- [x] 9a42479: values for unpatched fixtures kept.
- [x] 5ffba93: backup reads the whole .luma into RAM on the state thread; two saves in one second; long names; read errors skip silently.
- [x] fecb969: `follow_footprints` doesn't update `taken` (two fixtures can take the same slots).
- [x] e5cbf2b: a member's `Shutdown /Save` now pushes its stamps/profiles to the master.
- [x] 0d0a6bf: PacketGap cap per line, not per driver; saved values > 2000 not clamped.
- [x] 15c5278: the marker covers only the load; SIGTERM during a slow load leaves it.
- [x] 5fb218d: unknown windows also dropped on a member snapshot; not on Import UserProfile.
- [x] 6628d3b: `follow_renames` treats a LoadShow as a rename; `seen_names` never cleared.
- [x] 5fc7547: `"attribute"` string match misses the `Att` alias (use `registry::resolve`).
- [x] c68d8e0: Delete Sequence / Delete Cue / Move Cue not in `ObjectDef::planned`.
- [x] 14b086c: check `library/shows/demo.luma` needs regenerating.
- [x] 43b0333: `missing_ranges` took `enumerate_name`'s doc comment.

## Docs
- [x] lumasession.md:119 CLIENT_PROTOCOL 9 · decisions:550 E-OUTPUT-003 · §26 Q40 timeouts/watchdog ("HTTP never waits") · :3432 asset hashing on a worker · :2160 "a cue stores only changes" vs blocked values · §29 + sequences.md release fades and R1-62 · :1413 cap is Group-only · :1874 + fixtures-patch-output.md:127 PacketGap 0–2000/half period · fixtures-patch-output.md:63 /Mode /Revision, 0-channel · §20 + CLAUDE.md: the two load-time rewrites (Q20 footprint, Q32 windows) as decided exceptions · CLAUDE.md Oops/split, :69 owner-first, :39 shell 30 s = keys · http-api.md:85 `busy`, :185 `missing`/`datapool_missing` · show-files.md:130/:109/:88 · roadmap:374 backup tick, :207 Q31 answered · macros.md:227 · keys.md:68 locker or Admin · commands.md:46 range rule · issues 04/05/06/09 status + `issues/done/`.
