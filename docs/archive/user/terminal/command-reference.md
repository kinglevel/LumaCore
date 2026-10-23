---
title: Command Reference
category: docs
---
# LumaCore Command & Function Reference

**Created:** 2026-03-10
**Status:** Living document — updated as phases progress
**Source:** reference console 2.3 keyword reference, LumaCore REQUIREMENTS.md, CONTEXT.md decisions

This reference maps every command verb and keyword LumaCore will support, organized by function. Each entry shows the reference console origin (if any), the LumaCore phase where it's implemented, and any LumaCore-specific divergences.

## Legend

| Column | Meaning |
|--------|---------|
| **Keyword** | The command as typed |
| **Abbrev** | Shortest accepted console-style abbreviation |
| **reference console** | Present in reference console? |
| **Phase** | LumaCore implementation phase |
| **Req** | Requirement ID(s) |

---

## 1. Navigation

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Enter` | | No | 2 | — | Claim and enter an object for editing (`Edit <obj> <n>` claims the object; `Exit` / `..` releases it) |
| `cd` | | No | 2 | — | Navigate tree without claiming |
| `pwd` | | No | 2 | — | Print current tree position |

---

## 2. Selection & Addressing

Keywords for selecting fixtures and navigating the object tree.

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Fixture` | `Fix` | Yes | 1 | TERM-01 | Select fixture(s) by number or name |
| `Group` | `Gro` | Yes | 2 | CTRL-02 | Recall group (restores selection) |
| `Thru` | `T` | Yes | 2 | TERM-03 | Range operator: `1 Thru 10` |
| `+` | | Yes | 2 | TERM-03 | Add to selection |
| `-` | | Yes | 2 | TERM-03 | Remove from selection |
| `If` | | Yes | 2 | TERM-05 | Universal scope resolver / conditional filter (see below) |
| `Where` | | Yes | 2 | TERM-05 | Alias for `If` (reference console muscle memory) |
| `IfActive` | `IfA` | Yes | 2 | — | Filter by programmer-active state (not value) |
| `IfOutput` | `IfO` | Yes | v2 | — | Filter by output state |
| `IfProgrammer` | `IfP` | Yes | v2 | — | Filter by programmer state |
| `Select` | `Sel` | Yes | 2 | — | Explicit select (usually implicit) |
| `Selection` | | Yes | 2 | TERM-02 | Reference current selection object |
| `SelectFixtures` | `SelFix` | Yes | v2 | — | Select fixtures matching criteria |
| `All` | | No | 2 | — | Select all fixtures (LumaCore addition) |
| `Next` | | Yes | 2 | — | Select next fixture in selection order |
| `Previous` | `Prev` | Yes | 2 | — | Select previous fixture |
| `Root` | | Yes | 2 | TERM-06 | Enter browse mode / reference tree root |

### `If` Grammar (Phase 2 — needs testing)

`If` is LumaCore's canonical scope resolver. `Where` is an alias.

```
# Standalone — select from object
If Seq 1                              # Select all fixtures in Sequence 1
ResetAll; If Seq 1                    # Reset programmer, then select from Seq 1

# Suffix — filter current selection by value
At 50 If Dimmer > 0                   # Set 50% only on fixtures with dimmer > 0

# Chained — intersection
Fixt 1 If Grou "FOH" If Gro "Spots"  # Fixture 1 only if in BOTH groups

# Metadata
Fixt 1 Thru 100 If Type "Spot"       # Filter by fixture type
```

`IfActive` targets attributes **active in programmer** (not value check):
```
IfActive Dimmer At 50                 # Only touch fixtures where dimmer is programmed
```

---

