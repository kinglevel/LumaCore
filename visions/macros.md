---
title: Macros
category: docs
---
# Macros

**Design, not built:** this page is the macro system as decided — macro lines, variables, the executor — written before it was built. Today the station has the macro pool in the tree, `List Macro`, several statements on one line and UI keys that run a line per gesture; `Go` / `Store` / `Info Macro` are `Planned` in the generated reference (`docs/user/reference/objects.md`). The decision records are `docs/decisions/<slug>` — the related ones today are `variables-three-tiers` and `keys-uikey-gestures`; macros get their own record when their step is picked up (see `docs/plans/active/md-upgrade.records.md` for the slugs). `§N` below names a section of the frozen log `docs/archive/decisions-2026-10.md`.

This document defines the macro system for LumaCore, informed by established console
macro models and aligned with `Lum` sentence shapes and command families
([lum.md](../docs/user/syntax/lum.md)). The current syntax of `Macro` is in the
[object reference](../docs/user/reference/objects.md#macro).

## Status

Markers below: **Built** = in the code today · **Planned** = decided, not built.

- **Built:** the macro pool in the tree (`DataPool.Macros`, `List Macro`);
  several statements on one line with `;`; `Echo "text"` to mark steps in the
  System Monitor; UI keys, which run a command line per gesture like a small macro
  ([keys.md](../docs/user/keys/keys.md)).
- **Planned:** the macro executor and everything that needs it — storing and
  editing macro lines, running a macro, waits, variables, assignment to
  executors, import. `Macro` and its verbs are published as Planned: they reply
  `isn't built yet (planned)` [E-CMD-003] and change nothing.

## Principle

A macro is a stored sequence of `Lum` commands that can be executed as a single
action. Macros live in the `Macro` pool under `DataPools` and are shared between
all users in the show.

## Tree Home

```text
Root.ShowData.DataPools.DataPool(N).Macros.Macro(N)
```

Each macro contains an ordered list of macro lines. Each line holds one command
and its execution properties.

## Macro Lines (Planned)

A macro line has these properties:

- `Command`
  - the `Lum` command to execute
- `Wait`
  - time before advancing to the next line
  - numeric seconds, `Follow` (immediate), or `Go` (pause until manual Go+)
- `Enabled`
  - whether the line participates in execution
- `Execute`
  - `Yes`: line runs automatically (implicit Please)
  - `No`: command placed on command line for user interaction

## Lum Sentence Shapes

Macros use the standard `Lum` command families. Creation, editing, and execution
follow the verb distinctions in [lum.md](../docs/user/syntax/lum.md).

### Store / Create (Planned)

```text
Store Macro 1
Store Macro 1 "QuickSave"
```

Stores an empty macro in the pool. Follows the `Store/Update` family.

### Edit (Planned)

```text
Edit Macro 1
```

Opens the macro for line editing. Inside the macro context:

```text
Insert                                    # add a new line
Set 1 Property "Command" "SaveShow"       # set line 1 command
Set 1 Property "Wait" "0.1"              # set line 1 wait time
Delete 3                                  # delete line 3
```

### Execute / Trigger (Planned)

```text
Macro 1
Macro "QuickSave"
Go+ Macro 1
```

Bare object reference or `Go+` starts execution. This follows the `Playback`
family — macros behave like runnable show objects.

### Assign To Executor (Planned)

```text
Assign Macro 1 At Executor 1
Assign Macro "QuickSave" At Page 1.5
```

Follows the `Assignment` family. Uses the same `Assign ... At ...` shape as
universe and sequence assignment.

### Inspection

```text
Info Macro 1
List Macro
```

Follows the `Inspection` family. **Built:** `List Macro` lists the pool's tree
node. **Planned:** `Info Macro` with the macro's lines.

### Delete (Planned)

```text
Delete Macro 1
Delete Macro 1 /NoConfirmation
```

### Copy (Planned)

```text
Copy Macro 1 At 2
```

### Update (Planned)

```text
Update Macro 1
```

## Variables In Macros (Planned)

Macros can use `Lum` variables for dynamic content. Variable references use `$`
prefix syntax.

```text
Set UserVar MyFixture "1"
Set GlobalVar DebugMode "On"
```

Referencing in macro commands:

```text
Fixture $MyFixture At Full
```

Variable scopes align with the `Lum` variable tiers (decisions §5; only
`GlobalVar` exists today, and only `List GlobalVar` reads it):

- `UserVar` — visible only to the creating user profile
- `ShowVar` — visible to all users, saved with show
- `GlobalVar` — runtime-only, visible to all users in session

Variable operations available in macros:

```text
Set UserVar Name "Value"
Set GlobalVar Counter 1
Info UserVar Name
Delete GlobalVar Counter
```

Variables support integer, float, and text types.

## Slash Modifiers (Planned for macros)

Macros support the standard `Lum` slash modifier pattern:

```text
Delete Macro 1 /NoConfirmation
Store Macro 1 /Merge
```

## Macro Pool Properties (Planned)

- `Label` — display name
- `Appearance` — visual style in pool view
- `Note` — documentation text
- `Lock` — prevent accidental edits
- `Scribble` — icon or visual marker

These are standard pool object properties shared with other pool types.

## Macro Calling Macros (Planned)

A macro can trigger other macros:

```text
# Line inside Macro 1
Macro 2
```

When a macro calls another macro directly, the called macro stops when the
calling macro stops. Use `Call Macro` for independent execution.

## Import / Export (Planned)

`Import` reads GDTF files today. Macros should be importable from the library:

```text
Import Macro "library/macros/QuickSave.lua"
```

A factory library of predefined macros should ship with LumaCore for common
workflows (save show, change user, selection helpers).

## Example Macros (Planned)

These show the intended shape; none runs yet.

### QuickSave

```text
Macro 1 "QuickSave"
  Line 1: SaveShow              (Execute: Yes, Wait: Follow)
```

### RemOff (Remove Unselected)

```text
Macro 2 "RemOff"
  Line 1: Delete Group 999 /NoConfirmation    (Wait: 0.1)
  Line 2: Store Group 999 /Merge              (Wait: 0.1)
  Line 3: Fixture Thru - Group 999            (Wait: 0.1)
  Line 4: At 0                                (Wait: 0.1)
  Line 5: Group 999                           (Wait: 0.1)
  Line 6: Delete Group 999 /NoConfirmation    (Wait: 0.1)
```

Line 3 already works typed: `Fixture Thru - Group 999` selects every fixture
but the group's (a bare `Thru` is every number, the one range rule). A `Delete`
in a stored line (a macro or key line) never asks, with or without
`/NoConfirmation`.

### World From Selection

Needs Worlds, which aren't built (no `World` object yet).

```text
Macro 3 "WorldIsSel"
  Line 1: Delete World 999 /NoConfirmation    (Wait: Follow)
  Line 2: Store World 999                     (Wait: Follow)
  Line 3: World 999                           (Wait: Follow)
```

## Design Rules

- Macro commands are standard `Lum` sentences — no special macro dialect
- Wait timing controls pacing between lines, not a separate scheduler
- Macros are show objects with pool identity, not ephemeral scripts
- Variable references should resolve at execution time, not at store time
- `Lum` verbs (`Store`, `Edit`, `Delete`, `Copy`, `Assign`, `Info`, `List`)
  work on macros the same way they work on any other pool object
