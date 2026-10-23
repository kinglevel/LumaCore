---
id: 0046
title: The protocol contract sits in model, below the tree
area: crates
status: built
date: 2026-10-16
built_in: 34g2c
supersedes: [archive §34 L8185–8252 "only the interface", archive §34 L8758–8767 "net on tree"]
superseded_by: —
confirmed_by: [tidy `crate_edges_point_down`, `EDGES`]
source: [archive §34 L8462–8503, archive §34 L8936–8946, archive §34 L8758–8767]
---
# The protocol contract sits in model, below the tree

## Context
34g2 found the Line types could not move "to the protocols" above the tree: the tree holds lines, so that is a cycle. Three read-only agents read QLC+, OLA, Godot, Linux, PipeWire, Chromium, Bevy and Zed (BlinderKitten the warning). The common shape: the contract — data, interfaces, pure logic — low; the model holding it by name; the implementations with sockets and threads high.

## Decision
`lumacore-protocols` lives in `crates/model/` below the tree and holds no socket and no thread: the line data (`Network`, `Line` and their kin with their `Walk` / `Identified` impls), `ProtocolDef`, the driver and listener traits, the input merger the tick reads, `LinePlan`, and `Nic` as plain data. It depends on base, runtime and registry only; the tree depends on it and its `register` calls the protocols'. `lumacore-net` lists NICs from the OS and keeps sockets, mDNS, PTP and discovery, on base, runtime and protocols — never on the tree. `crates/network/` means "has sockets or threads": net, clocksync and the implementations in `network/protocols/` (`lumacore-lines`, `dmx/` artnet · sacn, `data/` osc). The one wall-clock `now_ms` is `lumacore_base::clock::now_ms`. The layer order (each row uses only the rows above it): foundation · runtime → protocols → fixtures · tree · patch, gdtf, luma, render · state · commands and network · ui, server · station.

## Consequences
A type the show tree holds is defined at or below the tree, never in an I/O crate; a crate that opens a socket or spawns a thread is in `network/`. A new edge upward fails tidy `crate_edges_point_down` against `EDGES`. The `*_ops` over `ShowData` stay in the tree. Who declares a protocol: see protocol-crates-declare-themselves.
