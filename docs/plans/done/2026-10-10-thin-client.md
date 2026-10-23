> **Closed 2026-10-20:** built — the thin client (users, passwords, tokens, the Connect page, one OS window per screen; roadmap Done, commit `6f25903`), carried on by 12a3's basic client and §36's WebUI crate.
> Leftovers live in `docs/meta/roadmap.md` → Next "Thin client leftovers" (HttpOnly cookie, per-user filtering, the macOS bundle, Connect completion, the GUI pass).
> Kept as history: the paths below predate the crate split (§34) and `crates/ui/lumacore-webui` (§36).

# Handoff — thin client, users, screen configs (2026-10-10)

Branch: `termiham` (now `main`). Plan agreed with the user; phase 1 started in this session.

> **Audit 2026-10-11 (step 11a):** branch is now `main`; code paths moved in step 8 (`apps/lumacore-station/src/{core,terminal,net,lumasession,server}/`, clients in `apps/`). What's still open from this handoff is listed in [docs/meta/roadmap.md](../../meta/roadmap.md) → Next.


## Goal

The app becomes a **launcher + thin client** that can connect / log in to any LumaCore server
and log out to pick another, so protocol and multi-client issues surface early. This is a
deliberate LumaCore divergence — established consoles do not connect to external servers (only a
session join at station level, or a web remote).

## LumaCore today (before this work)

- `lumacore-app` runs the server in-process (`server::start_server_in_thread`) and loads
  `http://localhost:47656`; frontend hardcodes `ws://localhost:47656/ws/shell`; WS bridge logs
  everyone in as `Admin`.
- Multicast discovery beacon `239.255.76.67:47655` every 750 ms (`SessionDiscoveryWire`: station,
  authority, cmd/shell ports, sessions, engines, build) — no HTTP port. No mDNS.
- `lumacore-cli` already starts *Unconnected*, `CMDLINE <ip>` connects, `Logout` returns.
- `UserProfile` = identity + data (`role`), no passwords; `Login "name"` auto-creates profiles.
- Shell sessions are per user (`user_to_session`); per-client screen state in `ClientView`.
- Live windows in `Root.Runtime` screens — **not saved**; only stored views are in the show.

## Decisions

| Topic | Decision |
|---|---|
| Local server | App spawns `lumacore-server` as a **child process**; keeps running on logout, stops on quit (prompt) |
| Shutdown / Restart from a client | Acts on the connected server; **extra warning when it is not the local server**; client returns to Connect, auto-reconnects after Restart |
| Discovery | **Both**: extend the multicast beacon with HTTP/UI info **and** mDNS `_lumacore._tcp`. Native side browses, hands the list to the Connect page |
| Connect page | Bundled with the app: discovered servers, manual address, *Start local server*, user + optional password. Auto-reconnect to last server + user. **One connection per window** (File › New Window) |
| Keywords | `Connect "Station"` / `Connect IP 10.0.0.5 [User "LD" ["pw"]]`, `Disconnect` (= `Logout`): client-side, via a UI event. `Login "user" ["pw"]` = switch user **on the same server** |
| Login | User + optional password; **no auto-create** (Admin creates users); passwords **masked** in echo / history / session log; bare `Login "LD"` for a user with a password prompts in a hidden field |
| Users vs profiles | **Split**: `ShowData.users` {name, password hash, rights, remote login allowed, profile, default screen config} → shared `UserProfiles` |
| Screen configs | Show data on the profile, **any number of screens** (MacBook: 1, Workstation: 5). View-session changes write through; restored on start / load / login. Editing / switching configs marks the show unsaved |
| Which config a machine uses | Remembered by the **client app, per server + user**; first connect uses the user's default |
| Screens in the client | **One OS window per screen**; window monitor / position / size / maximized remembered locally |
| Show / hide | Which screens are open, ◀ ▶ and bar toggles, scroll positions: **per machine, per screen**, never in the show — replaces today's browser localStorage |
| Commands | `ScreenConfig n` (switch this client), `Store ScreenConfig n "name"`, `Label/Delete ScreenConfig n`, `Store Screen [n]`, `Delete Screen n` |

## Phases

1. **Show-data foundation**
   - `ShowData.users` + migration from existing `UserProfile`s (one user each, same name/role).
   - Password hashing; `Login "user" ["pw"]` without auto-create; masking; admin user commands.
   - `UserProfile.screen_configs` {no → {name, screens: no → ScreenContent}}; screen ↔
     (user, config, screen); write-through + dirty; restore on start / load / login.
   - `ScreenConfig` / `Screen` commands.
