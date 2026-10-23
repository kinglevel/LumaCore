---
title: Requirements
category: docs
---
# Requirements: LumaCore

**Defined:** 2026-03-10
**Core Value:** A terminal-first lighting console with console-style object tree for patching GDTF fixtures, controlling attributes, and outputting DMX — open and extensible.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### GDTF & Fixture Management

- [x] **GDTF-01**: User can import a .gdtf file and extract channel maps and DMX modes
- [x] **GDTF-02**: User can patch a fixture to a universe and address using an imported GDTF profile (via terminal or web UI)
- [x] **GDTF-03**: System detects and prevents DMX address conflicts when patching
- [x] **GDTF-04**: User can view a list of all patched fixtures with their universe, address, and mode (`List Fixture`)
- [x] **GDTF-05**: User can unpatch (remove) a fixture (`Delete Fixture 1`)

### Terminal & Command System

- [x] **TERM-01**: User can enter console-inspired commands (`Fixture 1 Thru 10 At 50`)
- [x] **TERM-02**: Selection persists between commands as a first-class Programmer object
- [x] **TERM-03**: User can select fixtures with ranges (`1 Thru 10`), addition (`+`), subtraction (`-`)
- [x] **TERM-04**: User can select fixtures by name (`Fixture "Mac Aura 1"`) or number (`Fixture 1`)
- [x] **TERM-05**: User can filter selections (`Fixture 1 Thru 100 If Group "Wash"`)
- [x] **TERM-06**: User can navigate the object tree in console style (browse pools, objects)
- [x] **TERM-07**: Terminal provides tab completion for fixture names, group names, and commands
- [x] **TERM-08**: User can recall command history
- [x] **TERM-09**: Built-in help system: `Help At` for docs, `At?` for quick syntax hint, runnable examples
- [x] **TERM-10**: Configurable verbosity levels (summary / verbose / debug)
- [x] **TERM-11**: All commands logged to session log file automatically

### Command Verbs — Control

- [x] **CMD-01**: `At` — set dimmer percentage (`At 50`, `At Full`, `At Zero`)
- [x] **CMD-02**: `At` — set raw attribute values (`Attribute "Pan" At 127`)
- [x] **CMD-03**: `At` — apply preset (`At Preset 4.1`, `At Color 1`)
- [x] **CMD-04**: `Full` / `Zero` — shortcut dimmer commands
- [x] **CMD-05**: `On` / `Off` — toggle fixture output
- [x] **CMD-06**: `Highlight` — highlight selected fixtures for physical identification
- [x] **CMD-07**: `Solo` — solo selected fixtures (dim everything else)
- [x] **CMD-08**: `Black` — blackout all output
- [x] **CMD-09**: `Park` / `Unpark` — lock fixture (`Park Fixture 1`), channel (`Park DMX 1.1`), or named universe (`Park DMX "Truss".234`)
- [x] **CMD-10**: `Clear` — deselect + release programmer values (console style)

### Command Verbs — Edit

- [x] **CMD-11**: `Store Group` — save current selection as named group
- [x] **CMD-12**: `Store Preset` — save current values to preset pool (`Store Color 1`, `Store Dimmer 1`)
- [x] **CMD-13**: `Delete` — remove any object from the tree (`Delete Fixture 1`, `Delete Group 2`)
- [x] **CMD-14**: `Copy` — duplicate objects (`Copy Group 1 At 2`)
- [x] **CMD-15**: `Update` — update existing object with current values
- [x] **CMD-16**: `Edit` — enter edit mode for an object

### Command Verbs — Inspection

- [x] **CMD-17**: `Info` — human-readable inspection (`Info Fixture 1` → attributes, DMX mapping, current values)
- [x] **CMD-18**: `Dump` — raw debug data dump (`Dump Fixture 1` → internal state)
- [x] **CMD-19**: `List` — universal list command (`List Fixture`, `List Group`, `List Universe`)

### Command Verbs — Undo

