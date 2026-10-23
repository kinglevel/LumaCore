---
title: TODO — Object Model Gaps
category: docs
---

# TODO: Object Model

Design decisions and pending implementation work identified by comparing LumaCore against
an established console. Decisions confirmed in session 2026-10-09.

---

## Architectural decisions (intentional divergences from established consoles)

| Common approach | LumaCore approach | Why |
|---|---|---|
| Attribute definitions global on the live patch; per-user attribute preferences as thin override | Two-layer: global `AttributeDefinition` pool on `ShowData` + per-`UserProfile` `UserAttributeOverride` (encoder resolution, readout, note, hidden only) | Same split, but global layer lives on `ShowData` not the live patch; per-user layer is intentionally narrow |
| `FeatureGroup` determines both encoder page AND preset pool category | `FeatureGroup` determines encoder default grouping; `PresetGroup` is independently named with its own `preset_mode` | Free naming: "Stage Wash Colors", "Club Cyc"; encoder layout and preset storage decoupled |
| 9 fixed preset categories tied to `FeatureGroup` | Arbitrary named `PresetGroup`s with `preset_mode`; active group selected per `UserProfile` | `store preset 1` targets `selected_preset_group`; explicit form `store preset "Name".1` also valid |
| Appearance = icon + color combined handle | Separate `icon: Option<IconRef>` + `color: Option<[u8;4]>` | Cleaner split; color is RGBA tint, icon is a pool ref |
| 12 fixed FID type names (Fixture/Channel/…) | Show-definable namespaces with name + alias | "Houselights/HL" → operator types "HL 1 at full" |
| GUID not usable on Fixture | MVR UUID as primary Fixture GUID; user-changeable with safeguards | Round-trip MVR fidelity |
| `SCRIBBLE` pool (text labels) | Not planned | Not needed |

---

## Priority 1 — implement before Phase 3

### NodeMeta expansion

`NodeMeta { id, guid, name }` needs these fields on **all ShowData objects** (fixtures, sequences,
groups, presets, macros, layouts, timecodes, everything in the ShowData tree):

```rust
pub note: Option<String>,      // operator notes, survives show save
pub color: Option<[u8; 4]>,   // RGBA pool-button tint
pub icon: Option<IconRef>,     // ref into global icon set or show icon_pool
pub locked: bool,              // prevent accidental edits
pub hidden: bool,              // hide from operator views
```

`IconRef` is a discriminated ref:
```rust
pub enum IconRef {
    Builtin(String),   // key into LumaCore's global built-in icon set
    Show(u32),         // ID into ShowData.icon_pool (show-local custom)
}
```

**File:** `crates/lumacore-tree/src/tree.rs`

---

### ShowData icon pool

Global icon sets live in the LumaCore binary/asset bundle (not in shows). Show-local custom
icons live in a new pool on ShowData:

```rust
pub struct ShowIcon {
    pub meta: NodeMeta,
    pub data: ShowIconData,
}
pub enum ShowIconData {
    Builtin(String),      // mirrors a global key; present for explicit overrides
    Custom(Vec<u8>),      // raw PNG/SVG bytes
}
// add to ShowData:
pub icon_pool: BTreeMap<u32, ShowIcon>,
```

Only ShowData objects can have their icons changed (not RuntimeState), so shows are portable and
the full visual state is reconstructed on load.

**File:** `crates/lumacore-tree/src/show_data.rs` (or new `icon.rs`)

---

### Fixture: position, rotation, FID namespaces, MVR UUID

**File:** `crates/lumacore-tree/src/fixture.rs`

```rust
// Add to Fixture:
pub position: Option<[f32; 3]>,    // metres, Z-up right-handed (same as MVR)
pub rotation: Option<[f32; 3]>,    // degrees, XYZ Euler
pub fid_memberships: Vec<FidMembership>,
```

```rust
pub struct FidMembership {
    pub namespace_id: u32,
    pub fid: u32,
}
```

A fixture can belong to multiple namespaces with independent IDs in each (Fixture 5 can also
be HL 2 — different numbers, operator-assigned).

`NodeMeta.guid` IS the MVR UUID after import. The operator can reassign it with warnings;
no silent re-keying. Implement a guarded rename operation in the command layer (Phase 3).

**FID namespace table on ShowData:**

```rust
pub struct FidNamespace {
    pub id: u32,
    pub name: String,    // "Houselights"
    pub alias: String,   // "HL"
}
// add to ShowData (or DevicesData):
pub fid_namespaces: BTreeMap<u32, FidNamespace>,
```

