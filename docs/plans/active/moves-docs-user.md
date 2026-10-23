# Moves out of docs/user (md upgrade, 2026-10-20)

One line per move, old path → new path. Each moved file opens with a "Design, not built"
paragraph naming its related records; its relative links were repointed.

- `docs/user/presets/presets.md` → `visions/presets.md`
- `docs/user/macros/macros.md` → `visions/macros.md`
- `docs/user/playback/timelines.md` → `visions/timelines.md`

Folders emptied by the moves and removed: `docs/user/presets/`, `docs/user/macros/`
(`docs/user/playback/` keeps `sequences.md`).

Created, not moved:

- `visions/effects.md` — the effects design written as a vision from the frozen log's §28
  (`docs/archive/decisions-2026-10.md`); it **supersedes the handoff copy** `docs/archive/handoffs/effects.md` (moved there by the handoff triage; a copy of
  §28 as of 2026-10-11 with three dead paths).

Deleted as duplicates (the `visions/metadata/` copies stay):

- `visions/UI-UX/FIGMA/LumaCore Lighting Console/src/imports/LumaCore_SB_Concept.md` (byte-identical to `visions/metadata/LumaCore SB Concept.md`)
- `visions/UI-UX/FIGMA/LumaCore Lighting Console/src/imports/LumaCore_SB_tab_bar_analysis.md` (differed from `visions/metadata/LumaCore SB tab bar analysis.md` only in its frontmatter title)

Links elsewhere that still name the old paths (outside this task's files; for the other tasks):

- `docs/meta/roadmap.md` (two items name `docs/user/presets/presets.md` and `docs/user/playback/timelines.md`) — the roadmap rewrite links `visions/presets.md` / `visions/timelines.md`.
- `docs/dev/architecture/crate-map.md` (the Docs columns and the doc → crate table name the three old paths) — the docs/dev task.