2. **Server side of thin clients**
   - Beacon: HTTP port, UI engine, station id; mDNS `_lumacore._tcp` advertise (TXT: station,
     http port, version, build).
   - `GET /api/station`, login user list (`has_password`, remote login allowed).
   - WS hello with user + password → session token (for auto-reconnect); reject bad logins.
   - Frontend WS URL from `location.host`.
3. **Thin-client app**
   - Spawn / stop local `lumacore-server`; Connect page (custom protocol); native beacon + mDNS
     browser → page via IPC; login; navigate to the server's UI.
   - One window per screen of the chosen config; local display positions; File › New Window.
   - `Connect` / `Disconnect` UI events; auto-reconnect; remote-shutdown warning.

## Open / later

- View buttons are per screen on some consoles; LumaCore keeps view buttons per user.
- TLS / auth for the HTTP engine beyond the login token.
- Phones: a "Phone" screen config is just a config with one small screen.

## Phase 1 status (built, compiles; tests not run)

- **Users** — `ShowData.users` (`lumacore-tree` `User`), resolvers `user`, `profile_id_for_user`,
  `profile_for_user(_mut)`, `role_for_user`, `ensure_users` (migration on load / new show /
  `ensure_admin_user`). All name-based profile lookups in `lumacore` go through the resolvers.
- **Passwords** — `terminal/passwords.rs`: PBKDF2-HMAC-SHA256 (100k, random salt);
  `mask_secrets` masks `Login … "pw"`, `Connect … User … "pw"`, `Password "pw"` /
  `/Password "pw"` in echo, history and session log.
- **Login** — `Login [User] "name" ["pw"]`: existing users only, password checked; UI client
  without a password gets a `password_prompt` UI event (hidden field). In a remote shell the
  client is moved to the user's shell session (`ClientView.switch_user` →
  `SessionManager::move_client_to_user`) and gets a `user_changed` UI event; the browser rejoins
  as that user. Local REPL switches `state.user`.
- **User commands** — `terminal/commands/users.rs`: `User` (list), `Store User "n" [/Role]
  [/Profile] [/Password]`, `Set User "n" Password|Role|RemoteLogin|Profile|ScreenConfig v`
  (own password allowed), `Delete User "n"`; Admin only; last Admin protected.
- **Screen configs** — `UserProfile.screen_configs`; screens carry `(config, screen)`;
  `ScreenStore::{open_screen, take_changed, restore}` + `default_layout()`;
  `terminal/commands/screens.rs`: `sync` (server loop + before every save; marks unsaved when a
  saved layout really changed), `saved`, `restore_all` (show load / new show), commands
  `ScreenConfig [n]`, `Store/Label/Delete ScreenConfig n`, `Store Screen [n]`, `Delete Screen n`.
  `POST /api/screens {user, config?, screen?, join?}` opens a screen from the show; the
  browser sends `?screen=` (default 1) and its config choice (localStorage per user, until the
  thin client keeps it per server + user); `screen_config` UI event switches it.

### Known gaps (phase 2 fixes most)
- The WS hello still logs in as `Admin`: after a reconnect the server side is Admin again while the
  page thinks it is the logged-in user. HTTP endpoints take `user` unauthenticated
  (`/api/screens`, `/api/views?user=`).
- Users are not browsable in the object tree yet (`Root.ShowData.Users`).
- `Root.Runtime.Sessions.<n>` is still keyed by profile number.
- Multiple screens of a config open only via `?screen=n` in a browser — one OS window per screen
  comes with the thin-client app (phase 3).

## Phase 2 status (built, compiles; tests not run)

- **Station identity** — `library/station_settings.json` gains `station_id` (stable uuid) and
  `token_secret` (created on first start).
- **Login tokens** — `terminal/passwords.rs`: `issue_token` / `verify_token` /
  `authenticate(user, password, remote)`. Token `v1.<user hex>.<expiry>.<pw fingerprint>.<hmac>`,
  HMAC-SHA256 with the station secret: survives restarts, expires after 30 days, revoked by a
  password change. Remote logins need `User.remote_login`.
- **Shell hello** — `ShellClientPacket::Hello {user, width, height, password?, token?,
  client_addr?}`; the server authenticates (token, else user + password) and refuses with an
  `Error` packet. `lumacore-cli --password` (or `$LUMACORE_PASSWORD`).
