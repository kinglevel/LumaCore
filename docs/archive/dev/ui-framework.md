# LumaCore Web UI Framework — Design Decisions

Decisions captured from the panel system prototype. Each item records the question, options, current default, and rationale.

## The UI kit (step 12, decisions §30)

Every window, menu, dialog, pop-up and bar is built from the kit
(`crates/ui/lumacore-webui/web/kit/`, plain custom elements, ES modules, no build
step); a missing element is designed as a mockup first
([docs/dev/ui-kit/mockups/](ui-kit/mockups/README.md)). Built so far (12a1):

| File | What |
|---|---|
| `kit/tokens.css` | Theme tokens: 14 palette colours (Dark = the user's console concept, Daylight, Night) + derived roles + sizes (larger on coarse pointers). The only file with raw colours (tidy `frontend_colours_only_in_the_theme`). |
| `kit/theme.js` | Presets (`<html data-theme>`), tokens as concrete colours (`token()`), xterm's theme from the tokens. |
| `kit/pointer.js` | The pointer core: `hold()` ("show me more": a still press for the profile's hold time with a ring, plus right-click and the browser's long-press) and `drag()` (pointer capture, per pointer, ends on up / cancel / lost capture). |
| `kit/icon.js` | `<luma-icon name>` from the vendored Lucide sprite (+ own glyphs). No emoji in the chrome (tidy `frontend_icons_come_from_the_kit`). |
| `kit/button.js` | `<luma-button icon variant disabled>` (a real `<button>` inside; disabled stays visible). |
| `kit/menu.js` | The menu (`#ctx`): window menu and context menus, `--lc-row` rows. |
| `kit/showroom.js` | The Showroom window type: every kit element in every state. |
| `kit/kit.css` | Touch baseline (`touch-action`, focus-visible, 16 px fields on touch) and the elements' styles. |

The page waits for `window.LumaKit` (event `lumakit`) before it boots.

---

## D1 — Executor strip position

**Default: Fixed strip at bottom**

| Option | Description |
|--------|-------------|
| **Fixed strip at bottom** *(default)* | Always present. Cannot close or move. Safe during show. |
| Panel in split tree | Executors become a normal tile — can be placed anywhere, resized, or closed. |

**Rationale:** Established consoles keep executors permanently docked at bottom. A floating panel gives flexibility but risks accidental close mid-show.

---

## D2 — Close button visibility

**Default: Show on header hover** — for a mouse (`any-hover`). On touch screens there is no ×: the window menu (tap the tab's gold dot) closes (decisions §30).

| Option | Description |
|--------|-------------|
| **Show on header hover** *(default)* | The × only appears when hovering the panel header. Cleaner look. |
| Always visible | The × is always shown on every panel header. |

**Rationale:** Hover-only reduces clutter. Always-visible prevents confusion when a new user cannot find how to close a panel.

---

## D3 — Multi-tab panels

**Default: Multiple tabs per panel**

| Option | Description |
|--------|-------------|
| **Multiple tabs per panel** *(default)* | Each panel shows all its named tabs. The status dot follows the active tab. |
| Single tab (panel name only) | Only the active view is shown as the header. Cleaner, less chrome. |

**Rationale:** The amber status dot lives inside the active tab and moves on click. This decision controls whether sibling tabs are visible at all.

---

## D4 — How to open new panels

**Default: the window menu** — tap the tab's gold dot, hold the tab or the header (ring), or right-click; the same menu for mouse and touch (decisions §30). Originally: right-click → split.

| Option | Description |
|--------|-------------|
| **Right-click → split** *(default)* | Context menu on any panel header: Split Right / Split Down / Change content / Close. |
| Visible + buttons | Add ⊞/⊟ icons in every panel header. More discoverable, more chrome. |
| Drag panel type in | A sidebar of panel types — drag one into a region to split it there. |

**Rationale:** Right-click is fast but invisible to new users. A dedicated + button is discoverable but adds chrome to every header. Drag-in is the most flexible but requires a panel-type sidebar.

**Decided (window spawning):**

- **Split Right / Split Down** leave the clicked panel as it is and add an **empty panel**: one placeholder window of type `empty` with an *Add window* button. Splitting is a UI-only layout edit.
- **Tabs are windows**, not kinds: a panel can hold any number of windows of any type, Terminal included.
- **Add Window picker**: category tabs from the window-type registry plus *All* with a filter over name, category and keywords; typing jumps to *All*. Context menu: *Add window…*, *Change window type…*, *Rename…*, settings, close.
- Picking runs a **command**, so it echoes in the terminal:

  | Action | Command |
  |---|---|
  | Fill an empty panel | `Store Window <no> "Type"` |
  | Add window (tab beside `<no>`) | `Store Window "Type" /Pane <no>` |
  | Change window type | `Store Window <no> "Type" /Overwrite` (same number and tab, settings reset) |
  | Rename tab | `Label Window <no> "name"` |

  Typed `Store Window "Type"` lands beside the focused window (the browser reports it with `{type:"focus", window}`). Also `Delete Window [screen.]n | [screen.]*`, `Set Window n <key> <value>` (also `+ 1`, `- 1`, `* 2`, `Default`, `Toggle`, `Now`), `Reset Window n`, `Window n` (focus), bare `Window` (list). Closing tabs and panels stays a silent layout edit.
- **Window bar and `Self`** (decisions §30 "12c"): every window bar item, window-menu entry and settings-window button is a command line with `Self`, sent with the window as its owner (`{type:"command", cmd, window}`); the station resolves `Self` to `Window <screen>.<n>` before the line runs or is forwarded, so the journal and System Monitor name the window. Live titles, lit items, chips and `Self` key LEDs come from the `windows` feed on `/ws/feeds`. Window presets (`Store WindowPreset`, `WindowPreset n`) live on the profile; `Export|Import View|ScreenConfig|WindowPreset` write JSON to `library/ui/<kind>/`.

---

## D5 — Layout persistence

**Decided: server-owned, per screen (split tree instead of grid)**

- **Live windows** live in `Root.Runtime.Users.<user>.Screens.<n>` (`ScreenStore` in `lumacore-runtime`, shared with UI engines). A screen is one screen (desktop, tablet, phone); a browser joins one (`?vs=N`, else the last used, else a new one).
- **Windows** are numbered; each has a window type, typed settings (schema in `Root.Resources.WindowTypes`), appearance and scroll position. `set` on the window path edits it live.
- **UI engines** (`Root.UIEngines`) render screens; they decide *how*, not *what*.
- **Stored views** live on the user (`UserProfile.views`), with **view buttons** per user (`UserProfile.view_buttons`, shared by all of that user's screens). Store copies a screen into a view; recall copies it back — no live link.

---

## D6 — Terminals: stacked rendering

**Decided: one shell session per user, composited once per terminal viewport.**

| Layer | State | Lives in |
|---|---|---|
| User session (shared by all the user's clients) | command line, command history, output, suggestions, NAV popup, watch pane, a terminal's place in an open dialog | `TuiApp` in `RemoteCommandLine` |
| Client = one screen / connection | screen, focus, focused window, viewports | `ClientView` on `ClientHandle` |
| Viewport = one terminal window | size, scroll (anchored while scrolled up), overlays, layers | `ViewState` |

- `TuiApp::compose(frame, state, &mut ViewState)` stacks the layers (watch, output, status, path, input) a view shows and draws overlays (suggestion menu, ghost hint, NAV popup) only when `overlays` is set. `draw()` composes the session's own view (local REPL, CLI clients, tests).
- The browser sends `{type:"viewports", viewports:[{view, cols, rows}]}` with every visible Terminal window, the primary (focused, else last focused) first. The server renders one frame per viewport; binary WS frames carry a 4-byte LE viewport id. The primary viewport gets the overlays and the scroll keys; running a command snaps only it to the bottom, other views keep their position.
- Clients without viewports (CLI) get one frame at their own size, so clients no longer shrink each other.
- Since every client of a user shares the session, the command line and history show on every screen and device of that user.
- **Theme** (decisions §30 "12b"): the TUI is drawn in the user's UserProfile theme. Drawing code names colours by role (`tui_look::role`, the 16 terminal slots); after composing, `tui_look::paint` turns every cell into the theme's truecolor (the same slots as xterm's `terminalTheme()` on the page) with the theme's ground painted; `core::look` resolves presets and adjusters like `kit/theme.js` (tests check both against the kit files). A device in the normal theme (Meow+F9, `ThemeOverride`) gets the factory preset. Tidy: no colour in `terminal/` outside `tui_look.rs` (and the ANSI encoder).
- Layers make command-line-only / history-only windows a `ViewState` choice, not new rendering code.

---

## D7 — Command line bar

**Decided: a command line bar on every screen.**

```
[◀][PURR][ [Admin@host] Root> █                 ][ status icons… ][ONLINE][KBD][▶]
 50   66   stretch — 1-row viewport, input layer   contextual           50    50
```

- Full width, 50 px, below both sidebars, on every screen.
- **Command line**: a one-row xterm viewport (`BAR_VIEW`) with `layers: "input"` — the same prompt, syntax colours and ghost hint as the terminals, from the shared session, so it is in sync on every screen and device. Clicking it focuses the command line (white frame); Esc lands here when no Terminal window is open.
- **PURR** (the logo button; renamed from MEOW because Meow is the modifier key, decisions §17) toggles the Command Line History popup: a viewport (`HISTORY_VIEW`) with `layers: "output"`.
- **Status icons** are contextual (inactive ones are hidden): HI, SOLO, BO, WALK, selected fixtures, parked, sACN, WATCH, DEBUG, claims. Colours follow the status theme group. The server sends `{type:"status", status}` per client when it changes.
- **Session indicator** and the **shortcut mode** toggle (click runs `Shortcuts`).
- **◀ / ▶** hide the control bar and the view bar (F12 hides both, Meow+F10 the view bar; decisions §30 "12a3"), remembered per screen in the browser.
- Viewports carry `layers: full | input | output`; full terminals are listed first so the primary terminal keeps the suggestion menu; popups are never drawn in views under 6 rows.

---

## D8 — Shutdown menu, Menu, Backup

**Decided: overlays whose buttons run commands.**

- **Shutdown Menu** (⏻, Meow+F11): Shutdown, Lock (`On DeskLock`), Restart, Logout, Disconnect. Software-only layout — no Reboot.
- **Menu** (⚙): a 4×3 grid — Patch, Live Patch, Connector Configuration, Desk Lights & Color Theme / Network, In & Out, DMX Protocols, Settings / **Backup**, **Quick Save**, Show Creator, Preferences and Timings. Unbuilt tiles are greyed out. Title `Menu [<show>]` + software version.
- **Backup**: current show (unsaved badge), editable description, Load / New Show / Save As / Save / Delete, AutoSave Interval, Startup Show; a show browser with search for Load / Save As / New Show / Delete; autosave countdown in the title.
- Commands (`terminal/commands/backup.rs`):

  | Command | |
  |---|---|
  | `SaveShow ["name"]` | quick save / save as (untitled → `NewShow_<date>UTC`) |
  | `LoadShow "name"`, `NewShow ["name"]`, `DeleteShow "name"` | ask first |
  | `Shutdown`, `Restart` | ask first, 10 s countdown in the UI unless `/NoAutoClose` |
  | `Show /Description "…"`, `Show /Autosave <min>`, `Show /Startup Last\|None\|"name"` | settings |

  Options: `/NoConfirmation` (`/NC`), `/Save`, `/NoSave`, `/NoAutoClose`. Without `/NoConfirmation` the command asks (see **Dialogs** below): OK / Cancel, or Save / Don't save / Cancel when the show has unsaved changes; the answer re-runs it with `/NoConfirmation`. One-shot paths (UDP, HTTP, `--cmd`) are refused and told the flag.
- **Dialogs** (decisions §30 "12b"): every question the station asks — a line's confirm, OopsConfirmation, Unclaim Patch, `Debug`, `Shutoff`, Store over a used group, `Import UserProfile` with the same GUID, `Login` without a password, `Label` / `Move` / `Copy` without a name or target, a claim ask, the merge decision — is one dialog the station holds (`core::dialogs`: a command returns `DispatchResult::Ask`, or a text handler leaves it in `AppState::asking`; the shell opens it with `dialogs::open`). Each client's `Status` carries the dialogs meant for it (`dialogs`: the line's device or CLI client; claim asks: the owner's clients; the merge decision: every client) and `dialogs_closed` (answered here or elsewhere, for a notice). A client answers with `DialogAnswer {id, button, inputs, toggles, selectors}`; the station closes the dialog everywhere first, then fills the button's command-line template and runs it as the answering user. Countdowns are the station's (`dialog_tick`, housekeeping); a touch sends `DialogPause`. No countdown = it waits; a dialog whose client is gone 30 s counts as Cancel. The web draws them with the kit's dialog (`showDialog`), a terminal (CLI, the station's console) as a centered window in the user's theme (`terminal/repl/tui_dialog.rs`). Nothing opens over DeskLock: the surface counts them as "n waiting".
- **Unsaved changes**: any pushed undo frame marks the show dirty; save / load / new clear it.
- **Autosave**: every N minutes (show setting, default 15) a changed, named show is saved by the server loop.
- **Startup show** (station `library/station_settings.json`): last show (default), none, or a named show — loaded at startup, so `Restart` and a rebuild come back to the same show.
- **Restart** re-executes the program (`server::restart_process`), so a rebuilt binary is picked up; Shutdown ends the whole program, desktop app included.
- `GET /api/shows` lists the library and the current show; the status snapshot carries `show {name, dirty, description, autosave_minutes, autosave_in, startup}`.

---

## Related architecture

- Split tree: `Split { dir, ratio, children }` / `Leaf { windows: [no…], active }` (`LayoutNode`), windows in `Screen.windows`
- REST API:
  - `GET/POST /api/screens`, `GET /api/screens/{no}`, `PUT /api/screens/{no}/layout`
  - `GET /api/window-types` (name, label, category, icon, description, keywords, hidden, settings; since 12c the window bar: `actions`, `bar_default`, `bar_min`, the live `title` template and `chips` — how a window is built ships in the registry, the floor plan is show data)
  - `GET /api/views` → `{views, buttons}`; `POST /api/views/{no}/recall|label`; `DELETE /api/views/{no}`; `PUT /api/view-buttons/{slot}`
  - view writes run console commands (`Store View`, `View`, `Label View`, `Delete View`, `Assign View … At ViewButton …`, `ViewButton n`, `Store/Label/Delete ViewButton n`) through `ConsoleBridge`
- Stored views are multi-screen (content per screen): a single-screen view recalls onto the current screen, a multi-screen view onto the screens it was stored from. View-session numbers are reused on rejoin so they survive restarts.
- View button taps are **object taps on the server command line** (WS `{type:"tap"}`): on an empty line they call the button (`ViewButton n` → recall); with `Store` pending (or tap-and-hold) a UI client gets the Store View dialog (a station ask, `views::store_ask`: name + one toggle per screen), whose Store runs `Store ViewButton n "name" /Overwrite /Screen "a,b"`; terminal-only sessions store directly. The browser reports its screen with `{type:"live_screen"}` so commands without `/Screen` target it.
- SSE (`/api/events`): `screen_changed {no, revision}`, `views_changed`
- Left sidebar: 50 px, 50×50 icon buttons
- Right ViewBar: 100 px, fixed 20 slots paged 8 at a time, ▲/▼ nav at corners
