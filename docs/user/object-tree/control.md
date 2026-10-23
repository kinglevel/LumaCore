---
title: Control
category: docs
---
# Control

Canonical language source: [lum.md](../syntax/lum.md)

Command syntax is in the generated [command reference](../reference/commands.md)
and [object reference](../reference/objects.md).

This document captures the current intended high-level definition of control in
LumaCore.

It is intentionally concise and conceptual. The goal is to preserve how live
operator control, stored show objects, and tree inspection should relate to one
another.

## Principle

LumaCore mixes live operator control and object-tree inspection in one shell
world.

That means:

- `Lum`
  - is the operator-facing control language
- `TreeShell`
  - is the tree-aware inspection and navigation language
- control should always map back to stable object locations

## Main Layers

Control in LumaCore should stay split into clear layers:

- live programmer control
- authored playback control
- authored datapool control objects
- runtime execution state

These layers are related, but they are not the same thing.

## Programmer

The programmer is the live working control state for a user.

It should hold:

- selection
- active values
- temporary working intent

It should not be confused with:

- saved presets
- playback output
- patch data
- datapool storage

Ownership rule (decisions §2, §4, §10):

- the `Programmer` belongs to its `User`, not to the `UserProfile`: a user can switch
  profile and keep their work
- it is **saved with the show** and shared by every station and client the user is logged
  in on (one programmer per user)
- the programmer cannot be unclaimed or transferred; it is the user's permanent live
  workspace
- values follow LTP between programmers: touching an attribute moves it out of other
  users' programmers; Blind values take nothing; claimed fixtures are refused
- **selecting does not claim**; `Claim Fixture <n>` locks a fixture
- claims are keyed by the object's GUID (`Root.Runtime.Claims`, `List Claims`);
  `Edit <obj> <n>` claims the object until you leave the editor (`Exit` / `..`); changing
  commands on an object another user holds, or anything inside it, are refused
  (E-CLAIM-001)
- `Claim <obj>` on another user's object asks them: they hand it over (`Unclaim <obj>`) or
  keep it (`Claim <obj> /Keep`); no answer in 10 s or a logged-out owner hands it over; an
  Admin takes it with `Claim <obj> /Force`. A Patch draft is never taken by timeout
- `Clear Programmer User 2 Thru 5` clears other users' programmers (Setup rights
  and up); the undo step goes on the affected user's history
- programmers of old shows start empty (they used to live on the profile)

ObjectTree alignment:

```text
Root.ShowData.Users
└── User {n}
    └── Programmer      selection + values

Root.ShowData.UserProfiles
└── UserProfile         views, screens, keys, settings
```

## Playback

Playback is authored show behavior that can run in runtime.

At a simple level it includes:

- `Sequence`
- `Cue`
- `Executor`
- `Master`
- `Effect`

ObjectTree alignment:

```text
Root.ShowData.Playback
├── Sequences
├── Effects
└── Executors

Root.ShowData.Playbacks     what runs (decisions §32), never undone
└── Live                    saved, resumed on load
    ├── Sequences           current cue, started by, fader, rate, paused
    └── Masters             2.1 Grand · 2.2 / 2.4 Highlight / Solo for everybody · group masters
```

Running playback state is saved with the show and replicated, but never undone (Oops never
undoes a Go — decisions §13). On show load every running sequence resumes snapped at its
saved cue.

## DataPool Control Objects

DataPools hold stored operator-facing show objects that should remain explicit
and inspectable.

Common examples:

- `Group`
- `Layout`
- `Macro`
- `World`
- `Filter`
- `Timecode`

ObjectTree alignment:

```text
Root.ShowData.DataPools
└── DataPool("Default")
    ├── Groups
    ├── Layouts
    ├── Macros
    ├── Worlds
    ├── Filters
    └── Timecodes
```

Important:

- datapool objects are authored show truth
- runtime control may reference them
- runtime control should not silently replace them

## Timecode

Timecode should stay split into two layers:

- authored `Timecode` objects in `DataPool`
- runtime transport state in the `Timecode Engine`

This keeps:

- authored timeline structure in show data
- live play position and sync state in runtime

## Lum Alignment

Users should be able to move between live control and stored control objects
through stable nouns.

Examples:

```text
Fixture 1 Thru 4
At 50
Store Group 1
Store Timecode 1
Info Group 1
Info Timecode 1
```

Rule:

- `Lum` should expose the control noun
- `ObjectTree` should explain where that noun lives
- ownership should stay explicit through `Claim` and `Unclaim`
- programmer ownership is implicit and always present for the user
- bare `Claim` should use current command scope when no explicit object is given
- `cd` never claims; `Enter` claims areas, `Edit` claims objects

## Design Rule

Users should be able to:

- understand where control is coming from
- inspect the tree behind what is running
- see the difference between programmer state and stored show state
- understand when a datapool object is authored truth versus runtime use
