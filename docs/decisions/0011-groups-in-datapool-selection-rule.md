---
id: 0011
title: Groups only in the selected DataPool; the add-or-replace selection rule
area: groups
status: built
date: 2026-10-11
built_in: step 7
supersedes: [archive §22 L1733–1735 "FixtureGroup fixture_groups removed", archive §22 L1787–1788 "recall searching every datapool"]
superseded_by: —
confirmed_by: [E-GROUP-001, E-GROUP-002, E-GROUP-003, groups_tests.rs]
source: [archive §22 L1733–1800, archive §22 L1815–1822, archive §22 L1863–1882]
---
# Groups only in the selected DataPool; the add-or-replace selection rule

## Context
Groups once lived in a separate `FixtureGroup` list on `ShowData`, recall searched every datapool, and selection behaviour was invented. Step 7 (2026-10-11) studied a reference console: a group stores only the selection with grid cells, and the add-or-replace rule below is what the console does.

## Decision
Groups live only in a DataPool (`Group { meta, members, master, preserve_grid, move_grid_cursor, pawtern }`; a `GroupMember` is a fixture GUID plus grid cell; order is the Vec order; fixtures only). `Group 3` means the user's selected datapool for store, recall, list and window alike (`AppState::user_groups`); `DataPool 2 Group 3` addresses another, and the owner stays with the line (`AppState::datapool_override` only while the verb runs). Every selection — Fixture, Group, lasso, window taps — goes through `Programmer::apply_selection(fixtures, cells, SelectMode::{Plain, Add, Replace}, preserve, cursor)`: the programmer keeps a value-since-select flag; a plain select adds while it is clear, the first plain select after a value replaces and clears it; `+` always adds; `Group N - Fixture 3` deselects. The selection carries grid positions and a per-user cursor. `Store` over an existing group is refused with the choice `/Merge`, `/Overwrite`, `/Remove` (Overwrite keeps the GUID). A deleted fixture leaves every member link in place; recall skips it, the reference check lists it. Group master mode is on the Group, its level in `ShowData.Playbacks.Live.Masters.groups`.

## Consequences
Never write `selected_fixtures` directly for a user selection, never look a group up across pools, never add a group-only store path — Store, Delete, Copy and Move are the one object door (see its record), and deleting a referenced group goes through its delete ask. Decided, not built: a references view per missing fixture with Resurrect / Delete links; sub-fixture members.
