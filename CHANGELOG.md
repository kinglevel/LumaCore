# Changelog

All notable changes to LumaCore, newest first. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); one entry per step or part, with its
commit. Numbers in old lines (format and protocol numbers) are history — the code owns the
current values. Decisions are in `docs/decisions/`; the frozen log is
[docs/archive/decisions-2026-10.md](docs/archive/decisions-2026-10.md) (`§` links below).

## [Unreleased]

### Added
- Keys per shortcut mode and press cycles (decisions 0295): a binding belongs to All, KBD, PRG or NAV (`keybindings::Layer`; ``Bind "E" `Edit` `EditSetting` /Shortcuts "PRG"``, `Unbind … /Shortcuts`, Bind's `/Description` declared at last); a mode's binding wins over an All one, yours over LumaCore's, and Unbind with none of yours turns LumaCore's key off on your profile (again: back). A letter, digit, sign or Space without Ctrl / Alt types in KBD and binds only in PRG / NAV (E-KEY-006). A UiKey's Press can be a press cycle and some presses type onto the line (`cycle`, `types`, Meow alike; lum.md: lines in a row are one value, `Value::Lines`); the next press of a key replaces what it typed while the line still ends with it, no timer; a running press completes the line (`Fixture 1` + `. .` → `Fixture 1 Zero`). PRG's keys are LumaCore's rows (`keybindings::PRG_KEYS`: E Edit › EditSetting, Alt+A Assign › Label, Ctrl+C Copy › Paste › Insert, Delete ResetSelection › ResetActive › ResetAll, … ) — the terminal's letter table and its `aa` timer are gone; inside an open quote every key types; Space and the mode letters act only on an empty line. A profile key's lines are its properties: `Set UIKey "<id>" "Press" | "Types" | "Hold" | … | "MeowPress" …` (a locked key: E-KEY-007). `List KeyBindings` shows the screen's mode. The show format bumped (an empty step). ([0295](docs/decisions/0295-keys-in-shortcut-layers-and-press-cycles.md), [mockups](docs/dev/ui-kit/mockups/2026-10-23-profile-editor/03-cycles.html))
- Object paint (decisions 0294): a Paint is `#rrggbb`, a palette word (Red … Magenta, Black, White — stored as hex) or a link to a theme paint slot, `Set Group 3 "Paint" Theme "Position"` (a syntax addition: `named-one = OBJECT QUOTED` as a value, in lum.md); `none` clears. One parser, `lumacore_base::Paint::parse` (tags, feature groups, every object; `paint::hex_rgb` the one hex reading, window colour settings too); a wrong value is E-SET-003. A profile's theme has paint slots beside its 14 screen colours — LumaCore's Dimmer, Position, Gobo, Color, Beam, Focus, Shutter, FX, Control, Shapers, Video, Special 1–4 and its own: `Set … Theme Paint "<slot>"`, `Store … Theme Paint "<slot>" ["#rrggbb"]`, `Reset …`. Feature groups follow the slot of their name (moved off the old `Color` word onto the one Paint property). The colour an object shows is resolved per viewer on the station (`lumacore_state::paint::Painter`): its own Paint → the painted tag with the highest `Priority` (a new Tag property; ties by tree order; a tag without Paint takes its parent's) → the pool's colour; a link to a slot the viewer's theme lacks is kept, named and falls back. Pools, the patch grid and the Tags members (rows tinted), the Tree window and NAV (a swatch; the TUI a coloured mark), the inspector's "Colour from", the Device Sheet's name, the view bar, the cue sheet, the encoder bar, the attribute board and the preset pool read it; "Colour from" is a list column. Every show object with a NodeMeta takes a Paint (cues, macros, views, executors, effects, DataPools, profiles, images joined). `<luma-paint>` (`kit/paint.js`, in the Showroom) picks: Palette (12 hues × 4 tones + greys + none, from the station), Free, Theme (your slots), recent colours and the mode kept on the profile (`PaintMode`), Examples, the hex field, the line; from any Paint settings cell and the grid's edit card (painting many is one line); the old Theme button that sent a word Paint refused is gone. The theme editor's Paint tab (the slots, Add a slot, Reset to LumaCore). The Rules window (window type 16): the AttributeRules in order, drag to reorder (`Move AttributeRule`), the pattern and group cells (`Set AttributeRule n "Pattern" | "FeatureGroup"`, new properties; a change makes a rule learned), Delete, Which rule? marking the rule that places a name. Two feeds, `paint` and `rules`; the show format bumped with its step (bytes → hex, a feature group on its template colour → its slot); `SESSION_PROTOCOL` and `CLIENT_PROTOCOL` bumped. `kit-paint.spec.ts`, `kit-rules.spec.ts` ([0294](docs/decisions/0294-object-paint-palette-free-theme-links.md), [mockups](docs/dev/ui-kit/mockups/2026-10-23-paint/README.md))
- Presets link into cues (decisions 0293): a cue value is plain steps or a preset link (`CueData`); a value called from a preset is stored as the link only and plays the preset as it is now — the plan compiler resolves it each compile through `PresetIndex` (the fixture's own entry, else the universal one; none = no value), so a changed preset needs no cache invalidation and nothing is copied; tracking and blocked values compare links as links. The programmer keeps each value's home (`ProgValue.origin`) when a plain value replaces it. `Update` alone is one station ask holding the kit's tree in pick mode (`Dialog.tree`: the presets your values came from with the cues that follow them — untick one and it keeps its old look by an Extract first — and the cues with plain values you changed; Content OnlyExisting / AddNew / Overwrite (→ `/Merge` / `/Add` / `/Overwrite`), the Recast and Protect switches kept on the profile as `UpdateContent` / `UpdateRecast` / `UpdateShield`; the lines it will run, redrawn as you tick on the page and in the TUI); its lines run as ``Update `…` `` — one statement, one Oops step. New lines: `Update Preset <ids> /Merge|/Add|/Overwrite`, `Update Sequence <m> [Cue <n>] /Original|/Active [/Shield]`, `Recast Preset <ids> [Sequence <m> [Cue <n>]]` (asks first, + / − per cue), `Extract Preset <ids> [Sequence … [Cue …]]`, `Extract Sequence <m> [Cue <n>]`; deleting a preset that cues use asks with the cues and keeps their values (Oops brings the preset and every link back). The Presets pool counts the cues that follow a preset; an inspector's Used by names the cue (`Sequence 1 Cue 3 "Split"`, and a pool object's GUID reaches the inspector now). The Device Sheet's `Show` (Value / Link / Both, default Both): the `devices` feed sends the preset per linked cell, the text `P3 Split · 85`, the preset bar either way, the hover names both. Preview in Blind is greyed as planned (no Blind output exists). The rig's Sequence 1 "Presets" links Presets 1–3. Show format bumped with its step (a cue value that carried its preset keeps only the link); `CLIENT_PROTOCOL` bumped. `kit-update.spec.ts` ([0293](docs/decisions/0293-presets-link-into-cues-update-recast-extract.md), [mockups](docs/dev/ui-kit/mockups/2026-10-23-preset-links/README.md))
- The Tree window (decisions 0292): window type 15 — NAV's tree and inspector in the web and the app. `<luma-tree-window>` (`kit/treewindow.js`) draws `<luma-pathbar>` (new in the kit and the Showroom: crumbs, or a field for a tree path or a GUID — whole or its first 8 digits; a gone GUID is a toast), `<luma-tree>` in its new host mode (the station's rows, drawn virtual, fold / search as events) and `<luma-inspector>` (identity with copy buttons through `kit/clipboard.js`, facts, every property as a `<luma-settings>` cell — a change is its `Set` line, ↺ its `Reset`, values flash when they change — Used by / Links to that jump). Two feeds, `tree` and `inspector`, from `lumacore_views::tree_window` (the same `tree_rows` and `inspector::view` as `ls` and NAV; `inspector::Row.field` carries the cell), built on the state thread as the user. Its settings: Follow None / Selection (the object your newest line named, a fixture meaning the selection's last; My NAV planned), Show All / Changed only, Split, and its view state Path / Open / Search as `Set Self` lines. The row menu is the registry's one context menu plus Copy GUID / path / as target / Set line and Open in the terminal's NAV (`cd` + `Shortcuts Navigator`; `openContext` takes the host's `extra` entries); Bookmark… is greyed until a word is decided. A phone draws the tree, a tap opens the inspector as the one sheet. NAV's filter and GUID lookup moved to `tree_window` (`matches`, `path_of_guid`). `kit-tree-window.spec.ts` ([0292](docs/decisions/0292-tree-window-nav-in-the-web-and-app.md), [mockup](docs/dev/ui-kit/mockups/2026-10-23-tree-window/README.md))
- NAV mode is `ls`'s rows and an inspector (decisions 0291): one row builder, `lumacore_objects::tree_rows`, for `ls`, NAV and the inspector — a row's path is its number or declared name, never its label, so every object opens (`DataPool 1` did not). NAV draws the tree on the left (only the rows that fit; digits jump to a slot, `/` filters with a hidden count, `n` / `N`) and the inspector on the right, following the cursor, built by `lumacore_views::inspector` from the registry `EditSetting` reads: Slot, Kind, GUID, Path, facts (a fixture's Type / Mode / Location, a datapool's pools), every property with its kind and `Set` line, ● and `r` (`Reset`) where it differs from its declared default, `v` changed only, Used by / Links to (Enter follows, red "missing" for a gone GUID). Copy keys (`y`, `yg yp ya yn ys`) through the clipboard door; Space marks rows into one range line; `m` / `'` bookmarks by GUID on `ShowData.Users.{n}.Bookmarks` (`ShowSubtree::Bookmarks`, never undone); `p` pins; `c` cd; `?` help from `NAV_KEYS`; the gesture log (`L`, the profile's `NavLog`); values that change under the inspector flash. Fixed on the way: `ls` drew `└──` above the row after it, and a live entry shows its text (a master's level, a claim); `Info DataPool` is built (its pools and counts; a datapool's pools count their own objects in `ls`); `Info Fixture`'s channel table lines up; `cat` / `head` / `tail` / `stat` / `tree` / `find` / `grep` / `du` / `wc` / `diff` take a path the way `cd` does (`cat 1` in ShowData is Devices under its name; a name that isn't there is E-SET-004). The show format bumped (bookmarks, `NavLog`; an empty step). `copy.spec.ts` presses `yp` in the page's terminal ([0291](docs/decisions/0291-nav-mode-ls-rows-and-an-inspector.md), [mockups](docs/dev/ui-kit/mockups/2026-10-23-nav-mode/README.md))
- Copy to the user's clipboard on every client (the NAV inspector's copy keys, [mockup](docs/dev/ui-kit/mockups/2026-10-23-nav-mode/02-inspector.html)): one door in the TUI, `repl::clipboard::copy`, queues the `CopyText` UI event (`CLIENT_PROTOCOL` bumped); `lumacore-cli` writes OSC 52 to its terminal (over SSH too) and the desktop clipboard when it has one, the station's own console OSC 52; the page (`kit/clipboard.js`) the browser's clipboard, the older copy command, or a toast holding the text selected for Ctrl+C (`toast(…, { hold })`, in the Showroom); the desktop app takes it over its IPC (`copy`) and writes the OS clipboard natively (`arboard`). Separate from the station's object clipboard (`Copy` / `Paste`). `copy.spec.ts` ([http-api.md](docs/dev/api/http-api.md#copy-to-the-users-clipboard))
- The Meow License on every login (decisions 0290): `POST /api/login` (the page, the app, autologin, `?autologin=now`), a shell's user + password hello, `Login` on any terminal and a user switch wait for the user's consent — one station dialog (title, the good-vibes text, the vibe-coded warning as an alert line, "I consent — let the meow in" / "Not me", the fine print; the text held once in `lumacore-state` `consent`), drawn by the kit's dialog over the surface, the TUI and the CLI. Its buttons are lines: the new word `Consent` finishes the login (the station log: "<user> consented to the Meow License"), `Logout` drops it. Until then the users family's gate refuses every other line (E-USER-010) and the login's token counts as no login over HTTP (`/api/me` answers `consent: true`); a DeskLock unlock never asks. Station-local, never saved; no wire change (the existing `Dialog` with topic `consent`). The page waits on the shell before it loads the desk or tells the app; the shell's URL carries the token too. lum.md and the reference gain `Consent`; `login-sysmon.spec.ts` drives the flow; the e2e support consents for its logins (`server.login`, `Cli.connect`, `connected`, `cmd`, `promptUser`); `readme-shots.spec.ts` captures `consent.png` ([0290](docs/decisions/0290-meow-license-consent-on-every-login.md), [mockup](docs/dev/ui-kit/mockups/2026-10-23-meow-consent/README.md))
- Older show files open upgraded (decisions 0289, superseding 0082): a show from `OLDEST_FORMAT` up is read whole and upgraded one step per format bump on its JSON (`show_file::upgrade`: 31 → 32 the theme's Tint gives way, 32 → 33 view buttons per screen, 33 → 34 the one preset pool), opens as unsaved changes and says from which format (the reply, the System Monitor); its file stays until a save keeps it as a version. Profile files, `Import Show`, versions / recovery copies and the `library/ui` files read through the same door; the Load show dialog says "upgrades on load". `LoadShow "demo"` builds the demo from its sources when it is stale, as a start with it does. A test fails when a format has no step; the rig is taken back to format 31 and up again.
- The one preset pool (decisions 0288): a DataPool's presets are one numbered pool, `Preset <n>` (it was two-part; lum.md changed with it). `Store Preset [n]` keeps every active programmer value, Universal when each attribute holds one value for all its fixtures, else Selective; `/Merge`, `/Overwrite`, `/Remove`; Store alone takes the next free number. `At Preset n` puts the values on the selection (or the line's own target) and each keeps the preset's GUID; `Call Preset n` is written out as At; a bare `Preset n` selects its fixtures. Presets take tags (`Assign Tag … At Preset n`). Window type 14, Presets: follows the encoder bar's feature group (or pinned, or All), tag chips on top, Packed or Fixed slots, cells with the fit to the selection, the mode letter, a preview of the values, a square per feature group, the fixtures using it and your programmer's frame; tap = the window's action (At, SelFix At, SelFix, Call, None); Store + a tap on an empty cell stores there with the lit tags; hold → Edit values… opens the preset editor (`kit/preseteditor.js`). The ten Planned typed preset words, `PresetGroup` and the profile's `SelectedPresetGroup` are gone; a window's setting choices are Lum words now; the rig stores three presets and a tag; the show format and `CLIENT_PROTOCOL` bumped (`Tap.window`, the `pools` feed's `feature` / `tags`). `kit-presets.spec.ts` ([0288](docs/decisions/0288-one-preset-pool-follows-the-programmer.md), [mockup](docs/dev/ui-kit/mockups/2026-10-23-presets/README.md))
- The view bar holds any object, per screen: `ViewButton <screen>.<n>` (a bare number is the line's screen) holds a view, a group, a sequence or a macro by kind and GUID; a tap is the object's own line (`View 3`, `Sequence 4` = Select, `DataPool 2 Group 1`), written out before rights and Oops; an empty tap only says so; Store on an empty button stores this screen as a new view there; `Assign <Object> <n> At ViewButton <s>.<n>` from the command line, from Assign + a pool tap + a button tap, from the button's Assign… list, or by dragging a pool cell onto the bar. The right sidebar rides the `viewbar` feed (`lumacore_views::view_bar`) — the stored-button bug is gone (the bar compared view numbers with GUIDs, `/api/views` found the profile by the login's name) — as many buttons as fit, ▲ ▼ by set, the kind's colour and icon. `CLIENT_PROTOCOL` bumped. One pool window type per pool: Macros (12) and Views (13; each cell draws the view's panes, its screens and buttons; a Profile setting shows another profile's views, view only) beside Groups and Sequences; the show format bumped. `kit-viewbar.spec.ts` ([0287](docs/decisions/0287-view-bar-any-object-pool-windows.md), [mockup](docs/dev/ui-kit/mockups/2026-10-22-view-bar/README.md))
- The theme editor (step 12g): `Theme` opens `kit/themeeditor.js` from the ThemeBg board — the base, six adjusters with a contrast readout, the 14 colours, Pick on screen, A | B, Import / Export JSON, Reset all, a preview, Hold: normal theme; a drag previews, the release is one line. BackgroundHue / BackgroundSaturation replace Tint (both derivations, the props, the show format bumped, the look test recomputed from theme.js); the Showroom button, `kit-theme.spec.ts` ([0286](docs/decisions/0286-theme-editor-background-adjusters.md))
- The top bar is a status strip: `<luma-status-strip>` (`kit/statusstrip.js`) — the mark (a ring and the station's name: standalone / master, member with its latency, busy, reconnecting, lost, idle; a tap opens the station's details), the Show tab (a tap: `Menu "Backup"`), User / Profile / DataPool / Sequence / LumaSession, the modes as word indicators that are the UI keys themselves (Blind, Highlight, Freeze, Solo, Oops with its step count, Meow), the clock; the `--lc-mode-*` tokens in every theme; `Status.strip` (`CLIENT_PROTOCOL` bumped). The six keys left the left sidebar; the command line's ONLINE key and its mode and LumaSession icons went; the old "Admin · undefined" is gone. The e2e support reads `CLIENT_PROTOCOL` from `lumacore-wire` (its copy had fallen behind). `kit-topbar.spec.ts` ([0285](docs/decisions/0285-top-bar-status-strip.md), [mockups](docs/dev/ui-kit/mockups/2026-10-22-top-bar/README.md))
- The Device Sheet: window type 11 on `<luma-device-grid>` (a `<luma-grid>` subclass, `kit/devicegrid.js`) — every device in tree order with its folders as rows, the attributes in scope under their feature groups, Fixture and Channel modes, the Absolute / Fade / Delay / Output / DMX layers, Programmer only, Selected only, Selection first, Folders, Merge cells; each value coloured by where it comes from (your programmer active / not, new / lower / tracked / blocked in the selected sequence's cue, another sequence, park, fading) through the one value-marks module (`kit/marks.js`, the `--lc-v-*` tokens in every theme), also on the encoder bar; every gesture a line (`Fixture 785`, `- Fixture 785`, `Fixture 785 Attribute "Pan" At -45`); the sampled `devices` feed built by `lumacore_views::devices`, `CLIENT_PROTOCOL` bumped; `patch::tree_order` the one tree order (siblings of different parents no longer interleave); the Showroom section, `kit-devices.spec.ts` ([0283](docs/decisions/0283-device-grid-and-value-marks.md), [0284](docs/decisions/0284-device-sheet-window.md), [mockups](docs/dev/ui-kit/mockups/2026-10-22-device-sheet/README.md))
- The md upgrade closed on the code side: the docs in the tidy gate — `claude_md_stays_short` and `the_docs_pass_docs_check` (`crates/dev/lumacore-tidy/src/docs.rs`), the README ratchet lowered to the crates still without one ([0282](docs/decisions/0282-docs-checked-in-the-tidy-gate.md))

### Fixed
- A profile's Settings… opened only the theme editor, under the menu: the page sent any settings reply carrying the theme's Base to it. Now the whole profile opens in the settings window (Look's "Theme editor" opens it above), the theme editor only for a `Theme` line and above every menu; the drill-in offers Paint slots, Update and MVR import; a profile's Theme / Paint / LockScreen cells show the user home. Touch sizes for tree boxes and the Rules window's search. (5f6fc39c)

### Changed
- The md upgrade: `CLAUDE.md` cut to invariants, decision records under `docs/decisions/`, the log frozen as an archive, this changelog, the roadmap cut to Now / Next / Later, ideas under `visions/notes/` ([plan](docs/plans/active/md-upgrade.md))

## 2026-10-20

### Added
- §40 part 40d: the tree in the Patch dialog — nodes only in the sidebar, fixtures only in the pane, one lift onto a node, `Assign` / `Move` / `Delete /KeepChildren` / `List` / `Info` on a folder — `04ea21b3` · [§40]
- §40 part 40f: the demo is the rig — `demo.luma` built from `library/mvr` + `library/poolxml` + `library/demo/manual.lum`, `demo::ensure` the one door, the golden, every test on the rig — `fac1d483` · [§40]
- §40 part 40e: `formats/lumacore-poolxml` + `Import PoolXml "file" Groups [/Drive] [/DataPool]`, `scripts/unix/poolxml_groups` — `2d426400` · [§40]
- §40 part 40c: Import MVR with the tree — folders as nested fixtures, `MvrIdType` / `/IdType`, an id-less folder gets the next free second id — `cfccc3c6` · [§40]
- §40 part 40b: the id model — the patch keyed by a `Slot`, `fid` / `id_type` / `cid` on `Fixture.Mvr`, Generic = MVR 3, `order`, format 31 — `3d19607c` · [§40]
- §40 part 40a: the GDTF reader as the vendored `gdtf` fork with the spec's defaults, the FixtureID text fallback — `19579cdb` · [§40]
- §39: the settings home icons — three homes (show file · user · station) as icons on every cell, gold where it differs from the tab's — `884165dd` · [§39]
- 34g4 part 4c: the grid's review mode, the review slab, the Import cell's two-step dialog — `6d1e4547` · [§34]
- 34g4 part 4b: `Import Mvr {rows} /Apply`, a cell's `/Take` / `/Offer`, the placement asks per missing group, Keep refused while a review stands — `5ca55dca` · [§34]
- 34g4 part 4a: the MVR import review on the draft — the profile's preset properties, the `Mvr` object, `List` / `Info` / `/Cancel`, the `patch` feed's review — `62ec3662` · [§34]
- 38c-2: the `lumacore-mvr` format crate and the files feed's MVR columns — `c553d300` · [§38]
- 38c-1: the files element's phone rule — `075c5e3a` · [§38]
- Import MVR hardened 1–4 (the two-step dialog, the line and the preset, the review on the draft, the placement asks) — `d64fa522` `545a22f8` `4f48496a` `274b4971` · [§34]
- Decisions §39 and its mockups — `a6afce76` `996d5e17` `c8666de5`
- The md upgrade's plan and the 237 candidate records — `90832eb5` · [plan](docs/plans/active/md-upgrade.md)

### Changed
- Second ids decided as three ids (FID · IDType · CID, `Fixture Media 7003`) instead of tags; built with 40b · [§40]
- The change path's cost on the rig cut from 1.0 s to 82 ms a line (`lumacore_base::fingerprint`, sysmon snapshot as counts, four model crates at opt-level 1) — `fac1d483` · [§40]
- Roadmap: what the Patch tree leaves to do after 40d — `7e1fe90e`

### Removed
- `MoreIds` (replaced by `IDType` + `CID`, 40b) — `3d19607c`

## 2026-10-19

### Added
- 34g4 round 3 part 4d4: drag and drop + lasso in the DMX map on the pointer core's one `lift` gesture — `69a0009b` · [§34], [handoff](docs/plans/done/2026-10-19-4d4-hardening.md)
- Round 3 part 4d3b: Edit Patch — the slot zoom, `kit/editpatch.js`, the four ways in — `e8f5ac7d` · [§34]
- Round 3 part 4d3a: `Identify` — the word, the output layer above all but Park, the station's ends — `de89744a` · [§34]
- Round 3 part 4d2: the DMX map zoomed out — Universes List | Map, usage, the conflict list's fixes — `14c7c3c8` · [§34]
- Round 3 part 4d1: an overlap parks both — one `SlotMap` for the plan and the patch feed — `0c393b00` · [§34]
- Round 3 part 4c2: the Insert fixtures dialog (Choose + Place) — `3eb19b82` · [§34]
- Round 3 part 4c1: Insert fixtures' data — `/api/fixture-types`, `/api/fixture-type`, Upload `POST /api/gdtf` — `39a6a490` · [§34]
- Round 3 part 4b: the function block; Assign / Unassign At DMX are live — `fc2da0c2` · [§34]
- Round 3 part 4a: the Live / Draft patch grid — `b3f102bf` · [§34]
- Round 3 part 3c: the value dialog (renumber / address), `kit/value.js` — `f9c1f726` · [§34]
- Round 3 part 3b: the one context menu — `/api/object-menus`, `ContextDef`, `kit/context.js` — `debe05a7` · [§34]
- Round 3 part 3a: the claim button, the ring above dialogs, slab states — `77a1f7ce` · [§34]
- Round 3 part 2: `Import FixtureType`, the GDTF file index — `db5ee5c5` · [§34]
- Round 3 part 1c: the MVR node delete answers, Multipatch, Unassign At Tag — `44763ac1` · [§34]
- Round 3 part 1b: `Fixture n DMX`, `/Swap`, Move `/Skip` `/By`, the second id — `1658999d` · [§34]
- Round 3 part 1a: Store / Assign / Unassign At DMX, the DMX object — `6c969363` · [§34]
- Round 3 left-out items: a bare Multipatch is the next copy, Sort and Offset in the function block, the thumbnail in Details, member uploads reach the master, the violet dot — `59c78b56` `79317a7c` `d7479871` `fec073b2` · [§34]
- Identify / Probe / Find hardened: the word, `/As`, lum.md — `34c9897f` · [§34]
- RunTool / EditTool hardened: the `Tool` object, the edge line, Tab focus, the Esc ladder; mockup approved — `760f2f15` `41d705b4` · [§34]
- Phone hardening (§37): the audit, the phone rule `kit/phone.js`, the one sheet, decisions 1–7 built, `phone.spec.ts` — `50f3c159` · [§37]
- 38a: the `lumacore-drives` crate and the `Drive` object — `8f889eb8` · [§38]
- 38b-1: the `files` feed + `POST /api/upload` — `f8e274b0` · [§38]
- 38b-2: `<luma-files>` in the Showroom — `94936ca9` · [§38]
- 38b-3: Backup › Library on the drives rail — `02ba8b75` · [§38]
- 38b-4: the "which file?" ask is the element (`InputKind::File`), `/Drive` on every file word — `60befc86` · [§38]
- §38 Drives and the one files element, hardened — `66b998dc` · [§38]
- e2e in parallel: spec files one worker each, 23 min → 4 min — `8d2d45d3`
- e2e green: the 24 old failures fixed — `84ceb412`; the load-flaky tests settled — `ed6d2d10`
- CLAUDE.md: what to run when — `c88f2f32`

### Changed
- WebKit first in the e2e project order — `19c4a8d2`
- Edit Patch's function row buttons narrower — `495103ad`
- RunTool / EditTool after the round 3 patch window — `4d8f1852`

## 2026-10-18

### Added
- 34g4 patch part 1: the data — `Devices.Dmx.Patch`, `Fixture.Mvr` / `Fixture.Luma`, `Devices.Metadata.MVR`, format 30 — `fe0d2b3b` `9e4655ce` · [§34]
- 34g4 patch part 2a: the Devices claim and draft, live device edits — `cf4aa3db` · [§34]
- 34g4 patch part 2b: issue 11's universe questions in the Devices draft — `fcc58496` · [§11]
- 34g4 patch part 3a: the `patch` feed and the Patch menu — `cf6866cf` · [§34]
- 34g4 patch part 3b: the Patch dialog in the kit (Devices › DMX › Patch) — `dfbd73df` · [§34]
- 34g4 patch part 3c: the MVR tree's words (`MvrLayer` / `MvrGroup` / `MvrClass` / `MvrPosition`), one nesting rule, the ≡ menu — `79ca559a` · [§34]
- 34g4 patch entry: one Patch tile, the claim button, ✕ asks, a lost holder loses the draft — `bf3dc392` · [§34]
- §36 1–5: `lumacore-webui` (embedded, `--web-root` overrides), the app's OS code in `platform/` modules, `default-members` without the app, the bench row with server / CLI / app columns — `d72b33e2` · [§36]
- Build times: the git stamp in the station, `[profile.local]`, `jobs = -2`, build stamps, `build bench` — `c1276bfe`; `bench --remote` — `ab7fca5e` · [§34]
- Test speed part 1: systemtest 33 s → 2 s, `prune_objects` against rustc's leaking `.o` files, the Claude Code hook — `6fad945e` `93af75c2` · [handoff](docs/plans/active/2026-10-18-test-speed.md)
- The Root object and node numbers (§33 53–54) — `81d1d869` · [§33]
- 34g4 patch round 3: step 0's missing tests — `0cb27243`; the round's words hardened and in lum.md (item 19) — `c88f62b5` `cbdaa1d0`; the build cut into seven parts — `6e396a89` · [§34], [handoff](docs/plans/active/2026-10-18-patch-round3.md)
- Research: Map & clone and parts against real GDTF Share fixtures — `ce7fda3f` `41f10746`
- Decisions: 34g4 patch entry hardened, 3c hardened, part 3 mockup round 2, build times hardened — `5a857dd5` `2fa9af81` `fe2912db` `d0118f3d` `eefe95d7`

### Changed
- With no Devices draft, `Set` on a non-live column needs the claim — `90001754`
- Scripts run on macOS's Bash 3.2 — `86ecf894`; `*.local.env` ignored — `b12db2a7`

## 2026-10-17

### Added
- §12 tags part 1: the tag tree (`lumacore-tags`), `NodeMeta.tags`, `ShowData.tags`, format 28 — `c16d9f69` · [§12]
- §12 tags part 1b: NodeMeta in levels — `Identity` on every node, `Annotation` on every ShowData object, format 29 — `6e706cca` · [§12]
- §12 tags part 2: every object declares how it reads a dotted id (`IdReading`) — `94b226e9` · [§12]
- §12 tags part 3a: the `Tag` object (`lumacore-cmd-tags`), Path ids in the one resolver — `48c94d33` · [§12]
- §12 tags part 3b: verbs on a tag reach what it carries (`/R`) — `6b48ca90` · [§12]
- §12 tags part 4a: EditSetting's Meta and Tags tabs, the kit's `<luma-tree>` — `4991e1b9` · [§12]
- §12 tags part 4b: the Tags pool window, a tag's own settings — `d3248e5a` · [§12]
- 34f7c: a protocol is an object, a line one row (`Sacn 1` / `ArtNet 1` / `Osc 1`, `LocalUniverse` + `Amount`, `Mode` by Assign order), format 27 — `1fe4c85a` · [§34]
- Issue 11: DMX routes as objects under their line, keyed by slot — `1e81992d` · [§11]
- Decisions: tags hardened, 1b hardened, 34g4 patch hardened, 34f7c hardened, issue 11 hardened; lum.md's dotted ids — `12320744` `1f82b559` `2ba22477` `188d4113` `bff126fe` `d415740b`

### Removed
- `Output` / `Input` lines and `/Protocol` — a line is its protocol's row (34f7c) — `1fe4c85a`

## 2026-10-16

### Added
- 34f3a: `lumacore-registry`, the error catalogs its first table — `47458e3b` · [§34]
- 34f3b: the global flags are catalog rows; tests install by hook — `69216106`
- 34f3c: the feeds are catalog rows — `7fe28307`
- 34f3d: the window types are catalog rows — `15facb8d`
- 34f3e: tools and the built-in UI keys are catalog rows — `b2cf9b29`
- 34f3g1a: the command words are the registry's — `ce80e6ec`; 34f3g1b: `CommandDecl` rows + handlers — `76618f5e`
- 34f3g2: the objects are catalog rows (`ObjectDecl`, `def_of`, one namespace at freeze) — `60f19f0b`
- 34f3f: the menus are catalog rows; 34f3 done — `51a78cab`
- 34f4: `lumacore-views`, the one source of what any UI shows — `55ce762b`
- 34f5: `lumacore-dmx` | `lumacore-render` | `lumacore-protocols`, `PlaybackEngine` → `RenderEngine` — `697fe39c`
- 34f6: `lumacore-net`, the leaf network I/O — `cbcae8c0`
- 34f7a: each protocol a crate declaring itself; `lumacore-lines` the runtime — `356969b3`
- 34f7b: lines name their protocol row, protocol settings declared on the rows, E-SHOW-007 / 008, E-LUMA-006, format 26 — `ffd705ad`
- 34f8: the crates sit in folders by domain — `c0b9c9d3`; 34f8b: the station is `apps/lumacore-station` — `379300bd`
- 34f9: `lumacore-engines` and `lumacore-clocksync` — `1b5f1809`
- 34f10a: `lumacore-lanes` — `bcaed57a`; 34f10b: `lumacore-shell` — `a45df4f0`; 34f10c: `lumacore-lumasession` — `bbe29d37`; 34f10d: `lumacore-server` — `9069788f`; 34f10e: `lumacore-sweep` — `3b3f991c`
- 34f11: the station only assembles — `12dff7ad`
- 34g1: the renames — `model/`, `dev/`, `lumacore-headers` — `68b401af`
- 34g2a: `lumacore-sysmon` — `4696e6f2`; 34g2b: `lumacore-luma` — `3459a1cc`; 34g2c: `lumacore-protocols` below the tree — `0e713032`; 34g2d: `lumacore-fixtures` — `e3d3f52d`
- 34g3a: the command rows in families — `59ceb210`; 34g3b: the objects in families — `7d07e004`; 34g3c: dispatch's special cases are hooks — `cbc76ea9`
- 34g4 cuts 1–9: `lumacore-suggest` — `ebb4b9a1`; `cmd-treeshell` + `tests/catalog` — `8bb9b847`; `cmd-keys` — `26b0ef0e`; `cmd-showfile` — `4d5cd890`; `cmd-lumasession` — `1e8a1125`; `cmd-station` — `b893e8e2`; `cmd-devices` — `4a8ed48c`; the station's network into `cmd-station` — `b12ae68e`; `cmd-protocols` — `1cf25227`
- Decisions 21–25 (the catalog, protocol crates, folders by domain, the assembler, `commands/`) — `be7ae99e` `b462c50e` `aa42821a` `7e5ec14f` `fc747bdf`
- 34f7b hardened; lum.md typed values and words written as declared — `42557f31` `85b3cb72` `a26a3c64`

### Removed
- `lumacore-dmx` (folded into `lumacore-protocols` / `lumacore-lines`, 34f7a) — `356969b3`

## 2026-10-15

### Added
- §33 part 1: `lumacore-syntax` — one lexer, one total parser, print, cursor, highlight; every ` ```lum ` example a test — `9d820d88` · [§33]
- §33 part 2: the vocabulary from the registries, binding, `#[derive(Args)]`, the typed boundary; shim counter at 447 — `89ce82db`
- §33 part 3a: the tree-shell fence typed (13 Unix words), decisions 53–56 — `de699cf1`
- §33 part 3b1: default verbs — a bare object runs its declared verb; `SelectFixtures`, `Call` — `b2adea3c`
- §33 part 3b1½: the one resolver (`core::resolve`), tidy ratchet `old_resolvers_only_go_down` — `cf2ebadb`
- §33 part 3b2: the default object — a keyword alone sets it, a bare number is written out first (issue 02) — `640a82ec`
- §33 part 3c1: the one property registry, one typed `Set` — `59f88df9`
- §33 part 3c2 + 3c2b: the hand-written `Set` parsers on the registry; Theme / Paint / LockScreen objects; backtick lines — `1080af75`
- §33 part 3c3: every `Set` line is the registry's; the target-less forms go — `a4f1663f`
- §33 part 3d1: one typed `Label` — `f7936d39`; 3d2: the patch area typed, `Assign Universe … /Universe` — `7c5e1819`; 3d3: Groups typed — `f002b032`; 3d4: TimingPreset / CurvePreset — `1e6272d1`; 3d5: Output / Input / Network — `4c848a75`; 3d6: `Store Universe` — `8347300a`; 3d7: scoping owners, Goto / Store Cue owner first, On / Off / Toggle / Edit Sequence — `77e18274` `7d30b716`; 3d8: Master on the registry — `ee8dfa6e`
- Notifications hardened (§33 3c3 N1–N10) — `4235cc5a`; §33 decisions 92–94 (one List / Info / Dump) — `73739d05`
- 34a: code structure and file headers — the rules, the banner tool, tidy's structure rules — `87f36073` · [§34]
- 34b: `lumacore-state` cut out of the station (34b1–6, the boundary 53 → 0) — `11dc513b`
- 34c: `lumacore-objects` cut out, dispatch with it — `2dc55773`; 34d: nested dispatch depth cap (E-CMD-019) — `b0036ccb`
- 34e: every crate is `lumacore-<category>` — `0becbe09`
- 34f1: `lumacore-terminal` — `b691e407`; `lumacore-tidy` with `docs/meta/tidy.md` — `64bc6d05`; 34f2: trace macros in `lumacore-trace` — `ee808721`
- §35 the one object door: 35a `pools::door` — `09e4fd37`; 35b Group on the door, Cut / Paste / Insert / Compact — `caf33ba2`; 35e quoted `*` patterns — `f3f1ba95`; 35f `mv` / `cp` / `rm` — `c70f70bb`; 35g1 Sequence, TimingPreset, CurvePreset — `983c59bb`; first-free sequences, cues append — `71e7b587`; 35g2 Output and Input, the station clipboard — `47f4ee9e`; 35g3 Fixture and Universe move — `2100e953` · [§35]
- Decisions §33 (Lum syntax hardened), §34 16–20, §35 — `422f23ef` `232963bc` `0becbe09` `f8ae09c6` `f4ffcabb` `fa87058a` `3d8961aa` `76c5f2b4`

### Removed
- `root`, `rm`, `whoami`, `env`, `history` from the tree shell (3a) — `de699cf1`
- The Call catch-alls (`Universe 3` no longer creates) (3b1) — `b2adea3c`; the Fixture / Universe `context_number` (3b2) — `640a82ec`
- The target-less `Set` forms, `settings::handle_set`, `set::legacy_words`, `set_rows::route`, the objects' own Set handlers, the dead `UserPreferences` copies (3c3) — `a4f1663f`
- The hidden `Create` alias of Store and the objects' Create handlers (3d5, 3d6) — `4c848a75` `8347300a`
- Option short forms on typed words (`/o` → E-SYN-006) (3d3) — `f002b032`

## 2026-10-14

### Added
- 12f part 1: `Playbacks.Live` registry, masters, the gesture channel — `8b129d2b` · [§30]
- 12f part 2: CueZero, OffCue and echoes (`Top On|Off`, MaxEchoes / EchoMax) — `57ca4f13` · [§32]
- 12f part 3: `<luma-fader>` — `aa8b096b`; part 4a: value timing — `3b748980`; part 4b: the encoder bar — `8e8594af` `5df7d2d1`; part 4c: Cue timing modes — `67fea56b`
- 12f part 5: the Oops window — `acfa0e02`; part 6a: Sequences, Timing and Curve presets in DataPools (format 22) — `76986a50`; part 6b: the Sequence pool — `9f1facbd`
- 12f part 7a: a feature group's fade Path — `d2b4ff43`; part 7b: sequence window A — `de7de66b`; part 8: sequence window B, the timeline — `2f4fcb19`
- Store creates empty pool objects — `6c014fbc`; Store Cue (Cue = Store's default object, the store ask) — `f499ed42` · [§30]
- 12f-a part 1: the standard attribute table, the data-driven resolver, the GDTF importer's families, per-feature-group cue timing — `71a04014` · [§30]
- 12f-a part 2: `Attribute`, `FeatureGroup`, `AttributeRule` syntax — `debe880d`; part 3: the profile's attribute layer, encoder bars, waiting attributes — `f6dee424`; part 4a: `<luma-attr-board>`, the `attributes` feed — `8a14189b`; part 4b: the Encoder bar tab, "n WAIT" — `1518ba12`
- 12e part 2: `<luma-pool>`, the object sheet, NodeMeta colour and icon — `3aff4b9d`; part 3: editor tools (`ToolDef`), the list button, the group editor, tidy against hand-made hold timers and window-level pointer listeners — `ed3c45d9`; part 4: miniature lists (`<luma-grid view="mini">`), the phone desk folds the sidebar and swipes — `eb42a778` · [§30]
- Users & Profiles follows storage (Settings | Content, drill in) — `88f854fb` `c14ba5c2` · [§30]
- Declared flags + parse once, steps 1–8: name index — `19b0d0eb`; `FlagDef` — `3bf43cfb`; `line::parse` — `2e3ec2d5`; one parse in dispatch — `e870aae4`; E-CMD-018 — `576b255a`; typed flag reading — `fdcebe00`; help / reference / completion / sweep — `18bb3a53`; the families — `4bec6d94` `7b3248a8` `5e86c4a4` `4e7e3709` `96f01f7b` `cc1873e0` `06859ca4` · [§31]
- Typed `TreePath` steps 0–7: the golden — `75fffdec`; the table — `b1daf52c`; navigation — `20eccddc`; the VFS — `6e14c201`; ls metas / claims / one parse — `833e194b` `74c3e9e2` `4ef2e017`; List and scope — `9b64574f`; completion, tree shell, TUI — `a4810392`; guards and tidy `tree_paths_are_typed` — `807baaf2`; the path bugs it fixes · [§31]
- `tree Root.Drives` stops after 300 ms or 2,000 lines — `1d271509` `17992452`
- `Arc` parts: sequences, fixture types, profiles shared by every published copy — `85ab2391` `6dd0b75f` `999b3233`; the plan merge keyed by attribute index, the compiler keeps running sequences' offers — `d14339a7` `2b422ac2` · [§31]
- Benchmarks of the costly paths; every error code checked against the catalogs — `c7565b8f`; typed replies step 1 (`Reply { text, failed, code }`) — `79f619b8`
- Session stream: raw JSON for shows and snapshots, broadcasts encoded once — `55640c1d`; state job queue: two lanes — `4d24af52`
- Real-time scheduling for the render and send threads — `e697b910`; line timing step 1 (doubled / skipped frames) — `b00e856c`
- Registry rows state only what differs (`..CommandDef::BASE`) — `d0d524a0`; UDP command port never waits on the state thread — `6c64ce24`; replication journal byte cap — `ef4e35e1`; CLI: a lost station is a disconnect — `18a66ded`
- Build and tooling: dev profiles, lints, pinned toolchain, dependency cleanup, git hygiene — `4f83e0cb`; clippy clean — `0fb90861`; builds measured, no crate split — `47e25879`
- Desktop app: a window stays on its station, HTTP replies capped — `a57bb6b1`; IPC as one typed enum — `8f059779`
- Delete User names the OSC lines that run as the user — `42bfab5d`; OSC line users decided, two stale station tests fixed — `3fc6abd5`; systemtest follows the demo's fixture numbers (181 / 181) — `2924c0da`
- Decisions §31 (code review hardening), §32 (console study), 12f / 12f-a design locked; roadmap 0c — `3a87b938` `6de1c4a1` `135cfec1` `ac0c4c78`
- Merge `bugfixes` into main — `2c2131bf`

### Removed
- Unused dependencies (nom, terminal_size, theymx, sacn, dmx, and more); the `lumacore-tree` re-export shim files — `4f83e0cb`
- 888 default lines from the registry rows — `d0d524a0`; the last hand flag parsing — `06859ca4`

## 2026-10-13

### Added
- Step 12c: the window bar, relative numbers on `Set Window`, one state rule through `Self.<Property>`, layered resets, `WindowPreset` on the profile, `Self` = the line's owner, window types declared in Rust → JSON, `Export|Import View | ScreenConfig | WindowPreset`, the layout format decided (registry → JSON, no KDL) — `e847dffb` · [§30]
- Step 12c2: Default a locked template, new profile from Default / copy / import, delete user and profile as station asks with coded refusals, rights only from the User, the kit tile grid, multiplatform contracts + guards (tidy + Playwright), desk view on phone / tablet — `5d4e5f4e` · [§30]
- Step 12d parts 1–5: `/ws/feeds`, rights levels, `<luma-grid>` + `<luma-slab>` + `<luma-setup>` + `MenuDef`, the menus on the frame and the station table, Backup (Versions, Recovery, the Load show dialog) — `91c23675` · [§30]
- Remote station edits, direct and hardened — `a3ae58fe` · [§30]
- Step 12e part 1: narrow command line, one hold, split grip pill, window scroll report — `f99c628b`; 12e hardening and design locked — `27756bfb`
- The R1 bug report fixes (R1-01 … R1-71, the review's should-fix list, every question answered) — `dd680ff6` … `c7c81cd2` · [report](docs/research/r1-bugreport/luma-bugreport-R1.md)
- Issue 04 Admin refusals' codes — `c4b8e309`; issues 05 / 06 Show options and Restart / Shutdown on the change path — `e5cbf2b5`; issue 07 quoting (a quoted word is a value, a bare `/word` an option) — `eec2ba53`; issue 09 `float_roundtrip` — `14b086c3`; issue 20 a–h sweep harness — `2bc08d32`
- Merge of `bugfixes`: backups → Versions, one Delete ask in dispatch, `.starting` + `running`, busy on 12d's paths — `3215dbd7` `206b4ee9` `faa154c0` · [§30]
- `At <value> Attribute "X"` sets that attribute — `5fc75476`
- Developer launcher Plan and Status, dev scripts, live build output — `77c779c6` `4738f1ce` `dc13146d` `fc71b0fd`
- Issues from the first sweep run; the sweep sandbox capped — `279e5dea`

### Removed
- `/api/events`, `/api/station/events` and every poll (12d part 1); `legacy`, `npOpen` and the old overlays (12d part 4) — `91c23675`
- The page's client-only Delete confirm (R1-61) — `16a478db`

## 2026-10-12

### Added
- Step 11a: docs restructure — one roadmap (`TODO.md`, `requirements.md`, `project.md` merged in), `docs/archive/` with its README, the generated reference, one LumaSession doc, the decisions contents table, visions and handoffs sorted, the remaining files fixed — `a038f89a` · [docs plan](docs/plans/done/2026-10-11-docs-plan.md)
- Step 11b: rights all or nothing; transitive election; `Bind` key GUIDs; codes on every refusal; editor-only words say so; stale comments and scripts fixed; `MEOWNITOR [ip]` Planned; `help lum` pages fixed; Preset List path; `If` Planned; shortcuts doc; `Programmer.values` a `BTreeMap`; the sweep stops counting kept links; `Status::Planned` on pretending words; status per `ObjectDef` verb; `Root.Drives` only through tree-shell words; the sweep sandbox; `systemtest` compiles; §18 renames; every datapool's groups in the tree; `Show "x" /Save` = `SaveShow`; `EditSetting <object>`; web keyboard Release / Hold; GDTF native (`.luma` a zip, format 6); the derived `FixtureType`; cue timing per part; one-only selections; the selected sequence; Highlight from GDTF; per-user Highlight / Solo masters; E1.31 input per spec; no allocation per packet — `ed8d840f` · [§29]
- Step 11c: Oops-able UI, selection and programmer steps, categories, one list per user, station steps, `ShowSubtree::Selection`, `Oops <n>` / `/NoOops` / the profile's Oops settings, show load clears every list — `512a353e` · [§29]
- Step 11d: screenshots through the client (`POST /api/screenshots`), vendored xterm + capture lib, `Screenshot [Screen n] [User x]`, F11 / Print Screen, `scripts/unix/screenshots`, the first look at every window — `85edb4a5` · [§29]
- Step 12 hardening: touch UI kit decisions, research, locked mockups — `90020ec2` · [§30], [mockups](docs/dev/ui-kit/mockups/README.md)
- Step 12a1: UI kit foundations — `frontend/kit/` custom elements + pointer core, two-tier theme tokens, Lucide sprite + vendored fonts, touch baseline, hold = show me more, tidy colour / icon rules, test hooks + Playwright touch projects + Showroom, the 11d capture fixes — `b158e404` · [§30]
- Step 12a2: show format 7 — the `.luma` folder tree, embedded media, the Image pool, the profile's Theme / LockScreen / Icons / Images, owner-first words, GDTF in the tree, files by hash over the stream, asset drives, `Export|Import UserProfile`, uploads on a member go to the master — `7c84db23` · [§30], [§8]
- Step 12a3: user surface — apps as basic clients (connect page, `LumaHost`), one lock / login / connect surface, pre-login data, device ids, the profile's theme + `NormalTheme`, `DeskLock`, Admin's fixed password, core keys F9–F12, screen windows remember their display, the Screens overview, `User.Color` (format 8), the on-screen keyboard and standard dialog as kit elements, e2e migrated, CLI password ask — `ceb5fc0f` · [§30]
- Step 12b: dialogs — one server-held dialog, every ask on it, who gets it, no timeout = waits, terminals draw it, the TUI in the user's theme, relative numbers, `luma-keypad`, `luma-popup`, `Keyboard` per device, more asks, `/Remember` and `/Clear`, docs and tidy rules, pick lists for links — `7f5bd8c3` · [§30]
- Step 12b2: servers list with `leaving`, Meow = Shift, F12 full screen, the command line bar, text selection rule, typed setting descriptors, `<luma-settings>`, window settings shell, named choices, Users & Profiles window, menu tiles with icons, one feed rule, the System Monitor batch, one `/ws/feeds`, DMX at 60 Hz, Realtime at 60 fps, the suggestion engine, per-user Usage — `894cd79b` · [§30]
- Decisions: 12a2 / 12a3 hardening, 12b hardening — `1e006e83` `052a0d14` `7d5808da`

### Removed
- `--tls-cert` / `--tls-key` (parsed, never used); `core/language.rs` archived to `docs/archive/code/`; the removed Range from help text (11b) — `ed8d840f`
- `TODO.md`, `requirements.md`, `project.md` (merged into the roadmap and archived, 11a) — `a038f89a`
- The March phases and their plan files; the v1 / v2 split; the React + Vite + Tailwind stack (never used); the `SCRIBBLE` pool (not needed) — `a038f89a`

## 2026-10-11

### Added
- Step 4b: networks, output lines + routes, protocol runtimes from show content — `e4b128fc` · [§11]
- Step 4c: DMX input + per-channel merge, one Line type, the DMX window — `5b48ea8c` · [§11]
- Step 4d: Network / DMX Protocols / LumaSessions menus, LumaSession naming, the staged patch — `3b65944b` · [§11]
- Step 5: one rights gate, Remote Login holds, passwords masked, logout revokes — `d76f80bc` · [§19]
- Step 6a: whole-show change path, programmers per user, undo in Runtime, the show format version — `d14f6a8d` · [§21]
- Step 6b: the LumaSession link — TCP stream, members forward to the master, ordered results — `511655fb` · [§21]
- Step 6c: heartbeat, election, IdleMaster, two-masters prompt, `Claim LumaSession` — `990cb2b4` · [§21]
- Step 6d: strict GUID identity, merge on rejoin, reference check, ownership — `3ae128f6` · [§21]
- Step 7: groups in the datapool, the add-or-replace selection rule, grid, group masters, one pool window — `c2b99bb2` · [§22]
- Step 8: the `lumacore-wire` client contract, clients under `apps/`, station layers — `5e1e4655` · [§23]
- Step 9: the 60 Hz render engine, plan compiler, line rates, session clock, `Runtime.Health` — `7e0ec483` · [§24]
- Step 9b: app ↔ local server — version check, signals, hung stop, save guard, the demo-show test — `2f71191c` `1a199c6f` · [§23]
- Step 10: the command sweep, fuzz targets, property tests, `reach` + the drive door — `95cd6b0c` · [§25]
- Effects design (§28, design only) + user show files — `58a0f06e` · [§28], [handoff](docs/archive/handoffs/effects.md)
- System monitor research into 6c / 6d — `2e014eb0`

### Removed
- UNIV-03..06 universe ranges, `Split` / `Merge Universe` — ranges dropped in 4b; routes live on universes — `e4b128fc` · [§11]
- DMX-04 44 fps output — replaced by the 60 Hz render thread + per-line `Rate` (step 9) — `7e0ec483` · [§24]
- GDTF-09 subfixture addressing (cells instead); CUE-02 effects engine and Pawterns (§28 Effects + Pawtern); phaser steps inside `Value` (values static, dynamics are effects) — `58a0f06e` · [§28], [§3]
- Tauri desktop shell running the engine in-process — the app is a thin client on `lumacore-wire` (step 8) — `5e1e4655` · [§23]
- Authenticating the UDP command port, `/ws/monitor` and the diagnostics endpoints — stays open on isolated rigs (step 5) — `d76f80bc` · [§19]

## 2026-10-10

### Added
- Step 0: quick fixes + cleanup baseline (fmt, clippy, http tests, Art-Net, discovery spin, OSC break) — `533f7c4b` · [§26], [review](docs/plans/done/2026-10-10-review-roadmap.md)
- Step 1: one registry, one namespace for commands and objects — `90bebb28`; systemtest runs in `target/systemtest/work` — `888b4083` · [§15]
- Step 2: the state thread, the automatic change path, error codes — `c92c1907` · [§26]
- Step 3: mode objects (Blind, Highlight, Freeze, Solo), knock in / out, one key system; Blind suppresses programmer output — `31a16723` · [§16], [§17]
- Step 4a: the playback engine skeleton, one `Value` type, programmers LTP, Sequence → Cue → Part, Go / Go- / Goto — `9f10b69b` · [§27]
- Thin client (users, passwords, tokens, Connect page, one window per screen), object registry, System Monitor, UI keys + CleanStart, one feedback system — `6f259039` · [thin-client](docs/plans/done/2026-10-10-thin-client.md), [ui-keys](docs/plans/done/2026-10-10-ui-keys.md), [feedback-system](docs/plans/done/2026-10-10-feedback-system.md)
- The object model from the console probe: NodeMeta, the show icon pool, Fixture position / rotation / MVR UUID, SequenceSettings, cue blocked + MIB, PresetGroups, two-layer attribute definitions — `2552448c`
- Handoff: step 2 done, step 3 next — `fa0d42c1`

### Removed
- CMD-08 `Black` — blackout is the grand master off (4a) — `9f10b69b` · [§16]
- The abbreviation table — short forms are aliases in the registry only (step 1) — `90bebb28` · [§15]

## 2026-10-09

### Added
- Split panels, panel focus, the unbounded group pool, the HTTP engine in `Root.UIEngines` — `402288a5`; server-side key routing, panel focus events, the window type registry — `77f364f1` · [panels](docs/plans/done/2026-10-09-panels-focus-key-routing.md)
- View-sessions in Runtime: live windows with typed settings, editable with `Set` — `86d2371a`
- The group prototype (`FixtureGroup`, REST API, tabbed panel UI) — `c89cb0d5`; the first prototype (dynamic ViewBar) — `3b7551da`; the panel UI framework — `5181261b`
- The `lumacore-app` desktop shell — `f719ee0c`

## 2026-10-08

### Changed
- Repo cleanup: dev-tool artifacts removed, code quality fixes — `89614649`

## 2026-08-13

### Changed
- Local tooling config — `bf798332`

## 2026-03-16

### Added
- Proxmox CI orchestration for multi-station testing — `33d71c0c`

## 2026-03-15

### Added
- Replication journal, GUID-based fixture refs, `Label` and `Move` — `5ecb3ab0`
- Subtree-granular undo — `01fd77a4`
- Terminal / remote / systemtest modules restructured, six new test domains — `1136f134`
- Art-Net, OSC and the enriched `EngineContext` — `4c8a7a85`
- Shutdown propagation, remote architecture docs, visions, the web scaffold — `5b516ede`
- `stop_developer`, TUI visual grouping — `a4141105`; Tracy built by `dev_init` — `540db758`
- The March requirements built (archived `requirements.md`): GDTF-01..05 import, patch, conflict check, list, unpatch · TERM-01..11 command line, selection, `If` filter, tree, completion, history, help, verbosity, session log · CMD-01..21 (but CMD-08), PROG-01..03, CTRL-01..03 · UNIV-01/02 named universes, `Assign Universe … At …` · DMX-01..03 sACN multicast + unicast · WATCH-01..04 · DBG-01/02 live DMX readout and tree inspection · SHOW-01/02 save and load `.luma` ([§8], [§20]) · API-01..06 headless server, REST + WebSocket, web UI, many clients, one Oops for all · DESK-01 the desktop app as a thin client · DMX-06/07 Art-Net out + ArtPoll · INTG-01 OSC input lines · CUE-01 (partly) sequences, cues, Go / Go- / Goto, tracking · DOC-01 build docs for contributors

## 2026-03-14

### Added
- The runtime module system with Perfetto and sACN modules — `7aa6ed82`
- The System Monitor and profiling — `d7a3605b`; macOS runs — `3d4e5ae9`; Windows starts — `c507d585`
- Definitions restructured into `user/` and `dev/` — `85f02e22`; `PatchData` / `ProtocolsData` nesting — `268fbd77`

## 2026-03-13

### Added
- Definitions and system tooling — `b2f54df0`; the codebase aligned with them (timecode, effect engines, nav tree, completer) — `d96421cb`
- Lum sentence shapes — `a8016e88`; the logo and lore — `a982b563`

## 2026-03-12

### Added
- The migration into this repository — `c77a3537`; the Rust toolchain guide — `1b324895`
- The terminal — `89efb8ea`; the root tree reshaped — `e63c1df6`; the sACN sender — `879c5cea`

## 2026-03-10

### Added
- Initial commit: the LumaCore README — `9c5046eb`

[§3]: docs/archive/decisions-2026-10.md#3-pawtern-system-replaces-distribute
[§8]: docs/archive/decisions-2026-10.md#8-show-file-format-luma
[§11]: docs/archive/decisions-2026-10.md#11-protocol-architecture
[§12]: docs/archive/decisions-2026-10.md#12-tags--visual-browser-future
[§15]: docs/archive/decisions-2026-10.md#15-one-registry-one-namespace-2026-10-10
[§16]: docs/archive/decisions-2026-10.md#16-mode-objects--blind-highlight-freeze-solo-2026-10-10
[§17]: docs/archive/decisions-2026-10.md#17-keys-and-keymaps-2026-10-10
[§19]: docs/archive/decisions-2026-10.md#19-network-security-2026-10-10
[§20]: docs/archive/decisions-2026-10.md#20-show-file-versioning-2026-10-10
[§21]: docs/archive/decisions-2026-10.md#21-undo--replication-coverage-2026-10-10
[§22]: docs/archive/decisions-2026-10.md#22-groups-2026-10-10
[§23]: docs/archive/decisions-2026-10.md#23-code-structure-2026-10-10
[§24]: docs/archive/decisions-2026-10.md#24-render-thread-rules-2026-10-10
[§25]: docs/archive/decisions-2026-10.md#25-guards-and-work-order-2026-10-10
[§26]: docs/archive/decisions-2026-10.md#26-engine-ownership-errors-and-robustness-2026-10-10
[§27]: docs/archive/decisions-2026-10.md#27-playback-engine-2026-10-10
[§28]: docs/archive/decisions-2026-10.md#28-effects-2026-10-11
[§29]: docs/archive/decisions-2026-10.md#29-cleanup--docs-findings-oops-categories-2026-10-11-step-11-hardening
[§30]: docs/archive/decisions-2026-10.md#30-touch-ui-kit-2026-10-12-step-12-hardening
[§31]: docs/archive/decisions-2026-10.md#31-code-review-hardening-2026-10-13
[§32]: docs/archive/decisions-2026-10.md#32-binary-study-hardening-2026-10-14
[§33]: docs/archive/decisions-2026-10.md#33-lum-syntax-crate-hardening-2026-10-14
[§34]: docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15
[§35]: docs/archive/decisions-2026-10.md#35-one-object-door-store--delete--copy--move--paste-2026-10-15
[§36]: docs/archive/decisions-2026-10.md#36-webui-the-thin-client-and-3d-rendering-2026-10-18
[§37]: docs/archive/decisions-2026-10.md#37-phone-hardening-2026-10-19
[§38]: docs/archive/decisions-2026-10.md#38-drives-and-the-one-files-element-2026-10-19
[§39]: docs/archive/decisions-2026-10.md#39-the-settings-home-icons-2026-10-20
[§40]: docs/archive/decisions-2026-10.md#40-the-demo-rig-nested-fixtures-and-second-ids-2026-10-20
