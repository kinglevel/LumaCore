---
id: 0263
title: Universe moves and deletes ask; fixtures lose the address
area: patch
status: built
date: 2026-10-18
built_in: 34g4 part 2b
supersedes: []
superseded_by: —
confirmed_by: [`objects/patch/universe_ask.rs`]
source: [archive §11 L757–766, archive §34 L9556–9563, archive §34 L9746–9747]
---
# Universe moves and deletes ask; fixtures lose the address

## Context
Issue 11 put fixtures and universes in the draft and left the wiring on its slots by number (see staged-patch-draft). A universe that moves or goes therefore meets lines that stay where they are; An established unpatch keeps a fixture in the show when its address goes.

## Decision
In the draft a universe moves to another number by its No cell or by Move to… for several (`Move Universe 3 + 4 At 20`; the door swaps or keeps spacing, see renumber-is-move-skip-by). `Move Universe … At …` asks when input lines stay on a slot it leaves — Move anyway / Cancel; MergeMode goes with the universe (its one home), the lines stay on their slots. Universe's one Delete question names the fixtures patched on it (in the draft, and the live ones while a draft is open) and the lines left on its slot; on yes the universe goes and those fixtures lose their address there and stay in the patch (groups, presets and cues keep them; `Delete Fixture` is what removes one). A slot without a universe: an output sends nothing for it, an input is ignored, the grid shows `no universe`; a new universe there picks the wiring up again; Keep's question names the lines left on empty slots.

## Consequences
Never unpatch silently and never delete a fixture with its universe. A change to what a universe carries goes through `universe_ask.rs`, the one place that shapes both questions.
