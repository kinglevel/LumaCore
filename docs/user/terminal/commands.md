---
title: Commands
category: docs
---
# How commands work

This is a short guide to typing commands. The syntax of every word — verbs,
objects, aliases, modifiers, rights — is in the generated reference:

- [Command reference](../reference/commands.md)
- [Object reference](../reference/objects.md)

The language itself (sentence shapes, targets, values, ownership) is in
[lum.md](../syntax/lum.md).

## Verb first

A command that changes something starts with its verb:

```text
Store Group 1
Assign Universe 1 At Sacn 1 /Universe 1
On Blind
Off Master 2.1
```

A bare object calls it: `Group 1` selects the group, `Sequence 1` lists its cues,
`Blind` toggles Blind, `View 3` recalls a view. Mode words take their verb in
front — `Blind On` is refused (E-CMD-005).

## Objects and IDs

Objects are named by keyword and ID:

```text
Fixture 1
Fixture 1 Thru 4
Group 1 + Group 2 - Fixture 5
Cue 1.5 Sequence 2
Select Sequence 2
```

`Select` doesn't pick fixtures: it sets your one selected DataPool, Sequence or
Timecode — what pool objects resolve in and what a bare `Go` runs (decisions §29).

- `Thru` makes a range, the same for every object (decisions §2): numbers that
  don't exist are skipped (only none at all is an error), `6 Thru 1` keeps that
  order, `5 Thru` runs to the last and `Thru 5` from the first, a bare `Thru`
  is every number (`Fixture Thru - Group 3`).
- `+` always adds to the selection; `-` takes out.
- `Copy`, `Move` and `Assign` keep the spacing of what they take: a gap stays a
  gap (`Copy Group 1 Thru 3 At 10` without Group 2 → 10 and 12), and nothing is
  copied unless every target is free (or `/Merge` / `/Overwrite`).
- A plain select adds until a value is set; the next plain select replaces.
- Names in quotes work where an object has one: `DataPool "BuskSet"`,
  `Login "LD"`. A one-word name may go without quotes; a name with spaces needs
  them (`Label Group 3 "Wash Front"`). In quotes, a word is always text: `"All"`
  is a group called All, `"/nc"` a name, never the option.
- A `/word` without quotes is an option (`/Overwrite`, `/nc`).
- File names are relative to their library folder and may name a sub-folder:
  `Export Fixture 1 "rig/front.json"` writes `library/export/rig/front.json`;
  `Import FixtureType "AuraXB.gdtf"` reads `library/gdtf/`. A full path or `..` is refused.
- Inside, every object has a GUID. The number is only its slot: `Move` keeps the
  GUID, `Copy` makes a new one.

## At

`At` gives a value or a target:

```text
Fixture 1 At 50
At Full
Master Group 3 At 50 Fade 2
Copy Group 1 At 2
Patch 4x "LED Par" At 1.001
Assign Universe 1 Thru 6 At Sacn 1 /Universe 1
```

## Flags

A `/` word modifies a command:

```text
LoadShow "Festival 26" /NoSave /NoConfirmation
Store Group 4 /Merge
Unclaim Patch /Keep
Clock /Watch
List Fixture /v
```

Commands that ask a question (save first? are you sure?) take the answer as a
flag, so they also work in macros and scripts.

## Several statements

`;` separates statements on one line. They run in order:

```text
Fixture 1 Thru 10 ; Next
ResetAll ; Group 3 ; At 50
```

## Running a line (Please)

Enter runs the line. There is no separate `Please` word.

`Meow` is the cat's execute key: with fixtures selected it toggles their
programmer values on and off (knock in / deactivate); with nothing selected it
says meow. On a keyboard, holding the Meow key (default Shift, `Set UserProfile "MeowKey"`) runs a
key's Meow lines — see [Keys](../keys/keys.md).

## Rights

Every user has one **rights level** (a ladder, decisions §30 "12d"); each
level may do everything of the ones below it:

- **Guest** — look (`List`, `Info`, `Help`, the tree shell), log in, their
  own device (DeskLock, full screen, NormalTheme), calling views and screens,
  their own password, colour and profile look
- **Playback** — run the show and use the programmer: select, set values,
  Go / Go- / Goto, masters, Blind / Highlight / Freeze / Solo, arranging their
  own windows and views; never store
- **Presets** — Playback, plus updating existing presets (`Update`, `Store …
  /Merge`) and saving the same show (`SaveShow`)
- **Program** — build the show: store, copy, move, delete and label show
  content, the live patch (fixture name, note, address), park, claims
- **Setup** — the rig, the station and the LumaSession: `Claim Patch` and the
  full patch, networks, lines and routes, `Set Station` / `Set Interface`,
  Join / Leave / Invite / Dismiss / `Claim LumaSession`, merge and autosave
  settings, `LoadShow`, Shutdown / Restart, `Clear Programmer User`
