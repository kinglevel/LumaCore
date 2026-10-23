---
id: 0045
title: Each protocol is a crate adding its ProtocolDef row
area: protocols
status: built
date: 2026-10-16
built_in: 34f7a
supersedes: [archive §34 L8091–8099 "lines go up"]
superseded_by: —
confirmed_by: [`protocol::find`, the catalog freeze check, the E-DMX codes]
source: [archive §34 L8185–8252, archive §34 L8768–8785]
---
# Each protocol is a crate adding its ProtocolDef row

## Context
After `lumacore-dmx` held Art-Net, sACN, the merger and an `enum Sender` with a `match` arm per protocol, the question was how large projects do it: OLA's small `plugin_api` under `plugins/e131`, QLC+'s `QLCIOPlugin`, sqlx drivers on `sqlx-core`, rustls providers installed once in `main`. The common rule: a plugin sits on a small interface below the engine, never on the engine, and the binary wires the plugins in.

## Decision
`lumacore-protocols` is the interface: the `ProtocolDef` catalog row (name, kind Dmx | Data, directions, default port, capabilities `Sync` / `Discovery` / `SharedPort`, its universes and settings, its opener), the `DmxDriver` and `ControlListener` traits, and everything protocol-agnostic (`Packet`, `SourceKey`, `SourceName`, `InputMerger`, `InputLayer`, `LinePlan`, the E-DMX codes). `lumacore-artnet`, `lumacore-sacn` and `lumacore-osc` each hold their wire on `lumacore-net` + protocols and add their row in `register` (`r.rows(..)`); readers use `lumacore_registry::get().rows::<ProtocolDef>()` and `protocol::find`. `lumacore-lines` is the runtime (frame sampling, pacing, input layers to `RenderHub`, `ProtocolRuntimes::reconcile`); it finds a driver by its row and never names a protocol — no `enum Sender`. The station's `register` lists the protocol crates; nothing else does (the sweep's and the demo-loading test catalogs register them as dev-dependencies). Edges: net ← protocols ← {artnet, sacn, osc}; protocols ← render ← lines; Art-Net and sACN never see render. Dmx rows merge input per channel in the tick; Data rows turn messages into command lines run as the line's user. `lumacore-dmx` is gone.

## Consequences
A new protocol is one runtime crate with a row and, since 34f7c, one command crate for its object (see lines-one-type-runs); it touches neither `lumacore-lines` nor the state crate. Art-Net's nodes are the generic `Discovery` capability. Each crate's error codes stay in its own catalog. Placement of the interface crate: see protocol-contract-below-the-tree.
