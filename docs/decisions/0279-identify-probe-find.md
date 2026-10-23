---
id: 0279
title: Identify is one object above everything but Park
area: patch
status: built
date: 2026-10-19
built_in: round 3 4d3a
supersedes: []
superseded_by: —
confirmed_by: [`EngineInput::identify`, `identify::IdentifyTier`, `/As`, lum.md "Identify", `identify::tick`]
source: [archive §34 L9914–9916, archive §34 L10465–10515]
---
# Identify is one object above everything but Park

## Context
Edit Patch needs to flash a fixture, probe a raw address and find a fixture whose address is unknown. The user chose one word over three; a test must prove an Identify is never forgotten on.

## Decision
`Identify` is one word, an object owning what it flashes (`Fixture`, `DMX`; `LumShape::ONE`), default verb On: `Identify Fixture 101 Thru 106`; a bare `Identify` = your selection; `Identify DMX 1.150` (Probe: an address or a run, patched or not); `Identify DMX <footprint> /As Fixture n` (Find: a probe in a fixture's shape; Patch here runs one Assign); `Off Identify [target]`. Setup and up; scope Master — it runs on the station that sends; never stored, never an Oops step, never replicated. One per user on `AppState::identify` (a new line replaces yours), published with the engine input; the plan compiles the targets into `identify::IdentifyTier` (a conflict-parked fixture too, Park's slots left out) and the tick writes them over the merged output after DMX input: GDTF defaults, the dimmer a sine, the colour round the hue circle (RGB until the attribute map brings CIE). Above programmers, playback, group and grand masters, Solo and DMX input; a parked channel stays parked. It ends on `Off Identify`, when the dialog that started it closes (Edit Patch runs `Off Identify Fixture Thru; Off Identify DMX Thru`), at logout or a lost holder, on `Unclaim Root.ShowData.Devices`, and after its own timeout (`identify::tick`); a leftover is ended and logged; the patch dialog's strip names who runs one.

## Consequences
Every leave path is tested; never a second flash path or a client-side timer. The CIE colour comes with map-and-clone-attribute-maps; another user's Identify in the strip is open.