- **Admin** — everything: users and profiles, `Claim … /Force`, `NewShow`,
  `CleanStart`, `Debug`

Every statement passes one rights gate, whatever it came from (terminal, app,
web, OSC, a key, a macro — a line is refused whole). What a line needs is
declared: the word's level, or the named object's level for the verb
(`Store View` is Playback, `Store Group` Program, `Store Sacn` Setup, `Store
User` Admin) — the [command](../reference/commands.md) and
[object](../reference/objects.md) references list them, `Help <word>` says
"Needs:". A refused statement replies E-RIGHTS-001 with the level it needs
and yours ("'Claim Patch' needs Setup — you are Program."). Logins and
passwords are in [lum.md](../syntax/lum.md) and the `User` object.

Rights come only from the **user** (decisions §30 "12c2"): a profile holds
views, screens, keys and settings, never a level. Only an Admin grants levels
(`Set User "Bob" "Rights" Setup`); the user Admin is always Admin (E-USER-005)
and a show always keeps an Admin (E-USER-004). Users and profiles are
Admin's: `Store User`, `Delete User`, `Store` / `Copy` / `Label` / `Delete` /
`Import UserProfile` (you may still set your own password and colour). An OSC
input line runs as its user and needs one (E-RIGHTS-003): it never runs as
Admin.

## Users and profiles

- **Default** is the template every show has (UserProfile 1, locked): new
  profiles start as a copy of it, it can't be changed, renamed or deleted
  (E-USER-002) and nobody works in it (E-USER-008).
- **New user**: a bare `Store User` asks for the name, rights level, profile,
  an optional password and the colour. `Store User "Anna" /Rights Program
  /Profile New /Colour Teal` gives Anna her own profile (a copy of Default
  named after her, "Anna 2" when taken); `/Profile "Stage"` shares one.
- **New profile**: a bare `Store UserProfile` asks — from Default, a copy of
  another profile, or a file. `Store UserProfile "Stage" [/CopyOf "Bob"]`,
  `Copy UserProfile "Bob" At "Stage 2"`, `Import UserProfile "Stage" /As
  "Stage 2"` (a copy with new GUIDs). A profile is never blank.
- **Rename / password**: `Set User "LD" "Name" "Anna"` (the user keeps
  everything; Admin keeps its name, E-USER-005), `Label UserProfile 3 "Front"`;
  `Set User "LD" "Password" Ask` asks (masked).
- **Delete a user**: `Delete User "Bob"` always asks first and says what goes:
  where Bob is logged in, his programmer values, selection and claims, and —
  when nobody else works in it — a switch to delete his profile too
  (`/WithProfile`). Bob is logged out everywhere (shells, tokens), his claims
  are released; one Oops step brings Bob, his programmer, selection and profile
  back (not the session or claims). Refused: Admin (E-USER-005), yourself
  (E-USER-003), the last Admin (E-USER-004).
- **Delete a profile**: `Delete UserProfile "Stage 2"` asks (screens, views,
  presets that go); a profile someone works in is refused and names them
  (E-USER-006).
- **This device**: `Set Device "ScreenConfig" 2` makes a tablet always show
  config 2; `Set Device "ScreenConfig" Follow` follows the user again.
- **Hold full screen**: holding a pane on a phone or tablet opens it full
  screen; with the profile's `HoldFollows` on (default) the desk maximises it
  too (`Set Self Maximized On|Off`), off keeps it on that device.

## Where a command runs

In a LumaSession a station is either the master or a member. The reference marks
each word's **scope**:

- **Master** — show changes run on the master, as you. A member forwards them
  and gets the result back.
- **Station** — runs on the station you are on (`Join`, `Invite`, `Shutdown`).
- **Client** — runs in your shell only (`Help`, `ls`, `Info`).

## Oops

Almost everything you change can be undone with `Oops` (aliases `hiss`,
`unmeow`) and redone with `Redo`: values, selections, pool objects, the patch,
windows and layouts, views and screens, watches, users, settings, keys and this
station's settings. Each user has one Oops list; Oops always undoes your newest
step (`Oops 3` the newest three). Running playback (Go, masters, faders) and the
Blind, Highlight, Freeze and Solo modes are never undone, and loading a show
clears every list.

- `List Undo` shows the list, newest first and numbered, each step with its
  category (Programmer, Selection, Show, Patch, UI, Users, Settings, Keys,
  Station); `List Undo /Patch` shows one category, `/ThisStation` the steps
  made here.
