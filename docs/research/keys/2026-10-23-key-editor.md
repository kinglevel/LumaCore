# Key editor research: the shortcut modes today, and how good key editors work

Date: 2026-10-23. Read-only research for the UserProfile editor's Keys tab (mockups
`docs/dev/ui-kit/mockups/2026-10-23-profile-editor/01-profile-editor.html` and `02-keys.html`).

## 1. What LumaCore has today

**The shortcut mode.**
- `ShortcutMode { Keyboard, Programmer, Navigator }` lives in `crates/model/lumacore-state/src/app_state.rs`. Its labels are KBD / PRG / NAV, and `next()` cycles them.
- It is one field, `AppState.shortcut_mode`, with Programmer as the default. `with_client` in `repl/command_line.rs` swaps the path, user, screen and device per client, but not the mode, so one mode holds for the whole station.
- The wire's `Status.mode` is per client but carries that one value.
- `Shortcuts [Keyboard|Programmer|Navigator]` sets the mode (`lumacore-cmd-keys/src/shortcuts.rs`). F10 is the core UiKey `shortcuts`. NAV's Esc runs `Shortcuts Keyboard`.
- The plans disagree on where the mode should live:
  - `docs/plans/done/2026-10-09-panels-focus-key-routing.md` says "per view-session".
  - The same plan also says "the mode is global".
  - `docs/meta/roadmap.md` and `visions/notes/ideas-2026-10.md` say "per screen".

**What each mode does with a key.** In the terminal (`repl/tui_input.rs`), a key goes through these steps in order: dialog → NAV → routing → bound key (UiKey) → PRG letters → shared keys.
- **KBD:** typing with the ghost and completion.
- **PRG:** 13 hard-coded letters.
  - Each of these appends a word: a At (aa = At 100), t Thru, f Fixture, g Group, s Store, u Update, o Off, p Preset, e Edit, i If.
  - On an empty line these run: h Highlight, b Blind, l List.
  - Backspace on an empty line runs `reset`; ↑ / ↓ on an empty line run Previous / Next.
- **NAV:** the `NAV_KEYS` table in `repl/tui_nav.rs`, about 30 keys:
  - movement, Tab / Shift-Tab, 0–9 slot;
  - `/` filter, n / N hit;
  - Space mark, Delete line, r Reset line;
  - the y copies, m / ' bookmarks;
  - v changed only, p pin, c cd, L log, ? help;
  - Backspace `cd ..`, held `cd /`.
  - Those last two lines are hard-coded, although §33 decision 30 made them profile data from a shipped default profile.
- **Shared keys:** Enter, Space (abbreviations), Tab (complete, or `quicksheet` on an empty line), ↑↓ history, PgUp/PgDn, Ctrl-Z/Y, Alt-←/→ cd history, Ctrl-C, Ctrl-D.
- **The web page** forwards keys to the station's terminal (`index.html`, `websocket/shell.rs`). It handles some locally: the suggestion list, F12, the lock surface, PrintScreen on key-up. A plain Tab is always taken by the suggestions, so `quicksheet` never reaches the web.
- **The app** adds only its menu accelerators.

**Bindings.**
- `keybindings::VANILLA` is a static table of 8 bindings: Ctrl-L cls, Ctrl-H help, F9 / Pause DeskLock, F10 Shortcuts, F11 / PrintScreen Screenshot, F12 Sidebars.
- `CORE_KEYS` are F9–F12, Pause and PrintScreen with any modifier (E-KEY-005). The debug layer is always empty.
- `resolve_bindings` gives debug > profile > vanilla. A `KeyBinding` is {key, ui_key, description} and has no mode.
- Key names (`bind.rs`): Ctrl / Alt / Shift, then a single character, a named key or F1–F12; E-KEY-002 refuses anything else.
- `Bind` refuses no letter, although 0170, §30 "12a3" and `docs/user/keys/keys.md` say letters are never bound.
- There are 13 core UiKeys (`lumacore-tree/src/ui_keys.rs`); their Meow lines are in `CORE_MEOW`.
- The terminal fires Press and DoublePress only; the web adds Hold and Release.
- `KeyCache` starts with Alt as Meow until the profile loads, while the profile's default is Shift.

