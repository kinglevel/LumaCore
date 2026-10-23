---
id: 0013
title: Scenes are state configurations, not timelines
area: playback
status: decided
date: 2026-10-11
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §6 L293–315]
---
# Scenes are state configurations, not timelines

## Context
Game shows and esports need the rig to know a state — which team is red, whether a player is alive, which booth is active — and to switch presets and pools on it, which a cue list does badly. The March design set Scenes apart from Sequences for that; the step 11a audit (2026-10-11) lists them as not built.

## Decision
A Scene is a stateful configuration with node-based logic, never a timeline (that is a Sequence). Scenes are DataPool objects (`DataPool.Scenes`), addressed by a two-part name — `Scene "WWTBAM"."FinalAnswer"` activates a state, `Scene "WWTBAM"."WrongAnswer"` transitions to another, `Scene "CSGO"."PlayerAliveOrKilled"` is a dynamic per-player state — and a fixture can take one (`Fixture 1 At Scene "WWTBAM"."FinalAnswer"`). Use cases: team management and colour assignment, per-player state, loading and unloading preset pools on a condition ("if Team.Color == Red, use this preset"). A future pool of abstract mappings with a node-based editor loads twenty teams into booths so the rig knows what is happening.

## Consequences
Not built. When it comes: a Scene is a pool object on the one object door with a `PoolDef`, its words declared in the catalog and in lum.md first (the dotted name needs the dotted-id discussion, see that record), a `ShowData` subtree with its arms and file entry, GUID links to presets and fixtures (see guids-everywhere-links), and its running state in `ShowData.Playbacks` beside sequences. Nothing in the engine or the show file refers to Scenes today, and no slot is reserved for them.
