---
title: HTTP API
category: docs
---
# HTTP API

The HTTP engine serves the web UI and the desktop app: REST reads, two
WebSockets (everything live on `/ws/feeds`, the terminal on `/ws/shell`; no
SSE stream and no poll since 12d) and the static frontend. A second, separate engine
serves Perfetto traces.

Source: `crates/ui/lumacore-http/` (routes in `src/router.rs`), started by
`HttpUiEngine` in `crates/engine/lumacore-engines`.

## Goals

These come from the old "API, Web UI and UI engines" note (archived in
[`docs/archive/dev/integrations/api-web-ui.md`](../../archive/dev/integrations/api-web-ui.md)).
They still hold:

- The server owns the truth. Browsers and the app only show it.
- Every change is a **command**. The HTTP engine holds a read-only snapshot of
  the show; anything that must land in the show runs as a command line through
  `ConsoleBridge::execute` or the `/ws/shell` bridge, as the logged-in user.
  It passes the same rights gate, undo and replication as typed input.
- Saved views live on the user in the show (`ShowData.UserProfiles`). The live
  windows of a screen live in `Root.Runtime` (screens). They are not the
  same thing.
- Browser and terminal clients share one model (the object tree and the same
  commands). There is no web-only model.

## Build and start

The engine is behind the `http` feature:

```bash
cargo build -p lumacore-station --features http
```

`HttpEngine::start()` spawns an OS thread `http-engine` with its own
multi-threaded tokio runtime (2 workers, `http-worker`). HTTP work never
touches the DMX render thread.

| Flag (`lumacore-server`) | Default | Effect |
|---|---|---|
| `--http-port` | `REMOTE_HTTP_PORT` (`lumacore-wire`) | Listen port |
| `--port-base N` | — | HTTP on N+6 (with cmd N+3, shell N+4, stream N+8) |
| `--no-http` | off | No HTTP engine |
| `--web-root` | none (built in) | Serve the WebUI from this folder instead of the copy built into the binary; the scripts pass `crates/ui/lumacore-webui/web` (live edits, decisions §36 2) |
| `--tls-cert`, `--tls-key` | — | Parsed, but not wired up yet (the crate has an optional `tls` feature) |

**Per-NIC access.** The listener drops a connection that arrived on a local
address whose interface does not serve HTTP (`Set Interface "en1" "HTTP" No`,
`ConsoleBridge::http_allows`). Loopback is always served.

## Reading state

Handlers never call the state thread for reads. They read what the state
thread publishes:

- `Arc<ArcSwap<ShowData>>`: the show, replaced after every change.
- `StationView` (menus, station info) and the console bridge's published views.
- `DiagnosticsSnapshot` (`src/state.rs`): thread health, build info, uptime,
  show path and counts, sACN running, sysmon history. The `diagnostics`
  periodic task on the state thread stores a new one every second.
- Engine output (DMX frames, `Runtime.Health` atomics, the render engine's
  per-frame ring `RenderStats::ring`) for the `dmx`, `health` and
  `health.render` feeds on `/ws/feeds`.

## Login

`POST /api/login` with `{user, password?}` returns `{ok, user, token}` and sets
the `lumacore_token` cookie (30 days). A token is sent as that cookie, an
`Authorization: Bearer` header, or `?token=` (`src/auth.rs`). Routes marked
**login** answer 401 without a valid token. `POST /api/logout` revokes the
token. Admin's password is always `admin` (it can't be set, and is always asked for; §30 "12a3"). The shell WebSocket takes `?device=<id>`: the device id (one per app install, or per browser tab) that DeskLock and the normal theme are kept by on the station. Remote logins follow the user's `remote_login` setting; the rights gate
in dispatch decides what a user may run.

## REST routes

A request that needs the state thread waits at most 10 s. When the station is busy and the
request didn't run, the reply is **503** with `{ok: false, message}` carrying E-CMD-017 — try
again (R1-67, Q40). A layout write whose device-lock check can't run is refused with 503 too,
never let through.

