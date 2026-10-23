# Handoff — object registry: objects publish themselves (2026-10-10)

Branch: `termiham`. Decided with the user; implementation in progress.

## Problem

Verbs publish themselves (`CommandDef` in `terminal/commands/registry.rs` → help, verb
resolution, completion). **Object keywords don't**: every place that takes an object keeps its
own hardcoded list, and they drift. Found while adding `User` (not suggested, `list user` listed
profiles):

- dispatch: sub-keyword chains in `store` / `delete` / `label` / `assign` / `set`, bare-object
  arms (`fixture`, `group`, `window`, `view`, …), `create`
- per-module matches: `list::handle_list` (40 targets), `info::handle_info` (23),
  `show::handle_dump` (7), `edit::cmd_label` (3), `edit::cmd_edit`, `store::cmd_copy`,
  `store::cmd_update`, `move_cmd::cmd_move`
- completer: `OBJECT_TYPE_KEYWORDS`, `store` subtypes (two different copies), `delete`, `info`,
  `list` targets
- tree: `completer.rs` `TREE`, `navigation.rs` `nav_children_at` ("mirrors" it), `vfs.rs`
  `dynamic_children_at` / `resolve_object_by_id` path matches

## Decisions

- Objects **publish** their keyword, aliases, which verbs they take (with handlers), instance
  completions and tree location. Dispatch, completion, `List`, `Info`, help and the object tree
  all derive from that. A command that exists but isn't discoverable counts as broken.
- **Explicit registration**: each owning module defines its `ObjectDef`; one
  `OBJECTS: &[&ObjectDef]` list of references collects them (no `inventory` crate). A unit test
  checks the registry (unique keywords/aliases, every object has a verb, …).
- **All objects now**, not only the new ones.
- **The tree too**: objects publish their tree path, children and object views; the two static
  trees and the VFS path matches go away.

## Design

```rust
// terminal/objects/mod.rs
pub enum Verb { Call, Store, Delete, Label, Assign, Set, Edit, Copy, Move, Update, Create, List, Info, Dump }
pub type Handler = fn(&mut AppState, &[&str]) -> DispatchResult;   // args after the keyword
pub type Completer = fn(&AppState, &str) -> Vec<String>;           // instances for the next token

pub struct ObjectDef {
    pub keyword: &'static str,            // display form, "ViewButton"
    pub aliases: &'static [&'static str], // lowercase extra names: plural, "vb"
    pub description: &'static str,
    pub verbs: &'static [(Verb, Handler)],
    pub complete: Option<Completer>,
    pub tree: Option<TreeDef>,
}
pub struct TreeDef {
    pub path: &'static str,                                  // "Root.ShowData.Users"
    pub children: fn(&Root) -> Vec<DynChild>,
    pub object: fn(&Root, u32) -> Option<NodeView>,
}
pub static OBJECTS: &[&ObjectDef] = &[&windows::OBJECT, &views::VIEW, …];
```

- **Dispatch**: `<verb> <object> …` → the object's handler for that verb (args after the
  keyword); `<object> …` (an object used as a verb) → its `Call` handler. Non-object arguments
  (tree paths, `list root.showdata`) fall through to the existing code.
- **Completion**: after an object verb → the objects that support it; after `<verb> <object>`
  → the object's `complete`; after a callable object → its `complete`.
- **List-only things** (Library, Settings, Watches, Undo, Selection, Claims, KeyBindings, …) are
  registered as objects with just a `List` verb.
- Handlers may delegate to existing functions (e.g. `List Fixture` → `list::handle_list(state,
  ["fixture", …])`) during migration — the registry is the source of truth for which object /
  verb combinations exist.

## Steps

1. `terminal/objects/` types, registry, lookup; dispatch + completer consult it.
2. ObjectDefs for every object (new: Window, View, ViewButton, User, UserProfile, ScreenConfig,
   Screen, Show; existing: Fixture, FixtureType, Universe, Range, Output, Group, DataPool, Preset
   family, Sequence, Cue, Executor, Master, Effect, Macro, Timecode, EffectEngine, Server,
   Session, Station, Client, Network, Clock, Programmer, ShowData, UIEngine, variables, …).
3. Remove the hardcoded lists (completer, dispatch chains, `LIST_TARGETS`).
4. Tree: objects publish path / children / object view; one static structural tree; drop the
   `navigation.rs` mirror and the VFS path matches. Add `Root.ShowData.Users` (no password hashes).
5. Registry unit test; docs (`docs/dev/…`) and the CLAUDE.md "add a command" note.

## Status (built, compiles; tests not run)

- `terminal/objects/mod.rs`: `Verb`, `ObjectDef` (keyword, aliases, description, verbs, complete,
  tree), `TreeDef`, `OBJECTS`, `find`, `supporting`, `dispatch`, `complete`, `tree_at`,
  `tree_children_of`, helpers (`out`, `with_keyword`, `filter`, `quote`, `patch_claimed`,
  `json_view`); unit tests (unique names, every object has a verb, callable objects are command
  words).
- Object definitions next to their owners: `objects/patch.rs` (Fixture, FixtureType, Universe,
  Range, Output, DataPool), `objects/playback.rs` (Sequence, Cue, Executor, Master, Effect,
  Macro, Timecode, EffectEngine), `objects/system.rs` (Programmer, Selection, UserProfile,
  ShowData, Server, Session, Station, Client, Network, Clock, UIEngine, WindowType, Library,
  Settings, KeyBindings, Watches, Claims, Undo, GlobalVar), `commands/store.rs` (Group, Preset,
  Dimmer/Position/Color/… preset types), `windows.rs` (Window), `views.rs` (View, ViewButton),
  `screens.rs` (ScreenConfig, Screen), `users.rs` (User), `backup.rs` (Show).
- Dispatch: `objects::dispatch` before the verb match; the object chains in store / delete /
  label / assign / create / split / merge and the bare-object arms are gone (fallbacks list
  the objects that take the verb). `Set <setting>` still reaches the settings code.
- Completion: registry first (`Store ▸`, `Store Window ▸`, `Window ▸`); `OBJECT_TYPE_KEYWORDS`,
  the store / delete / info / list lists and `LIST_TARGETS` are gone; Fixture publishes `Thru`.
  `List ▸` also offers the tree's top nodes; `cd` / `ls` shorthands are walked from the tree.
- `/watch <object>` expands to `list <object>` for objects that publish List.
- Tree: `nav_children_at` is the one structural tree (the completer's `TREE` copy is gone) and
  merges the collections objects publish; `vfs::dynamic_children_at` / `resolve_object_by_id`
  ask the registry. New `Root.ShowData.Users` (no password hashes).

### Found while migrating
- `ShowVar` / `UserVar` (and `Set/Info/Delete …Var`) are documented in help but not
  implemented; they were suggested after `List` without working. Only `GlobalVar` (List) is
  registered now.
- `watch patch` / `watch gdtf` no longer auto-prefix `list` (not objects); use
  `watch "list patch"`.
- The `list` / `info` / `dump` implementations still match keywords internally — the registry
  decides which combinations exist and routes to them.
