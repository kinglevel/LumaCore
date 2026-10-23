# lumacore-cmd-protocols

The protocol lines' shared words: the one set of line handlers every protocol's object plugs
into, `DMX [In]`, and the console's DMX Protocols menu.

## Purpose

A protocol is an object and a line is one DMX Protocols row (decisions §34 "34f7c
hardened"): `Sacn 1`, `ArtNet 1`, `Osc 1`. Each protocol's object is its own crate's under
`commands/objects/protocols/` (`lumacore-cmd-sacn`, `-artnet`, `-osc`, mirroring
`network/protocols/`); this crate holds what they share, written once — "never duplicate per
protocol":

- `line_object!` builds a protocol's object: Store (its options are the line's declared
  settings), the door's verbs (Delete with its one ask, Copy, Cut, Paste, Move, Insert, Compact),
  typed List and Info (bare `Info ArtNet` is the protocol itself), the properties and the tree
  under `Protocols.DMXProtocols` / `Protocols.DataProtocols`.
- `dmx_line_verbs!` adds a DMX protocol's `Assign <P> <n> At Universe …` (an input run, Assign's
  order is the data flow) and `Unassign <P> <n>`.
- `props`: what every line of a kind has, declared once (DMX: Mode, LocalUniverse, Amount,
  Target, Rate, Send, PacketGap, Sync, InputPriority; Data: the user), and the builder of a
  protocol's own from its row (`SacnUniverse` / `PortAddress`, `Priority`, `Port`).
- `lines`: Store, the write checks (a line's run fits its protocol's range, two outputs never send
  one protocol universe to one destination, a protocol's lines keep its modes), the tree views.

A line names its Network, which is `lumacore-cmd-station`'s (the station's network, decision
25.17) — this crate sits on it. `Assign Universe 5 Thru 8 At Sacn 1` is the Universe's verb (the
kernel's), which finds the line by its object's tree, never by naming a protocol. The drivers are
`lumacore-lines` and the protocol crates.

## Owns / Doesn't own

- **Owns:** the shared line handlers and properties (`lines`, `props`), `line_object!`,
  `dmx_line_verbs!`, DMX In (`dmx`), the DMX Protocols menu (`menus`), the rows (`rows`: DMX).
- **Doesn't own:** each protocol's object (`lumacore-cmd-sacn`, `-artnet`, `-osc`), the Network
  and the station's NICs (`lumacore-cmd-station`), the Universe's Assign / Unassign (the
  kernel's), the plan (the kernel's `protocol_plan`), the drivers (`lumacore-lines`).

## Public surface

- `register` — DMX In and the menu, after the station's.
- `line_object!`, `dmx_line_verbs!`, `lines::*`, `props::*`.

## Rules

- One `Line` type and one set of handlers for every protocol. [unchecked]
- A line on an unmapped network never sends. [test protocol_assign_creates_a_line_that_sends_only_once_mapped]
- A line carries one run of universes. [test protocol_a_line_carries_one_run_and_needs_no_claim]

## Edges

- **Depends on:** lumacore-registry, lumacore-wire, lumacore-syntax, lumacore-syntax-derive,
  lumacore-tree, lumacore-protocols, lumacore-objects, lumacore-cmd-station.
- **Used by:** lumacore-cmd-sacn, lumacore-cmd-artnet, lumacore-cmd-osc, lumacore-server,
  lumacore-terminal, lumacore-sweep, lumacore-tidy.

## Decisions

- [§11](../../../../docs/archive/decisions-2026-10.md) — networks, lines, DMX input
- [§34 25.17](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate, on the station's network
- [§34 "34f7c hardened"](../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — a protocol is an object, a line one row

## Testing

- `cargo test -p lumacore-cmd-protocols` — DMX In; the lines with the protocols, the station's
  network and the patch are `tests/catalog`'s (`lines`, `protocols`).
