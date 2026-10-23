# lumacore-shell

The TCP shell port: one thread per connected client, hosting its terminal and running every key,
line, gesture, tap and answer as a job on the state thread.

## Purpose

How a terminal client (the CLI, the desktop app's TUI, a browser's `/ws/shell` bridge) works a
station over the shell port, `REMOTE_SHELL_PORT` in lumacore-wire (decisions §23 "Step 8"). Its thread reads the hello (protocol,
device, size), logs the user in (a token or user + password, remote login rules), attaches the
client to its Shell session, then reads packets and queues keys in typed order — never dropped
while the station is busy, each waiting at most `SHELL_WAIT` (30 s; suggestions `SUGGEST_WAIT`,
1 s, skipped behind queued keys) — and sends back the frames, status and UI events the terminal
rendered for that client. A monitor client gets the System Monitor stream. It is to the terminals
what `lumacore-http` is to pages; the context it serves is a `ShellHost` (the station's server).

## Owns / Doesn't own

- **Owns:** `handle_shell_client`, the packet framing (`read_shell_packets`, `send_shell_packet`),
  the waits (`SHELL_WAIT`, `SUGGEST_WAIT`) (`shell`); `ShellHost` and the gestures it runs on the
  state thread (`host`).
- **Doesn't own:** the listener that accepts clients (the server), the terminals and their
  sessions (`lumacore-terminal`), the state thread (`lumacore-lanes`), logins and tokens
  (`lumacore-objects`' `passwords`), the network view (the server).

## Public surface

- `handle_shell_client(stream, addr, state)` — one client's thread.
- `ShellHost` — what the served context provides (`split`, `refresh_runtime`, `request_shutdown`,
  the System Monitor).
- `read_shell_packets`, `send_shell_packet` — the framing.
- `register` — its place in the catalog (no rows of its own).

## Rules

- A shell's keys, lines, taps and answers wait in one queue in order and are never dropped. [unchecked]
- A suggestion behind queued keys or past its wait is skipped, never stale. [unchecked]
- It reaches the state only through its `StateHandle`, never a lock. [compiler]
- It depends on the lanes, the terminals, objects, tree and the wire. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-registry, lumacore-lanes, lumacore-wire, lumacore-tree, lumacore-objects, lumacore-terminal, lumacore-sysmon.
- **Used by:** lumacore-station.

## Decisions

- [§23 "Step 8"](../../../docs/archive/decisions-2026-10.md) — the shell port
- [§30 "12b", "Merge of bugfixes"](../../../docs/archive/decisions-2026-10.md) — dialogs, busy coverage
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (24.2), its README (18)

## Testing

- `cargo test -p lumacore-shell` — the packet framing.
- End to end: the station's shell tests, systemtest, e2e.