## 3. Control — Setting Values

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `At` | | Yes | 2 | CMD-01/02/03 | Set value: dimmer %, attribute, or preset |
| `Full` | | Yes | 2 | CMD-04 | Shortcut: dimmer 100% |
| `Zero` | | Yes | 2 | CMD-04 | Shortcut: dimmer 0% |
| `On` | | Yes | 2 | CMD-05 | Knock in: activate the selection's (or named fixtures') programmer values; `On Blind` switches a mode on |
| `Off` | | Yes | 2 | CMD-05 | Knock out: remove those fixtures' values from the programmer (bare `Off` also deselects); `Off Blind` switches a mode off |
| `Highlight` | `Hi` | Yes | 2 | CMD-06 | Mode object (per user): `On Highlight` / `Off Highlight` / bare `Highlight` toggles |
| `Lowlight` | `Low` | Yes | v2 | — | Inverse of highlight |
| `Solo` | | Yes | 2 | CMD-07 | Mode object (per user): only the user's selection keeps its dimmer |
| ~~`Black`~~ | | Yes | — | CMD-08 | Removed: blackout is the grand master (`Master 1`) switched off |
| `Park` | | Yes | 2 | CMD-09 | Lock fixture/channel value |
| `Unpark` | | Yes | 2 | CMD-09 | Release parked fixture/channel |
| `Release` | `Rel` | Yes | v2 | — | Release values back to default |
| `Freeze` | | Yes | 2 | — | Mode object (per user): programmer above playbacks, `On Freeze` / `Off Freeze` |
| `Stomp` | | Yes | v2 | — | Override tracking values |
| `Flip` | | Yes | v2 | — | Flip pan/tilt to alternate position |
| `Align` | | Yes | v2 | — | Distribute values across selection |
| `Absolute` | `Abs` | Yes | v2 | — | Set absolute value mode |
| `Relative` | `Rel` | Yes | v2 | — | Set relative value mode |

---

## 4. Programmer & Reset

> LumaCore separates **Clear** (terminal screen) from **Reset** (programmer state).
> `Reset` follows a 4-stage industry-standard progression:
> 1st clears selection, 2nd deactivates values, 3rd empties programmer, 4th clears terminal buffer.

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Reset` | `Res` | Yes | 2 | CMD-10 | Progressive programmer reset (4-stage: selection → values → programmer → terminal) |
| `ResetAll` | `ResA` | Yes | 2 | CMD-10 | Reset programmer completely in one step |
| `ResetActive` | `ResAct` | Yes | v2 | — | Deactivate all active values (visible but won't store) |
| `ResetSelection` | `ResSel` | Yes | v2 | — | Clear fixture selection only, values untouched |
| `Programmer` | `Prog` | Yes | 2 | PROG-01 | Reference programmer object (one per user, saved with the show) |
| `Blind` | `Bli` | Yes | 2 | — | Mode object (per user): edit without output, `On Blind` / `Off Blind` / bare `Blind` toggles |
| `Preview` | `Pre` | Yes | v2 | — | Preview changes without committing |

---

## 5. Store & Edit — Object Management

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Store` | `Sto` | Yes | 2 | CMD-11/12 | Save selection/values to group/preset |
| `Delete` | `Del` | Yes | 1/2 | CMD-13, GDTF-05 | Remove object from tree |
| `Copy` | `Co` | Yes | 2 | CMD-14 | Duplicate object |
| `Move` | `Mv` | Yes | 2 | — | Renumber object (Fixture/Group/Universe). GUIDs stay stable. |
| `Update` | `Upd` | Yes | 2 | CMD-15 | Update existing object with current values |
| `Edit` | `Ed` | Yes | 2 | CMD-16 | Enter edit mode for an object |
| `Label` | `Lab` | Yes | 2 | — | Set/change object name |
| `Insert` | `Ins` | Yes | v2 | — | Insert object at position |
| `Remove` | `Rem` | Yes | v2 | — | Remove (softer than Delete) |
| `Cut` | | Yes | v2 | — | Cut object (for paste) |
| `Paste` | | Yes | v2 | — | Paste cut/copied object |
| `Swap` | | Yes | v2 | — | Swap two objects |
| `Clone` | | Yes | v2 | — | Clone with references |
| `Exchange` | `Exc` | Yes | v2 | — | Exchange fixture types |
| `AutoCreate` | `AutoC` | Yes | v2 | — | Auto-create objects from context |
| `Claim` | | No | 2 | — | Claim object for editing (`Claim Fixture 3` locks a fixture; selecting never claims; `Claim Patch` starts a patch draft). On another user's object it asks them; no answer in 10 s hands it over (6d) |
| `Claim … /Keep` | | No | 6d | — | Owner's answer to a claim request: keep the object |
| `Claim … /Force` | | No | 6d | — | Admin: take the object at once |
| `Unclaim` | | No | 2 | — | Release claimed object (`Unclaim Patch /Keep` or `/Discard`); also the owner's "hand over" answer |
| `List Claims` | | No | 6d | — | Who holds what (`Root.Runtime.Claims`; alias `claimed`); replaces `ListClaimed` |

