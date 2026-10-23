---
title: Object Tree
category: docs
---
# Object Tree

Canonical language source: [lum.md](../syntax/lum.md)

Every object keyword, its verbs, its tree place and its writable properties are in
the generated [object reference](../reference/objects.md) (it replaces the old
`all-objects.md` catalog, archived in `docs/archive/user/object-tree/`). To walk
the tree, see [tree-shell.md](../terminal/tree-shell.md).

This document captures the current intended high-level object tree direction for
the full `Root` structure, with extra focus on `ShowData.Playback`,
`ShowData.DataPools`, `ShowData.EffectEngines`, `Runtime`, and `UIEngines`.

It is intentionally concise and conceptual. The goal is to preserve the shape
and meaning of the tree before every detail is fully implemented.

## Root

```text
Root
├── ShowData
│   ├── ShowSettings
│   ├── Devices
│   │   ├── Dmx
│   │   │   └── Patch      fixture types, fixtures, universes
│   │   └── Metadata
│   │       └── MVR        layers, groups, classes, positions, mapping definitions, files
│   ├── 3DSpatial
│   ├── Playback
│   ├── Playbacks          running playback state (saved, replicated, never undone)
│   ├── DataPools
│   ├── Users
│   │   └── User {n}       login, rights level, profile link
│   │       └── Programmer selection + values (saved, shared by the user's stations)
│   ├── UserProfiles
│   │   └── UserProfile
│   │       ├── Views
│   │       ├── KeyBindings
│   │       ├── Preferences
│   │       ├── Permissions
│   │       └── UserVariables
│   ├── Protocols
│   │   ├── Networks
│   │   ├── DMXProtocols      Sacn, ArtNet — each protocol's lines by number
│   │   ├── DataProtocols     Osc
│   │   └── Bidirectional
│   ├── EffectEngines
│   └── Variables
├── Runtime
│   ├── Network            data networks: Interfaces, Lines, Nodes, Sources
│   ├── LumaSessions       Stations, Members, Listeners, Connections
│   ├── NetworkEngines
│   ├── Users
│   │   └── {n}            Modes, Undo, Screens.{n}.Windows.{no}
│   ├── SessionEngines
│   ├── RenderingEngines
│   ├── Devices
│   ├── GlobalVariables
│   ├── Journal
│   └── Health
├── UIEngines
│   └── UIEngine*          (e.g. 1 "HTTP")
├── Resources
│   └── WindowTypes
├── Diagnostics
├── StationSettings
└── Library
```

## Root Branches

### ShowData

`ShowData` is the saved show truth.

It contains the authored objects that should travel with the show:

- `ShowSettings`
- `Devices`
- `3DSpatial`
- `Playback`
- `Playbacks`
- `DataPools`
- `Users`
- `UserProfiles`
- `Protocols`
- `EffectEngines`
- `Variables`

Variable tiers:

- `ShowVar`
  - `Root.ShowData.Variables`
- `UserVar`
  - `Root.ShowData.UserProfiles.{Username}.UserVariables`

Important syntax alignment:

- `Fixture`
  - lives under `Root.ShowData.Devices.Dmx.Patch.Fixtures`
- `FixtureType`
  - lives under `Root.ShowData.Devices.Dmx.Patch.FixtureTypes`
- `Universe`
  - lives under `Root.ShowData.Devices.Dmx.Patch.Universes`
