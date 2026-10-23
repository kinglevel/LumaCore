---
title: Keys
category: docs
---
# Keys

How keys work in LumaCore (decisions §17). Syntax of each word is in the
reference: [`Bind`](../reference/commands.md#bind),
[`Unbind`](../reference/commands.md#unbind),
[`UIKey`](../reference/objects.md#uikey),
[`KeyBindings`](../reference/objects.md#keybindings).

## Three layers

```text
keyboard key  →  UI key  →  command lines
F5               user.f5     Press: "List Fixture"
```

- A **keyboard binding** only names a UI key. It never holds a command line.
- A **UI key** is any button of the interface: a hardware-style key, a toolbar
  button, a plugin button, a key on your keyboard. It holds one command line per
  gesture.
- The **command lines** are ordinary Lum lines. A key works like firing a small
  macro.

Wings, the web UI, the app and the keyboard all trigger the same UI keys. The UI
only reports "key X, gesture Y, Meow held or not"; the server picks the line and
runs it, through the same rights gate as typed commands.

## Gestures

Each UI key has a line per gesture:

| Gesture | When |
|---|---|
| `Press` | the key goes down (runs at once) |
| `Release` | the key comes up |
| `Hold` | held past your hold time (default 500 ms) |
| `ReleaseAfterHold` | released after a Hold fired |
| `DoublePress` | a second press within your double-click time (default 300 ms) |

A `DoublePress` without its own line counts as a second press. Example: the ⚙
Menu key opens the menu on Press and does a Quick Save on DoublePress.

Timing is per user profile: `Set UserProfile "HoldTime" 600`, `Set UserProfile "DoublePress" 400`.

The terminal can't see key release on most systems. It fires `Press` (and
`DoublePress` by timing); the other gestures need the app or web UI.

## Meow lines

**Meow** is LumaCore's modifier key. Holding it while you press a
key runs the key's **Meow lines** — a second set of gestures. A key without a Meow
line falls back to its plain line.

- Meow is a held key, default Shift: `Set UserProfile "MeowKey" Alt|Ctrl|Shift`. On a Mac, F9–F12 need fn (or "Use F1, F2, etc. keys as standard function keys").
- On-screen and web keys get a latching Meow button.

## Core keys (F9–F12)

Core shortcuts are F keys and special keys, never letters; F1–F8 are free for your own keys and, later, executors. Core keys
can't be taken by `Bind` (E-KEY-005), with or without modifiers, in any shortcut mode.

| Key | Does |
|---|---|
| F9 / Pause | DeskLock — lock this device; on a locked one the key unlocks only for the user who locked it or an Admin |
| Meow+F9 | Normal theme (the factory preset); held = while held |
| F10 | Shortcut mode KBD / PRG / NAV |
| Meow+F10 | View bar |
| F11 / Print Screen | Screenshot |
| Meow+F11 | Shutdown menu |
| F12 | Hide / show the sidebars and view bar of this window |
| Meow+F12 | Native full screen of this window |

On a touch screen the same things are a tap away: the window menu (tap a
tab's dot) has **Full screen (bars)** (F12), **Screenshot** (F11) and
**Maximize**; the Shutdown menu has Full screen, Screenshot and this device's
**Screen config** (`Set Device "ScreenConfig" <n>|Follow`).

## Phones and tablets: the desk in miniature

A phone or tablet shows the desk's screen with every pane in its place. A pane
smaller than its window's minimum draws its content smaller (a miniature);
anything under 44 px there can't be tapped — a quick tap says *Too small — hold
to open …*. **Hold a pane (≈ 0.4 s, the ring fills)** to open that window full
screen at real size: ‹ › go to the neighbour pane, × (or holding the title)
goes back. With the profile's `HoldFollows` On (the default) every client of
the screen — the desk too — shows the window maximised (`Set Self Maximized
On|Off`); `Set UserProfile "HoldFollows" Off` keeps it on your device only. A
desk maximises a window from the window menu (**Maximize**).

## Three kinds of UI keys

| Kind | Id | Where | Who can change it |
|---|---|---|---|
| core | e.g. `menu`, `blind` | built in, `Root.Resources.UIKeys` (not saved) | nobody — fully locked |
| plugin | `plugin.<plugin>.<button>` | in the plugin, in `DataPool.Plugins` (saved with the show) | edit the plugin |
| user | `user.<name>` | on your user profile (saved with the show) | everyone on that profile |

`List UIKey` lists the keys you can use; `Info UIKey <id>` shows one key's lines.
When ids clash, core wins over plugin over user.

## Shortcut modes and the Programmer's keys

F10 switches the screen between three shortcut modes (decisions 0295):

- **KBD** — every letter, digit and sign types; F keys and Ctrl / Alt keys still work.
- **PRG** — single keys are console words: `F` Fixture, `G` Group, `P` Preset, `A` At,
  `E` Edit, `S` Store, `U` Update, `T` Thru, `I` If, `O` Off, `L` List, Alt+A Assign,
  Alt+C Cue, Alt+S Sequence, Ctrl+C Copy, Ctrl+M Move, Ctrl+D Delete, Ctrl+S Select,
  Ctrl+O On, Alt+T Stomp, Alt+G Goto, Ctrl+Alt+F SelectFixtures, Ctrl+P Pause, Ctrl+G Go,
  Ctrl+F Full, Delete clears (Selection › Active › All; held: all), `H` / `B` / Alt+F /
  Ctrl+Alt+S the modes, Space Go and Alt+Space Pause (Space and the mode letters only on an
  empty line), Ctrl+Z Oops, Ctrl+Alt+H Help. With Meow held some keys give another word
  (Shift+F FixtureType, Shift+P FeatureGroup, Shift+G Tag, …). Inside an open quote every
  key types. `List KeyBindings` shows the whole map of your mode.
- **NAV** — the tree's keys (press `?` there).

### Press cycles

Press a key again and its word changes in place: `E` puts `Edit` on the line, `E` again turns
it into `EditSetting`; Alt+A gives `Assign`, then `Label`; Ctrl+C `Copy`, `Paste`, `Insert`.
This works while the word is still the last thing on the line — type anything else and the key
starts over; there is no timer. Some presses run instead of typing: `Fixture 1` then `.` `.`
runs `Fixture 1 Zero`, `+` `+` runs `… At + 10`.

## Bind and Unbind

```text
Bind "F5" `List Fixture` /Description "Quick list"   -- F5 → user key user.f5, every mode
Bind "F6" "blind"                                    -- F6 presses the core Blind key
Bind "E" `Edit` `EditSetting` `EditRecipe` /Shortcuts "PRG"   -- your own E in PRG, a cycle
Set UIKey "user.prg.e" "Types" 1 Thru 3              -- its presses type onto the line
Unbind "E" /Shortcuts "PRG"                          -- LumaCore's E is back
Unbind "L" /Shortcuts "PRG"                          -- LumaCore's L off on your profile
List KeyBindings
```

- ``Bind <key> `<command line>` `` (the line in backticks, decisions §33 91) puts the line into
  the user key `user.<key>` (`user.prg.<key>` with `/Shortcuts "PRG"`) and binds the key to it.
  Several lines are a press cycle.
- `Bind <key> <UI key id>` binds the key to an existing UI key.
- `/Shortcuts "KBD" | "PRG" | "NAV"` binds in that mode only; without it the key works in
  every mode. A letter, digit, sign or Space (no Ctrl / Alt) types in KBD, so it binds only in
  PRG or NAV (E-KEY-006).
- A user key's lines are its properties: `Set UIKey "<id>" "Press" `…` `…``, `"Types"` (the
  presses that type; the others run), `"Hold"`, `"Release"`, `"ReleaseAfterHold"`,
  `"DoublePress"`, and the same with `Meow` in front. LumaCore's keys are locked (E-KEY-007):
  bind your own in their place.
- Bindings work at once — no restart.
- `Unbind` removes your binding; LumaCore's default for that key comes back. With none of
  yours there, it turns LumaCore's key off in that mode on your profile; `Unbind` again brings
  it back.
- Key names: `F5`, `Ctrl-R`, `Alt-S`, `Ctrl-Shift-L`, `E`, `.`, `Space`, …
  An unknown key name is refused (E-KEY-002).

Bindings resolve in three layers: debug bindings (debug mode only), then your
profile's bindings, then LumaCore's; within each, a mode's own binding wins over an every-mode
one.

## Checked command lines

- A line is checked when it is saved: an unknown word is refused (E-KEY-001); a
  planned word is saved with a warning.
- A saved line keeps **full words**: every unquoted word that starts a
  single command or object word is written out (`Lab Gro 3` → `Label Group 3`), so
  a new word in a later version can't change your key. A name is a word too: write
  names in quotes (`Label Group 3 "Sto"`), or `Sto` becomes `Store`. Quoted text
  is never changed.
- When a show loads, every user key and plugin button is checked again; problems
  go to the System Monitor ("Keys").

## Key lights

A UI key can show live state from mode objects: Blind lit while your Blind is on.
Each user sees their own modes (`Runtime.Users.{User}.Modes`).

## Built and planned

- **Built:** the three layers, five gestures in the model and in the app/web
  clients, Meow lines, core / plugin / user keys, `Bind` / `Unbind` per shortcut mode,
  press cycles, the Programmer's keys as LumaCore's rows, a user key's lines with `Set UIKey`,
  per-profile timing, checks on save and load, key lights for the mode objects.
- Still planned: the key editor in the profile editor, the shortcut mode per screen, NAV's
  own keys as rows, hold = repeat — see [the roadmap](../../meta/roadmap.md).

## Recovery: CleanStart

When a show or the UI is unusable, `CleanStart` (Admin) starts clean:
an empty show (it asks to save unsaved changes first), startup show None and
default station settings. Show files on disk stay. The app's home screen offers
it for the local server to anyone — the way back when nothing else works. See
[`CleanStart`](../reference/commands.md#cleanstart).

## Background

Keys used to be a key → command map in three layers (vanilla, debug, user), and
user bindings were "active after restart". Step 3 made one key system: a binding
names a UI key, command lines live only in UI keys, and changes work at once. The
model follows a common console convention: a virtual key with a value per gesture. The design notes are in
`docs/plans/done/2026-10-10-ui-keys.md`.
