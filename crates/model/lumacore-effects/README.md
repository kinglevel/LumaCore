# lumacore-effects

The effect engine data model: authored engine configurations with their graph nodes,
generators and parameters, as the tree saves them.

## Purpose

Plain data below the tree for `Root.ShowData.EffectEngines` (decisions §28): an
`EffectEngineConfig` is one authored, programmable engine — its type, whether it is enabled, a
processing graph of `EffectGraphNode`s linked by GUID, `EffectGenerator`s, named input and
output points and `EffectParameter`s. It is distinct from the basic playback `Effect` that lives
in `lumacore-playback`; this is where node-based engines, generators and custom effect logic
are authored once Effects is built. Today the crate holds the shapes and their identity walk and
nothing runs them: the decided model (effect lines on a DataPool object, forms, calls, stomp,
the cue as the sync point) is recorded and waits for its step.

## Owns / Doesn't own

- **Owns:** `EffectEnginesData`, `EffectEngineConfig`, `EffectGraphNode`, `EffectGenerator`,
  `EffectParameter`, and their `Walk` implementations.
- **Doesn't own:** the playback `Effect` and the phaser fields on `Value` (`lumacore-playback`,
  `lumacore-base`), any engine that runs an effect (`lumacore-render`, not built), the words
  that store one (`lumacore-objects`), the place in the tree (`lumacore-tree`), the Pawtern pool
  (decided, not built).

## Public surface

- `EffectEnginesData` — the pool under `ShowData`, a `BTreeMap<u32, EffectEngineConfig>`.
- `EffectEngineConfig { meta, engine_type, enabled, graph, generators, inputs, outputs,
  parameters }`.
- `EffectGraphNode { meta, node_type, connections }` — connections are GUID links.
- `EffectGenerator`, `EffectParameter` — each with a `NodeMeta`.

## Rules

- Every type carries a `NodeMeta` and `Walk`s its fields with an exhaustive destructure, so a
  link is a GUID and a new field doesn't compile until classified. [compiler]
- Nothing here runs: no tick, no clock, no value math. [tidy crate_edges_point_down]
- It depends on base only. [tidy crate_edges_point_down]
- A change to the effect model is a decision first (its records below), then code. [unchecked]

## Edges

- **Depends on:** lumacore-base.
- **Used by:** lumacore-tree (re-exported into `ShowData`).

## Decisions

- `docs/decisions/0071-effects-model-and-control.md` — effects as a DataPool object with lines, forms,
  calls and stomp (decided, not built)
- `docs/decisions/0072-effects-deterministic-anchors.md` — the cue is the sync point; phase slides on
  handover
- `docs/decisions/0071-effects-model-and-control.md` — a line holds a selection or a Group pointer
- `docs/decisions/0014-pawtern-pool.md` — Pawtern replaces Distribute and the usual pattern tools (slot reserved)
- History: [docs/archive/decisions-2026-10.md](../../../docs/archive/decisions-2026-10.md)
  §3, §28; the open items in [docs/meta/roadmap.md](../../../docs/meta/roadmap.md)

## Testing

- `cargo test -p lumacore-effects` — the crate has no tests of its own today; it compiles its
  `Walk` impls.
- Its types are walked on every load and in the identity tests of `cargo test -p lumacore-tree`
  and `cargo test -p lumacore-objects` (`identity_tests.rs`), and saved and reloaded by the demo
  golden in `cargo test -p lumacore-catalog-tests`.
