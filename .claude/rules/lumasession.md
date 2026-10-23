---
paths:
  - crates/server/**
  - crates/commands/objects/lumacore-cmd-lumasession/**
  - crates/commands/objects/lumacore-cmd-station/**
  - crates/network/lumacore-net/**
  - crates/foundation/lumacore-wire/**
  - crates/foundation/lumacore-lanes/**
  - apps/**
---

# LumaSession, the server, stations, clients

Records: `master-executes-results-flow`, `state-homes-by-type`, `one-state-thread`, `master-loss-election`, `merge-on-rejoin`, `lumasession-membership-services`, `station-table`, `remote-station-edits`, `claims-by-tree-path`, `rights-one-gate`, `session-stream-json-zstd-journal-cap`, `same-build-across-a-lumasession`, `shutdown-signals-local-server`, `app-thin-client`, `wire-crate-clients`, `naming-vocabulary`, `station-only-assembles`.

## Words

- LumaSession is the station-to-station side (never "network session", never `Session`; no aliases): `Join LumaSession "Festival"` / `<host>`, `Leave LumaSession`, `Invite Station "X"`, `Dismiss Station "X"`, `Claim LumaSession`; runtime at `Root.Runtime.LumaSessions`; terminal sessions are the `Shell` object; "Network" means only a data network.

## Ownership of state

- `ShowData` is replicated to every station; only the master changes it, by executing commands as the sending user and pushing the results. `Runtime` holds session-wide runtime state (logins, modes, membership, clock, claims), replicated and never saved. Station-local state (identity, settings, keystore, UI, command lines, screens, clients) is never replicated; a station's rig wiring is its row of `ShowData.Stations`, applied by that station only. A new field goes in exactly one of the three; the split is enforced by where the type lives.
- On a member, dispatch forwards every line whose `registry::line_scope` is Master over the stream (`core::session_link::forward`, the link outbox, pending until the reply); the master runs it as that user and `link::flush` sends the result (`(epoch, seq)` subtree snapshots + undo steps + session runtime) to every member, applied in order. `ObjectDef::scope` says where an object runs. Never a second sync path; never a slot-keyed distribution path.
- Core asks for LumaSession work through its outbox (`session_link::LinkOut` on `AppState::link`); it never calls `lumasession/`; `Invite` / `Dismiss` / `Join` queue a request sent after the job. No network I/O inside a job.

## The server

- `lumacore-server` is the single runtime owner: `ServerBuilder` starts it after the binary installed the catalog (`lumacore_station::install_catalog`; the server only checks it). One state thread (`state.rs`; the handle and lanes in `lumacore-lanes`) owns `ServerContext`; shells, the command port, discovery, OSC and HTTP reach it only through `StateHandle::call` / `send`; no global lock. Two lanes: the shell lane (shells, a logout's revoke, the stream, elections, signals: unbounded, served first) and the machine lane (HTTP, feeds, the command port, discovery: bounded; a full lane refuses at once, E-CMD-017 / 503, counted in `Info Health Threads`). Interactive callers use `call_within` (HTTP answers 503, the shell "busy"; a shell's keys, lines, taps and answers wait in one queue in order and are never dropped); a watchdog reports `busy` on `/api/station` and the `station` feed. Long disk work runs on `core::background` and comes back as a follow-up command line.
- After each change it publishes `Arc<ShowData>` (ArcSwap) plus a `StationView`; HTTP reads only those. Every state-thread job is `panic_report::guarded`; a panic rolls the show back and replies with a report number (E-INTERNAL-001).
- Station services: the UDP command port (LumaSession requests, session clock `Time` pings answered at the socket), the TCP shell port, the discovery beacon (multicast out of every NIC that serves LumaSession), HTTP, the LumaSession stream (zstd JSON frames: join, forwarded commands, results, snapshots, merge, files a member lacks by SHA-256 `FileWant` / `File`). The port numbers are constants in `lumacore-wire`; two servers on one machine use `--station B --port-base <n>`. Per-NIC services filter the command port and discovery by source subnet and Shell / HTTP by local address; loopback always serves.

## Membership, loss, merge

- Heartbeat, warn and lost thresholds on the stream; members elect (`election::elect`: priority → results applied → online time → station id) and follow the winner with a new epoch; a station alone is IdleMaster (no DMX unless output when alone); two masters → the lower epoch is fenced and its operator decides. `ShowData.session` keeps the patch version and every station that ever joined (re-invited when back). A clean shutdown withdraws mDNS and sends a `leaving` beacon.
- Merge on rejoin: the decision Merge / Use Master / Use Mine / Cancel previews the master's show (`Hello.peek`); `Join … /Merge` makes the master 3-way merge by GUID (`lumacore_tree::merge`: master wins conflicts, edit beats delete, moves apply, playbacks the master's; patch changed on both sides blocks, E-LUMA-005). Every losing side autosaves first.
- Another station's own settings (`Set Station "Tower" "Priority" High`) go straight to its command port as `SessionRequest::EditStation`, off the state thread, never through the master or the show; the target checks rights and logs it; asks happen on the sender; the reply is `old → new` plus a `Put back:` line (Oops doesn't undo it); the grid shows `pending` until discovery reports the value.
- Rights: one level per user on `User.rights` (`Guest < Playback < Presets < Program < Setup < Admin`), only Admin grants, the user Admin is always Admin (E-USER-005); Remote Login holds (loopback only, E-RIGHTS-002); passwords are masked once in dispatch; `POST /api/logout` revokes the token. Rigs are isolated networks: keep security light; the command port, HTTP reads and station trust stay open by decision.

## Shutdown and the app

- `Shutdown` / `turnoff` → `DispatchResult::Exit` → the state thread leaves its loop; `Server::wait` stops inputs (OSC) → outputs (goodbye) → HTTP → listeners, joining every thread against one deadline (`SHUTDOWN_DEADLINE`); anything late is named in the sysmon log. Restart re-executes the binary. Tests that need a lost server use SIGKILL.
- Clients compile only `lumacore-wire` and the WebUI's files; nothing under `apps/` depends on `lumacore`. `SESSION_PROTOCOL` (station ↔ station, E-LUMA-001) and `CLIENT_PROTOCOL` (a client's hello, E-WIRE-001) bump on any wire change, no shims. The desktop app spawns the `lumacore-server` next to it and never attaches silently to another version (`/api/station` gives `client_protocol`, `pid`, `exe`, `started_ms`; Replace / Use it / Cancel; Force stop only on a click). The station only assembles: `apps/lumacore-station` is the catalog and thin binaries, nothing else.
