---
id: 0030
title: A publish copies only what changed
area: engine
status: built
date: 2026-10-14
built_in: the code review's snapshot work (built 2026-10-14)
supersedes: [archive §26 L2490–2492 "programmer snapshot not built"]
superseded_by: —
confirmed_by: none
source: [archive §31 L6726–6737]
---
# A publish copies only what changed

## Context
After every change the state thread publishes a frozen `Arc<ShowData>`; on a rig an `At` republished the whole show. §26 had promised a programmer snapshot; the benchmarks put the cost here, so this was ordered before typed replies (asked again 2026-10-14).

## Decision
The programmer snapshot comes first, then `ShowData`'s fields become `Arc` parts one at a time (`Arc::make_mut` on write), so a publish copies only what changed. Built: an `Arc` per sequence, fixture type and user profile (copy-on-write; the saved form unchanged, `FORMAT_VERSION` untouched by this). The change path passes its set of changed subtrees on: in `EngineInput` (the plan compiler redoes only those; a running sequence's offers stay in `SequenceCache` while its `Arc` and cue stay) and to the feed hub's revisions (a pool window follows its datapool, not the whole show). Feed revisions stay content fingerprints, never counters. The merge is keyed by the tier's attribute index.

## Consequences
A new big `ShowData` field is an `Arc` part from the start. A line's cost on a big show is the change path's two fingerprints — never add a second walk of the show per line (see the change path). Typed replies follow this work (see typed-replies-text-at-edge); `DispatchResult::Output` carries `Reply { text, error }` when built. The feed hub rebuilds only when `lumacore_http::Revisions` move, never on a timer.
