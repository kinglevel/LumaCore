---
id: 0293
title: Presets link into cues; Update, Recast, Extract
area: commands
status: built
date: 2026-10-23
built_in: preset links (2026-10-23)
supersedes: [one-preset-pool-follows-the-programmer]
superseded_by: —
confirmed_by: [objects::preset::links tests, a_cue_value_from_a_preset_keeps_only_its_link, an_older_show_upgrades_step_by_step_to_today, a_linked_value_shows_its_preset_by_the_show_setting, a_linked_preset_is_used_by_its_cues, kit-update.spec.ts]
source: [session 2026-10-23, preset links rounds 1–3 — docs/dev/ui-kit/mockups/2026-10-23-preset-links/ (03 approved: "approved"), how a reference console updates, recasts and extracts presets (session research)]
---
# Presets link into cues; Update, Recast, Extract

## Context
Store Cue copied a preset's value with its tag, so a cue kept a stale copy. Record 0288 left a live link, Update into cues and embedded presets for later; this record supersedes only those parts of it (its pool stands). Not taken: a fallback copy beside the link, an Oops step per line, a grid instead of a tree.

## Decision
A cue value is plain steps or a link (`CueData`); a value called from a preset is stored as the link only, one per fixture and attribute, and the plan compiler resolves it through the preset each compile (the fixture's own entry, else the universal one; none = no value). The programmer remembers each value's home (`ProgValue.origin`) when a plain value replaces it. `Update` alone is one station ask with a tree (presets your values came from, the cues that follow as leaves, cues with plain values), Content OnlyExisting · AddNew · Overwrite (→ `/Merge` · `/Add` · `/Overwrite`; words of their own), Recast and Shield switches (kept on the profile). Its lines run as one statement — ``Update `<statements>` `` takes Update, Recast and Extract statements in backticks and runs them inside one line: one Oops step. `Update Sequence m Cue n /Original|/Active [/Shield]` writes plain values. `Recast` adds and drops links; `Extract` turns links into the preset's current values. Deleting a used preset asks, then extracts. The Device Sheet's `Show` is Value / Link / Both.

## Consequences
The show format bumped with its step (preset-tagged cue values become links); `CLIENT_PROTOCOL` bumped (`Dialog.tree`, selector texts, toggle flags, the `devices` feed's `show`). Preview in Blind is planned (no Blind output exists). Store Preset still flattens; embedded presets come with a cycle check — the tree already has loop rows.
