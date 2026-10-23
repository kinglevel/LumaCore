---
id: 0009
title: Select one DataPool, Sequence, Timecode; SelectFixtures selects fixtures
area: commands
status: built
date: 2026-10-15
built_in: step 11b; the verb split with the Lum syntax parts
supersedes: [archive §29 L3202–3203 "selected_datapool on profile"]
superseded_by: —
confirmed_by: [E-PLAY-003, E-CMD-014]
source: [archive §29 L3194–3215, archive §29 L3281–3284, archive §33 L7182–7188, archive §33 L7244–7247, archive §33 L7279–7290]
---
# Select one DataPool, Sequence, Timecode; SelectFixtures selects fixtures

## Context
Playback words needed a target without a sequence named, and the selected datapool sat on the profile. Step 11b (2026-10-11) gave each user one-only selections: `Select` selects objects as default objects, apart from the fixture selection; a many-object selection was discussed and dropped. The Lum syntax hardening (§33, 2026-10-15) split the verbs into Select and SelFix.

## Decision
Besides the fixture selection each user has one selected DataPool, Sequence and Timecode on `ShowData.Users.{n}.selected` (GUID links; saved, replicated, an Oops step in the Selection category). `Select Sequence 3`, `Select DataPool 2`, `Select Root.ShowData.DataPools.2`, `Select Timecode 1`; `Deselect Sequence` clears one; `Select` on any other kind is refused with E-CMD-014 naming the kinds it takes. `SelectFixtures` selects fixtures (Fixture, Group, Cue, Preset) through `Programmer::apply_selection`; default verbs follow: `Group 3` = `SelectFixtures Group 3`, `Preset 4.1` = `SelectFixtures Preset 4.1`, `Sequence 3` = `Select Sequence 3`; `Select Group 3` is refused naming `SelectFixtures`. The selected sequence is the target of bare `Go`, `Go-`, `Goto Cue`, `Off Sequence` and `Store Cue` (`AppState::selected_sequence`); none → E-PLAY-003, never Sequence 1. The selected datapool is where `Group 3`, `Preset 4.1`, `Sequence 5` resolve. Deleting the selected object clears the selection (`AppState::clear_dangling_selections`). `cd` never selects; `Clear` never touches them.

## Consequences
A new one-only kind (Page, World, Layout, Timer, Camera) is a field on `selected` and a kind `Select` takes; a new fixture-selecting word goes through `SelectFixtures` and `apply_selection` (see groups-in-datapool-selection-rule). Never guess a default sequence. Decided, not built: `Attribute "Pan"` / `FeatureGroup` / `EncoderBar` as `Select`, and `Executor 201` = `Select Executor 201` with the Master Section on the selected sequence (an empty executor clears it).