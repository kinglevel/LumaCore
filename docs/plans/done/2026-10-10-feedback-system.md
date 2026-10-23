> **Closed 2026-10-20:** built — the one feedback system (progress, flash, cards, status on the System Monitor; roadmap Done, commit `6f25903`)
> Leftovers live in `docs/meta/roadmap.md` → Next "Feedback system leftovers" (Message Center window, envelope icon, audience client, CLI feedback lines, the progress API).
> Kept as history: the paths below predate the crate split (§34) and the WebUI crate (§36).

# Handoff — one feedback system: progress, notifications, flashes, statuses (2026-10-10)

Branch: `termiham` (now `main`). Designed with the user; first step built (see Status).

> **Audit 2026-10-11 (step 11a):** branch is now `main`; code paths moved in step 8 (`apps/lumacore-station/src/{core,terminal,net,lumasession,server}/`, clients in `apps/`). What's still open from this handoff is listed in [docs/meta/roadmap.md](../../meta/roadmap.md) → Next.

Research behind it: a study of how an established console shows save progress, "Show saved",
notifications and a plugin progress API (not kept in the tree).

## Why

Feedback is scattered today: a top-centre `Notice` UI event (Quick Save "Show … saved"), `Cmd`
sysmon lines, terminal output, the command line status snapshot, the reconnect overlay. A reference console has
four unrelated systems (native progress overlay, Message Center notifications, the bottom flash,
the Status Center). LumaCore unifies them: **one event stream, the System Monitor as its
backbone, several surfaces**.

## Model

```
FeedbackEvent {
  id, seq, time,
  kind:       progress | flash | message | status,
  importance: critical | important | normal | info,     // set by the emitter, never by users
  level:      info | warn | error | alert,               // colour / icon (priorities)
  category:   system | cmd | power | network | usb | chat | plugin,
  source:     station, user?, client?, plugin?,
  title, text,
  progress:   { range: Option<(from, to)>, value } ,     // no range = open-ended counter
  audience:   station | user | client,
  lifetime:   timed(ms) | permanent | until_stopped,
}
```

- **Every event is a System Monitor line** (structured sysmon line + the feedback fields), so the
  log, `/ws/monitor` and the log file carry everything; nothing is only visual.
- **Kinds**
  - *progress* — lifecycle start → update (value, range, text) → stop; with a range it fills,
    without one it is a counter (SaveShow: bytes written); several at once, stacked.
  - *flash* — a short confirmation ("Show saved"), ~1 s.
  - *message* — kept, has level + category; shown per the notification settings.
  - *status* — an ongoing on/off state (HI, SOLO, PARK, sACN, claims, …); the command line icons.
- **Importance = the safety floor** (users choose *how*, never *whether*, for important events):

  | Importance | Examples | Audience | User may change |
  |---|---|---|---|
  | critical | power loss, server shutting down / restarting, disk full, save failed | station | nothing — always a pop-up |
  | important | Saving show (progress), Show saved / loaded / new, CleanStart, station joined / lost, thin client / user connected / disconnected | station | card vs pop-up, timed vs permanent — never off |
  | normal | warnings / errors of commands, plugin messages | per category settings | anything, incl. off |
  | info | own command results, routine logs | sysmon (+ own command line) | — |

  Plugins may not raise *critical*.

## Surfaces (every screen of every UI; terminals print one line)

- **Progress** — top-centre pill, translucent, stacked; `title : text`; fill when ranged.
- **Cards** — top-right stack, 300 px; level colour + icon; timed (5 s)
  or permanent (close button).
