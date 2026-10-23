# Plan · the UserProfile editor and the key editor (2026-10-23)

The user approved the mockups on 2026-10-23 ("looks good, implement"), with every recommended answer:
`docs/dev/ui-kit/mockups/2026-10-23-profile-editor/` 01 (the profile editor), 02 (a keyboard per
shortcut mode) and 03 (press cycles). Research: `docs/research/keys/2026-10-23-key-editor.md`.

## Steps

1. **The profile's Settings…** opens the whole profile in `<luma-settings>`, not the theme editor.
   - The theme editor opens only for a `Theme` line, drawn above the menus.
   - The missing setting groups are offered: Update, MVR import, Paint slots, Attributes.
   - The Theme, Paint and LockScreen cells show the user home.
2. **The key model and words.**
   - A binding belongs to a shortcut layer: ``Bind "s" `Store` /Shortcuts "PRG"``. Without the option it binds in every mode; `Unbind … /Shortcuts`.
   - A UiKey's Press can be a cycle: `Set UIKey "user.e" "Press" `Edit` `EditSetting``.
   - `Types` lists the presses that type onto the line; every other press runs, as keys do today.
   - Meow the same way ("MeowPress", "MeowTypes").
   - The other gestures are settable.
   - `FORMAT_VERSION` steps up with its upgrade step.
3. **PRG's defaults are catalog rows**, not a table in the terminal.
   - The LumaCore map from mockup 03: A = At, Alt+A = Assign › Label, and the rest.
   - The terminal resolves a key: the layer's bindings, then All, then the vanilla rows.
   - It runs the cycle rule: the next press replaces the key's word while it is still the last on the line, with no timer; a row that runs, runs the line.
   - The PRG letter table in `tui_input.rs` goes.
   - NAV's keys show in the editor from `NAV_KEYS`. A profile binding in the NAV layer wins over them.
4. **The Keys tab:**
   - a `keys` feed from `lumacore-views` (layers, bindings, keys' lines);
   - the kit element `<luma-keymap>` (the keyboard, the modifier layers, the key panel), added to the Showroom;
   - the bindings in a `<luma-grid>`;
   - every gesture is a `Bind` / `Unbind` / `Set UIKey` line.

## Not in this round

These are the mockups' later items:
- the shortcut mode per device screen (it stays one value);
- rebinding NAV's own built-in actions (they aren't command lines yet);
- hold = repeat;
- rows guarded by the line's first token;
- state-computed cycles;
- executors on Ctrl / Alt + F1–F8;
- the numpad's own key names;
- the F10-hold cheat sheet;
- export / import of a key map.
