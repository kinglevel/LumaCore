---
title: CLI Terminal
category: docs
---
# CLI Terminal

How the LumaCore terminal works as an operator surface: the screen, the prompt,
completion, keys, shortcut modes, watches and how a terminal attaches to a
server.

- The language: [lum.md](../syntax/lum.md)
- Every word's syntax and aliases: [command reference](../reference/commands.md),
  [object reference](../reference/objects.md)
- How commands work: [commands.md](commands.md)
- Tree-shell words (`ls`, `cd`, `cat`, …): [tree-shell.md](tree-shell.md)
- LumaSessions (stations working together):
  [lumasession.md](../../dev/networking/lumasession.md)

## Principle

The terminal is a client surface, not an engine. It renders, takes input and
sends it to the server. It owns neither the object tree nor the show.

It should feel like a console that happens to be text: responsive, aware of
context, and never hiding what the system is doing.

The same terminal runs in three places: `lumacore-cli` in a real terminal, the
Terminal window of the app and web UI, and the in-process `lumacore` REPL. The
server renders every shell (the remote ones as ANSI frames), so they behave the
same.

## Screen

The terminal is a full-screen TUI with three zones:

```text
┌──────────────────────────────────────────────┐
│ Path bar (top)                               │
│   path, claims, selection, status            │
├──────────────────────────────────────────────┤
│ Output area (middle)                         │
│   scrollable command output + watch pane     │
├──────────────────────────────────────────────┤
│ Input line + status bar (bottom)             │
│   prompt, text input, ghost hint, modes      │
└──────────────────────────────────────────────┘
```

- The path bar and input line are always visible.
- The output area scrolls (PageUp / PageDown); the others don't.
- The watch pane, when active, sticks to the top of the output area.

## Prompt

The prompt shows who you are, where you are, and your scope:

```text
[Admin@Luma01] Root>
[Admin@Luma01] Patch>
[LD@FOH] Fixture>
```

- the user is the logged-in (acting) user
- the host is the station name
- the scope comes from where you are in the tree

## Path bar

The top row shows:

- the user and the tree path
- claims: yours in orange, other users' dimmed, stacked like
  `[Programmer] [Patch] [Fixture 1]`
- the programmer with its selection count
- status: output lines running (green), active watches (magenta), `DEBUG` (red)

## Status bar

The status bar shows the shortcut mode and your modes:

| Label | Meaning |
|---|---|
| `KBD` (dim) | Keyboard mode: normal typing |
| `PRG` (green) | Programmer mode: single-key shortcuts |
| `NAV` (blue) | Navigator mode: tree browser |
| `HI` | your Highlight is on |
| `SOLO` | your Solo is on |
| `BLIND` | your Blind is on |
| `FRZ` | your Freeze is on |
| `BO` (red) | blackout: the grand master is off (`Off Master 2.1`) |
| `WALK:n/m` | stepping through the selection with Next / Previous |

Modes are per user (`Runtime.Users.{User}.Modes`); `BO` is the show's grand
master and the same for everyone.

## Input line

- single-line editing
- **ghost hint**: a dimmed suffix — the newest line from your history that
  starts with what you typed, else the first completion; Right at the end of
  the line takes it all, Alt+Right takes the next word
- **verb highlighting**: a known verb is green, an unknown one red
- questions open a window in the middle of the terminal (below)

## Suggestions and completion

The station suggests what fits at the cursor — the same list in the terminal,
above the command line bar of a screen and (in full screen) in the palette.
It comes from the registries, so only what is valid at the cursor is offered
(grammar first); then it is ranked by how well it matches what you typed (a
fuzzy match: `grp` finds `Group`), then by what you use most (your usage,
saved with the show), then by kind and number. History lines that start with
what you typed come first. A word that isn't built yet is listed dimmed
("isn't built yet (planned)"); one your rights level may not run is dimmed with the
reason (E-RIGHTS-001).

