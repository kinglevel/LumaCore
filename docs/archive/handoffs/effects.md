# Handoff — effect generator design (2026-10-11)

Read this first when continuing the effects work. Source of truth: `docs/meta/decisions.md`
**§28 Effects** (copied in full below, as of 2026-10-11). If the two disagree, decisions.md
wins — update this file when §28 changes.

## Status

> **Audit 2026-10-11 (step 11a):** committed in `58a0f06`; branch `main`. The crate is
> `lumacore-base` since step 8 (`crates/foundation/lumacore-base/src/value.rs`, `lumacore_base::Value`)
> — read every `lumacore-core`/`lumacore_core` below as that. The "group has only
> `fixture_ids`" bug is wrong: step 7 built `GroupMember` with `GridCell` (x, y, z, rotation,
> mirror) — the "group grid with rotation" order item is done. The GDTF cell bugs are fixed
> in step 11b (§29), before effects. The node-engine note moved to
> `visions/notes/effect-engines.md`. Open items: [docs/meta/roadmap.md](../docs/meta/roadmap.md) → Next.

- **Design only.** Every question is answered; no code is written. §27's `Value` text is
  marked superseded by §28. Nothing is committed (decisions.md also carries other
  uncommitted step-4d doc edits).
- **Wait for "go"** before implementing. Once (2026-10-11) a code change was started on a
  design answer ("yes remove that") and the user stopped it: design talk = docs only.
- **First step on go:** remove the phaser fields from `lumacore_core::Value`
  (`crates/lumacore-core/src/value.rs`; only that file uses `Step`/`steps`, plus the
  re-exports in `lumacore-core/src/lib.rs` and `lumacore-tree/src/lib.rs`). No compat
  shim for the old step form — forward only, no migrations (user rule 2026-10-11). Then
  follow the build order at the end.
- **Known bugs found during research (independent of effects):**
  - GDTF importer drops geometries: pixel fixtures import as one cell with a wrong DMX
    footprint (`crates/formats/lumacore-gdtf/src/importer.rs:98-103`, footprint `:173-178`); the
    external `gdtf` 0.2.0 crate already parses `ReferenceGeometry`, `Break`, `Matrix`,
    `Relation`, `DmxChannel.geometry/dmx_break`.
  - `crates/formats/lumacore-gdtf/src/resolver.rs:50-57` keys virtual channels by attribute name
    only — every cell's Dimmer collides.
  - §22 promises grid positions on groups, but `Group` (`crates/show/lumacore-tree/src/datapool.rs:313`)
    has only `fixture_ids`.
  - `crates/show/lumacore-effects` is a stub (node-graph `EffectEngineConfig`) — unrelated to the
    §28 effect pool; the node-engine idea stays separate (`docs/user/effects/effect-engines.md`).

## How the user works on this

- Harden by **asking** (AskUserQuestion, recommended option first), several rounds; record
  each answer in §28 right away.
- Cat names welcome: **Pawtern** (spread), **Meow** (modifier), **Prowl** (preset gradient).
  The user picked `Lo`/`Hi`, `Stomp`/`Unstomp`, `Disable`/`Enable`.
- Reference: a live reference console (read only) and its manual.

## Research summary (sources)

- **An older generation**: an effect pool with lines; **Base ± Size** (no high/low); cues
  store a **call** (Go/Off, intensity, speed, softness, fade) — editing the effect changes
  every cue. Liked: fast and direct. Disliked: outside Blind/priorities/tracking.
- **A middle generation**: effect **layers** per fixture tracked separately from values; cues
  **copy** layer values. Pain: stopping effects later, phase drift, deleting a form silently
  breaks effects.
- **A current generation**: "everything is a phaser"; sines built from steps; no clean stomp,
  phasers break on patch changes, speed can't fade cue to cue. Leaving a phaser has no own
  timing — normal cue layers; sequence off = **OffCue** timing.
- **Others**: ChamSys FX (per-head editing, spread/grid modes, FX update every cue), Hog
  (table/size/rate/offset, fanning), Eos (step/absolute/relative, effect on any channel
  list), Avo shape generator, Onyx.
- **GDTF 1.2** (`mvrdevelopment/spec` gdtf-spec.md): cells = `GeometryReference` instances
  of a top-level template with `Break DMXOffset`; channels on the template are instanced per
  reference; `Position` 4×4 matrices (translation in column 3, +X right, +Z up);
  `DMXMode@Geometry` is the mode's root; `Relation` Multiply = virtual/master dimmer. Real
  files also use plain nested geometries and nested references.

