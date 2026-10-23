# lumacore-wire

The client ↔ station contract: the protocol numbers, the ports, and every packet a client and a
station both compile.

## Purpose

A client (the CLI, the desktop app, the web page's shell bridge) compiles this crate and nothing
else of LumaCore (decisions §23 "Step 8"). It holds what goes over the wire between them — the
UDP command port's request and reply, the shell port's packets, the status a station sends each
shell client, the discovery beacon, the server-held dialog and the suggestions — and the two
protocol numbers both sides compare. Station ↔ station stream frames carry show types and are
not here; they live in `lumacore-lumasession`.

## Owns / Doesn't own

- **Owns:** `SESSION_PROTOCOL` and `CLIENT_PROTOCOL` with `client_mismatch`; the port constants
  (`REMOTE_CMD_PORT`, `REMOTE_SHELL_PORT`, `SESSION_DISCOVERY_PORT`, `REMOTE_HTTP_PORT`,
  `REMOTE_PERFETTO_PORT`, `REMOTE_STREAM_PORT`), the discovery groups and `MDNS_SERVICE`;
  `StationKind`; `RemoteRequest` / `RemoteReply` and `send_request` (`command`);
  `ShellClientPacket` / `ShellServerPacket`, `UiEvent`, `SettingField`, `FieldInput` (`shell`);
  `Status` and its parts (`status`); `Dialog`, `DialogAnswer`, `DialogButton` (`dialog`);
  `SessionDiscoveryWire`, `EngineInfo`, `bind_discovery_listener_at` (`discovery`);
  `Suggestions`, `SuggestItem` (`suggest`).
- **Doesn't own:** what a request does (the server), who may send one (the rights gate in
  `lumacore-objects`), the sockets bound to a NIC (`lumacore-net`), the session stream
  (`lumacore-lumasession`), any rendering of a packet (the terminals, the WebUI).

## Public surface

- `SESSION_PROTOCOL`, `CLIENT_PROTOCOL`, `client_mismatch(client, station)` — the version
  gate (E-LUMA-001, E-WIRE-001).
- The port and multicast constants, `discovery_group(alternative)`, `StationKind`.
- `send_request`, `send_request_to`, `try_health` — one JSON request, one JSON reply.
- `ShellClientPacket`, `ShellServerPacket`, `RemoteShellKey`, `UiEvent`, `GesturePhase`.
- `Status`, `DeviceStatus`, `ShowStatus`, `LumaSessionStatus`, `ClaimStatus`, `Decision`.
- `Dialog`, `DialogAnswer`, `DialogButton`, `Pick`, `Countdown`.
- `SessionDiscoveryWire`, `Capability`, `EngineInfo`, `bind_discovery_listener_at`.

## Rules

- A client compiles only this crate: no lumacore dependency, no async runtime, no UI.
  [tidy crate_edges_point_down]
- Any change to what a number covers bumps it; there is no compatibility shim, and a hello
  without a number reads as zero and is refused by name.
  [test hello_without_protocol_reads_as_zero + test mismatch_message_names_the_code]
- `Status` uses the frontend's field names, so the page reads it as sent.
  [test status_roundtrips_with_the_frontend_field_names]
- A beacon says `leaving` only when the station does. [test a_beacon_is_leaving_only_when_it_says_so]
- A dialog's answers are named buttons, never positions in a list. [unchecked]

## Edges

- **Depends on:** nothing.
- **Used by:** lumacore-app, lumacore-cli, lumacore-net, lumacore-clocksync, lumacore-state,
  lumacore-objects, lumacore-http, lumacore-shell, lumacore-terminal, lumacore-server,
  lumacore-lumasession, lumacore-suggest, the `cmd-` crates, systemtest, catalog, fuzz.

## Decisions

- `docs/decisions/0037-wire-crate-clients.md` — clients compile only this crate; two protocol numbers
- `docs/decisions/0035-app-thin-client.md`, `docs/decisions/0029-shutdown-signals-local-server.md` — what the
  app reads from a station
- `docs/decisions/0186-one-server-held-dialog.md` — the dialog shape
- `docs/decisions/0173-one-suggestion-engine.md` — the suggestion packets
- `docs/decisions/0185-client-feeds-one-socket.md` — the feed hello carries `CLIENT_PROTOCOL`
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §23, §30

## Testing

- `cargo test -p lumacore-wire` — `src/tests.rs`: the hello, the status round trip, the
  mismatch message, the beacon.
- The packets on a live station: `tests/systemtest` and `tests/fuzz` (`shell_client.rs`,
  `shell_server.rs`, `remote_request.rs`, `beacon.rs`, `time_reply.rs` decode them).
