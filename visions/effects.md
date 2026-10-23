---
title: Effects
category: vision
---
# Effects

**Design, not built:** this page is the effect generator as decided on 2026-10-11, written
as a vision. Today the station publishes the words (`Effect`, `Stomp`, `Unstomp`, `IfStomp`,
`Disable`, `Enable`, `Shuffle` are `Planned` in the generated reference,
`docs/user/reference/commands.md` and `objects.md`) and the `Effect` pool in the tree;
nothing runs an effect. The decision records are `docs/decisions/<slug>`:
`effects-model-and-control`, `effects-deterministic-anchors`, `effects-model-and-control` and
`pawtern-pool` (see `docs/plans/active/md-upgrade.records.md` for the slugs). The full text,
with the research verdict and the build order, is §28 of the frozen log
`docs/archive/decisions-2026-10.md`; this page supersedes the copy in `docs/archive/handoffs/effects.md`.

## What an effect is

An `Effect` is a DataPool object, like a group or a sequence. It is a living pattern, not a
recording: a cue *calls* it, the way some older consoles did, and the pattern keeps running while cues
come and go. The effect stores no fixture list of its own; what it runs on comes from the
call — a fixture selection or a group, which it follows live (fixtures added to the group join
at once, the pattern re-spreads, overrides stay).

Every link an effect holds — calls, Lo / Hi presets, forms, groups — is a GUID, never a pool
number. Deleting something an effect uses names what will break and asks first. `Edit` /
`Enter Effect 5` claims the effect; other users' edits wait until it is released.

## Lines

An effect is a list of **lines**, `Effect 5.2`. Everything that makes the pattern is shared by
all fixtures of a line: its attribute(s), `Form`, BPM, width, direction (Fwd / Bwd / Bounce),
NShot, the phase rule, the layout and the Pawtern. Only two things are per fixture, kept as
sparse overrides in the effect: **Lo / Hi** (a number, a `Thru` fan, a physical unit or a
preset reference) and **Disable / Enable**. A disabled fixture keeps its slot — the pattern
never shifts. One master **Rate** per effect scales all its lines' BPM.

The **layout** says how the phase spreads: `Linear` follows the selection or group order with
groups, blocks and wings; `Grid` uses the group's grid by axis (X / Y / Diagonal / Radial),
honouring each member's rotation and mirror. A line is **absolute** or **relative**; relative
offsets are physical, so +10° means the same on every fixture.

Lines come in two kinds. A **wave line** is the normal one: attributes, Lo / Hi and a Form. A
**Prowl line** links a **Prowl** from its own pool: a gradient timeline of stops (presets or
values) with gradient sliders shaping each fade, stops snapping to a beat grid, colour blended
in a per-Prowl colour space. Walking several presets happens only there; steps never exist in
values or wave lines. Both kinds share everything else — BPM, Rate, phase rule, layout,
Pawtern, blend, Disable, anchors — and the same place in the merge.

## Forms

`Form` is a pool of 0–1 waveforms: sine, cosine, ramps, triangle, PWM, step, random, chase,
Phase1/2/3, circle, figure 8, plus user keyframe and formula forms. A multi-dimensional form
drives several attributes at once. Deleting a form in use asks first. `Random` forms are
seeded by effect, line, fixture, cell and cycle number: the same on every run and on every
station of a LumaSession.

## Blend and the effect band

Effects merge as their own **band**, not as value steps: the static merge first, then the
effect band, then the masters and Solo. In order: Park, then Stomp, then the effect band, then
static values (LTP), then defaults; DMX input is merged after the render.

Each line has a `Blend`: `Replace` (absolute; LTP among replacing lines), `Add` (the relative
default), `Subtract`, `Multiply` (Lo / Hi as factors — a dimmer flicker scaling the cue
level), `Max`, `Min`. Several lines on one attribute all apply, in a fixed order that never
depends on when they started: Replace, then Add / Subtract, then Multiply, then Min / Max.
Inside one effect the bottom line wins a Replace conflict; between playbacks the higher
priority wins, then the call whose cue started latest.

## Calls

`Effect 5` turns the effect on: a call in your programmer, on your selection, a running
playback. Clearing the programmer stops it. `Store Cue` stores the call. A cue's call holds the
effect, its fixtures (or the group), its **size** (`Effect 5 At 60`), its Rate, and a subset of
fixtures to leave out. Call parameters track independently; size and Rate fade cue to cue
(`Effect 5 Rate 2` with a 4 s fade speeds the effect up over 4 s, the phase never jumps). No
per-fixture values live in cues.

**Size is a master.** On an absolute line it fades the strength between the static value
underneath and the full effect; on a relative line it scales the offset from the middle out
toward zero. Fades, OffCue and Stomp use the same envelope.

Edits of a running effect — Lo / Hi, a disabled fixture — go **into the effect**, never the
cue: live, undoable, replicated, shown on every cue running it. Under Blind the edit is a
preview; leaving Blind sends it live with Program Time.

Effects also run straight from an executor (`Assign Effect 5 At Executor 1.1`: the fader is
size, Go / Off start and stop, the Go is the anchor) and from the pool. A call can be stored
**paused**: it glides to a stop over the cue's effect timing, tracks paused until a later cue
un-pauses it, and resumes on its original clock — no new anchor.

A line on a **speed master** takes the master's tempo as its BPM and keeps a ratio (1, ½, 2 …),
so lines on one master stay locked; the call's Rate still scales. Tap and Learn change tempo
only, never phase.

## Stomp — the four ways to stop

