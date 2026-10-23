# vendor/

Third-party crates LumaCore carries a changed copy of. Each is pinned over its crates.io
version with `[patch.crates-io]` in the root `Cargo.toml` and excluded from the workspace
(a path dependency, never a member; tidy's rules don't apply to it).

## gdtf (0.2.0, MIT, https://github.com/cpdt/gdtf-rs)

The GDTF reader under `lumacore-gdtf`. Decided 2026-10-20 (decisions §40 "The demo rig"):
of the 1 084 GDTFs embedded in 186 real MVR files, the unchanged crate refused 672 — the
planning tools (Capture, Depence) export a stripped GDTF per type and leave out every
attribute the spec gives a default for. The copy changes only how the XML is read; the
types, the validation and the API are upstream's. Every change is marked `// LumaCore:`.

| Where | Was | Now |
|---|---|---|
| `Attribute.PhysicalUnit` | required | default `None` (the spec's default) |
| `FeatureGroup.Pretty`, `Attribute.Pretty` | required | default `""` |
| `Attribute.Color` | `-nan(ind)` refused the file | no colour |
| every geometry's `Position` | required | identity |
| `DMXChannel.Highlight` | required (`None` or a value) | default `None` (optional in the spec) |
| `ChannelSet.DMXFrom` | required, `-1` refused the file | default `0`, a negative start is `0` |
| `Gamut.Points` | one string, never parsed right | `{x,y,Y}{x,y,Y}…` point by point, a bad point skipped |
| `Beam.ColorRenderingIndex` | `u8`, `300` refused the file | clamped to 0–255 |
| `FixtureType.Description` | required | default `""` |
| `Wheel Slot.Color` | `-nan(ind)` refused the file | white |

Upstream's own README follows in `gdtf/README.md`. To take a newer upstream: copy it over,
re-apply the rows above (search `LumaCore:`), run `cargo test -p lumacore-gdtf` and the
scan in decisions §40 against `../ExternalData/BombeeRigMetadata/mvr/`.
