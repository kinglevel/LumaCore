# lumacore-net

The station's leaf network I/O: its NICs and sockets bound to them, the command-port and shell
sockets, the mDNS record, the linuxptp reader and what a discovery beacon tells about a peer.

## Purpose

Everything that opens a socket or reads the machine's network state and decides nothing (decisions
§34 20.6, 22). The protocol crates bind to a NIC through it, the server opens its ports through it,
and nothing here knows the show, the state thread or LumaSession's rules. It is the bottom of the
network side: on the wire types, the runtime's health types and the protocols' `Nic` — never the tree.

## Owns / Doesn't own

- **Owns:** `Nic`, the interface list, how a network mapping finds one and a socket listening on
  one (`nics`); binding the UDP command port, the shell listener and the discovery sender, reading a
  request and sending a reply (`socket`); the station's mDNS record (`mdns`); reading linuxptp
  through `pmc` (`ptp`); `DiscoveryPeerState`, one peer as its beacons describe it (`discovery`).
- **Doesn't own:** the packets on the wire (`lumacore-wire`), what a request does and the threads
  that serve the ports (the server's `net/`), the error text of a refusal (`lumacore-state`
  errors), which NIC a network maps to (the station table, `lumacore-objects`), the session clock
  and the election (`lumasession`), the DMX protocols (`lumacore-protocols` and its protocol crates).

## Public surface

- `nics::{Nic, list, resolve, listen}` — the station's interfaces.
- `socket::{bind_server_socket_at, bind_shell_listener_at, bind_discovery_sender, recv_request,
  send_reply}` — the command port and the shell listener.
- `mdns::{advertise, Advertisement}` — the `_lumacore._tcp` record, withdrawn on a clean stop.
- `ptp::{spawn_ptp_reader, parse}` — linuxptp, read-only.
- `DiscoveryPeerState` — a peer from its beacons.

## Rules

- It decides nothing: no show, no state thread, no LumaSession rule. [compiler + tidy crate_edges_point_down]
- A socket for a NIC is bound to that NIC's address; there is no "auto" interface. [unchecked]
- A reply too large for a packet is replaced by a refusal, never lost. [unchecked]
- PTP is read only; LumaCore never configures it. [unchecked]

## Edges

- **Depends on:** lumacore-base, lumacore-runtime, lumacore-protocols, lumacore-wire.
- **Used by:** lumacore-protocols, lumacore-artnet, lumacore-sacn, lumacore-lines, lumacore-objects, lumacore-views,
  lumacore-station, systemtest.

## Decisions

- [§11](../../../docs/archive/decisions-2026-10.md) — networks and NICs
- [§23 "Step 8"](../../../docs/archive/decisions-2026-10.md) — the ports
- [§24 "Step 9 hardening"](../../../docs/archive/decisions-2026-10.md) — PTP, read-only
- [§30 "12b2 hardening"](../../../docs/archive/decisions-2026-10.md) — mDNS withdrawn on a clean stop
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (20.6, 22), its README (18)

## Testing

- `cargo test -p lumacore-net` — NIC resolution, the pmc parser.
