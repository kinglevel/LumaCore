# lumacore-sacn

sACN (E1.31) on one NIC, declared as a DMX protocol: a hand-encoded sender bound to one network interface, universe discovery and sync; a receiver that joins the input lines' multicast groups and drops stale sequence numbers.

## Purpose

One protocol's wire and its driver (decisions §11, §34 22). Its `register` adds its `ProtocolDef`
row to the catalog — the name lines use, its directions, port and capabilities (Sync) and the
function that opens it — so the line runtime runs it without naming it. Nothing here decides
when a line sends or what it sends.

## Owns / Doesn't own

- **Owns:** `SacnSource` and its packets (`source`), `SacnReceiver` and its parsers (`receiver`), `SacnDriver` (`driver`), the `SACN` row (`Sacn`, universes 1–63999) and its
  setting `Priority` (0–200, default 100; output lines only).
- **Doesn't own:** what a protocol is and the input merger (`lumacore-protocols`), the NICs and
  binding to one (`lumacore-net`), when and what a line sends (`lumacore-lines`), the frames
  (`lumacore-render`).

## Public surface

- `SacnSource`, `SacnReceiver`, `Destination`, `receiver::{parse_data_packet, parse_sync_packet}` — sACN on one NIC.
- `register` — its row in the catalog.

## Rules

- The send allocates, locks and logs nothing between its render regions. [tidy render_paths_never_lock_log_or_allocate]
- A socket is bound to the mapped NIC's address; there is no "auto" interface. [unchecked]
- It depends on the interface, net and the registry only — never the renderer. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-protocols, lumacore-net, lumacore-registry.
- **Used by:** lumacore-station, lumacore-sweep, fuzz; the test catalogs of lumacore-objects,
  lumacore-suggest and lumacore-tree (dev).

## Decisions

- [§11](../../../../../docs/archive/decisions-2026-10.md) — networks, lines, DMX input
- [§24 "Step 9 hardening"](../../../../../docs/archive/decisions-2026-10.md) — the audio-style send path
- [§34](../../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (22), its README (18)

## Testing

- `cargo test -p lumacore-sacn` — packet encoding and parsing, sequence numbers, sync.
- The decoders are fuzzed (`tests/fuzz/`, the long run: `./scripts/unix/fuzz_long`).