- `Sacn`, `ArtNet`, `Osc` (a protocol's line)
  - lives under `Root.ShowData.Protocols.DMXProtocols.Sacn` (`…DataProtocols.Osc`)
- `Sequence`
  - lives under `Root.ShowData.Playback.Sequences`
- `Effect`
  - lives under `Root.ShowData.Playback.Effects`
- `TimingPreset`
  - lives under `Root.ShowData.Playback.TimingPresets`
- `CurvePreset`
  - lives under `Root.ShowData.Playback.CurvePresets`
- `Executor`
  - lives under `Root.ShowData.Playback.Executors`
- `Group`
  - lives under `Root.ShowData.DataPools.DataPool(...).Groups`
- `Layout`
  - lives under `Root.ShowData.DataPools.DataPool(...).Layouts`
- `Filter`
  - lives under `Root.ShowData.DataPools.DataPool(...).Filters`
- `World`
  - lives under `Root.ShowData.DataPools.DataPool(...).Worlds`
- `Macro`
  - lives under `Root.ShowData.DataPools.DataPool(...).Macros`
- `Timecode`
  - lives under `Root.ShowData.DataPools.DataPool(...).Timecodes`
- `DynamicState`
  - lives under `Root.ShowData.DataPools.DataPool(...).DynamicStates`
- `User`
  - lives under `Root.ShowData.Users`; each user owns its `Programmer` (selection and
    values) and points at a `UserProfile` (views, screens, keys, settings) — decisions §10
- `UserProfile`
  - lives under `Root.ShowData.UserProfiles`
- `RenderEngine`
  - lives under `Root.Runtime.RenderingEngines`
- `Shell`
  - lists the logged-in users and their shell clients; their runtime state is
    `Root.Runtime.Users.{n}` (`Modes`: Blind, Highlight, Freeze, Solo — decisions §16;
    `Undo`; `Screens`)
- `LumaSession`
  - the server-to-server mesh, under `Root.Runtime.LumaSessions`
- `UIEngine`
  - lives under `Root.UIEngines`
- `EffectEngine`
  - lives under `Root.ShowData.EffectEngines`

### Runtime

`Runtime` is live execution state.

This is where active runtime-only data belongs, such as:

- logged-in users and their modes and undo history
- LumaSession membership
- active runtime variables
- live rendering/output state
- engine health and execution status

`Runtime` is never saved. Its session-wide parts (users' modes and undo, membership,
claims) are replicated to every station of a LumaSession (from step 6b); station-local
state lives in `Root.StationSettings` and is never replicated. Running playback state is
not runtime: it is `ShowData.Playbacks`.

```text
Root.Runtime
├── Network
├── LumaSessions
├── NetworkEngines
├── Users
├── SessionEngines
├── RenderingEngines
├── Devices
├── GlobalVariables
├── Journal
└── Health
```

This can grow later as the runtime becomes more explicit.

Important engine alignment:

- `Network`
  - is live observable data-network state (interfaces, line status, Art-Net nodes,
    input sources), not authored protocol assignment
- `LumaSessions`
  - is the server-to-server mesh: discovered `Stations`, `Members` known to the
    master, `Listeners`, `Connections`
- `NetworkEngines`
  - are shared runtime network service engines
- `Users`
  - are the logged-in users, numbered like `ShowData.Users`
  - `Users.{n}.Modes` holds Blind / Highlight / Freeze / Solo; `Users.{n}.Undo` the
    user's undo history (never saved)
  - `Users.{n}.Screens.{v}` is one screen (desktop, tablet, phone…)
    of that user: a split-tree layout of numbered `Windows`
  - `…Screens.{n}.Windows.{no}` is one live window: its window type,
    typed settings (validated against `Root.Resources.WindowTypes`),
    appearance and scroll position
  - windows are edited here: `cd` into a window and `set <key> <value>`, or
    `set <path> <key> <value>`; every UI engine showing it updates live
- `SessionEngines`
  - is a placeholder; LumaSession logic is not a registry engine
- `RenderingEngines`
  - are real runtime engines
- `Journal`
  - the last 1000 changes (`List Journal`, `Watch Events`)
- `Health`
  - thread health (`Health.Threads.*`)
- runtime transport and timecode execution belong conceptually in runtime even when
  their exact tree branches are still evolving

Variable tiers:

- `GlobalVar`
  - `Root.Runtime.GlobalVariables`

### UIEngines

`UIEngines` contains the live presentation layer: *how* things are rendered,
not *what*.

```text
Root.UIEngines
└── UIEngine*      e.g. 1 "HTTP" — kind, status, address, live_screens served
```

Notes:

- `UIEngine` is the language-facing singular noun.
- A UI engine renders screens; it does not own them. What is on screen
  (windows, their settings and placement) lives in
  `Root.Runtime.Users.{User}.Screens`, so different engines (web today,
  other renderers later) can show the same windows.
- Window types and their settings schemas are renderer-agnostic and live in
  `Root.Resources.WindowTypes`.
- Saved user views belong in `ShowData.UserProfiles` (stored by copying a
  screen's windows; recall copies them back).

### Resources

`Resources` is the asset domain.

This is the home for resolved or show-usable assets such as:

- images
- textures
- videos
- audio
- models
- materials
- shaders

### Diagnostics

`Diagnostics` is a top-level system concern.

It should contain system-wide observability and health information, such as:

- messages
- warnings
- status
- performance
- runtime health
- crash information

### StationSettings

`StationSettings` holds machine-local settings that do not travel with the
show.

Examples:

- station identity
- network interfaces
- protocol IDs
- machine-local tuning

Important syntax alignment:

- `StationSettings`
  - maps to `Root.StationSettings`

### Library

`Library` is both a filesystem directory (`library/`) and a root-level branch
in the object tree. It provides shortcut access to fixture libraries, shows,
logs, and imported resources. The software will not start without `library/`
present — it is a hard requirement, not auto-created.

Examples:

- fixture libraries
- shows
- imported resources
- GDTF / MVR content

## Devices

```text
Root.ShowData.Devices
├── Dmx
│   └── Patch
│       ├── FixtureTypes
│       ├── Fixtures
│       └── Universes
└── Metadata
    └── MVR
        ├── Layers
        ├── Groups
        ├── Classes
        ├── Positions
        ├── MappingDefinitions
        └── Files
```

Notes (history: §34 "34g4 patch hardened" of [the frozen decision log](../../archive/decisions-2026-10.md); the rules are the patch records in [docs/decisions/](../../decisions/)):

- A node per device kind: `Dmx` today; later kinds on other base protocols get their own.
- A fixture is its meta (the MVR uuid as its GUID, its name, its note), its `Mvr` part (every
  MVR 1.6 `<Fixture>` field: type, revision and mode links, breaks, one world matrix, the second
  id `IDType` + `CID`, …) and its `Luma` part (what MVR lacks: MasterReact). Its number is the
  fixture's own (MVR's FixtureIDNumeric).
- `Metadata.MVR` holds what MVR files bring besides fixtures, by UUID; the patch and 3DSpatial
  share it. Imports fill it (34g4 part 4).

## Playback

```text
Root.ShowData.Playback
├── Sequences
│   └── Sequence
│       ├── Cues
│       ├── Tracks
│       ├── Parts
│       ├── Assignments
│       └── Priority
├── Masters
├── Effects
├── TimingPresets
├── CurvePresets
└── Executors
```

Notes:

- `Sequences` are the main timeline/show-running objects.
- `Cues` live inside `Sequence`, not as a separate top-level pool.
- `Masters` are playback-facing control masters.
- `Effects` are standalone playback effects.
- `TimingPresets` define transitions of values on fixtures.
- `CurvePresets` define fade shapes such as S-curves.
- `Executors` are control handles for running or triggering playback-oriented
  behavior and related devices.

## DataPools

```text
Root.ShowData.DataPools
├── DatapoolPriorityOrders
├── DataPool("SongName1")
├── DataPool("SongName2")
├── DataPool("SongName3")
├── DataPool("SongName4")
└── DataPool("SongName5")
```

Notes:

- `DatapoolPriorityOrders` defines how datapools stack against each other.
- Higher-numbered datapools take priority over lower-numbered datapools.
- Datapools can represent songs, themes, blackouts, gameshows, storage pools,
  or user-shareable collections.
- Each datapool still contains its own reusable object collections.

### DataPool Contents

```text
Root.ShowData.DataPools
└── DataPool("SongName")
    ├── Groups
    ├── PresetPools
    ├── Layouts
    ├── Filters
    ├── Worlds
    ├── Macros
    ├── DistributePresets
    ├── DynamicStates
    ├── Plugins
    ├── Timecodes
    └── Priorities
```

Notes:

- `Distribute` is the command.
- `DistributePresets` are the stored distribution objects used by that command.
- `DynamicStates` are stateful, logic-oriented show objects, not timelines and
  not classic full-scene recalls.
- `Timecodes` belong to the datapool layer, not directly to playback
  timelines.
- `Priorities` are local priorities inside a datapool.

## Variables

```text
Root
├── ShowData
│   ├── Variables
│   └── UserProfiles
│       └── UserProfile
│           └── UserVariables
└── Runtime
    └── GlobalVariables
```

Variable nouns in `Lum`:

- `ShowVar`
  - saved with the show
  - stored in `Root.ShowData.Variables`
- `UserVar`
  - saved with the user profile
  - stored in `Root.ShowData.UserProfiles.{Username}.UserVariables`
- `GlobalVar`
  - runtime only
  - stored in `Root.Runtime.GlobalVariables`

## Lum Alignment

The object tree should follow `Lum` noun usage as closely as possible.

That means:

- language-facing object nouns should map cleanly to real tree branches
- singular `Lum` nouns may point into plural tree collections
- `Set`, `Assign`, `Info`, and `List` should read naturally against the tree

Examples:

```text
Info Fixture 1
Set Fixture 1 "Name" "Front Left"
Assign Universe 1 At Sacn 1 /Universe 1
Info User "LD"
Set UserProfile "Verbosity" Summary
Set Station "Name" "Luma01"
Set UserVar Name "FrontTruss"     -- planned: variables are a design for later (visions/macros.md)
```

These should map conceptually to:

```text
Fixture      -> Root.ShowData.Devices.Dmx.Patch.Fixtures
Universe     -> Root.ShowData.Devices.Dmx.Patch.Universes
Sacn         -> Root.ShowData.Protocols.DMXProtocols.Sacn
ArtNet       -> Root.ShowData.Protocols.DMXProtocols.ArtNet
Osc          -> Root.ShowData.Protocols.DataProtocols.Osc
Network      -> Root.ShowData.Protocols.Networks
Executor     -> Root.ShowData.Playback.Executors
User         -> Root.ShowData.Users
UserProfile  -> Root.ShowData.UserProfiles
Programmer   -> Root.ShowData.Users.{n}.Programmer
Shell        -> Root.Runtime.Users (logged-in users and their shell clients)
LumaSession  -> Root.Runtime.LumaSessions
Station      -> Root.Runtime.LumaSessions.Stations (this station: Root.StationSettings)
StationSettings -> Root.StationSettings
UserVar      -> Root.ShowData.UserProfiles.{Username}.UserVariables
ShowVar      -> Root.ShowData.Variables
GlobalVar    -> Root.Runtime.GlobalVariables
```

## Naming Rule

To keep the tree aligned with `Lum`:

- use stable singular nouns in the language
- `Fixture`
- `Universe`
- `Output`
- `Executor`
- `Sequence`
- `Effect`
- `User`
- `Station`
- use plural collections in the tree where needed
- `Fixtures`
- `Universes`
- `Sequences`
- `Effects`
- `Users`
- `UserProfiles`
- keep the mapping obvious enough that a user can guess where an object lives

## EffectEngines

```text
Root.ShowData.EffectEngines
└── EffectEngine
    ├── Graph
    ├── Generators
    ├── Inputs
    ├── Outputs
    └── Parameters
```

Notes:

- `EffectEngine` is the language-facing singular noun.
- `EffectEngines` is the show-data collection of authored effect engine
  configurations.
- This is where node-based engines, generators, shader-style logic, and custom
  effect plugins should live as authored truth.
- This branch is distinct from basic playback `Effects`.
- Runtime execution of effect engines should still feed runtime/render engines
  instead of replacing them.

## Engine Alignment

The object tree should follow `Lum` noun usage as closely as possible for
engine-facing branches too.

That means:

- use stable singular nouns in the language
- `RenderEngine`
- `UIEngine`
- `EffectEngine`
- use plural collections in the tree where needed
- `RenderingEngines`
- `UIEngines`
- `EffectEngines`

Important distinctions:

- `RenderEngine`
  - means a live runtime rendering subsystem under `Root.Runtime`
- `UIEngine`
  - means a live presentation subsystem under `Root.UIEngines`
- `EffectEngine`
  - means an authored programmable effect system under `Root.ShowData`

These should not collapse into one generic `Engine` noun.

## Mental Model

- `ShowData`
  - saved show truth
- `Runtime`
  - live execution truth
- `UIEngines`
  - live presentation truth
- `Devices`
  - authored and resolved device/control structure
- `Playback`
  - timeline and show-running objects
- `DataPools`
  - reusable show content, per pool/song/context, with pool-level priority
- `RenderingEngines`
  - live runtime rendering subsystems
- `UIEngines`
  - live presentation subsystems
- `EffectEngines`
  - authored programmable logic and generator systems