| Method | Path | Login | What |
|---|---|---|---|
| GET | `/api/health` | — | `{state: ok\|degraded, threads}` from `Runtime.Health` |
| GET | `/api/metrics` | — | `{threads, runtime: {fixtures, universes, outputs, sessions, protocols_running}}` |
| GET | `/api/info` | — | `{build: {version, profile, target_os, target_arch, git_commit, build_id}, uptime_secs, show_path, …counts, protocols_running}` |
| GET | `/api/station` | — | `{name, id, version, build, client_protocol, pid, exe, started_ms, http_port, shell_port, show, users: [{no, name, rights, color, profile, profile_no, values, selected, has_password, remote_login, logged_in: [station…], look}], profiles: [{no, name, locked, used_by: [user…], screens, views, window_presets, preset, look}], status: {dmx, role, lumasession, unsaved}, busy}` (`locked` = the template Default, decisions §30 "12c2"; `busy`: the state thread has been on one turn of its loop for more than 5 s, R1-67). Open by decision (§30 "12a3"): the lock / login surface draws users, presence and the status line from it before a login (live: the open `station` feed); the app checks the local server's version with it. |
| POST | `/api/unlock` | — | `{device, password}` unlocks a DeskLocked device (the password of the user who locked it; Admin's is always `admin`): `{ok}`, 401 with `{ok:false, message}` |
| POST | `/api/login` | — | See Login |
| POST | `/api/logout` | — | Revoke the token (queued: a busy station revokes it when it gets to it, never drops it), clear the cookie |
| GET | `/api/me` | login | `{user, screen_configs: {default, configs: [{no, name, screens}]}, look: {theme: {preset, adjust, palette}, lock_screen: {image: {sha256, ext, width, height}\|null, show_clock, text, auto_lock_minutes}}}` — the look of the user's profile (§30 "12a2") |
| GET | `/api/ui-keys` | login | The user's UI keys and gesture timing (`double_click_ms`, `hold_ms`, `keys`) |
| GET | `/api/feedback` | login | Notification settings and what is active now |
| GET | `/api/sysmon` | login | `{count, lines}`: System Monitor history |
| GET | `/api/screens` | — | Live screens (summary) |
| POST | `/api/screens` | login | Join `{join}` or the user's screen `{config, screen}`; returns the screen |
| GET | `/api/screens/{no}` | — | Layout and windows (settings resolved with type defaults) |
| PUT | `/api/screens/{no}/layout` | login | `{revision, root, windows}`: replace the structure (own screens only; existing windows keep what the station holds, new ones store only settings that differ from the type's defaults); returns `{revision}`, 409 + `{screen}` on a stale revision, 423 from a DeskLocked device (`X-Luma-Device`) |
| POST | `/api/screenshots?id=&screen=` | login | A page's capture (PNG body, ≤ 64 MB) for a `Screenshot` that asked it; saved under `library/screenshots/<station>/`; `{ok, path}`, 409 when not asked for or not the uploader's screen |
| GET | `/api/media/{sha256}` | — | A picture from the station's store by hash (`Cache-Control: immutable`); open like the other reads, the lock screen shows it before a login; 404 when this station doesn't have it |
| POST | `/api/media?name=` | login | Upload a file (≤ 16 MB) into `library/media/` under a safe name: `{ok, name}`; the client then runs `Import Image "<name>"` (or `Import UserProfile Image …`) like anyone typing it. On a LumaSession member the file also goes to the master (stream `Upload`, ahead of the forwarded line): imports always run on the master |
| GET | `/api/window-types` | — | Built-in window types: settings schemas, window bar (`actions`, `bar_default`, `bar_min`), live `title`, `chips` (decisions §30 "12c") |
| GET | `/api/menu-types` | — | The menu registry (`MenuDef`s, decisions §30 "12d"): static, open like the window types |
| GET | `/api/object-menus` | — | Every object's context menu, from the registry; static and open |
| GET | `/api/tool-types` | — | The editor tools (decisions §30 "12e"); static and open |
| GET | `/api/fixture-types?drive=` | login | The show's fixture types and the library's `.gdtf` files (a drive's folder with `drive`) |
| GET | `/api/fixture-type?…`, `/api/fixture-type/thumbnail?…` | login | One type's facts (modes, from 1) and its thumbnail |
| POST | `/api/gdtf?name=` | login | Upload a `.gdtf` into the library (body limit = the media limit) |
| POST | `/api/upload?…` | login | The one upload route for every file dialog ("This device", decisions §38 5; `files::MAX_BYTES`) |
| GET | `/api/shows` | — | The `shows` feed's build, one-shot (the Backup menu uses the feed): `{current, facts, kinds, shows: [{name, saved, size_text, format_text, by, station, versions, loaded, …}], history_of, history, recovery, trash, usage, disk, rules}` |
| GET | `/api/show-parts?show=x[&version=<stamp>\|&recovery=<copy>]` | — | The Load show dialog (one-shot, from the file's manifest): `{show, facts, loads, refused, parts: [{key, label, sum, objects: [{ref, label, kind}]}]}` |
| GET | `/api/network` | login | Menu → Network / DMX Protocols, one-shot (the menus use the `network` feed): interfaces and services, networks and this station's map, lines with routes and live state, universes, input sources, Art-Net nodes |
| GET | `/api/lumasessions` | login | Menu → LumaSessions, one-shot (the menu uses the `lumasessions` feed; the surface's Servers… list reads it once): this station's settings and status, every station discovery sees, members |
| GET | `/api/views` | login | The user's stored views and view buttons `{views, buttons}` |
| POST | `/api/views/{no}/recall` | login | `{screen}`: recall onto a screen |
| POST | `/api/views/{no}/label` | login | `{name}`: rename |
| DELETE | `/api/views/{no}` | login | Delete the view |
| PUT | `/api/view-buttons/{slot}` | login | `{view}`: assign (0 clears) |
| GET | `/api/pools/{pool}?datapool=n` | login | A pool window's content (`{pool, window, actions, datapool, cells}`); `datapool` 0 or absent = the user's |
| GET | `/api/pools/{pool}/{no}` | login | One pool object's editor content (Group: members and grid), one-shot; the editor uses the `pool_editor` feed |

The view routes that run a command name the page's device with an `X-Luma-Device: <id>` header (or
`?device=<id>`); a locked device's lines are refused like its shell's (E-LOCK-001).

The menu, show and pool routes are read-only. Save, Load, network edits and
pool gestures run as commands over `/ws/shell` so they echo in the command line.
The view routes run the same commands the terminal uses (`Store View`, `View`,
`Delete View`, `Assign View … At ViewButton …`).

## No SSE, no poll

`/api/events` and `/api/station/events` (SSE) are gone since 12d (decisions
§30 "12d hardening": "Everything live on `/ws/feeds`"): screen events are the
`screen` feed, the station view the open `station` feed. The page polls
nothing — fetch is for one-shot loads (window types, UI keys, login, info,
`/api/me`, a screen after its `screen_changed`). Tests hold it:
`the_router_has_only_the_feeds_and_shell_sockets_and_no_sse` (lumacore-http)
and tidy `frontend_has_no_event_stream_or_fetch_poll`.

## WebSockets

| Path | Login | What |
|---|---|---|
| `/ws/shell` | token in hello | Bridge to the TCP shell port (see below) |
| `/ws/feeds` | opens without; `auth` in band | Everything live in a page, as subscriptions (see below) |

Two sockets only (decisions §30 "12b2 hardening": one feed rule): a new feed
is a feed on `/ws/feeds`, never a new socket or stream. Tests hold it on both
sides (`the_router_has_only_the_feeds_and_shell_sockets_and_no_sse` in
lumacore-http, tidy `frontend_feeds_ride_one_socket`).

`websocket/state_sync.rs` (`/ws/state`) and `websocket/preview.rs`
(`/ws/preview`) are stubs and not routed.

### `/ws/feeds`

One socket per page (`src/websocket/feeds.rs`, shared builds in
`src/websocket/hub.rs`; the page side is the kit's `FeedClient`,
`web/kit/feeds.js`). **Every feed sends one batch when it is subscribed
(the state, or the history), then changes in batches per period — never one
item per message.** Batches due in one tick go as **one frame** (a JSON
array of messages); the page takes it apart.

**Handshake and login** (decisions §30 "12d hardening"). The protocol number is
`CLIENT_PROTOCOL` in `lumacore-wire`; it is bumped on any wire change, with no shims. The
socket opens without a login. The station's first message:

```json
{"op":"hello","client_protocol":<CLIENT_PROTOCOL>,"epoch":"…","user":null,
 "feeds":[{"name":"station","kind":"state","open":true,"params":[]},…],"max_subs":64}
```

`feeds` is the catalog's feeds (lumacore-http registers its `FEEDS` rows, decisions §34 21). A page that sees another
`client_protocol` than when it loaded reloads (the station was updated).
Feeds marked `open` (only `station`: users, presence, show status — the lock
screen) work before a login. A login token on the upgrade (cookie, bearer,
`?token=`) logs the socket in at once (`user` in the hello); in band:

- `{"op":"auth","token":"v2.…"}` → `{"op":"authed","user":"Bob"}`; every
  subscription stays (another user's feeds start over as theirs). A token the
  station refuses: `{"op":"error","code":"E-FEED-006",…}`, nothing changes.
- `{"op":"auth","token":null}` (logout) → each feed that needs a login ends
  with `nosub` E-FEED-002, then `{"op":"authed","user":null}`. A token revoked
  elsewhere (`POST /api/logout`) does the same at the station's next counter
  change. The FeedClient keeps such subscriptions and sends them again after
  the next `authed`.

**Subscriptions.** The page sends (text):

- `{"op":"sub","id":3,"feed":"health.render","period_ms":16,"since":90400,"epoch":"…","rev":…}`
  — subscribe (an `id` in use is replaced: new parameters). `period_ms` is the
  send period, clamped to ≥ 16 ms (default per feed); it never changes the
  sample rate. `since` + `epoch` resume a samples feed after the last `seq`
  the page has (ignored when `epoch` isn't this server run's); `rev` is the
  revision of a state the page has.
- `{"op":"unsub","id":3}`.

Every `sub` is answered `{"op":"subbed","id","feed","kind"}` or
`{"op":"nosub","id","code","message"}`:

| Code | When |
|---|---|
| E-FEED-001 | No such feed |
| E-FEED-002 | The feed needs a login (or the login ended) |
| E-FEED-003 | Parameters missing or wrong (a pool, an object `no`; an op without `id`) |
| E-FEED-004 | More than 64 subscriptions on the socket |
| E-FEED-005 | (close reason, code 4008) the page doesn't keep up |
| E-FEED-006 | `auth` with a token the station refuses (`op: error`) |
| E-FEED-007 | Not JSON, or an op other than `auth` / `sub` / `unsub` (`op: error`) |

`Info Error E-FEED-…` explains each (`lumacore_http::errors`).

**Batches.** Every batch carries one envelope:
`{feed, kind: "state"|"samples", id, epoch, from_seq, to_seq, gap, …}`.
`epoch` is this server run's id (a restart changes it: the page starts over).

- **Samples** (`health.render`, `sysmon`, `feedback`, `screen`): every item has
  a `seq`. A first batch after a `since` says `recovered: true`, or
  `recovered: false` when the station no longer holds it (then the whole ring
  follows). Items the station dropped before a page got them arrive as
  `gap: {from, to}` — never a silent loss.
- **State, counter-driven** (`station`, `network`, `lumasessions`, `shows`,
  `pools`, `pool_editor`, `attributes`, `detail`, `windows`): **shared builds** — one build per
  (feed, user, params), only while someone subscribes it, rebuilt when the
  station's own counters move (`lumacore_http::Revisions`: the published
  show, the station view, the show status, claims, the menu views, the show
  library on disk, the screen events; never a polling timer), serialized once
  and framed for every subscriber. Each batch carries `rev`, a 53-bit
  fingerprint of the data (the same state has the same rev — across
  reconnects and restarts). A `sub` with the page's `rev` gets
  `{…, "rev", "unchanged": true}` and no data; otherwise the whole `data`.
  Small states are replaced whole; the keyed lists (`shows` by `name`,
  `pools` cells by `no`, `pool_editor` members by `fixture`) send
  `"base": <rev>, "diff": {list, key, fields, set: {key: row}, del: [key],
  order?}` when the page has `base` and the diff is under half the data
  (`order` only when the rows' order isn't the old one with deletes out and
  new rows appended). The FeedClient puts it together (`applyDiff`): a
  window's `batch` always gets the whole `data`; a diff on another base makes
  it subscribe again without `rev`.
- **State, sampled** (`health`, `dmx`): read each period, sent when changed.

**Limits** (Home Assistant's): the station writes through one queue per
socket — 256 messages fill it, or 64 waiting for 5 s, and the socket closes
with code 4008 (reason E-FEED-005); while the queue is behind, state batches
wait (only their newest goes later); a send stalled for 2 s closes it. The
station pings every 15 s and drops a page it heard nothing from for 45 s. The
page reconnects with a jittered backoff (0.5–8 s) and resumes: samples by
`since`, states by `rev`.

| Feed | Kind | Open | Default period | Payload |
|---|---|---|---|---|
| `health.render` | samples | | 50 ms (the Realtime window asks 16) | Every render frame from the engine's ring (`FRAME_RING` = 640, ~10 s): `stages: [names]`, `cols: {seq, t_ms, frame_us, headroom_us, jitter_us, late, stage_us: [[…] per stage]}`. Columnar JSON. |
| `health` | state (sampled) | | 250 ms | `data`: `Runtime.Health` (render totals, lines' send numbers, the session clock — all members' on the master, PTP when linuxptp runs — threads); sent when it changes |
| `sysmon` | samples | | 50 ms | On subscribe the station's whole System Monitor history (5000 lines) in one batch, then new lines: `cols: {seq, time_ms, elapsed_ms, level, subsystem, message}` |
| `feedback` | samples | | 50 ms | `items`: the user's feedback lines (station-wide and their own; progress, flashes, messages, statuses — sysmon lines with a `feedback`); on subscribe what is active now (and, with `since`, the messages missed) |
| `dmx` | state (sampled) | | 16 ms | Subscribe with `universes: [1, …]` (or `universe`) and `follow_selection`. JSON `data: {universes: [all], patches: [{universe, exists, spans}]}` when the show changes; `follow: <universe>` when Only Selection moves; **binary frames** with the values of the subscribed universes that changed (idle universes send nothing) |
| `pools` | state | | 100 ms | Subscribe with `pool` ("Group" or a window type) and `datapool` (null / 0 / "Selected" = the user's selected one, a number, or a datapool's GUID). `data`: `{pool, window, actions, datapool, moving, cells}` (or `{missing: <pool>}` when the pool isn't there, `{missing: <pool>, datapool_missing: true}` when the pinned DataPool is gone — never the user's selected one instead, R1-55); `moving` (a group master fading) makes the build run again every 100 ms until it rests |
| `windows` | state | | 100 ms | Subscribe with `screen` (the page's live screen number). `data`: `{screen, maximized, windows: {"<no>": {type, title, bar, items, states, chips, values, picks, keys, preset}}}` — each window's live title, `WindowBar` / `WindowBarItems`, every Bool setting's and stateful action's lit state, the chips that show (`{id, label, line}`), every setting's value as shown, `picks` (the pick list of each pinned link setting, e.g. DataPool, Universe), the LEDs of the profile's keys that read `Self` and the preset GUID it came from; resolved by the station through `Self.<Property>` (decisions §30 "12c", `lumacore_views::window_state`). Another user's screen is `{missing}`. Rebuilt on show and screen changes |
| `station` | state | yes | 100 ms | `GET /api/station`, live: users with presence and look, profiles, the status line — the lock / login surface and Users & Profiles |
| `network` | state | | 100 ms | `GET /api/network`, live (Menu → Network, DMX Protocols). The state thread builds this view (and lists the NICs for it) only while a page subscribes it |
| `lumasessions` | state | | 100 ms | `GET /api/lumasessions`, live (Menu → LumaSessions); built only while subscribed |
| `shows` | state | `history`, `drive` | 100 ms | `GET /api/shows`, live (Backup): the show's status and facts, the library (a drive's with `drive`), the history of `history` (default the loaded show), recovery copies, use and free disk; re-read when a file is written through the station (`drive::writes`) |
| `files` | state | `drive`, `folder`, `kind` | 250 ms | A drive's library folder (decisions §38 5): `files` keyed by `name` — folders first, then the kind's files (shows, gdtf, mvr, media, userprofiles) with size and modified; rebuilt when the drives or the library's writes move |
| `screen` | samples | | 16 ms | Subscribe with `screen` (the page's live screen; 0 = none). `items`: `{event: "views_changed"}`, `{event: "screen_changed", no, revision}`, `{event: "reload_ui", screens: [n]\|null}`, `{event: "capture", id, screens}` (`Screenshot`: a page showing one of `screens` renders itself and uploads to `POST /api/screenshots?id=…&screen=<n>`), each with its `seq` (the station keeps 256). A fresh subscription starts at the newest (an old ReloadUI never runs again); a `since` resumes. **The subscription is the screen's presence**: while it stands the page counts as a viewer of `screen` — who a `Screenshot` asks (decisions §29 "11d"); `subbed` comes after the station registered it |
| `pool_editor` | state | | 100 ms | Subscribe with `pool`, `no` (and `datapool`). `data`: one pool object's editor content (Group: `{no, name, mode, preserve_grid, members: [{fixture, …}], …}`) or `{missing}` (the editor closes) |
| `attributes` | state | | 100 ms | An attribute board (decisions §30 "12f-a"): the show's feature groups, or a profile's own with `profile` (its number). `data`: `{target, title, owner, locked, template, templates, groups: [{no, name, color, icon, members}], attrs: [{name, new, used, how, why, file, main, show_group, types, readout, resolution, …}], types: [{name, attrs}]}` or `{missing}`. Opened by the `attribute_board` UI event (`Edit [UserProfile] Attribute`) |
| `detail` | state | | 100 ms | A Users & Profiles detail (decisions §30 "Users & Profiles follows storage"): `kind` `profile` or `user` and its `no`. `data`: a profile's `keys`, `window_presets`, `images`, `icons`, `screens`, `waiting`, `encoder_bars`, or a user's `work` and `usage`; `{missing}` when it is gone |
| `encoder` | state | | 50 ms | A live screen's encoder bar (decisions §30 "12f"), param `screen` (the live screen). `data`: `on`, `mode` + `modes` (word, label, planned), `layer` + `layers`, `group`, `page`; Attributes: `layout`, `strip` (no, name, color, icon, programmed, mini levels), `pages`, `feature`, `selection`, four `encoders` (attr, pretty, readout, value, levels, programmed, fade / delay `{text, value}`, resolution, `lines` value / fade / delay with `{value}`) or null; Masters: `masters` (label, sub, line, value); Cue timing / Effects: `planned`; `{missing}` when the screen is gone |
| `oops` | state | | 100 ms | Your Oops list (decisions §30 "12f"): `steps` oldest first (`line`, `category`, `station`, `time_ms`, `main`; the newest 200), `undone` (next redo first), your `profile` number, `locked`, `popup_dim`, the Oops `filters` (`name`, `label`, `on`) |
| `tags` | state | `tag` (a GUID, empty = none picked), `deep` | 100 ms | The Tags pool window (decisions §12 "Tags hardened"): `nodes` (`path`, `guid`, `name`, `colour`, `count`, `badge`), `deep`, the picked `tag` (`path`, `guid`, `name`, `own_action`, `lines`: `select`, `settings`, `label`, `child`, `top`, `delete`), its `members` (`guid`, `kind`, `no`, `name`, `target`, `via` = `direct` or `from <tag>`), `kinds`, the grid's `columns`; `missing` when the picked tag is gone |
| `patch` | state | — | 100 ms | The Patch setup dialog (decisions §34 "34g4 patch hardened" 9, 12): `mode` (`draft` for the holder of the Devices draft, else `live`), `holder`; `fixtures` (`no`, `guid`, `name`, `note`, `fixture_type`, `mode`, `address`, `pos_x`…`rot_z`, `master_react`, `tags`, `group`, `parent`, `class`), `unsorted`, the MVR `tree` (`kind` layer / group, `no`, `guid`, `name`, `parent`, `count`), `classes`, `universes`, `fixture_types` |
| `sequence` | state | `sequence` (`"selected"` or a GUID) | 100 ms | A sequence sheet (decisions §30 "12f" sequence window A): `sequence`, `datapool`, `guid`, `name`, `running`, `current`, `select`, `feature_groups` (`name`, `word`, `color`), `paths`, `cues` (CueZero first, OffCue last: `cue`, `key`, `name`, `trig`, `fade` / `outfade` / `delay` / `outdelay`, `values`, `current`, `run`, `fixed`, `line`, `groups` — per feature group its own `fade` / `delay` / `outfade` / `outdelay` / `path`, null = the cue's); `{missing, note}` when there is none |

The DMX binary frame (little-endian, every array on an even offset):
`u32 id` (the subscription), `u32 seq`, `f64 time_ms` (Unix), `u16 count`,
`u16 0`, then per universe `u32 universe`, `u8 flags` (1 = has input),
3 × `u8 0`, `values[512]`, `ours[512]`, `winners[512]` (0 unpatched,
1 default, 2 ours, 3 park, 4 input), `input[512]`, `lines: u16[512]` (the
input line that won the channel) — 3080 bytes per universe after the 20-byte
header.

The FeedClient (`sub(feed, params, {batch, binary, reset, subbed, error})`,
`auth(token)`) reconnects with a jittered backoff while anything is
subscribed, sends `auth` before its subscriptions, resumes with `since` /
`rev`, checks the epoch, puts keyed diffs together, marks gaps (NaN rows in
its typed-array `SampleRing`), drops fast feeds to 250 ms while the page is
hidden (`visibilitychange`; still every sample, fewer messages), and leaves
drawing to each window's own `requestAnimationFrame` loop (`drawLoop`).

### `/ws/shell`

A window's own lines (its bar, its window menu, its settings window) send
`{"type":"command","cmd":"Set Self Universe + 1","window":4}` and a key in a
window bar `{"type":"key_action",…,"window":4}`: `Self` in the line is that
window of the page's screen (the shell packets `Command.window` /
`KeyAction.window`). Without `window`, `Self` is the
focused window (`{"type":"focus","window":n}`); with neither the line answers
E-CMD-016.

A proxy to `localhost:<shell port>` that speaks the `lumacore_wire` shell
packets (JSON + newline), the same types the CLI uses. The login travels in
the hello: the token (cookie or `?token=`), or with no token `Admin` without a
password (accepted only while Admin has none).

- **To the browser.** Binary frames: 4-byte little-endian viewport id
  (0 = untagged), then ANSI for that xterm.js. Text frames: JSON control
  messages `{"type":"ui","event":…}` and `{"type":"status","status":…}`.
- **From the browser** (text): `key` (DOM KeyboardEvent), `key_hold` /
  `key_up` (a keyboard key held past the profile's hold time / released, with
  `held`: the bound UI key's Hold / Release / ReleaseAfterHold, §17, §29),
  `focus`, `live_screen`, `tap` (view button, pool button), `command` (a UI
  line, echoed like typed input), `key_action` (UI key gesture with Meow),
  `gesture` (`{id, line, phase: "move"|"end", window?, label?, from?, value?}`
  — a fader's or encoder's line, decisions §30 "12f": run like a typed line but
  quietly — no echo, history, Usage or journal per move; while the station is
  busy only the newest move of a gesture waits, an end never drops; the end files
  one journal entry and one System Monitor line under "Faders", e.g. "Fader
  Master 2.1: 100 → 63.5 · Admin"),
  `dialog_answer` (`{id, button, inputs, toggles, selectors}` — the answer to
  one of the station's dialogs, decisions §30 "12b"), `dialog_pause` (`{id}`,
  a touch on a countdown), `resize`, `viewports` (terminal windows, focused
  first). Anything else is raw xterm input (paste).
- **Dialogs in the status**: `status.dialogs` — the
  open questions for this client (title, text, rows, details, inputs with
  their kind: text / password / number / link with its pick list, toggles,
  selectors, buttons with kind and disabled reason, primary, cancel,
  countdown `{deadline_ms, total_s, default, paused}`, topic) — and
  `status.dialogs_closed` (`{id, title, by, answer}`, a few seconds).
- **12c2**: `status.hold_follows` (the profile's
  `HoldFollows`: a held pane is maximised on the shared screen with
  `Set Self Maximized On`, else only on the device) and
  `status.device.screen_config` (`Set Device "ScreenConfig" <n>|Follow`; absent
  = follow the user). The live screen JSON and the `windows` feed carry
  `maximized` (the window held full screen, or none). A deleted user's shells
  get `{"type":"ui","event":{"LoggedOut":…}}` and close.

## Static files

The engine serves the WebUI (`lumacore-webui`, decisions §36 2): the files
built into the binary, or the `--web-root` folder when it exists (`ServeDir`,
directories serve `index.html`). A path without a file extension falls back to
`index.html` (page routing); an unknown `/api/` or `/ws/` path stays 404 on the
built-in files. Every WebUI response has `Cache-Control: no-store` so WKWebView
never shows a stale build. CORS allows any origin; responses are compressed.

## Perfetto engine

A separate engine, feature `perfetto`, on its own port (`REMOTE_PERFETTO_PORT` in
`lumacore-wire`). Thread `perfetto-engine`, single-threaded tokio runtime. Source:
`crates/engine/lumacore-engines/src/perfetto.rs` (the routes) and
`crates/ui/lumacore-http/src/endpoints/trace.rs` (the handlers); the collector
(`TRACE_COLLECTOR`) is in `crates/foundation/lumacore-trace/src/`.

| Method | Path | What |
|---|---|---|
| GET | `/api/trace?seconds=N` | Chrome Trace Event JSON (last N seconds if given) |
| GET | `/api/trace/download?seconds=N` | The same as a `trace.json` download for ui.perfetto.dev |

See [profiling.md](../build/profiling.md).

## Tests

`crates/ui/lumacore-http/src/tests/` (endpoints, engine, state, static files,
WebSocket): `cargo test -p lumacore-http`. Browser flows:
[testing.md](../testing.md) (Playwright).
