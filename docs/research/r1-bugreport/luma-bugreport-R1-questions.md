# R1 — open questions (grey items)

**Checked against the docs (2026-10-13):** see [luma-bugreport-R1-docs-check.md](luma-bugreport-R1-docs-check.md) — 16 settled (6 against ★), 6 contradictions, the rest open. Status and how to continue: top of [luma-bugreport-R1.md](luma-bugreport-R1.md).

Every R1 item was checked against the code, `decisions.md` and the related docs (2026-10-13). All
72 items are real (a few only in part); R1-72 is a duplicate of issue 20. Clear fixes are being
built; the items below need a decision first. **★ = recommendation.** Answer per number
(e.g. "Q3 b, Q7 ★, rest ★").

## Time and numbers
- **Q1 (R1-04) Session clock ahead of the master — may it step back?** a) forward-only, slow down to close a gap · ★b) step back only before the first lock, and a member holds rendering until locked; a > 1 s panic step rebases fades · c) steps either way, fix the §24 wording
- **Q2 (R1-12) `At 150` / `At -5`:** ★a) clamp to 0–100 everywhere (group master too, one rule) · b) refuse E-CMD-008 everywhere
- **Q3 (R1-11) Pool numbers:** a) cap · b) no cap, virtualized window · ★c) both: cap 1–9999 + only visible cells drawn
- **Q4 (R1-13) Memory blow-ups (OOM, 16 GB allocs) as fuzz/sweep findings?** ★a) yes: rss/malloc limits, amend §25 · b) no
- **Q5 (R1-19) Input size caps (§25 says none):** ★a) caps tied to existing limits: `.luma` entry ≤ manifest size ≤ 256 MB, stream frame/zstd ≤ 320 MB, shell line ≤ 1 MB · b) keep §25 (fix only the 1 GiB preallocation and the O(n²) shell scan, which are being fixed anyway)

## Command line
- **Q6 (R1-34, R1-08) One range rule for every `Thru`:** ★ keep typed order (§2) · missing ids are skipped, error only when none exist · open ranges `1 Thru` / `Thru 5` allowed · `-` subtracts inside the expression, a line *starting* with `-` deselects. Alternatives: ascending order, error on any missing id, no open ranges.
- **Q7 (R1-35) A statement replies with an error after it already changed something:** ★a) roll it back, no step filed · b) keep and file the partial change · c) per command
- **Q8 (R1-37) What does `At + 10` add to?** a) the fixture's current output · ★b) your programmer value if you have one, else the current output · c) as now: your programmer value, else 0
- **Q9 (R1-38) Programmer values of a deleted fixture:** ★a) removed in the same step (Oops brings both back) · b) kept but skipped everywhere · c) Delete refused while someone holds values
- **Q10 (R1-62) `Oops` / `Meow` inside a compound line (`Clear; Oops`):** ★a) refused (E-CMD-006) — they must stand alone · b) split into separate steps · c) only as the last statement
- **Q11 (R1-71, R1-D7) Abbreviations:** ★a) keep typed prefix resolution (write it into §15), but stored lines (keys, later macros) are saved with full words so a new registry word can't change them · b) refuse prefixes in stored lines · c) remove prefix resolution, aliases only

## Playback
- **Q12 (R1-27) Releases and Off Sequence:** a) always snap (today) · ★b) a release in a timed cue fades by that cue's timing; Off Sequence snaps until the OffCue exists · c) both fade
  (Handovers — Clear, Unpark, Off Freeze, Blind — will snap; only a Go/Goto/Go- starts a cue fade. That follows from the decisions and is being fixed.)
- **Q13 (R1-28) Goto / Go- timing:** ★a) the target cue's timing for every changed channel, Go- behaves like Goto (asserts) · b) the storing cue's timing (today) · c) a sequence "Goto time", default snap
- **Q14 (R1-29) The running cue disappears (Oops, merge, later Delete Cue):** a) the sequence turns off · ★b) it moves to the nearest earlier cue, snapped (off if none) · c) stays, listed as broken
- **Q15 (R1-30) `Go Cue 5` / `Go- Cue 2` (today the cue is silently ignored):** a) go to cue 5 with its timing, no assert (like a Go+ Cue) · ★b) refuse with E-CMD-006 "use Goto Cue 5" for now · c) alias of Goto
- **Q16 (R1-31) Store a value equal to what tracks in:** a) drop it (changes only) · b) keep it as a protected *blocked* value · ★c) drop by default, `Store … /Block` keeps it (later). Plus: `CueObject.blocked` → removed (see Q45)
- **Q17 (R1-47) A user in Blind — do their Highlight and Solo reach the stage?** ★a) no, Blind hides the whole user (probe a reference console first) · b) yes, as today (§27 literal) · c) per-user preference

