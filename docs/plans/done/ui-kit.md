> **Closed 2026-10-20:** built — step 12's kit through 12d part 5 as recorded below (decisions §30); every window is drawn from `crates/ui/lumacore-webui/web/kit/` (§36) — the `apps/lumacore-app/frontend/index.html` path named below is gone.
> Leftovers live in `docs/meta/roadmap.md` → Next 1 "Touch UI kit" (12e–12g). The research is in `docs/research/ui-kit/`.
> Kept as history: "where it stands" is 2026-10-13.

# Handoff — touch UI kit (step 12)

Decisions: [decisions.md §30](../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening) ·
Plan: [roadmap](../../meta/roadmap.md) Next 1 (12a–12g) ·
Q&A: [decisions-qa](../../archive/docs/archive/docs/plans/done/2026-10-10-decisions-qa.md) "Step 12 hardening".

## Where it stands (2026-10-13)

- Research done, kept unchanged in [docs/research/ui-kit/](../../research/ui-kit/):
  - `lumacore-inventory.md` — every window, overlay, dialog and building block in today's
    `apps/lumacore-app/frontend/index.html` with line numbers; every touch breakage; the
    server-side schemas a kit can render; e2e selectors that a refactor would break.
  - two studies of an outside console's UI element set and UI test framework (deleted
    2026-10-23 before the public release).
  - `online-touch.md` — Open Stage Control, QLC+, Chataigne, Companion, webaudio-controls,
    NexusUI, console (Eos / MagicQ and others) conventions; Pointer Events, `touch-action`, long press on iOS,
    `visualViewport`, webview quirks; libraries; the recommended architecture.
  - `themes.md` — REAPER Theme Adjuster / tweaker, a per-use console theme (what to
    avoid), VS Code / Ableton / Material / Radix; the proposed LumaCore theme model.
- Mockups: 14 boards, **locked 2026-10-12**, stored in `docs/dev/ui-kit/mockups/2026-10-12/` (sources open offline in a browser, PNGs); canvas https://claude.ai/artifact/FpJtQJw5bGAi5URPM3C4pf (private). 12a hardened; **12a1 built and committed** (`b158e40`, §30 "12a1 built"). **12a2 built and committed** (`7c84db2`, §30 "12a2 built"); **12a3 hardened** (§30 "12a2 / 12a3 hardening"): 12a2 = show format 7 (`.luma` folder tree + manifest, embedded media beside manifests, image / icon pools, look on the profile, GDTF FixtureTypeID + revisions, asset drives, profile export / import); 12a3 = the user surface (apps as basic clients, the one lock / login / connect surface, DeskLock per device, NormalTheme, built-in keyboard). 12a3 hardened and its mockups locked (`docs/dev/ui-kit/mockups/2026-10-12-12a3/`); **12a3 built** (§30 "12a3 built": devices + DeskLock, NormalTheme, core keys F9–F12, Admin = admin, format 8 `User.Color`, `<luma-surface>` / `<luma-avatar>` / `<luma-keyboard>` / dialog / `<luma-overview>`, the app's bundled connect page), committed `ceb5fc0`. **12b hardened** (§30 "12b hardening": one server-held `UiEvent::Dialog` with answer packets, close-everywhere-then-run, terminals' text form, relative numbers on the server, keypad, pick list + real links, per-device Keyboard setting + VK button, English layout); 12b mockups **locked** (`docs/dev/ui-kit/mockups/2026-10-12-12b/`: Asks, Inputs, PickList, VK, Terminal — TUI dialog as a centered window in the profile theme); **12b built** (§30 "12b built": server-held dialogs in the Status, terminals' centered window in the profile theme, relative numbers, keypad, pick list, Keyboard mode + VK button; not committed yet); committed `7f5bd8c`. **12b2 hardened** (§30 "12b2 hardening", before 12c, one step: servers list goodbye + beacon decides, Meow = Shift + held light, F12 full screen with corner handle + floating command line, command line bar ground, text selection only where you read or type, the window settings shell + typed descriptors + named choices pulled from 12c, a Users & Profiles window next to LumaSessions, Menu icons); + client feeds rule, System Monitor / Realtime / DMX at frame rate, suggestion engine foundations); 12b2 mockups **locked** (`docs/dev/ui-kit/mockups/2026-10-12-12b2/`, 8 boards); **12b2 built** (§30 "12b2 built": discovery goodbye, Meow = Shift, F12 full screen, typed settings + `<luma-settings>`, Users & Profiles `<luma-people>`, Menu icons, `/ws/feeds` + `<luma-vlist>` / `<luma-graph>`, suggestions `<luma-suggest>` + Usage, format 9, CLIENT_PROTOCOL 7, SESSION_PROTOCOL 8; committed `894cd79`); **12c hardened** (§30 "12c hardening", 2026-10-13: window bar = pinned settings + declared actions as `Set` / `Self` lines with `…` overflow and auto-hide, one state rule through `Self.<Property>` (bar state, auto chips, live title, key LEDs), layered resets, `WindowPreset` on the profile (holds everything), `Self` = windows now with an owner model, resolved where the line starts; two models — floor plan JSON in the show, window types declared in the Rust registry → JSON (no KDL); JSON export / import of View / ScreenConfig / WindowPreset into `library/ui/<kind>/`; leftover pick lists); 12c mockups **locked** (`docs/dev/ui-kit/mockups/2026-10-13-12c/`: WindowBar, BarSettings, Presets, SelfExport; canvas row 10); **12c built** (§30 "12c built": `<luma-window-bar>`, `Self`, `WindowPreset`, resets, `windows` feed, JSON export / import, format 10, CLIENT_PROTOCOL 8; not committed yet); **12c2 hardened + mockups locked** (§30 "12c2 hardening" / "design locked": safe users & profiles, compact menu tiles, multiplatform contracts + guards, desk view on a phone with hold-to-full-screen; `mockups/2026-10-13-12c2/`); **12c2 built** (§30 "12c2 built": locked Default, station asks for new / delete users and profiles, `<luma-tiles>` / `<luma-maxbar>` / `miniature()` / `toast()`, overlay + 44 px + token-font contracts, phone desk miniature + hold = full screen + `Set Self Maximized`, `Set Device ScreenConfig`, multiplatform spec; format 11, CLIENT 9, SESSION 9; committed `afeddb1`); **12d hardened** (§30 "12d hardening": `MenuDef` registry → `<luma-setup>`, Network / DMX Protocols / LumaSessions / Backup / Settings, typed station props, Backup recovery copies + versions + startup ask (format 12), everything live on `/ws/feeds` with open pre-login feeds + shared builds + revisions, rights ladder Guest…Admin; research in `docs/research/ui-kit/12d-*.md`); 12d mockups **locked** (`docs/dev/ui-kit/mockups/2026-10-13-12d/`: `<luma-grid>` + the slab in every setup menu, edit card, Load show dialog, show history matrix, setting kinds, power toggle A; canvas row 12); waits for go. 12g note: background hue + saturation adjusters replace Tint (§30, test board `ThemeBg` on the canvas, row 8).

