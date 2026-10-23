# lumacore-osc

OSC input lines, declared as a Data protocol: one UDP listener per line, whose messages become
command lines run as the line's user.

## Purpose

The first Data protocol (decisions §11, §31 "OSC line users", §34 22.2). Its `register` adds
the `Osc` row — input lines only, Data kind, port 8000, its setting `Port` — so the line runtime starts a listener
per OSC line without naming it. `/lumacore/cmd <string>` runs a line, `/lumacore/go|back|stop`
run `Go`, `Go-`, `Stop`; every message goes to the state thread as an `InboundEvent` with the
line's user. Later MIDI, MSC and timecode are more Data crates like it (in `network/protocols/data/`).

## Owns / Doesn't own

- **Owns:** `OscListener`, the OSC packet decoder (`parse_osc_packet`), what a message becomes as
  the line's user, the `Osc` row, its `Port` setting and its opener.
- **Doesn't own:** which lines listen (`protocol_plan`, `lumacore-lines`), running the command and
  its rights (the state thread, dispatch), what a protocol is (`lumacore-protocols`).

## Public surface

- `OscListener::start`, `listener::open` — one input line's listener.
- `parse_osc_packet` — the decoder (fuzzed).
- `register` — its row in the catalog.

## Rules

- A line's commands run as its user, never Admin as a fallback. [unchecked]
- It depends on the interface, base, tree and the registry only. [compiler + tidy crate_edges_point_down]

## Edges

- **Depends on:** lumacore-protocols, lumacore-base, lumacore-tree, lumacore-registry.
- **Used by:** lumacore-lines (through the row), lumacore-sweep (its fuzz entry), lumacore-station;
  the test catalogs of lumacore-objects and lumacore-suggest (dev).

## Decisions

- [§11](../../../../../docs/archive/decisions-2026-10.md) — input lines
- [§31 "OSC line users"](../../../../../docs/archive/decisions-2026-10.md) — the line's user
- [§34](../../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
  (22.2), its README (18)

## Testing

- `cargo test -p lumacore-osc` — packet decoding, what a message becomes.