- **Tab** takes the selected suggestion (one candidate: taken at once); Tab on
  an empty line opens the **Quicksheet** — the tree here plus every command
  word by category
- **Up / Down** move in the list while it is open, else they search history
- **Enter** always runs the line as typed — never the highlighted suggestion
- **Esc** closes the list
- `Set UserProfile "Suggestions" Auto | TabOnly | Off` (per profile, default Auto): the list
  opens while you type, only on Tab, or never (then Tab completes in place and
  there is no ghost hint)

Candidates follow the position:

| Position | Candidates |
|---|---|
| first word | verbs and object keywords |
| after `cd`, `ls`, `cat`, … | tree paths (absolute from Root or relative) |
| after an object keyword | its IDs and names |
| after `At` | attribute names and value keywords |
| after `Attribute` | attributes of the selected fixtures |
| after `Help` | every word |
| after `Unwatch` | the running watches |

The popup shows "Suggestions N of M"; each row has a sign of its kind
(`↺` history, `▶` command, `⌘` object keyword, `◆` object, `#` value), the
selected row is in the accent colour, descriptions are dim.

Short forms are aliases from the registry (for example `f` = Fixture,
`u` = Update). The list lives only in the
[reference](../reference/commands.md) — every word shows its aliases there.

## Keys

| Key | Does |
|---|---|
| Enter | run the line |
| Tab | take the selected suggestion; on an empty line: Quicksheet |
| Up / Down | move in the suggestions while open, else history search (substring, any case) |
| Right | take the ghost hint (at end of line) |
| Alt+Right | take the next hint word, or go forward in `cd` history |
| Alt+Left | go back in `cd` history |
| PageUp / PageDown | scroll the output |
| Ctrl+Z / Ctrl+Y | undo / redo an input edit |
| Ctrl+C | clear the line, or cancel a question |
| Ctrl+D | leave the shell |
| Esc | cancel a question, close suggestions; it never clears the line |
| Shift+F10 | cycle the shortcut mode |
| Backspace on an empty line | depends on the shortcut mode (below) |

Key bindings come in three layers: debug bindings (debug mode only), your own
bindings, then the vanilla defaults. A binding names a UI key; the UI key holds
the command lines. See [Keys](../keys/keys.md) for `Bind` and gestures.

The terminal can't see key release on most systems. It fires `Press` (and
`DoublePress` by timing); the other gestures need the app or web UI.

## Shortcut modes

Three modes, cycled with `Shortcuts` or Shift+F10 (Keyboard → Programmer → Navigator,
shown as KBD → PRG → NAV), or set directly (`Shortcuts Programmer`; the short names
`Shortcuts PRG` work too).

**KBD (Keyboard)** — normal typing, full hints and completion.

**PRG (Programmer)** — Console-style stacking. Noun keys add a word with a trailing
space; action keys run at once on an empty line.

| Key | Type | Effect |
|---|---|---|
| F | add | `fixture ` |
| G | add | `group ` |
| T | add | `thru ` |
| A | add | `at ` (double tap: runs `at 100`) |
| S | add | `store ` |
| U | add | `update ` |
| O | add | `off ` |
| P | add | `preset ` |
| E | add | `edit ` |
| I | add | `if ` (no `If` word is built yet) |
| H | run | `highlight` |
| B | run | `blind` |
| L | run | `list` |
| Left | run | `previous` (empty line) |
| Right | run | `next` (empty line) |
| Backspace | run | `reset`, one stage per press (empty line) |

Backspace resets only on a fresh press: auto-repeat never steps through the
reset stages, and a press within ~400 ms of erasing a character is ignored. So
holding Backspace to erase a command stops at the empty line.

Example: F → 1 → A → 50 → Enter = `fixture 1 at 50`.

