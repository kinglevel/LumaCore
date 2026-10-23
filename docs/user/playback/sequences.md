# Sequences, cues and selections

How playback works today (decisions §27, §29). Every word and its syntax is in
the generated reference: [`Sequence`](../reference/objects.md#sequence),
[`Cue`](../reference/objects.md#cue), [`Select`](../reference/commands.md#select),
[`Master`](../reference/objects.md#master).

## Your selected sequence

Each user has one **selected sequence**. A `Go`, `Go-`,
`Goto Cue 3`, `Store Cue 4` or `Off Sequence` that names no sequence acts on it.

```text
Store Sequence 2 Cue 1     the first cue makes Sequence 2
Select Sequence 2          now a bare Go runs Sequence 2
Go
Store Cue 2                stored into Sequence 2
Deselect Sequence
Go                         → Error: No sequence selected … [E-PLAY-003]
```

LumaCore never guesses Sequence 1. `cd` into a sequence doesn't select it;
deleting it clears the selection. You also have one selected **DataPool**
(where `Group 3` and `Preset 4.1` resolve; without one, the first) and one
selected **Timecode**. `Select` takes only these three kinds — fixtures are
selected with `Fixture <n>` or a group.

## Cue timing

A cue part is like a cue stacked inside the cue: every part runs its own timing
from the same Go. Per attribute:

- a dimmer going **up** and every non-dimmer attribute use **Fade** and **Delay**;
- a dimmer going **down** uses **OutFade** and **OutDelay**;
- attributes the fixture's GDTF marks as **snap** (wheel slots, gobo index)
  jump at the end of their delay instead of fading.

```text
Set Cue 2 "Fade" 3           3 s in-fade for cue 2 of your selected sequence
Set Cue 2 "OutFade" 1.5
Set Sequence 4 Cue 2 "Delay" 0.5
```

Fades run on the session clock (every station in a LumaSession fades the
same). A `Go` during a fade starts each attribute from where it is.

Only a `Go`, `Go-` or `Goto` starts a cue fade. A value a cue **releases**
fades by that cue's time too (a dimmer going down by its OutFade); handing a
channel over to another source (Clear, Unpark, Off Freeze, Blind, Off
Sequence) snaps. Storing into the running cue moves its fade to the new
values (R1-27, Q12).

## CueZero, OffCue and echoes

Every sequence has two fixed cues (decisions §32): **CueZero**, the start state every run tracks
from, and the **OffCue**, the timing used when it goes off. Store into CueZero like any cue; the
OffCue holds only timing:

```text
Store Sequence 3 Cue 0                          what a run from the top starts from
Set Sequence 3 "CueZeroMode" OnlyUsedDimmers fill CueZero with the used dimmers' defaults
Set Sequence 3 Cue "OffCue" "OutFade" 2             echoes fade out over 2 s
```

`CueZeroMode` follows a common console convention: `Off` (the default), `AllUsedAttributes`, `OnlyUsedDimmers`.

**Echoes** are extra runs of a sequence beside its live playback — hold a pool cell (or type the
lines) to fire the first cue again and again:

```text
Top On Sequence 3        an echo: CueZero → the first cue with its timing
Top Off Sequence 3       that echo plays its OffCue and is gone
Off Sequence 3           the live run and every echo go off
```

Echoes merge **intensity HTP** with the live run and each other (overlapping ramp-downs stack)
and every other attribute **LTP** (the newest run wins). `Set Sequence 3 "MaxEchoes" 4` caps them
(a full list refuses the newest Top On; 0 = no echoes), `EchoMax 30` ends an echo after 30 s, and
an echo whose holder goes away (a client gone, a device locked) ends by itself. Echoes are never
saved and never undone.

## Highlight and Solo

`Highlight` brings your selected fixtures to their **GDTF highlight values**
(`DMXChannel@Highlight` in the fixture's file). A channel without one stays as
it is — there is no built-in "dimmer to full". Highlight sits above programmers
and cues and below park; the grand and group masters still scale intensity.

`Solo` takes the fixtures you haven't selected down.

Both have a **master per user** (`Master "Highlight"` / `Master "Solo"`), and `Master 2.2` / `Master 2.4`
cap them for everybody (the grand group):

```text
Master "Highlight" At 50              half way to the highlight values
Master "Solo" At 30                   the others go to 70 %
Set Master "Highlight" "Mode" AlwaysOn      on whenever the level is above 0
Set Master "Solo" "Mode" Disabled           the key switches Solo at full
```

Mode `Toggle` (the default): the key switches the effect, the level is its
strength.
