# 12d "menus" — research for the hardening (2026-10-13)

Read-only sweep: decisions.md (§8, §9, §10, §11, §19, §21 6b–6d, §23, §29 11d, §30 all parts),
roadmap.md, CLAUDE.md, docs/user (reference, show-files, fixtures-patch-output, keys), docs/dev
(ui-framework.md D8, api/http-api.md, networking/lumasession.md, ui-kit/mockups README + boards),
docs/archive (TODO.md menu/backup lists, networking docs), handoffs (ui-kit.md, decisions-qa,
research/lumacore-inventory.md, inspiration-style.md, online-touch.md),
visions/notes/terminal-menu-mode.md. Code: frontend/index.html (5232
lines), frontend/kit/*, net/menu_views.rs, lumacore-http router / endpoints / websocket/feeds.rs,
server/state.rs (StationView), core/commands/{backup,settings,edit_setting}.rs,
core/objects/{system,protocols}.rs, lumacore-tree StationSettings, lumacore-wire Status.

---

## 0. The 12d promise (verbatim) and what it rests on

- Roadmap 12d: "Network / DMX Protocols / LumaSessions / Backup in one setup layout (title status,
  left tabs, content, function block, power toggle), pushed instead of 1 s polling".
- §30 "Decided": "**Menus (12d): a console setup layout, ours.** Network / DMX Protocols / LumaSessions /
  Backup share one layout: status in the title bar, tabs on the left, the content (station grid,
  line sheet) in the middle, a function block of 60 px action buttons + bound setting cells, a big
  state-coloured power toggle; pushed over the existing SSE / WS (no 1 s polling); every button
  stays a command."
- §30 Findings: "Menus poll every 1 s with ad-hoc JSON." / the reference has "one setup-menu layout, ≥ 40 px
  targets, no hover".
- §30 12c2: "(… the Network / DMX Protocols / LumaSessions / Backup rebuild stays 12d with a 44 px
  stop-gap)". Q&A: "Scope | Contracts + all audit fixes now; setup menus' kit rebuild stays 12d."
- §30 12a hardening: "Old dialogs / menus / settings keep working until the part that owns them
  replaces them (12b dialogs, 12c settings, 12d menus, …)".
- §30 strict rule: every menu "composed only of kit elements … A window that needs something the kit
  lacks waits: the new element is designed first as a mockup artifact". Roadmap 12a item still open:
  "Tidy rule: windows, menus, dialogs compose only `luma-*` kit elements … switched on as each part
  replaces the old code".
- Locked boards for 12d (mockups/2026-10-12): **Network** ("Setup layout: tabs, service cells,
  function block, power toggle") and **LumaSessions** ("Station cards, actions follow the
  selection"). No DMX Protocols board, no Backup setup-layout board exist. (The 4d mockup
  https://claude.ai/artifact/U2kNxUADmPt5JvHJMN3y12 predates the kit.)

---

## 1. Shared infrastructure today

**Overlay family** (index.html 798–897): `.lc-scrim` + `.lc-box` +
`.lc-box-title` (icon + title + info text + 50 px `lc-close`) + `.lc-box-body` — the 12c2
overlay contract (box ≤ screen − 32 px, inner scroll). Inside Network / DMX Protocols / LumaSessions
everything is hand-built `innerHTML` (`np-tabs`, `np-sub`, `np-body`, `np-foot`, `np-station`,
`np-table`, `np-btn`, `np-editor`), with the 44 px stop-gap CSS (`index.html:695`: "the 44 px floor
as a stop-gap until their kit rebuild (12d)").

**Polling** (index.html 3915–3975): `npOpen` → `setInterval(npRefresh, 1000)` fetching
`/api/network` or `/api/lumasessions`; re-render only when the JSON string changed and nothing is
focused / no `.np-editor`; `npRun(cmd)` sends the line over `/ws/shell` then refetches at 350 ms and
1200 ms. Two-step confirm `data-arm` ("Click again: …" for 3 s).

**Server side**: `net/menu_views.rs` builds `network(state)` and `lumasessions(state)` as ad-hoc
`json!` on the state thread inside `station_view()`; `StationView` is published (ArcSwap) **every
250 ms** (`STATION_VIEW_EVERY`) or on show change / `publish_station_now`. `network()` calls
`lumacore_dmx::nics::list()` (getifaddrs) each time → a syscall on the state thread 4×/s even with
no menu open. HTTP: `GET /api/network`, `GET /api/lumasessions` (login), `GET /api/shows` (no login),
`GET /api/station` + SSE `/api/station/events` (500 ms check loop). `/ws/feeds` feeds today:
`health.render`, `health`, `sysmon`, `feedback`, `dmx`, `pools`, `windows` — **no menu feed**.
`Status` (per client, ~50 ms, sent on change) already carries `show: ShowStatus` (name, dirty,
description, autosave_minutes, autosave_in, startup, format, merge default/timeout, references) and
`lumasession: LumaSessionStatus` (status, health, latency, master, pending, show_status, decision).

**Opening**: Menu tiles call client functions (`openNetworkMenu()` …). The `Menu` command only knows
`Menu ["Menu"|"Backup"|"Shutdown"]` (reference, rights Guest, scope Client). Network / DMX Protocols /
LumaSessions / Users & Profiles / Screens can't be opened by a command line or a UI key.

**Kit today** (custom elements): luma-avatar, button, icon, keypad, keyboard, popup (pick list),
dialog (openDialog / showDialog), people, overview, surface, vlist, suggest, tiles, settings,
window-bar, maxbar, graph; plus menu.js, toast, miniature, feeds.js (FeedClient). `<luma-settings>`
already does "tabs on the left from sections + a grid of typed cells, every change one line", has an
`embedded` mode and container-query collapse below ~600 px. **Missing for 12d**: a setup-menu shell
(title status chips + left tabs + content + function block), a status chip, a power toggle, a
station card grid, a sheet / table with inline typed cells (roadmap puts "lists, sheets" in 12f),
a routing matrix, a file/show browser list, a "last command" echo line (shown on the Network board),
a swipe button (roadmap 12b2: "a swipe-button element is still open").

**Typed descriptors**: `EditSetting` builds `SettingField`s only for Show (`Show /<Option>` lines),
User, Window, Self and objects with `TreeDef::write` props. Output / Input / Network have props;
**Station and Interface have none** (hand-parsed `settings::set_station` / `set_interface_services`;
reference shows no property table) → no bound cells possible for station settings without new
props. Show settings write through `Show /Autosave n` etc., not `Set`.

**Rights**: `Set` is Programmer (so `Set Station Online Off`, `Set Interface … No`, `Set Station
OutputWhenAlone` need only Programmer); Join / Leave / Invite / Dismiss Programmer; SaveShow /
LoadShow / NewShow / DeleteShow Programmer; Menu Guest. §30: "rights (disabled, not hidden, for a
Guest)". §10: merge default + timeout "need Admin rights".

---

## 2. Per menu

### 2.1 Network

**(1) Today.** Tabs Interfaces / Networks (top strip). Interfaces: table name, address, Yes/No per
LumaSession / Shell / HTTP (`Set Interface en1 HTTP No`; loopback fixed). Networks: No, name, NIC
`<select>` (`Set Network "Rig" Interface en1|none`), state ("not mapped — nothing is sent", "no
interface 'x' here"), lines on it, Delete (disabled with lines). Footer: Store Network (name input),
Output when alone toggle, Resume DMX output (when held), "LumaSession: On" (two-step →
`Set Station Online Off`) / "LumaSession: Off — turn on". Title info: "LumaSession on" / "DMX output
held" / "LumaSession off — only localhost is served". Data `/api/network` polled 1 s.

**(2) Rules.**
- §11 4d: "**Network menu:** Interfaces tab — every NIC (name, address, link) with Yes/No per service:
  **LumaSession** … **Shell** (47654), **HTTP** (47656); loopback always serves Shell and HTTP; default
  Yes everywhere … station-local, changing one restarts that listener. Networks tab — the show's
  networks → NIC dropdown, link, lines on it, state; output when alone."
- §11 4b: "A line on an **unmapped network never sends** … no Auto fallback, also not for a first run."
- §11 4d: "**LumaSession off** stops everything networked: sessions (it leaves first), discovery, Shell
  and HTTP on NICs (localhost stays), DMX protocols."
- §10 / §21 6c: "output when alone" per-station, **off by default**; dismissed/leaving → output held
  "until their operator resumes it".
- §21 6c: Link down on the stream's NIC ("no cable on en1") — not shown in the menu today.
- CLAUDE.md: station-local is never replicated; Oops category Station (11c: "Station settings and
  watches ride in a step as `UndoFrame.local`").
- Network board (locked): title "Network · desk1 · station-local, never in the show", status chips
  **Online · LumaSession: standalone · sACN 1/1**; left tabs **Interfaces · Networks · Stations**;
  interface rows with big green Yes cells (loopback "Always"); function block "Actions: Store
  Network…, Map network to NIC…", "Settings: Output when alone Off"; power toggle **"LumaSession
  on"**; a last-command echo line under the content.

**(3) Contradictions / stale.**
- The board's power toggle runs `Set Station LumaSession On|Off` (Network.dc.html:91) — that word
  sets the session **name**; on/off is `Set Station Online On|Off`. And the UI calls `Online`
  "LumaSession: On" though it also stops Shell/HTTP on NICs and DMX protocols — wider than
  LumaSession (§18/§11 naming: LumaSession = server-to-server only).
- The board has a **Stations** tab; §11 gives the Network menu only Interfaces + Networks (stations
  are the LumaSessions menu).
- Board fills "Yes" cells green; §30 look: "status colours only as small marks (green online dot …)".
- §11 asks for NIC "link" in both tabs; `/api/network` sends no link state.
- ui-framework.md / CLAUDE.md say "Menu → Network" overlays are "console-style overlays" — fine, but
  lumacore-inventory screenshots (08 network) are pre-12a (historical, research kept unchanged).

**(4) Gaps.** Power toggle meaning and label (Online vs LumaSession); whether Stations tab exists;
"Map network to NIC…" = pick list (`PropKind::Link`-like) over NICs — needs a NIC pick list
descriptor; Station/Interface props aren't declared (no bound cells, no `EditSetting Station`);
rights for Online Off / service off (Programmer today; a reference console = Setup); the dangerous ones (Online Off,
HTTP off on the NIC this page uses) still use "click again" — should become a station ask
(`/NoConfirmation`); AssetRoot → folder mapping (12a2, station-local, `Set AssetRoot`) and
`MulticastBase`, `ThemeOverride` have no menu home; NIC list refresh cadence (today getifaddrs every
250 ms on the state thread).

### 2.2 DMX Protocols

**(1) Today.** Top tabs = one per network (dot in/out/io, NIC tag); sub-tabs Lines / Routing /
Discovery. Lines: table Line (OUT/IN badge), name, protocol, Target / Port + User (OSC) as free
inputs, Priority (input) or InputPriority select, Merge select ("…" when mixed), routes as ranges,
Enabled Yes/No, State, Delete (click again). Routing: universes × lines matrix, input Merge column;
a cell opens an absolutely positioned editor (Universe a thru b, protocol universe, Assign /
Unassign). Discovery: sACN sources + Art-Net nodes on that network. Footer: direction / protocol
selects + name + Store Line (`Store Output "x" /Protocol sACN /Network "Rig"`), "Map … in Menu →
Network". Not shown: Rate, Send Full/Changes, PacketGap, Sync, Mode, the line's measured send rate
(List Health), sACN priority of outputs is a free text input. Data = the same `/api/network`,
polled 1 s.

**(2) Rules.**
- §11 4d: "**DMX Protocols menu:** one tab per network (Rig, Guest, …, coloured in/out/both), each
  with sub-tabs **Lines** (outputs and inputs, settings, live state), **Routing** (matrix: local
  universes × the network's lines; clicking a cell opens an editor — protocol universe, range,
  Assign / Unassign) and **Discovery** (sACN sources and Art-Net nodes heard there)."
- §11 4d: "**No claim for protocols:** networks, lines, routes and merge modes are edited live without
  the Patch claim".
- §11 4c: "**One Line type** … one set of handlers … parameterized by `Direction`"; cross-direction
  settings refused (E-SET-001).
- §11 4b: line status "sending N universes / network not mapped / socket error (code) / silent (not
  master)"; 4c "receiving 6/6", "source lost"; status-bar icon per protocol.
- §11 4b: only standalone / master / IdleMaster-with-output-when-alone send; others "silent (not
  master)".
- §11 Live Patch: output-line settings may change without the Patch claim (Setup/Admin rights).
- §30: "content (station grid, **line sheet**)"; Output props: Target, Priority 0–200, Rate 1–60,
  Send Full/Changes, PacketGap 0–20000, Sync, Enabled, Network (Link with pick list + Add New since
  12b); Input: Target, Enabled, InputPriority (Choice), Port, User (Link, optional).

**(3) Contradictions / stale.** "One tab per network" with the setup layout's "tabs on the left"
— two tab levels (network × Lines/Routing/Discovery) don't map onto one left tab column; no board
shows DMX Protocols in the setup layout. A power toggle has no defined meaning here (a reference console's DMX
protocols menu has per-protocol enables). §11 4b "Errors: two universes sending the same protocol
universe on one line" — the matrix editor surfaces this only as a command reply.

**(4) Gaps.** Tab model (networks left + sub-tabs top? or Lines/Routing/Discovery left + network
filter chip?); the sheet element (12f lists the sheet); routing matrix element + its editor as a
station ask with inputs (12b dialog) vs an inline kit pop-up; which line settings are cells in the
sheet vs in the function block (selected line's typed `EditSetting Output n` cells?); live line
state on a **member** (it never sends, so every line reads "silent (not master)") — should it show
the master's send state?; selection of a line/cell for the function block is client state (§22 /
§30 pools: "never add … client-only state"); high-rate parts (source list, receiving n/m) want the
`health` feed's line table?

### 2.3 LumaSessions

**(1) Today.** One table of every discovered station (name + member tag, IP, LumaSession, location,
status, priority, invite, version, show status), click selects (client state); station form
(name, LumaSession, location, priority, multicast base — free inputs / selects running `Set
Station …`); footer Invite Station, Dismiss Station, Join <session> (`Join LumaSession ip:port`),
Claim master (click again), Leave LumaSession (click again), Invite On/Off, LumaSession On/Off
(= Online). Title "desk1 · Master · LumaSession "Festival"". Data `/api/lumasessions` polled 1 s.
The merge / two-masters decision is a station dialog (12b).

**(2) Rules.**
- §11 4d: "**LumaSessions menu (full):** station grid from discovery (name, IP, LumaSession,
  location, role, priority, status, version); Invite / Dismiss / Join / Leave; this station's name,
  LumaSession name, location, priority, Invite on/off, LumaSession on/off."
- §11 4d: Invite "joins without asking and takes the session's show; unsaved changes are kept
  as an autosave copy first"; dismissed/leaving → standalone, DMX output **held off**.
- §21 6c: heartbeat 500 ms / warn 1.5 s / lost 5 s; `Claim LumaSession` takes the master role; known
  stations (`List Station`: In Session / Lost / Known), `Delete Station` (master, Admin; refused
  while connected); MulticastBase "shown in the LumaSessions menu"; show status set (NoShow …
  Negotiating …) "in the discovery beacon, the LumaSessions menu".
- §21 6c built: `Info LumaSession`: Link (health, latency, last heard, resends), Epoch, Show Status,
  Decision; status-bar LumaSession icon.
- §21 6d: merge default/timeout in ShowSettings (Admin); `Pause LumaSession`.
- §11 Q&A: "Identity | Named sessions found by discovery; join by address still possible."
- LumaSessions board (locked): title `"Festival" · epoch 4`, chips **This station: Master · Clock
  locked**; left tabs **Stations · Discovered · Settings**; station **cards** (name, role badge
  Master/Member/Invited/Lost, IP · prio, health dot "OK / Warn 1.6 s / Lost 12 s", latency, users);
  function block "Selected: FOH" with Invite / Dismiss (danger) / **Hand master over = `Claim
  LumaSession (on FOH)`** / Leave (danger), each showing its command; power toggle **In "Festival"**.

**(3) Contradictions / stale.**
- Board: "Hand master over … (on FOH)" — no command hands the master role to *another* station;
  `Claim LumaSession` runs on the claiming station (§21 6c). Either a new form (`Claim LumaSession
  Station "FOH"` run on the master) or the button only exists on the station itself.
- Board power toggle "In 'Festival'" vs §11 "LumaSession on/off" (= `Online`) vs today's footer —
  three meanings (membership vs station networking).
- Board card fields (per-member latency, health age, users) exist on the master
  (`Runtime.LumaSessions.Members`, link health) but `/api/lumasessions` sends only `connected` +
  users per member; "Invited" isn't a state the data carries.
- Board "Discovered" tab splits the §11 "station grid from discovery" into members vs others.
- §10 "Merge All / Keep Master / Keep Mine / Standalone" names superseded (annotated) — fine.
- Roadmap Later: "`Join LumaSession <host>` by address didn't join in the step-7 live test" and
  "Invite runs the join as the station's first logged-in user" — the menu's Join uses ip:port.

**(4) Gaps.** Tabs (Stations / Discovered / Known / Settings?); Known stations (show's list) with
Delete Station; whether the session password idea (roadmap Later) gets a cell; station-name /
LumaSession / location as bound cells → needs Station props; dangerous actions as station asks;
clock tab/chip (board shows "Clock locked": `Runtime.Health.Clock`); a member's view (actions
disabled with reasons); push design (discovery changes ~1 s beacons).

### 2.4 Backup

**(1) Today.** Kit `<luma-tiles>` (12c2) Load / New Show / Save As / Save / Delete; current show
name + "unsaved" badge; description textarea (`Show /Description "…"` on change); AutoSave
Interval `<select>` (`Show /Autosave n`) and Startup Show `<select>` (`Show /Startup Last|None|"x"`);
title info "Autosave in 14:45"; a show browser (search, name input, Go button, rows with modified
time and size; double-click runs `LoadShow "x"` / `SaveShow` / `NewShow` / `DeleteShow`). Data:
`Status.show` (pushed) + `GET /api/shows` fetched on open and when name/dirty changes (no timer).
Every ask (Save / Don't save / Cancel, overwrite) is a station dialog (12b).

**(2) Rules.**
- §9: "**Master-initiated** … **Network-synchronized** … Every station gets a recovery copy · On crash
  recovery, offer to restore from autosave · Autosave location: `library/shows/.autosave/`" (status
  table: partly).
- §21 6d: "Every losing side autosaves first (`.autosave/{show}_{reason}_{station}_{time}`)".
- §23 9b: save guard E-SHOW-004 (`{name}_format{N}_…` copy, `/Overwrite`).
- ui-framework.md D8: Backup = "current show (unsaved badge), editable description, Load / New Show
  / Save As / Save / Delete, AutoSave Interval, Startup Show; a show browser with search …; autosave
  countdown in the title".
- show-files.md: timed autosave saves the named show **in place**; "The Backup menu shows when the
  next one is due".
- §30 12c2: "one kit **tile grid** for Menu, Shutdown and Backup".
- Roadmap: Ideas "Backup: drive selector + eject, show history, partial load masks …, export media
  on save, enumerate file name on description change"; UI-keys leftovers "A backup copy on every
  save (keep up to 10 per show)"; Later "Network-synced autosave", "Autosave for untitled
  shows", "'Startup show failed to load' pop-up".

**(3) Contradictions / stale.**
- §9 "Autosave location `.autosave/`" vs built timed autosave saving **in place** (show-files.md);
  `.autosave/` holds only signal / invite / merge / split-base copies.
- 12c2 just made Backup a tile grid; 12d puts Backup in "one setup layout" with left tabs, function
  block and a power toggle — not reconciled (what is Backup's power toggle? its tabs?).
- `backup.rs` module doc and the `Show` object reference still say "`.luma` JSON files" (a zip since
  format 6/7, §8 annotated).
- ui-framework.md D8 cites `terminal/commands/backup.rs` (now `core/commands/backup.rs`).

**(4) Gaps.** Tabs (Show / Library / Autosaves-Recovery / Startup?); listing and restoring
`.autosave/` copies (§9 recovery promise; no UI or command lists them); show browser element (kit
list); Description / Autosave / Startup as bound cells — they write via `Show /Option`, not `Set`
(fine as `SettingField.line`, but no props); `/api/shows` has no login while the menu routes do;
library list push (file system changes outside the station aren't noticed); rights (Guest).

### 2.5 Other Menu tiles

Today (`MENU_TILES`, index.html 3828): enabled Network, DMX Protocols, Backup, Quick Save
(`SaveShow`), LumaSessions, Users & Profiles (`<luma-people>` floating window, `/api/station` + SSE),
Screens (`openScreensOverview` → app window or `/?overview` tab). Dimmed "planned": Patch, Live
Patch, Connectors, Desk lights, In & Out, Settings, Show Creator, Preferences.

- **Patch / Live Patch**: §11 staged patch built (terminal only); roadmap Later "Patch menu over the
  staged patch draft"; §11 "closing a menu that claimed it" asks Keep/Discard/Cancel; Live Patch
  rules (§11 4b) partly built (offsets not). Not in 12d's list.
- **Settings**: roadmap Ideas "Settings pop-up: User Configuration, Date and Time, USB
  Configuration, Software Update, Extension Configuration, Local Settings". Natural home for
  station-local settings that have none: station name/location, ThemeOverride, MulticastBase,
  AssetRoot mapping, Startup show, Keyboard default.
- **Preferences (and Timings)**: profile preferences moved into Users & Profiles (12b2: every
  `UserPreferences` field a `UserProfile` prop) — the tile is now redundant or a shortcut.
- **Desk lights (& Color Theme)**: the 12g theme editor (Users & Profiles → Look waits for 12g).
- **In & Out**: DMX/MIDI/timecode/LTC I/O; overlaps DMX Protocols input lines and OSC lines;
  MIDI / timecode inputs are Ideas.
- **Connectors, Show Creator**: no decision anywhere.
- **Screens**: opens the overview (12a3); not a setup menu.
- Roadmap Ideas line "Menu tiles still grey: Live Patch, Connector Configuration, Desk Lights &
  Color Theme, In & Out, Show Creator, Preferences and Timings" omits Patch and Settings (both grey).
- ui-framework.md D8 still says "Menu (⚙): a 4×3 grid" (now 15 tiles) and the Shutdown menu
  list lacks Full screen / Screenshot / Screen config (12c2).

---

## 3. Cross-cutting contradictions / stale statements

1. decisions.md contents table: "§30 … 12a1 + 12a2 + 12a3 + 12b + 12b2 built" — 12c and 12c2 are built.
2. CLAUDE.md: `FORMAT_VERSION` = 10 and "`SESSION_PROTOCOL` 8" — 12c2 built format 11, CLIENT 9,
   SESSION 9.
3. §30 12d "pushed over the existing SSE / WS" vs §30 12b2 "**One `/ws/feeds` per page** (old
   per-feed endpoints go, no shims)" and "every feed … one batch when a window opens … then changes
   in batches per tick". Station events are still SSE (`/api/station/events`, a 500 ms check loop).
4. §30 12b: "The menus' 'click again' and Delete without use stay as they are" — 12d must decide
   whether they become station asks (every other dangerous action is one since 12b/12c2).
5. Locked Network board: `Set Station LumaSession Off` (wrong word), a Stations tab (not in §11),
   green-filled state cells (vs "status colours only as small marks").
6. Locked LumaSessions board: "Hand master over … (on FOH)" has no command.
7. §9 autosave location vs in-place timed autosave (above).
8. 12c2 "Backup = tile grid" vs 12d "Backup in the setup layout".
9. Naming: UI label "LumaSession: On/Off" for `Online`, which also stops Shell/HTTP/DMX.
10. `.luma` "JSON files" in backup.rs doc + `Show` object reference; ui-framework.md D8 paths/grid.
11. Roadmap Ideas grey-tile list stale (Patch, Settings missing).
12. http-api.md lists `/api/network` / `/api/lumasessions` as the menus' data — will change with push.

## 4. Gaps needing a decision before 12d can be built

- **Scope**: the four menus only, or also Settings (station settings home) / Preferences /
  Desk lights shortcuts; Patch stays out?
- **Shape source**: menus declared in a Rust registry → JSON (like 12c window types: tabs, function
  block actions as command templates, bound fields from typed props, power toggle def, status chips),
  or each menu composed by hand in the page from kit elements.
- **Menus as overlays vs window types** (dockable Network / DMX Protocols windows; the reference = overlays).
- **Push**: new State feeds on `/ws/feeds` (`network`, `lumasessions`, `shows`?) vs Status vs SSE;
  StationView cost (recomputed + getifaddrs every 250 ms even with no menu open).
- **Typed station props**: declare Station / Interface / NIC-service / network-map settings as props
  (`EditSetting Station`, `Set Station …` from descriptors) so cells bind; Show settings `Show /Opt`.
- **Tabs per menu**, function block contents per tab, the selection model (client highlight + a
  template `{selected}` vs a server-side selection), what "power toggle" means per menu (Network =
  Online; LumaSessions = Online or Join/Leave; DMX Protocols = all lines enabled / hold output?;
  Backup = none / autosave?), its colour states (§30 look rule), and a confirm (ask) on off.
- **Dangerous actions** as station asks (Online Off, Leave, Dismiss, Claim master, Delete line /
  network, HTTP off on the serving NIC).
- **Rights**: Programmer vs Admin for station networking; Guest sees disabled with reason.
- **Phone / narrow layout** for left tabs + function block (12c2 contract: container queries,
  44 px floor, `…`); a reference console's function block is 240 px tall.
- **Kit elements to design (mockups first)**: setup shell, status chip, power toggle, station cards,
  sheet with typed cells, routing matrix + cell editor, show browser list, echo line, swipe button.
- **Open by command**: `Menu "Network" | "DMXProtocols" | "LumaSessions" | "Users" …` (UI keys,
  palette, tests).
- **Member behaviour**: DMX Protocols line states on a member; LumaSessions actions on a member.
- **Backup recovery**: list / restore `.autosave/` copies; "backup copy on every save".
- **Tests**: e2e for the menus' functions (only multiplatform fit + screenshots touch them today);
  switch on the "menus compose only kit elements" tidy rule.

## 5. Candidate hardening questions (recommended option first)

1. **Which menus are 12d?** (a) Network, DMX Protocols, LumaSessions, Backup + a new **Settings**
   (station-local settings without a home: name, location, ThemeOverride, MulticastBase, AssetRoot
   mapping, Startup show) — because those settings have no UI and share the same layout; (b) only
   the four; (c) four + Patch menu over the draft (too big; the Patch sheet is its own part).
2. **Where does the menu's shape come from?** (a) Declared in a Rust registry (`MenuDef`: tabs,
   chips, actions = command templates, fields = typed props, power) → `GET /api/menus` JSON, drawn by
   one kit `<luma-setup>` — same model as 12c window types, testable, plugin-ready; (b) composed per
   menu in the page from kit elements; (c) console-style one hand-written layout per menu.
3. **Overlay or window?** (a) Overlay (floating window like Users & Profiles), one kit shell, a phone
   gets it full screen — as established consoles and today; (b) also a dockable window type per menu; (c) window types only.
4. **Push transport?** (a) New State feeds on `/ws/feeds` — `network`, `lumasessions`, `shows` —
   subscribed while the menu is open, latest-wins, built from the published StationView only when
   subscribed (drop the 250 ms always-on rebuild / getifaddrs) — follows the 12b2 one-socket rule;
   (b) carry them in each client's Status; (c) SSE like `/api/station/events`.
5. **Bound setting cells: how?** (a) Declare Station, Interface services and the network→NIC map as
   typed props (`EditSetting Station`, NIC pick list), function-block cells are `SettingField`s drawn
   by `<luma-settings>` cells — one write path, no hand-made `Set` paths (12b2 rule); (b) keep the
   hand-parsed `Set Station` and hand-describe fields in the menu; (c) free inputs as today.
6. **What is the power toggle?** (a) Per menu, declared: Network + LumaSessions = `Set Station Online
   On|Off` labelled **"Online"** (not "LumaSession", it stops Shell/HTTP/DMX too), with a station ask
   before Off; DMX Protocols = `Set Station HoldOutput` ("DMX output on / held"); Backup = none —
   one meaning per state, like a network-enable switch; (b) LumaSessions' toggle = Join / Leave as on the board;
   (c) only Network has one.
7. **Network tabs?** (a) Interfaces · Networks (as §11), no Stations tab (that's LumaSessions); fix the
   board's command to `Set Station Online`; (b) keep the board's three tabs; (c) one combined sheet.
8. **DMX Protocols tab model?** (a) Left tabs = the networks (with in/out dot and NIC), a segmented
   Lines · Routing · Discovery at the top of the content; function block = Store line + the selected
   line's typed cells (`EditSetting Output n`); (b) left tabs Lines · Routing · Discovery with a
   network filter chip; (c) a single line sheet across networks.
9. **LumaSessions tabs and cards?** (a) Stations (cards: members + discovered, role badge, health dot,
   latency, users) · Known (show's known stations, Delete Station) · This station (bound cells) —
   send the per-member health the master has; (b) the board's Stations · Discovered · Settings;
   (c) one table as today.
10. **"Hand master over" on the board?** (a) Add `Claim LumaSession Station "FOH"` run on the master
    (an Admin hands over; the target is told) — matches the board, a reference console lets any desk move it;
    (b) only "Claim master" on the station itself (drop the board's per-station button);
    (c) both.
11. **Selection in a menu** (which station / line / cell the function block acts on)? (a) Client-only
    highlight that fills the action's template (`Invite Station "{selected}"`) — like dialog input
    templates; the line still carries everything; (b) a server-side selection (`Select Station …`,
    today E-CMD-014); (c) no selection — actions per row.
12. **Dangerous actions?** (a) Station asks (12b dialog, `/NoConfirmation`) for Online Off, Leave,
    Dismiss, Claim master, Delete line / network, HTTP/Shell off on the interface this page came in
    on — drops the last "click again" pattern; (b) keep "click again"; (c) hold-to-confirm buttons.
13. **Rights?** (a) Station networking (`Set Station Online`, `Set Interface`, `Set Network …
    Interface`, MulticastBase, Join/Leave/Invite/Dismiss/Claim) = Admin, lines/routes stay
    Programmer, Guest sees everything disabled with the reason — console-style Setup rights, isolated rigs;
    (b) leave Programmer everywhere; (c) a new Setup right.
14. **Backup in the setup layout?** (a) Yes: left tabs Show (current show, description, Save / Save As
    / Quick Save, autosave cell, startup cell) · Library (show browser: Load / New / Delete) ·
    Recovery (`.autosave/` copies with reason, restore = `LoadShow`), no power toggle; the tile grid
    stays only for Menu / Shutdown; (b) keep the 12c2 tile pre-menu + browser, only restyle;
    (c) Backup leaves 12d.
15. **Opening menus by command?** (a) `Menu "Network" | "DMXProtocols" | "LumaSessions" | "Users" |
    "Settings"` (registry list of menus), tiles run it, UI keys can bind it; (b) client functions as
    today; (c) object words (`Network` alone opens the menu).
16. **Phone / narrow?** (a) Container queries: left tabs become a scrolling top strip, the function
    block becomes a bottom sheet (actions + `…`, power toggle full width), cards one column; 44 px
    floor; (b) the same three columns scaled (miniature); (c) hide the function block behind a
    button.
17. **State colour (look rule)?** (a) Power toggle may fill with the state colour (a big
    power control), every other cell shows state as a small dot (inspiration rule) — redraw the
    Network board's green cells; (b) green-filled cells as on the board; (c) no fills anywhere.
18. **Kit elements and mockups?** (a) New board set (row 12): setup shell + chips + power, station
    card, sheet with typed cells, routing matrix + cell ask, show browser, phone frames — approve,
    then build; DMX Protocols and Backup boards are new; (b) reuse the 2026-10-12 boards as locked
    and draw only DMX Protocols / Backup; (c) build without new boards.
19. **Member view of DMX Protocols?** (a) Show the master's line states (they ride in the replicated
    runtime / health) with "sent by <master>" — an operator on any desk sees what the rig gets;
    (b) local only ("silent (not master)"); (c) both columns.
20. **Echo line** (board: "› Set Interface en0 LumaSession Yes")? (a) Yes, the menu shows the last
    line it ran and its reply (teaches the syntax, errors visible); (b) rely on the command line
    bar; (c) toast.
