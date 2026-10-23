---
title: Software Init
category: docs
---
# Software init

How LumaCore starts, who owns what, and how the server and its clients live
together. The developer details are in
[engines.md](../../dev/engines/engines.md) and
[ARCHITECTURE.md](../../../ARCHITECTURE.md).

## One owner

```text
lumacore-server     the one runtime owner: show, state, commands, ports, engines
lumacore-cli        a terminal client (wire protocol only)
lumacore-app        the desktop app; starts lumacore-server next to it
browser             the web UI, served by the station (HTTP feature)
scripts/unix/…      helpers: start_server, stop_server, connect.sh,
                    start_developer, stop_developer, smoke_test, fuzz_long
```

Rules:

- **the server boots once**; clients attach many times
- clients compile only `lumacore-wire`; a client with another
  `CLIENT_PROTOCOL` is refused (E-WIRE-001)
- scripts start, stop and attach; they never become a second runtime
- server-side checks (rights, claims) always win over a client's idea

## Boot order

`lumacore-server` starts in this order (`server::ServerBuilder::start`):

1. **Logging** — the server log, the System Monitor log
   (`library/logs/sysmon-YYYY-MM-DD.log`, 7 days), the panic hook, the banner.
2. **Show** — build the state and load the show (the `--show` file, else the
   startup show: the last show by default, `Show /Startup …`). Authored state is
   loaded before anything network-facing runs.
3. **Ports** — bind the UDP command port, the TCP shell port and the TCP
   LumaSession stream (the numbers are `lumacore-wire`'s port constants;
   `--port-base` shifts them). A port that can't be bound stops the start.
4. **Services per NIC** — which interfaces serve LumaSession, Shell and HTTP
   (`Set Interface …`; loopback always serves).
5. **HTTP setup** (with the `http` feature, unless `--no-http`).
6. **State thread** — from here one thread owns all state; everything else
   reaches it through its handle.
7. **Network threads** — the command port, the shell listener, the stream
   listener, session-clock sync, the PTP reader, discovery (the UDP beacon).
8. **Engines** — capability-gated, started in order:
   - `RenderEngine` — the one renderer (60 Hz render thread + plan compiler)
   - `PerfettoEngine` — the trace server (`perfetto` feature)
   - `HttpUiEngine` — REST / WebSocket (`http` feature)
9. **Protocols** — handed to the state thread, which starts the sACN / Art-Net
   output drivers and OSC input listeners from the show's lines, and reconciles
   them after every show change.
10. **"Server ready"** in the System Monitor.

Not engines:

- **LumaSession** is not an engine. It is the server-to-server link (stream,
  election, merge, clock) run by the network threads and the state thread.
- **Protocols** are not engines. Output and input lines come from the show
  (decisions §11).

## Shutdown

- `Shutdown` / `turnoff` (asks first), `LumaCore-Shutoff` (always asks),
  `Restart` (re-runs the binary).
- SIGTERM / SIGINT / SIGHUP work like `Shutdown /NoSave` after an autosave copy
  of unsaved changes (`library/shows/.autosave/{show}_signal_…`); a second
  signal exits at once.
- The stop order: inputs (OSC) → outputs (sACN / Art-Net say goodbye) → HTTP →
  network threads, all joined against one 5 s deadline. Anything late is named
  in the System Monitor.

## Clients

- A client connects only after the server answers its health check.
- A user logs in; the shell attaches to that user's command line.
- Many clients may attach; a client may reconnect without restarting the server.
- The server stays authoritative when a client is outdated, resized or detached.
- `Logout` in `lumacore-cli` returns to its outer prompt, never to a second
  engine.
- The app never attaches silently to a server of another version (protocol,
  build, or a binary rebuilt since it started): it offers Replace / Use it /
  Cancel. A stop that hangs past 10 s offers Force stop.

Terminal details: [cli-terminal.md](../terminal/cli-terminal.md).

## Two servers on one machine

`lumacore-server --station B --port-base 48660` runs a second station: its files
in `library/stations/B/`, its ports from the base (command N+3, shell N+4, HTTP
N+6, stream N+8).

## Background

Notes kept from the early design of startup (2026-03):

- **Phases.** Startup was first described as six phases: process bootstrap,
  configuration and logging, show and user load, runtime service bind, session
  and client attach, interactive operation. The boot order above is what that
  became.
- **Engine families.** The early design named a Runtime Owner Engine, Session
  Engine, Network Engine, UI Engine, Render Engine, Diagnostics Engine and
  Integration Engine. Only three engines exist (Playback, Perfetto, HttpUi); the
  rest became plain layers of the server (`server/`, `net/`, `lumasession/`,
  `core/`) and crates (the protocols and their line runtime, `lumacore-lines`).
- **Lum nouns for startup.** `Server`, `Client`, `Station`, `Shell` became real
  objects (`Info Server`, `List Client`, `List Station`, `List Shell`).
  "Session" alone is not a noun: `LumaSession` is server-to-server, `Shell` is a
  user's terminal. A `Monitor` noun stayed an idea; the System Monitor is a window
  and `lumacore-cli --meownitor`.
- **Borrowed ideas.** From QLC+: load the document first, then managers, then
  UI and network. From Mixxx: one owner of core services and a careful start
  sequence. From Stargate: keep engine and UI apart for crash containment. From
  openDAW: explicit boot entry points and a headless/app split. From console
  practice: keep live session state apart from the saved show.
- **What to avoid.** A CLI that grows into a second engine; boot logic scattered
  across scripts, client and server; binding services before the show is loaded;
  mixing the developer rebuild flow with normal startup.
