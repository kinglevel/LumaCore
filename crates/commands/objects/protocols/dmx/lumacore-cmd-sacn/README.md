# lumacore-cmd-sacn

The `Sacn` object: sACN lines.

## Purpose

The `Sacn` object: an sACN line is one DMX Protocols row (decisions §34 "34f7c hardened") — `Sacn 1` on its network sends (Mode Output) or receives (Mode Input) a run of local universes as sACN universes. Its properties are the DMX lines' shared ones (`lumacore-cmd-protocols`) plus sACN's own from its row in `lumacore-sacn`: `SacnUniverse` and `Priority`. Everything a line does — Store, Delete and the door's verbs, `Set`, List, Info, Assign / Unassign — is the one set of line handlers in `lumacore-cmd-protocols`, declared here with `line_object!` and `dmx_line_verbs!`; this crate only names the protocol. It mirrors `crates/network/protocols/dmx/lumacore-sacn`, which sends and receives.

## Owns / Doesn't own

- **Owns:** the `Sacn` object (`SACN`), its Store options, its Import Show hook.
- **Doesn't own:** the line handlers and shared properties (`lumacore-cmd-protocols`), the protocol's row and wire (`lumacore-sacn`), the plan (the kernel's `protocol_plan`), the line threads (`lumacore-lines`).

## Public surface

- `register` — the protocol's row, the shared line words, then this object and its hook.
- `SACN`, `OBJECTS`, `HOOKS`.

## Rules

- No handler of its own: every verb is the shared line handlers'. [unchecked]

## Edges

- **Depends on:** lumacore-registry, lumacore-syntax, lumacore-syntax-derive, lumacore-protocols,
  lumacore-objects, lumacore-cmd-protocols, lumacore-sacn.
- **Used by:** lumacore-station, lumacore-sweep, lumacore-tidy, the catalog tests.

## Decisions

- [§11](../../../../../../docs/archive/decisions-2026-10.md) — networks and lines
- [§34 "34f7c hardened"](../../../../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — a protocol is an object, a line one row

## Testing

- `cargo test -p lumacore-cmd-protocols`; lines with several protocols and the patch are `tests/catalog`'s (`protocols`, `lines`).
