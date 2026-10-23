# Handoff — Panel splits, focus, UIEngines, key routing (2026-10-09)

Status: sections 1 committed (402288a). **Key routing (section 5) implemented, uncommitted, builds, not yet run in the app.**
Next: try it in the app, then section 6 (session split).
All work below is **uncommitted** on `main`. `cargo build` and `cargo build -p lumacore --features perfetto,http` pass.
None of it has been run in the app beyond the user confirming the split menu works.

---

## 1. Done (uncommitted)

### Frontend — `apps/lumacore-app/frontend/index.html`
- **Split tree + tab context menu**, ported from the claude.ai artifact "LumaCore Panel System"
  (https://claude.ai/artifact/3Tt2NC6ypz9u2jjz5yK6W9). Styles: `.snode/.schild/.dhandle`, `#ctx/.ci`.
  - Right-clicking a tab or the header opens: Split Right, Split Down, Add tab…, Close \<tab\>, Close panel.
  - Split **moves** the tab if the panel has other tabs, otherwise **duplicates** it. Terminal is a singleton, so it can't be duplicated.
  - Drag handles resize panels; the ratio is saved on mouseup. A hover × closes a panel. Closing collapses the split into its sibling.
- **Layout is server-owned**: `GET/PUT /api/panels/layout`. The SSE `layout_changed` event applies layouts from other clients and view recall (our own echo is skipped by JSON compare).
  - `normalizeNode()` sanitises the server tree: drops unknown kinds, duplicate singletons and empty leaves.
- **Terminal host**: xterm is opened once into a persistent `termHost` element and re-parented between panels.
- **Groups pool is unbounded**: slots are added in chunks of 40 as you scroll (`GROUP_CHUNK`, `fill()`). `makeGroupBtn()`. Selection is synced across multiple Groups panels via `data-no`.
- **Panel focus** (client-local, stored as a child-index path `focusPath`):
  - `pointerdown` on a panel focuses it. Indicator: amber 1px frame (`.panel.focused::after`) plus a tinted header.
  - Terminal focused → `term.focus()`, otherwise `term.blur()` (`syncKeyboardFocus`).
  - Split focuses the new panel; closing the focused panel focuses its sibling.

### Server
- `crates/lumacore-http/src/layout.rs`
  - The default layout is now Leaf `tabs: ["terminal","groups"]`. Tabs are kind ids, not labels.
  - Documented `SplitDir`: H = side by side, V = stacked.
- **`Root.UIEngines` publishes the HTTP engine:**
  - `lumacore-tree/src/tree.rs`: `UIEngineState` gained `name, kind, status, address, port, web_root` and derives `Serialize`. `UIEnginesRoot.engines` is now `BTreeMap<u32, _>` with `publish()`.
  - `lumacore/src/engines/mod.rs`: `http_ui_engine_state(addr, web_root)`.
  - `server.rs` and `bin/lumacore-server.rs` publish it right before `HttpEngine::start`. Status is hardcoded "Running", so a later bind failure is not reflected.
  - `terminal/vfs.rs`: `dynamic_children_at("root.uiengines")` and `resolve_object_by_id("root.uiengines")`, so `cd Root.UIEngines`, `list` and `UIEngines.1` work.

---

## 2. User decisions (authoritative)

**Typing and editing keys**
- Printable keys always go to the **command line**, even when a panel has focus. The panel **keeps focus**. "Type commands anywhere."
- Panel focused with **unexecuted text** on the command line:
  - arrows navigate the panel;
  - **Enter executes** the command;
  - Backspace does nothing (for now).
- **Backspace = `reset`.** `reset` is the existing 3-stage console Clear: selection → deactivate values → empty programmer. `clear` stays the terminal-buffer clear.
- **Delete is UI navigation**, e.g. delete the focused group in the Groups window (with confirmation). It no longer runs reset.

**Per mode**
- **KBD mode:** Backspace is a normal backspace that erases text in the focused tab/window. It never resets.
- **NAV mode, empty line:** Backspace goes **up one level**. **Holding Backspace goes to root level.**
- PRG hold-Backspace was not specified, so plan: no hold gesture in PRG (fresh press = one stage, auto-repeat ignored).
- **Enter on an empty line depends on the shortcut mode.** Proposed: KBD/PRG → activate the focused item (select group); NAV → open the edit/properties popup.
- **Esc (the usual console chain):** close popup → otherwise move focus to the terminal → otherwise clear the command line.

**Where state lives**
- Routing lives on the **server**, per session, so terminal-only CLI sessions and the elevated web UI behave the same.
- **Keybindings are per user**, not global.
- **Sessions model (target):**
  - Each user has a session.
  - A user can have **multiple view-sessions** (phone, tablet, other screens, different resolutions).
  - A screen can join an existing view-session (same live layout) or start a new one. View buttons are the same in both cases.
  - **Programmer and command line are shared per user.**
  - **Shortcut mode (KBD/PRG/NAV) and focused panel are per view-session.**
  - View-session state should live under **`Root.Runtime`**. Stored views belong **on the user** (`UserProfile`), not `ShowData`.
- **Build order:** key routing **first** on the current one-session-per-user model; split sessions afterwards.

---

## 3. Research findings

### LumaCore today
- **Modes**
  - `ShortcutMode {Keyboard, Programmer, Navigator}` is in `terminal/app_state.rs:17-45`; default PRG.
  - The mode is **global** (`AppState.shortcut_mode`, not swapped per session).
  - Shift-F10 / the `shortcuts` command switches it.
- **Key dispatch:** `terminal/repl/tui_input.rs:36-417`.
  - Order: NAV popup → keybinding map → PRG letter shortcuts → NAV (only on an **empty line**) → shared keys.
  - PRG Delete runs `reset` even when the line has text (`:362-369`).
  - PRG ←/→ on an empty line run prev/next.
  - NAV: ↑/↓ cursor, → expand, ← collapse/parent/`cd ..`, Enter opens a property popup, Delete pre-fills `delete <name>`.
  - Completion menu and history: Up/Down. Tab completes, or runs `quicksheet` on an empty line.
- **Protocol**
  - TCP :47654 carries newline JSON: `ShellClientPacket {Hello, MonitorHello, Key{key:{code,modifiers}}, Resize, Disconnect}` and `ShellServerPacket {Frame{ansi}, Error, Closed}` (`remote/wire.rs:97-120`).
  - The CLI (`bin/lumacore-cli.rs`) and the web client use the **same server-side `TuiApp` per session** (`terminal/repl/shell_session.rs`).
  - Web: xterm `onData` raw bytes → VT parser in `lumacore-http/src/websocket/shell.rs` → key packets.
  - Known parser gaps: Alt+key, Shift+Tab, unhandled CSI; 0x08 → Ctrl+H = `help`.
  - The WebSocket bridge **hardcodes user "Admin"** (`ws shell.rs:56`).
- **Users and data**
  - `UserProfile` (`lumacore-tree/src/programmer.rs:448`) already has `programmer`, `keybindings` and a `views: Vec<UserView>` stub.
  - Keybindings are hardcoded to user 1: `TuiApp::new` `.get(&1)` (`repl/mod.rs:166-180`) and `commands/settings.rs:36`. They are cached on `TuiApp` and never rebuilt.
  - `SessionRegistry` (`session_manager.rs:86`) is 1:1 user↔session. A second login mirrors the same session.
  - `TuiApp.textarea` holds the command line, per session.
  - The views are `ShowData.view_pool`/`view_slots`. The HTTP engine works on a **clone** of `ShowData` taken at startup, so HTTP-saved views never reach the real show.
  - `RuntimeState` is in `lumacore-session/src/lib.rs:18`; `sessions: HashMap<String, UserSession>` is keyed by username.
- **Docs drift**
  - `docs/user/terminal/cli-terminal.md` describes old NAV behaviour (C/M/L/I keys that don't exist).
  - `docs/user/object-tree/object-tree.md` says live session state lives in `Root.UIEngines`; the user wants Runtime, so update it.

---

## 4. Proposed key table (agreed except where marked)

The command line has text: Enter executes, whatever has focus. Printable keys → command line.

| Key | KBD | PRG | NAV |
|---|---|---|---|
| Arrows, panel focused | move panel focus cursor | move panel focus cursor | move panel focus cursor |
| Arrows, terminal focused | suggestions/history | ←/→ prev/next on empty line, ↑/↓ history/menu | tree navigator (existing) |
| Enter, empty line, panel focused | activate focused item | activate focused item | edit/properties popup |
| Backspace, terminal focused, text | erase char | erase char | erase char |
| Backspace, empty line | erase in focused window (panel decides) | `reset` one stage; fresh press only | up one level; **hold → root** |
| Backspace, panel focused + text on line | nothing | nothing | nothing |
| Delete, panel focused | delete focused object (confirm) | same | same |
| Esc | close popup → focus terminal → clear line | same | same |

**Key repeat.** The browser has `KeyboardEvent.repeat`. Plain terminals (CLI) don't report repeat, so fall back to timing:
- treat Backspace less than ~150 ms after the previous one as a repeat;
- PRG reset is ignored if a character was erased in the last ~400 ms;
- NAV "hold": repeats continuing ≥ ~1 s since the first press → `cd` root, once.

---

## 5. Implementation plan — key routing (NEXT)

1. **Wire protocol** (`remote/wire.rs`, mirror in `lumacore-http/src/websocket/shell.rs`):
   - `ShellClientPacket::Key` gains `repeat: Option<bool>` (`#[serde(default)]`).
   - New `ShellClientPacket::Focus { target: "terminal" | "panel", kind: Option<String> }`.
   - New `ShellServerPacket::Ui { event }`, e.g. `{action:"move",dir}`, `activate`, `edit`, `delete`, `erase`, `back`, `root`, `focus_terminal`.
2. **Web bridge** (`ws shell.rs`):
   - Accept structured `{type:"key", key, code, ctrl, alt, shift, meta, repeat}` JSON from the browser and convert it to a Key packet. This fixes the Alt/Shift+Tab gaps.
   - Keep raw-bytes handling only for paste.
   - Forward `{type:"focus"}`.
   - Server Ui packets → WS. Use a distinct framing: e.g. ANSI as **Binary** and control JSON as **Text**, or a JSON envelope for both.
3. **Frontend:**
   - Use a document-level `keydown` handler for all keys, both when the terminal is focused (`term.attachCustomKeyEventHandler` → send + return false) and when a panel is focused. Skip Cmd/Meta combos and real `<input>` targets. Call `preventDefault` on forwarded keys.
   - Send `{type:"focus"}` on every focus change.
   - Handle incoming Ui events in the focused panel. Groups needs a keyboard focus cursor, activate = toggle selection, delete = `DELETE /api/groups/{no}` with confirm.
   - `focus_terminal` → move focus to the terminal panel.
4. **Server routing** (`terminal/repl/tui_input.rs`, `shell_session.rs`):
   - Add `focus: UiFocus { Terminal | Panel{kind} }` on `TuiApp` (per session for now).
   - Add a `route_key` front stage implementing section 4. Panel-targeted keys emit Ui events instead of hitting the textarea.
   - Change Backspace semantics per mode, plus repeat/hold tracking on `TuiApp`.
   - Implement the Esc chain.
   - PRG Delete no longer resets.
   - `RemoteShellSession` returns frames plus Ui events to the session manager → TCP.
5. CLI (`bin/lumacore-cli.rs`) is unchanged apart from sending `repeat: None`. Focus is always Terminal.
6. Update `docs/user/terminal/cli-terminal.md` (modes and key table).

## 6. Later (after key routing)
- **Session split:**
  - user session (command line, programmer, per-user keybindings, stored views on `UserProfile`) plus N view-sessions under `Root.Runtime` (layout, focus, shortcut mode);
  - join an existing view-session or create a new one;
  - HTTP requests carry user and view-session identity;
  - drop the "Admin" hardcode;
  - stop the HTTP engine using a cloned `ShowData`.
- Make keybindings per user: replace the `get(&1)` hardcodes and rebuild `TuiApp.keybindings` on attach and after `bind`.
- Move `ShowData.view_pool/view_slots` into `UserProfile` (replacing the `views` stub). Update `object-tree.md` and `ui-framework.md`.
- Prototype leftovers not yet ported: executor strip, a visible split button, drag-a-panel-type-in.

---

## 7. Windows, window settings, views (added later 2026-10-09)

### Decisions
- **Placement:** keep the **split tree** (flexible multi-platform), not a fixed grid.
- **Recall:** **copies**; edits don't touch the stored view until stored again.
- **Live windows** are canonical in **Runtime view-sessions**: `Root.Runtime.Sessions.<user>.ViewSessions.<n>.Windows.<no>`. **`set` is done on the Runtime path.**
- **`Root.UIEngines.<n>`** = *how* it renders (engine render settings plus which view-sessions it serves), not *what*.
  - Future engines may render differently, so window types and settings stay renderer-agnostic.
- **Stored views** live on the user: `Root.ShowData.UserProfiles.<user>.Views`. **View buttons** are per user, shared by all of that user's view-sessions.

### Phases
1. **DONE (uncommitted):** window type registry.
   - `lumacore-tree/src/window_types.rs`: `WindowType`, `SettingDef` (bool / int range / enum), `coerce()`, `resolve_settings()`, with unit tests (not run).
   - Published at `Root.Resources.WindowTypes`.
   - `GET /api/window-types`.
   - The frontend builds its settings popover from that schema; settings are stored per panel in `leaf.settings.<kind>` (new `settings` map on `PanelNode::Leaf`).
   - Terminal settings: font_size, cursor_blink. Groups settings: slots, columns, rows, size, show_empty.
   - Also fixed: the Groups pool rendered 2000+ slots because it wasn't really scrolling (WebKit didn't treat its `height:100%` as a fixed size); it's now `position:absolute; inset:0`.
2. **DONE (uncommitted, builds, not run): view-sessions + windows in Runtime.**
   - `lumacore-session/src/view_sessions.rs`: `ViewSessionStore` (shared `Arc<RwLock>` in `RuntimeState.view_sessions`), with `LayoutNode` (Split / Leaf{windows, active}), `Window{no, kind, settings, appearance, scroll}`, `join_or_create`, `replace_layout`, `update_window` and change listeners.
   - HTTP subscribes and emits the SSE event `view_session_changed {no, revision}`.
   - Endpoints: `GET/POST /api/view-sessions`, `GET /api/view-sessions/{no}`, `PUT /api/view-sessions/{no}/layout` (validates kinds and singletons, coerces settings).
   - Tree: `terminal/vfs_runtime.rs` publishes `Root.Runtime.Sessions.<userNo>.ViewSessions.<n>.Windows.<no>`. A new "Folder" DynChild kind keeps named levels in `cd` paths.
   - `set <key> <value>` inside a window, or `set <path> <key> <value>`, edits it live.
   - `UIEngines.1` lists `view_sessions`.
   - Frontend joins a view-session (`?vs=N`, else the last one used, from localStorage), saves via PUT and applies SSE changes by revision. Window numbers follow their tabs (`leaf.wins`); moving a tab keeps its number, duplicating makes a new one.
   - **Disconnected until phase 3:** the old `/api/panels/layout` + `HttpState.panel_layout` and view recall (`layout_changed`) are no longer used by the frontend.
   Original plan:
   - A shared `Arc<…>` store in `RuntimeState` (`lumacore-session`); `lumacore-http` can reach it via `lumacore-tree` → `lumacore-session`, so there is no crate cycle.
   - The store notifies listeners on change, so HTTP can broadcast SSE.
   - Each window gets a stable `no` in the layout leaf, plus settings, appearance and scroll position.
   - `set` on `Root.Runtime.Sessions.<u>.ViewSessions.<n>.Windows.<no>` validates through `SettingDef::coerce` and updates the UI live.
   - `UIEngines.1` gets render settings and a list of the view-sessions it serves.
   - This replaces the single global `HttpState.panel_layout`.
3. **DONE (uncommitted, builds, not run): views on the user.**
   - `UserProfile.views: BTreeMap<u32, StoredView{no, name, root, windows}>` (old `views: []` still loads) and `UserProfile.view_buttons: slot → view`.
   - `ShowData.view_pool/view_slots`, `/api/panels/layout`, `HttpState.panel_layout` and `PanelLayout` were removed.
   - Commands (`terminal/commands/views.rs`): `Store View n "name" [/Screen s]`, `View [n] [/Screen s]`, `Label View n "name"`, `Delete View n`, `Assign View n At ViewButton slot` (0 clears).
   - HTTP: the `ConsoleBridge` trait (`lumacore-http`), implemented by `remote::ServerConsoleBridge`, runs those commands as the user and reads `{views, buttons}`. Endpoints: `/api/views…` and `/api/view-buttons/{slot}`. The store's `views_changed` hub drives the SSE `views_changed` event.
   - Frontend view bar: click recalls onto this screen; right-click stores the screen there, updates, renames, clears the button or deletes the view.
   - Known: `lumacore-http` unit tests construct an outdated 3-field `HttpState` (already broken before this work).
   - **Added after phase 3 (Store + tap, Store View Options):**
     - Views are multi-screen: `StoredView.screens: view-session → ScreenContent{root, windows}`.
     - `/Screen "1,2"` lists; `/Overwrite` is accepted (storing always overwrites).
     - ViewButton commands: `ViewButton n`, `Store/Label/Delete ViewButton n`.
     - Object taps: WS `tap` → `ShellClientPacket::Tap` → `TuiApp::handle_tap`. A tap completes a pending command, or runs the default action.
     - Store + tap or tap-and-hold → `UiEvent::StoreView` → browser dialog (label, screen toggles, All/None/OK) → `POST /api/views/store`.
     - `ShellClientPacket::ViewSession` sets `AppState.current_view_session` per shell.
     - `join_or_create` reuses a free requested number.
     - Not done: the screenshot toggle (a stored screenshot image for single-screen views); typed `Store View n` without `/Screen` stores directly (no dialog).
   Original plan:
   3. **Views on the user** (`UserProfile.views` replaces the `ShowData.view_pool/view_slots` stub), store and recall by copy, per-user view buttons. Also stop the HTTP engine working on a cloned `ShowData`.