## Patch and DMX
- **Q18 (R1-49) Routes and merge modes must stay live while a Patch draft is open (§11):** ★a) move `routes` + `merge_mode` out of `Universe` into a show-level protocols table keyed by universe GUID (format bump) · b) dual-write: those fields go to draft and live · c) change §11: they are drafted too. (Also closes issue 11.)
- **Q19 (R1-50) `Patch … At 99.1` when Universe 99 doesn't exist:** ★a) create it and say so (user doc) · b) ask Create / Cancel in the dialog
- **Q20 (R1-52) The re-derived GDTF footprint differs from a patched fixture's channel count (e.g. after the R1-48 fix):** a) keep the old count, list it · ★b) take the new count if the slots are free, else list a conflict · c) refuse the load
- **Q21 (R1-54) PacketGap × universes can exceed the line period:** a) refuse the Set · ★b) driver caps the total gap at ½ the period (shown in `Info Output`) + max lowered to 2000 µs · c) only lower the max
- **Q22 (R1-54) Share ports 5568 / 6454 with other programs on the host?** ★a) yes, keep SO_REUSEPORT, show "not pinned" warning · b) no
- **Q23 (R1-54) Fixtures without a physical Dimmer vs grand master / blackout / Solo:** ★a) virtual dimmer: masters scale the colour channels · b) masters scale colour, no virtual dimmer attribute · c) leave, list them
- **Q24 (R1-54) Mode with 0 DMX channels:** a) allowed, occupies nothing · ★b) refused (new E-PATCH-009) for now
- **Q25 (R1-54) Choosing mode and revision on Patch (always mode 0 / revision 0 today):** ★a) `Patch "X" /Mode "Ext" /Revision n`, default first mode + newest revision; ambiguous type name refused · b) mode only, always newest

## LumaSession
- **Q26 (R1-22) A master rebooted with its old disk show re-invites the IdleMaster that kept the session going:** ★a) the IdleMaster raises the existing decision (Use Master / Use Mine / Cancel, Merge greyed), default Use Mine · b) the IdleMaster's show wins automatically · c) as now + forced autosave. Sub: ★persist the split base so it survives a restart.
- **Q27 (R1-25) OSC input only where lines send:** ★ full `silence()` rule like DMX input (a lone IdleMaster without OutputWhenAlone also stops listening) · or only "not a member / not fenced"

## Users and lock
- **Q28 (R1-42) DeskLock — who may unlock, and does it survive a restart?** ★a) the user who locked it, or Admin with `admin`; memory-only · b) same, saved across restarts · c) any user with their own password
- **Q29 (R1-43) B prepared Blind values on fixture 5, then A wants to claim it:** ★a) the claim is refused while anyone (Blind included) holds a value · b) claim granted; B's values stay blind · c) claim granted; the merge ignores others' values on claimed fixtures

## UI
- **Q30 (R1-55) A pool window pinned to a DataPool whose GUID is gone:** ★ show "missing" (never silently fall back) · or follow the selected datapool
- **Q31 (R1-56) What is a window's saved scroll position?** a) pixels · ★b) first visible row/object · c) drop scroll from the floor plan and WindowPreset
- **Q32 (R1-57) A saved window whose type no longer exists:** ★a) kept as an "unknown type" placeholder (needs a kit element → mockup first) · b) dropped with a sysmon notice · c) refuse the load
- **Q33 (R1-58) Two users share a profile — do their live screens follow each other's layout edits?** a) yes, everywhere · b) only on load / Oops / view recall · ★c) only for the same user
- **Q34 (R1-60, R1-11) Pool window rebuild:** a) patch the hand-built grid now · b) design `<luma-pool>` (mockup → your approval → kit) · ★c) a minimal stop-gap now (diff by cell, keep scroll, cap auto-fill) + `<luma-pool>` mockup in 12e
- **Q35 (R1-61) Delete from a pool / view-bar gesture:** a) no ask, Oops covers it · b) the station always asks · ★c) profile setting `DeleteConfirmation Always|Never` (default Always), asked by the station like OopsConfirmation

