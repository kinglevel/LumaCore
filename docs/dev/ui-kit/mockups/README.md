# UI kit mockups

The approved designs for the touch UI kit (step 12, [decisions §30](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)).
The UI is built from these; a window that needs an element not shown here waits until the
element is designed as a mockup, approved and added here (the strict rule in §30).

Each version is a dated folder, kept unchanged (archive, never delete). To change the design,
copy the folder to a new date, edit, and record why in §30.

## Versions

| Version | Status | Canvas |
|---|---|---|
| [2026-10-12](2026-10-12/) | **Locked** 2026-10-12 ("Yes lock the design") | https://claude.ai/artifact/FpJtQJw5bGAi5URPM3C4pf (private) |
| [2026-10-12-12a3](2026-10-12-12a3/) | **Locked** 2026-10-12 ("Approved") — adds the 12a3 states; the 2026-10-12 set stays | same canvas, row 6 |
| [2026-10-12-12b](2026-10-12-12b/) | **Locked** 2026-10-12 ("approved") — dialogs, keypad, pick list, keyboard, terminal; the earlier sets stay | same canvas, row 7 |
| [2026-10-12-12g-test](2026-10-12-12g-test/) | **Test board**, decided 2026-10-12 ("A tint replaced") — background hue + saturation; the 12g Theme board starts from it | same canvas, row 8 |
| [2026-10-12-12b2](2026-10-12-12b2/) | **Locked** 2026-10-12 ("approved") — 12b2: menu icons, Users & Profiles, window settings, F12 full screen, fixes, System Monitor, Realtime, suggestions / palette; replaces the 2026-10-12 Settings board for window settings | same canvas, row 9 |
| [2026-10-13-12c](2026-10-13-12c/) | **Locked** 2026-10-13 ("approved") — 12c: window bar (pins, actions, live title, chips, `…`, auto-hide), settings pins + Window bar tab + resets, `WindowPreset`, `Self`, JSON export / import | same canvas, row 10 |
| [2026-10-13-12c2](2026-10-13-12c2/) | **Locked** 2026-10-13 ("approve") — 12c2: safe user / profile asks, compact menu tiles, phone fixes, desk view on a phone (hold a pane = full screen); ResponsiveViews is the record of the options not taken | same canvas, row 11 |
| [2026-10-13-12d](2026-10-13-12d/) | **Locked** 2026-10-13 — 12d: setup menus on `<luma-grid>` + the slab (Network, DMX Protocols grid, LumaSessions, Backup, Settings, Users & Profiles), power toggle A; desktop + phone per board | same canvas, row 12 |
| [2026-10-13-12e](2026-10-13-12e/) | **Locked** 2026-10-13 ("lock it") — 12e: `<luma-pool>` (object sheet, NodeMeta colours, phone matrix, 30 × 30 miniature), split grip pill, phone desk (folded sidebar, swipe stop, miniature sheets), group editor tools (grouped bottom toolbar, low phone sheet, Lucide icons, yellow `===` list mark), narrow command line. Plain `.html` boards (artifact rounds 1–5), not `.dc.html`; the tool icons load Lucide from a CDN | artifacts (private): [round 1–2](https://claude.ai/artifact/KVgctMjNoZbRuuM41Hn8GP), [round 3](https://claude.ai/artifact/GcXNCnqoL8LMe9dDzptPm6), [round 4](https://claude.ai/artifact/WGbWBfSfndWwRw9iqmnXP5), [round 5](https://claude.ai/artifact/1FTf3WsyH1vCjaSt8r7KVi) |
| [2026-10-14-12f](2026-10-14-12f/) | **Locked** 2026-10-14 ("approved") — 12f controls + 12f-a attributes: fader (filled bar, relative, multi-finger), gesture channel, Oops window, Sequence pool (executor style), sequence window A (vertical sheet + curve with handles) / B (horizontal timeline), the encoder bar (four encoders, Mode above Layer, Values / Effect layers as side-by-side vertical faders, smart feature-strip minis, GDTF wheel carousels, phone swipe), the attribute board (`<luma-attr-board>`: first dialog, By group / By fixture type, popover details, header menu, scrubber), Encoder bar tab, rules + store filters. Plain `.html` boards (artifact rounds), Google Fonts only | artifacts (private): controls [r7](https://claude.ai/artifact/TdYdAnbssZeFkM7ZJyv4Dj), encoder [r12](https://claude.ai/artifact/Gp94wDjVPirgZvV1CeV6yj), attributes [r1](https://claude.ai/artifact/8Y84Zmvm7Cm8b9B39mDrby) · [r7](https://claude.ai/artifact/Y5A1TwGoJeLBpvf21fyxkZ) |
| [2026-10-17-34g4](2026-10-17-34g4/) | **Draft, not locked** (34g4 patch hardening, decisions §34 "34g4 patch hardened") — the Patch setup dialog (Devices › DMX › Patch, one grid for Live / Draft / Import, Exit, ≡ layer menu, Duplicate, Metadata tab, Import MVR File / Xchange, review, placement pop-up, Export MVR), import option C, placement asks, scope rules, export layout, Luma layers vs the MVR tree. `patch-dialog.html` predates point 12 (it still draws Luma layers; the sidebar becomes the MVR tree grouped by tree / class / tag) | artifacts (private): [patch dialog](https://claude.ai/artifact/EQaJWVzigNDuB8gcN2rcpH) · [import C](https://claude.ai/artifact/5rJBCacNVhNGnv31eHoxVa) · [placement](https://claude.ai/artifact/JyLcJDSXZwWrTHMGZzvzgn) · [scope](https://claude.ai/artifact/VUeqbV55AdxwYBi2We4t2K) · [export](https://claude.ai/artifact/W7yrSWxwVcgxhCn4bWCS2M) · [one tree](https://claude.ai/artifact/5QtZypTaknPFWmo3weyiNV) |
| [2026-10-18-34g4](2026-10-18-34g4/) | **Round 2 locked** 2026-10-18 ("Lock it and build"; 34g4 part 3: Live, Draft, sidebar, ≡ menu, Duplicate, Metadata, Exit) — `patch-dialog.html` after point 12 and parts 1–2: the sidebar is the MVR tree (layers › nested groups, "here" marks), grouped by MVR tree / class / tag; Tags and MVR group columns; the build's live columns; group ≡ menu, Duplicate group, Metadata, Exit; lines in the decided syntax (`Claim Root.ShowData.Devices`). The import / export screens are still round 1 (parts 4–6). **`patch-lock.html` locked** 2026-10-18 ("approved"; the entry hardening): one Patch tile, the dialog opens Live, the bottom-right claim button — Live · "Hold to claim and edit" → Claimed · Draft (gold; a press leaves: Save / Discard / Cancel) → Claimed · Bob (greyed) —, the ✕ question while claimed, the lost holder's line; it replaces the round-2 page's Exit and claim chip button | artifact (private): [round 2](https://claude.ai/artifact/2xMJAtWRJYtfivTrrDTSPD) · [lock](https://claude.ai/artifact/Mv55W5N2hn261bjZ7rmZrD) |
| [2026-10-17-tags](2026-10-17-tags/) | **Round 2 locked** 2026-10-17 ("Go"; Tags hardening, decisions §12 "Tags hardened") — round 1 (Tags dialog, ≡ menu, assign dialog, Tags cell, tile pool, Delete; superseded in parts) and round 2 (approved direction: two-pane Tags pool window, EditSetting with Meta + Tags on any object, patch sidebar by tag with Direct / Deep, Tags cell, Delete) | artifacts (private): [round 1](https://claude.ai/artifact/DSudqD4nkvrwBVdU5PFtLK) · [round 2](https://claude.ai/artifact/WUhj1nGebwFfTinGt3GS8q) |
| [2026-10-18-34g4-r3](2026-10-18-34g4-r3/) | **Round 3** (34g4 patch hardening, round 3; decisions §34) — the Patch grid, Edit Patch entry, the DMX map and its cards / clone, address tools, renumber, insert fixtures, multipatch parts, the attribute map and its editing, the context menu, touch move, universe crossing, ghost wrap, the pixel mapper, the edit tool; its [README](2026-10-18-34g4-r3/README.md) says what each page asks and what was decided | plain `.html` pages |
| [2026-10-19-mvr-import](2026-10-19-mvr-import/) | **Import MVR and drives** (34g4 part 4, decisions §38 "Drives and the one files element") — the drives round (drive names, lines, service, the root library) and the files element (desk, phone), then the import dialog, its options, line, placement and review; see its [README](2026-10-19-mvr-import/README.md) | plain `.html` pages |
| [2026-10-19-phone](2026-10-19-phone/) | **Phone hardening** (decisions §37) — one page per problem from the phone audit: the Patch dialog, landscape, labels, Insert, the map, the slot grid, the context sheet; see its [README](2026-10-19-phone/README.md) | plain `.html` pages |
| [2026-10-20-demo-rig](2026-10-20-demo-rig/) | **The demo rig, nested fixtures and second ids** (decisions §40) — seventeen questions, one page each (GDTF parsing, which rig, the demo data, nested fixtures, CIDs, the tree index, the id shape, the keyword, untyped CIDs, the folder word / line / values, groups, shows, the commit, the manual demo, names); see its [README](2026-10-20-demo-rig/README.md) | plain `.html` pages |
| [2026-10-20-home-mark](2026-10-20-home-mark/) | **The settings home mark** (§30 "12d": every settings cell says where its value lives) — the home icons and the mark; see its [README](2026-10-20-home-mark/README.md) | plain `.html` pages |
| [2026-10-20-patch-tree](2026-10-20-patch-tree/) | **The patch tree** (40d, decisions §40 "40d hardened") — the sidebar, the mix, sorting, the lab; see its [README](2026-10-20-patch-tree/README.md) | plain `.html` pages |
| [2026-10-22-device-sheet](2026-10-22-device-sheet/) | **Approved** 2026-10-22 ("all looks good, implement"; records 0283, 0284) — round 1 of the device sheet — `<luma-device-grid>` and the value marks, the window on the rig; see its [README](2026-10-22-device-sheet/README.md) | plain `.html` pages |
| [2026-10-22-top-bar](2026-10-22-top-bar/) | **Approved** 2026-10-22 ("all looks good implement"; record 0285) — round 1 (the top bar: the A1 S1 status strip, the modes from the sidebar, the status mark) — see its [README](2026-10-22-top-bar/README.md) | plain `.html` pages |
| [2026-10-22-view-bar](2026-10-22-view-bar/) | **Approved** 2026-10-22 ("yes this is the way for the future, lets implement this"; record 0287) — round 1 (the right sidebar: why stored view buttons don't show, a View pool window, any object on a view button) — see its [README](2026-10-22-view-bar/README.md) | plain `.html` pages |
| [2026-10-23-presets](2026-10-23-presets/) | **Approved** 2026-10-23 ("looks good"; record 0288) — round 1: one preset pool that follows the programmer's feature group (one flat pool, tags, live-preview cells); five questions. See its [README](2026-10-23-presets/README.md) | plain `.html` pages |
| [2026-10-23-meow-consent](2026-10-23-meow-consent/) | **Approved** 2026-10-23 ("Meow consent A is good"; record 0290) — round 1: the Meow License consent on every login, one station dialog, buttons A (one gold button + a quiet "Not me"); five questions. See its [README](2026-10-23-meow-consent/README.md) | plain `.html` pages |
| [2026-10-23-logo](2026-10-23-logo/) | **Approved** 2026-10-23 — round 1: the LumaCore mark and README hero; the cat-head prompt mark is the main mark, the moon mark is kept. See its [README](2026-10-23-logo/README.md) | plain `.html` pages |

## What a version folder holds

- `*.dc.html` — the boards' exact sources from the Design canvas (`<x-dc>` template + a small
  `Component` class). Open any of them **straight in a browser**: `support.js` beside them is a
  small offline runtime for the format, so the interactive parts (swipe button, toggle, fader,
  keypad, window menu, theme sliders, lock / switch user) work from the repo.
- `canvas.json` — the canvas layout (board positions and sizes, row titles).
- `lockbg.jpg` — the built-in lock-screen background (from the user's console concept in
  `inspiration/`, darkened and blurred).
- `png/` — a snapshot of every board, made with `node docs/dev/ui-kit/mockups/snapshot.mjs
  [version]` (uses the Playwright install of `tests/e2e`).

To edit on a canvas again, publish the folder's `canvas.json` and `*.dc.html` to a Design canvas
(the `/_blob/…` image URL replaces `./lockbg.jpg` there).

## Boards (2026-10-12)

| Board | Shows | Part |
|---|---|---|
| [Main](2026-10-12/Main.dc.html) · [png](2026-10-12/png/Main.png) | Palette (14 colours, Dark / Daylight / Night), sizes, type, icons | 12a |
| [Elements](2026-10-12/Elements.dc.html) · [png](2026-10-12/png/Elements.png) | Buttons, hold ring, swipe button, toggle, segmented, property cells, fader, encoder pop-up, pick list, list, toast, progress | 12a–12f |
| [Keyboards](2026-10-12/Keyboards.dc.html) · [png](2026-10-12/png/Keyboards.png) | Keypad / calculator, built-in keyboard, `Keyboard: Auto / OS / Built-in / Off` | 12a3 (keyboard), 12b (keypad, setting) |
| [Chrome](2026-10-12/Chrome.dc.html) · [png](2026-10-12/png/Chrome.png) | Gold tab dots, tap the dot → window menu, window bar (checkboxes, drop-downs, `…`), auto-hidden bar | 12a, 12c |
| [PoolHold](2026-10-12/PoolHold.dc.html) · [png](2026-10-12/png/PoolHold.png) | Hold on a pool object → full-screen selector with the action row | 12e |
| [Tablet](2026-10-12/Tablet.dc.html) · [png](2026-10-12/png/Tablet.png) | 1024×768 layout with faders | 12e |
| [Phone](2026-10-12/Phone.dc.html) · [png](2026-10-12/png/Phone.png) | 390×844, command line bar above the OS keyboard | 12e |
| [Dialogs](2026-10-12/Dialogs.dc.html) · [png](2026-10-12/png/Dialogs.png) | One standard dialog: Shutdown countdown, rejoin merge, Label, claim ask | 12b |
| [Settings](2026-10-12/Settings.dc.html) · [png](2026-10-12/png/Settings.png) | Window settings shell (tabs, pins, Window bar tab, presets), EditSetting Show | 12c |
| [Network](2026-10-12/Network.dc.html) · [png](2026-10-12/png/Network.png) | Setup layout: tabs, service cells, function block, power toggle | 12d |
| [LumaSessions](2026-10-12/LumaSessions.dc.html) · [png](2026-10-12/png/LumaSessions.png) | Station cards, actions follow the selection | 12d |
| [Theme](2026-10-12/Theme.dc.html) · [png](2026-10-12/png/Theme.png) | REAPER-style editor: adjusters, 14 swatches, pick on screen, A/B, hold for the normal theme | 12g |
| [Lock](2026-10-12/Lock.dc.html) · [png](2026-10-12/png/Lock.png) | One surface for lock, login and connect: clock, station, avatars, switch user, `Servers…`, status line | 12a3 |
| [LockPhone](2026-10-12/LockPhone.dc.html) · [png](2026-10-12/png/LockPhone.png) | The same on a phone | 12a3 |

The look follows the user's console concept — see
[inspiration-style.md](../../../../docs/research/ui-kit/inspiration-style.md).

## Boards (2026-10-12-12a3)

Added for step 12a3 ([§30 "12a3 hardening"](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)); the earlier Lock / LockPhone / Keyboards / Dialogs boards still hold.

| Board | Shows |
|---|---|
| [Connect](2026-10-12-12a3/Connect.dc.html) · [png](2026-10-12-12a3/png/Connect.png) | The app's bundled connect page: countdown, servers (wrong version, local server, Clean start, address, auto-connect, Kiosk), busy |
| [Surface](2026-10-12-12a3/Surface.dc.html) · [png](2026-10-12-12a3/png/Surface.png) | Station surface states: nobody picked, Admin password + wrong, built-in keyboard (password mode), locked with "2 waiting", countdown; avatars with the user's colour + profile name, remote-login-off dimmed |
| [Screens](2026-10-12-12a3/Screens.dc.html) · [png](2026-10-12-12a3/png/Screens.png) | A locked device with two screens (lowest full, others blurred), Lock tile on the touch sidebar, Shutdown menu (Meow+F11) with Lock |
| [Prompts](2026-10-12-12a3/Prompts.dc.html) · [png](2026-10-12-12a3/png/Prompts.png) | The standard dialog on the connect page: Clean start, another version, hung stop |
| [Keys](2026-10-12-12a3/Keys.dc.html) · [png](2026-10-12-12a3/png/Keys.png) | Core keys F9–F12 (F1–F8 free), console alignment |
| [Overview](2026-10-12-12a3/Overview.dc.html) · [png](2026-10-12-12a3/png/Overview.png) | Screens overview (displays, bring forward, open / hide, send to display, native full screen) and tiled mode |

## Boards (2026-10-12-12b)

Added for step 12b ([§30 "12b hardening"](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)); the Dialogs, Elements and Keyboards boards of 2026-10-12 still hold. The Copy dialog's `/NoName` toggle and data-pool input only show the input kinds.

| Board | Shows |
|---|---|
| [Asks](2026-10-12-12b/Asks.dc.html) · [png](2026-10-12-12b/png/Asks.png) | Every ask on the standard dialog: Store over existing, Unclaim Patch, Oops confirm, Import profile, critical feedback, answered elsewhere, locked with "3 waiting" |
| [Inputs](2026-10-12-12b/Inputs.dc.html) · [png](2026-10-12-12b/png/Inputs.png) | A dialog with number / link / toggle / choice inputs (answers into the button's template); the keypad with old value, relative input and the unit swipe |
| [PickList](2026-10-12-12b/PickList.dc.html) · [png](2026-10-12-12b/png/PickList.png) | Pick list: filter, required (no None) and optional links, the missing-link row, Add New, which settings get it |
| [VK](2026-10-12-12b/VK.dc.html) · [png](2026-10-12-12b/png/VK.png) | VK button on the command line bar (tap show / hide, hold = Auto / OS / Built-in / Off per device and screen), the built-in keyboard typing into the command line |
| [Terminal](2026-10-12-12b/Terminal.dc.html) · [png](2026-10-12-12b/png/Terminal.png) | The dialog as a centered TUI window over the dimmed shell, in the user's profile theme (Dark / Daylight / Night) |

## Boards (2026-10-12-12b2)

Added for 12b2 ([§30 "12b2 hardening"](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)). The WindowSettings board replaces the 2026-10-12 Settings board's window part (its Window bar tab stays for 12c).

| Board | Shows |
|---|---|
| [Menu](2026-10-12-12b2/Menu.dc.html) · [png](2026-10-12-12b2/png/Menu.png) | Menu tiles with a Lucide icon above the label, the new Users & Profiles tile next to LumaSessions, unbuilt tiles dimmed |
| [UsersProfiles](2026-10-12-12b2/UsersProfiles.dc.html) · [png](2026-10-12-12b2/png/UsersProfiles.png) | Floating Users & Profiles window: users + profiles list, profile tabs (Look · Lock screen · Input & keys · Oops · Command line), user cells, command buttons |
| [WindowSettings](2026-10-12-12b2/WindowSettings.dc.html) · [png](2026-10-12-12b2/png/WindowSettings.png) · [pick](2026-10-12-12b2/png/WindowSettings-pick.png) | One settings shell, tabs from sections (strip hidden at one), DataPool pick list with Selected DataPool, Font size Default, Columns Fit, universe pick list |
| [FullScreen](2026-10-12-12b2/FullScreen.dc.html) · [png](2026-10-12-12b2/png/FullScreen.png) | F12: every bar hidden, the corner handle (touch / desktop faded), typing floats a command line in |
| [Fixes](2026-10-12-12b2/Fixes.dc.html) · [png](2026-10-12-12b2/png/Fixes.png) | Servers list dots (green / amber / grey / gone), Meow held vs latched, the command line bar ground, the macOS F-key hint, where text can be selected |
| [SysmonList](2026-10-12-12b2/SysmonList.dc.html) · [png](2026-10-12-12b2/png/SysmonList.png) | System Monitor: history in one batch, a virtual list, paused / live |
| [Realtime](2026-10-12-12b2/Realtime.dc.html) · [png](2026-10-12-12b2/png/Realtime.png) | Every render frame: numbers + 10 s graphs per stage, budget line, a late frame, a gap; the feed messages |
| [Suggest](2026-10-12-12b2/Suggest.dc.html) · [bar](2026-10-12-12b2/png/Suggest-bar.png) · [palette](2026-10-12-12b2/png/Suggest-palette.png) | `luma-suggest`: the list above the bottom bar with ghost text; the full-screen palette (sections, preview, Enter / Meow+Enter, Tab into an item) |

## Boards (2026-10-13-12c)

Added for 12c ([§30 "12c hardening"](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)). They replace the window-bar parts of the 2026-10-12 Chrome and Settings boards.

| Board | Shows |
|---|---|
| [WindowBar](2026-10-13-12c/WindowBar.dc.html) · [png](2026-10-13-12c/png/WindowBar.png) | System Monitor bar (Pause = `Self.AutoScroll`, Wrap, Level, Time, Clear = view-local, search chip), live titles on the tabs, DMX `Universe ‹ ›` = `Set Self Universe ± 1`, a narrow pool's `…` overflow, a short Realtime's hidden bar listed in the window menu; each line next to its resolved journal line |
| [BarSettings](2026-10-13-12c/BarSettings.dc.html) · [png](2026-10-13-12c/png/BarSettings.png) | Settings window: pin per cell, ↺ on changed cells, Load / Save preset, Reset window; Window bar tab (on / off, ordered items with drag / ↑ ↓ / ×, the type's actions, Load defaults); the four reset layers |
| [Presets](2026-10-13-12c/Presets.dc.html) · [png](2026-10-13-12c/png/Presets.png) | `WindowPreset` load (preselects where the window came from) and save (new / overwrite asks), the ask for a reset over several windows, the commands |
| [SelfExport](2026-10-13-12c/SelfExport.dc.html) · [png](2026-10-13-12c/png/SelfExport.png) | What `Self` is per line source (typed, bar, window menu, key, member → master, macro / plugin later), E-CMD-016 in a terminal; `Export|Import View / ScreenConfig / WindowPreset` JSON into `library/ui/<kind>/` (floor plan only) |

## Boards (2026-10-13-12c2)

Added for 12c2 ([§30 "12c2 hardening"](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)). They replace the 2026-10-12-12b2 Menu board's tile sizes.

| Board | Shows |
|---|---|
| [UserAsks](2026-10-13-12c2/UserAsks.dc.html) · [png](2026-10-13-12c2/png/UserAsks.png) | Station asks: New user (name, role, profile own copy of Default / share, password, colour), New profile (from Default / copy of / import), Delete user (what goes, logged out, claims released, delete own profile too), Delete profile, the coded refusals |
| [MenuTiles](2026-10-13-12c2/MenuTiles.dc.html) · [png](2026-10-13-12c2/png/MenuTiles.png) | The kit tile grid: compact tiles (icon 24, one-line label), auto-fill columns by box width, phone 2 columns of icon + label rows |
| [PhoneFixes](2026-10-13-12c2/PhoneFixes.dc.html) · [png](2026-10-13-12c2/png/PhoneFixes.png) | Phone: PURR · line · VK · ⋯ (status in ⋯), suggestions within the screen; settings window title + × kept, actions in ⋯, tabs scroll; Users & Profiles in one column (Users / Profiles switch, locked Admin / Default) |
| [HoldFullscreen](2026-10-13-12c2/HoldFullscreen.dc.html) · [png](2026-10-13-12c2/png/HoldFullscreen.png) | The desk's view on a phone: same panes in place, miniature, < 44 px not tappable, hold a pane = full screen (‹ › ×), the desk follows; portrait / landscape |
| [ResponsiveViews](2026-10-13-12c2/ResponsiveViews.dc.html) · [png](2026-10-13-12c2/png/ResponsiveViews.png) | Record only: options A–E (collapse, per-size layouts, fold to fit, own config per device, scale) — not taken |

## Boards (2026-10-13-12d)

Added for 12d ([§30 "12d hardening"](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)). They replace the 2026-10-12 Network and LumaSessions boards. SetupSessions, SetupBackup and SetupSettings share one board engine, so the grid, slab, edit card and asks behave the same.

| Board | Shows |
|---|---|
| [SetupDmx](2026-10-13-12d/SetupDmx.dc.html) · [png](2026-10-13-12d/png/SetupDmx.png) | `<luma-grid>`: cells / rows / columns, latched + / −, Thru, All / None / Invert, Edit chip, the edit card (tabs, ‹different values›, one line), new-object line, filter row, the tools slab with icons |
| [SetupNetwork](2026-10-13-12d/SetupNetwork.dc.html) · [png](2026-10-13-12d/png/SetupNetwork.png) | Network › Interfaces on the grid (services as cells, one line for a column on many NICs), hardware rows (New / Cut / Delete greyed), power toggle A (Online / Offline / failing) |
| [SetupUsers](2026-10-13-12d/SetupUsers.dc.html) · [png](2026-10-13-12d/png/SetupUsers.png) | Users & Profiles on grids + slab, desktop and phone sharing one selection, safe delete ask |
| [SetupSessions](2026-10-13-12d/SetupSessions.dc.html) · [png](2026-10-13-12d/png/SetupSessions.png) | LumaSessions: Stations grid / cards, multi-station edit, hand master over, Known, This station |
| [SetupBackup](2026-10-13-12d/SetupBackup.dc.html) · [png](2026-10-13-12d/png/SetupBackup.png) | Backup: Library, Show · History (List / object Matrix), Recovery, the Load show dialog (parts / single objects), coloured show actions |
| [SetupSettings](2026-10-13-12d/SetupSettings.dc.html) · [png](2026-10-13-12d/png/SetupSettings.png) | Settings: Station · Show · Assets · About, kinds (Aa / 123 / I/O / ≡), home marks, rights levels |
| [PowerToggle](2026-10-13-12d/PowerToggle.dc.html) · [png](2026-10-13-12d/png/PowerToggle.png) | The power toggle looks A–D; **A** approved |

## Boards (2026-10-13-12e)

Each round is a whole artifact; earlier rounds keep the variants that were not taken (the record). What
was approved is in [§30](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)
"12e hardening" → "12e round 5 questions".

| Board | Shows | Approved |
|---|---|---|
| [Round1-CommandLine](2026-10-13-12e/Round1-CommandLine.html) · [png](2026-10-13-12e/png/CommandLine.png) | Narrow command line: today vs 12e (prompt shrinks to `>`, scrolls to the cursor, ‹ › marks); round 1's pool selector A/B and band handle (not taken) | Command line tab |
| [Round2-PoolSplit](2026-10-13-12e/Round2-PoolSplit.html) · [png pool](2026-10-13-12e/png/Pool.png) · [png split](2026-10-13-12e/png/Split.png) | `<luma-pool>`: hold → object sheet, Move / Copy target pick, NodeMeta / pool-type colours, desk, phone full screen, 30 × 30 miniature (names from 40 px); split handle: grip pill is the touch target | Pool, Split |
| [Round3-PhoneTools](2026-10-13-12e/Round3-PhoneTools.html) · [png phone](2026-10-13-12e/png/Phone.png) · [png tools](2026-10-13-12e/png/GroupTools.png) | Phone desk (folded sidebar, hold → full screen, swipe stop); group editor: grip + ↑ ↓, Tools → grouped bottom toolbar (desk, stays open) / low sheet (phone), Lucide icons on a fixed grid, yellow `===` mark on buttons that open a list | Swipe stop, tools, icons, mark |
| [Round4-MiniGridBench](2026-10-13-12e/Round4-MiniGridBench.html) · [png](2026-10-13-12e/png/MiniGridBench.png) | Miniature `<luma-grid>` A scaled / B dense text / C visual summary / D hybrid | B for sequences, D for the fixture sheet |
| [Round5-MiniSheets](2026-10-13-12e/Round5-MiniSheets.html) · [png](2026-10-13-12e/png/MiniSheets.png) | The two refined miniatures on a bench and on the phone desk | Both |

## Boards (2026-10-14-12f)

Each round is a whole artifact; earlier rounds keep the variants that were not taken (the record). What
was approved is in [§30](../../../archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening)
"12f hardening" → "12f / 12f-a design locked".

| Board | Shows | Approved |
|---|---|---|
| [Round01–07-Controls](2026-10-14-12f/Round07-Controls.html) · [png](2026-10-14-12f/png/Round07-Controls.png) | Faders (look B, relative, second finger fine), the gesture channel logs, encoder (superseded by rounds 8–12), Oops window, Sequence pool (executor style, muted gold active cue, blue run bar, hold = Edit), sequence window A (vertical sheet, feature-group curves, Path list, draggable handles, stable 2 s+ scale) and B (horizontal timeline) | Faders, Oops window, Sequence pool, sequence window A + B (round 7) |
| [Round08-EncoderConcepts](2026-10-14-12f/Round08-EncoderConcepts.html) | Encoder bar concepts A tabs + cells / B smart cells / C chips → slider bank; sheet columns Compact / Full / Minimal | A (with B's smart bits); Full columns |
| [Round09–11](2026-10-14-12f/Round11-EncoderFaders.html) · [png](2026-10-14-12f/png/Round11-EncoderFaders.png) | Four encoders (Dynamic Presets layout), Layer sets, then vertical faders side by side with the chosen one widened, smart feature-strip minis, harder programmer red, phone swipe, tap-outside + PopupDim | Round 11 layout |
| [Round12-EncoderModes](2026-10-14-12f/Round12-EncoderModes.html) · [png](2026-10-14-12f/png/Round12-EncoderModes.png) | Mode above Layer: Attributes, Cue timing vertical / horizontal, Masters, Effects; suggested Pawtern, Selection, Executors, Stage | Approved ("basic concept of our encoderbar") |
| [Attr-Round01](2026-10-14-12f/Attr-Round01.html) · [png](2026-10-14-12f/png/Attr-Round01.png) | Patch / profile new-attribute table (replaced by the board), **Encoder bar tab**, **store filters + resolver rules** | Encoder bar tab; rules + filters (filters built later) |
| [Attr-Round02](2026-10-14-12f/Attr-Round02.html) | New-attribute variations A tray / B inbox / C one at a time / D board | D |
| [Attr-Round03–07](2026-10-14-12f/Attr-Round07.html) · [png](2026-10-14-12f/png/Attr-Round07.png) | The attribute board hardened: details presentations (popover taken), header menu, sub-attributes follow, + group, Unused, By fixture type mini boards, destination chips, first dialogs (patcher / other users), pick list ×/Cancel, phone scrubber | Round 7 |


## Boards (2026-10-14-users-profiles)

Users & Profiles follows storage (decisions §30 "Users & Profiles follows storage"), locked 2026-10-14.

| File | What | Picked |
|------|------|--------|
| `Round01-Structure.html` ([artifact](https://claude.ai/artifact/6GdXA1Wecmpwho9sFA4Fxo)) | Structure A / B / C, how a detail opens, part tab sets; desk + phone | B (grid, then detail) · drill in · Settings / Content |
| `Round02-Phone.html` ([artifact](https://claude.ai/artifact/HLdwfLdZHvSsvnozE3RsnR)) | Three phone layouts: P1 a screen per level, P2 bottom bar + part picker, P3 one scrolling page | **P1** (approved) |