**NAV (Navigator)** — the tree on the left, exactly the rows `ls` prints (slot, "name", kind,
counts), and the inspector on the right following the cursor: the object's Slot, Kind, GUID and
Path, its facts (a fixture's type, mode and address, a datapool's pools), every property with its
value and kind (● = not its default) and its links (what uses it, what it points at; red
"missing" when that object is gone). NAV's keys work while the command line is empty; whatever it
changes is a command line it puts there or runs, and the gesture log above the command line says
what (`L` turns the log off; your profile keeps it, `Set UserProfile "NavLog"`).

| Key | Effect |
|---|---|
| Up / Down, PageUp / PageDown, Home / End | move |
| Right / Left | open; close, go to the parent row, or `cd ..` |
| Enter | tree: open, or to the inspector · property: its `Set` line on the command line · link: go there |
| Tab | tree ⇄ inspector |
| digits | jump to that slot |
| `/`, `n` / `N` | filter the pool by name or number (the inspector by property) · next / previous hit |
| Space | mark rows: they become one range on the command line (`Group 1 Thru 3 + 7`) |
| `y` | inspector: copy the focused row to your clipboard |
| `yg` `yp` `ya` `yn` `ys` | tree: copy the GUID, path, target (`Fixture 101`), name, the property's Set line |
| `r` | on a changed ● property: its `Reset` line |
| `v` | only the changed properties |
| `m` + letter, `'` + letter | bookmark the object (by GUID, kept on you in the show) · jump to it, wherever it moved |
| `p` | pin the inspector to the object |
| `c` | `cd` to the row |
| Delete | its `Delete` line on the command line |
| `?` | every key |
| Esc | close · drop the marks · leave NAV |
| Backspace (hold ~1 s) | up one level (`cd ..`); held, to the top (`cd /`) |

### Panel focus (app and web UI)

In the app and web UI the terminal is one panel among others. Clicking a panel
gives it **focus** (amber frame). Focus only decides where *navigation* keys go —
typing always goes to the command line. Routing happens on the server per
command line, so a terminal-only client behaves like the terminal panel.

With a panel (e.g. Groups) focused:

| Key | KBD | PRG | NAV |
|---|---|---|---|
| Letters / digits | command line | command line (shortcuts apply) | command line |
| Arrows, PageUp/Down, Home/End | move the panel cursor | move the panel cursor | move the panel cursor |
| Enter, line has text | run it | run it | run it |
| Enter, empty line | activate the focused item | activate the focused item | edit the focused item |
| Backspace, line has text | nothing | nothing | nothing |
| Backspace, empty line | erase inside the panel | `reset` one stage | up one level (hold: top) |
| Delete | delete the focused item (asks) | same | same |
| Esc | focus back to the terminal | same | same |

## History

- Up/Down search the history by substring (any case); the first Up keeps what you
  typed as the search text, Down goes back toward it.
- `History [filter] [-n N]` lists it.
- `cd` history is separate (Alt+Left / Alt+Right), capped at 50.

## Running a command

1. the line goes into history and the session log
2. a dimmed copy of `> line` is printed
3. the line is dispatched (one statement after another, split on `;`)
4. output is appended and the view scrolls to the bottom

A command can reply with output, a question, a logout, or an exit.

A **question** (decisions §30 "12b") is the station's dialog, the same one the
web page shows: in a terminal it opens as a window in the middle of the dimmed
shell, in your theme — title and countdown in its border, the text, a details
list, input fields (a password is masked), a countdown bar, the buttons.
←/→ or Tab choose, Enter presses, 1–9 press a button, Esc = Cancel; any key
stops a countdown. `lumacore-cli` shows the station's frames, so it gets the
same window. Answering closes it on your other screens too; the line it runs
is echoed like a typed one.

A one-shot command (`lumacore-cli --cmd`, the UDP command port, HTTP) can't
answer: it is refused and told which flag answers it (`/NoConfirmation`,
`/Save`, `/NoSave`, `/Keep`, `/Overwrite`, …).