- **HTTP** — `auth.rs`: token from the `lumacore_token` cookie, `Authorization: Bearer`, or
  `?token=`; `AuthUser` extractor. `GET /api/station` (public: name, id, version, build, ports,
  show, users with `has_password` / `remote_login`), `POST /api/login` (sets the cookie),
  `POST /api/logout`, `GET /api/me`. View-session join/layout and all `/api/views*` use the
  logged-in user (layout writes only to the user's own screens). Served with connect
  info (remote vs local logins).
- **WS bridge** — forwards the token and the real client IP in the hello; a refused shell sends
  `{"type":"error"}` before closing. No token = `Admin` without password (accepted only while
  Admin has no password).
- **Discovery** — beacon `SessionDiscoveryWire` gains `http_port`, `station_id`, `version`,
  `show`; `ServerContext.http_port`. **mDNS** (`mdns-sd` 0.13): `remote/mdns.rs` advertises
  `_lumacore._tcp.local.` → HTTP port, TXT `id`, `station`, `shell`, `version`.
- **Browser** — WebSocket URL from `location.host`; login screen (users from `/api/station`,
  password field for users with one, auto-login of the last user when they have no password);
  connects only once logged in; `user_changed` carries a new token; `Logout` / `Disconnect`
  (alias) → `logged_out` → login screen.

### Known gaps
- The UDP command port (:47653) and `/ws/monitor` are still unauthenticated; diagnostics
  endpoints (`/api/health`, `/api/metrics`, …) and `/api/groups`, `/api/shows` are public.
- SSE events are broadcast to everyone (no per-user filtering).
- Cookie is not HttpOnly (the page swaps it on `Login`); no TLS.

## Phase 3 — decisions (asked 2026-10-10)

| Topic | Decision |
|---|---|
| Launch | **Home screen** first: countdown (5 s, *Start now* / *Cancel* Esc), then connect. Cancel → server list (mDNS + beacon, *Start local server*, manual address) |
| Auto-start target | The **last connection** (local server, or a remote one found again by station id); first run = local server as Admin. Toggle "Connect to the last server at launch" |
| Local server already running | **Attach** (not the app's to stop on crash/watch) |
| Login | In the app's home window (user list from `/api/station`). Logout → this server's users, Disconnect → server list |
| Credentials | **No memory**: passwords and tokens are never stored; a user with a password is asked each time (tokens live only in the running app) |
| Quit | Two: **Quit Client** (⌘Q, the local server keeps running; next launch attaches) and **Quit Server and Client** (⌥⌘Q, a shutdown confirmation with Save / Don't Save) |
| Connection lost | Reconnect overlay with Cancel → server list; a crashed local child is restarted once |

## Phase 3 status (built, compiles; smoke-tested the server side only)

- **One binary**: `lumacore-app --server` runs the server (UI engine serving the frontend); the app
  starts it as a child in its own process group (`local_server.rs`: probe / spawn / wait_ready /
  shutdown over the local command port / show state). Restart re-execs in place (same pid).
- **Home window** (`apps/lumacore-app/home/home.html`, served as `lumacore://localhost/`):
  countdown, server list, local server state, manual `host[:port]`, users + password, Quit Server
  and Client confirmation. IPC to the app; the app calls `home.*`.
- **Discovery** (`discovery.rs`): mDNS browse `_lumacore._tcp` + beacon listener, merged by
  station id.
- **Sessions** (`app.rs`): `/api/login` + `/api/me` (now with `screen_configs`), then **one OS
  window per open screen** of the config (prefs per server + user, else the user's default). Each
  screen is an incognito web view with `window.LUMACORE_APP` (server, user, token, config, screen,
  saved page state); the page skips its own login. Window › Screens toggles screens; window
  position / size / maximized / open and ◀ ▶ bars are remembered per machine
  (`prefs.rs`, `~/Library/Application Support/LumaCore/client.json`). File › New Window = another
  home window (another server or user). Red close: hides a screen; the last window quits the client.
- **Server**: `UiEvent::LoggedOut {disconnect}`, `UiEvent::Connect {target, user, password}`;
  `Connect` and `Disconnect` are their own `CommandDef`s (`Disconnect` was an alias of `Logout`).
  Shutdown menu: Logout and Disconnect tiles.
- **Server UI in the app** (`index.html`, `APP` mode): `user_changed` / `screen_config` /
  `logged_out` / `connect` / sidebars go to the app; reconnect overlay (also in browsers); power
  confirmations warn when the server is another machine; `lumacoreApp.quitServer()` /
  `notice()`.

### Not done / to check
- Not run as a GUI yet: home flow, multi-screen windows, menu, quit paths need a manual pass.
- `Connect` has no station-name completion (the server doesn't know what the app discovered).
- macOS .app bundle: ATS exception for `http://` LAN servers, `Resources/frontend`.
