# lumacore-fixtures

The fixture data the show holds: fixture types (with their revisions, modes, geometry and channel
functions), fixtures with their MVR part, the MVR metadata, the show's attribute definitions and
universes.

## Purpose

Data only, below the show tree (decisions §34 25.10, 25.14): `ShowData` holds these types
(`Root.ShowData.Devices.Dmx.Patch.FixtureTypes`, `.Fixtures`, `.Universes`, `Devices.Metadata.MVR`), and anything that
needs a fixture's shape without the whole show — GDTF import, the render plan's channel tier —
names this crate. A universe holds its input merge mode (the protocols' `MergeMode`, `lumacore-protocols`);
the protocol lines name universes by number, a universe never names a line (§34 "34f7c"). Every type has its GUID and walks its links (`identity`).

## Owns / Doesn't own

- **Owns:** `FixtureType`, `Revision`, `Mode`, `ChannelFunction` and the GDTF-derived parts
  (`fixture_type`); `Fixture`, `LumaFixture`, `FixtureBreak` (`fixture`); `MvrFixture`, `Matrix` (world, mm, and
  its angles), `CustomIdType` (`fixture_mvr`); `MvrMetadata` and its entries (`devices_meta`); `AttributeDefinition`,
  `FeatureGroup`, `ActivationGroup`, `AttributeRule`, `ProfileAttributes`, `EncoderBarLayout`
  (`attribute_def`); `Universe` (`universe`); their `Identified` / `Walk` impls (`identity`).
- **Doesn't own:** parsing a `.gdtf` (`lumacore-gdtf`) or an `.mvr` (`lumacore-mvr`, 34g4 part 4), the patch's rules (`lumacore-patch`), the
  ops over `ShowData` (`lumacore-tree`'s `universe_ops`, `output_ops`), the lines
  (`lumacore-protocols`).

## Public surface

- `Fixture`, `FixtureType`, `Revision`, `Mode`, `ChannelFunction`, `AttributeType` — the patch's data.
- `MvrFixture`, `Matrix`, `CustomIdType`, `MvrMetadata` — what MVR says about the patch.
- `Universe` — a universe and its merge mode.
- `AttributeDefinition`, `FeatureGroup`, `ProfileAttributes` — attributes and their layout.

## Rules

- Data only: no logic over the whole show, no I/O. [unchecked]
- Every type is classified for identity (`Walk` with an exhaustive destructure). [compiler]
- A bare `mvr` names only the format crate; the data here is named for what it maps onto
  (`fixture_mvr`, `devices_meta`). [unchecked]

## Edges

- **Depends on:** lumacore-base, lumacore-protocols.
- **Used by:** lumacore-tree, lumacore-gdtf, lumacore-patch, lumacore-render, lumacore-state, lumacore-objects.

## Decisions

- [§11, §21 "6d"](../../../docs/archive/decisions-2026-10.md) — the patch, strict identity
- [§34 25.10, 25.14](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — this crate
- [§34 "34g4 patch hardened"](../../../docs/archive/decisions-2026-10.md#34-code-structure-and-file-headers-2026-10-15) — the fixture's MVR part, the metadata

## Testing

- `cargo test -p lumacore-fixtures` — the fixture, type and universe data, the matrix and its angles.