- **12d part 3 built (2026-10-13)**, §30 "12d part 3 built": kit `<luma-grid>` (grid.js), the
  edit card (editcard.js), `<luma-slab>` + `<luma-power>` (slab.js), `<luma-setup>` (setup.js),
  kind badges / home marks / rights greying in `<luma-settings>`; Showroom "12d" sections
  (showroom12d.js, a stand-in station applies the lines to demo rows); Rust `core::menu_defs`
  (→ `/api/menu-types`), `core::settings_props`, `commands::set_rows`, the `Menu` and
  `GridColumns` objects.
- **12d part 4 built (2026-10-13)**, §30 "12d part 4 built": Network, DMX Protocols (feed-driven
  left tabs + top strip), LumaSessions (Stations grid | cards with per-network NIC columns, Known,
  This station) and Users & Profiles on `<luma-setup>`; the old overlays and `legacy` gone (Backup
  keeps its overlay); Settings › Assets + About; the station table `ShowData.Stations` (wiring in
  the show, identity on the station); station asks for Online Off / HoldOutput / Leave / Claim /
  deletes; `Claim LumaSession Station "X"`; format 14, SESSION 11. Screenshots:
  `target/screenshots/12d-part4/`.
- **12d part 5 built (2026-10-13)**, §30 "12d part 5 built": Backup on `<luma-setup>` (Library ·
  Show · History with List | Matrix · Recovery), the fixed coloured show actions, the Load show
  dialog (kit dialog: facts, five parts All / Pick / None, Pick… toggles) → `LoadShow` or ONE
  `Import Show` line; `core::backups` (versions on every save, recovery copies, `.trash`, pins,
  tiered cleanup + cap, startup ask, manifest facts + GUID → hash index + history),
  `core::show_import`, `Version` / `Recovery` objects, `GET /api/show-parts`, the `shows` feed's
  `history` param; `legacy` and the Backup overlay gone; setup.js: `matrix` view, `owner`,
  `slab-act` / `action` hooks; format 15. Screenshots: `target/screenshots/12d-part5/`. 12d is
  built; snapshots stay open.

- **12e hardened + design locked (2026-10-13)**, §30 "12e hardening" → "12e design locked": five
  artifact rounds (`docs/dev/ui-kit/mockups/2026-10-13-12e/`); research in this session's §30 notes
  (swipe-out pool actions, long-press-to-create, a selection grid's 16 grid tools, Appearance vs
  NodeMeta colour, no mini windows). Waits for go; build order at the end of "12e design
  locked".
- **12e built (2026-10-14)** in four parts, §30 "12e part 1 built" … "12e part 4 built": the
  station's narrow line, one hold, `kit/split.js`; `<luma-pool>` + `kit/sheet.js` + NodeMeta
  Color / Icon; `core::tools` + `kit/tools.js` + the `===` list mark + `<luma-grid reorder>` +
  `kit/groupeditor.js`; `<luma-grid view="mini">` + the phone's folded sidebar and swipe. Commits
  f99c628, 3aff4b9, ed3c45d, eb42a77. Open: the list mark on older pick buttons; mini views wait
  for the sequence / fixture-sheet windows.

## How to work

- Mockups first, built with the real tokens and sizes; the user's feedback goes into §30 before go.
- While building: drive the real UI in Chrome (own tab, a test station — never the user's app or
  server) and capture with `Screenshot` / `./scripts/unix/screenshots`; compare with the mockup.
- Keep or deliberately migrate the e2e selectors (`#ask-actions`, `#settings-pop`, `#ctx`,
  `.lc-overlay.open`, `.pool-selector`, `.sm-state`).
