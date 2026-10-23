# lumacore-registry

The catalog: every crate adds its declarations to it at startup, and everything reads them from
it.

## Purpose

One place that says what exists — the error codes, global flags, feeds, window types, tools,
built-in UI keys, commands, objects and menus (decisions §34 21; an object's properties stay
beside its row, 21.10). It holds
**pure data** (no `AppState`), so it sits at the bottom and any crate can add rows and read them;
what a command *does* is bound above it, in dispatch. Like a database's catalog: written once at
start, checked once by `freeze()`, read-only for the rest of the run.

## Owns / Doesn't own

- **Owns:** `Registry` (the builder: `enter`, `errors`, `global_flags`, `feeds`, `window_types`,
  `tools`, `keys`, `freeze`), `Catalog` (the
  frozen rows and their lookups), the row types every crate names (`Verb`, `FlagDef`,
  `FlagValue`, `FeedDecl`, `FeedKind`, `WindowType` and its parts, `SettingDef`, `SettingKind`,
  `Status`, `ToolDecl`, `ToolAsk`, `KeyDecl`, the command words in `command`: `Doc`, `Undo`,
  `Scope`, `Args`, `Rights`, `CommandClass`, `Reach`, `CommandFamily`, `CommandDecl`; `ObjectDecl`, `LumShape`; `MenuDef` and its parts), `FreezeError`, the one process-wide catalog (`install`, `install_with`, `get`,
  `installed`).
- **Doesn't own:** the rows (each crate's `register`), the handlers (dispatch, lumacore-objects),
  what the rows mean to the user (each crate's docs).

## Public surface

- `Registry::new`, `enter(crate)`, `errors(catalog)`, `global_flags(flags)`, `feeds(feeds)`,
  `window_types(types)`, `tools(groups, tools)`, `keys(keys)`, `commands(rows)`, `objects(rows)`, `menus(menus)`,
  `freeze()` — what a crate's `register(&mut Registry)` and an assembler use.
- `install(catalog)`, `install_with(register)` — once per process: the server, the tools, a test.
- `get()` → `&'static Catalog`: `error(code)`, `errors()`, `global_flags()`, `global_flag(word)`,
  `feeds()`, `feed(name)`, `window_types()`, `window_type(name)`, `tool_groups()`, `tools()`, `tool(id)`, `keys()`,
  `commands()`, `command(word)`, `objects()`, `object(word)`, `menus()`, `menu(name)`,
  `crates()`.
- `Verb`, `Status`, `FlagDef`, `FlagValue`, `FeedDecl`, `FeedKind`, `WindowType`, `SettingDef`,
  `SettingKind`, `ToolDecl`, `ToolAsk`, `KeyDecl` — the declarations' types.
- `installed()` — for a debug check that may run before the catalog is in.

## Rules

- A crate's `register` first calls the `register` of the crates it builds on, then adds its own
  rows; `enter` makes it count once. [test a_crate_enters_once]
- `freeze()` refuses a catalog that doesn't check — a duplicate error code across crates is
  found at start, not when the line runs. [test freeze_refuses_a_code_two_catalogs_declare]
- Every crate's error catalog is in the one catalog. [test every_crate_below_registers_its_catalog]
- No two global flags share a name or a short form. [test freeze_refuses_two_global_flags_of_one_name]
- A feed is declared once. [test freeze_refuses_a_feed_declared_twice]
- A window type's number and name are its own. [test freeze_refuses_a_window_type_number_or_name_twice]
- A tool sits in a toolbar group. [test freeze_refuses_a_tool_outside_the_toolbar_groups]
- A UI key is declared once. [test freeze_refuses_a_ui_key_twice]
- One word names one command. [test freeze_refuses_two_commands_of_one_word]
- A word is a command or an object, never both (§15). [test freeze_refuses_a_word_that_is_a_command_and_an_object]
- A menu's grid names an object in the catalog. [test freeze_refuses_a_grid_whose_object_isnt_in_the_catalog]
- Every crate on the registry has its `register` and a test-only hook that installs its catalog
  before its first test. [tidy crates_on_the_registry_register_and_install]
- The catalog is frozen for the run: installed once, never changed, no lock. [unchecked]
- It depends on base (and serde) only and holds no `AppState`. [tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-base.
- **Used by:** every crate that declares something (syntax, tree, patch, gdtf, dmx, http, state,
  objects, the station, tidy).

## Decisions

- [§15](../../../docs/archive/decisions-2026-10.md) — one namespace (freeze checks it as the tables move in)
- [§26](../../../docs/archive/decisions-2026-10.md) — error codes and their catalogs
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — the catalog (21), this README (18)

## Testing

- `cargo test -p lumacore-registry` — the builder and freeze's checks.
- A crate's tests get its catalog before their first test: its `lib.rs` has
  `#[cfg(test)] #[ctor::ctor] fn test_catalog() { lumacore_registry::install_with(crate::register) }`.
