# lumacore-lines

The line runtime: one driver thread per (protocol, NIC) sending the render engine's frames on
output lines and receiving input lines, and one listener per Data line — started, re-planned
and stopped to match the show.

## Purpose

How DMX leaves and enters the station (decisions §11 "Step 4b/4c hardening", §34 22.1). After
every show change (and once a second, for NIC changes) `ProtocolRuntimes::reconcile` makes the
running drivers match `protocol_plan::desired`. A driver holds one protocol's sockets on one NIC
on its own thread: each output line samples the newest `RenderedFrame` at its own `Rate`, sends
its run of universes (`Full` or `Changes` with repeats and keep-alive, `PacketGap`, `Sync`), and
each input line's packets become layers the tick merges. Plans are swapped live; a different NIC
is a new driver. Every protocol is opened through its catalog row (`ProtocolDef`): this crate
never names Art-Net, sACN or OSC.

## Owns / Doesn't own

- **Owns:** `NetworkDriver`, `DriverShared`, the send loop and the input loop (`lines`);
  `ProtocolRuntimes`, which drivers and Data listeners run, the line states, nodes and
  sources it publishes to `Runtime.Network` (`runtimes`).
- **Doesn't own:** what should run (`lumacore_objects::protocol_plan`), what a protocol is and
  its row (`lumacore-protocols`), the packets and sockets (`lumacore-artnet`, `lumacore-sacn`,
  `lumacore-osc`), the frames (`lumacore-render`), which protocols the build has (the station's
  `register`).

## Public surface

- `ProtocolRuntimes::{new, reconcile, stop_all}` — owned by the state thread.
- `NetworkDriver` — one driver thread per (protocol, NIC).
- `DriverShared` — the plans and stats shared lock-free with the state thread.

## Rules

- The send allocates, locks and logs nothing between its render regions (checked in debug builds
  too). [tidy render_paths_never_lock_log_or_allocate]
- A protocol is found by its row, never by name in code: a line names a row of this build. [test a_protocol_is_an_object_named_by_its_row]
- A line on an unmapped network never sends. [test unmapped_networks_never_send]
- Only a standalone station or the session master sends or listens. [test mapped_lines_send_on_their_nic_unless_not_master]
- One driver per (protocol, NIC) sends outputs and receives inputs. [test input_lines_join_the_driver_of_their_nic]

## Edges

- **Depends on:** lumacore-objects, lumacore-render, lumacore-protocols, lumacore-net, lumacore-tree, lumacore-base, lumacore-registry.
- **Used by:** lumacore-station.

## Decisions

- [§11](../../../../docs/archive/decisions-2026-10.md) — networks, lines, rates, sync, input
- [§24 "Step 9 hardening"](../../../../docs/archive/decisions-2026-10.md) — the audio-style send
- [§34](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (22.1), its README (18)

## Testing

- `cargo test -p lumacore-lines` — the send cadence, what runs where, every protocol's row.
- Lines end to end: systemtest.
