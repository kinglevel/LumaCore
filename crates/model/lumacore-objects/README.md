# lumacore-objects

The registry tables, every object with its verbs, dispatch, and what reads the registry:
resolve, rights, claims, the Lum vocabulary, menus, settings and the demo rig.

## Purpose

The upper half of the core on `lumacore-state` (decisions §34 7, 15, 16): the command families
push their `CommandDef` and `ObjectDef` rows into the catalog (`commands::registry::rows`), and
`commands::dispatch` runs a line — statements split, the rights gate, the change path, the asks,
forwarding to a master through the outbox. One resolver turns every target into objects; one
gate checks every line from every transport; one door writes every pool. It names nothing above
it: LumaSession work leaves as an outbox request, station work as a `DispatchResult`. The `cmd-` crates add their
families the same way; `demo::build` makes the rig every test runs on.

## Owns / Doesn't own

- **Owns:** `commands::dispatch`, `commands::registry` and its `rows/*` families,
  `commands::typed`, `door`, `set`, `store`, `oops`, `hooks`, `test_words`; `objects::{root,
  patch, playback, presets, protocols, modes, system, poolxml}`; `resolve`; `rights`; `claims`;
  `lum`; `menu_defs`; `settings_props`; `tools`; `vfs`; `protocol_plan`; `references`; `demo`.
- **Doesn't own:** `AppState`, the change path, props, dialogs, the door's writes
  (`lumacore-state`, re-exported by name), the station's words (the `cmd-` crates), the parser
  (`lumacore-syntax`), what a UI shows (`lumacore-views`).

## Public surface

- `commands::dispatch::dispatch` — a line in; a reply, an ask or an exit.
- `commands::registry::{all, resolve, run_of}`, `objects::{all, def_of}`.
- `resolve::{resolve, resolve_one, fixtures}` — the one resolver.
- `rights::require`; `claims`; `lum::vocabulary`.
- `demo::{build, ensure, fresh_state}`.
- `register(&mut Registry)` — the tables, tools, menus.

## Rules

- One rights gate for every line of every transport; passwords masked first.
  [test security_one_rights_gate_for_every_line + test security_passwords_are_masked_before_journal_undo_and_replication]
- Every target becomes objects through `resolve`, the one range rule; old lookups only go down. [test the_one_range_rule + tidy old_resolvers_only_go_down]
- Editors claim, `cd` never; a claimed object refuses every line.
  [test ownership_every_line_shape_is_refused_on_a_claimed_object + test ownership_cd_never_claims_and_claims_is_a_place]
- Every object has a unique GUID, every link resolves, a copy is new.
  [test identity_every_object_has_a_unique_guid_and_every_link_resolves + test identity_copies_are_new_objects]
- A keyword alone sets the default object, no Oops step; a bare number takes the change path. [test a_keyword_alone_sets_the_default_object_and_files_no_oops_step + test a_bare_number_takes_the_change_path]
- No keyword match outside the registry; typed `Node`s; the disk only through `drive`.
  [tidy no_keyword_lists_outside_the_registry + tidy tree_paths_are_typed + tidy core_reaches_the_disk_only_through_drive]
- Too deep nesting is refused; a planned word replies planned and changes nothing.
  [test lines_nested_too_deep_are_refused + test planned_words_reply_planned_and_change_nothing]
- The demo show builds, loads and matches its golden. [test the_demo_show_builds_loads_and_matches_its_golden]

## Edges

- **Depends on:** lumacore-fixtures, lumacore-drives, lumacore-luma, lumacore-sysmon,
  lumacore-registry, lumacore-state, lumacore-tree, lumacore-base, lumacore-wire, lumacore-gdtf,
  lumacore-mvr, lumacore-poolxml, lumacore-patch, lumacore-protocols, lumacore-net, lumacore-render,
  lumacore-syntax, lumacore-syntax-derive.
- **Used by:** every `cmd-` crate, lumacore-lines, lumacore-lumasession, lumacore-terminal,
  lumacore-views, lumacore-suggest, lumacore-server, lumacore-station.

## Decisions

- `docs/decisions/0111-one-registry-one-namespace.md`
- `docs/decisions/0124-one-resolver.md`, `docs/decisions/0121-bare-object-default-verb.md`,
  `docs/decisions/0123-default-object-per-user.md`
- `docs/decisions/0161-rights-one-gate.md`, `docs/decisions/0004-claims-by-tree-path.md`
- `docs/decisions/0133-one-object-door.md`, `docs/decisions/0245-objects-split-kernel-and-domain-crates.md`
- `docs/decisions/0094-demo-show-test.md`
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §15, §19, §23, §33, §34, §35, §40

## Testing

- `cargo test -p lumacore-objects` — the `*_tests.rs` files beside the commands,
  `resolve_tests.rs`, `ownership_tests.rs`, `identity_tests.rs`, `commands/dispatch/tests.rs`.
- The whole catalog and the goldens: `cargo test -p lumacore-catalog-tests`; the sweep is a
  long run only (`./scripts/unix/fuzz_long`).