## Key user statements (their words, condensed)

- "we hate phasers, the older ones were ok" — effects running on cues like the older
  generation, but adjustable per fixture.
- Subfixtures as other consoles do them are bad; the effect generator maps to sub-pixels. Cells are defined in the
  fixture type; a fixture is one object everywhere; cells are not selectable in the grid.
- A selection grid is OK and belongs in groups, but must show each fixture's rotation.
- Editing Lo/Hi of a running effect pushes into the effect, not the cue ("we dont want data
  scattered everywhere"); speed shared, values not linked.
- Each line is linear or grid; a single fixture can be disabled in the effect grid.
- Stomp sits above the effect, is stored and tracks; going back needs `Unstomp`.
  `Stomp Fixture 3` stomps all attributes of the fixture.
- `Effect 5` turns the effect on, as a running playback in the programmer;
  clearing stops it. One BPM per line, one master ratio per effect; Lo/Hi; OffCue; dimmer
  out-fade; each cue has effect fade/delay.
- Phaser fields: "remove". Two relative effects add; blend modes (add, multiply, subtract …).
  A cue scales Rate with a fade.
- Start effects in any order and get the same result. The cue start is ground zero for sync;
  effects added later to the cue stay in sync. Store of an out-of-sync programmer effect:
  snap, or fade with Program Time. With a sequence selected the programmer follows its
  current cue; tracked effects keep their own cue clock.
- Size is a master: absolute fades strength, relative scales from the middle out (toward 0).
  Inside an effect the bottom line wins. Speed master = BPM; tap = tempo only.
- Links are GUIDs, never numbers; deleting a linked object warns and asks. Editing claims.
- "lets ditch steps in effects, hi and low is king" → **Prowl**: a dedicated pool object, a
  gradient timeline of presets/values, its own editor, not reachable from fixtures, played
  by a dedicated Prowl line that still respects Stomp.
- Effects show in the sequence sheet as recipe-like lines, as squares; select one to
  adjust and `Update` that cue. Every cue its own clock.
- Highlight never stops motion (only the fixture type's highlight values); Solo shows the
  effect; Blind = as if not running; Freeze only lifts programmer priority.

## §28 Effects (copy of docs/meta/decisions.md)


Researched against three generations of an established console, its forums, a live
console, and the GDTF 1.2 spec. Verdict: the oldest got the structure right (a cue *calls* a
living effect: base ± size, forms, fast), the middle one got per-fixture layers right, the
newest phasers ("everything is a phaser", sines built from steps, no clean stomp, breaks on
patch changes) are what we avoid.

- **Model.** `Effect` is a DataPool object; lines are `Effect 5.2`. A line has: attribute(s),
  `Form` (pool of 0–1 waveforms: sine, cosine, ramps, triangle, PWM, step, random, chase,
  Phase1/2/3, circle, figure 8; user keyframe/formula forms; multi-dimensional forms drive
  several attributes; deleting a form in use asks first), **BPM per line**, width, direction
  (Fwd/Bwd/Bounce), NShot, phase rule, **layout** (`Linear` = selection/group order with
  groups/blocks/wings, or `Grid` = the group's grid by axis X/Y/Diagonal/Radial, honouring
  each member's rotation/mirror), a Pawtern, and mode absolute/relative (relative offsets are
  physical, so +10° means the same on every fixture). The effect stores no fixture list.
- **Shared vs per fixture.** Everything that makes the pattern is shared by all fixtures of a
  line: form, BPM, width, direction/bounce/NShot, phase rule, layout. Only **Lo/Hi** (number,
  `Thru` fan, physical unit or preset reference) and **Disable/Enable** are per fixture, kept
  as **sparse overrides** in the effect (fixtures without one follow the line default). One
  master **Rate** per effect scales all lines' BPM.
- **Edits go into the effect, never the cue.** Editing Lo/Hi or disabling a fixture of a
  running effect writes the Effect object live (undoable, replicated) and shows on every cue
  running it. Under Blind the edit is a preview; leaving Blind sends it live with Program
  Time. Disabled fixtures keep their slot (the pattern never shifts).
- **Effect band.** Effects merge as their own band, not as `Value` steps: static merge first,
  then the effect band, then masters/Solo. Merge order: Park > **Stomp** > effect band >
  static values (LTP) > defaults; DMX input after render.
- **No phasers in values.** The phaser fields leave `lumacore_core::Value` (supersedes §27's
  "steps" shape): `steps`, width, transition, accel, decel, speed, phase, measure go; a value
  is static (absolute / relative / preset / release). No migration for the step form (forward only).
- **Blend modes.** Each line has a `Blend`: `Replace` (absolute; LTP among replacing lines),
  `Add` (relative default), `Subtract`, `Multiply` (Lo/Hi as factors, e.g. a dimmer flicker
  scaling the cue level), `Max`, `Min`. Several lines on one attribute all apply: two
  relative effects add.
- **Deterministic.** Effects can be started in any order, at any moment, and give the same
  output: the effect band is a pure function of (running calls, show content, clock).
  - Blend modes apply in a **fixed order**, never by start order: Replace → Add/Subtract →
    Multiply → Min/Max. Within a group the ops commute (sums, products, min/max).
  - **The cue is the sync point.** When a cue starts it grabs the clock: that moment is the
    cue's **anchor**, zero time for every effect it calls. A line's position is
    `(clock − anchor) × BPM × Rate + phase rule`, so effects added to the cue later (Update,
    live edits, a new call stored into it) run in sync with the ones already there when they
    share BPM, and the order effects were started in never matters. Rate changes integrate
    (no jump).
  - A call that **tracks** into later cues keeps its anchor (no jump at the next Go);
    changing size/Rate never re-anchors. Restarting the call's cue (Go on it again, Goto)
    takes a new anchor. NShot lines count cycles from the anchor.
  - The **programmer** is a running playback with one anchor for all its effects (so effects
    turned on one by one stay in sync):
    - **Sequence selected and running:** the programmer uses its **current cue's anchor**.
      Effects started now take it at once; on every Go of that sequence the running
      programmer effects move to the new cue's anchor (snap, or slide over Program Time).
      Storing them into that cue then needs no resync.
    - **No sequence selected:** its own anchor, taken when its first effect starts, dropped
      when the programmer is cleared.
    - **Selected sequence turned off:** back to its own anchor, taken at that moment;
      **another sequence selected:** to that sequence's current cue anchor — both snap or
      slide over Program Time.
    - Only the programmer follows: effects tracking in the sequence keep their own cue's
      anchor (no jump at Go), even if that leaves them out of sync with the programmer.
      Storing an out-of-sync effect into a running cue slides it to that cue's anchor over
      Program Time (handover below).
  - **Handover = phase slide, not crossfade.** An effect started randomly in the programmer
    and stored into a cue runs on the programmer's anchor until the programmer lets go
    (Clear, or a store that clears). If the cue is running, the effect then moves to the
    cue's anchor: **snapped** when Program Time is off, and with Program Time on the phase
    **slides** to the cue's sync over that time (per line, the shortest way, at most half a
    cycle; the integrated phase shifts, the amplitude never dips). The same slide applies
    whenever one effect line moves between two sources with different anchors. If the cue
    isn't running, nothing syncs: the effect stops with the programmer (Program Time fade)
    and the cue's next Go takes a fresh anchor.
  - Anchors are runtime playback state (replicated with the playback); on show load
    running sequences resume with a fresh anchor (as they resume snapped, §13).
  - `Random` forms are **seeded** (effect, line, fixture, cell, cycle number): the same
    every run and on every station of a LumaSession.
  - The clock is session-wide runtime (replicated with `Runtime`), so every station renders
    the same frame.
- **Running.** `Effect 5` turns the effect on, like MA1: a call in the user's programmer on
  the current selection, a running playback. Clearing the programmer stops it. `Store Cue`
  stores the call. A cue's call holds: effect, fixtures (group reference re-spreads on group
  edits), size (`Effect 5 At 60`), Rate, and a fixture subset to stop (`Off Effect 5 Fixture
  1 Thru 10`, per line with `Effect 5.2`). Call parameters track independently; size and
  Rate fade cue to cue — a cue scales Rate (`Effect 5 Rate 2` with a 4 s fade speeds the
  effect up over 4 s, phase never jumps), on top of the effect's master ratio. No
  per-fixture values in cues.