---

## 6. Inspection & Debug

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Info` | `Inf` | No | 1 | CMD-17 | Human-readable object inspection |
| `Dump` | `Du` | No | 2 | CMD-18 | Raw debug data dump |
| `List` | `Li` | Yes | 1/2 | CMD-19, GDTF-04 | Universal list command |
| `ListReference` | `ListRef` | Yes | v2 | — | List references to an object |
| `List Reference` | | No | 6d | — | The reference check: links that point at nothing, containment loops (`/Broken` = only the problems) |
| `ListOwnership` | `ListOwn` | Yes | v2 | — | List object ownership |
| `Export` | `Exp` | Yes | 1 | — | Export object to JSON file |
| `Help` | `?` | Yes | 2 | TERM-09 | Built-in help system |

---

## 7. Undo

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Oops` | | Yes | 2 | CMD-20/21 | Undo the user's last action (stackable: `Oops 3` undoes 3 steps; aliases: `hiss`, `unmeow`) |
| `Redo` | | No | 2 | CMD-21 | Redo the user's last undone action |

Each user has their own history (`Runtime.Users.{n}.Undo`); it is never saved in the show.
Everything is undoable except running playback state (Oops never undoes a Go).

---

## 8. GDTF & Patching

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Import` | `Imp` | Yes | 1 | GDTF-01 | Import .gdtf file |
| `Patch` | `Pa` | Yes | 1 | GDTF-02/03 | Patch fixture to universe+address |
| `Unpatch` | `Unp` | No | 1 | GDTF-05 | Remove fixture patch (alias: `rm`) |
| `FixtureType` | `FixT` | Yes | 1 | — | Reference fixture type in tree |
| `Multipatch` | `Multi` | Yes | v2 | — | Multi-instance fixture patching |
| `DMXAddress` | `DMXA` | Yes | 2 | — | Reference/set DMX address |

---

## 9. Universe & Output Management

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Universe` | `Uni` | Yes | 1 | UNIV-01 | Create/reference universe |
| `Assign` | `Ass` | Yes | 1 | UNIV-02/03 | Route a universe to a line: `Assign Universe 1 Thru 4 At Output 1 Universe 1` |
| ~~`Split`~~ / ~~`Merge`~~ | | No | — | UNIV-04/05 | Removed with universe ranges (step 4b) |
| `Create` | `Cre` | No | 1 | — | Create universe (alias for Store) |
| `DMXUniverse` | `DMXU` | Yes | v2 | — | Reference DMX universe layer |
| `Unassign` | | No | 1 | — | Remove a route: `Unassign Universe 1 At Output 1` |
| `Network` | | No | 4b | — | A data network the show names (`Store Network "Rig"`); each station maps it to a NIC |
| `Output` / `Input` | | Yes | 4b/4c | — | Protocol lines (sACN, Art-Net, OSC) on a network |

---

