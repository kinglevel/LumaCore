---
title: Runtime
category: docs
---
# Runtime

Canonical language source: `docs/user/syntax/lum.md`

This document captures the current intended high-level definition of runtime in
LumaCore.

It is intentionally concise and conceptual. The goal is to preserve what the
shared live runtime owns and how it stays separate from saved show truth.

## Principle

LumaCore should run as one shared runtime with clear ownership.

Main roles:

- `lumacore-server`
  - the runtime owner
- `lumacore-cli`
  - a connector and terminal client
- scripts
  - local orchestration helpers

## Ownership Rule

The server owns:

- the real application state, on one state thread (decisions §26)
- runtime services
- command authorization (one rights gate in dispatch for every transport, §19)
- Shell sessions and LumaSession membership
- live engine state

The CLI owns:

- connecting
- login and attach flow
- terminal rendering for the attached user
- reconnect and detach behavior

Scripts orchestrate:

- start
- stop
- connect
- rebuild and restart flow

Scripts should not own runtime policy.

## ObjectTree Alignment

Runtime should stay explicit in the tree:

```text
Root.Runtime
├── Network            data networks: Interfaces, Lines, Nodes, Sources
├── LumaSessions       Stations, Members, Listeners, Connections
├── NetworkEngines
├── Users              {n}: Modes, Undo, Screens
├── SessionEngines     placeholder
├── RenderingEngines
├── Devices            reserved for what devices report back
├── GlobalVariables
├── Journal            the last 1000 changes
└── Health             Threads.*
```

Important:

- runtime branches are live truth
- they are not saved datapool objects
- they are not replacements for `ShowData`
- `Runtime` is **session-wide**: replicated to every station of a LumaSession (6b), never
  saved. Station-local state (`Root.StationSettings`, interfaces, UI, command lines,
  screens, clients) is never replicated (decisions §10).
- running playback state is **not** runtime: it is `ShowData.Playbacks` (saved, replicated,
  never undone — §13)

## LumaSessions And Shells

A LumaSession (server-to-server) is runtime state, not show data
(`Root.Runtime.LumaSessions`; see [lumasession.md](../networking/lumasession.md)). Terminal
sessions are the `Shell` object: they belong to the server, outlive individual client
connections and support reattach.

Per-user runtime:

```text
Root.Runtime.Users
└── {n}                 RuntimeUser, numbered like ShowData.Users (decisions §10/§16)
    ├── Modes           Blind / Highlight / Freeze / Solo (never saved; start off)
    ├── Undo            the user's undo history (never saved)
    └── Screens    screens → Windows
```

The user's programmer and selection are **not** runtime: they live on
`ShowData.Users.{n}.Programmer`, are saved with the show and shared by every station and
client the user is logged in on.

## DataPool Vs Runtime

`DataPool` objects are authored show truth.

Examples:

- `Group`
- `Macro`
- `Layout`
- `Timecode`

These live under:

```text
Root.ShowData.DataPools.DataPool(...)
```

Runtime may execute, reference, or inspect them, but should not redefine where
they belong.

## Runtime State

Runtime state includes:

- LumaSession membership and logged-in users (modes, undo)
- watches
- loaded/next cue (running state itself is `ShowData.Playbacks`)
- output activity
- engine status
- connected clients
- current network health

Runtime state should not be treated as saved show truth.

## Lum Alignment

`Lum` should make runtime inspectable without confusing it with stored show
objects.

Examples:

```text
Info Server
List LumaSession
List Shell
Info Shell 1
Watch Clock
```

Rule:

- use `Lum` nouns for inspectable runtime concepts
- keep authored object nouns mapped to `ShowData` and `DataPool` branches
