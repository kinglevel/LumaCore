# lumacore-base

The bottom of the workspace: the value, identity, clock, undo and replication types every
other crate builds on, with no LumaCore dependency of its own.

## Purpose

What every domain crate needs and none may own alone (decisions §34 19.1): the one attribute
`Value`, a node's identity and meta, the GUID and the `Walk` that classifies every field, the
session clock the render tick reads, the fingerprint the change path hashes subtrees with, the
undo and replication frames, the error-code trait, the key-binding model and the shared thread
join. It knows no show, no tree and no station; it defines the shapes they agree on.

## Owns / Doesn't own

- **Owns:** `Value` / `Absolute` / `Step` and `next_stamp` (`value`); `Guid`, `Identified`,
  `Walk`, `Walker`, `IdentityReport` (`identity`); `Identity`, `Annotation`, `NodeMeta`,
  `Meta`, `TreeNode`, `IconRef` (`node`); the session clock (`clock`); `fingerprint::of`;
  `UndoStack`, `UndoFrame`, `ShowSubtree`, `OopsKind`, `OopsCategory` (`undo`);
  `ReplicationJournal`, `ReplicationOp` (`replication`); `Coded`, `ErrorInfo` (`error_code`);
  `IdPool`; `KeyBinding` (`keybindings`); `TreeEvent`; `InboundEvent`; `VariableValue`;
  `embedded_json`; `threads::join_until`.
- **Doesn't own:** what a subtree is (`lumacore-tree`), the images the change path keeps
  (`lumacore-state` oops), who disciplines the clock (`lumacore-clocksync`), the catalog the
  codes go into (`lumacore-registry`), any file or socket.

## Public surface

- `Value`, `Absolute`, `next_stamp` — the one value type; DMX bytes exist only in the tick.
- `Guid::named`, `Identified`, `Walk`, `Walker` — identity and the exhaustive field walk.
- `NodeMeta`, `Identity`, `Annotation` — meta in levels, saved as one flat `meta`.
- `clock::session_ms` and the rate / step entry points — lock-free, allocation-free reads.
- `fingerprint::of` / `into` — a hash of anything `Serialize`, no text, no allocation.
- `UndoStack`, `UndoFrame`, `ShowSubtree`, `ReplicationJournal`, `ReplicationOp`.
- `Coded`, `ErrorInfo` — a typed error carries its `E-AREA-NNN`.
- `IdPool`, `KeyBindingMap`, `TreeEvent`, `InboundEvent`, `join_until`.

## Rules

- A show type's identity comes only through its `NodeMeta`; a default meta never shares a
  GUID, and the walker finds duplicates, broken links and loops.
  [test a_default_meta_never_shares_a_guid + test walker_finds_duplicates_broken_links_and_loops]
- Value stamps only go up; old value forms are refused, never migrated.
  [test stamps_only_go_up + test value_round_trips_and_refuses_old_forms]
- The session clock is continuous and never runs backwards except by `step`.
  [test continuous_monotonic_and_rate]
- The same value fingerprints the same, a change moves it, and two shapes never collide.
  [test the_same_value_hashes_the_same_and_a_change_moves_it + test shapes_do_not_collide]
- Every vanilla key binding names a UI key. [test every_vanilla_binding_names_a_ui_key]
- A replication op carries its journal epoch and station.
  [test ops_carry_journal_epoch_and_station_and_restart_changes_identity]
- It depends on no workspace crate. [tidy crate_edges_point_down]

## Edges

- **Depends on:** nothing.
- **Used by:** lumacore-registry, lumacore-syntax, lumacore-tree and every model crate,
  lumacore-net, lumacore-render, lumacore-objects, the catalog tests.

## Decisions

- `docs/decisions/0007-guids-everywhere-links.md`, `docs/decisions/0006-node-meta-levels.md` — identity and meta
- `docs/decisions/0028-session-clock-rate-overlay.md` — the clock
- `docs/decisions/0008-change-path-undo-subtrees.md`, `docs/decisions/0052-one-line-costs-two-fingerprints.md` —
  undo frames and the fingerprint
- `docs/decisions/0021-master-executes-results-flow.md` — replication ops
- `docs/decisions/0129-errors-coded.md` — the code trait
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §12, §21, §24, §26, §27, §40

## Testing

- `cargo test -p lumacore-base` — every module's tests sit beside it (`clock.rs`,
  `fingerprint.rs`, `identity.rs`, `node.rs`, `undo.rs`, `replication.rs`, `value.rs`,
  `keybindings.rs`, `id_pool.rs`); the undo mirror test is a proptest.