- **Flash** — bottom band, full width, green outline + text (~1 s, fades).
- **Pop-up** — centred modal (OK / Confirm).
- **Status** — command line bar icons (today's status snapshot becomes status events).
- Layering: notifications above windows and screen overlays, below modals.

## Settings (until the Message Center window)

Per user profile: defaults per category × level; `Set Notifications <category> <level>
None|Card|Popup`, `Set NotificationMode Timed|Permanent`. Important events clamp: `None` is
ignored for them. The Message Center window (later) edits the same settings and lists messages.

## API (one for core, commands and plugins)

- Rust: `feedback::progress_start(title) -> ProgressId`, `progress_range`, `progress_set`,
  `progress_inc`, `progress_text`, `progress_stop`; `feedback::flash(text, level)`;
  `feedback::message(level, category, importance, text)`; `feedback::status(key, on, …)`.
  Progress handles stop on drop (no orphan bars).
- Plugins later: start / range / set / increment / text / stop progress calls map 1:1.
- Transport: the feedback events ride the existing feeds (sysmon structured line → `/ws/monitor`
  for windows; UI events to clients for surfaces); one renderer per surface in the web UI.

## Migration (agreed scope of the first step)

1. **Backbone**: `FeedbackEvent` on top of structured sysmon; station / user / client audience
   routing; importance floor; profile notification settings + commands.
2. **Shows**: SaveShow / LoadShow / NewShow / CleanStart — progress pill (byte counter for saves,
   written in chunks) + important flash ("Show saved", "Show loaded", …); replaces today's
   `Notice` and the Quick Save notice.
3. **Stations + clients**: station joined / lost (session discovery), thin client / user
   connected / disconnected → important cards for everyone.
4. **Shutdown / Restart**: critical pop-up to everyone (with the countdown), tied to "Server
   restarting…" and the reconnect overlay.
5. **Status icons**: the command line icons produced as status events instead of the per-client
   status snapshot.

Not in this step: the Message Center window (message list, confirm, settings grid), the
envelope icon count, plugin API bindings.

## Status (first step built; unit + e2e tests pass)

- `apps/lumacore-station/src/feedback.rs`: `Feedback {id, kind, importance, category, title, text,
  user, progress, key, glyph, active}`, `Progress` (start / range / set / inc / text / update /
  stop, throttled 50 ms, stops on drop), `flash`, `message`, `status` (changes only),
  `publish_statuses` (HI, SOLO, BO, PARK, sACN, WATCH, DEBUG from the server loop),
  `ClientPresence` (connected / disconnected per user + address, 3 s grace), `station_joined /
  lost`, notification defaults + `settings_json`, active registry for late joiners.
- Sysmon: `SysmonLine.feedback`, transient lines (progress ticks: live only, not in the ring or
  the log file), `Level::Alert`; `set_echo` for `--RootMonitor`; the TCP monitor now subscribes to
  the ring (history + live) — `ServerContext.monitor_senders` removed.
- HTTP: `/ws/feedback` (login; active state first, then feedback lines for the user — station-wide
  or theirs), `GET /api/feedback` (settings + active).
- Settings: `UserPreferences.notifications` ("category.level" → none / card / popup) and
  `notification_mode`; `Set Notifications <category> <level> None|Card|Popup|Default`,
  `Set NotificationMode Timed|Permanent` (UI reloads via `UiEvent::FeedbackSettings`).
- Migrated: SaveShow (chunked write via temp file + rename, "Saving showfile <name> : <MB>" counter,
  "Show saved" flash, failed save = critical), LoadShow (progress, "Show loaded", failure =
  important error), NewShow ("New show"), CleanStart (important warning), Shutdown / Restart
  (critical alert to everyone; 400 ms before exit), station joined / lost, user connected /
  disconnected, station status icons (removed from the per-client status snapshot), the Quick
  Save notice (gone; key-action errors and local notices are cards now).
- Web UI: `#fb-progress` (stacked pills, fill when ranged, ≥ 600 ms visible), `#fb-flash`
  (bottom band), `#fb-cards` (top-right, timed / permanent, close), `#fb-popup` (queued), status
  icons from status events; the reconnect overlay says "Server restarting — reconnecting…".
- Tests: `feedback.rs` unit tests; e2e `feedback.spec.ts` (8) with two users in two browsers.

### Not done yet
- Terminal (lumacore-cli) clients don't print feedback lines (they see them in the System Monitor).
- Message Center window, envelope count, plugin API bindings (the progress calls).
- Audience `client` (only station / user so far).
