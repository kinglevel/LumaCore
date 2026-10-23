---
title: Objects
category: docs
---
# Objects

Canonical language source: `docs/user/syntax/lum.md`

This document captures the current intended high-level definition of `Object`
in `Lum`.

## Principle

In `Lum`, an `Object` is the thing a sentence is about.

An object may be:

- selected
- inspected
- stored
- updated
- assigned
- deleted
- used as context for later actions

Examples:

```text
Fixture 1
Group 5
Universe 1
Sequence 2
Timecode 1
User "Admin"
Station 1
```

## ObjectTree Alignment

An object noun should point to a real branch or node family in the tree.

Examples:

```text
Fixture     -> Root.ShowData.Devices.Dmx.Patch.Fixtures
Group       -> Root.ShowData.DataPools.DataPool(...).Groups
Sequence    -> Root.ShowData.Playback.Sequences
User        -> Root.ShowData.Users
UserProfile -> Root.ShowData.UserProfiles
Programmer  -> Root.ShowData.Users.{n}.Programmer
Shell       -> Root.Runtime.Users (logged-in users and their shell clients)
LumaSession -> Root.Runtime.LumaSessions
Station     -> Root.Runtime.LumaSessions.Stations
Network     -> Root.ShowData.Protocols.Networks (a data network)
Claims      -> Root.Runtime.Claims
```

## Identity (6d)

Every object in the show has a GUID. The number is only its slot: a copy gets a new GUID,
a move keeps it, and `Store` over an existing group keeps the group's GUID. Every stored
link between objects is a GUID (executor → sequence, line → network, fixture → mode, …).
Merges, claims and undo match objects by GUID, never by slot. Command text (macro lines,
key gestures) stays text.

## LumaSession Objects

For server-to-server language (decisions §11 4d, §18), the runtime objects are:

- `LumaSession`
  - the mesh of stations joined together (`Join LumaSession "Festival"`,
    `Leave LumaSession`, `List LumaSession`, `Info LumaSession`); no aliases
- `Station`
  - a running server that joins a LumaSession (`Invite Station "FOH-2"`,
    `Dismiss Station "FOH-2"`, `Set Station …`)
- `Client`
  - one attached device or window (CLI, app window, browser tab)
- `Shell`
  - a logged-in user's terminal session (`List Shell`)
- `Interface`
  - a NIC of this station and which services it serves

`Station` properties include name, LumaSession, location, priority, invite, online, hold
output, output when alone, and status (`Standalone`, `Master`, `Member`, `IdleMaster` when a station lost its master and
is alone).

"Session" alone is not an object; "Network" means only a data network.

## Ownership Objects

For ownership language, the important objects are:

- `User`
  - the owner identity; owns its programmer and selection
- `Programmer`
  - the user's permanent live workspace, saved with the show and shared by every station
    and client the user is logged in on
- claimable show/runtime objects: any existing object with a GUID (`Fixture`, `Sequence`,
  `Group`, …) and the areas `Patch`, `ShowData`, `Devices`, `DataPools`, `Protocols`,
  `UserProfiles`, `ShowSettings`, `StationSettings`
- `Claims`
  - who holds what (`List Claims`, `Root.Runtime.Claims`), keyed by GUID
- `Reference`
  - the reference check: links that point at nothing (`List Reference [/Broken]`)

Rules:

- a `User` may own multiple claimed objects at once
- `Programmer` is always owned by its user
- claimed objects remain inspectable by other users
- selecting does not claim; `Claim Fixture <n>` locks a fixture
- claiming `Patch` edits a draft; `Unclaim Patch` asks Keep / Discard / Cancel
- `Edit <obj> <n>` claims the object until you leave the editor; `cd` never claims
- changing commands on an object another user holds (or anything inside it) are refused
  (E-CLAIM-001)
- `Claim <obj>` on someone's object asks them (`Unclaim <obj>` hands over,
  `Claim <obj> /Keep` keeps it; no answer in 10 s hands it over); Admin `Claim <obj> /Force`
  takes it
