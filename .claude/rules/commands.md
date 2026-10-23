---
paths:
  - crates/model/lumacore-objects/**
  - crates/model/lumacore-state/**
  - crates/model/lumacore-tree/**
  - crates/model/lumacore-runtime/**
  - crates/model/lumacore-playback/**
  - crates/model/lumacore-tags/**
  - crates/foundation/lumacore-syntax/**
  - crates/foundation/lumacore-syntax-derive/**
  - crates/foundation/lumacore-registry/**
  - crates/commands/**
  - tests/catalog/**
  - docs/user/syntax/lum.md
---

# Commands, objects, dispatch, the kernel

Records: `one-registry-one-namespace`, `lum-grammar-one-parser`, `typed-handlers-derive-args`, `one-resolver`, `one-property-registry`, `set-one-typed-handler`, `bare-object-default-verb`, `default-object-per-user`, `owner-first-syntax`, `quoting-values-options`, `selection-ordered-one-range-rule`, `flags-and-options-declared`, `at-verb-on-target`, `dispatch-special-cases-are-hooks`, `change-path-undo-subtrees`, `oops-categories-statements`, `selections-one-only`, `groups-in-datapool-selection-rule`, `programmer-owned-by-user`, `programmers-ltp-fixture-claims`, `mode-objects-verb-first`, `claims-by-tree-path`, `guids-everywhere-links`, `node-meta-levels`, `errors-coded`, `reach-drive-door`, `command-sweep-dynamic`, `tree-paths-typed-node`, `one-object-door`, `catalog-one-registry`, `three-homes-for-a-test`, `presets-link-into-cues-update-recast-extract`, `object-paint-palette-free-theme-links`.

## Rows

- Verbs are `CommandDef`s in their family's file (`commands/registry/rows/<family>.rs`, `ROWS`; one family per command crate): `{ run, decl: CommandDecl { … } }`, the handler beside its row; names, `Doc`, `undo: ReadOnly | Untracked | Tracked(OopsKind)`, `family`, `scope`, `status`, `reach`, `rights` (+ `arg_rights`), `args`, `class`, `default_object`, `hold`, `fence`. Objects are `ObjectDef`s in their module, listed once in the family's `OBJECTS`: keyword, aliases, `Doc`, verbs + handlers (incl. `Call`, `List`, `Info`, `Dump`), `pool`, `delete_ask`, `complete`, `props`, `tree`, `reach`, `rights(verb, args)`, `default_verb`, `lum`. A row names only what differs from `BASE`. Dispatch runs `registry::run_of(row)`; readers use `registry::all()` / `objects::all()`; dispatch's special cases are `Hook` rows families push, never names in dispatch.
- Every name means one thing across both registries (`registry::resolve` → `Word`); an object is never also a command; one name per word, no abbreviation table (the registry aliases are gone; stored lines carry full words; personal `!` aliases on the profile are decided, not built). Never match on keyword strings elsewhere (tidy fails on a match naming two or more registered words; a shared value domain gets `// tidy: not-keywords (<reason>)`). A word that is not built is `status: Planned` / `ObjectDef::planned`, replies `isn't built yet (planned)` [E-CMD-003] and changes nothing; never a stub that acknowledges.
- New handlers are typed: a `#[derive(Args)]` struct implementing `commands::typed::Command`, registered as `run: typed::<C>`; the shim count only goes down.

## Lines

- `lum.md` is the language: the parser crate matches its grammar block; every ```lum example is a test (the crate's and the station's vocabulary, `tests/catalog` `lum/tests.rs`, `PENDING` only shrinks); the vocabulary is built from the registries (`core::lum`; golden `lum_words.golden.txt`). Any syntax change is a hard discussion first.
- Quoting: a quoted word is a value, a bare `/word` an option; handlers get quoted words with their quotes and store them with `tokens::unquote`; a name with spaces must be quoted; a line as a value goes in backticks. A filename goes through `drive::library_file` (relative to its `library/…` folder, no absolute path or `..`).
- Nested objects are addressed owner first in tree-path order (`DataPool 2 Group 3`, `UserProfile "Bob" Icon 3`; bare owner = your own); the owner stays with the line (an ask's buttons, an editor's context; `AppState::datapool_override` only while the verb runs).
- The one range rule (`parser::number_list`): `3`, `1 Thru 4`, `1 Thru`, `Thru 4`, joined by `+` / `-`; numbers that don't exist are skipped; every object word uses it; Copy / Move keep the spacing.
- Flags and options are declared (`FlagDef`, `choices`); an unknown flag is E-CMD-018; lines nested too deep are E-CMD-019. `<target> At <value>` is the At verb. A counted keyword alone sets `User.default_object` (replicated, no Oops step); a bare number or name is written out first (`dispatch::write_out_bare`: `3 At 50` = `Group 3 At 50`, `Store 2` = `Store Cue 2`); a stored key line with one is refused (E-SYN-008). A bare object runs its `ObjectDef::default_verb` (`Sequence 3` = `Select Sequence 3`); never a per-object catch-all `Call`.
- Every target becomes objects only through `core::resolve` (`resolve` / `resolve_one` / `fixtures`; names case-insensitive, owners, typed order, E-CMD-007); never a new lookup, range parser or `unquote` copy (`old_resolvers_only_go_down`). `Set <target> "<Property>" <value>` is one typed handler on the property registry (`commands::set::route`; no target-less `Set`; relative input; `Thru` spread; everything undeclared is read-only and `Set` says why).
- A Paint (`#rrggbb`, a palette word, `Theme "<slot>"`, `none`) is read only by `lumacore_base::Paint::parse` (every hex by `paint::hex_rgb`); `PropKind::Paint` on every object with a NodeMeta, `keyed_paint!` for keyed rows; theme slots are the theme `Paint` object's properties (an added one through `Props::open`).
- Errors: `E-AREA-NNN` from the crate's catalog; `errors::render` / `render_code` / `reply` (never "Error: Error:"); general codes E-CMD-006 usage, 007 no such object, 008 value, 009 nothing to act on, 010 not possible now, 011 no user, 012 file / disk, 013 editor only, 014 Select kinds, 015 no settings, 017 busy. Every command line runs in `panic_report::guarded`.

## The change path, Oops, selections

- Dispatch fingerprints every `ShowData` subtree before and after a mutating line; the changed subtrees get the undo frame, the replication op and the `Runtime.Journal` entry; subtrees come from the exhaustive destructure in `change::subtrees`. Writes outside commands (load, sync, undo restore) call `ShowData::after_load()` and bump `AppState::show_generation`. Never a second walk of the show per line.
- Oops: one list per user (`Runtime.Users.{User}.Undo`, replicated, never saved), always the newest step; every statement of a line is its own step (rights checked for all first; one that asks stops the line and its answer runs the rest, `Ask::then_run`). Delete asks once, in dispatch (`dispatch::delete_first`; an object shapes it with `ObjectDef::delete_ask`, rows via `set_rows::RowDelete`); handlers never ask again. The category is only a label (`dispatch::step_category`). Station settings and watches ride as `UndoFrame.local`. `/NoOops` files no step; the profile's `preferences.oops` filters and confirms. A show load clears every list; playback and the modes are never undone.
- Each user has one selected DataPool, Sequence and Timecode on `ShowData.Users.{n}.selected` (`Select Sequence 3`, `Deselect`; other kinds E-CMD-014; deleting clears it). Bare `Go` / `Goto Cue` / `Off Sequence` use it; none → E-PLAY-003, never Sequence 1. `Store Cue` with none creates a sequence and selects it; Cue is Store's default object. `cd` never selects.
- Groups live only in the user's selected DataPool (`AppState::user_groups`); every selection goes through `Programmer::apply_selection` (plain select adds until a value is set, then replaces; `+` always adds); never write `selected_fixtures` directly. Programmer and selection live on `ShowData.Users.{n}`; a `UserProfile` is only views, screens, keys and settings. Set values only through `programmers::program` (LTP; Blind takes nothing; claimed fixtures refused); selecting claims nothing; `Claim Fixture n` locks one.
- Modes (Blind, Highlight, Freeze, Solo) are objects taking `On` / `Off` / `Toggle` verb-first (`Blind On` → E-CMD-005); state per user in `Runtime.Users.{User}.Modes` with the Highlight and Solo masters.
- Claims are keyed by GUID in `Runtime.claims` (replicated); `Edit` claims and releases on exit; `cd` never claims; changing another user's object is refused (E-CLAIM-001); `Claim` asks the owner, Admin `/Force`. Every stored link is a GUID; broken links are kept and listed (`core::references`, `List Reference /Broken`), never dropped. Meta comes in levels (`Identity`, `NodeMeta`); a number is only its pool's key.
- Reach: every row declares `Show | TreeShell | Drive`; `registry::line_reach` counts an object argument and a path naming `Root.Drives`; `core` touches files only through `core::drive`.
- Presets in cues (record `presets-link-into-cues-update-recast-extract`): a value called from a preset is stored in a cue as the link only (`CueData::from_programmer`); a programmer value keeps its home in `ProgValue.origin` when a plain value replaces it. Update / Recast / Extract write through `objects::preset::links`; the Update ask's lines run as one statement, ``Update `…` `` (only Update, Recast and Extract lines, each with its question answered: one Oops step). Deleting a preset that cues link extracts them first (`DeletePreset`), in the same line.
- The one object door (`pools::door`) does every pool write: `next_free` is the first unused number (cues append), Move swaps, Store / Copy ask once, identity is minted only in `insert` / `duplicate`; Copy / Cut fill a per-user clipboard of live GUIDs, Paste finishes; `Insert` shifts, `Compact` renumbers; `mv` / `cp` / `rm` run through the doors.

## Tests

- A line needing words from more than one command crate → `tests/catalog`; the kernel's machinery → its stand-in words (`commands/test_words.rs`); else beside its code. Test binaries install their crate's catalog from the `#[cfg(test)] #[ctor::ctor]` hook in `lib.rs`. The command sweep is generated from the registries at run time; never a hand-written command list.
