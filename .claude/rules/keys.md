---
paths:
  - crates/commands/objects/lumacore-cmd-keys/**
  - crates/model/lumacore-tree/src/ui_keys*
  - crates/ui/lumacore-webui/web/kit/layouts/**
---

# Keys

Records: `keys-uikey-gestures`, `keys-in-shortcut-layers-and-press-cycles`, `keyboard-per-device`, `desklock`, `tag-squares-and-picker`.

- Esc only closes (record `tag-squares-and-picker`): a dialog, a picker, the suggestions, a window's focus; it never clears the command line (`tui_input` `route_key`).
- Keyboard binding → UiKey id → command line per gesture (Press, Release, Hold, ReleaseAfterHold, DoublePress) plus Meow lines. Command lines live only in UiKeys (`lumacore_tree::ui_keys`); never put a command line in a `KeyBinding`. `lumacore-cmd-keys` binds (`Bind` makes `user.<key>`, its GUID derived from the profile and the key id), checks lines on save and on show load (`Hook::ShowReplaced`); a stored key line with a bare number or a `/Watch` is refused (E-SYN-008).
- A binding belongs to a shortcut layer (`keybindings::Layer`, `slot`; `Bind … /Shortcuts "PRG"`): a mode's over All, a profile's over LumaCore's, an empty UiKey turns a key off. A letter, digit, sign or Space without Ctrl / Alt types in KBD and binds only in PRG / NAV (E-KEY-006). A UiKey's Press may be a cycle (`cycle`, `types` = the presses that type, `press_rows`); the terminal's `press_bound_key` runs the rule (the next press replaces what the key typed while the line still ends with it; no timer). PRG's keys are `keybindings::PRG_KEYS` rows, never a table in the terminal; a profile key's lines are `Set UIKey "<id>" "Press" …` (E-KEY-007 locked).
- Core keys (`keybindings::CORE_KEYS`) can't be bound (E-KEY-005); F1–F8 stay free: F9 / Pause DeskLock · Meow+F9 NormalTheme · F10 shortcuts · Meow+F10 ViewBar · F11 / PrintScreen Screenshot · Meow+F11 Shutdown menu · F12 full screen of the windows · Meow+F12 native FullScreen. Meow = Shift by default (`Set MeowKey`; the sidebar button lights while held).
- NAV's keys are the TUI's `NAV_KEYS` (`repl/tui_nav.rs`, the `?` overlay reads them; record `nav-mode-ls-rows-and-an-inspector`): they act only while the command line is empty or holds NAV's own range line, and a NAV key that changes something puts or runs a line; never a letter key bound in NAV that edits the show directly.
- The built-in keyboard's layouts are data files (`web/kit/layouts/*.json`: value, shifted, AltGr per key); the `Keyboard` mode lives on the device per screen. Key lines are checked, never trusted: a line that names a word the registry lacks fails the save with its code.
