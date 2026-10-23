---
title: Code Structure
category: docs
---
# Code Structure

How every `.rs` file is laid out, how files and crates relate, and where shared code lives —
decided in [§34 of the archived log](../../archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15),
checked by the tidy rules in `crates/dev/lumacore-tidy` (each rule, its reason and its escape in
[docs/meta/tidy.md](../../meta/tidy.md)). The station only assembles (§34 24.1,
`the_station_only_assembles`).

The aim: well-designed crates used well, short files that say what they hold, and one place for
each thing — no two pieces of code that almost do the same.

## 1. Crates

Which crate owns what, the folders and the direction are [crate-map.md](crate-map.md); the
allowed edges are `EDGES` in `lumacore-tidy` (`crate_edges_point_down`).

- **One direction.** A crate depends only on crates below it. A new edge is an API change: add
  it to `EDGES`, with a reason.
- **The kernel is two crates**: `lumacore-state` (AppState, the change path, Oops, the props'
  types, errors, dialogs, the outbox) and `lumacore-objects` on it; the objects crate re-exports
  the state's modules, so callers name `lumacore_objects::…` and the two crates write `crate::…`.
  There is no `core` facade in the station any more.
- **Every crate sits in its group's folder** (`GROUPS`, `crates_sit_in_their_group`); the station
  is only `src/catalog.rs` and a thin main per binary.
- **A crate's value is what it does not depend on.** Keep the vocabulary crates small; put what
  only one crate uses in that crate. No small helper crates.
- **Never** `pub use` another crate's items wholesale; name what you re-export.

## 2. Files: one concept each

- **One folder per object, one file per verb** (nushell's commands, rust-analyzer's assist
  handlers):

  ```
  lumacore-objects/src/
    objects.rs                 OBJECTS — the one list
    objects/group.rs           REGISTRY: GROUP, its Props
    objects/group/store.rs     COMMANDS: StoreGroup · LOGIC: store_group_at
    objects/group/delete.rs    COMMANDS: DeleteGroup
    objects/group/copy.rs      COMMANDS: CopyGroup · LOGIC
    objects/group/master.rs    COMMANDS · LOGIC: the group master
    objects/group/select.rs    LOGIC: selecting a group
    objects/group/views.rs     VIEWS: tree children, the Info text
    objects/group/tests.rs     TESTS
  ```
- **Modules are `foo.rs` + `foo/`**, never `mod.rs` (`modules_are_foo_rs_not_mod_rs`; then
  `clippy::mod_module_files`).
- **Size:** a file should stay under ~500 lines; over **800** fails tidy unless it is in
  `LONG_FILES` with a written reason (a generated table, one state machine). Functions: clippy
  `too_many_lines` (100). A Part over ~500 lines, or a file that needs both COMMANDS and IO, is
  a sign to split.
- **Names:** a file is named after the one thing it holds (`store.rs`, `master.rs`), never
  `misc.rs` / `utils.rs` at a crate's root. Two files must not differ only by a plural or a
  prefix (`backups.rs` / `backup.rs`, `reference.rs` / `references.rs` are to be merged or
  renamed).

## 3. Headers — every `.rs` file

Five levels, after a reference console's system-test files. Banners are **generated, never drawn**:

```
./scripts/unix/headers --file groups        the file header
./scripts/unix/headers LOGIC                a Part header
./scripts/unix/headers --section "Store / Update" "Store your selection into …"
```

`banners_match_their_tag` re-renders every tag line and fails if the art above it differs;
`headers_stand_in_order` checks the shape below.

### 3.1 File banner and module doc

The file opens with its banner — its module name, or a short form of it (≤ ~11 letters, the
banner must fit 100 columns) — and the tag line; then the module doc in a fixed shape:

```rust
//  ██████╗ ██████╗  ██████╗ ██╗   ██╗██████╗ ███████╗
// ██╔════╝ ██╔══██╗██╔═══██╗██║   ██║██╔══██╗██╔════╝
// ██║  ███╗██████╔╝██║   ██║██║   ██║██████╔╝███████╗
// ██║   ██║██╔══██╗██║   ██║██║   ██║██╔═══╝ ╚════██║
// ╚██████╔╝██║  ██║╚██████╔╝╚██████╔╝██║     ███████║
//  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝     ╚══════╝
// FILE: groups

//! Groups: a DataPool's named fixture lists and their masters.
//!
//! Owns:          Store / Delete / Copy / Update Group, members, the group master.
//! Doesn't own:   the selection rule (lumacore-tree programmer.rs), the pool window (pools.rs).
//! Entry points:  GROUP, StoreGroup, DeleteGroup, CopyGroup, UpdateGroup.
//! Decisions:     §22 "Step 7", §33 3d3.
```

All four labels are required (`Doesn't own:` says where that lives). Only `use`, `mod`, `#![…]`
and `#[cfg(…)]` may stand before the first Part.

### 3.2 Parts — the categories of a file

A banner per category, a fixed vocabulary in a fixed order; a file uses the Parts it needs and
never invents others:

| # | Part | What goes in it |
|---|---|---|
| 1 | **TYPES** | the file's own structs, enums, consts, type aliases, ratchet lists |
| 2 | **INIT** | construction and startup: `new`, `Default`, builders, `spawn`, `start` / `stop` |
| 3 | **REGISTRY** | what the registries read: `ObjectDef`, `CommandDef` rows, `Props` / `Property`, `FlagDef` |
| 4 | **COMMANDS** | typed entry points: `#[derive(Args)]` structs + `impl Command` |
| 5 | **LOGIC** | the work: rules, mutations, calculations — no reply text, no I/O |
| 6 | **VIEWS** | reading out: tree children / views, feed JSON, List / Info text, asks and dialogs |
| 7 | **IO** | disk, sockets, threads — only in the I/O doors (`drive`, `net`, `protocols`, `show_file`, `backups`) |
| 8 | **HELPERS** | small private functions used only in this file |
| 9 | **TESTS** | `#[cfg(test)] mod tests;` (the body in `foo/tests.rs`) |