## 10. Sequence & Playback (Phase 3 / v2)

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Sequence` | `Seq` | Yes | v2 | CUE-01 | Reference sequence object |
| `Cue` | | Yes | v2 | CUE-01 | Reference cue within sequence |
| `Go+` | | Yes | v2 | — | Go forward in sequence |
| `Go-` | | Yes | v2 | — | Go backward in sequence |
| `Goto` | | Yes | v2 | — | Jump to specific cue |
| `Pause` | | Yes | v2 | — | Pause running cue; `Pause LumaSession` stops the merge prompt's countdown (6d) |
| `Record` | `Rec` | Yes | v2 | — | Record cue |
| `Top` | | Yes | v2 | — | Go to top of sequence |
| `Kill` | | Yes | v2 | — | Kill running playback |
| `Toggle` | `Tog` | Yes | v2 | — | Toggle playback on/off |
| `Flash` | `Fla` | Yes | v2 | — | Flash playback |
| `Temp` | | Yes | v2 | — | Temporary activation |
| `Block` | | Yes | v2 | — | Block tracking |
| `Unblock` | | Yes | v2 | — | Remove tracking blocks |

---

## 11. Macro System (Phase 3)

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Macro` | `Mac` | Yes | 3 | MACRO-01/02 | Record/replay command sequences |
| `Lua` | | Yes | 3 | DBG-04 | Execute Lua script |
| `LuaFile` | `LuaF` | Yes | 3 | DBG-04 | Execute Lua file |

---

## 12. Watch & Monitoring (Phase 3)

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Watch` | `Wa` | No | 2 | WATCH-01/02/03/04 | Live-updating monitor (implemented) |

Watch targets:
- `Watch DMX 1` — 512-channel universe view
- `Watch Fixture 1` — attribute values for fixture
- `Watch Selection` — auto-update as selection changes
- `Watch Output` — sACN packet rate, timing stats

---

## 13. Preset Pools (Phase 3)

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Preset` | `Pre` | Yes | 3 | POOL-01/04/05 | Universal preset pool |
| `Color` | `Col` | Yes | 3 | POOL-02 | Color preset pool |
| `Dimmer` | `Dim` | Yes | 3 | POOL-01 | Dimmer preset pool |
| `Position` | `Pos` | Yes | 3 | POOL-03 | Position preset pool |
| `Gel` | | Yes | v2 | CLR-01 | Named gel library |

---

## 14. Show & System

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `SaveShow` | `Save` | Yes | 3 | SHOW-01 | Save current show |
| `LoadShow` | `Load` | Yes | 3 | SHOW-02 | Load show file |
| `NewShow` | `New` | Yes | 3 | — | Create new empty show |
| `Info Show` | | No | 6d | — | The loaded show: name, show id, format, status, references, merge default and timeout (`Info Show "name"` reads a file) |
| `Show /MergeDefault` | | No | 6d | — | Admin: the merge prompt's default answer, `Merge\|UseMaster\|UseMine\|Cancel` |
| `Show /MergeTimeout` | | No | 6d | — | Admin: seconds until the default runs (`0` = no prompt, `Unlimited` = waits; default 60) |
| `User` | `Us` | Yes | 2 | — | Reference user object |
| `UserProfile` | `UsP` | Yes | 2 | — | Reference/switch user profile |
| `LogIn` | | Yes | v2 | — | Log in as user |
| `LogOut` | | Yes | v2 | — | Log out current user |
| `Clear` | `Clr` | Yes | 2 | CMD-10 | Clear terminal screen (`clear`/`cls`/`clearterm`) |
| `Exit` | `q` | No | 1 | — | Release claim and go up one context level (aliases: `quit`, `q`) |
| `Shutdown` | | No | 1 | — | Quit LumaCore immediately (alias: `turnoff`) |
| `LumaCore-Shutoff` | | No | 2 | — | Shut down with confirmation prompt |
| `Logout` | | No | 2 | — | Disconnect shell session, return to client (alias: `logoff`) |

---

