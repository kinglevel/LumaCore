# LumaCore UI — inventory and critique of the current framework (2026-10-12)

Scope: `apps/lumacore-app/frontend/index.html` (4600 lines: CSS 12–743, body markup 745–1004,
script 1005–4598), `apps/lumacore-app/home/home.html` (520 lines, the app's home window),
the server-side UI descriptions in `crates/`, the screenshots in `target/screenshots/`, and the
UI docs/decisions. All paths relative to the repository root. Line numbers
are `index.html` unless another file is named.

Short version: one hand-written HTML file, no components, no build step, vendored xterm 5.3.0
+ fit 0.8.0 + modern-screenshot 4.7.0 (`frontend/vendor/`). The *data model* is good and
server-driven (window types with typed settings schemas, `PoolDef` cells, `UiEvent`s, every
action a command line). The *view layer* is not: every overlay/dialog is hand-built markup
with its own CSS family, buttons come in ~8 visual variants, there is no form/field
renderer shared between the three "settings" pop-ups, and the input model is mouse-first
(split drag is `mousedown`-only, ×-close is hover-only, every secondary action is behind
right-click, hit targets 18–24 px, text 8–11 px).

---

## 1. Inventory

### 1.1 Shell of the page

| Part | Markup | Code | Data | Notes |
|---|---|---|---|---|
| Top bar (`#topbar`: dot, "LumaCore", user · screen) | 747–751 | dot in `connect()` 1111; text in `applySession` 1611 | `/ws/shell` state | 28 px, 11 px text |
| Left sidebar / control bar (`#sidebar-left`) | 754–766 | `data-key` gesture handler 1390–1436; `renderKeyLeds` 2264; `toggleSidebar` 2240 | `GET /api/ui-keys`, status | 50×50 `.sl-btn` (CSS 47); ⏻, BLND/HI/FRZ/SOLO (9 px text, CSS `.sl-mode`), filler, 🐾 Meow latch, ⚙ Menu, AT (inline style, no handler beyond title) |
| Panel area (split tree) | `#panel-area` 768 | `renderLayout` 1850, `buildNode` 1858, `buildLeaf` 1877, `attachDrag` 1928, `doSplit` 1964, `closeTab` 1973, `closePanel` 1984 | `POST /api/screens`, `GET/PUT /api/screens/{n}[/layout]`, SSE `screen_changed` (`loadLayout` 1631, `saveLayout` 1654, `onScreenChanged` 1669) | Full DOM rebuild on every change (`area.innerHTML=''`); persistent hosts for terminals / sysmon / dmx / realtime views are re-parented |
| Panel header + tabs | built in `buildLeaf` 1884–1916 | `activateTab` 1956; ctx menu `showCtx` 2005 | — | `.panel-hdr` 30 px, `.ptab` 10.5 px text; close × `.panel-close` 18×18, opacity 0 until header hover (CSS 83–90) |
| Context menu (`#ctx`) | 970 | `renderCtx(e, items)` 2028 — the one generic item renderer (`{hdr}`, `{ico,lbl,fn,off,danger,keep}`, `null`=separator) | — | Opened only from `contextmenu` events; items 6 px/10 px padding (~26 px rows) |
| Add Window picker (`#picker-overlay`) | 975–983 | `openPicker` 2082, `renderPicker` 2097, `renderPickerItems` 2119, `pickType` 2146, keyboard 2165 | `GET /api/window-types` (`loadWindowTypes` 2356) | Data-driven from registry (category tabs, filter over keywords). Runs `Store Window …` commands |
| Command line bar (`#cmdbar`) | 773–781 | `initCmdBar` 2208; `renderStatus` 2272; `updateNet` 2342; `uiShortcut` 2249 | `/ws/shell` (xterm viewport `BAR_VIEW`, `layers:"input"`), `{type:"status"}`, `/ws/feedback` statuses | ◀ / PURR / one-row xterm / status icons (`.cb-icon`, 9 px text, CSS 288–294) / NET / KBD / ▶ |
| PURR history pop (`#history-pop`) | 784–787 | `toggleHistory` 2217 | xterm viewport `HISTORY_VIEW`, `layers:"output"` | Fixed popup, 42vh |
| Right ViewBar (`#sidebar-right`) | 770–774 | `renderViewBar` 4226, `attachTap` 4249 (pointer, 600 ms hold), `showViewButtonCtx` 4269, `viewPageUp/Down` 4286 | `GET /api/views`, SSE `views_changed`, WS `{type:"tap", object:"viewbutton"}` | 100 px wide, 50 px slots, number 8 px / name 10 px |
| Feedback surfaces | `#fb-progress`, `#fb-flash`, `#fb-cards`, `#fb-popup` 805–815 | `fbHandle` 1244, `fbProgress` 1266, `fbFlash` 1293, `fbCard` 1301, `fbPopup`/`fbShowPopup` 1319 | `GET /api/feedback`, `/ws/feedback` | One system (handoff feedback-system). Cards top-right 300 px — overlap the view bar and window content (screenshots 01, 03) |
| Connection lost (`#conn-overlay`) | 796–802 | `showConnLost` 1169 | WS close | `.lc-overlay` family |
| Login (`#login-overlay`) | 788–795 | `boot` 4489, `login` 4505, `showLogin` 4551 | `GET /api/station`, `POST /api/login`, `/api/me` | Browser only; in the app the home window logs in |

### 1.2 Window types (registry → renderer)

Server registry: `crates/show/lumacore-tree/src/window_types.rs:277` `builtin_window_types()`,
published at `Root.Resources.WindowTypes` (`apps/lumacore-station/src/core/objects/system.rs:497–550`)
and `GET /api/window-types` (`crates/ui/lumacore-http/src/endpoints/window_types.rs:10`).
Client map: `TAB_KINDS` 1515–1522 (label, glyph, tab-dot colour, `build`, optional `onUi`);
types the client has no `build` for are dropped from the picker (`pickerTypes` 2064).

| no | name | Builder | Data source | Settings (schema) | Controls |
|---|---|---|---|---|---|
| 1 | `terminal` | `buildTerminalPanel` 1525 → `termFor` 1043 | `/ws/shell` binary frames per viewport id; `viewports` msg (`syncTermSizes` 1062) | font_size, cursor_blink | xterm (hand-tuned theme `TERM_OPTS` 1022) |
| 2 | `groups` | `buildPoolPanel(win,'Group')` 2433 + `makePoolBtn` 2510 | `/ws/pools` subscribe (`poolsConnect` 2394, `poolSubscribe` 2416); editor `GET /api/pools/{pool}/{no}` | `pool_settings()` window_types.rs:196 (action, datapool, first_index, slots, columns, rows, size, font_size, show_title, show_empty, 4 colours as Text max 9) | generic cell renderer; gestures `poolPointer` 2583; ctx `poolCtx` 2554; selector 2640; editor 2691 |
| 3 | `empty` (hidden) | `buildEmptyPanel` 1534 | — | — | one "Add window" button |
| 4 | `sysmon` | `buildSysmonPanel` 3040 → `sysmonFor` 2904 | `/ws/monitor` (`sysmonConnect` 2872) | font_size, level, subsystems, search, auto_scroll | hand-built toolbar (`.sm-bar` input/select/button) duplicating 2 of the settings |
| 5 | `dmx` | `buildDmxPanel` 3398 → `dmxFor` 3175 | `/ws/dmx` (per window socket) | universe, address, view, readout, address_mode, level_bar, columns, show_value/attribute/id, only_selection | hand-built toolbar (`.dx-bar`) duplicating 4 settings; hover-only status line (`mouseenter` 3276) |
| 6 | `realtime` | `buildRealtimePanel` 3151 → `realtimeFor` 3062 | `/ws/health` (per window socket, 4 Hz) | none | read-only HTML tables; own escaper `rtEsc` 3060 |

Pool window details: `PoolDef` (`apps/lumacore-station/src/core/pools.rs:18`) = window type name,
`actions: [PoolAction{name, command template "{obj}"}]`, `cells: fn(ctx)->Vec<PoolCell>`,
`detail: Option<fn>`; `PoolCell` (pools.rs:53) = `no, guid, name, color "#rrggbbaa", icon,
owner (red lock), referenced, locked, applicability all|some|none, bar {kind, value 0–1, text},
badges`. Only Group has one (`core/commands/groups.rs:1309`). The cell renderer is generic; the
**editor is Group-specific** (`openPoolEditor` 2691 returns early for other pools and just sends
`Edit <pool> n`), and polls `GET /api/pools/Group/n` every 1 s (`setInterval` 2708).

Gestures on a pool cell (`poolPointer` 2583): pointerdown → hold timer (profile `hold_ms`) →
full-screen selector; move > 6 px → drag ghost → `Move/Copy … At n` (Copy = Ctrl/Meta held —
impossible on touch; `pool_drag Off` profile switch is the only touch guard); pointerup →
`poolTap` 2541 → WS `{type:"tap", object, no, action}`. No `setPointerCapture`, no
`touch-action` on `.gpbtn`, so on touch the browser may scroll/pan the grid and fire
`pointercancel` mid-drag (there is no pointercancel handler on pool buttons).

### 1.3 Overlays / menus (all `.lc-overlay` + `.lc-box`, CSS 341–398)

| Overlay | Markup | Open / render | Data | Build style |
|---|---|---|---|---|
| Shutdown Menu `#power-overlay` | 818–828 | `openMa('power-overlay')` 3525 (UI key `power`, Alt+X, `open_menu` event) | — | Static `.lc-tile` grid (3 cols), inline `onclick="powerCommand('…')"` |
| Menu `#menu-overlay` | 831–849 | `openMenu` 3545 | `/api/info` for version | Static 4×4 `.lc-tile` grid; 8 of 13 tiles `disabled` hard-coded; inline onclicks |
| Backup `#backup-overlay` | 852–896 | `openBackup` 3912, `loadShows` 3919, `renderBackupPre` 3930, `backupMode` 3947, `renderShowList` 3964, `backupGo` 3994 | `GET /api/shows`, status `show` | Static markup + ids (`bk-*`), own input CSS (CSS ~375–397); emoji icons 🗁 🗋 ⭳ that don't render |
| Network `#net-overlay` | 899–906 | `npOpen` 3565 → `renderNetworkMenu` 3618 | `GET /api/network` **polled 1 s** (`npRefresh` 3574) | Template-string `innerHTML` tables; `data-cmd` / `data-arm` / `data-set` / `data-pick` attribute wiring (`npWire` 3605, `npWireProtocols` 3802) |
| DMX Protocols `#proto-overlay` | 909–917 | `renderProtocolsMenu` 3693, `npLinesTable` 3741, `npMatrix` 3771, `npDiscovery` 3790, `npRouteEditor` 3816 | same, polled | Same mini-framework; route editor is an absolutely positioned `.np-editor` with ids `re-a/re-b/re-w` |
| LumaSessions `#luma-overlay` | 920–927 | `renderLumaMenu` 3862 | `GET /api/lumasessions` polled 1 s | Table + station form (`.np-station`) + footer buttons |
| Connection lost, Login, fb-popup | see 1.1 | | | |

Common mechanics: `openMa`/`closeMa` 3525–3534 (blur terminals, backdrop `mousedown` closes
except ask/login), Esc closes the top one (3535). Two separate overlay systems coexist: the
`.lc-overlay` family (class `open`) and the older id-listed family `#confirm-overlay,
#prompt-overlay, #store-overlay, #picker-overlay, #objset-overlay` (CSS 638–642, own `open`
rule, own `z-index:100`), whose box is `.shutdown-box` (CSS 672) instead of `.lc-box`.
`shouldForward` (1446) has to enumerate both families by id to stop key forwarding.

### 1.4 Pop-ups / dialogs

| Dialog | Markup | Code | Trigger | Shape |
|---|---|---|---|---|
| Confirm (server message box) `#ask-overlay` | 930–937 | `openAsk` 4031 | UiEvent `confirm` {title, message, command, ok, unsaved, countdown} | Buttons built by a local `button()` closure; reruns `command /NoConfirmation [/Save|/NoSave]`; countdown bar |
| LumaSession merge decision | reuses `#ask-overlay` | `openDecision` 4094 | status `lumasession.decision` {id, title, message, details[], options[[label, command]], default, deadline_ms, paused, merge_blocked} | Second copy of the button + countdown code |
| Claim ask (Keep / Hand over) | reuses `#ask-overlay` | `openClaimAsk` 4155 | status `claim_asks[0]` {object, from, deadline_ms} | Third copy of the button + countdown code |
| Oops confirm (`OopsConfirmation`) | arrives as `confirm` UiEvent | `openAsk` | server `oops::confirm_first` | — |
| Password prompt | `#prompt-overlay` | `promptText(…, {password:true})` 1792 | UiEvent `password_prompt` | Builds a `Login "u" "pw"` line client-side |
| Local confirm `#confirm-overlay` | 996–1003 | `confirmAction(text, fn)` 3504 | pool Delete, view Delete | OK button hard-labelled "Delete" in markup |
| Text prompt `#prompt-overlay` | 986–994 | `promptText` 4400 | Rename window/pool/view, Move/Copy to… | One text field; Esc/Enter |
| Window settings popover `#settings-pop` | 971 | `openSettings(e, no)` 3413 | ctx menu "… settings…" | **The only schema-driven form**: renders `TAB_KINDS[kind].settings` (bool→checkbox, enum→select, text, int→number); positioned at the mouse; Apply writes `win.settings` and PUTs the layout (no command, no echo) |
| EditSetting pop-up `#objset-overlay` | 961–971 | `openObjectSettings` 4364, `closeObjectSettings` 4388 | UiEvent `edit_settings` {title, fields:[{name, kind (free text label), value (text), line "Set … {value}"}]} | Every field is a plain unstyled `<input>` (no bg/border CSS → native look, 36 px, blue focus ring); kind shown as a hint; quoting decided client-side |
| Store View dialog `#store-overlay` | 939–950 | `openStoreDialog` 4295, `closeStoreDialog` 4337 | UiEvent `store_view` / ctx menu | Label + screen toggle buttons (`.sv-scr`) + All/None/OK |
| Pool selector (hold) | built in JS | `openSelector` 2640 | hold on a pool cell | Full-screen grid of cloned buttons |
| Group editor | built in JS | `openPoolEditor` 2691, `renderEditor` 2743 | ctx Edit / NAV Enter | List with HTML5 `draggable` reorder (2753–2763 — **does not work on touch**) + grid with pointer drag (`touch-action:none` CSS 248) |

### 1.5 Home page of the app (`apps/lumacore-app/home/home.html`)

Separate document (served as `lumacore://localhost/`, wry `WebViewBuilder` in
`apps/lumacore-app/src/app.rs:1596`): views countdown / servers / users / busy, confirm dialogs
`confirmClean` 346, `confirmStop` 376, `confirmOther` 407, `confirmHung` 439. **Copies the
`:root` tokens of index.html verbatim** (home.html:7–18) and has its own `.btn`, `.section`,
`.server`, `.check`, `.link` classes — a second, divergent design system. **No viewport meta.**

---

## 2. Building blocks vs duplication

### 2.1 What exists and is reusable

- **Tokens** (`:root` 13–24): `--bg --chrome --chrome2 --border --dim --text --amber --green
  --red`, `color-scheme: dark`. Only colours — no spacing, radius, font-size, hit-size, z-index
  or motion tokens. 61 distinct hard-coded hex colours remain in CSS/JS (status colours
  `#ffd700 #00ff00 #00a2ff #ff9f2e #3cc8e8 #ff6b6b #9fd0ff …`, xterm theme, pool defaults).
- **`renderCtx(e, items)`** 2028 — the only real generic widget (declarative item list).
- **`openSettings`** 3413 — a schema→form renderer, but private to window settings.
- **`.lc-overlay/.lc-box/.lc-title/.lc-tile/.lc-ico/.lc-info`** — a de-facto modal + tile kit.
- **`fbCard`/`fbPopup`/`fbProgress`/`fbFlash`** — feedback surfaces, reasonably generic.
- **UI-key gesture layer** 1390–1436: any element with `data-key` gets press / release / hold /
  release-after-hold / double, Meow, LEDs (`renderKeyLeds` 2264). This is the right primitive
  for a touch button kit (already pointer-event based).
- **`np*` mini-framework** 3560–3613: `data-cmd` (run command), `data-arm` (two-step
  confirm button), `data-set`/`data-pick` (input/select → `Set …`), `npRun` (command + 2 delayed
  refreshes). Good idea (declarative command binding) but local to the three menus.
- `esc` 2535 / `rtEsc` 3060 / `npEsc` 3560 — three identical escapers.
- `poolColumns(grid)` 2812 — grid keyboard navigation helper reused by the picker.

### 2.2 Duplication / inconsistency (concrete)

- **Buttons — ~10 visual families:** `.sl-btn`, `.cb-tri/.cb-purr/.cb-icon`, `.ptab`,
  `.pk-tab/.pk-item`, `.lc-tile`, `.np-btn` (+ `.yes/.no/.go/.danger`), `.np-tabs button`,
  `.btn-cancel`, `.btn-shutdown`, `.store-ok`, `.bk-go`, `.store-x`, `.pe-foot button`,
  `.sm-bottom`, `.dx-bar button`, `.vbtn`, `.vb-nav`, `.login-user`, `.sv-scr`, plus inline
  `style="color:var(--amber);border-color:var(--amber)"` for primary OK (968, 992, 3480). The
  "primary" button is spelled 4 ways (`.store-ok`, `.np-btn.go`, `.bk-go`, inline style); the
  "danger" button 3 ways (`.btn-shutdown`, `.np-btn.danger`, `.ci.danger`).
- **Inputs — 7 copies of the same rule** (bg `--bg`, 1px `--border`, radius 3–4, amber focus):
  `.pk-filter input`, `#login-password`, `#bk-desc/#bk-search/#bk-newname/.bk-settings select`,
  `.sp-row input/select`, `.sm-bar input/select`, `.dx-bar input/select/button`,
  `.np-table/.np-foot/.np-editor/.np-station input`, `.store-row input`, `#prompt-input` — and
  `#objset-fields input` which has **none** of it (native look, blue focus — screenshot 13).
  Heights vary 20–36 px.
- **Tabs — 4 implementations:** panel tabs `.ptab`, picker `.pk-tab`, menu tabs
  `.np-tabs/.np-sub`, (+ home). Different radii, colours, active state (amber text vs amber
  border vs bg).
- **Modals — 2 families** (`.lc-overlay`+`.lc-box` vs id-list + `.shutdown-box`), 3 title
  styles (`.lc-title` span, `h2` in `.shutdown-box`, `.pk-hdr h2`, `.store-hdr h2`), 3 close
  buttons (`.store-x`, `.pe-close`, none).
- **Confirm-style dialogs — 4 code paths:** `openAsk`, `openDecision`, `openClaimAsk` (each
  re-implements `button()` + countdown tick + primary focus), and `confirmAction`. Plus
  `npWire`'s `data-arm` "click again" inline confirmation, a fifth pattern.
- **Settings forms — 3 renderers for one concept:** window settings (`openSettings`, typed
  schema), EditSetting (`openObjectSettings`, untyped text fields), and the hand-built
  toolbars in sysmon/dmx that edit the same window settings a second way (`store()` closures
  2012 & 3199). The menus' `data-set`/`data-pick` are a fourth way of editing typed properties.
- **WebSocket plumbing — 6 sockets + SSE, each with own reconnect:** `/ws/shell` 1114,
  `/ws/feedback` 1229, `/ws/pools` 2397, `/ws/monitor` 2875, `/ws/health` 3128 (one per
  Realtime window), `/ws/dmx` 3213 (one per DMX window), SSE 4432; plus polling (`npRefresh`
  1 s, group editor 1 s). No shared subscribe/retry/backoff helper.
- **Grid/list widgets:** pool grid, picker grid, menu tile grid, store screens grid, DMX
  sheet, login user grid, backup list, sysmon list, np tables — all separate.
- **Icons:** Unicode glyphs/emoji everywhere (⏻ ⚙ 🐾 ⊞ ≣ ▦ ⏱ ⇄ ⌘ ≋ 🗀 🗁 🗋 ⭳ 🗑 🔒 ⛓ ◉ ⌨ ◑ ◐ ❄ ◎ ⇋
  ⬒ ◫ …). Server window types also send a glyph (`WindowType.icon`). Several don't exist in the
  system font (Backup tiles render as bars — screenshot 10), and they render differently per
  OS/WebView (emoji vs text presentation). No icon set, no SVG sprite.
- **Markup by template strings:** 56 `innerHTML` assignments, 41 inline `onclick="…"`, 15
  inline `style=`; HTML escaping is manual and easy to forget (e.g. `renderCtx` 2035 writes
  `item.lbl` raw — labels include window labels and pool names, e.g. 2560, 2012).
- **Re-render strategy:** `renderLayout` destroys and rebuilds every panel on any layout
  change; menus re-render whole `innerHTML` every second unless "busy" (focused input).

### 2.3 Weak points of the named areas

- **Window settings popover** (3413): positioned at the mouse point, 230 px wide, 11.5 px text,
  72 px number fields; Apply bypasses the command line (PUT layout) while EditSetting goes
  through `Set` lines — two write paths for "settings"; colours edited as `#rrggbb` text; a
  groups-only note is hard-coded (3452); `inputs[spec[0].key].focus()` crashes for a type with
  no settings (guarded only by the ctx menu hiding the entry).
- **EditSetting pop-up** (4364): `SettingField.kind` is a human label (`"minutes (0 = off)"`,
  `"Merge/UseMaster/UseMine/Cancel"` — `apps/lumacore-station/src/core/commands/edit_setting.rs:27`,
  `PropKind` label via `props::kind_label`) so the client cannot render a select / toggle /
  stepper; everything is a text box; quoting is guessed (`/\s/` or kind==="text`).
- **Prompt/confirm**: global single-instance state (`promptFn`, `confirmFn`), no stacking, OK
  label fixed to "Delete", no validation/inline error, no numeric keypad.
- **Network / LumaSessions / DMX Protocols overlays**: fixed 1040 px box (`.np-box` CSS 565),
  wide tables with `white-space:nowrap`, 20–22 px tall buttons/selects, inline-editable table
  cells, an absolutely positioned route editor, polling + "don't re-render while focused"
  heuristics, two-step `data-arm` confirm that changes the button text for 3 s. Not data-driven
  (column sets and command strings are hard-coded in JS; server sends ad-hoc `json!` from
  `apps/lumacore-station/src/net/menu_views.rs:13,90`). Unusable on a phone (no reflow).

---

## 3. Input handling and touch

### 3.1 What is there

- Pointer events: UI-key gestures (1390–1414, capture phase, `pointercancel` handled), view
  buttons (`attachTap` 4249, `pointerleave` cancels), pool cells (2583), group editor grid
  (2779). `touch-action:none` only on `.pe-cell` (CSS 248).
- Keyboard: every key forwarded to the server (`forwardKey` 1454) unless an overlay/input has
  focus (`shouldForward` 1442); Alt+B/V/X local (`uiShortcut` 2249); picker / ask / prompt /
  store / settings have Enter/Esc; pool navigation comes back as `ui` events (`poolUi` 2822).
- Viewport meta: `width=device-width, initial-scale=1.0` (line 5) — but `html,body
  {overflow:hidden}` and fixed px layout; home.html has none.
- `devicePixelRatio`: only used for screenshot scale (4457). xterm handles its own DPR.

### 3.2 Everything that breaks or degrades on touch (tablets / phones)

1. **Split resize handles are mouse-only**: `attachDrag` uses `mousedown/mousemove/mouseup`
   (1929–1951); 4 px handle with ±4 px invisible hit area (CSS 65–72) ≈ 12 px. On iOS/iPadOS
   WebKit compat mouse events don't fire for drags → splits cannot be resized at all.
2. **Close × on panels is hover-only** (`opacity:0` until `.panel-hdr:hover`, CSS 83–90,
   ui-framework.md D2) and 18×18 px. On touch it is invisible (a first tap may make it appear
   via sticky hover in some WebKits — inconsistent).
3. **Right-click-only actions** (D4): split, add window, change type, rename, window settings,
   close tab (`showCtx` 1886/1904), pool Store/Edit/Label/Move/Copy/Delete/Settings
   (`poolCtx` 2527), pool-window settings (2499), sysmon/dmx settings + "jump to newest"
   (3026, 3385), view button store/rename/clear/delete (4239). iOS Safari/WKWebView do **not**
   fire `contextmenu` on long-press; Android Chrome does but also starts text selection. No
   long-press fallback exists, and long-press on pool cells is already taken by the selector.
4. **Hover-only information**: 35 `title=` tooltips + 8 `.title=` assignments carry real info
   (status icon meaning, merge-blocked reason 4121, lock owner, "Click to pause the
   countdown"); DMX sheet channel details only on `mouseenter` (3276, 3382); `:hover` styles
   (30 rules) are the only affordance for many controls.
5. **Hit targets**: `.panel-close` 18 px, `.ptab` ~26 px tall, `.np-btn` ~22 px, `.sp-row`
   inputs ~22 px, `.sm-bar`/`.dx-bar` controls ~20 px, `.ci` ~26 px, `.store-x` text-sized,
   `.fb-x` 14 px text, view-bar ▲/▼ ok (50 px), sidebar 50 px ok, pool cells ≥ 56 px ok,
   DMX cells 44×34 ok. Recommended ≥ 44 px (Apple) / 48 dp (Material).
6. **Font sizes**: 8 px (`.gp-num`, `.vbtn-num`, `.gp-badges`, `.gp-bar span`), 9 px
   (`.cb-text`, `.sl-mode`, `.gp-lock`), 9.5–10.5 px common; base 13 px. iOS zooms the page
   when focusing an input < 16 px (inputs are 11–12 px) unless `maximum-scale=1`/
   `font-size ≥ 16px`.
7. **Pool drag Copy needs Ctrl** (2586, 2608) — impossible on touch; drag vs scroll conflict
   (no `touch-action` on `.group-pool`/`.gpbtn`, no pointer capture, no `pointercancel` handler
   in `poolPointer`) → a scroll gesture can become a Move, or a drag gets cancelled silently.
8. **Group editor member reorder** uses HTML5 drag-and-drop (`draggable`, `ondragstart`,
   2753–2763) — not supported on touch in WebKit/most mobile browsers.
9. **Double-click actions**: `ondblclick` on backup rows (3983, load/save at once) and login
   users (4577); UI-key double press relies on synthesized `click` timing (1416) — on touch
   devices with tap delay / double-tap-zoom this misfires (no `touch-action:manipulation`).
10. **Hold timers**: pool hold uses `uiKeys.hold` (500 ms), view button `HOLD_MS` 600 (4248),
    UI keys `uiKeys.hold` — three timers, two values; no visual hold progress; iOS long-press
    also triggers the callout/selection unless `-webkit-touch-callout:none; user-select:none`.
11. **Virtual keyboard / command line**: the command line is an xterm (hidden textarea).
    Typing is forwarded from `keydown` (`xtermKey` 1496 → `forwardKey`); Android IMEs send
    `key:"Unidentified"` / composition events → `shouldForward` drops them (1444) and the
    characters only arrive through `term.onData` as raw "paste" bytes (1052) — no
    completion/history semantics, possible double input. No on-screen keypad, no VK button
    (roadmap item `ShowVKButton`). No `visualViewport` handling: when the soft keyboard opens
    the 50 px cmdbar at the bottom of a `height:100%; overflow:hidden` page is covered (iOS
    doesn't resize the layout viewport). No `inputmode`/`enterkeyhint` on any input.
12. **Viewport units**: 7 uses of `100vh/100vw` (overlay max sizes, `#history-pop` 42vh, fb
    cards) — wrong on mobile (URL bar); no `dvh`/`svh`, no `env(safe-area-inset-*)`.
13. **Fixed widths**: overlays 800–1040 px, `.pk-box` 800, `.backup-box` 860, `.ask-box` 420,
    `.login-box` 440, `#settings-pop` 230 at the pointer, sidebars 50 + 100 px always, cmdbar
    `#cb-line` `min-width:300px` → on a 390 px phone the bar overflows; no `@media` query
    anywhere (0 hits).
14. **Backdrop close** uses `mousedown` (3532, 2164, settings 3498) — on touch, compat mouse
    events fire late or not at all; a tap outside may not close.
15. **Scrolling**: pool grid, sysmon list, DMX grid, np body are `overflow:auto` without
    `overscroll-behavior:contain`; body is `overflow:hidden` so iOS rubber-banding of the
    whole page is avoided, but nested scrollers can chain.
16. **Focus/selection**: no `:focus-visible` styling (keyboard focus on most buttons is
    invisible or native blue); `user-select:none` only on bars — long-press on text in panels
    selects text; the focused panel concept (amber frame) is set on `pointerdown`.
17. **Multi-touch**: none (no faders/encoders yet). The kit will need pointer capture per
    pointerId for simultaneous faders.

---

## 4. Server-side UI descriptions a generic kit can render from

| Source | Shape | Where | Used by the client? |
|---|---|---|---|
| Window types | `WindowType {no, name, label, category, icon (glyph), description, keywords[], hidden, settings: [SettingDef]}`; `SettingDef {key, label, type: bool \| int{min,max} \| enum{options:[[value,label]]} \| text{max_len}, default, hint?}` + `coerce`, `resolve_settings` | `crates/show/lumacore-tree/src/window_types.rs:19–125`; `GET /api/window-types`; `Root.Resources.WindowTypes`; `Set Window n <key> <value>` | Yes — picker + `openSettings`. **Best schema in the codebase.** Gaps: no colour, float, slider/percent, file, link/object-ref kinds; no grouping/sections; no per-setting rights; no "live" vs "apply" flag |
| Tree writable props | `Prop {name, field, kind: Text \| Bool \| Int{min,max} \| Choice(&[..]) \| Link{kind, resolve}, rights, doc}`, `WriteDef {props, apply}` on `TreeDef::write` | `apps/lumacore-station/src/core/props.rs:18–61`, `META` 64–93 | Only flattened into `SettingField` strings for EditSetting — **the type information is lost on the wire** |
| EditSettings UiEvent | `{title, fields: [SettingField{name, kind: String (label), value: String, line: "Set X n Prop {value}"}]}` | `crates/foundation/lumacore-wire/src/shell.rs:242, 295`; built in `core/commands/edit_setting.rs:27,49` | `openObjectSettings` 4364 — text inputs only |
| Pool | `PoolDef {window, actions:[PoolAction{name, command}], cells fn, detail fn}`; `PoolCell {no, guid, name, color, icon, owner, referenced, locked, applicability, bar{kind,value,text}, badges[]}`; `/ws/pools` `{type:"pool", id, pool, window, actions, datapool, cells}` | `apps/lumacore-station/src/core/pools.rs:18–79`; `endpoints/pools.rs` | Yes — generic cell renderer; actions feed the "Tap action" enum. Editor (`detail`) has no schema → Group-only UI |
| UiEvent (server → UI) | tagged `action`: move{dir}, activate, edit, delete, erase, back, root, focus_terminal, store_view{target,no}, focus_window{no}, password_prompt{user}, screen_config{no}, user_changed{user,token}, logged_out{disconnect}, connect{…}, edit_settings{…}, open_menu{menu}, close_menu, notice{text,level}, feedback_settings, confirm{title,message,command,ok,unsaved,countdown} | `crates/foundation/lumacore-wire/src/shell.rs:185–270` | `handleUiEvent` 1785 dispatch table. `open_menu` only knows `menu/backup/shutdown` strings; no generic "open dialog with this schema" event |
| Status snapshot | `{mode, modes[], walk, selected, claims[{object,owner,mine}], claim_asks[], lumasession{status, health, latency_ms, pending, master, show_status, decision{…}}, show{…}, user}` | `lumacore_wire` typed `Status` | `renderStatus` 2272 (decision and claim-ask pop-ups are driven from status, not from UiEvents) |
| UI keys | `UiKey {id, label, icon, tooltip, owner, press, release, hold, release_after_hold, double_press, meow{…}, state (tree path → LED), locked}`; `GET /api/ui-keys` | `crates/show/lumacore-tree/src/ui_keys.rs` | Gesture layer + LEDs; **labels/icons/tooltips are not used to render buttons** — the sidebar buttons are hard-coded markup |
| Feedback | `{kind progress\|flash\|message\|status, importance, level, category, title, text, progress{range,value}, key, glyph, active}` | `apps/lumacore-station/src/feedback.rs` | Yes, `fbHandle` |
| Menu views | ad-hoc `serde_json::json!` for Network / LumaSessions | `apps/lumacore-station/src/net/menu_views.rs:13,90`; `StationView.network/lumasessions` (`server/state.rs:121`) | Rendered by hand-written templates, no schema |
| Object registry | `ObjectDef` (keyword, aliases, Doc, verbs, `pool: Option<&PoolDef>`, `tree: TreeDef{write}`) / `CommandDef` (args shape, rights, status Planned) | `core/objects`, `core/commands/registry.rs` | Not exposed to the web UI except through completion in the shell |

What a generic kit could render today without server changes: window settings forms (typed),
pool grids (typed cells + actions), UI-key buttons (label/icon/tooltip/gestures/LED), feedback.
What needs a wire change to become data-driven: EditSetting (send `PropKind` as a typed
schema like `SettingDef`), menus (a view schema: tables with columns + per-cell control kind +
command template), dialogs (a generic `dialog {title, message, fields?, buttons:[{label, command,
style, default}], countdown?}` UiEvent would replace confirm/decision/claim-ask special cases).

---

## 5. Screenshots (target/screenshots, 1400×900, Chromium headless)

- **01_start / 02 terminal**: tiny 9 px sidebar labels (BLND/HI/FRZ/SOLO) in dim grey, 8 px
  view-bar slot numbers, empty view bar; the "Connected" card covers the top of the view bar
  and part of the panel (no reserved area). Status icons 9 px text.
- **03 groups**: 12×12 fixed grid fills the panel; only 3 groups — 141 empty, numbered cells
  (show_empty default true) dominate; title bar "Groups DataPool 1 · Select" is a third header
  row (window tab, pool title, grid). The second tab "Groups" next to the active one is the
  default layout's second window — two windows with the same label look like a bug.
- **04 sysmon**: whole list dark red because it is "PAUSED" (scrolled up = red background,
  console rule) — reads as an error state at first glance; toolbar search/select ~20 px tall.
- **05 dmx**: 20th column cut off at the right edge ("ColorAd…"), known finding; toolbar
  controls 20 px; legend on the right in 11 px.
- **06 realtime**: Clock row `undefined µs` / `undefined ppm`; right-hand stage labels clipped
  ("Inputs 6 µs 17…"); monospace tables with very wide empty columns.
- **07 menu**: 13 tiles in a 4×4 grid, 8 disabled (greyed, low contrast), LumaSessions alone in
  row 4; Quick Save marked by a green left bar only; no icons on menu tiles while Shutdown and
  Backup tiles have icons.
- **08 network**: Yes/No buttons ~20 px tall, green text only (state shown by text colour),
  disabled loopback "Yes" barely distinguishable; footer buttons differ in style from the tabs.
- **09 DMX Protocols**: network tab "Default not mapped" as an amber-bordered pill, sub-tabs
  another style, footer select/input/button heights differ (22–26 px); large empty body.
- **10 backup**: tile icons render as orange bars (🗁 🗋 ⭳ missing glyphs), 🗑 renders as a
  colour emoji — mixed icon styles; AutoSave Interval select says "Off" while the header says
  "Autosave in 14:47"; Save/Save As tiles use a green left bar, Delete a red one.
- **11 LumaSessions**: footer of 7 small buttons with mixed states (amber disabled, red
  disabled, green "Invite: On", red "LumaSession: On" = an *on* state coloured as danger);
  station form uses uppercase micro-labels (10.5 px) unlike other forms.
- **12 power**: amber outline icons (good), 3-column grid with Disconnect alone in row 2.
- **13 EditSetting Show**: a different dialog family — native grey inputs with a blue focus
  ring (all other inputs focus amber), labels left/hint below, enum fields (MergeDefault) as
  free text, Cancel/OK styled like `.btn-cancel` with inline amber.
- **home-1..4**: separate visual language ("MEOW LumaCore" header, `.btn`, uppercase section
  labels); "Clean start…" link sits on top of the "ON THE NETWORK" heading (overlap, known);
  quit dialog's Save button gets a blue native focus ring; boxes fixed ~560 px centred.

General: four different "primary" treatments (amber outline, amber text, green left bar, blue
focus ring); disabled states inconsistent (opacity .3 / .35 / .4 / .5); no common spacing scale
(paddings 3/4/5/6/7/8/10/12/14/16/18/24/28 px).

---

## 6. Docs, decisions and open items (file:line)

### 6.1 docs/dev/ui-framework.md
- D1 executor strip fixed at bottom (default) — ui-framework.md:7–16 (not built).
- D2 close × on header hover (default; "always visible prevents confusion") — :20–29.
- D3 multiple tabs per panel — :33–42.
- D4 open panels via right-click → split; alternatives "visible + buttons", "drag panel type
  in" — :46–56; window spawning decided (empty pane, Add Window picker, commands
  `Store Window …`) — :58–72.
- D5 layout server-owned per screen, stored views, view buttons — :76–83.
- D6 terminals stacked rendering, viewports + layers — :87–101.
- D7 command line bar (50 px) — :105–120.
- D8 Shutdown menu / Menu / Backup overlays whose buttons run commands; confirm UI event
  pattern (`/NoConfirmation`) — :124–145.
- Related: REST list, SSE, sidebar 50 px / ViewBar 100 px dimensions — :149–161.

### 6.2 docs/dev/api/http-api.md
- Goals: server owns truth, every change is a command, no web-only model — :20–29.
- REST routes table (window types, screens, pools, network, lumasessions, views, feedback,
  ui-keys) — :79–108; "menu, show and pool routes are read-only" — :110–113.
- SSE events — :115–127. WebSockets (`/ws/shell`, monitor, feedback, health, dmx, pools) —
  :129–141; `/ws/state`, `/ws/preview` stubs not routed — :140–141; shell packet types from
  the browser (`key`, `key_hold`, `key_up`, `focus`, `live_screen`, `tap`, `command`,
  `key_action`, `resize`, `viewports`) — :150–159.
- Static files `Cache-Control: no-store` (WKWebView staleness) — :163–166.

### 6.3 Handoffs
- thin-client: "Phones: a 'Phone' screen config is just a config with one small screen" —
  docs/plans/done/2026-10-10-thin-client.md:87; one OS window per screen, local display positions —
  :59–60, :174–185; Not done: GUI manual pass, macOS bundle — :194–197.
- feedback-system: surfaces + layering ("notifications above windows, below modals") —
  docs/plans/done/2026-10-10-feedback-system.md:54–62; Message Center window not done — :95–96,
  :125–128; plugin progress API names — :76–77.
- ui-keys: keys are data, UI engine only reports gestures — docs/plans/done/2026-10-10-ui-keys.md:29;
  plugin keys with icons + live state, "no placement hints" — :32, :36; plugin manifest TOML
  `[[button]]` (label, icon file or glyph, tooltip, press/double/hold, state) — :41–72;
  "Actions are commands — a plugin exposes behaviour as published commands, never UI code" —
  :74–75; Later: key editor window, placement in bars, plugin icons, live state rendering —
  :94–99.

### 6.4 visions/notes
- terminal-menu-mode.md:12–32 (menu mode in the TUI; also roadmap Later).
- client-types.md:16–28 (clients declare a role; partial sync) — relevant to phone/tablet
  clients.
- controllers-and-stations.md:10–58 (controllers per station, mappings per user).
- watch-quake-panel.md (TUI only). effect-engines.md, transport-types.md — not UI.

### 6.5 docs/archive/decisions-2026-10.md (UI-relevant)
- §11 Menus, LumaSession, staged patch — three console-style overlays, Network tabs, DMX
  Protocols tabs/sub-tabs/routing matrix editor, LumaSessions grid — decisions.md:641–690;
  DMX window spec — :630–640.
- §17 Keys and Keymaps — UiKey per gesture, Meow, LEDs, gestures in app/web — :807–834.
- §21 decision pop-up (message box: Merge / Use Master / Use Mine / Cancel, countdown,
  click pauses) — :1106, :1122–1129, :1172, :1218, :1296–1297.
- §22 Unified pool window — PoolDef, `/ws/pools`, gestures (tap, hold → full-screen selector,
  drag moves / Ctrl-drag copies, `pool_drag Off` for touch, right-click menu), group editor,
  window settings like established consoles — :1383–1421; built — :1450–1468; the view widget / pool model —
  :1333–1337.
- §23 App ↔ local server pop-ups (another version, hung stop) — :1680–1739 (e.g. :1700, :1800).
- §29 11b: `EditSetting <object>` pop-up in web/app, terminal list — :2783–2786, :2836–2837;
  Sequence pool window "with the UI kit" — :2745–2746.
- §29 11c: Oops window "comes with the touch UI kit"; OopsConfirmation pop-up — :2883–2891,
  :2933.
- §29 11d: capture in every client incl. WKWebView / WebKitGTK / WebView2, vendored libs, the
  frontend loads nothing from the internet — :2955–2959; screenshots script from the
  registries — :2969–2977, :2999–3001; findings for the UI kit — :3008–3013; "After step 11:
  UI first — touch friendly, implement all elements needed for future development: a
  touch-first UI element kit the later screens (playback, pools/editors, patch, feedback) are
  built from" — :3014–3016.

### 6.6 docs/meta/roadmap.md (open items for / around the kit)
- Next §1 Touch-friendly UI element kit — roadmap.md:184–192: element kit (buttons, faders,
  encoders, lists, grids, sheets, pop-ups, keypads) :189; virtual keyboard button on the
  command line bar :190; fix 11d capture findings :191; look at every
  screen via `./scripts/unix/screenshots` :192.
- With the kit: Sequence pool window :146; Oops window :162.
- Feedback: Message Center window :211; ProgressButton / plugin progress API :215.
- App: macOS bundle :223; manual GUI pass :225.
- Keys/plugins: UIKey editor + placement in bars :231; plugin manifest + Lua + icons + live
  state :232; keyboard double presses :233.
- Executors/faders :251–252; "Startup show failed to load" pop-up :270; per-client overlay
  state (completion / confirm shared per user) :287; Patch menu :294.
- Panel prototype leftovers: executor strip, visible split button, drag a panel type in :306;
  Store View without dialog :307; shortcut mode per screen :308; feedback leftovers :309.
- UI and windows :318–334: more window types :320; Command Line / History windows :321;
  Move/Copy Window :322; PoolDef for other pools :324; encoder bar :329; grey menu tiles :330;
  Settings pop-up pages :331; Shutdown menu extras :332; Backup extras :333; terminal menu
  mode :334.
- Visual regression (CompareImages) after the kit settles the look :350; Image pool / view
  thumbnails :351; Windows build :370.
- Dropped: React 19 + Vite + Tailwind stack — "the web UI is plain HTML/JS" :398.

---

## 7. Critique summary (for the kit design)

1. **Keep** the server-driven model: window-type `SettingDef`, `PoolDef/PoolCell`, UiKey
   gestures, every action a command line, `renderCtx`-style declarative item lists. The kit
   should be *renderers for these schemas*, not new hand-built screens.
2. **Unify** to one modal/dialog primitive (title, body, fields, buttons with
   role primary/danger/neutral, optional countdown, stacking, Esc/Enter, backdrop close on
   pointer) — replaces `.lc-overlay` + `.shutdown-box` families and openAsk / openDecision /
   openClaimAsk / confirmAction / promptText / store dialog code paths.
3. **One field renderer** for `SettingDef` and (after a wire change) typed `PropKind`:
   toggle, stepper/number with keypad, select/segmented, text, colour swatch, object link —
   used by window settings, EditSetting, menus' editable cells and sysmon/dmx toolbars.
4. **One button** (sizes S/M/L with ≥ 44 px touch size, variants, LED/state, `data-key`
   gestures, hold progress), **one tab bar**, **one table/list** (virtualised for sysmon),
   **one grid** (pool/picker/menu tiles), **one icon set** (inline SVG sprite, vendored).
5. **Tokens** for spacing, radius, font sizes, hit size, z-layers (panel < notifications <
   overlays < modals < critical), motion; replace 61 literal colours; share with home.html.
6. **Input layer**: Pointer Events everywhere with `setPointerCapture`, `touch-action`
   per widget, a long-press = context-menu fallback (with the pool's hold-selector conflict
   resolved, e.g. a visible ⋯ button or swipe), visible header actions (D2/D4 alternatives),
   `:focus-visible`, `dvh` + `visualViewport` for the soft keyboard, `@media (pointer:coarse)`
   sizing, safe-area insets, `touch-action:manipulation` to kill double-tap zoom.
7. **One connection layer** (subscribe/retry/backoff) and push instead of 1 s polling for
   menus and the group editor.
8. **Tests**: Playwright runs Chromium only at 1400×900 (`tests/e2e/playwright.config.ts`);
   add WebKit + Firefox projects and a touch/phone project (`hasTouch`, `isMobile`, 390×844,
   1024×768) — e2e selectors currently bind to ids like `#ask-actions`, `#settings-pop`,
   `#ctx`, `.lc-overlay.open`, `.pool-selector`, `.sm-state` (keep or migrate them
   deliberately).