The terminal is drawn in your UserProfile theme (truecolor, its background
painted), and in the normal theme while your device shows it.

## Watches

A watch re-runs a read-only command or object view on a timer (default 200 ms)
and keeps it at the top of the output. A line that changes anything (`At 50`,
`Go`, `Store …`) is refused — only read-only lines can be watched
[E-CMD-010]:

```text
Watch Clock
Watch Selection
Watch Fixture
Clock /Watch=500
List Watch
Unwatch
Unwatch Clock
```

- the pane sits above normal output, under a dim `─── watch ───` line
- while any watch runs, the output is cleared before each command

## Colors

| Color | Meaning |
|---|---|
| Green | success, known verb, active status |
| Red | error, unknown verb, debug mode, blackout |
| Cyan | DMX addresses, universe IDs, popup border |
| Yellow | warnings, selected fixtures, mode labels |
| Orange | your claims in the path bar |
| Magenta | watch indicator |
| Dim | old output, ghost hints, other users' claims |
| Bold | table headers, verb highlights, status labels |

Fixture selection: not selected = default, selected = bright yellow, in the walk
pool = dark yellow, the walked one = bright yellow bold.

## Errors and tables

- errors start with a red bold `Error:` and end with their code
  (`[E-AREA-NNN]`); `Info Error <code>` explains one
- GDTF import warnings use a yellow `!` with a count header
- tables: bold headers, cyan addresses, yellow warnings, green totals
- `Clear` (aliases in the reference) wipes the output buffer

## lumacore-cli

`lumacore-cli` attaches to a server over the shell port (the number is a constant of
`lumacore-wire`). It compiles only the wire crate, never the console engine.

```text
lumacore-cli                         log in as Admin on 127.0.0.1
lumacore-cli --target 10.0.0.5 --user LD --password pw
lumacore-cli -C "list fixture"       one command, reply, exit
lumacore-cli --health                health check on the command port
lumacore-cli --shutdown              ask the server to shut down
lumacore-cli --meownitor             the System Monitor (Meownitor)
```

The password can also come from `$LUMACORE_PASSWORD`. `--cmd-port` and
`--shell-port` pick other ports (a second server on one machine).

When it isn't attached, the CLI has a small outer prompt (`Unconnected>`):
`HELP`, `LOGIN [name]`, `CMDLINE [ip]` (single-line replies), `EXIT [code]`.
`Logout` in a shell returns there, never into a second engine.

### Meownitor

`lumacore-cli --meownitor` connects to a server's shell port and streams its
System Monitor live (the same log as `library/logs/sysmon-*.log`): startup,
network, engine, LumaSession and error events. The `MEOWNITOR [ip]` word at the
outer prompt only prints a banner for now.

Still planned: an interactive monitor at that prompt — see [the roadmap](../../meta/roadmap.md).

### Coexistence

- one server per workspace (or per `--station` with `--port-base`)
- many terminals may attach; each logs in as a user
- a terminal may reconnect without restarting the server
- the server stays authoritative when a terminal is outdated, resized or detached
- a client speaking another `CLIENT_PROTOCOL` is refused (E-WIRE-001)

## Background

- **Abbreviations.** This page once kept its own abbreviation table. It drifted
  (`bo` → the removed `Black` word, `u` listed as Universe while it is Update).
  Aliases now come only from the registry and show in the generated reference.
- **Parser assistance.** The plan was a per-user "parse summary" and "parse
  debug" shown after each command, set with a user setting. That setting never
  existed. What exists is `Set UserProfile "Verbosity" Summary|Verbose|Debug` and the per-command
  `/v` and `/d` flags. Still planned: sentence previews and prompt leaning
  (`Lum>` / `Tree>`) — see [lum.md](../syntax/lum.md) and [the roadmap](../../meta/roadmap.md).
- **Engine boundaries** are in [engines.md](../../dev/engines/engines.md).
