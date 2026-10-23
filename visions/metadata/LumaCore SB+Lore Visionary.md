---
title: LumaCore SB+Lore Visionary
category: vision
---
# LumaCore Visionary UI
## A design direction for a next-generation lighting console in the browser

**Platform:** React application running in a web browser  
**Product:** LumaCore  
**Shell identity:** TreeShell  
**Core mascot / emotional anchor:** Luma, the black cat beneath the moon

---

## 1. Why this document exists

This document synthesizes the uploaded UI analyses, the LumaLore theme language, the LumaCore concept direction, and the visual identity suggested by the LumaCore TreeShell logo.

The goal is not to copy existing lighting consoles.
The goal is to define what **LumaCore should become**:

- as powerful as the industry-standard desks
- as visually modern as the newer touch-first systems
- more structurally honest than legacy consoles
- more browser-native than any traditional desk-on-a-PC clone
- more object-driven, inspectable, and future-facing than the old generation

LumaCore should feel like **a real console core exposed through a modern software shell**, not like a web app pretending to be a desk.

---

## 2. The vision in one sentence

**LumaCore is a dark, moonlit, object-native lighting control environment where operators climb structure instead of fighting windows, and where a browser-based UI feels as immediate, serious, and fast as a flagship console.**

---

## 3. What the references taught us

## Flagship desks: what to keep
The flagship desk lineage proves that users trust interfaces that are:

- dense when density is useful
- spatially stable
- modular
- state-rich
- tab-driven
- fast under pressure
- built around repeatable operational regions

It also shows that a lighting UI works best when it treats the screen like a **control surface**, not a marketing canvas.

## ETC / Apex-style direction: what to learn
The ETC-style references show the value of:

- better software hierarchy
- focused task panes
- touch-capable color tools
- modern modular workspaces
- information density without excessive faux hardware chrome

This is a strong bridge between traditional desk logic and better software design.

## Daslight: what to borrow
Daslight proves that a lighting UI can be:

- more visual
- more touch-native
- more effect-centric
- more approachable
- more direct-manipulation driven

Its strongest lesson is that **creative objects should be visible and graspable**, not buried under command grammar.

## Competitor legacy style: what to avoid
Older amber-heavy control surfaces have strengths:

- seriousness
- precision
- strong technical identity
- restrained feedback language

But they also risk becoming:

- too flat emotionally
- too sepia and old-world
- too cramped
- too tied to yesterday's hardware metaphors

LumaCore should inherit the discipline, but not the graveyard feeling.

---

## 4. The LumaCore personality

LumaCore is not "orange engineer UI" and it is not "generic SaaS dark mode".
It should feel like:

- moonlit, not tungsten
- deep, not muddy
- cosmic, not playful
- intelligent, not sterile
- alive, not loud
- serious, not corporate
- technical, but still a little magical

The TreeShell logo gives the emotional direction clearly:

- **deep night background**
- **full moon glow**
- **violet-to-cyan atmosphere**
- **sharp silhouette**
- **calm confidence**
- **curiosity and ascent**

This means the UI identity should be built around a feeling of **quiet night power**.
Not neon cyberpunk, not orange industrial, not silver hardware mimicry.

---

## 5. Core design principles

## 5.1 Console-first, browser-native
LumaCore runs in the browser, but should not look like a web dashboard.
It should behave like a professional console workspace with:

- near-zero ambiguity
- stable panel geography
- strong keyboard support
- fast touch targets
- minimal modal interruption
- local-feeling responsiveness

The browser is the transport layer.
The product identity is still a **console**.

## 5.2 Structure is visible
A central part of the Luma/TreeShell idea is that the system's object structure should not be hidden.
The operator should be able to move through:

- show
- datapool
- programmer
- selection
- worlds
- groups
- fixtures
- attributes
- recipes
- phasers
- runtime branches
- transport state
- network state

This should feel inspectable, navigable, and alive.
The UI must reveal the tree instead of flattening everything into disconnected pages.

## 5.3 Context matters
Luma acts where it stands.
That means the UI should constantly make clear:

- current object context
- current selection scope
- current destination
- current write target
- current mode
- current source of truth

A modern lighting UI should reduce the operator's need to mentally track invisible context.

## 5.4 Density with hierarchy
LumaCore should support dense expert views, but density must be **information-rich, not chrome-rich**.
We want:

- more content
- less fake hardware
- more semantic grouping
- less border noise
- more hierarchy
- less visual shouting

## 5.5 Color must mean something
Color should not be decoration.
It should communicate:

- mode
- family
- state
- confidence
- warning
- live value
- ownership of an object class

But the overall shell color identity must still belong to LumaCore.

---

## 6. Visual language

## 6.1 Base atmosphere
The base UI should sit in a dark nocturnal palette:

- background layers in near-black and blue-black
- panel surfaces in charcoal-indigo and quiet night grey
- subtle moonlit gradients, only where they help orientation
- restrained glow reserved for active state and key surfaces

The world should feel like a stage after the house lights go out.

## 6.2 Canonical brand colors
Derived from LumaLore and the logo:

- **moon-purple** `#af5fff`
- **deep console blue** `#5f87d7`
- **star-cyan** `#87d7ff`
- **quiet night grey** `#5f5f87`

Recommended supporting shell neutrals:

- **void black** `#090b14`
- **blue-black** `#0f1320`
- **panel graphite** `#171c2b`
- **raised surface** `#1f2740`
- **soft text** `#cdd5e6`
- **muted text** `#8e97ad`

## 6.3 Accent philosophy
The shell should primarily use:

- purple for identity and selected mode family
- deep blue for structure and active workspace framing
- cyan for interactive precision and live focus

Functional secondaries can still exist:

- green for healthy/live/allowed
- amber for caution, timing emphasis, or imported legacy semantics
- red for destructive or fault state
- yellow only where intensity/energy semantics truly benefit from it

This is important:
**amber should no longer be the soul of the UI.**
In LumaCore, amber is a utility accent, not the worldview.

## 6.4 Contrast model
The contrast strategy should be:

- dark fields recede
- active surfaces lift slightly
- outlines are thin and sparse
- selected objects glow subtly instead of yelling
- text contrast is high where speed matters, softer where it does not

This keeps the UI calm under load.

---

## 7. Layout philosophy

## 7.1 The shell model
The overall application should be understood as a **shell**, not just a set of pages.

Recommended top-level structure:

- **Top command/context rail**
- **Left TreeShell / object navigator**
- **Central working canvas**
- **Right inspector / detail / write targets**
- **Bottom live rail / playback / status / quick actions**

This gives the UI a durable operator geography.

## 7.2 Stable operational regions
One reason major consoles work is that their regions stay predictable.
LumaCore should preserve that idea while remaining modular.

Suggested stable roles:

### Top rail
For:
- current show context
- path / object breadcrumb
- transport / time / tempo
- command line
- record destination
- user/profile/world/session indicators
- quick search / universal jump

### Left rail: TreeShell
For:
- object tree
- filtered navigation
- show structure
- branch hopping
- scope and world switching
- favorites / pins / recents

### Center canvas
For:
- the primary task pane
- selection grids
- cue editors
- timeline views
- phaser editors
- stage / plot / 3D / patch surfaces
- matrices / recipes / modulator views

### Right inspector
For:
- current object details
- parameter editing
- property inspection
- linked references
- history / source tracing
- help for the current node

### Bottom rail
For:
- executor surfaces
- live values
- programmer state
- encoders / parameter family strip
- timing tools
- output meters
- compact notifications

## 7.3 Panels, not fake hardware
Every view should feel like a tool surface, but not by using metallic skeuomorphism.
Instead use:

- soft framed modules
- thin separators
- strong header rhythm
- clear local tabs
- consistent title rows
- docking and rearrangement
- grid logic and snap placement

This keeps it serious without feeling old.

---

## 8. Navigation model

## 8.1 Tabs still matter
The references show that tabs work in lighting because they mirror thought flow.
LumaCore should absolutely use tabs, but refine them.

Use tabs at three layers:

### Layer 1: workspace domains
Examples:
- Program
- Output
- Patch
- Timeline
- Stage
- Assets
- Network
- System

### Layer 2: local mode families
Examples:
- Dimmer
- Position
- Color
- Beam
- Focus
- FX
- Media
- Recipe

### Layer 3: contextual submodes
Examples:
- Fixtures / Groups / Presets
- Raw / Cooked / Source
- 2D / 3D / Table
- Steps / Curves / Distribution

## 8.2 Tabs should feel lunar, not office-software
Visually, tabs should be:

- low profile
- slightly rounded or gently cut
- dark by default
- lit by subtle purple/blue edge energy when active
- grouped tightly for speed
- readable in peripheral vision

Avoid overfilled candy-colored tabs.
They should feel like a **serious command strip under moonlight**.

## 8.3 Breadcrumb + tree + search
Browser software gives us an opportunity classic desks rarely exploit properly:

- fast global search
- path-based object jumping
- breadcrumb navigation
- pinned object history
- link-following between related objects

LumaCore should make object travel powerful.
This is one of its biggest differentiators.

---

## 9. Component language

## 9.1 Object tiles
The UI should treat core show objects as first-class visible tiles:

- groups
- presets
- sequences
- phasers
- recipes
- worlds
- macros
- layouts
- fixtures

Tiles should be:

- spatially memorable
- color-aware
- label-first, icon-assisted
- easy to scan
- easy to trigger
- easy to inspect

## 9.2 Tables and sheets
Tables remain essential for expert workflows.
But they should be cleaner than legacy sheets.