## Files and robustness
- **Q36 (R1-14) `station_settings.json` unreadable at startup:** ★a) keep a `.bak` of the last good file and fall back to it; move the bad file aside; sysmon ERROR · b) refuse to start · c) defaults, bad file moved aside
- **Q37 (R1-16) Backups:** ★a) every save rotates the old file to `library/shows/.backup/<name>_<n>.luma`, keep 10; cap `.autosave/` at 20 per show · b) timed autosave writes only to `.autosave/` · c) leave, fix §9 text
- **Q38 (R1-17) A startup that dies hard (abort / OOM):** ★a) a `.starting` marker: if still there next start, skip the startup show · b) operator uses `--clean-start`
- **Q39 (R1-66) The plan compiler panics:** ★a) skip that generation, retry on the next change, report once · b) engine rule (2 in 10 s → down)
- **Q40 (R1-67) The state thread stalls:** ★a) `call` timeouts for interactive callers (HTTP 10 s, shell 30 s → E-CMD-010 "busy") + watchdog (sysmon WARN after 5 s, `busy` in `/api/station`) · b) watchdog only · c) nothing
- **Q41 (R1-69) Station maps (network → NIC, AssetRoot → folder):** ★a) stay keyed by name (works across shows), each station re-keys its own map when it sees a rename by GUID — written down as the one exception to "links are GUIDs" · b) keyed by GUID · c) GUID with name fallback
- **Q42 (R1-70) Long disk work from a command (hashing a 20 GB asset):** ★a) a generic background outbox: a worker, then a follow-up command line as the user · b) special case for assets · c) accept the stall, document it

## Docs vs code
- **Q43 (R1-D5) Phaser fields in `Value`:** a) remove now (format bump) · b) at the start of Effects · ★c) remove the dead legacy parsing + false comment now, fields with Effects
- **Q44 (R1-D6) Effect → DataPool, DistributePresets → Pawterns:** a) now · ★b) at the start of Effects (docs get a note now). And `Hi`/`Lo` (`hi` is already Highlight): ★ effect-line properties (`Set Effect 5.2 Lo 20`), not registry words; publish Stomp/Unstomp/IfStomp/Form/Disable/Enable/Shuffle as Planned
- **Q45 (R1-D8) Inert saved fields (priority, restart_mode, auto_start/stop, soft_ltp, xfade, MIB, swap/kill protect, CueTrigger, blocked):** a) keep + roadmap items · ★b) remove them (format bump), re-add when built · c) keep blocked + trigger only
- **Q46 (R1-D9) Settings/bar descriptors carry rights (§30)?** ★a) add a `disabled` reason, roadmap 12d · b) drop the sentence from §30

## Format bump
Several fixes change the saved show (running cue by GUID R1-29, GDTF offset lists R1-48, views by
profile screen R1-59, `UserProfile.role` removed R1-45, and Q18/Q35/Q45 if chosen). They will go
into **one** `FORMAT_VERSION` 11 with a regenerated demo show.

