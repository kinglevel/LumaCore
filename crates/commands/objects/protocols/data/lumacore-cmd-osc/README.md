# lumacore-cmd-osc

The `Osc` object: OSC lines.

## Purpose

The `Osc` object: an OSC line (decisions §31 "OSC line users", §34 "34f7c hardened") listens on its network and port; what it receives runs as its user. Its properties are the Data lines' shared ones (`lumacore-cmd-protocols`) plus OSC's own from its row in `lumacore-osc`: `Port`. Everything a line does — Store, Delete and the door's verbs, `Set`, List, Info — is the one set of line handlers in `lumacore-cmd-protocols`, declared here with `line_object!`; this crate only names the protocol. It mirrors `crates/network/protocols/data/lumacore-osc`, which sends and receives.

## Owns / Doesn't own

- **Owns:** the `Osc` object (`OSC`), its Store options, its Import Show hook.
- **Doesn't own:** the line handlers and shared properties (`lumacore-cmd-protocols`), the protocol's row and wire (`lumacore-osc`), the plan (the kernel's `protocol_plan`), the line threads (`lumacore-lines`).

## Public surface

- `register` — the protocol's row, the shared line words, then this object and its hook.
- `OSC`, `OBJECTS`, `HOOKS`.

## Rules

- No handler of its own: every verb is the shared line handlers'. [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-syntax, lumacore-syntax-derive, lumacore-protocols,
  lumacore-objects, lumacore-cmd-protocols, lumacore-osc.
- **Used by:** lumacore-station, lumacore-sweep, lumacore-tidy, the catalog tests.

## Decisions

- [§11](../../../../../../docs/archive/decisions-2026-10.md) — networks and lines
- [§34 "34f7c hardened"](../../../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — a protocol is an object, a line one row

## Testing

- `cargo test -p lumacore-cmd-protocols`; lines with several protocols and the patch are `tests/catalog`'s (`protocols`, `lines`).