```rust
// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC
```

### 3.3 Sections — topics inside a Part

A star box, exactly 100 columns: a rule, the title, an optional 1–3 line purpose, a rule.

```rust
// *************************************************************************************************
// Store / Update
// Store your selection into a group of your DataPool; Update merges it in.
// *************************************************************************************************
```

They replace the old `// ── X ──` lines (still in older files until each is moved).

### 3.4 Steps — inside a long function

Only in a function over ~40 lines; more than ~5 steps means split the function.

```rust
fn store_group_at(state: &mut AppState, id: u32, mode: StoreMode) -> String {
    // ----- Checks -----
    …
    // ----- Merge / Overwrite / Remove -----
    …
    // ----- Reply -----
    …
}
```

### 3.5 Doc comments

Banners, tags, Sections and Steps are plain `//` comments — never `///` / `//!` — so rustdoc and
the generated reference stay clean. `///` stays what it is: one comment per public item.

## 4. Tests

- A module's unit tests: `#[cfg(test)] mod tests;` in `foo.rs`, the body in `foo/tests.rs` —
  one way (`tests_live_in_foo_tests_rs` counts today's sibling `*_tests.rs` and `#[path]` files).
- Where a test goes (§34 25.16): a line needing words from more than one command crate →
  `tests/catalog`; the kernel's own machinery → its stand-in words
  (`crates/model/lumacore-objects/src/commands/test_words.rs`); everything else beside its code.
- Behaviour tests are data-driven over the boundary — a command line in, a reply and a tree
  change out — not over internal functions. Every test runs on the rig (`demo::fresh_state()`).

## 5. Shared doors — one place for each thing

Each of these has one home; everything else calls it. `shared_doors_have_no_copies` counts the
patterns that give a copy away (each count only goes down), and review catches the rest.

| Thing | The door | Replaces |
|---|---|---|
| Files on disk | `drive` | any `std::fs` in core (tidy) |
| Objects from a target | `resolve` (`resolve`, `resolve_one`, `resolve_owned`, `slot`, `slot_owned`) | number / name lookups, range parsers (§33 57) |
| Show files | `show_file` | any other reader / writer |
| Error replies | `errors` (`render_code`, `reply`, `not_found`, `exists`, `usage`) | hand-written "not found." / "exists" / "Usage: …" text |
| The next free number | `pools::next_free` — **the first unused number, everywhere** (§34 13) | `(1..).find(…)`, `keys().next_back().map_or(1, …)` |
| A DataPool for one call | `AppState::in_datapool(dp, \|s\| …)` | `std::mem::replace(&mut state.datapool_override, …)` (not `?`- or panic-safe) |
| A scope while a child resolves | `scope::ScopedCell<T>` (one guard type) | four hand-made thread_local guards |
| Asks | `dialogs::{label_ask, name_ask, delete_ask}` | copied `Ask::new(…).input(…).button(…)` chains |
| Words and values | `props::{on_off, parse_colour, rgba_hex}`, `text::pct`, `tokens::unquote` | local parsers accepting other words |
| Plain Store / Delete / Label / Move / List | `objects::shared::collection::Collection` | hand-written CRUD per collection |
| An object's declaration | `ObjectDef::BASE` + builders | field-by-field `ObjectDef { … }` literals, per-object macros |
| A property | `Property::keyed` / `Property::named` | per-object property macros |
| Tree children / views | `TreeDef::btree(node, \|r\| &r.…)` | hand-written `*_children` / `*_view` pairs |
| A handler's errors | `Result<Reply, Coded>` with `?` | `match … { Err(e) => return out(e) }` |
| A fixture by number, second id or GUID | `PatchData::{fixture, fixture_mut, fixture_by_fid, fixture_by_id, fixture_by_guid, insert_fid, insert_fixture, remove_fid, fids, by_number}` (§40 5: the map's key is a `Slot`, never a number) | `patch.fixtures.get(&no)` and every `(no, f)` loop that took the key for the number |

Helpers that only one category uses go in that category's folder (`objects/group/util.rs`);
helpers two categories use get a named door here. Never a crate-root `utils.rs`.

## 6. Ratchets

Today's code doesn't meet every rule yet; those rules count and may only go down — and must be
lowered when the count drops (the test says to what). The counts live in the tests' constants
and in [tidy.md](../../meta/tidy.md), never here:

| Rule | Counts |
|---|---|
| `files_have_headers` | files without a file header |
| `files_stay_short` | `LONG_FILES` (files over the limit, each with its reason) |
| `modules_are_foo_rs_not_mod_rs` | `mod.rs` files |
| `tests_live_in_foo_tests_rs` | sibling `*_tests.rs` / `#[path]` test files |
| `shared_doors_have_no_copies` | copies per door (`COPIES`) |
| `old_resolvers_only_go_down` | number / name lookups outside `resolve` |
| `the_shim_count_only_goes_down` (`tests/catalog`) | handlers still taking `&[&str]` |

A file is moved to this shape once, when its area is touched anyway (§34 14).

## 7. A new or moved file — the checklist

1. It holds one concept; its name says which.
2. File banner + module doc with the four labels.
3. Parts from the vocabulary, in order; Sections inside; Steps only in long functions.
4. Under ~500 lines (800 hard).
5. Shared things through their doors (§5), nothing copied.
6. Its tests in `foo/tests.rs`.
7. `cargo test -p lumacore-tidy` passes.
