# lumacore-cmd-keys

The keys: keyboard shortcut → UiKey → command lines, and the words that bind them.

## Purpose

A binding only names a UiKey; the UiKey holds one command line per gesture (Press, Release, Hold,
ReleaseAfterHold, DoublePress) plus its Meow lines (decisions §17). This crate holds `Bind` (a
command line becomes the user key `user.<key>`, stored with the full words), `Unbind`,
`Shortcuts` (the shortcut mode), the UIKey and KeyBindings objects, the checks of a key's lines
— on save (`Bind` refuses an unknown word, a bare number, an undeclared flag) and after a show
replaced the last (its `Hook::ShowReplaced` flags them in the System Monitor) — and the E-KEY
codes. The core keys (F9–F12, Pause, Print Screen) can't be bound (E-KEY-005); F1–F8 stay free.
The keys' data stays in the tree (`UiKeys` on the profile, `Root.Resources.UIKeys`); the users'
crate may use this one, never the other way round (§34 25.4).

## Owns / Doesn't own

- **Owns:** the rows (`rows`: `ROWS`, `OBJECTS`, `HOOKS`), `Bind` / `Unbind` and the key names
  (`bind`), the checks (`lines`), finding keys and bindings (`lookup`), UIKey (`ui_key`),
  KeyBindings (`key_bindings`), Shortcuts (`shortcuts`), E-KEY-001…005 (`errors`).
- **Doesn't own:** the key data and the layers (`lumacore-tree` `ui_keys`, `keybindings`), the
  core key list (`lumacore-base` `keybindings`), what a key runs (the terminal, the page).

## Public surface

- `register` — the keys' rows, hook and error codes, after the kernel's.
- `all_keys`, `find`, `bindings`, `command_for` — the terminal's key handling.
- `parse_key_name`, `check_line`, `key_problems`, `check_after_load`.

## Rules

- Core keys can't be bound. [test core_keys_are_f9_to_f12_and_cant_be_bound]
- A key line is checked on save. [test a_key_line_with_an_unknown_flag_is_refused_on_save]
- A command line lives only in a UiKey, never in a binding. [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-base, lumacore-syntax, lumacore-syntax-derive,
  lumacore-sysmon, lumacore-wire, lumacore-tree, lumacore-objects.
- **Used by:** lumacore-terminal, lumacore-sweep, lumacore-tidy, lumacore-catalog-tests.

## Decisions

- [§17](../../../../docs/archive/decisions-2026-10.md) — keys: binding → UiKey → command lines
- [§30 "12a3"](../../../../docs/archive/decisions-2026-10.md) — the core keys
- [§34 25.4](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
- [0295](../../../../docs/decisions/0295-keys-in-shortcut-layers-and-press-cycles.md) — keys per shortcut mode, press cycles, `Edit KeyBindings`

## Testing

- `cargo test -p lumacore-cmd-keys` — Bind, the checks, the core keys.