## Answers (2026-10-13, from the user; probes on a reference console and its manual)
- **Q8 b**: `At + 10` adds to your programmer value, else the fixture's current output (manual, Minus keyword: "reduce 10 % from the current dimmer value"; "the percentage readout").
- **Q11 a**: prefixes resolve when typed — a reference console does the same (probe: `Fixtu 2`, `Fixtur 3 Thru 4`, `Fi 5` select fixtures; manual: "each keyword has its own shortcut", ties by priority). Stored lines (keys, later macros) are saved with the full words.
- **Q16 b**: an equal value stored into a cue is kept as a **blocked** value (confirmed after a probe through the programmer, which the user judged inconclusive without real DMX).
- **Q22 a**: share ports, show "not pinned".
- **Q26 ★a**: the IdleMaster asks Use Master / Use Mine / Cancel (Merge greyed), default Use Mine; the split base survives a restart.
- **Q37**: one timestamped backup per save beside the named show, which stays one file (a reference console keeps up to 10). LumaCore: `library/shows/backups/`.
- **Q39**: keep the last plan, never retry the same input, a latched "stage output stale since change N" in Health + System Monitor until a compile works; 3 panics on different changes within 60 s stop the compiler (last plan stays) until a show load or an explicit restart. Never blackout. (Research: OTP restart intensity, Chromium GPU fallback, Kubernetes back-off, Ardour graph swap, Unity/Bevy failed shaders.)
- **Q44 ★**: effect Hi/Lo are effect-line properties; planned effect words published as Planned.
- **R1-42**: a REST line that names no device runs; devices that connect while the station is locked start locked.
- **Q2**: clamp (a reference console clamps in every readout mode: `At 150` → 100 %, `At -5` → 0 %, `Pan At 1000` → the end of the range; typed numbers are read in the profile's ValueReadout, Dimmer `At` stays percent). Clamp 0–100 % now; readout units and `At 10 Attribute "Pan"` are on the roadmap.
- **Q3 ★c**: pool numbers 1–9999 and only visible cells drawn.
- **Q9 ★a**: a deleted fixture's programmer values and selection go in the same step.
- **Q10 b (probe)**: each statement of a line is its own step and `Oops` undoes the newest (`Fixture 1` then `Fixture 2; Fixture 3; Oops /nc` leaves Fixtures 1+2; a Store before the line stays).
- **At before Attribute (probe)**: `At 10 Attribute "Pan"` sets Pan 10, the Dimmer untouched — built.
- **Format 11**: one bump later, collecting R1-48, Q18, R1-29/Q14, R1-59, R1-45 (Q45 keeps its fields, so not that).
- **R1-22**: the split base is saved on disk and survives a restart.
- **Q6 a**: missing ids in a range are skipped (the selection closes up, no holes); an error only when none exist. One range parser for every Thru. (User: keeping a Patch draft rebuilds the show and sends it out on the LumaSession — Keep is one replicated change today; check.)
- **Q13**: layers — show defaults (Goto = cue timing, Go back / Go fast / Go back fast = seconds) and a per-sequence override ("Default" = the show's); Go+ always the cue's timing. Plus **user timings** on the profile: an on/off switch for all of them, and each value either seconds or `CueTiming` (so a user overrides only what they want).
- **Q14 a**: the running cue gone → the sequence sits on the nearest earlier cue (snapped), off if none.
- **Q15 a**: `Go Cue 5` / `Go- Cue 2` refused for now (E-CMD-006, "use Goto Cue 5").
- **Q18 a**: routes and merge modes move to a show-level protocols table by universe GUID (format 11); closes issue 11.
- **Q20 b**: a changed GDTF footprint takes the new count if the slots are free, else a listed conflict.
- **Q21 b**: PacketGap total capped at half the period (shown in Info Output); max 2000 µs.
- **Q23**: roadmap — relations / virtual dimmer later.
- **Q24 a**: a 0-channel mode is allowed (occupies nothing).
- **Q25 a**: `Patch "X" /Mode "…" /Revision n`; default first mode + newest revision; ambiguous type refused.
- **Q31 b**: a view / WindowPreset saves the first visible row; the page reports it when storing.
- **Q32 b**: a saved window whose type is gone is dropped with a System Monitor notice.
- **Q34 c**: a stop-gap now (redraw changed cells, keep scroll, draw only visible cells), `<luma-pool>` mockup in 12e.
- **Q36 (none of a–c)**: keep as built — defaults in memory, the bad file is never written over (no `.bak`, never refuse to start).
- **Q38 a**: a `.starting` marker; still there at the next start → the startup show is skipped.
- **Q40 a**: call timeouts (HTTP 10 s, shell 30 s → E-CMD-010 busy) + a watchdog (sysmon WARN after 5 s, `busy` in /api/station).
- **Q42 a**: a generic background worker; the follow-up runs as a command line for the user.
- **Q45 a**: keep the inert fields, mark them Planned, add roadmap items.
