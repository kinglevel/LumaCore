---
title: LumaCore
category: docs
---
# LumaCore

What LumaCore is, and how `Lum`, the object tree and datapools fit together.
Short and conceptual; the language is in [lum.md](../syntax/lum.md), every word's
syntax in the [reference](../reference/commands.md).

## Identity

`LumaCore` is an open lighting control system built around a terminal-first
console workflow.

At the simplest level, it is:

- a shared show-control runtime (one server owns the show)
- a tree-shaped show model
- a console-style command language
- a live output system for lighting (sACN, Art-Net) and later show engines

It is not:

- just a terminal app
- just a DMX sender
- just a GDTF parser
- just a web app

## Core names

- `LumaCore` — the whole system
- `Luma` — the cat, and the runtime cursor that moves through the system
- `Lum` — the command language
- `TreeShell` — the tree-aware shell for navigation and inspection
  ([tree-shell.md](../terminal/tree-shell.md))

## Object tree

Everything is one tree, `Root`:

```text
Root
├── ShowData          the show: saved, replicated to every station
├── Runtime           live session state: replicated, never saved
├── StationSettings   this station only: never replicated
├── UIEngines         the UI engines rendering screens (HTTP / web)
├── Resources         built-in assets
│   ├── WindowTypes   the kinds of window (Terminal, Groups, Realtime, …)
│   └── UIKeys        the core UI keys
├── Diagnostics       build info, message and status centers
├── Library           files on disk: GDTF, shows, presets, MVR
└── Drives            this machine's volumes and their files (read-only view)
```

Every field has exactly one home:

- `ShowData` is the authored show. Only the LumaSession master changes it.
- `Runtime` is live execution: logged-in users and their modes, membership,
  clock, claims, health, journal.
- `StationSettings` (and interfaces, the network → NIC map) belong to this
  machine.

The full tree is in [object-tree.md](../object-tree/object-tree.md); each object
keyword and its tree place is in the
[object reference](../reference/objects.md).

## DataPool

`DataPool` is the main container for reusable operator objects in the show:

```text
Root.ShowData.DataPools
└── DataPool 1
    ├── Groups             fixtures in order with grid spots
    ├── Sequences          cues and their timing
    ├── TimingPresets      reusable cue timing
    ├── CurvePresets       reusable fade curves
    ├── PresetGroups       preset pools — Store and Info are Planned in the reference
    ├── Macros             the macro pool — Go, Store and Info are Planned
    ├── Timecodes          a pool you can select — nothing runs a timeline yet
    ├── Layouts
    ├── Filters
    ├── Worlds
    ├── DistributePresets
    ├── DynamicStates
    ├── Plugins            plugins and their UI keys
    └── Priorities
```

Groups, sequences and the timing and curve presets live in your selected datapool.
The other branches exist in the tree before their features are built: what a word
can do today is the [object reference](../reference/objects.md), which marks the
rest **planned — not built yet**; the designs are under
[visions/](../../../visions/README.md).

Operator nouns should map to datapool branches instead of floating as
undocumented ideas.

## Main domains

- fixtures and patch
- programmer control
- playback
- datapools
- output (networks, lines, routes)
- LumaSessions (stations working together)
- users, rights and claims
- integrations (OSC input, later MIDI and timecode)

## Design rule

LumaCore prefers:

- explicit nouns
- stable tree locations
- one authoritative runtime (the server; the master in a LumaSession)
- the same engine behind every client (terminal, app, web)

It avoids:

- parallel runtimes
- UI-only truth
- hidden state users can't inspect
