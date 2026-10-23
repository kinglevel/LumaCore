---
id: 0029
title: Clean shutdown on signals; the app never attaches blindly
area: engine
status: built
date: 2026-10-11
built_in: step 9b
supersedes: [archive §26 L2511–2513 "inputs state thread outputs"]
superseded_by: —
confirmed_by: [SHUTDOWN_DEADLINE, watch_signals]
source: [archive §23 L2121–2161, archive §23 L2180–2219, archive §26 L2509–2513, archive §26 L2523–2531]
---
# Clean shutdown on signals; the app never attaches blindly

## Context
A pre-step-8 server was still running; the new app found something answering on the local HTTP port and attached without checking, so the page talked to an old backend. `kill` or a closed terminal stopped the server with no goodbye and no autosave.

## Decision
`Shutdown` / `turnoff` from any client → `DispatchResult::Exit` → `ctx.shutdown_requested` → the state thread leaves its loop. SIGTERM / SIGINT / SIGHUP (`lumacore_server` `watch_signals`, unix) run like `Shutdown /NoSave` after an autosave copy of unsaved changes (`.autosave/{show}_signal_{station}_{time}`); a second signal exits at once. `Server::wait` stops in order: inputs (OSC) → outputs (sACN sends stream-terminated, Art-Net stops) → HTTP → listeners and network threads, every thread joined against one `SHUTDOWN_DEADLINE`, anything late named in sysmon; a clean stop withdraws mDNS and sends a `leaving` beacon. **The app never attaches to a local server silently.** `/api/station` gives `client_protocol`, `pid`, `exe`, `started_ms`; `lumacore-server --version` prints the build id. The local server is "another version" when its client protocol differs or is missing, its build differs from the `lumacore-server` next to the app, or that binary was rebuilt after it started; the app asks Replace / Use it (only when the protocol matches) / Cancel, and a replace with unsaved changes asks Save / Don't Save. A stop that hangs past `STOP_WAIT` offers Force stop (SIGTERM, then SIGKILL) **only on a click**, only for a pid the server reported itself, on this machine, whose executable is a LumaCore server.

## Consequences
A new engine or listener joins the ordered stop with the shared deadline, never its own. Tests that need a lost server (no announcement) use SIGKILL. Shutdown from a client stays the one path through dispatch.
