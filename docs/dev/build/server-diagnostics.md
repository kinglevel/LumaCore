# Server Diagnostics (System Monitor)

The LumaCore server keeps a console-style System Monitor ("sysmon"): the
station's log of what happens — startup, logins, every user's commands, show
changes, warnings and errors. Every meaningful state transition is logged once
— no loops, no floods.

Code: `crates/foundation/lumacore-sysmon` (`monitor.rs`: the ring, levels, subscribers, the
log file; `feedback.rs`; `panic_report.rs`) — a logging system every crate can write to; the
state thread writes through the server's context, and code without one queues lines in
`AppState::sysmon_pending`. Beside it under `library/logs/`: the server's own output
(`server/`) and the panic reports.

## Line format

Every line is a `SysmonLine`: `seq`, `time_ms` (wall clock), `elapsed_ms`
(since server start), `level`, `subsystem`, `message`, and optionally a
`feedback` event (progress, flash, message, status). The classic text form is:

```
{elapsed}  {subsystem:>14} : {prefix}{message}
```

Example (from a real start):
```
  0h00m00.008s         Server : LumaCore server v0.1.0/debug/macos-aarch64
  0h00m00.013s         Server : Host: Mac.local (192.0.2.10)
  0h00m00.045s           Show : Show loaded
  0h00m00.049s        Network : UDP command listener bound on 0.0.0.0:47653
  0h00m00.064s         Server : *  Server ready
```

## Levels

`Info`, `Warn`, `Error`, `Alert` (red: shutdown, restart, power
loss). A message that starts with `[WARN]` or `[ERROR]` sets the level; the
prefix is kept out of the structured message and added back in the text form
(`[WARN] `, `[ERROR] `, `[ALERT] `).

## Where it goes

The server keeps the last `RING_SIZE` lines in memory. Readers:

| Reader | How |
|---|---|
| stderr | `lumacore-server --RootMonitor` echoes every line |
| Meownitor | `cargo run -p lumacore-cli -- --meownitor` streams the text form over the shell port (`MonitorHello`) |
| Web System Monitor window | the `sysmon` feed on `/ws/feeds` (login): the whole history in one batch, then live lines in batches; `GET /api/sysmon` for the history. See [http-api.md](../api/http-api.md) |
| Feedback | the `feedback` feed on `/ws/feeds`: the lines that carry a feedback event, per user |
| Log file | `library/logs/sysmon-YYYY-MM-DD.log` (local date and time in front of each line), kept `LOG_DAYS` days |

Clients join history and live lines by `seq`; a line can arrive twice, so skip
known `seq`s.

## Subsystem tags

Tags seen in the code today (not a closed list; a new subsystem may add one):

| Tag | Used for |
|-----|----------|
| `Server` | Startup banner, build, OS, host IP, PID, ready, health checks, `purrrrrrrrrr` every 30 s |
| `Show` | Show load/save, user profiles |
| `Network` | UDP/TCP socket binds, client connections |
| `Discovery` | Multicast listener, beacon sender, peer found/lost |
| `HTTP` | HTTP engine listening, HTTP logins |
| `Engine` | Thread spawns, engines started/stopped |
| `Session` | Shell login/logout, attach/detach, resize, monitor connect |
| `Cmd` | Every executed command line (`[User@Screen n] line → result`) |
| `Command` | Command-port requests (`--cmd`, OSC, remote): who, what, refusals |
| `LumaSession` | Join, leave, invite, dismiss, master loss, election |
| `Merge` | Merge on rejoin |
| `Clock` | Session clock lock, steps |
| `Render` | Render engine panics and overload episodes (`RenderWatch`, once per episode) |
| `Output` | Output distribution changes (who sends what on which NIC) |
| `sACN` | sACN output state at startup |
| `System` | Feedback statuses (e.g. `sACN 1/1 on`) |
| `References` | Broken links after load, join, merge, Oops |
| `Claims`, `Keys`, `Programmer` | Claim asks, key binding checks, programmer events |
| `Shutdown` | Ordered shutdown steps |

## Expected startup sequence

```
  0h00m00.008s         Server : LumaCore server v0.1.0/debug/macos-aarch64
  0h00m00.009s         Server : Build 0.1.0/debug/macos-aarch64/58a0f06e96cf-dirty
  0h00m00.012s         Server : OS: macos macOS 25.6.0 (aarch64)
  0h00m00.013s         Server : Host: Mac.local (192.0.2.10)
  0h00m00.013s         Server : PID 40039
  0h00m00.044s           Show : Loading showfile: demo
  0h00m00.045s           Show : Show loaded
  0h00m00.048s           sACN : No sACN output configured
  0h00m00.049s           Show : User profiles: 3 (Admin, Debug, LD)
  0h00m00.049s        Network : UDP command listener bound on 0.0.0.0:47653
  0h00m00.049s        Network : TCP shell listener bound on 0.0.0.0:47654
  0h00m00.049s        Network : TCP LumaSession stream listener bound on 0.0.0.0:47658
  0h00m00.060s           HTTP : HTTP engine listening on 0.0.0.0:47656
  0h00m00.061s         Engine : ShellServer thread spawned
  0h00m00.061s      Discovery : Multicast listener bound on 0.0.0.0:47655 (groups 239.255.76.67 / 239.255.76.68)
  0h00m00.061s      Discovery : Beacon sender ready (750ms interval)
  0h00m00.062s         Engine : DiscoveryRecv thread spawned
  0h00m00.062s         Engine : DiscoverySend thread spawned
  0h00m00.064s         Engine : 'playback' started
  0h00m00.064s         Engine : 'http' started
  0h00m00.064s         Server : ****************************************************
  0h00m00.064s         Server : *  Server ready
  0h00m00.064s         Server : ****************************************************
```

Output lines start later, after the first protocol reconcile
(`Output : distribution changed: …`).

## Expected shutdown sequence

```
  0h06m48.470s       Shutdown : SIGTERM: shutting down (unsaved changes kept in library/shows/.autosave/…)
  0h06m48.922s       Shutdown : Closing 1 shell session(s)
  0h06m49.056s       Shutdown : Listeners closed
  0h06m49.056s       Shutdown : Server shut down after   0h06m49.056s
```

From a shell `Shutdown` the first line is `shutdown requested from shell
session`. Threads that miss the 5 s `SHUTDOWN_DEADLINE` are named in the log.

## Philosophy

- **Log state transitions, not ticks.** Every line should represent something
  that happened or changed. (The 30 s `purrrrrrrrrr` is the one deliberate
  heartbeat.)
- **No floods.** Discovery peers are logged once on first appearance and once
  when lost. Render overload is logged once per episode.
- **Every line should help.** If a line doesn't help you understand what happened
  or debug what went wrong, it shouldn't exist.
- **Passwords never appear.** Dispatch masks secrets before a line reaches
  sysmon.

## Smoke test

```bash
./scripts/unix/smoke_test
```

It builds the release server, starts it, checks the startup sysmon lines, shuts
it down cleanly and checks the shutdown lines.
