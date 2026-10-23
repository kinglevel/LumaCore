---
id: 0231
title: The station's modules become crates in their groups
area: crates
status: built
date: 2026-10-16
built_in: 34f9–34f10e
supersedes: [archive §34 L8091–8094 "lumacore-dmx splits along", archive §34 L8320–8321 "net/sync.rs"]
superseded_by: —
confirmed_by: [crate_edges_point_down, the_station_only_assembles]
source: [archive §34 L8091–8118 (items 20.3–20.6, 20.10), archive §34 L8315–8331 (24.2), archive §34 L8803–8891]
---
# The station's modules become crates in their groups

## Context
After 34f8 put the crates in folders, 24.2 named where each of the station's remaining modules goes, so none would be born outside its group. The hub was the knot: `ServerContext` held every layer's state, so LumaSession, the shell port and the state loop had to be cut on narrow host traits rather than moved whole. `net/sync.rs` was planned for the shell and stayed with the server (it builds `Root.Runtime`'s network view); the sweep waited for LumaSession.

## Decision
The station's code lives in crates, each on a host trait where it needed the hub: `engine/lumacore-engines` (the `Engine` trait, the registry, the render / HTTP / Perfetto wrappers); `network/lumacore-clocksync` (the session clock discipline); `foundation/lumacore-lanes` (`StateHandle<C>`, the two lanes, the busy watchdog, generic over a `Host`); `ui/lumacore-shell` (the TCP shell port on a `ShellHost`); `server/lumacore-lumasession` (link, election, stream, replication, the `LumaSession` struct, on a `SessionHost` the server implements); `server/lumacore-server` (`ServerBuilder`, `ServerContext`, the state loop, the command port, the hub files); `dev/lumacore-sweep` (the sweep and `fuzzing`, on objects, LumaSession and the protocols, never the server — so a fuzz build never compiles the server). The render crate holds the plan, compiler and tick; the line drivers sit in `lumacore-lines`. A module renames its `mod.rs` and gets its README and headers as it moves.

## Consequences
A new piece of server-side work goes into the crate that owns it, not the server, unless it is the hub itself; a host trait grows a method instead of a crate reaching for `ServerContext`. Tidy's `CUT_OUT` lets the station-reading rules still see the moved code under its old folder name. Open: none.
