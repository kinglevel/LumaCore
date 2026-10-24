# Findings for the user — things the docs work could not do

Docs-only runs write here what needs a code change or a decision. One line each, newest first; the gardening pass appends. Remove a line when it is done.

## From the preset pool build (2026-10-23)

- **Later, per record 0288:** Global mode (per fixture type: a fixture-type key in `PresetEntry`, a format bump), `Update` into the preset and the cues that use it, a live link (the render reads `Value.preset` — today a value keeps the GUID and its copy), embedded presets and recipes, editing values in the preset editor (it only shows them).
- **The use count** counts fixtures holding a preset's value in any programmer; running cues aren't counted yet.
- **Store from a non-Presets window** assigns no tags (only a Presets window's tap carries its window); a typed `Store Preset` from the command line keeps none either.
- **Feature groups by name:** a following window matches the encoder bar's group to the show's feature groups by name; a profile that renames a group shows no presets for it.

## From the view bar build (2026-10-23)

- **The current view gold** (record 0287): the Views pool and the bar don't yet mark the view a screen shows; a screen keeps no link to the view it was recalled from (a recall copies). Needs a runtime "recalled from" on the live screen, cleared by an edit.
- **A touch drag from a pool onto the bar**: the pool's drag is the mouse-only one in `kit/pool.js` (not the pointer core's `lift`); on touch, Assign + two taps and the button's Assign… list do it.
- **Move / Copy View** aren't verbs: a drag inside the Views pool is refused by the line; views renumber only by Store / Delete.
- **Macro's default verb (Go) is Planned**: a macro on a view button or a Macros pool tap replies "isn't built yet (planned)".

## From the device sheet build (2026-10-22)

- **A first GDTF write on a fresh checkout fails**: the import writes `library/fixturetypes/<sha>.gdtf` relative to the working directory without creating the folder (E-CMD-008 "No such file or directory"); old checkouts pass only because earlier runs left the folder in a dozen crates. Seen in `the_demo_show_builds_loads_and_matches_its_golden` and `the_patcher_can_answer_their_own_bar_apart` in a new worktree. Code: create the folder before the write.
- **`Park Fixture 1 Attribute "Pan" At 30`** (lum.md's example) is refused by the handler ("no matching attributes", E-CMD-009); `cmd_park` reads `Park Fixture 1 Pan At 30`. The lum test only parses the line. Either the handler or lum.md changes, decided first.

## From the md upgrade (2026-10-20)

- **Code header `//! Decisions:` lines** cite the archive's `§N "sub-entry"` titles; they stay valid (the archive keeps the numbering) but the records are the rules now. A later pass may add the record slug beside the `§`; code, not done here.
- **`lumacore-shutoff`** is still a binary / command while the syntax decision folds it into `Shutdown` (archive §33 52); `Watch` is both a verb and an object against one-name-per-word (§33 12 / 29, "to ask at 3e"); E-CMD-005 / 006 / 018 were to retire into E-SYN (§33 34) and have not.
- **Tests the archive names that do not exist**: `dialogs_tests`, `menus_tests` (the records cite the tidy rules and e2e specs instead); `views' column_json test` has no exact name. If they should exist, they are code.
- **`docs/user/reference/`** is generated: if a Planned word's status changed in code, `gen-docs` must be rerun and `the_reference_is_current` will say so; not run here (no cargo).
- **e2e config** is the one place the worker count and expected duration live; docs now point there instead of stating minutes.
- **Scripts**: `scripts/unix/docs_check` is the one script added (the stated exception); the loop helpers the plan sketches (`agent_loop`, claim / record) were not needed because the run was driven from one session; the nightly gardening agent runs the `doc-garden` skill with `.claude/md-upgrade.settings.json` as its fence.