## 15. Effects & Pawterns (v2)

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Effect` | `Eff` | Yes | v2 | CUE-02 | Effects engine |
| `Pawterns` | `MAt` | Yes | v2 | — | Matrix/fan distribution tool |
| `Shuffle` | | Yes | v2 | — | Randomize selection order |
| `Speed` | `Sp` | Yes | v2 | — | Set effect speed |
| `Phase` | `Pha` | Yes | v2 | — | Set effect phase offset |
| `Width` | | Yes | v2 | — | Set effect width |
| `BPM` | | Yes | v2 | — | Set beats per minute |

---

## 16. Timing Keywords

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Fade` | `Fa` | Yes | v2 | — | Set fade time |
| `Delay` | `De` | Yes | v2 | — | Set delay time |
| `CueFade` | | Yes | v2 | — | Cue-level fade |
| `CueDelay` | | Yes | v2 | — | Cue-level delay |
| `SnapDelay` | | Yes | v2 | — | Snap with delay |
| `Transition` | `Trans` | Yes | v2 | — | Transition type |

---

## 17. Network & LumaSession

| Keyword | Abbrev | reference console | Phase | Req | Description |
|---------|--------|-----|-------|-----|-------------|
| `Join LumaSession` | — | Yes | 2 | — | Join a LumaSession by name or address |
| `Join LumaSession … /Merge` | — | Yes | 6d | — | Rejoin and merge this show into the master's (3-way, by GUID; E-LUMA-005 if not possible) |
| `Leave LumaSession` | — | Yes | 2 | — | Leave the LumaSession (DMX output held) |
| `Claim LumaSession` | — | — | 6c | — | Take the master role (new epoch); Use Mine in the rejoin prompt |
| `Pause LumaSession` | — | — | 6d | — | Stop the merge prompt's countdown (the operator decides) |
| `Invite Station` | — | Yes | 2 | — | Invite a station into this LumaSession |
| `Dismiss Station` | — | Yes | 2 | — | Send a member out of this LumaSession |
| `LumaSession` | — | — | 4d | — | The server-to-server mesh: `List LumaSession`, `Info LumaSession` (no aliases) |
| `Shell` | — | — | 4d | — | Logged-in users and their shell clients: `List Shell` |
| `Interface` | `nic` | Yes | 4d | — | This station's NICs: `Set Interface en1 LumaSession\|Shell\|HTTP Yes\|No` |
| `Station` | — | Yes | 4d | — | Stations: `Set Station Name\|LumaSession\|Location\|Priority\|Invite\|Online\|HoldOutput\|OutputWhenAlone …` |
| `StationSettings` | `StaS` | Yes | 1 | — | Machine-local settings |

---

## 18. Special Operators

| Operator | reference console | Description |
|----------|-----|-------------|
| `;` | Yes | Command separator (chain commands) |
| `.` | Yes | Dot notation for tree paths: `Root.ShowData.Fixtures.1` |
| `..` | Yes | Parent reference in tree |
| `/` | Yes | Path separator (alternative to dot) |
| `=` | Yes | Assignment |
| `"name"` | Yes | String literal / object name |
| `Nx` | No | Batch multiplier: `Patch 4x "Mac Aura" At 1.1` (LumaCore addition) |

---

## 19. LumaCore-Only Commands

Commands that don't exist in reference console — LumaCore originals.

| Keyword | Abbrev | Phase | Req | Description |
|---------|--------|-------|-----|-------------|
| `Watch` | `Wa` | 2 | WATCH-* | Live monitor (reference console uses View/Window system instead) |
| `Info` | `Inf` | 1 | CMD-17 | Human-readable inspection |
| `Dump` | `Du` | 2 | CMD-18 | Raw debug dump |
| `Create` | `Cre` | 1 | — | Explicit object creation |
| `Unpatch` | `Unp` | 1 | GDTF-05 | Remove fixture patch |
| `Echo` | | 2 | — | Print text to terminal |
| `Debug` | | 2 | — | Toggle debug mode (admin-only) |
| `Banner` | | 2 | — | Show ASCII art banner |
| `Clock` | | 2 | — | Show current time |
| `PrecisionClock` | | 2 | — | Precision timing diagnostics |
| `Shortcuts` | | 2 | — | Cycle shortcut mode (KBD/PRG/NAV) |
| `Meow` | | 2 | — | Execute/confirm (knock-in). Also the name of the modifier key |
| `DMX` | | 2 | — | Direct DMX channel control |
| `LumaCore-Shutoff` | | 2 | — | Shutdown with confirmation |