**Docs.**
- 0170 (key → UiKey → lines per gesture; letters never bound).
- Archive §17: shortcuts live on the profile, and "flagged in the hotkey editor".
- §30 "12a3": only F keys for core.
- §33 decision 30: NAV keys are profile data.
- 0169: the Keyboard setting lives on the device.
- 0291 / 0292: NAV's keys, and the Tree window reuses them.
- `docs/plans/done/2026-10-10-ui-keys.md`: a key editor window was deferred.

## 2. Other key editors

Sources: VS Code docs, the Blender manual, JetBrains docs, DaVinci Resolve and Premiere Pro guides, Apple's Final Cut Pro guide, Avid lessons, the Ableton manual, Resolume docs, the REAPER wiki, which-key.nvim, kitty / WezTerm / tmux docs, and lighting-console manuals (Hog 4, Eos, MagicQ, Onyx, QLC+).

- **A full keyboard linked both ways with a command list** (Resolve, Premiere, Final Cut Pro, Avid). Click a key to list what it does; find a command to light its keys (Final Cut Pro's Keyboard Highlight).
- **Colour by scope.**
  - Premiere: application-wide purple, panel green, and both colours when a panel overrides.
  - Resolve: two-tone keys.
  - Final Cut Pro: diagonal stripes for reserved keys.
- **The selected key lists every scope at once.** Resolve's Active Key panel shows Application, Media Pool, Timeline and so on.
- **Modifier layers.** Buttons relabel the keyboard, and so does holding the real modifier (Resolve, Final Cut Pro, Premiere, Avid's Shift).
- **Search by key and by command.** VS Code and JetBrains have a record-keys toggle in the search box.
- **Source and reset.** VS Code's Source column with `@source:user`, a revert arrow per changed row (Resolve), Restore per keymap (Blender).
- **Built-in sets are read-only; edits make a copy** (JetBrains). Export / import, and presets that mimic other apps (Resolve, Premiere).
- **Modes.**
  - Blender's per-editor and modal keymaps; REAPER's sections; Vim's mode maps.
  - kitty modes have an unknown-key policy and a timeout. WezTerm's key tables are a stack, with the active table on the status bar.
  - Hog shows "MAP" on the command line and drops to typing whenever a text field opens.
- **Learn mode in place** (Ableton, Resolume): everything bindable gets an overlay; click it, then press a key.
- **A which-key popup** shows what the next key does in this mode.
- **A troubleshoot log** of key → rule → command (VS Code).
- **Lighting consoles:** Hog, Eos, MagicQ and Onyx have fixed tables or macros. None has a visual editor.

**Pitfalls.**
- **An ANSI-only keyboard.** ISO adds `IntlBackslash` (`<>`) and the tall Enter; the Nordic layouts put `/` on Shift-7 and `?` on Shift-+.
- **Ctrl+Alt is AltGr on Windows,** and AltGr types `@ £ $ € { [ ] } \ |` on Nordic layouts. A Ctrl-Alt binding takes characters the command line needs.
- **The Mac.** F keys need fn unless the system setting is on; Mission Control and the OS take some combos; a browser never gets Cmd-W/T/N.
- **The terminal** can't see release or hold.
- **Colour as the only signal.** Add stripes, a lock, a corner or a bar.
- **A global binding silently shadowed by a scoped one** (Blender). Show the shadowing on the key.
- **Unsaved changes** (Resolve's Save button). LumaCore's lines are saved as they run.

## 3. What the mockup proposes (open questions, not decisions)

1. **A binding belongs to a mode:** `Bind "S" `Store` /Shortcuts "PRG"`; without the option it binds in every mode. This is a syntax change and needs a yes. `Mode` is taken by a fixture's DMX mode.
2. **PRG's letters and NAV's keys become shipped LumaCore defaults** (UiKeys `prg.*`, `nav.*`, bound per mode) instead of tables in the terminal. They can be changed, moved, turned off and reset on the profile.
3. **Letters always type in KBD;** in PRG and NAV any key can be bound.
4. **The mode lives per device screen.**
5. **Keys are named by what they type,** as today. The editor draws the device's layout and warns about AltGr.
6. **The numpad gets its own key names.**
7. **A read-only cheat sheet** of the current mode on F10-hold or `?`.
8. **Export / import, and Copy from profile.**
