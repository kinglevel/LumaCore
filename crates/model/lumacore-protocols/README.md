# lumacore-protocols

The protocol contract, below the show tree: the line data the show holds (Network, Line,
MergeMode, …), `Nic`, the `ProtocolDef` row each protocol crate adds to the catalog, the driver
traits it implements, and the protocol-neutral DMX pieces they share — the input merger, the
input layers and the line plans. No sockets, no threads.

## Purpose

The contract every protocol plugs into and the show is made of (decisions §34 22, 25.14: like
QLC+'s `qlcioplugin.h` below its document, Linux's `net_device` + ops below the drivers, Servo's
`shared/*_traits`). It sits below `lumacore-tree`, so `ShowData` holds its line types; the
implementations — sockets, threads, the OS's NICs — stay above, in `lumacore-net`,
`lumacore-lines` and the protocol crates. A protocol crate (`lumacore-artnet`,
`lumacore-sacn`, `lumacore-osc`) declares its name, kind (Dmx or Data), directions, port and
capabilities and the function that opens it; the line runtime (`lumacore-lines`) asks the row and
its capabilities, never a protocol's name. A line is one DMX Protocols row of its protocol (decisions §34 "34f7c hardened"): it
lives under its protocol's node in the show (`Line.protocol` is that node's name: `Sacn`,
`ArtNet`, `Osc`), with `Mode` Output | Input and one run of local universes (`local_universe` +
`amount`, slots by number, never links) starting on the wire at `universe`. The row also declares
what a line without a target does, the wire universes a line takes (the property that names them —
`SacnUniverse`, `PortAddress` —, range, label, parse), its own line settings (`Line.settings`:
sACN's Priority, OSC's Port) and its protocol-wide settings (none yet). The input merger turns every source's packets into one
layer per line and universe (E1.31 priority, then HTP, loss after 2.5 s).

## Owns / Doesn't own

- **Owns:** the line data and its identity / walk (`line`); `Nic` (`nic`); `ProtocolDef`, `Kind`, `Capability`, `Open`, `DriverIdentity`, `Node`,
  `ControlStart`, the `DmxDriver` and `ControlListener` traits, the lookup and its freeze check
  (`protocol`); `InputMerger`, `InputPlan`, `InputLayer`, `Packet`, `SourceKey`, `SourceName`
  (`input`); `LinePlan` (`line_plan`); `DmxError` and the E-DMX catalog (`errors`).
- **Doesn't own:** the wires (the protocol crates), when a line sends (`lumacore-lines`), the
  frames (`lumacore-render`), listing the OS's NICs (`lumacore-net`), the plans' contents
  (`protocol_plan`), the ops over `ShowData` (`lumacore-tree`'s `output_ops`).

## Public surface

- `protocol::{all, find}` — the protocols of this build, by name.
- `Line::{setting, setting_u64, declared}`, `wire_label`, `mode_label` — a line read through its row.
- `DmxDriver`, `ControlListener` — what a protocol's driver implements.
- `InputMerger`, `InputPlan`, `InputLayer` — input sources into layers the tick merges.
- `LinePlan` — what one output line sends.
- `register` — the E-DMX codes and the protocol rows' freeze check.

## Rules

- No protocol is named here; each declares itself in its own crate. [unchecked]
- One name, one protocol; the opener fits the kind. [unchecked]
- It never depends on the tree, the renderer or anything with sockets. [compiler + tidy crate_edges_point_down]
- Nothing allocates per received packet (reused buffers, `SourceName`). [unchecked]

## Edges

- **Depends on:** lumacore-base, lumacore-runtime, lumacore-registry.
- **Used by:** lumacore-tree, lumacore-net, lumacore-artnet, lumacore-sacn, lumacore-osc,
  lumacore-render, lumacore-lines, lumacore-objects, lumacore-views, lumacore-station.

## Decisions

- [§11](../../../docs/archive/decisions-2026-10.md) — networks, lines, DMX input
- [§34](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (22, 25.14), its README (18), the catalog (21)

## Testing

- `cargo test -p lumacore-protocols` — the input merge, the line data, the error catalog.
