---
title: All Objects
category: docs
---
# All Objects

Canonical language source: `docs/user/syntax/lum.md`

This document is the object index for LumaCore.

It treats [syntax/Lum.md](../syntax/Lum.md)
and [ObjectTree.md](./ObjectTree.md) as
the main anchors, and it refers outward to the other definition docs instead of
redefining everything here.

## Principle

An object is a language-facing noun that should map to a real place in the
tree, a real runtime concept, or a real authored show concept.

Primary references:

- [syntax/Objects.md](../syntax/Objects.md)
- [syntax/Lum.md](../syntax/Lum.md)
- [ObjectTree.md](./ObjectTree.md)

## Root-Level Objects

- `Root`
  - see [ObjectTree.md](./ObjectTree.md)
- `ShowData`
  - see [ObjectTree.md](./ObjectTree.md)
- `Runtime`
  - see [Runtime.md](../../dev/engines/Runtime.md)
- `UIEngines`
  - see [ObjectTree.md](./ObjectTree.md)
- `Resources`
  - see [ObjectTree.md](./ObjectTree.md)
- `Diagnostics`
  - see [ObjectTree.md](./ObjectTree.md)
- `StationSettings`
  - see [ObjectTree.md](./ObjectTree.md)
- `Library`
  - see [ObjectTree.md](./ObjectTree.md)

## Show Objects

- `ShowSettings`
  - see [ObjectTree.md](./ObjectTree.md)
- `Devices`
  - see [ObjectTree.md](./ObjectTree.md)
- `Patch`
  - see [FixturesPatchOutput.md](../fixtures/FixturesPatchOutput.md)
- `LivePatch`
  - see [FixturesPatchOutput.md](../fixtures/FixturesPatchOutput.md)
- `Protocols`
  - see [ObjectTree.md](./ObjectTree.md)
- `Playback`
  - see [Control.md](./Control.md)
- `DataPools`
  - see [ObjectTree.md](./ObjectTree.md)
- `UserProfiles`
  - see [ObjectTree.md](./ObjectTree.md)
- `EffectEngines`
  - see [ObjectTree.md](./ObjectTree.md)
- `Variables`
  - see [ObjectTree.md](./ObjectTree.md)
- `3DSpatial`
  - see [ObjectTree.md](./ObjectTree.md)

## Fixture And Patch Objects

- `FixtureType`
  - see [FixturesPatchOutput.md](../fixtures/FixturesPatchOutput.md)
- `Fixture`
  - see [Objects.md](../syntax/Objects.md)
- `Universe`
  - see [FixturesPatchOutput.md](../fixtures/FixturesPatchOutput.md)
- `Output`
  - see [FixturesPatchOutput.md](../fixtures/FixturesPatchOutput.md)

## Playback Objects

- `Sequence`
  - see [Control.md](./Control.md)
- `Cue`
  - see [Control.md](./Control.md)
- `Executor`
  - see [Control.md](./Control.md)
- `Master`
  - see [Control.md](./Control.md)
- `Effect`
  - see [Control.md](./Control.md)
- `TimingPreset`
  - see [ObjectTree.md](./ObjectTree.md)
- `CurvePreset`
  - see [ObjectTree.md](./ObjectTree.md)

## DataPool Objects

- `DataPool`
  - see [ObjectTree.md](./ObjectTree.md)
- `Group`
  - see [Control.md](./Control.md)
- `Layout`
  - see [Control.md](./Control.md)
- `Macro`
  - see [Control.md](./Control.md)
- `World`
  - see [Control.md](./Control.md)
- `Filter`
  - see [Control.md](./Control.md)
- `Timecode`
  - see [Timelines.md](../playback/Timelines.md)
- `DynamicState`
  - see [ObjectTree.md](./ObjectTree.md)

## Time Objects

- `Track`
  - see [Timelines.md](../playback/Timelines.md)
- `Event`
  - see [Timelines.md](../playback/Timelines.md)
- `NoteEvent`
  - see [Timelines.md](../playback/Timelines.md)
- `ControlChangeEvent`
  - see [Timelines.md](../playback/Timelines.md)
- `PitchBendEvent`
  - see [Timelines.md](../playback/Timelines.md)
- `ProgramChangeEvent`
  - see [Timelines.md](../playback/Timelines.md)
- `MetaEvent`
  - see [Timelines.md](../playback/Timelines.md)
- `Marker`
  - see [Timelines.md](../playback/Timelines.md)
- `Clip`
  - see [Timelines.md](../playback/Timelines.md)
- `Region`
  - see [Timelines.md](../playback/Timelines.md)
- `MarkerTrack`
  - see [Timelines.md](../playback/Timelines.md)
- `TempoTrack`
  - see [Timelines.md](../playback/Timelines.md)
- `SignatureTrack`
  - see [Timelines.md](../playback/Timelines.md)
- `LoopRegion`
  - see [Timelines.md](../playback/Timelines.md)
- `Duration`
  - see [Timelines.md](../playback/Timelines.md)
- `TimeBase`
  - see [Timelines.md](../playback/Timelines.md)

## User And Variable Objects

- `User`
  - see [Objects.md](../syntax/Objects.md)
- `UserProfile`
  - see [ObjectTree.md](./ObjectTree.md)
- `Programmer`
  - see [Control.md](./Control.md)
- `UserVar`
  - see [Lum.md](../syntax/Lum.md)
- `ShowVar`
  - see [Lum.md](../syntax/Lum.md)
- `GlobalVar`
  - see [Lum.md](../syntax/Lum.md)

## Runtime And Session Objects

- `Server`
  - see [Runtime.md](../../dev/engines/Runtime.md)
- `Session`
  - see [NetworkSessions.md](../../dev/networking/NetworkSessions.md)
- `Client`
  - see [NetworkSessions.md](../../dev/networking/NetworkSessions.md)
- `Station`
  - see [NetworkSessions.md](../../dev/networking/NetworkSessions.md)
- `Network`
  - see [NetworkEngine.md](../../dev/networking/NetworkEngine.md)
- `RunningPlayback`
  - see [ObjectTree.md](./ObjectTree.md)

## Engine Objects

- `Runtime Owner Engine`
  - see [Engines.md](../../dev/engines/Engines.md)
- `Session Engine`
  - see [SessionEngine.md](../../dev/networking/SessionEngine.md)
- `Network Engine`
  - see [NetworkEngine.md](../../dev/networking/NetworkEngine.md)
- `Render Engine`
  - see [Engines.md](../../dev/engines/Engines.md)
- `UIEngine`
  - see [ObjectTree.md](./ObjectTree.md)
- `EffectEngine`
  - see [ObjectTree.md](./ObjectTree.md)
- `Timecode Engine`
  - see [Timelines.md](../playback/Timelines.md)

## Syntax References

For the language roles behind these objects, see:

- [Lum.md](../syntax/Lum.md)
- [Objects.md](../syntax/Objects.md)
- [Functions.md](../syntax/Functions.md)
- [Targets.md](../syntax/Targets.md)
- [Values.md](../syntax/Values.md)

## Rule

If a noun matters in `Lum`, it should eventually appear here and point back to
its stronger definition source.