Command parser resolves "HL 1" → find namespace by alias "HL" → find fixture with fid=1 in
that namespace_id.

**File for namespace table:** `crates/lumacore-tree/src/show_data.rs` or `devices.rs`

---

### Sequence settings

**File:** `crates/lumacore-playback/src/lib.rs`

Add `settings: SequenceSettings` to `SequenceObject`:

```rust
pub struct SequenceSettings {
    pub tracking: bool,                    // global tracking on/off for this sequence
    pub priority: PlaybackPriority,        // Ltp | Htp
    pub wrap_around: bool,                 // loop at end of cue list
    pub restart_mode: RestartMode,         // FirstCue | CurrentCue
    pub auto_start: bool,
    pub auto_stop: bool,
    pub soft_ltp: bool,                    // fade-to-off instead of snap on release
    pub xfade_mode: bool,                  // crossfade between cues
    pub mib_enabled: bool,                 // Move-In-Black globally
    pub swap_protect: bool,
    pub kill_protect: bool,
}

pub enum PlaybackPriority { Ltp, Htp }
pub enum RestartMode { FirstCue, CurrentCue }
```

Deferred: `INPUTFILTER`, `OUTPUTFILTER`, `PLAYBACKMASTER`, `SPEEDMASTER`, per-direction timing
presets (`TIMINGGOTO` etc.) — add when command layer needs them.

---

### Cue: tracking block + MIB preference

**File:** `crates/lumacore-playback/src/lib.rs`

A cue can be a "blocking" cue: no tracked values from prior cues pass through. Every attribute
NOT explicitly stored in a blocked cue's values is treated as released/no-change rather than
tracking forward.

```rust
// Add to CueObject:
pub blocked: bool,
pub mib_preference: MibPreference,
```

```rust
pub enum MibPreference { Normal, Forced, Inhibit }
```

Also extend `CueTrigger`:
```rust
pub enum CueTrigger {
    Go,
    Follow,
    Time(f64),
    // new:
    Sound,
    Bpm(f64),
    Midi,
}
```

---

## Priority 2 — Phase 3 (before Store/Recall Preset commands) ✓ DONE 2026-10-09

### PresetGroup / Preset (replaces PresetPool)

**File:** `crates/lumacore-tree/src/datapool.rs`

Replace `PresetPool` / `Preset` / `PresetValue` with:

```rust
/// A single attribute value stored in a preset.
///
/// From an export analysis (2026-10-09):
/// - `Percent` is an absolute 0–100 value: Dimmer, ColorRGB, Iris,
///   COLORTEMPERATURE, Tint, Shutter, COLORCROSSFADE — the overwhelming majority.
/// - `Physical` is a native-unit value (e.g. degrees): Pan, Tilt, Zoom.
/// - `Release` is the special release value.
/// Keeping Percent and Physical distinct lets attribute definitions add new physical units
/// later without a migration.
pub enum AttributeValue {
    Percent(f32),    // 0.0–100.0 — fixture-range-agnostic percent
    Physical(f32),   // native physical unit (degrees for angular attributes)
    Release,         // release this attribute from the programmer
}

pub struct Preset {
    pub meta: NodeMeta,                                     // color, icon, note, locked, hidden
    pub mode: PresetMode,                                   // Selective | Global | Universal
    pub values: Vec<PresetEntry>,                           // stored values
}

pub struct PresetEntry {
    pub fixture_id: Option<Uuid>,   // None = Universal/Global; Some = selective (per-fixture)
    pub attribute: String,          // GDTF canonical attribute name e.g. "Dimmer", "Pan"
    pub value: AttributeValue,
}

pub struct PresetGroup {
    pub meta: NodeMeta,
    pub preset_mode: PresetMode,    // Universal | Global | Selective — default mode for presets stored here
    pub presets: BTreeMap<u32, Preset>,
}
```

Replace `preset_pools: BTreeMap<u32, PresetPool>` in `DataPool` with:
```rust
pub preset_groups: BTreeMap<u32, PresetGroup>,
```

No DMX bytes anywhere. The `Percent`↔DMX and `Physical`↔DMX conversions happen at render
time using the fixture type's GDTF channel function data. This makes Universal presets
fixture-type-agnostic.

Not yet designed (add when needed): `INPUTFILTER`, `CUEPART`, per-value transition curves,
additional `Specials` variants (Off, Default — seen elsewhere but not yet needed).

---

### Attribute definitions — two-layer model

