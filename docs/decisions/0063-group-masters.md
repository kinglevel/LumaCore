---
id: 0063
title: Masters live in Playbacks.Live.Masters, never undone
area: playback
status: built
date: 2026-10-14
built_in: 12f part 1
supersedes: [archive §22 L1801–1804 "ShowData.Playbacks.group_masters GUID", archive §27 L2616 "Master 1 grand", archive §30 L5731 "ShowData.Playbacks.Masters"]
superseded_by: —
confirmed_by: [test "no master move files an Oops step", E-CMD-003]
source: [archive §22 L1801–1812, archive §22 L1883–1887, archive §30 L5726–5738, archive §30 L6249–6258, archive §30 L6553–6562]
---
# Masters live in Playbacks.Live.Masters, never undone

## Context
Step 7 built group masters with their level in `ShowData.Playbacks.group_masters` and the grand master as `Master 1` in an undoable pool. 12f took the master groups and addresses of established consoles, moved every master into §32's registry (picked over `ShowData.Masters`) and dropped `Master 1` without an alias.

## Decision
Masters are `Masters { grand: GrandMasters { master, on, highlight, solo }, groups }` at `ShowData.Playbacks.Live.Masters` — saved, replicated, never undone (the `Master` object is `Untracked`). The groups: `Master 2.1` is the grand master (`Master "Grand"`; Off = blackout, the level kept); `Master 2.2` / `2.4` cap everybody's Highlight / Solo (the engine gets each user's own strength × the cap); `Master Highlight|Solo` are your own, in `Runtime.Users.<u>.Modes`. `Master 1`, `2.3`, `2.5` and groups 3–5 reply planned (E-CMD-003; `Master 1` points at 2.1). Every master takes relative input through `playback::level_after_at` (`+ 10`, `* 2`, Full, Zero).

Group masters: the mode is on the Group (show data, undoable); the level lives in `ShowData.Playbacks.Live.Masters.groups` by group GUID, and deleting the group drops it. `Group 4 Master At 50 Fade 5` fades by the session clock (a reference console snaps); `Set … Master` snaps. `Group 4 At 50` stays select-and-set. The tick applies group masters after the grand master: Negative beats Positive, Scaling multiplies, Additive adds HTP; masters never touch Park or DMX input. A fixture's `MasterReact` (`Fixture.Luma`) is Grand | Group | None: None ignores every master and Solo; with Group, Additive group masters skip the grand master.

## Consequences
A master move is a gesture (see gesture-channel) and files no Oops step — a test checks it. Executors, pages, Flash, fader functions and speed masters come with "Playback depth". Never reintroduce an undoable master pool or a `Master 1` alias.
