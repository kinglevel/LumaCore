# lumacore-server

The station's server: startup and the ordered stop, `ServerContext`, the one state thread, the
UDP command port, discovery, the network view and the HTTP bridge.

## Purpose

What runs one station (decisions §23 "Step 8", §26). `ServerBuilder::start` loads the show, binds
the ports and starts every thread and engine; `Server::wait` stops them in order — inputs,
outputs (goodbye), HTTP, listeners — against one 5 s deadline, naming anything late. The state
thread owns `ServerContext` (the show, runtime and station state) and is reached only through its
`StateHandle`: it runs the jobs, the inbound events, the LumaSession link and the periodic work,
and publishes the show and a small `StationView` after each change, which HTTP reads. The
command port answers LumaSession requests and remote command lines; discovery sends the beacon
out of every NIC that serves LumaSession. The server implements the hosts the crates above ask
for: `ShellHost` (the shell port), `SessionHost` (LumaSession), `Host` (the lanes). It never
installs the catalog: the station's binaries do (decisions §34 24.1).

## Owns / Doesn't own

- **Owns:** `ServerConfig`, `ServerBuilder`, `Server`, `serve`, `restart_process` (`builder`);
  the state thread, `Published`, `StationView`, `HTTP_WAIT` (`state`); `ServerContext` and its host
  impls (`context`); `CommonCli`, logging, `build_state` (`app`); the command port's handler, the
  listener and discovery threads, the network view, the HTTP console bridge (`net`).
- **Doesn't own:** the catalog and the binaries (`apps/lumacore-station`), the lanes
  (`lumacore-lanes`), LumaSession (`lumacore-lumasession`), the shell port (`lumacore-shell`), the
  engines (`lumacore-engines`), the line runtime (`lumacore-lines`), what any UI shows
  (`lumacore-views`), REST and the feeds (`lumacore-http`).

## Public surface

- `serve(config)`, `ServerBuilder::start`, `Server::wait` — run a station.
- `ServerContext` — the state thread's context.
- `app::{CommonCli, build_state, init_logging}` — what the binaries share.
- `register`, `require_catalog` — its place in the catalog; the check before it starts.

## Rules

- One state thread owns the context; every other thread reaches it through its `StateHandle`. [compiler]
- No network I/O inside a job: joins, invites and remote edits are queued and sent after it. [unchecked]
- Every thread is joined against one shutdown deadline; a late one is named. [unchecked]
- It starts only on an installed catalog. [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-base, lumacore-wire, lumacore-lanes, lumacore-trace, lumacore-tree, lumacore-objects, lumacore-render, lumacore-engines, lumacore-net, lumacore-clocksync, lumacore-lines, lumacore-views, lumacore-terminal, lumacore-shell, lumacore-http, lumacore-lumasession, lumacore-sysmon, lumacore-luma, lumacore-cmd-showfile, lumacore-cmd-devices, lumacore-cmd-protocols, lumacore-drives.
- **Used by:** lumacore-station.

## Decisions

- [§23 "Step 8"](../../../docs/archive/decisions-2026-10.md) — the server and its ports
- [§26](../../../docs/archive/decisions-2026-10.md) — one state thread
- [§31](../../../docs/archive/decisions-2026-10.md) — the lanes, OSC users
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (24.2), its README (18)

## Testing

- `cargo test -p lumacore-server --features http` — the state thread, the command port's rights,
  the startup marker, LumaSession on a whole server (link, election, remote edits).
- systemtest; e2e for the browser and two stations.
