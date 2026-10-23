---
title: Fixtures Patch And Output
category: docs
---
# Fixtures, patch and output

How fixtures get into the show, where they are addressed, and how their DMX
leaves the station. Syntax for every word is in the reference:
[`Store`](../reference/commands.md#store),
[`Assign`](../reference/commands.md#assign),
[`Fixture`](../reference/objects.md#fixture),
[`FixtureType`](../reference/objects.md#fixturetype),
[`Universe`](../reference/objects.md#universe),
[`Network`](../reference/objects.md#network),
[`Sacn`](../reference/objects.md#sacn),
[`ArtNet`](../reference/objects.md#artnet),
[`Osc`](../reference/objects.md#osc),
[`Mvr`](../reference/objects.md#mvr). History: §11, §34 and §40 of
[the frozen decision log](../../archive/decisions-2026-10.md); the rules are in
[docs/decisions/](../../decisions/).

## The four parts

These stay separate — it keeps the system easier to grow and to trust:

1. **fixture definition** — what a fixture type is (GDTF)
2. **patch** — which fixtures exist and where they are addressed
3. **render** — the render engine turns the show into DMX, 60 times a second
4. **transport** — output lines send it on a network

## Fixture definition

Fixture types come from open standards first.

- GDTF import: `Import FixtureType "fixture.gdtf"` (a file of `library/gdtf/`, or
  on a drive with `/Drive`), or inline when patching:
  `Store Fixture 101 Thru 106 At DMX 1.001 /FixtureType "fixture.gdtf"`.
  Types live at `Root.ShowData.Devices.Dmx.Patch.FixtureTypes`.
- **GDTF native:** the original `.gdtf` file is the source of truth and travels
  **in the show file** (`Devices/Dmx/Patch/FixtureTypes/<n>/Revisions/<r>/source.gdtf`
  inside the `.luma` zip, see [show-files.md](../show-files/show-files.md)). The
  fixture type you see is derived from the file again on every load: the
  geometry tree, every channel function with its DMX and physical range and
  channel sets, the highlight values, snap, wheels, emitters and filters. Import
  is warn-only: what can be read is imported, spec problems are listed
  (`Info FixtureType`). A type keeps LumaCore's GUID for links and stores the
  file's own `FixtureTypeID`; importing the same `FixtureTypeID` again adds a
  revision. A fixture type is not edited by hand.
- The station keeps every `.gdtf` it has seen in `library/fixturetypes/` (a
  cache, not committed).
- **MVR import:** `Import Mvr "file"` (`/Drive`, `/Scope <node>`, the match and
  offset options in the reference) reads an MVR against your Devices draft and
  shows a **review** first — nothing is written. `List Mvr` / `Info Mvr` and the
  Patch dialog's review grid show each row's action; `Import Mvr <rows> /Apply`
  writes the ticked rows into the draft, `/Cancel` drops the review. A fixture
  inside a fixture in the file (MVR's child list, a console's folders) nests under
  it in the file's order. The review's defaults are your profile's "MVR import"
  properties (`Set UserProfile "MvrMatch" …`).
- Still planned: a fixture-type editor that writes a new `.gdtf` revision, MVR
  export — see [the roadmap](../../meta/roadmap.md).

## Patch

The patch answers: which fixtures exist, which mode they use, which universe and
address they sit on. It stays conflict-aware, inspectable and explicit.

```text
Root.ShowData.Devices.Dmx.Patch
├── FixtureTypes
├── Fixtures
└── Universes
```

- `Store Fixture 101 Thru 106 At DMX 1.001 /FixtureType "x" [/Mode "Ext"]
  [/Revision 2] [/Every 20]` patches fixtures; the universe is created if it
  doesn't exist (a refused line leaves none behind). The default is the first
  mode of the newest revision; a mode with 0 channels patches and takes no
  address. Two fixture types with the same name are refused (E-PATCH-010) until
  one is renamed (`Label FixtureType 2 "…"`).
- `Store Universe 1`, `Delete Universe 1 Thru 4`, `Delete Fixture 5`
  (`/KeepChildren` lifts what is under a folder fixture one level up).
- `Label`, `Move` (a number renumbers; `At` a node nests — links stay, they are
  GUIDs), `Set Fixture 3 "Name" …`.
- These patch edits need the **Devices claim** (`Claim Root.ShowData.Devices`,
  or `enter patch`, which claims and moves you there).

### A fixture's ids

A fixture carries MVR's three ids: its **FID** — the number you type,
`Fixture 849`; a fixture may have none, then it is a folder —, an **IDType**
(MVR's list: Fixture, Channel, Generic, Houselights, NonDim, Media, Fog, Effect,
Pyro, Marker) and a **CID**, a number within that type: `Fixture Media 7003`.
FIDs are unique in the show, CIDs within their type. The properties are
`"FID"`, `"IDType"` and `"CID"`; the drawing's text ids stay as `"FixtureID"`,
`"UnitNumber"` and `"Function"`. A fixture may hold fixtures (`List Fixture
Media 7003` lists a folder's) and never has sub-fixtures; every fixture keeps
its order under its parent.

A fixture's place is one world matrix: `Set Fixture 101 "PosZ" 6.5`,
`"RotY" -90` (metres and degrees, `+ 0.5` relative), read from the matrix and
written into it.

### The staged patch (draft)

Claiming Devices edits a **draft** of every device kind, as a console's patch menu does.

- The rig keeps running on the **live** patch while you edit. Output, the
  engine and the web UI all see the live patch.
- Your draft edits are not undone or replicated one by one.
- Another user's device edits are refused while your draft is open
  (E-PATCH-008), except the **live columns** (a live patch: a fixture's
  Name, Note, Address, PosX…RotZ, MasterReact; Program level). Those reach the
  rig at once, as their own Oops step, and your draft follows them — unless you
  changed that cell in the draft: then theirs waits, naming you.
- Leaving asks **Keep / Discard / Cancel** (`Unclaim Root.ShowData.Devices`, or
  answer up front with `/Keep` or `/Discard`). Keep files the whole draft as one
  change: one Oops step, one replication to the LumaSession. A holder who is
  gone for a few seconds loses the draft.
- The draft holds the fixtures **and the universes** (new, delete, move, label,
  MergeMode). The lines stay on their slots and stay live for anyone: moving a
  universe whose slot has input lines asks first (MergeMode goes with the
  universe, the inputs stay); deleting one names the fixtures on it — they lose
  their address there and stay in the patch — and the lines left on its slot;
  Keep names the lines left on slots without a universe.
- In a LumaSession, a patch changed on both sides blocks a merge on rejoin
  (E-LUMA-005).

### The Patch dialog

`Menu "Patch"` (the Menu's Patch tile) opens Devices › DMX › Patch with the
Patch, Fixture types and Universes grids. It opens on the **Live** patch: hold
Live to claim the draft, press Claimed to leave it; the ✕ asks while you hold
it. The sidebar is the MVR tree (layers › MVR groups › folder fixtures, Direct
or Deep), MVR class or tag; picking a node filters the grid, and the grid shows
fixtures only, in tree order, with an MVR node column. Sorting is the one lift:
selected rows dropped on a sidebar node run `Assign Fixture <rows> At <node>`
(`MvrLayer n`, `MvrGroup n` or a folder fixture); a lifted node runs `Move
<node> At <node>` through the nesting ask; the context menu's Move into… picks
a node. Deleting a node asks Delete all / Keep them (`/KeepChildren`) / Cancel.
The Import cell opens the MVR import (the file, then the preset as settings).

### Live Patch

Some changes are live and need no claim:

- re-patching one fixture: `Set Fixture 3 "Address" 2.101`, and the other live
  columns above
- networks and protocol lines (below)

Still planned on the patch: Duplicate, the Metadata tab, a desk capture — one
line each in [the roadmap](../../meta/roadmap.md).

## Universes, networks and protocol lines

Universes are show objects: a logical address space of 512 channels. How that
space leaves the station is three more things:

```text
Universe 1–6 ──carried by──▶ Sacn 1 "Rig" ──on──▶ Network "Rig" ──mapped to──▶ en1 (this station)
```

- A **Network** is named in the show: `Store Network "Rig"`.
- Each station maps the network to one of its NICs:
  `Set Network "Rig" "Interface" "en1"` (or an IP, or a subnet). The mapping is
  this station's row of the station table (`ShowData.Stations`), applied by
  that station only; its last-applied copy is in `library/station_settings.json`.
- Each **protocol** of the build is an object, and a numbered one is a **line**:
  one row of the DMX protocols table. `Sacn 1`, `ArtNet 1` (`Protocols.DMXProtocols`)
  and `Osc 1` (`Protocols.DataProtocols`), numbered per protocol:
  `Store Sacn 1 "Rig" /Network "Rig"`.
- A DMX line carries **one run** of local universes, `LocalUniverse` and the
  `Amount` after it, starting on the wire at its protocol's universe
  (`SacnUniverse`, Art-Net `PortAddress` as `5` or `0.0.5`):
  `Assign Universe 1 Thru 6 At Sacn 1 /Universe 1`. Without `/Universe` the line
  keeps its first protocol universe. Another run asks: Replace / New line /
  Cancel. More runs are more lines.
- **Assign's order is the data flow:** universes onto a line send (`Mode Output`),
  a line onto universes receives (`Mode Input`).
- `Unassign Universe 6 At Sacn 1` takes universes off either end of the run;
  `Unassign Sacn 1` leaves the line with none.
- The line names its universes **by number**: moving or deleting a universe never
  moves the line.

Rules:

- A line on a network that isn't mapped on this station **never sends**. There
  is no "auto" fallback.
- Only a standalone station or the LumaSession **master** sends. Members hold.
- Networks and lines are live: no Devices claim.
- Lines may cover the same universes (multicast plus a unicast visualiser, two
  networks, sACN and Art-Net). Two output lines of one protocol may not send the
  same protocol universe to the same destination (E-OUTPUT-003); a gap in a run
  is refused (E-OUTPUT-007).
- A line can't change protocol: store a new line and delete the old.
- A show naming a protocol this build lacks, or a setting its protocol doesn't
  declare, is refused at load (E-SHOW-007 / 008).
- `List Network` shows the mapping and this station's interfaces; `List Sacn`
  shows each sACN line's universes and live state; `Info Sacn` the protocol
  itself; `List Health` the real send rate.

### Line settings

Every DMX line has: `Mode`, `LocalUniverse`, `Amount`, `Target` (unicast IP;
none = sACN multicast, Art-Net ArtPoll + broadcast; on an input, the one source
accepted), `Rate` (how often it sends), `Send Full|Changes` (Changes: a change
at once plus a few repeats, then a keep-alive), `PacketGap` (µs between
universes, for old nodes; all lines on one network card together never spend
more than half a frame in gaps — `Info Sacn 1` shows what is used), `Sync`
(ArtSync / E1.31 sync), `InputPriority` (an input's band) and `Enabled`. What
only one protocol has is its own setting: sACN's `SacnUniverse` and `Priority`
(an output's), Art-Net's `PortAddress`. An Art-Net node's reported refresh rate
caps its line. The reference lists each setting with its range and default.

### Transports

sACN (E1.31, multicast or unicast) and Art-Net (ArtPoll + broadcast or
unicast) are built: one driver per protocol and NIC sends the output lines and
receives the input lines. Still planned: USB DMX — see
[the roadmap](../../meta/roadmap.md).

### DMX input

A line in `Mode Input` receives sACN or Art-Net and merges it into local
universes.

- `Store Sacn 2 "Guest" /Network "Guest" /Mode Input`, or
  `Assign Sacn 2 At Universe 1 Thru 6 /Universe 1`.
- Each universe's `MergeMode` decides how: `Off`, `Prio` (the line's
  `InputPriority` band: Super … Lowest), `HTP`, `LowTP`. The merge is per DMX
  channel, in the render tick. MergeMode belongs to the universe
  (`Set Universe 1 Thru 6 "MergeMode" HTP`).
- Sources merge by E1.31 priority then HTP; a silent source is dropped after a
  short loss timeout; the station's own packets are ignored.
- Park always wins. Masters and Solo never touch input.
- `DMX In 1` reads the raw input of universe 1.

An `Osc` line (`Store Osc 1 /Network "Rig" /Port 8000 /User "LD"`) runs received
commands as that user, with that user's rights. Like DMX input, it listens only
on the station that sends (standalone, or the LumaSession master), so a message
sent to every station runs once.

## Seeing the output

- `DMX 1` — what the engine outputs on universe 1; `DMX 1.1` — one channel.
- The **DMX** window shows universe values, which source won, and input.
- Still planned: `DMX 1.1 At 255` as a live DMX tester (the word is `Planned`
  in the reference) — see [the roadmap](../../meta/roadmap.md).

## DataPool relationship

Datapool objects organize fixtures for operators; they never replace the
definition or the patch. `Group` — fixtures in order with grid spots — is
built; `Layout` and `Macro` are designs for later
([visions/README.md](../../../visions/README.md)). They live under
`Root.ShowData.DataPools.DataPool …`.

Rule: the patch says what exists and where it is addressed; datapool objects say
how operators organize and reuse it.

## Background

Early drafts called outputs "transport assignments that publish universe data"
and kept them as one list (`Protocols.Outputs`). Step 4 split that into
networks (named in the show, mapped per station), lines (one type for both
directions) and routes (on universes), so a show moves between rigs by mapping
its networks to NICs, without editing the lines. §34 "34f7c" made each protocol
an object and each line one protocols row carrying a run of universes, so the patch
draft never moves the wiring; §40 gave a fixture MVR's three ids and let a
fixture hold fixtures.