- **Stopping, four ways.** `Disable Effect 5.2 Fixture 3` (in the effect, all cues) ·
  `Off Effect 5 Fixture 3` (the cue's call: this effect stops for those fixtures, slots kept)
  · `Stomp Fixture 3` (a tracked layer: no effect runs on any attribute of the fixture,
  whatever playback it comes from, while the stomp's source is active; it outputs the static
  value underneath; `Stomp Fixture 3 Attribute Pan` narrows) · `Off Effect 5` (whole call).
  `Unstomp Fixture 3` stores a tracked removal (like a release: it clears only its own
  source's stomp). Verb-first only: `Fixture 3 Stomp` is refused. A static value in a later
  cue never stops an effect by itself (no auto-stomp). Stomps are visible: sheet/DMX-window
  marker with source cue, `List Stomp`, `IfStomp`.
- **Timing.** Entering/leaving an effect fades the fixture's effect **amplitude** (absolute:
  blend effect → static underneath; relative: offset → 0); the effect clock keeps running.
  Each cue part has effect timing (`EffectFade`, `EffectDelay`, out-fade/out-delay; default
  linked to the cue times), above the cue part times; dimmer lines leaving use the out-fade.
  Fanned delays (`Thru`, Pawtern) make fixtures leave in a wave. Snap = fade 0. Stomp/Unstomp
  use the same timing. Sequence off fades every call by the **OffCue** timing (also
  Kill/Swap). Not needed: PhaserSync (cue anchors keep calls in sync; rejoining fixtures drop
  into their slot), PhaserMorph (phase is not in cues), AutoStomp.
- **Cells.** A patched fixture is one object everywhere (patch, selection, grid, programmer,
  presets, cues); cells are never selectable and there are no subfixtures. The cell layout is
  defined per mode on the **FixtureType** (count, order, positions, per-cell vs shared
  attributes, master relations), imported from GDTF and editable in the fixture-type menu.
  Only the effect generator expands cells (a line's `Resolution: Fixture | Cell`); cell
  positions are transformed by the fixture's grid position/rotation. Overrides leave room for
  `Fixture 3.4`. GDTF mapping: walk from `DMXMode@Geometry`; each `GeometryReference` to a
  channel-bearing template is a cell; slot = channel offset + `Break.DMXOffset` − 1; order by
  offset, layout from composed `Position` matrices; root-geometry channels are shared;
  `Relation Multiply` = master dimmer; also accept plain nested geometries and nested
  references. **Prerequisite bug:** the importer drops geometries, so pixel fixtures import
  as one cell with a wrong footprint (`lumacore-gdtf/src/importer.rs`), and `resolver.rs`
  keys virtual channels by attribute name only.
- **Grid.** Groups store order + grid X/Y + **rotation/mirror** per member (fixtures only);
  §22's grid positions are not implemented yet (`Group` has only `fixture_ids`).
- **Syntax.**
  ```
  Effect 5                               on (programmer call on the selection)
  Off Effect 5 [Fixture …]               stop all / those fixtures; Effect 5.2 = one line
  Effect 5 At 60 · Effect 5 Rate 2       call size · master ratio
  Edit Effect 5 · Enter Effect 5         edit without running · edit target [Effect 5.2]
  Store Effect 5.2 /Attribute Pan /Form Circle
  Set Effect 5.2 Form Sine | BPM 120 | Width 50 | Direction Bounce | NShot 1
  Set Effect 5.2 Layout Grid | Axis X | Phase 0 Thru 360 | Pawtern "Wings2"
  Effect 5.2 Fixture 1 Thru 8 Lo 20 Hi 50 Thru 100 · Effect 5.2 Group 2 Hi Preset 4.3
  Reset Effect 5.2 Fixture 3 · Disable / Enable Effect 5[.2] Fixture 3
  Stomp Fixture 3 · Stomp Group 2 · Unstomp Fixture 3
  Assign Effect 5.2 At Master 3.1        speed master
  ```
  Edit target: explicit `Effect 5.2` > entered effect > the only running line on that fixture
  and attribute filter > else refused listing the candidates. Shared properties with a
  fixture (`Effect 5.2 Fixture 3 BPM 90`) are refused. New words (`Effect`, `Form` objects;
  `Lo`, `Hi`, `Rate`, `Disable`, `Enable`, `Stomp`, `Unstomp`, `IfStomp`) are published via
  the registries, `status: Planned` until built; errors in an `E-EFX-NNN` catalog.
  `Set Effect 5.2 Blend Multiply` sets a line's blend mode.
- **Hardening (2026-10-11).**
  - **Size is a master.** Absolute lines: size fades the strength (0% = the static value
    underneath, 100% = full effect) — e.g. a dimmer 0–100. Relative lines: size scales the
    offset from the middle out toward 0 (Lo −20 / Hi +20 at 50% runs −10…+10; an asymmetric
    0…+40 at 50% runs 0…+20; 0% = no offset). Fades, OffCue and Stomp use the same envelope.
  - **Replace conflicts.** Inside one effect, lines read top to bottom and the **bottom line
    wins**. Between effects/playbacks: higher priority wins, then the call whose **cue
    started latest** (deterministic: cue starts are replicated).
  - **Speed masters.** A line on a speed master takes the master's tempo as its BPM and keeps
    a ratio (1, ½, 2 …), so lines on one master stay locked; the call's Rate still scales.
    Tap/Learn changes tempo only — never phase: the cue start stays ground zero.
  - **Links by GUID.** Calls, Lo/Hi/step presets, forms and groups are GUID links, never
    pool numbers. Deleting an object that anything links to (an effect called by any cue, a
    form, a preset in an effect) names what will break and asks for confirmation.
  - **Editing claims.** `Edit` / `Enter Effect 5` claims the effect; other users' edits are
    refused until it is released.
  - **Groups are live.** A call on a group follows it: added fixtures join at once on the
    cue's clock and the pattern re-spreads; overrides stay.
  - **Lo/Hi is king — no steps in wave lines.** Effects have two line kinds:
    - **Wave line** (the normal one): attribute(s) + Lo/Hi + Form, per-fixture overrides.
    - **Prowl line**: links (GUID) a **Prowl** from its own DataPool pool (reusable; editing
      it changes every effect using it). Walking several presets is only done here; steps
      never exist in values or wave lines (§27's phaser steps stay removed).
    Both kinds share everything else: BPM/speed master, Rate, phase rule, layout
    (Linear/Grid), Pawtern, Blend, Disable, cue anchors — and the same merge position
    (Park > Stomp > effect band), so Stomp, Off, OffCue and fades treat them alike.
  - **Prowl.** A **gradient timeline** of stops (presets or values) with gradient sliders
    between them shaping each fade — no Forms. Stops sit freely but **snap to a beat grid**
    (½, ¼, ⅓ beat); the timeline is measured in beats at the line's BPM. Colour stops blend
    in a per-Prowl colour space (default perceptual; options: fixture channels, hue). The
    Prowl is the same for every fixture (fixtures differ by phase spread only; presets
    already carry per-fixture values) and is deliberately **not reachable from fixtures** —
    no per-fixture overrides; it is edited only in the dedicated Prowl editor. A Prowl line
    drives every attribute its stops hold; a stop that leaves one out holds the previous
    stop's value.
  - **Effects in the sequence sheet.** Each cue shows its effect calls as recipe lines,
    drawn MA1-style as squares stacked in a list. Selecting one gives its controls (size,
    Rate, effect fade/delay, fixture subset, pause) and `Update` writes them into that
    cue's call. Pattern values (Lo/Hi, Prowl) still live in the effect.
  - **Every cue its own clock.** No cross-sequence sync: things that must be in sync go in
    one sequence.
  - **Executors.** Effects run from programmers and cues, **and** can be assigned straight to
    an executor or fired from the pool (`Assign Effect 5 At Executor 1.1`: fader = size,
    Go/Off start/stop; the Go is its anchor). Effects started by a sequence's cues are
    shown on that sequence's executor.
  - **Pause on a cue.** A call can be stored paused (`Pause` in the call's square): it
    holds where it is when the pausing cue fires, gliding to a stop over the cue's effect
    timing. Pause tracks until a later cue stores Unpause. Resuming keeps the call's
    **original cue clock** — no new anchor: it slides back into sync with that clock over
    the un-pausing cue's effect timing.
  - **Effect timing fallback.** A cue part's effect fade/delay override the cue's general
    fade/delay when set; when only the normal fade or delay is set, effects use those.
  - **Modes.** Highlight never stops an effect or motion: it only puts the fixture type's
    highlight values on the attributes the highlight personality defines; every other
    attribute keeps its effect, and the effect clock never stops. Solo still shows the
    effect running on the soloed fixtures. Blind: an effect started or edited under Blind
    is, for live output, as if it is not running (it exists only in the preview). Freeze
    only lifts the programmer's priority (effects in it included); motion never stops.
- **Order.** Remove the phaser fields from `Value` · GDTF cell import fix + FixtureType cell layout · group grid with rotation ·
  effect band in `resolve` · Form/Effect pools, calls in programmer and cues, call tracking,
  OffCue · overrides, effect sheet and grid view · Pawtern spread, speed masters · stomp ·
  cell resolution.