- [x] **CMD-20**: `Oops` — full undo stack covering both terminal commands AND web UI actions
- [x] **CMD-21**: Multiple Oops goes back further through undo history

### Programmer

- [x] **PROG-01**: Programmer tracks all touched/active values (console style)
- [x] **PROG-02**: Store saves from Programmer state
- [x] **PROG-03**: Clear releases Programmer values back to default/stored state

### Preset Pools

- [ ] **POOL-01**: Dimmer preset pool — store/recall dimmer presets
- [ ] **POOL-02**: Color preset pool — store/recall color presets with named pools (`Preset "DiscoColors"."dance"`)
- [ ] **POOL-03**: Position preset pool — store/recall pan/tilt presets
- [ ] **POOL-04**: Universal preset pool — store/recall all attributes together
- [ ] **POOL-05**: Preset pools addressable by number (`Color 1`) or name (`Preset "MyColors"."warm"`)

### Fixture Control

- [x] **CTRL-01**: User can control GDTF-defined attributes (color, position, gobo) on selected fixtures
- [x] **CTRL-02**: User can recall groups to restore fixture selection (`Group 1`, `Group "Front Wash"`)
- [x] **CTRL-03**: User can edit groups (add/remove fixtures)

### Universe & Output Management

- [x] **UNIV-01**: Universes are named + numbered objects (`Universe 1 "Front Truss"`)
- [x] **UNIV-02**: User can assign universe to sACN output via terminal (`Assign Universe 1 At sACN 1`)
- [x] **UNIV-03**: User can range-assign universes (`Assign Universe 2 Thru 8 At sACN 1 Thru 6`) — system tracks ranges as units
- [x] **UNIV-04**: User can merge universe ranges (`Merge Universe 1 Thru 2` → creates contiguous range)
- [x] **UNIV-05**: User can split universe ranges (`Split Universe 1 Thru 4 At 2` → [1-2] + [3-4])
- [x] **UNIV-06**: Output protocol assignments intelligently follow splits, merges, and moves

### DMX Output

- [x] **DMX-01**: System outputs DMX via sACN (E1.31) protocol
- [x] **DMX-02**: System supports both multicast and unicast sACN modes
- [x] **DMX-03**: System supports multiple DMX universes simultaneously
- [x] **DMX-04**: DMX output maintains consistent 44fps timing without visible flicker

### Macro System

- [ ] **MACRO-01**: User can record command sequences (`Macro Record 1`, run commands, `Macro Stop`)
- [ ] **MACRO-02**: User can replay macros (`Macro 1`)

### Watch & Monitoring

- [ ] **WATCH-01**: `Watch DMX 1` — live-updating DMX universe monitor in terminal
- [ ] **WATCH-02**: `Watch Fixture 1` — live-updating attribute values for a fixture
- [ ] **WATCH-03**: `Watch Selection` — auto-update display as selection changes
- [ ] **WATCH-04**: `Watch Output` — monitor sACN packet rate, timing stats, dropped frames

### Dev & Debug Loop

- [ ] **DBG-01**: User can inspect live DMX output values per universe/channel in terminal
- [ ] **DBG-02**: User can inspect object tree state to verify commands worked
- [ ] **DBG-03**: User can run a command and see resulting DMX channel changes (diff view)
- [ ] **DBG-04**: Lua system tests with pre/cmd/post/cleanup stages (reference console system test pattern)
- [ ] **DBG-05**: Lua tests can assert expected DMX output after commands
- [ ] **DBG-06**: Lua tests can assert object tree state after commands
- [ ] **DBG-07**: Lua tests can call API and verify JSON responses
- [ ] **DBG-08**: Lua tests can capture web UI screenshots for visual verification
- [ ] **DBG-09**: Debug loop is fully automated — run test suite, get pass/fail report

### Show Persistence

- [ ] **SHOW-01**: User can save the current show (patch, groups, presets, output config, macros) to a file
- [ ] **SHOW-02**: User can load a previously saved show file

