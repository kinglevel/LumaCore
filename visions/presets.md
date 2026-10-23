---
title: Presets
category: docs
---
# Presets

**Design, not built:** this page is the preset system as decided — pools, modes, Extract / Recast, recipe and embedded presets — written before it was built. Today the station has the one `Value` type presets share with the programmer and cues, the preset type words as published (`Planned` in the generated reference, `docs/user/reference/objects.md`), and the `TimingPreset` / `CurvePreset` pools; everything marked Planned below is still not built. The decision records are `docs/decisions/<slug>` — the related ones today are `playback-merge-order`, `sequences-in-datapools`, `attributes-two-layers` and `pawtern-pool`; presets get their own record when their step is picked up (see `docs/plans/active/md-upgrade.records.md` for the slugs). `§N` below names a section of the frozen log `docs/archive/decisions-2026-10.md`.

This document defines the preset system for LumaCore, informed by established console
preset models and aligned with `Lum` sentence shapes and command families
([lum.md](../docs/user/syntax/lum.md)). The current syntax of `Preset` and the preset type
words is in the [object reference](../docs/user/reference/objects.md#preset).

## Status

Markers: **Built** = in the code today · **Planned** = decided, not built ·
**Superseded** = decided differently later.

- **Built:** the one `Value` type that presets, the programmer and cues share
  (normalized or physical, with an optional preset reference — decisions §27);
  the preset type words (`Dimmer`, `Position`, `Gobo`, `Color`, `Beam`, `Focus`,
  `Control`, `Shaper`, `Video`, `All`) are published, so they complete and show in
  Help; attribute definitions (seeded from the GDTF standard attribute table)
  and feature groups in `ShowData`.
- **Planned:** preset pools and everything on this page that stores, calls,
  edits or shows a preset — S/G/U modes, Extract, Recast, embedded and recipe
  presets, filters, indicators. Only `List Preset` is built; `Store`, `Call` and
  `Info Preset` are published as Planned and reply `isn't built yet (planned)`
  [E-CMD-003], changing nothing.
- **Superseded:** multistep (phaser) presets. Decisions §28 makes a value
  static; all dynamics are effects (see the roadmap and
  `docs/archive/handoffs/effects.md`).

## Principle

A preset is a labeled reference that holds attribute values for fixtures. Cues
and the programmer reference presets rather than storing raw values directly.
Updating a preset propagates the change everywhere it is referenced.

## Tree Home

```text
Root.ShowData.DataPools.DataPool(N).PresetGroups
```

The tree has the `PresetGroups` branch; it stays empty until preset pools are
built. Presets are meant to be organized into pools by feature group:

```text
Root.ShowData.DataPools.DataPool(N).Presets.Dimmer
Root.ShowData.DataPools.DataPool(N).Presets.Position
Root.ShowData.DataPools.DataPool(N).Presets.Gobo
Root.ShowData.DataPools.DataPool(N).Presets.Color
Root.ShowData.DataPools.DataPool(N).Presets.Beam
Root.ShowData.DataPools.DataPool(N).Presets.Focus
Root.ShowData.DataPools.DataPool(N).Presets.Control
Root.ShowData.DataPools.DataPool(N).Presets.Shaper
Root.ShowData.DataPools.DataPool(N).Presets.Video
Root.ShowData.DataPools.DataPool(N).Presets.All(1..5)
```

Each feature group pool has a default input filter that only allows values for
that feature group to be stored. The `All` pools have no default filter and can
store any attribute values.

## Preset Modes (Planned)

Three modes determine how data is stored and which fixtures can use it:

### Selective (S)

- Data stored per individual fixture
- Only the specific fixtures present when stored can use the preset
- Most precise but fixture-specific

### Global (G)

- Data stored per fixture type
- All fixtures of the same type can use the preset
- Average value calculated when fixtures of same type have different values
- Selective overrides added for fixtures that diverge from the average

### Universal (U)

- Data stored with a universal fixture flag
- Can be used by any fixture with compatible attributes
- Color values transformed through the color engine across fixture types
- Most portable across shows and rigs

## Lum Sentence Shapes (Planned)

Presets use the standard `Lum` command families.

### Store

```text
Store Preset Color 1
Store Preset Color 1 "Red Wash"
Store Preset Dimmer 1
Store Preset All 1 "Opening Look"
```

Follows the `Store/Update` family. Captures current programmer values into the
specified preset pool.

### Store Options

```text
Store Preset Color 1 /Selective
Store Preset Color 1 /Global
Store Preset Color 1 /Universal
Store Preset Color 1 /Merge
```

Slash modifiers control the store mode and merge behavior.

### Recall / Call

```text
At Preset Color 1
At Preset "Red Wash"
At Color 1
Fixture 1 Thru 10 At Preset Color 1
```

Follows the `Action` family. Calling a preset places a reference in the
programmer — not a copy of the values.

### SelFix / At (Pool Action)

Default pool tap behavior:

1. No selection → select all fixtures that can use the preset
2. Second tap → call the preset into the programmer
3. With existing selection → first tap calls the preset

### Extract (Planned)

```text
Extract Selection
Extract Preset Color.1
Extract Fixture 1 Thru 10
```

Pulls preset values into the programmer as hard values with no reference link.
Useful when the user needs raw values independent of the preset.

### Edit

```text
Edit Preset Color 1
```

Opens the preset for editing. Fixtures are selected and values pulled into the
programmer. After adjusting values:

```text
Update Preset Color 1
```

### Update

```text
Update Preset Color 1
Update Preset Color 1 /Merge
```

Updates the preset with current programmer values. `/Merge` adds new content
without removing existing values.

### Delete

```text
Delete Preset Color 1
Delete Preset Color 1 /NoConfirmation
```

When a preset is deleted, values in cues that referenced it become hardcoded.

### Copy

```text
Copy Preset Color 1 At 2
```

### Inspection

```text
Info Preset Color 1
List Preset Color
List Preset All
```

### Assign Filter (Planned)

```text
Assign Filter "WashOnly" At Preset Color
Assign Filter 1 At Preset Color.1
```

Custom input filters can be applied to pools or individual presets.

## Preset Data Layers (Planned)

### Absolute Values

Standard stored values. No special marker.

### Relative Values

Values added to existing values rather than replacing them.

### Timing Values

Fade and delay times can be stored alongside attribute values. Preset timing
has higher priority than the Programmer Time master.

## Embedded Presets (Planned)

A preset can contain a reference to another preset instead of (or alongside)
raw values:

```text
Store Preset All 1 /Embedded
```

Up to 10 levels of nesting are allowed. This enables building complex presets
from simpler building blocks.

## Recipe Presets (Planned)

A recipe preset contains one or more recipe lines, each specifying:

- Selection (a group or current selection)
- Value (preset reference or hard values)
- Pawtern settings (decisions §3)
- Individual fade/delay values (speed and phase belong to effects, decisions §28)

Recipes can be recalculated from their source components when source presets
change.

Recipe templates can load their recipe into the programmer:

```text
Set Preset Color 1 Property "RecipeTemplate" On
```

## Multistep Presets (Phasers) — Superseded

The first design had presets with values in several steps working as phasers
(speed, phase, acceleration, deceleration, transition and width in linked
phaser layers, as on established consoles). Decisions §28 (2026-10-11) replaced this: a stored
value is static, and every dynamic is an effect. Kept here as background.

## Preset Pool Properties (Planned)

- `Label` — display name
- `PresetMode` — default mode for the pool (S, G, U)
- `InputFilter` — feature group or custom filter
- `Appearance` — visual style
- `PoolAction` — default tap behavior (At, SelFix/At, Call, Toggle, etc.)
- `CuePart` — target programmer part (default: 0)

## Preset Indicators (Planned)

Pool objects should display:

- Mode letter (S, G, U) or combination
- Color bar showing validity for current selection:
  - green = valid for all selected fixtures
  - yellow = valid for some selected fixtures
  - dimmed = not usable by selection
- Feature group indicator bar for presets with multi-group values
- Count of fixtures currently using the preset

## Presets In Cues (Planned)

When a cue is stored with preset references:

- The cue stores a reference to the preset, not the values
- If the preset is later updated, cues automatically use the new values
- If the preset is deleted, values transfer to the cue as hardcoded values
- If preset attributes are added or removed, a recast updates affected cues:

```text
Recast Preset Color 1
```

## Interaction With Programmer (Planned)

- Calling a preset places a reference in the programmer
- The fixture sheet shows the preset pool and number (e.g., `4.1`)
- Preset references and hard values can coexist in the programmer
- Absolute and relative layers are tracked separately

## Design Rules

- Presets are show objects with pool identity and `Lum` noun status
- `Store`, `Update`, `Delete`, `Copy`, `Edit`, `Info`, `List` work on presets
  the same way they work on any other pool object
- Calling a preset creates a reference — extracting creates a copy
- Preset mode (S/G/U) determines portability across fixtures and shows
- Feature group pools enforce attribute filtering by default
- Timing in presets takes priority over programmer time settings
- Preset updates propagate to all referencing cues without manual intervention