A common shape: attribute definitions live globally with feature groups and activation
groups; a per-user preference list (one entry per global attribute) is a thin override
layer covering only encoder resolution, readout mode, note, and hidden.

LumaCore mirrors this split with global canonical data on `ShowData` and a narrow per-user
override on `UserProfile`.

#### Global layer — add to `ShowData`

**File:** `crates/lumacore-tree/src/attribute_def.rs` (new) + `show_data.rs`

```rust
pub struct AttributeDefinition {
    pub name: String,                          // canonical GDTF name e.g. "Dimmer", "Pan"
    pub pretty: String,                        // short display label e.g. "Dim", "P"
    pub physical_unit: PhysicalUnit,           // e.g. LuminousIntensity, Angle — authoritative
    pub natural_readout: NaturalReadout,       // Percent | Physical
    pub feature_group_id: Option<u32>,         // ID into ShowData.feature_groups
    pub activation_group_id: Option<u32>,      // ID into ShowData.activation_groups
    pub main_attribute_id: Option<u32>,        // self-ref by ID; None = root attribute
    pub encoder_resolution: EncoderResolution, // global default
    pub intensity: f32,                        // 0.0–1.0 relative rendering weight
    pub mib: bool,                             // Move-In-Black applicable
}

pub enum PhysicalUnit {
    None, LuminousIntensity, Angle, ColorComponent, Frequency, // … full GDTF set
}
pub enum NaturalReadout { Percent, Physical }
pub enum EncoderResolution { Coarse, Fine, Ultra, Button }

pub struct FeatureGroup {
    pub id: u32,
    pub name: String,          // "Dimmer", "Position", "Color" — operator-editable
    pub preset_mode: PresetMode, // default PresetMode for presets stored from this group
}

pub struct ActivationGroup {
    pub id: u32,
    pub name: String,          // "PanTilt", "ColorRGB" — used to link mutually-exclusive attrs
}

// add to ShowData:
pub attribute_defs: BTreeMap<String, AttributeDefinition>,  // keyed by canonical name
pub feature_groups: BTreeMap<u32, FeatureGroup>,
pub activation_groups: BTreeMap<u32, ActivationGroup>,
```

`physical_unit` on `AttributeDefinition` is the authoritative source for display. The GDTF
importer populates it when building the standard attribute table; the render path reads
`ChannelFunction` for DMX math but display goes through this field. Fallback: `PhysicalUnit::None`
→ show bare 0–1.

Seed `attribute_defs` + `feature_groups` + `activation_groups` from the GDTF standard attribute
table on new show creation. These are show-level data — portable across user profiles and stations.

#### Per-user override layer — add to `UserProfile`

**File:** `crates/lumacore-tree/src/programmer.rs`

```rust
pub struct UserAttributeOverride {
    pub encoder_resolution: Option<EncoderResolution>, // None = use global default
    pub natural_readout: Option<NaturalReadout>,        // None = use global default
    pub note: Option<String>,
    pub hidden: bool,
}

// add to UserProfile:
pub attribute_overrides: BTreeMap<String, UserAttributeOverride>, // keyed by canonical name
pub selected_preset_group: u32,  // active PresetGroup ID for "store preset N" shorthand
```

`selected_preset_group` defaults to 1. `store preset 1` stores into
`data_pools[selected_datapool].preset_groups[selected_preset_group]` at slot 1.
Explicit form `store preset "Stage Wash Colors".1` bypasses the selection; numeric dot-notation
`store preset 3.1` addresses group 3, slot 1. Two users can have different active preset groups
simultaneously — LD in "Position", programmer in "Stage Wash Colors".

Encoder page/bank arrangement (which attributes appear on which encoder bank for this user) is
a separate concern from `UserAttributeOverride` — handled by `EncoderBarPool` per `UserProfile`
(not yet designed; add when UI layer needs it).

---

## What does NOT need to change

- DMX matrix (`lumacore-dmx`) — maps by `attribute_name` string, ignores grouping
- GDTF importer — produces `ChannelFunction` entries with `attribute_name` strings
- Programmer — uses `(fixture_guid, attribute_name) -> value`

---

## UI / console — not built yet (sessions 2026-10-09 – 2026-10-10)

Researched and decided, but not implemented. Design notes in
`docs/dev/ui-framework.md` (D4–D8) and `handoffs/2026-10-09-panels-focus-key-routing.md`.