### API & Web UI

- [ ] **API-01**: System runs as a headless server with REST API
- [ ] **API-02**: System provides WebSocket for real-time fixture state updates
- [ ] **API-03**: Web UI displays patched fixtures and their current values
- [ ] **API-04**: Web UI allows basic fixture control (select, set values)
- [ ] **API-05**: Multiple web clients can connect simultaneously
- [ ] **API-06**: Undo stack is unified — Oops works for both terminal and UI changes

### Documentation & Build

- [ ] **DOC-01**: Build process documented for contributors and vibe-coders
- [ ] **DOC-02**: Cross-platform build instructions for Linux, macOS, and Windows

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### DMX Output Expansion

- **DMX-05**: USB DMX output via ENTTEC/FTDI dongles
- **DMX-06**: Art-Net output (unicast, UDP port 6454)
- **DMX-07**: Art-Net polling for node discovery

### Advanced Fixtures

- **GDTF-06**: Multi-break fixture support (LED bars, pixel-mapped fixtures)
- **GDTF-07**: GDTF Share library compatibility (defensive parsing for 10k+ community files)
- **GDTF-08**: Full GDTF geometry, wheels, and physical properties
- **GDTF-09**: Subfixture addressing and control

### Color System Expansion

- **CLR-01**: Gel command (`At Gel "Lee"."119"`) with named gel libraries
- **CLR-02**: Smart color names (`At Color "Red"` — automatic resolution)
- **CLR-03**: Hex color input (`At Color #FF0000`)
- **CLR-04**: CIE xy coordinate input (`At Color CIE 0.64 0.33`)
- **CLR-05**: Color temperature input (`At Color CCT 3200`)

### Cue System

- **CUE-01**: Sequence/cue list with go/back and timing
- **CUE-02**: Effects engine (chase, rainbow, strobe patterns)

### Timeline & MIDI

- **TIME-01**: DAW-style timeline with tracks triggering sequences (Reaper/Cubase-style grid)
- **TIME-02**: Timeline looping and grid snapping
- **TIME-03**: MIDI input for timeline
- **TIME-04**: MIDI file import/export
- **TIME-05**: Timecode sync (LTC/MTC)

### Plugin & Platform

- **PLUG-01**: Lua 5.4 plugin system for user-extensible modules
- **PLUG-02**: Custom effect generator plugins
- **PLUG-03**: Plugin API versioning
- **PLUG-04**: Commands Pool — user-created commands backed by Lua with visual node editor
- **DESK-01**: Desktop shell (native-looking wrapper) — built as `apps/lumacore-app` (wry/tao): a launcher + thin client that compiles only `lumacore-wire` and starts `lumacore-server` as its child (decisions §23 "Step 8"; supersedes the March "Tauri runs the engine in-process")
- **DESK-02**: 3D visualizer with Rust GPU rendering streamed to web UI

### Integration

- **INTG-01**: MIDI/OSC input for external control
- **INTG-02**: MVR-xchange for visualizer connectivity

## Out of Scope

