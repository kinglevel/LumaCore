# lumacore-state

The station's state: `AppState`, the change path, Oops' state half, the property registry,
errors, dialogs, flags, the object door, the patch draft and the LumaSession outbox.

## Purpose

The lower half of the core (decisions §34 7, 15): it names nothing above it — no registry
table, no object, no command — so the objects and every `cmd-` crate build on one `AppState`. Handlers write the show directly; `change` fingerprints every `ShowData` subtree
before and after a mutating line and exactly the moved subtrees get the undo frame, the
replication op and the journal entry, nothing listed by hand. `props` is the one property
registry, `pools::door` the one object door, `dialogs` the one server-held question,
`tree_path` the typed `Node` table, `patch_draft` the draft a Devices claim edits,
`session_link` the outbox. Its disk side is `lumacore-luma`.

## Owns / Doesn't own

- **Owns:** `AppState`, `Reply`, `DispatchResult`, `UndoKind`; `change`; `oops` (snapshot /
  restore); `props` (`Props`, `Property`, `Access`); `defs` (`CommandDef`, `ObjectDef`);
  `dialogs` (`Ask`); `flags` (`GLOBAL_FLAGS`); `errors` (`CATALOGS`); `tree_path` (`Node`,
  `TreePath`); `pools::door` (`BTreePool`, `PoolRules`, `PoolOps`); `patch_draft`, `mvr_review`;
  `programmers`; `session_link` (`LinkHandle`); `stations`; `recovery`; `drives`; `passwords`;
  `text`; `tokens`.
- **Doesn't own:** the registry tables, the objects, dispatch, resolve and rights
  (`lumacore-objects`), the files on disk (`lumacore-luma`, `lumacore-drives`), the plan and
  the tick (`lumacore-render`), the parser (`lumacore-syntax`).

## Public surface

- `AppState` — the show, the runtime, the station, the generations, the draft.
- `Reply::ok` / `error`, `DispatchResult` (`Ask`, `Exit`), `build_info`, `parse_show`.
- `change::subtrees`; `oops::{snapshot_subtree, restore_subtree, restore_local}`.
- `Props`, `Property`, `Access::{Field, Keyed}`, `Props::apply`, `Props::live`.
- `errors::{render, render_code, reply}`, `dialogs::{Ask, open, for_client}`.
- `tree_path::{Node, TreePath}`; `pools::door`; `patch_draft::rebase`; `programmers::program`;
  `session_link::forward`.
- `register(&mut Registry)` — the error catalogs, the global flags.

## Rules

- The change path finds exactly the changed subtrees, and writes outside a command are never
  blamed on the next one. [test finds_exactly_the_changed_subtrees + test writes_outside_commands_are_not_blamed_on_the_next_one]
- Every crate below registers its catalog and its codes resolve in it.
  [test every_crate_below_registers_its_catalog + test crate_errors_resolve_in_the_catalog]
- Handlers read their flags by declaration; passwords are masked first.
  [test handlers_read_their_flags_by_declaration + test masks_login_passwords]
- A draft follows live edits cell by cell; a cell both sides changed is a conflict.
  [test untouched_cells_follow_the_live_side_and_both_changes_land + test a_cell_both_changed_is_a_conflict]
- A move swaps and keeps GUIDs; a copy gets new ones and asks before overwriting.
  [test a_move_onto_a_taken_number_swaps_and_keeps_guids + test a_copy_gets_new_guids_and_an_existing_target_asks_first]
- Every error reply carries a code. [tidy error_replies_carry_a_code]
- Code names a `Node`, never a string; the disk only through `drive`.
  [tidy tree_paths_are_typed + tidy core_reaches_the_disk_only_through_drive]
- It names nothing above it. [tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-fixtures, lumacore-sysmon, lumacore-registry, lumacore-tree,
  lumacore-base, lumacore-wire, lumacore-gdtf, lumacore-drives, lumacore-luma, lumacore-patch,
  lumacore-render, lumacore-syntax.
- **Used by:** lumacore-objects (which re-exports its modules), lumacore-terminal.

## Decisions

- `docs/decisions/0008-change-path-undo-subtrees.md`, `docs/decisions/0171-oops-categories-statements.md`
- `docs/decisions/0143-one-property-registry.md`, `docs/decisions/0141-property-kinds-values-reset.md`
- `docs/decisions/0186-one-server-held-dialog.md`, `docs/decisions/0113-flags-and-options-declared.md`,
  `docs/decisions/0106-tree-paths-typed-node.md`
- `docs/decisions/0133-one-object-door.md`, `docs/decisions/0262-staged-patch-draft.md`,
  `docs/decisions/0033-station-table.md`
- `docs/decisions/0227-layers-point-down.md`
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §4, §21, §26, §29, §30, §31, §33, §34, §35, §38

## Testing

- `cargo test -p lumacore-state` — `change.rs`, `errors.rs`, `passwords.rs`, `stations.rs`,
  `src/flags/tests.rs`, `src/patch_draft/tests.rs`, `src/pools/door/tests.rs`,
  `src/tokens/tests.rs`.
- Through the station's words: `cargo test -p lumacore-objects`,
  `cargo test -p lumacore-catalog-tests`.
