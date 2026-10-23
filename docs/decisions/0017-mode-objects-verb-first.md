---
id: 0017
title: Modes are objects: On, Off, Toggle verb-first, per user
area: modes
status: built
date: 2026-10-11
built_in: step 3; masters in 11b
supersedes: [archive §16 L1195–1197 "postfix Blind Off", archive §16 L1205–1206 "Black command", archive §16 L1213–1214 "global Highlight Solo overlays"]
superseded_by: —
confirmed_by: [E-CMD-005]
source: [archive §16 L1193–1229, archive §27 L2624–2626, archive §29 L3222–3230, archive §29 L3288–3290]
---
# Modes are objects: On, Off, Toggle verb-first, per user

## Context
The modes were global overlays with postfix words, and `Black` was a command. Step 3 hardening (2026-10-10) made them objects per user; step 4a gave them their output effects; step 11b (2026-10-11) built Highlight's output and the per-user masters.

## Decision
Modes are `ObjectDef`s (`core/objects/modes.rs`) with the verbs `On`, `Off`, `Toggle`, verb-first like every word: `On Blind`, `Off Highlight`, `Toggle Freeze`; a bare `Blind` toggles (its default verb, like pressing the key); The postfix `Blind On` is refused with E-CMD-005 naming `On Blind`. State is per user in `Runtime.Users.{User}.Modes` (`lumacore_runtime::Modes`): session runtime, replicated, never saved, always off at start. `Black` is not a word: blackout is the grand master switched off (its level kept). Each user has mode masters for Highlight and Solo (`ModeMaster`: level %, Toggle / AlwaysOn / Disabled; `Master Highlight|Solo At …|Mode …`) in the same place, never undone; the engine gets their strength through `UserLayer`. Highlight uses only the fixture's GDTF `DMXChannel@Highlight` (none → no change) on that user's live selection, above playback, below park; Solo dims the fixtures not selected, never above park. `On` / `Off` on fixtures, groups or the bare selection are knock in / knock out: `On` activates programmer values, `Off` removes them and bare `Off` also deselects.

## Consequences
A new mode is one more `ObjectDef` with the three verbs and a field on `Modes`, never a flag on the show or a global; its effect goes into the plan's value tier or the tick's modifiers (see freeze-lifts-programmer). Never add a postfix form or a second overlay path. Blind and Lowlight masters are decided, not built; so is pulling values in from cues and presets on `On`.