| Feature | Reason |
|---------|--------|
| Mobile native app | Web UI accessible on mobile browsers is sufficient |
| Video/media server integration | Different domain, future scope |
| Real-time chat/collaboration | Not a console feature |
| Proprietary fixture library format | GDTF is the standard, no custom format |
| reference console show file import | Legal concerns, different data model |
| Modifying built-in command verbs | Vanilla commands are immutable; users extend via Commands Pool |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| GDTF-01 | Phase 1 | Complete |
| GDTF-02 | Phase 1 | Complete |
| GDTF-03 | Phase 1 | Complete |
| GDTF-04 | Phase 1 | Complete |
| GDTF-05 | Phase 1 | Complete |
| UNIV-01 | Phase 1 | Complete |
| UNIV-02 | Phase 1 | Complete |
| UNIV-03 | Phase 1 | Complete |
| UNIV-04 | Phase 1 | Complete |
| UNIV-05 | Phase 1 | Complete |
| UNIV-06 | Phase 1 | Complete |
| DMX-01 | Phase 1 | Complete |
| DMX-02 | Phase 1 | Complete |
| DMX-03 | Phase 1 | Complete |
| DMX-04 | Phase 1 | Complete |
| TERM-01 | Phase 2 | Complete |
| TERM-02 | Phase 2 | Complete |
| TERM-03 | Phase 2 | Complete |
| TERM-04 | Phase 2 | Complete |
| TERM-05 | Phase 2 | Complete |
| TERM-06 | Phase 2 | Complete |
| TERM-07 | Phase 2 | Complete |
| TERM-08 | Phase 2 | Complete |
| TERM-09 | Phase 2 | Complete |
| TERM-10 | Phase 2 | Complete |
| TERM-11 | Phase 2 | Complete |
| CMD-01 | Phase 2 | Complete |
| CMD-02 | Phase 2 | Complete |
| CMD-03 | Phase 2 | Complete |
| CMD-04 | Phase 2 | Complete |
| CMD-05 | Phase 2 | Complete |
| CMD-06 | Phase 2 | Complete |
| CMD-07 | Phase 2 | Complete |
| CMD-08 | Phase 2 | Complete |
| CMD-09 | Phase 2 | Complete |
| CMD-10 | Phase 2 | Complete |
| CMD-11 | Phase 2 | Complete |
| CMD-12 | Phase 2 | Complete |
| CMD-13 | Phase 2 | Complete |
| CMD-14 | Phase 2 | Complete |
| CMD-15 | Phase 2 | Complete |
| CMD-16 | Phase 2 | Complete |
| CMD-17 | Phase 2 | Complete |
| CMD-18 | Phase 2 | Complete |
| CMD-19 | Phase 2 | Complete |
| CMD-20 | Phase 2 | Complete |
| CMD-21 | Phase 2 | Complete |
| PROG-01 | Phase 2 | Complete |
| PROG-02 | Phase 2 | Complete |
| PROG-03 | Phase 2 | Complete |
| CTRL-01 | Phase 2 | Complete |
| CTRL-02 | Phase 2 | Complete |
| CTRL-03 | Phase 2 | Complete |
| POOL-01 | Phase 3 | Pending |
| POOL-02 | Phase 3 | Pending |
| POOL-03 | Phase 3 | Pending |
| POOL-04 | Phase 3 | Pending |
| POOL-05 | Phase 3 | Pending |
| MACRO-01 | Phase 3 | Pending |
| MACRO-02 | Phase 3 | Pending |
| WATCH-01 | Phase 3 | Pending |
| WATCH-02 | Phase 3 | Pending |
| WATCH-03 | Phase 3 | Pending |
| WATCH-04 | Phase 3 | Pending |
| DBG-01 | Phase 3 | Pending |
| DBG-02 | Phase 3 | Pending |
| DBG-03 | Phase 3 | Pending |
| DBG-04 | Phase 3 | Pending |
| DBG-05 | Phase 3 | Pending |
| DBG-06 | Phase 3 | Pending |
| DBG-07 | Phase 3 | Pending |
| DBG-08 | Phase 3 | Pending |
| DBG-09 | Phase 3 | Pending |
| SHOW-01 | Phase 3 | Pending |
| SHOW-02 | Phase 3 | Pending |
| API-01 | Phase 4 | Pending |
| API-02 | Phase 4 | Pending |
| API-03 | Phase 4 | Pending |
| API-04 | Phase 4 | Pending |
| API-05 | Phase 4 | Pending |
| API-06 | Phase 4 | Pending |
| DOC-01 | Phase 4 | Pending |
| DOC-02 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 83 total
- Mapped to phases: 83
- Unmapped: 0

---
*Requirements defined: 2026-03-10*
*Last updated: 2026-03-10 — traceability mapped to phases after roadmap creation*