---

## 20. Linux/Unix Aliases

For terminal-native users. These are aliases, not primary keywords.

| Alias | Maps To | Phase |
|-------|---------|-------|
| `ls` | `List` | 1 |
| `rm` | `Delete` / `Unpatch` | 1 |
| `cd` | Navigate tree | 2 |
| `pwd` | Print current path | 2 |

---

## 21. Context Switching

Bare object keywords change the sticky prompt context (console style):

```
[Admin@LumaCore] Fixture> 5           # = Fixture 5
[Admin@LumaCore] Fixture> Universe     # switches context
[Admin@LumaCore] Universe> 1           # = Universe 1
```

**Sticky objects:** Fixture, Universe, Output, Group, Preset, Sequence, Macro, Effect

---

## reference console Keywords NOT Planned for LumaCore

Keywords from reference console 2.3 that are hardware-specific, redundant, or out of scope.

| Keyword | Reason |
|---------|--------|
| `Camera` | No 3D visualizer in v1 |
| `ChangeMulticastBase` | LumaCore uses per-universe config |
| `Chat` / `ChatJoin` / `ChatLeave` | Out of scope |
| `CancelSoftwareUpdate` / `SoftwareUpdate` / `SoftwareImport` | Different update model |
| `Console` | LumaCore is software-only |
| `CopyCrashLog` / `DumpLog` / `ListCrashLog` | Different logging model |
| `Display` / `ScreenConfiguration` / `ScreenContent` | Hardware-specific |
| `Drive` / `Eject` | Hardware-specific |
| `Executor` / `SpecialExecutor` / `Page` | Hardware fader model |
| `FaderCrossFade*` / `FaderMaster` / `FaderRate` / `FaderSpeed` / `FaderTime` / `FaderTemp` | Hardware fader model |
| `Houselights` | Hardware-specific |
| `Key` / `Keyboard` / `KeyboardShortcut` | Handled differently |
| `NDI` | Not in scope |
| `Pyro` / `Fog` | Specialty — future plugin |
| `Reboot` / `Restart` | OS-level, not console commands |
| `RDM` | Future scope |
| `RemoteHID` | Hardware-specific |
| software-switch commands | vendor-specific |
| `Video` / `Media` / `Image` / `Bitmap` | Media server — out of scope |
| `Quickey` | Hardware-specific |

---

## Coverage Summary

| Category | v1 (Phase 1-4) | v2 | Total |
|----------|----------------|----|-------|
| Navigation | 3 | 0 | 3 |
| Selection & Addressing | 12 | 3 | 15 |
| Control | 12 | 6 | 18 |
| Programmer & Reset | 4 | 3 | 7 |
| Store & Edit | 10 | 7 | 17 |
| Inspection & Debug | 5 | 2 | 7 |
| Undo | 2 | 0 | 2 |
| GDTF & Patching | 5 | 2 | 7 |
| Universe & Output | 7 | 1 | 8 |
| Sequence & Playback | 0 | 14 | 14 |
| Macros | 3 | 0 | 3 |
| Watch | 1 | 0 | 1 |
| Presets | 4 | 1 | 5 |
| Show & System | 9 | 2 | 11 |
| Effects | 0 | 7 | 7 |
| Timing | 0 | 6 | 6 |
| Network | 4 | 2 | 6 |
| LumaCore-Only | 17 | 0 | 17 |
| **Total** | **97** | **56** | **153** |

---

*Reference built: 2026-03-10*
*Source: reference console 2.3 manual, LumaCore REQUIREMENTS.md, CONTEXT.md*
*reference console keyword count (full): ~352. LumaCore planned: 126 (~36%). Remainder is hardware-specific or out of scope.*
