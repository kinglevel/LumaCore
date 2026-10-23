# lumacore-artnet

Art-Net 4 on one NIC, declared as a DMX protocol: ArtDmx out to the nodes found by ArtPoll, else to the network's directed broadcast; ArtDmx in, with ArtPollReply for input lines.

## Purpose

One protocol's wire and its driver (decisions §11, §34 22). Its `register` adds its `ProtocolDef`
row to the catalog — the name lines use, its directions, port and capabilities (Sync, Discovery (the nodes' refresh rate caps a line) and SharedPort (it sends and receives on the Art-Net port)) and the
function that opens it — so the line runtime runs it without naming it. Nothing here decides
when a line sends or what it sends.

## Owns / Doesn't own

- **Owns:** `ArtNetSource`, `ArtNetNode` and their packets (`source`), `ArtNetDriver` (`driver`), the `ARTNET` row (`ArtNet`) and its Port-Addresses
  (`port_address_label`, `port_address_of`: `net.subnet.universe`, 0–32767).
- **Doesn't own:** what a protocol is and the input merger (`lumacore-protocols`), the NICs and
  binding to one (`lumacore-net`), when and what a line sends (`lumacore-lines`), the frames
  (`lumacore-render`).

## Public surface

- `ArtNetSource`, `ArtNetNode`, `source::{artdmx_packet, parse_artdmx, parse_poll_reply}` — Art-Net on one NIC.
- `port_address_label`, `port_address_of` — a Port-Address as `net.subnet.universe` and back.
- `register` — its row in the catalog.

## Rules

- The send allocates, locks and logs nothing between its render regions. [tidy render_paths_never_lock_log_or_allocate]
- A socket is bound to the mapped NIC's address; there is no "auto" interface. [unchecked]
- It depends on the interface, net and the registry only — never the renderer. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-protocols, lumacore-net, lumacore-registry.
- **Used by:** lumacore-station, lumacore-sweep, systemtest, fuzz; the test catalogs of
  lumacore-objects and lumacore-suggest (dev).

## Decisions

- [§11](../../../../../docs/archive/decisions-2026-10.md) — networks, lines, DMX input
- [§24 "Step 9 hardening"](../../../../../docs/archive/decisions-2026-10.md) — the audio-style send path
- [§34](../../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (22), its README (18)

## Testing

- `cargo test -p lumacore-artnet` — packet encoding and parsing, ArtPoll nodes and targets.
- The decoders are fuzzed (`tests/fuzz/`, the long run: `./scripts/unix/fuzz_long`).