- A station setting (`Set Station …`, `Set Interface …`, a network's NIC) goes
  back on the station where it was changed.
- `/NoOops` (or `/NU`) after any command runs it without an Oops step.
- Profile settings: `Set UserProfile "CreateOops" No` files no steps at all;
  `Set UserProfile "OopsViews" No` (or `"OopsProgrammer"`, `"OopsSelection"`) leaves out those kinds;
  `Set UserProfile "OopsConfirmation" Always|Main|Never` asks before Oops — Main asks only for
  storing, moving, deleting, patching, importing and layout changes.
  `Oops /NoConfirmation` doesn't ask.

The reference says how Oops treats each word.

## Windows, Self and window presets

`Self` is the window a command line belongs to: typed, the window you have
focused; from a window's bar, its menu (the tab dot) or its settings window,
that window; a key placed in a window bar, its window (a key from the keyboard
uses the focused window). The station turns it into `Window <screen>.<n>`
before the line runs, so the journal, Oops and the System Monitor name the
window. A terminal without a focused window has no `Self` (E-CMD-016) — name
the window instead (`EditSetting Window 1.3`).

- `Set Self Universe + 1`, `- 1`, `* 2` — relative numbers; `Set Self Readout
  Default` puts one setting back; `Set Self AutoScroll Toggle` flips it;
  `Set Self Since Now` clears a System Monitor window (this window only — the
  station keeps its history).
- The window bar is two settings: `WindowBar On|Off` and `WindowBarItems
  "universe,prev,next"` (setting keys and the window type's actions, left to
  right); `Set Self WindowBarItems Default` brings the type's bar back. On
  screen: a pin on each cell of the window's settings puts it in the bar, the
  **Window bar** tab orders the items (drag the handle or ↑ ↓, × removes, +
  adds an action, Load defaults); what doesn't fit goes into the bar's `…`
  menu, and a window too small for its bar lists the items in its window menu
  (tap the tab dot). Every tap in the bar is one of these `Set Self` lines.
- `Reset Self` puts every setting and the bar back (label, colour and position
  stay); `Reset UserProfile "Bob" Window /Type DMX` does it for every DMX window
  of a profile and asks first.
- `Store WindowPreset "Big DMX"` keeps this window's settings, bar, label,
  colour and scroll on your profile; `WindowPreset 2` copies one into `Self`
  (presets are per window type; a bare `WindowPreset` / `Store WindowPreset`
  asks). They travel with `Export UserProfile`.
- `Export View 3`, `Export ScreenConfig 1`, `Export WindowPreset 2` write JSON
  to `library/ui/views|screenconfigs|windowpresets/`; `Import …` reads it back
  (as a copy with new GUIDs when the show already has them).

## Screenshots

`Screenshot` (or **F11** / **Print Screen**) saves a picture of each of
your open screens, exactly as they look, from every web or app window showing
them on this station; each one flashes green. The files go to
`library/screenshots/<station>/` as `YYYYMMDD_hhmmss_<user>_screen<n>.png`, and
the System Monitor and a notice say where. `Screenshot Screen 2` takes one
screen, `Screenshot User "LD"` someone else's.

## Planned words

Some words are published before they are built, so they can be found in Help
and completion. From step 11b they carry the status **Planned**: running one
replies "isn't built yet (planned)" with an error code and changes nothing.
Until then, a few unbuilt objects (`Macro`, `Executor`, `Effect`, `Timecode`,
`Preset`) answer "acknowledged" and do nothing.

## A first session

```text
Import FixtureType "fixture.gdtf"
Claim Patch
Patch 4x "LED Par" At 1.001
Unclaim Patch /Keep
Store Network "Rig"
Set Network "Rig" "Interface" "en1"
Store Sacn 1 "Rig" /Network "Rig"
Assign Universe 1 At Sacn 1 /Universe 1
Fixture 1 Thru 4 At Full
Store Group 1 "Front"
Group 1
At 50
Oops
SaveShow "Rehearsal"
```

- `Claim Patch` (or a bare `Patch`) takes the patch and edits a draft; patching
  with arguments needs that claim. `Unclaim Patch /Keep` puts the draft live.
- A line on a network that isn't mapped to a NIC on this station never sends.
  A line is its protocol's object: `Store Sacn 1`, then
  `Assign Universe 1 At Sacn 1 /Universe 1` (one run of universes per line).
- `Off Master 2.1` is a blackout (the grand master off); `On Master 2.1` brings it
  back.

## More

- [cli-terminal.md](cli-terminal.md) — the terminal: keys, completion, watch
- [tree-shell.md](tree-shell.md) — `ls`, `cd`, `cat`, `grep` and friends
- [Health and clock](../health/health-and-clock.md) — `List Health`, `Info Clock`
- [Keys](../keys/keys.md) — `Bind`, UI keys, gestures

## Background

This page used to hold syntax tables, an example session and an abbreviation
list. They drifted from the code (old `Patch Fixture 1 … At 1.1` form,
object-first `Universe 1 Assign …`, the removed `Black` word, `u` listed as
Universe while it is Update). The generated reference replaces them; aliases
live only there.
