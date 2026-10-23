---
title: Network Engine
category: docs
---
# Network Engine

Canonical language source: `docs/user/syntax/lum.md`

This document captures the current intended high-level definition of the
`Network Engine` in LumaCore.

It is intentionally concise and conceptual. The goal is to preserve the engine
boundary that owns live network services, discovery, interfaces, and transport
plumbing without confusing that with LumaSession semantics or output rendering.

> **Status (2026-10-11).** "Network Engine" is a role, not a registry engine: the
> registry engines are only `RenderEngine`, `PerfettoEngine` and `HttpUiEngine`
> (decisions §26/§27). The role is filled by the station's `net/` module (§23 "Step 8":
> shell server, UDP command port, discovery beacon, mDNS, HTTP bridge, menu views, PTP
> reader), with per-NIC filtering in `lumacore_lumasession::ServiceAccess`. LumaSession meaning
> lives in `lumasession/` ([lumasession.md](./lumasession.md)); protocol output lives in
> `lumacore_lines::runtimes` and `lumacore_lines::lines` (§11). Ports and protocol numbers:
> [lumasession.md](./lumasession.md) "Ports".

## Principle

The `Network Engine` should own network-facing runtime infrastructure.

That means:

- `Network Engine`
  - owns sockets, listeners, discovery, and wire transport services
- LumaSession (`lumasession/`)
  - owns membership, authority, and reconnect semantics
- protocol drivers (`lumacore_lines::runtimes`, `lumacore_lines::lines`)
  - own protocol-specific DMX or device publication
- `Integration Engine`
  - owns higher-level external system bridges built on top of network services

This follows the strongest local patterns:

- established consoles separate session/runtime networking from standard output protocols
- OLA shows protocol and device distribution behind a service layer
- QLC+ uses one network manager for discovery, TCP server/client communication,
  and network role handling
- openDAW separates document sync from signaling transport

## What It Owns

The `Network Engine` should own:

- interface enumeration and selection
- listener bind/unbind lifecycle
- UDP/TCP/WebSocket service ownership
- service advertisement and discovery
- host reachability and connection health
- transport routing between runtime services
- shared network diagnostics
- server endpoint configuration
- backpressure, buffering, and connection cleanup

It should not own:

- session membership meaning
- user authorization semantics beyond transport admission basics
- show playback
- render logic
- output protocol semantics that belong to specialized transport engines

## Current Shape

What the tree holds today:

```text
Root.Runtime
├── Network              data networks: Interfaces, Lines, Nodes, Sources
├── LumaSessions         Stations (discovery), Members, Listeners, Connections
├── Health               Render, Lines, Clock, Threads (incl. the network threads)
└── NetworkEngines       placeholder node, nothing registers there
Root.StationSettings     NIC services, network → NIC map (station-local)
```

Notes:

- `Root.Runtime.Network`
  - is live observable network state
- `Root.Runtime.LumaSessions`
  - holds discovery results, members, listeners and connections
- `Root.Runtime.Health`
  - every network thread registers there (`HealthBoard`)
- protocol lines and routes stay in `ShowData.Protocols`; the network → NIC mapping is
  station-local (`Root.StationSettings.networks`)

## Boundaries

### Network Engine Vs LumaSession

The `Network Engine` moves bytes and maintains service presence.

LumaSession (`lumasession/`, [lumasession.md](./lumasession.md)) decides what those bytes
mean for:

- membership
- reconnect
- authority
- client role/profile

### Network Engine Vs Protocol Drivers

The `Network Engine` owns shared network infrastructure.

Protocol drivers (one per protocol and NIC, `lumacore_lines::lines`, started by
`lumacore_lines::runtimes`) own protocol-specific publication such as:

- `sACN`
- `ArtNet`
- future remote device protocols

This matters because:

- output transport is not the whole network story
- sessions and integrations also need sockets, discovery, and health

### Network Engine Vs Integration Engine

The `Network Engine` provides shared transport and discovery services.

The `Integration Engine` uses those services for:

- REST APIs
- WebSocket fanout
- MVR-xchange
- MIDI/OSC bridges where network-facing

## Behavioral Rules

### 1. One Owner Per Bound Service

Shared listeners and well-known ports should be owned once by the runtime
owner.

That means:

- no duplicate bind attempts from CLI surfaces
- helper scripts orchestrate startup but do not become listeners
- service ownership remains server-side

### 2. Discovery Is First-Class

Discovery should be a named engine responsibility.

That includes:

- server discovery
- station discovery
- interface availability
- protocol discovery where relevant

### 3. Transport Layers Stay Separable

Sync, signaling, command ingress, and output publication may share runtime
services, but they should remain distinct transport lanes.

openDAW is useful here because it separates:

- document sync WebSocket service
- signaling WebSocket service

That suggests LumaCore should keep:

- command transport
- shell transport
- session sync transport
- signaling or integration transports

conceptually separate even when they share a host process.

Built that way: the UDP command port (:47653), the TCP shell port (:47654), discovery
(:47655), HTTP (:47656) and the LumaSession stream (:47658) are separate listeners; the
session clock's `Time` pings ride the command port so they never queue behind a snapshot
on the stream.

### 4. Interfaces Are Station-Local

Interface selection and local network tuning belong with machine-local runtime
or station settings, not with saved show truth.

Built: `Set Interface en1 LumaSession|Shell|HTTP Yes|No` and the network → NIC map are
`StationSettings`, never replicated; loopback always serves; `Set Station "Online" Off`
stops everything but loopback.

### 5. Health Must Be Observable

Network health should be inspectable at runtime:

- active listeners
- active connections
- discovery status
- dropped or failed connections
- interface state

## Repo Signals

Strong local references:

- `$SKILL/build/tools/lighting/repos/qlcplus/qmlui/tardis/networkmanager.h`
  - one manager owning UDP discovery, TCP server/client communication, and
    server/client roles
- `$SKILL/build/tools/lighting/repos/qlcplus/qmlui/tardis/networkmanager.cpp`
  - concrete announce, bind, packet send, and connection cleanup behavior
- `$SKILL/build/tools/audio/repos/openDAW/packages/server/yjs-server/server.js`
  - split sync and signaling services on one HTTPS server
- `$SKILL/build/tools/lighting/repos/ola/ola Analysis.md`
  - daemon/plugins architecture for protocol and device distribution
- [net/mod.rs](../../../crates/server/lumacore-server/src/net.rs),
  [net/spawn.rs](../../../crates/server/lumacore-server/src/net/spawn.rs)
  - command port, shell port, discovery, and their health counters
- [session.rs](../../../crates/server/lumacore-lumasession/src/session.rs)
  - `ServiceAccess`: per-NIC filtering of LumaSession, Shell and HTTP

## Improvement Suggestion

The best next improvement is to make `Root.Runtime.Network` and
`Root.Runtime.NetworkEngines` explicit in
[object-tree.md](../../user/object-tree/object-tree.md) (or drop the empty
`NetworkEngines` node), while keeping `ShowData.Protocols` as authored protocol
assignment truth.

That keeps:

- shared network infrastructure observable
- output transport engines specialized
- session semantics separate from low-level network ownership