### Sessions, users, clients
- [x] Logout / Disconnect → the app's user / server list (mDNS + beacon discovery, Start local
      server, manual address); `Connect`; login tokens (thin client phases 1–3,
      `handoffs/2026-10-10-thin-client.md`).
- [ ] Thin client: the UDP command port (:47653) and `/ws/monitor` are unauthenticated;
      diagnostics and `/api/shows` are public; SSE is not per user; cookie not
      HttpOnly; no TLS.
- [ ] Thin client: `Connect ▸` doesn't complete station names (discovery lives in the app, not the
      server); .app bundle (Info.plist ATS exception for `http://` servers, `Resources/frontend`).
- [ ] **Per-user keybindings** (`TuiApp::new` reads profile 1; `commands/settings.rs`).
- [ ] **Shortcut mode (KBD/PRG/NAV) per view-session** — still one global `AppState.shortcut_mode`.
- [ ] Per-client overlay state: tab completion and confirm prompts are shared session state, so a
      suggestion menu opened on one device shows on the user's other screens too.

### Windows and views
- [ ] More window types — only Terminal and Groups exist; most picker categories are empty.
- [ ] **Command Line** (input layer only) and **Command Line History** (output layer only) window
      types; the compositor already supports layer selection.
- [ ] `Move Window 5 At Window 2`, `Copy Window 5 At 2.9`.
- [ ] Stored views browsable in the object tree.
- [ ] Typed `Store View n` without `/Screen` stores directly (no Store View dialog);
      screenshot toggle for single-screen views.
- [x] Group taps run real commands: one pool window for every pool over `/ws/pools` (step 7, decisions §22).
- [ ] Other pools on the PoolDef (Presets, Sequences, Macros, …) — the window and feed are generic.
- [ ] A references view per missing object (cues / presets / groups pointing at a deleted fixture: Resurrect or Delete links) — decisions §22.
- [ ] Mouse wheel scrolling of the server-rendered output in terminals.

### Command line bar
- [ ] Virtual keyboard button (`ShowVKButton`) for tablets/phones.
- [ ] Link button (link a web remote's command line to the host's).
- [ ] Message center (message button with count) — no backing yet.
- [ ] System sample icons (CPU, memory, temperature, disk) and click actions on status icons
      (open the master / filter overlays).
- [ ] Encoder bar row under the command line bar.

### Shutdown menu
- [ ] Desk Lock (+ unlock), Reboot (console hardware only).
- [ ] Cancel a pending shutdown from a remote station within the countdown; `Shutdown IP …` /
      device targets; `/Wait`; update-and-shutdown.

### Menu (⚙) — greyed tiles
- [ ] Patch, Live Patch, Connector Configuration, Desk Lights & Color Theme, Network, In & Out,
      DMX Protocols, Show Creator, Preferences and Timings.
- [ ] Settings popup: User Configuration, Date and Time, USB Configuration, Software Update,
      Extension Configuration, Local Settings.

### Backup
- [ ] Drive selector (USB / removable drives, older-version libraries) and eject.
- [ ] Show history (previous saves of a show).
- [ ] Partial load masks (load only patch, data pools, …).
- [ ] Export media on show save; enumerate file name when the description changes.
- [ ] "Startup show failed to load" pop-up; now only logged.
- [ ] Autosave for untitled shows (only named shows autosave).
- [ ] Unify the old `Show "name" /save | /load` path with `SaveShow` / `LoadShow` (it doesn't update
      the current show / unsaved state).
- [ ] Unsaved tracking only sees undoable commands; settings changed outside them may not mark the
      show dirty.

### Housekeeping
- [ ] Fix the outdated `lumacore-http` unit tests (`src/tests/*.rs` build an old 3-field
      `HttpState` and lack the new `ConsoleBridge` methods).
- [ ] Tests for stacked rendering, window commands and the backup commands have not been run.

---

## Deferred (Priority 3)

| Item | Common name | Notes |
|---|---|---|
| Selection tricks | Selection-tricks pool | Selection distribution (fan/stagger/random) — no equivalent yet |
| `Agendas` | `Agendas` | Scheduled show events |
| `Remotes` | `Remotes` | Remote button config |
| `MVRxchange` | `MVRxchange` | MVR live exchange protocol |
| `MidiShowControl` | `MidiShowControl` | MIDI show control |
| `GelPools` | `GelPools` | Gel/filter library |
| `Quickeys` | `Quickeys` | Quick-key shortcut assignments |
| `Configurations` | `Configurations` | Named show configurations |
| Per-value tracking state | `Part.TRACKING` per value | After global sequence tracking switch works |