- `Disable Effect 5.2 Fixture 3` — in the effect, for every cue.
- `Off Effect 5 Fixture 3` — the cue's call: this effect stops for those fixtures, slots kept.
- `Stomp Fixture 3` — a tracked layer: no effect runs on any attribute of the fixture, whatever
  playback it comes from, while the stomp's source is active; the static value underneath is
  output. `Stomp Fixture 3 Attribute Pan` narrows it.
- `Off Effect 5` — the whole call.

`Unstomp Fixture 3` stores a tracked removal, clearing only its own source's stomp. The words
are verb-first only (`Fixture 3 Stomp` is refused). A static value in a later cue never stops
an effect by itself — there is no auto-stomp. Stomps are visible: a marker in the sheet and
the DMX window naming the source cue, `List Stomp`, `IfStomp`.

## Timing

Entering and leaving an effect fades the fixture's effect **amplitude** (absolute: effect to
the static value underneath; relative: offset to zero); the effect clock keeps running. Each
cue part has its own effect timing (`EffectFade`, `EffectDelay`, out-fade and out-delay),
falling back to the cue's fade and delay when unset; dimmer lines leaving use the out-fade.
Fanned delays make fixtures leave in a wave. Snap is fade 0. Sequence off fades every call by
the OffCue timing.

Highlight never stops an effect or motion: it puts the highlight values on the attributes the
highlight personality names and leaves the rest running. Solo shows the effect on the soloed
fixtures. An effect started or edited under Blind is, for the live output, not running. Freeze
lifts only the programmer's priority; motion never stops.

## Deterministic anchors

Effects can be started in any order, at any moment, and give the same output: the effect band
is a pure function of the running calls, the show and the clock.

**The cue is the sync point.** When a cue starts it grabs the clock: that moment is the cue's
**anchor**, zero time for every effect it calls. A line's position is
`(clock − anchor) × BPM × Rate + phase rule`, so an effect added to the cue later (an Update, a
live edit, a new call stored into it) runs in sync with the ones already there when they
share a BPM, and the start order never matters. Rate changes integrate, with no jump. A call
that tracks into later cues keeps its anchor; changing size or Rate never re-anchors;
restarting the cue (Go on it again, Goto) takes a new one. NShot lines count cycles from the
anchor.

The **programmer** is a running playback with one anchor for all its effects. With a sequence
selected and running it uses that sequence's current cue anchor, and follows every Go (snap, or
slide over Program Time), so storing into that cue needs no resync. With no sequence selected
it takes its own anchor when its first effect starts and drops it when cleared. Only the
programmer follows: effects tracking in the sequence keep their own cue's anchor.

**Handover is a phase slide, not a crossfade.** An effect started in the programmer and stored
into a running cue keeps the programmer's anchor until the programmer lets go, then moves to
the cue's: snapped with Program Time off, otherwise sliding the phase the shortest way (at most
half a cycle) over Program Time — the integrated phase shifts, the amplitude never dips. The
same slide applies whenever one line moves between two sources with different anchors.

Anchors are runtime playback state, replicated with the playback; on show load running
sequences resume with a fresh anchor. The clock is session-wide, so every station renders the
same frame. **Every cue is its own clock** — nothing syncs across sequences; what must be in
sync goes in one sequence.

## Cells

A patched fixture is one object everywhere; cells are never selectable and there are no
sub-fixtures. The cell layout is defined per mode on the fixture type (count, order,
positions, per-cell versus shared attributes, master relations), read from the GDTF geometry
tree and editable in the fixture-type menu. Only the effect generator expands cells (a line's
`Resolution: Fixture | Cell`), transforming cell positions by the fixture's grid position and
rotation.

## Syntax, as decided

```text
Effect 5                               on (programmer call on the selection)
Off Effect 5 [Fixture …]               stop all / those fixtures; Effect 5.2 = one line
Effect 5 At 60 · Effect 5 Rate 2       call size · master ratio
Edit Effect 5 · Enter Effect 5         edit without running · edit target [Effect 5.2]
Store Effect 5.2 /Attribute Pan /Form Circle
Set Effect 5.2 Form Sine | BPM 120 | Width 50 | Direction Bounce | NShot 1
Set Effect 5.2 Layout Grid | Axis X | Phase 0 Thru 360 | Pawtern "Wings2"
Set Effect 5.2 Blend Multiply
Effect 5.2 Fixture 1 Thru 8 Lo 20 Hi 50 Thru 100 · Effect 5.2 Group 2 Hi Preset 4.3
Reset Effect 5.2 Fixture 3 · Disable / Enable Effect 5[.2] Fixture 3
Stomp Fixture 3 · Stomp Group 2 · Unstomp Fixture 3
Assign Effect 5.2 At Master 3.1        speed master
```

The edit target is the explicit `Effect 5.2`, else the entered effect, else the only running
line on that fixture and attribute filter, else refused listing the candidates. `Lo` / `Hi`
are effect-line properties, not registry words. Errors get an `E-EFX` catalog. The sequence
sheet shows each cue's calls as stacked squares with their controls (size, Rate, effect
fade / delay, fixture subset, pause); `Update` writes them into that cue's call.

## In the sequence sheet and the pool

Effects in the sheet are drawn MA1-style, one square per call; the pattern (Lo / Hi, Prowl)
lives in the effect, the call (size, Rate, timing, subset, pause) in the cue. Effects started
by a sequence's cues show on that sequence's executor.

## Related ideas

Plugins, custom effect engines and a commands pool are an older note,
[notes/effect-engines.md](notes/effect-engines.md). The Pawtern (the selection-pattern system
that spreads a layout) has its own record, `pawtern-pool`.