Desired qualities:

- restrained grid lines
- sticky headers
- selectable cells with strong focus state
- inline editing
- compact but readable text
- optional zebra depth, very subtle
- fast filter row

## 9.3 Inspectors
Every major object should have a consistent inspector model:

- summary
- properties
- links
- derived/runtime state
- history or source tracing
- notes/help

The inspector is where TreeShell becomes understandable.

## 9.4 Timing and motion editors
Borrow from the best sequencer and show-control ideas:

- precise dark tracks
- readable timing rulers
- clear markers
- layered curves
- optional lane folding
- palette-linked motion and color clips

But keep it console-fast, not video-editor bloated.

## 9.5 Color tools
Color is a hero task and deserves world-class treatment.
LumaCore color tooling should support:

- direct visual picking
- named swatches
- emitter-aware control
- gel/library browsing
- palette relationships
- brightness-linked variants
- compare and audition states

The ETC-style focused color panes and the more approachable Daslight-like direct manipulation should both influence this.

## 9.6 Command line and Lum bridge
Because Luma speaks Lum, command should not be a forgotten side feature.
The shell should let users:

- type direct object sentences
- see path-aware suggestions
- inspect what a command will target
- jump between typed and clicked workflows

A browser console that truly bridges structured UI and native command language would be a major identity win.

---

## 10. Interaction philosophy

## 10.1 Direct when visual, exact when technical
LumaCore should support two equally legitimate workflows:

### Visual/direct
- click or tap objects
- drag clips and ranges
- manipulate color and timing fields
- touch palettes and tiles

### Exact/technical
- type Lum commands
- edit numeric values directly
- inspect source links
- operate through sheets and structured panels

The best lighting software does not force one religion.
It lets operators move between both instantly.

## 10.2 Progressive disclosure
Newer systems do this well.
LumaCore should default to:

- simple first layer
- advanced controls on reveal
- expert density when requested

This makes the software scalable from laptop browser to large touch surface.

## 10.3 Multi-input from day one
The UI must respect:

- mouse
- touch
- keyboard
- encoders / mapped surfaces
- remote companion views

A browser-based console must still feel good on a desk, on a touchscreen, and on a support laptop.

---

## 11. What LumaCore should feel like in practice

When opening LumaCore, the operator should feel:

- oriented immediately
- not blinded by chrome
- surrounded by meaningful structure
- invited to explore the system tree
- confident that every object can be found again
- able to work visually or technically without friction
- like the UI belongs to a future console, not a repainted legacy emulator

The system should feel:

**deep, calm, fast, inspectable, modular, nocturnal, and alive.**

---

## 12. Anti-goals

LumaCore should not become:

- a fake flagship-desk clone in React
- an orange-brown legacy desk cosplay
- a generic enterprise dark dashboard
- a neon gamer interface
- a flat minimalist toy that hides needed structure
- a purely scene-launcher app with weak object depth
- a skeuomorphic metal panel simulator

It should respect the console world without being trapped by it.

---

## 13. A recommended stylistic synthesis

If the references are distilled into one direction, the recipe is:

### From the flagship desks
Take:
- stability
- operator seriousness
- modular logic
- state visibility
- strong task zoning

Leave behind:
- excessive harshness
- legacy desk mimicry
- too much visual fragmentation

### From ETC-style software
Take:
- focused panes
- refined modular composition
- better software hierarchy
- strong specialized tools

Leave behind:
- occasional UX stiffness
- overcompartmentalization

### From Daslight
Take:
- touch confidence
- approachable effect editing
- visual object workflow
- direct color interaction

Leave behind:
- prosumer softness when expert density is needed
- too much clip-app personality in core console views

### From LumaLore and the logo
Take everything essential:
- moon-purple identity
- deep blue shell structure
- cyan precision accents
- calm darkness
- black-cat confidence
- curiosity
- climbing through structure

This is the soul that makes LumaCore itself.

---

## 14. Suggested visual keywords

Use these words when designing the UI, writing prompts, or reviewing screens:

- moonlit console
- dark object shell
- TreeShell navigation
- software-defined desk
- nocturnal precision
- cosmic industrial
- calm high-density
- violet-blue control surface
- inspectable structure
- browser-native console
- living object tree
- serious but magical

---

## 15. One-line product description

**LumaCore is a moonlit, object-native lighting console for the browser, combining flagship desk clarity with modern modular software design and a visible TreeShell of inspectable show structure.**

---

## 16. Final direction

The future of lighting consoles is not a flatter fake desk.
It is not a prettier cue list.
It is not a touch skin over old mental models.

The future is:

- structural
- inspectable
- modular
- command-capable
- direct-manipulation friendly
- deeply themeable
- browser-deployable
- visually calm but semantically rich

That is what LumaCore should become.

Luma climbs the TreeShell.
The operator should feel that too.
