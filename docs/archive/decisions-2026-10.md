---
title: Architecture Decisions
category: docs
---
# LumaCore Architecture Decisions

**Created:** 2026-03-10
**Status:** Living document — records architectural decisions made during design discussions

---

## Contents and status (2026-10-11)

Status per section against the code (audit of 2026-10-11, step 11a). **built** = in code ·
**partly** = some of it · **not built** = decided only · **record** = history/superseded parts
noted inline. The one plan of what's next is [roadmap.md](roadmap.md).

| § | Topic | Status |
|---|---|---|
| [1](#1-object-tree-structure) | Object tree | partly — MessageCenter, Scenes, Tags, Agendas, Media, RDMData not built |
| [2](#2-programmer-ownership) | Programmer ownership | built, superseded in parts by §21 (users own programmers) |
| [3](#3-pawtern-system-replaces-distribute) | Pawtern | not built (group slot reserved) |
| [4](#4-claims-system) | Claims | built (§21 "6d") |
| [5](#5-variable-system--three-tiers) | Variables | partly — GlobalVar only; ShowVar/UserVar not built |
| [6](#6-scenes--stateful-logic-configurations) | Scenes | not built |
| [7](#7-output-compositing--priority-stack) | Output compositing | partly — merge in §27; priority bands not built |
| [8](#8-show-file-format-luma) | Show file format | built (zip of the tree + manifest since format 7, now format 10; embedded media, asset roots); preload + media sync not built |
| [9](#9-autosave--show-recovery) | Autosave | partly — signal/merge/IdleMaster autosaves; network-synced autosave not built |
| [10](#10-network-session--multi-station-sync) | LumaSession sync | built; station states superseded by §21 |
| [11](#11-protocol-architecture) | Protocols | built (4b–4d); Live Patch offsets, Patch menu not built |
| [12](#12-tags--visual-browser-future) | Tags | hardened 2026-10-17 (a tree of tags, GUID links in NodeMeta, EditSetting Meta + Tags, pool window, `lumacore-tags`); lum.md written; built before 34g4's patch |
| [13](#13-playbacks) | Playbacks | partly — see §27 |
| [14](#14-commands-decided) | Commands decided | partly |
| [15](#15-one-registry-one-namespace-2026-10-10) | One registry | built; `Planned` per command and object verb (11b) |
| [16](#16-mode-objects--blind-highlight-freeze-solo-2026-10-10) | Mode objects | built (Highlight output + Highlight/Solo masters in 11b; Blind/Lowlight masters later) |
| [17](#17-keys-and-keymaps-2026-10-10) | Keys | built |
| [18](#18-naming-2026-10-10) | Naming | built (renames in 11b) |
| [19](#19-network-security-2026-10-10) | Security | built |
| [20](#20-show-file-versioning-2026-10-10) | Show versioning | built |
| [21](#21-undo--replication-coverage-2026-10-10) | Undo / replication | built; Oops categories + UI Oops in step 11c |
| [22](#22-groups-2026-10-10) | Groups | built; references view not built |
| [23](#23-code-structure-2026-10-10) | Code structure | built |
| [24](#24-render-thread-rules-2026-10-10) | Render thread + clock | built |
| [26](#26-engine-ownership-errors-and-robustness-2026-10-10) | Engine ownership, errors | built |
| [27](#27-playback-engine-2026-10-10) | Playback engine | partly — cue timing, selected sequence, Highlight built in 11b; bands, executors later |
| [25](#25-guards-and-work-order-2026-10-10) | Guards + work order | built (step 10; findings fixed and sandbox added in 11b) |
| [28](#28-effects-2026-10-11) | Effects | not built |
| [29](#29-cleanup--docs-findings-oops-categories-2026-10-11-step-11-hardening) | Cleanup (step 11) | built — 11a, 11b, 11c, 11d |
| [30](#30-touch-ui-kit-2026-10-12-step-12-hardening) | Touch UI kit (step 12) | 12a1 + 12a2 + 12a3 + 12b + 12b2 + 12c + 12c2 built; 12d part 1 (feeds) + part 2 (rights) + part 3 (grid, slab, setup frame, MenuDef) + part 4 (menus on the frame, station table) + part 5 (Backup) built; 12e hardened + design locked; 12e parts 1–4 built |
| [31](#31-code-review-hardening-2026-10-13) | Code review hardening | not built — decided 2026-10-13 |
| [32](#32-binary-study-hardening-2026-10-14) | Binary study: playbacks, echoes, effect targets | being decided — asked one at a time |
| [33](#33-lum-syntax-crate-hardening-2026-10-14) | Lum syntax crate (strict grammar) | decided 2026-10-15 (56 decisions); lum.md is the spec; parts 1–2 + 3a built on branch `lum-syntax` |
| [34](#34-code-structure-and-file-headers-2026-10-15) | Code structure and file headers | decided 2026-10-15; 34a–34d built (state + objects crates, dispatch down, depth cap); 34g4 patch hardened 2026-10-17 (Devices › DMX › Patch, MVR import / export; not built) |
| [35](#35-one-object-door-store-delete-copy-move-paste-2026-10-15) | One object door: Store / Delete / Copy / Move / Paste | decided 2026-10-15 (19 decisions, Lum words in lum.md); 35a, 35b, 35e, 35f, 35g1–35g3 built (the door; Group, Sequence, TimingPreset, CurvePreset, Output, Input, Fixture, Universe on it; the grid slab on the station's clipboard; Cut / Paste / Insert / Compact; quoted `*`; mv / cp / rm) |
| [36](#36-webui-the-thin-client-and-3d-rendering-2026-10-18) | WebUI, the thin client and 3D rendering | hardened 2026-10-18 (11 decisions); 1–5 built 2026-10-18 (roadmap 3b); 3D (6–10) not built |
| [37](#37-phone-hardening-2026-10-19) | Phone hardening | hardened 2026-10-19 (the audit; 7 decisions); built 2026-10-19 |
| [38](#38-drives-and-the-one-files-element-2026-10-19) | Drives and the one files element | hardened 2026-10-19 (6 questions, asked while hardening Import MVR); 38a–38c built 2026-10-20 |
| [39](#39-the-settings-home-icons-2026-10-20) | The settings home icons | decided and built 2026-10-20 |
| [40](#40-the-demo-rig-nested-fixtures-and-second-ids-2026-10-20) | The demo rig, nested fixtures and second ids | hardened 2026-10-20, 17 questions; the GDTF reader's fork built, the rest waits for the go |

---

## 1. Object Tree Structure

The definitive LumaCore object tree. This supersedes any earlier tree diagrams.

*(Note 2026-10-11, step 11a: the tree as built is in [../user/object-tree/object-tree.md](../user/object-tree/object-tree.md); MessageCenter, Scenes, Tags, Agendas, Media and RDMData below are not built.)*

```
Root
├── MessageCenter
│   ├── System (Spam / Warnings / Errors / Alerts)
│   ├── Cmdline
│   ├── Network
│   └── Power
│
├── ShowData
│   ├── Patch (claimable)
│   │   ├── FixtureTypes
│   │   ├── Fixtures
│   │   └── Universes
│   │       ├── DmxAddresses
│   │       ├── Layers
│   │       ├── Stages
│   │       └── Classes
│   │
│   ├── 3D / Environment (independently claimable)
│   │   └── Stage geometries, positions, 3D world
│   │
│   ├── DataPools
│   │   └── Default (or named pools)
│   │       ├── Groups
│   │       ├── Sequences
│   │       ├── Scenes (stateful, node-based, logic-driven — see §6)
│   │       ├── Macros
│   │       ├── Plugins
│   │       ├── PresetPools
│   │       ├── Layouts
│   │       ├── Pawterns (see §3)
│   │       ├── Effects (standalone, old-school — not buried in sequences)
│   │       ├── Priorities (user-defined priority pool — see §7)
│   │       ├── Filters
│   │       ├── Timers
│   │       ├── Timecodes (timecode shows / event lists)
│   │       └── Worlds (fixture filter subsets)
│   │
│   ├── UserProfiles
│   │   └── {Username}
│   │       ├── Programmer (permanently claimed by user — see §2)
│   │       ├── (undo lives in Runtime.Users.{User}.Undo — never saved, §21)
│   │       ├── Views
│   │       ├── ScreenConfigurations
│   │       ├── KeyboardShortCuts
│   │       ├── UserVariables (travels with profile — see §5)
│   │       ├── StorePreferences
│   │       ├── EncoderBarPool
│   │       ├── SmartViewPool
│   │       └── Cameras
│   │
│   ├── LivePatch (runtime-resolved channel data)
│   │   ├── AttributeDefinitions
│   │   └── Options (invert DMX, etc. — editable without network reload)
│   │
│   ├── Protocols
│   │   ├── Outputs (sACN, Art-Net, USB DMX, MIDI, PSN, UDP, HTTP POST)
│   │   ├── Inputs (MIDI, OSC, Remote triggers, PSN, UDP, HTTP GET, Timecode)
│   │   └── Bidirectional (OSC devices, web clients — protocols with feedback)
│   │
│   ├── Media (manifest only — references external assets)
│   │   └── References to images, sound files, video
│   │       (actual files in user-managed folders, locally/USB/network)
│   │
│   ├── Appearances
│   ├── Materials
│   ├── Meshes
│   ├── Scribbles
│   ├── Masters
│   │   ├── Grand (master dimmer)
│   │   ├── Speed (BPM/rate masters)
│   │   ├── Group (per-group intensity masters — mode on the DataPool Group, level in Playbacks, §22)
│   │   └── Playback (per-executor masters)
│   │
│   ├── Tags (fixture metadata — position, function, color)
│   ├── Agendas
│   ├── Playbacks (what's running — saved with show, recalls on load)
│   ├── Variables (ShowVariables — saved with show file)
│   └── ShowSettings
│       └── Autosave config (master-initiated, network-synced)
│
├── RDMData (volatile — can disappear, not critical)
│
├── Temp (runtime only — never saved)
│   ├── Users
│   │   └── {Username}
│   │       ├── ProgrammerCache
│   │       └── SessionState
│   ├── GlobalVariables (volatile — gone on restart)
│   ├── MidiMonitor
│   ├── ProgLayers
│   └── RunningPlaybacks
│
├── StationSettings (per-machine, not saved with show)
├── ColorTheme
├── GraphicsRoot
├── Menus
└── WebServer
```

---

## 2. Programmer Ownership

The Programmer is **permanently claimed** by its owning user. It cannot be unclaimed or transferred. It IS the user's workspace.

- Arc-wrapped (`Arc<RwLock<Programmer>>`) for output pipeline access
  *(Note 2026-10-11, step 11a: superseded — programmers live on `ShowData.Users` (§21 "Step 6"), the state thread owns them (§26) and the engine reads published snapshots (§27).)*
- Output thread reads programmer values without blocking the REPL
- Prompt indicator: `[Programmer]` is always present for the active user
- Future: other input modes beyond Programmer (Voice, AI, etc.)

### Selection Modes

The programmer supports different selection modes, shown in the prompt:

```
[Programmer:Select]     — standard fixture selection
[Programmer:Grid]       — grid-based selection
```

Future input systems (e.g., Voice:Listen) are a separate concept from selection modes — they would be additional input methods, not programmer modes.

Users can switch selection logic. The mode indicator updates in the path bar.

### Selection Order

Selection is an **ordered list** (`Vec<u32>`), not a set:
- `Fixture 1 Thru 6` → order [1,2,3,4,5,6]
- `Fixture 6 Thru 1` → order [6,5,4,3,2,1]
- **One range rule for every object word** (R1-34, Q6; `parser::number_list` and the selection
  parser): numbers that don't exist are skipped — the list closes up, no holes — and only none at
  all is an error; open ranges `5 Thru` (to the last) and `Thru 5` (from the first); `-` takes out;
  only existing numbers are visited, so `Thru 4294967295` costs nothing; a bare `Thru` is every
  number (`Thru - 3`). Every verb uses it (select, Delete, Copy, Move, Assign, On/Off). Copy,
  Move and Assign keep the sources' **spacing** (a gap stays a gap, counted from the first in
  typed order), and a Copy or Move checks every target first — it never leaves half a copy.
  Pool numbers are capped (Groups 1–9999) only for **new** numbers; an older show's group above
  the cap is named, labelled, moved down and deleted like any other (R1 review 10, 14).
- `Shuffle` randomizes the order
- `Next` / `Previous` walks the ordered list
- Groups preserve stored selection order on recall

---

## 3. Pawtern System (replaces "Distribute")

*Revised 2026-10-10:* **Pawtern** replaces the earlier `Distribute` verb. A Pawtern is a
DataPool object and also the action — `Pawtern 3` applies it to the selection, `Store Pawtern 3`
stores one. Groups reserve an (empty for now) `pawtern` slot so the show format needs no
migration when the feature lands.

```
Pawtern "Arc"          — arc distribution across selection
Pawtern "Rainbow"      — spread through color wheel
Pawtern "Linear"       — even linear spread
```

### Pawtern Contexts

Different systems apply typed Pawterns:

| Context | Command | Pawtern Types |
|---------|---------|-------------|
| **Grid** | `Grid Pawtern "Arc"` | 2D grid distributions |
| **3D** | `3D Pawtern "Radial"` | 3D spatial distributions |
| **Linear** | `Pawtern "Linear"` | 1D selection-order distributions |

Pawterns are typed — some only work in 3D, some only for grids. The pool holds all types with metadata about their valid contexts.

Node-based distribution editor is a future UI feature. The pool structure supports it from day one.

---

## 4. Claims System

Claims provide granular multi-user locking by tree path.

```
claim Patch           — lock Patch for editing
claim ShowData        — lock entire ShowData directory
claim Environment     — lock 3D environment independently
```

- Any tree node can be claimable
- Claiming a parent claims all children
- Admins can unclaim anyone
- `cd` into a claimable area is read-only (no auto-claim)
- Bare object keyword (e.g., `patch`) is the "quick claim" shortcut
- Programmer is permanently claimed (see §2)

**Programmers and fixture claims (2026-10-10, step 4 hardening)** — LumaCore's own model
(established consoles have no attribute locks):

- **Programmers are LTP, like sequences.** Any user can set any attribute; the latest touch
  wins the output, and the value **moves** into the toucher's programmer (it leaves the other
  programmer). Several operators can follow actors and next through fixtures without
  collisions. Programmers also merge LTP with playbacks (§7, Freeze lifts above).
- **Selecting does not claim.** Two users can select the same fixture. Today's derived
  "fixture-selection claim" (`ensure_fixture_selection_available`) goes.
- **Blind** values don't touch the stage and take nothing. Leaving Blind counts as a fresh
  touch: B's values take over (and move out of the other programmers).
- Store records the user's own active values.
- **`Claim Fixture 3`** locks the whole fixture: other users can't select it or set values for
  it. Claiming a fixture whose attribute another user holds live is refused, naming the
  holder. Explicit claims stay in the claims table; "who holds what" is derived from the
  programmers (no lock table).
- `Clear Programmer User 2 Thru 5` (Admin + Setup; the user is notified and can undo) is built
  with this in step 4a.

---

## 5. Variable System — Three Tiers

| Scope | Location | Saved? | Volatile? | Syntax |
|-------|----------|--------|-----------|--------|
| **UserVariable** | `UserProfiles.{name}.Variables` | Yes (travels with profile) | No | `Set $myVar 42` |
| **ShowVariable** | `ShowData.Variables` | Yes (travels with show) | No | `Set Show.$timeout 5` |
| **GlobalVariable** | `Root.Temp.GlobalVariables` | No | Yes (session only) | `Set Global.$debugFlag 1` |

UserVariables follow the UserProfile on export/import.

---

## 6. Scenes — Stateful Logic Configurations

Scenes are NOT timelines (that's Sequences). Scenes are **stateful configurations with node-based logic**.

```
Scene "WWTBAM"."FinalAnswer"              — activates a state
Scene "WWTBAM"."WrongAnswer"              — transitions to another state
Scene "CSGO"."PlayerAliveOrKilled"        — dynamic per-player state
Scene "CSGO"."Team1Color"                 — team color assignment
Fixture 1 At Scene "WWTBAM"."FinalAnswer"
```

Use cases:
- Game shows: team management, color assignments, booth configurations
- Esports: per-player state tracking, team colors, alive/killed states
- Abstract programming: load/unload preset pools based on conditions
- Node-based logic: "If Team.Color == Red, use this preset"

Scenes live in `DataPool.Scenes`. They work alongside Sequences but serve a different purpose — state vs. timeline.

Future: Pool of abstract mappings with node-based editor. Load 20 teams into booths so the rig knows what's happening.

---

## 7. Output Compositing — Priority Stack

> **2026-10-10:** the full merge order is now in §27 (follows a reference console's documented order): masters
> are output *modifiers* applied after the merge, not a layer above the programmer; external
> input, DMX tester and Super playbacks are added; Highlight/Solo apply after the masters.
> The Priority pool below stays (the reference bands are its default content).
> **Freeze — decided 2026-10-10 (step 4 hardening).** The programmer merges LTP with
> playbacks (latest change wins); Freeze lifts it above every playback except Super. The
> "Freeze Toggle" text below (March) is superseded.

The output pipeline merges values in this order (top wins):

```
1. Parked values          (NOTHING can override — locked)
2. Masters                (grand, group, speed) — superseded by §27: masters are output
                          modifiers applied after the merge (grand → world → group, §22)
3. Programmer             (active user values)
4. Playbacks              (running sequences, ordered by Priority pool)
5. Default values         (from GDTF fixture definitions)
```

### Freeze Toggle

The `Freeze` command flips the Programmer BELOW Playbacks:

```
Normal:    Park > Masters > Programmer > Playbacks > Default
Frozen:    Park > Masters > Playbacks > Programmer > Default
```

This lets playbacks take back attributes from the programmer's active values.

### Priority Pool

User-defined priority levels (not hardcoded tiers):

```
DataPool.Priorities
├── Priority 1 "Background"      — lowest
├── Priority 2 "Cues"
├── Priority 3 "Specials"
├── Priority 4 "Overrides"       — highest
```

Sequences reference a Priority by ID. Users can reorder, insert, rename. Much more flexible than fixed Low/LTP/High tiers.

---

## 8. Show File Format (.luma)

A `.luma` file is a zip of the `ShowData` tree — `manifest.json` first, one JSON file per top-level
field, a folder per pool member (`core::show_file`, since format 7; the current `FORMAT_VERSION` is 10;
see `docs/user/show-files/show-files.md`). `ShowSettings` carries `format_version`,
`saved_with_commit` and `show_id` (§20, §21 "6d").

> **Superseded in part.** Since step 11b (§29) a `.luma` is a zip (`show.json` +
> `fixturetypes/<sha256>.gdtf`, format 6). Format 7 (§30 "12a2 / 12a3 hardening", step 12a2)
> makes it a folder tree with a `manifest.json` and splits media into **two tiers**: images,
> icons and GDTF files (≤ 16 MB each, ≤ 256 MB per show) are **embedded** beside their
> manifests; large show-critical media stays on **asset drives** as below — the show names
> `AssetRoots`, each station maps them to its own folders (like networks to NICs). The text
> below now describes the asset-drive tier only.

### External Media (asset drives)

Media assets (images, sound, video) are NOT stored in the .luma file. They live in user-managed folders:

- Location: local disk, USB drive, network server — user decides
- The show file stores **paths/references** only
- Missing media = warning + skip (never crash)
- Users fully control folder structure
- System clearly reports missing media items
- On show load or OS boot, the referenced drive/folder must exist

### Media Sync Protocol

Networked stations must stay in sync with required media:

1. **Master is authoritative** — master knows what files are needed for the show
2. **Sync check on join** — when a station joins, it compares its media manifest against master
3. **Auto-discovery** — missing files are located anywhere on the network (any station that has them)
4. **Sync GUI** — appears automatically when a station is out of sync, shows what's missing/downloading
5. **Operator-approved** — sync only executes when an operator explicitly approves the sync command
6. **Cache on all devices** — media gets cached out to all connected stations for redundancy

The sync flow:
```
Station joins session
  → Compares manifest with master
  → If out of sync: Sync GUI appears showing diff
  → Operator reviews what needs to sync
  → Operator executes approved sync command
  → Files pulled from master or any station that has them
  → All stations cache a copy
```

---

## 9. Autosave & Show Recovery

- Config lives in `ShowData.ShowSettings`
- **Master-initiated**: the master station decides when to save
- **Network-synchronized**: master tells all stations to save simultaneously
- Every station gets a recovery copy
- On crash recovery, offer to restore from autosave
- Autosave location: `library/shows/.autosave/` (LumaSession and signal copies)
- **Backups** (R1-16, Q37; one `backups/<show>.backup_<UTC>.show` per save): every save
  first keeps the file it replaces. *Superseded by §30 "12d part 5" at the merge of the bugfixes
  branch (§30 "Merge of bugfixes"): the `backups/` folder maps to **Versions**
  (`library/shows/.versions/<show>/<stamp>.luma`, tiered cleanup); the branch's
  `library/shows/backups/` newest-10 scheme was dropped, its hardening kept (copied by the OS,
  never over a version from the same second, a long name fits, a read error stops the save).*

---

## 10. Network Session — Multi-Station Sync

*Rewritten 2026-10-10 after research (a reference console's manual + live probe; game netcode: Unreal, Godot,
bevy_replicon, Valve; collaborative/replicated systems: Figma, Zed, Redis, Raft, Kafka).
Supersedes "remote stations replay commands" here and in `network-sessions.md` §7.*

### Master executes, results flow out

- Every show-changing command from any station's clients, inputs and hardware goes to the
  **master**, which executes it **as the sending user** (never as the master's own user) and
  pushes the **result** (state of the changed subtrees) to every station. Stations never
  re-run commands; the command text travels only as a label for logs and undo.
- Every result carries `(epoch, seq)`. Stations apply strictly in order; a gap asks the master
  for the missing range, or a full snapshot if it is no longer in the journal. The epoch
  fences old masters: anything from a lower epoch is rejected.
- Joining checks protocol version and show format version first.
- The master is authoritative for save timing.

### Where state lives

| Kind | Where | Replicated |
|---|---|---|
| Show (`ShowData`), including **`Users.{n}`: programmer, selection** | every station (full copy, read-only except via master) | yes |
| Per-user mode flags (Blind, Highlight, Freeze, Solo) | `Runtime.Users.{User}` | yes, session runtime |
| Session runtime (membership, clock, claims) | `Runtime` | yes |
| Station-local (station settings, interfaces, keystore, UI, command lines, screens, clients) | a `Station` subtree | **never** |

- The split is enforced **by type** (which subtree a field lives in), not a per-node flag.
- **Programmer and selection belong to the `User`**, not the `UserProfile` (restating §2 —
  the code currently keeps them on `UserProfile`, which is drift to fix). A profile is how a
  user works (views, screens, keymaps, settings); a user can switch profile and keep their work.
- Programmer and selection are **saved with the show**; mode flags are not (always start off).
- The same user logged in on several stations/clients shares **one** programmer, selection
  and set of modes (tracking backup).
- One programmer per user now, stored so it can become several later without migration.
- `Clear Programmer User 2 Thru 5` clears other users' programmers (Admin and Setup rights);
  the affected user is notified and can undo it.

### Where commands run

- Each `CommandDef` declares `scope: Client | Station | Master`. Client: navigation, help,
  views. Station: station-local settings. Master: everything that changes the show or a
  user's state. Unset means Master.
- Feedback: programmer and selection changes show **instantly** on the sending station and
  are corrected if the master disagrees; show-structure changes (Store, Delete, Label, Move)
  wait for the master's result.
- Concurrent editing uses the existing **Claim** system (§4): opening an editor auto-claims
  the object, others see it read-only; claiming a claimed object asks the owner, who gets a
  10 s countdown; Admins can force it. Plain commands still apply in master order.

### Losing the master

- **Under the timeout (5 s):** the operator keeps working. Programmer changes stay instant;
  show-changing commands go into an ordered, reliable outbox, are marked *pending* in the UI
  and are resent until the master confirms. The rig reacts once the master has them.
- **After the timeout:** the station becomes **IdleMaster** — its own master, fully editable,
  with the pending outbox applied locally and everything since the split journaled. Unlike
  established consoles, network output stays **off** unless "output when alone" is on; local hardware ports
  hold their last frame. The status bar shows a clear "disconnected from <session>".
  Meanwhile the rest of the session elects a new master.
- **Reconnection:** known stations are re-invited automatically, with back-off (stop after 5
  failures per minute). Not diverged → silent fast-forward. Diverged → the merge
  prompt below. Patch changes on both sides block *Merge All*.
- **Health:** the 5 s timeout is a named constant with a debug-only override (like a reference console's
  hidden `DisableTimeouts`). `Info Station` and a status icon show latency, resends and
  outbox depth; a warning appears before the timeout hits.
- Station states: `Standalone`, `Connected`, `IdleMaster`, `GlobalMaster`.
  *(Note 2026-10-11, step 11a: superseded by the roles of §21 "6c": Standalone, Master, Member, IdleMaster, Fenced.)*
- Current code detects none of this (`connected` is never cleared, no election, no epoch,
  outbox never sent) — see the 2026-10-10 review.

### Master loss and election

- Automatic: highest priority (`Never`, `VeryLow`, `Low`, `Normal`, `High`), then
  longest online time (±5 s), then lowest stable station ID; 5 s timeout. The winner bumps
  the epoch before it starts output. `Never` stations can't become master; if only those
  remain the session ends.
- Every station already holds the full show and every user's programmer, so takeover needs
  no download.
- "Output when alone" is a per-station flag, **off by default** (a reference console warns that receivers
  flap between sources when it's on).

### Merge when stations rejoin or masters meet

- Four choices: **Merge All / Keep Master / Keep Mine / Standalone** (the March
  `network-sessions.md` §9 "keep session / keep joining / merge if compatible / cancel").
  *(Names superseded by §21 "6d": Merge / Use Master / Use Mine / Cancel; drawn by the standard
  dialog since §30 "12b hardening".)*
- Done better than established consoles: objects are matched by **GUID, not slot** (so moved and deleted
  objects merge correctly); **every losing side is auto-saved first**; a show UUID plus
  lineage tells "just behind" (fast-forward, no prompt) from "diverged" (prompt).
- Default mode and timeout live in `ShowSettings` (timeout 0 = apply the default at once,
  none = wait for the operator) and need Admin rights.

---

## 11. Protocol Architecture

```
Protocols
├── Outputs       sACN, Art-Net, USB DMX, MIDI, PSN, UDP, HTTP POST
├── Inputs        MIDI, OSC, Remote triggers, PSN, UDP, HTTP GET, Timecode (LTC, MIDI TC, Art-Net TC)
└── Bidirectional OSC devices, web clients — protocols with feedback loops
```

Timecode sources are Inputs. Timecode shows (event lists triggered by TC) are in `DataPool.Timecodes`.

Remotes appear in BOTH Inputs and Outputs (they are bidirectional by nature).

### Protocol runtime lifecycle (2026-10-10)

- Protocol settings live in `ShowData.Protocols`. A protocol runtime (sACN, Art-Net, OSC, …)
  **starts only when the loaded show has an enabled line of that protocol**, and stops when the
  last one is removed or the show is unloaded. Nothing protocol-related starts at boot.
- In a network session **only the master transmits output**.
- There is **one** renderer; output protocol runtimes attach drivers to it and never build their
  own renderer from a startup snapshot.
- An Input line (e.g. OSC) names a **user**; commands it receives run as that user, with that
  user's rights.

### Output routing (2026-10-10, step 4 hardening)

Researched against the repo docs (March: per-universe config, `fixtures-patch-output.md`,
`transport-types.md`) and a reference console (DMX Protocols rows: mode, destination, local universe +
amount, protocol universe, priority; interface per station).

- **Per universe, not protocol rows** — more powerful for syntax and a smarter UI. A universe holds
  a list of **routes** `{output line, target}`; one universe can go out on several lines
  (sACN + Art-Net, several unicast targets). Targets are DMX-only for now (sACN universe,
  Art-Net Port-Address); generalising to AddressSpace containers (`transport-types.md`)
  waits until a non-DMX transport arrives.
- An **output line** is a transport profile: protocol, mode (multicast / unicast / ArtPoll),
  target, priority, enabled. Modes reserve Input; sACN/Art-Net input is not built yet.
- **Syntax:** `Assign Universe 1 Thru 4 At sACN 10` (canonical; lum.md's
  `Universe 1 Assign sACN Universe 1` form is dropped). Each Assign adds a route (10, 11, 12,
  13), creating the protocol's line if none exists. Errors: two universes sending the same
  protocol universe on one line; two lines of one protocol sending one protocol universe on a
  network to the same destination (E-OUTPUT-003, also when `Set Output … Target|Network`
  makes them meet); several lines of that protocol and none named
  (`Assign Universe 1 At Output 2 Universe 10`). A range keeps its spacing on the wire
  (`Assign Universe 1 Thru 4 At …` without 2 → wires n, n+2, n+3).
- **Art-Net** is stored as one 15-bit Port-Address; typing accepts `ArtNet 1` or
  `ArtNet Net 0 SubNet 0 Universe 1`, and both are shown.
- ~~Ranges~~ and ~~station interface per protocol~~: superseded by the step 4b hardening below.

**Step 4b hardening (2026-10-10, a reference console's DMX port and sACN docs + live
console)** — driven by a festival **gateway** case: take guest sACN 1–6 in on one NIC, merge it
into local universes 1–6, send them as sACN 1–6 to the rig on another NIC.

- **Networks.** The show names the networks it uses (`ShowData.Protocols.Networks`: "Rig",
  "Guest"; `Store Network "Rig"`). Each **station** maps a network to one of its NICs
  (station-local, saved in the station config file next to the startup show:
  `Set Network "Rig" Interface en1`). A show is prepped with its inputs and outputs; a
  station only sets up its NICs. A line on an **unmapped network never sends** — LumaCore
  does not start sending until a user says where (no Auto fallback, also not for a first run).
- **Lines** are one direction each (as protocol rows): `Output` lines (`ShowData.Protocols.Outputs`)
  and `Input` lines (`…Inputs`). A line = protocol (sACN, Art-Net, OSC), network, mode
  (multicast / unicast; Art-Net: ArtPoll), target, priority, enabled; input lines add the
  input priority (Priority pool band, Super … Lowest) and the user for command inputs (OSC).
- **Routes** stay on universes `{line, target}`, both directions (in-routes used from 4c).
- **Merge** (4c, modelled in 4b): `Universe.merge_mode` Off / Prio / HTP / LowTP (show);
  in Prio the input competes at its line's input priority in the §27 merge — so §27's
  "external input on top" becomes "at its input priority". A line can set the mode of all its
  universes at once.
- **Commands:** `Store Network "Rig"` · `Store Output 1 "Out to rig" /Protocol sACN /Network
  Rig` (`Store Input …`) · `Set Output 1 Mode Unicast`, `Set Output 1 Target 10.0.0.5`,
  `Set Output 1 Priority 150` · `Assign Universe 1 Thru 6 At Output 1 Universe 1` (`At sACN 1`
  when exactly one sACN output line exists; with none it creates one on Network "Default",
  which must still be mapped) · `Unassign Universe 1 At Output 1`.
- **Ranges are dropped** (`Range`, `UniverseRange`): `Universe 1 Thru 4` covers grouping.
- **No migrations** while the format is still moving: old range/line data is not converted.
- **Who sends:** a station that never joined a session sends; in a session only the master;
  an IdleMaster (lost its session) only with the station's "output when alone" (§10).
- **Art-Net:** ArtPoll every 3 s on each mapped network; unicast to nodes that report the
  universe; otherwise the network's **directed broadcast** (e.g. 10.0.0.255). Discovered
  nodes at `Root.Runtime.Network.ArtNetNodes`.
- **OSC** is an input line too (port number, network, user); no line → no OSC (nothing at
  boot, no default line).
- **sACN CID:** one per station, stable (station config file).
- **Status:** per line in List/Info Output and `Root.Runtime.Network.Lines` — sending N
  universes / network not mapped / socket error (code) / silent (not master); the status-bar
  icon per protocol ("sACN 2/3", red on error).
- **Future fixtures with a bidirectional JSON-style API:** the engine's result is per
  fixture+attribute **Value** (physical/normalized throughout, like presets); DMX patch output
  is one adapter after it. Fixtures get an empty `connection` slot (no migration later);
  `Runtime.Devices.{fixture}` is reserved for what devices report back.
- **Station maps are keyed by name** (network → NIC, AssetRoot → folder): they work across shows. The one
  exception to "links are GUIDs": each station moves its own mapping when it sees an AssetRoot or Network
  renamed by GUID, wherever the rename ran (`AppState::follow_renames` on publish; R1-69, Q41).
- **Live Patch**: without the Patch claim (Setup/Admin rights) `Set` may change Name,
  Note, Tags, Hidden, Locked, address, Pan/Tilt offset and invert (not built yet, roadmap), position,
  routes and output-line settings; creating/deleting fixtures, universes and lines, fixture type/mode
  and fixture-type edits need `Claim Patch`. Live Sets are normal undoable changes (better
  than established consoles, which clear Oops); Unclaim Patch still clears every undo stack.

**Step 4c hardening (2026-10-11, a reference console's DMX priority and port docs,
its sACN / Art-Net and DMX sheet docs + live console
`DmxUniverse`, `DMXSheetSettings`)** — DMX input, merge and the DMX window, one step.

- **One Line type.** Output and Input lines share one `Line` struct (protocol, network,
  target, enabled + direction-specific settings) and one set of handlers (Store, Delete,
  Set, view, Assign/Unassign) parameterized by `Direction` — "unify this in code so we don't
  write it twice". `Output n` / `Input n` stay two keywords with two ID pools; a setting of
  the other direction is refused with a clear error (Priority on an input, MergeMode on an
  output).
- **Commands** mirror output: `Store Input 1 "Guest" /Protocol sACN /Network Guest` ·
  `Assign Universe 1 Thru 6 At Input 1 Universe 1` · `Unassign Universe 1 At Input 1` ·
  `Set Input 1 InputPriority Super` · `Set Input 1 Target 10.0.0.9` (accept that source only).
- **Merge mode lives on the universe**: `Universe.merge_mode`, default Prio;
  `Set Universe 3 MergeMode HTP` one universe, `Set Input 1 MergeMode HTP` every universe
  routed from that line (Info shows "…" when they differ). Input priority lives on the line.
- **Merge per DMX channel**, after render: each channel knows its winning source (park,
  frozen programmer, sequence, programmer, default) and that source's stamp. Off = input
  ignored; HTP / LowTP = max / min of ours and input; Prio = the input competes at its band.
  The input stamps a channel when its value changes (LTP inside a band). Channels with no
  fixture pass input through (gateway).
- **Input bands on today's layers** (until the Priority-pool bands exist, then they slot in
  with no show change): Super above everything but Park · Prog LTP with frozen programmers,
  above playbacks · Highest/High above playbacks and programmers · LTP latest change wins
  against playbacks and programmers · Low/Lowest only where nothing of ours gives a value
  (defaults, unpatched channels).
- **Park always wins**; the grand master, blackout and Solo touch only our own output — input
  is another desk's output and passes unchanged.
- **Several sources on one input universe:** E1.31 rule — highest sACN priority wins, equal
  priority HTP per channel; Art-Net HTP between sources. The result then competes at the
  line's band. sACN preview packets are ignored; our own CID (and for Art-Net our own IPs)
  is ignored, so a loop through our output never feeds back.
- **Source loss:** E1.31 — no packet for 2.5 s = source lost; stream-terminated releases at
  once. Its channels fall back to our output; line status "source lost". Art-Net: same
  timeout.
- **Who listens:** only the station that sends (standalone / master / IdleMaster with output
  when alone) opens input sockets and joins groups; others show "silent (not master)".
  sACN multicast joins on the mapped NIC and filters packets by the arrival interface (a
  socket bound to 5568 sees groups joined on every NIC).
- **ArtPollReply:** on a mapped network with an Art-Net input line, the listening station
  answers ArtPoll listing those Port-Addresses, so guest desks in ArtPoll/unicast mode find
  it. Nothing on unmapped networks.
- **Visibility:** line status in List/Info Input ("receiving 6/6", "source lost", "not
  mapped"); seen sources (name, CID or IP, sACN priority, universes) at
  `Root.Runtime.Network.Sources` and in Info Input; `DMX In 1` reads the raw received
  universe next to `DMX 1` (merged output); status-bar icon gets an input arrow.
- **DMX window** (a DMX sheet): window type `dmx` with Universe, Address, Readout
  (Percent/Decimal/Hex), Address Mode (u.aaa / absolute), Level Bar (Off/Background/Bar),
  Columns / Auto Columns, mask Value/Attribute/ID, Only Selection, and View (Output merged /
  Input raw). Cells are marked by winner (input tinted with its line, park, ours); fixture
  spans have rounded ends and alternating shades; a status line on hover shows fixture,
  attribute, winner, our value and the input value. DMX tester bar later.
- **Window data:** WebSocket subscription per window to the universes it shows; the server
  pushes changed universes at most ~20 fps from the engine's published frame + per-channel
  source map (ArcSwap only, never the state thread); stops when the window closes.


**Menus, LumaSession and staged patch (2026-10-11, "DMX Protocols window" hardening; a reference console's menus
`setup/network/{network, s_acn, art_net, dmx_left, station_control}` + live
`DeviceConfigurations.DMXProtocols`; mockups https://claude.ai/artifact/U2kNxUADmPt5JvHJMN3y12)**
— one step, built together.

- **LumaSession** is the server-to-server side (stations, join / invite / dismiss / leave,
  discovery, replication), everywhere and **without aliases**: the object `LumaSession`
  (`Join LumaSession "Festival"`, `Join LumaSession 10.0.0.5`, `Leave LumaSession`,
  `Invite Station "FOH-2"`, `Dismiss Station "FOH-2"`, `List LumaSession`); `JoinSession` /
  `LeaveSession` go; the shell-session object becomes `Shell`; `Station` stays;
  `Root.Runtime.Network.{Discovery, Connections, Listeners, Health}` move to
  `Root.Runtime.LumaSessions`. "Network" means only a data network; the `Network` keyword
  keeps that meaning.
- **Three menus**, overlays: **LumaSessions** (a session menu), **Network** (this
  station's interfaces and services, the networks → NIC map, output when alone), **DMX
  Protocols** (lines, routing, discovery).
- **LumaSessions menu (full):** station grid from discovery (name, IP, LumaSession,
  location, role, priority, status, version); Invite / Dismiss / Join / Leave; this station's
  name, LumaSession name, location, priority, Invite on/off, LumaSession on/off.
  - Sessions are **named and found by discovery**: beacons carry station name, LumaSession
    name, location, status, priority and version. Joining by address stays for routed
    networks.
  - **Invite**: a station with Invite on (default) joins without asking and takes the
    session's show; unsaved changes are kept as an autosave copy first. Invite off → the
    master is told the station doesn't accept invites. A station that **leads** that LumaSession
    (a master with members, or alone after changing the show) never joins silently — the inviter
    was away and its show may be older: its operator gets Use Mine (default) / Use Master /
    Cancel, Merge greyed (R1-22, Q26).
  - **Dismissed or leaving** stations become standalone with a copy of the show and their
    DMX output **held off** until their operator resumes it (or turns output when alone on);
    "standalone sends" applies only to stations that never joined.
  - **LumaSession off** stops everything networked: sessions (it leaves first), discovery,
    Shell and HTTP on NICs (localhost stays), DMX protocols.
- **Network menu:** Interfaces tab — every NIC (name, address, link) with Yes/No per service:
  **LumaSession** (discovery + command port + replication), **Shell** (47654), **HTTP**
  (47656); loopback always serves Shell and HTTP; default Yes everywhere (today's behaviour);
  station-local, changing one restarts that listener. Networks tab — the show's networks →
  NIC dropdown, link, lines on it, state; output when alone.
- **DMX Protocols menu:** one tab per network (Rig, Guest, …, coloured in/out/both), each with
  sub-tabs **Lines** (outputs and inputs, settings, live state), **Routing** (matrix: local
  universes × the network's lines; clicking a cell opens an editor — protocol universe,
  range, Assign / Unassign) and **Discovery** (sACN sources and Art-Net nodes heard there).
  Several protocols on one network is normal (a network is where, a protocol is how).
- **No claim for protocols:** networks, lines, routes and merge modes are edited live without
  the Patch claim (creating/deleting lines and networks no longer needs it). The claim is only
  for entering and leaving the patch.
- **Staged patch (Patch vs Live Patch):** claiming Patch takes a **draft** of the patch
  (fixtures, universes); patch commands while claimed edit the draft and the rig keeps
  running on the live patch. Leaving (`Unclaim Patch`, leaving it in the tree, closing a menu
  that claimed it) asks **Keep / Discard / Cancel** when something changed: Keep applies the
  draft in one undo step, Discard drops it, Cancel stays. While someone holds a draft, Live
  Patch edits to fixtures by others are refused ("Patch is being edited by …").
- **Issue 11 hardening (2026-10-17): the draft holds fixtures and universes; the wiring (the
  protocol lines) stays live, keyed by slot.** The code drafted a universe's routes and merge mode
  with it (they are fields of the universe), against the "no claim for protocols" line above.
  Decided one question at a time, after probing a reference console (live + dump + manual: universes
  are 1024 fixed slots in `LivePatch.DmxUniverses`; routes are rows under the protocol,
  `DeviceConfigurations.DMXProtocols.sACN.sACNData n` / `ArtNet.Art-Net-Data n`, each with a
  GUID, `LocalUniverse` (the slot, by number), `Amount`, the protocol universe, `Mode`,
  `Priority`, `MergeMode`, `InputPriority`; `MergeMode`'s one home is `DmxUniverse.MERGEMODE`,
  the row's cell only sets its slots together, "…" when they differ; a reference console keeps port
  configuration per device):
  1. **Fixtures and universes are the draft.** While Bob holds Patch, fixtures (Store, Delete,
     Move, Label, addresses) and universes (Store, Delete, Move, Label, `"MergeMode"`) are
     edited in his draft and reach the rig on Keep (one Oops step) or vanish on Discard.
     Fixture data is the source of truth and is guarded by the draft; others' edits to either
     are refused (E-PATCH-008 naming Bob, as today).
  2. **Wiring is the protocol lines** (§34 "34f7c hardened": `Sacn 1` = one protocol row with
     `LocalUniverse` + `Amount`; the route objects first decided here were replaced by that).
     A line names its slots **by number** — a wiring point, not a link (a declared identity
     like UI key ids), so the GUID rule holds and Bob's draft never moves a line: Move Universe
     moves the universe's name, fixtures and MergeMode, the wiring stays on its slots.
  3. **Lines are live, for anyone** with the rights, Bob included (wiring is tested while
     patching; DMX ports stay outside the patch): Assign / Unassign / Set on a line take the
     normal change path and need no claim.
  4. **MergeMode stays on the universe** (its one home; how the slot's merged input meets the
     console). A draft Move Universe whose old slot is covered by input lines names them and
     asks: Move anyway (MergeMode goes with the universe, the inputs stay on the slot) / Cancel.
  5. **A universe deleted in the draft leaves the lines on its slot:** the slot has no universe
     (an output sends nothing for it, an input is ignored, the grid shows `no universe`); a new
     universe there picks the wiring up again; Keep's question names the lines left on empty
     slots. Delete Universe in the draft names the fixtures patched on it, live and draft; on
     yes they are unpatched in the draft.
  Not built yet: with 34g4's patch overhaul, on 34f7c's lines. Since §34 "34g4 patch hardened"
  (2026-10-17) the claim and the draft are `Devices`' (every device kind), the patch lives at
  `Devices.Dmx.Patch`, and live device edits follow the Live Patch columns.
---

## 12. Tags — Visual Browser (Future)

Tags are fixture metadata labels (position, function, color, etc.).

Command: `Fixture 1 Thru 100 If Tag "FOH"` — filter by tag.

Future UI:
- Node-based circle browser showing tag relationships
- Can visualize DataPool items too (e.g., select "Song1" and see all firing sequences)
- Visual exploration of the object tree

**Tags, first decisions (2026-10-17, from the 34g4 patch hardening; a reference console probed).** Built
before the patch overhaul. Being hardened, one question at a time.
- **Reference console:** a show-wide Tags pool (`ShowData.Tags`, 1000 slots). Each tag holds `TagReference`
  children, one per object it is assigned to, each with a Protect flag. An object's `TAGS` is a
  view of those references: after `Assign Tag 1 At Fixture 769` it read `"FrontPark:0"`, and
  `Assign Off Tag 1 At …` emptied it again. Any object can be tagged: fixtures, sequences, cues,
  macro lines, recipes, other tags. A tag has a TagType (None / Kill Instant / Kill Delayed),
  ForwardCommands (playback commands run on every reference), a pool tap Action, an Appearance,
  and can be locked. Syntax: `Assign Tag n At …`, `Assign Off Tag "x" At …`, `Delete Tag "x"`.
- **Decided:**
  - **Both a pool and links.** The tag definitions live in a pool, `Root.ShowData.Tags`
    (number, name, GUID, appearance, note). Every taggable object holds GUID links to its tags, so
    an exported object links up again in another show.
  - **Every object in the tree can be tagged.** The patch sidebar can group by tag, alongside MVR
    tree and MVR class (§34 "34g4 patch hardened" 12).
  - **Importing another show's data asks Tags: Bring | Leave.** A dialog that asks what to do with
    every kind of data an import brings is on the roadmap.
  - **Playback behaviours come later, with playback:** tag types (Kill), Forward Commands,
    Protect and pool tap actions.

**Tags hardened (2026-10-17).** Asked one question at a time, with two rounds of mockups. Not
built; this step comes before the 34g4 patch overhaul.
1. **Taggable:** every show object with a GUID (everything under `Root.ShowData`). The links
   live in the shared `NodeMeta`, `meta.tags: Vec<Guid>`; an object with only a bare `guid` field
   gets `NodeMeta`. Runtime and station objects can't be tagged.
2. **A tree of tags.** A tag can have child tags, with exactly one parent, at any depth (All ›
   Lx1 › Right). A tag reaches its own objects plus everything under its children, recursively:
   `Off Tag "All"` reaches everything; `Go+ Tag "Lx1"` reaches Lx1's objects and Right's.
   - Cross-cutting sets use a second tag on the object, never a second parent.
   - `Assign Tag 1.1 At Fixture 5` gives fixture 5 only Lx1. It then counts as part of All
     through the tree, but not of Right.
3. **Stored as a real tree:** `Root.ShowData.Tags.<n>.<n>…`, numbered within the parent, so a tag
   carries its path. A move renumbers; links are GUIDs, so tagged objects never notice.
   - Names are unique among siblings. `Tag "Right"` works while only one exists; owner first
     narrows it: `Tag "Lx1" Tag "Right"`.
   - A tag has a number per parent, name, GUID, appearance, note, Action and its own tags (NodeMeta).
4. **A verb on a tag** expands, through the one resolver, to its objects (deep) and runs on each
   kind that takes the verb, skipping the rest with a summary ("12 fixtures, 2 groups skipped: no
   Go+"). If no member takes the verb, it is E-CMD-009. A bare `Tag 1.1` runs the default verb,
   which selects its fixtures, deep.
5. **Lines** (sketches; lum.md below):
   - `Assign Tag "Right" At Fixture 1 Thru 12` tags; `Unassign Tag "Right" At Fixture 3` untags.
   - `Assign Tag "Right" At Tag "Lx1"` nests (with one parent this moves it);
     `Unassign Tag "Right" At Tag "Lx1"` lifts it to the top.
   - `Store Tag 1.1.3 "Centre"`, `Label`, `Delete Tag`.
   - Delete asks once, naming the children and the objects: With children / Keep children (they
     move up one level) / Cancel. Objects are never deleted, only their links. One Oops step.
6. **Tags is a live column.** Tagging fixtures works while someone holds the Devices claim: the
   change goes to the live show and into their draft, unless they changed that fixture's Tags in
   the draft (refused, naming them). Tagging any other object never needs a claim.
7. **Crates:** `crates/model/lumacore-tags` holds the tree, the reverse index ("what has tag n",
   direct and deep), the member expansion the resolver calls, and the checks (one parent, no
   cycles, dangling links listed by the reference check). `crates/commands/objects/lumacore-cmd-tags`
   holds the Tag object and its words. `NodeMeta.tags` is plain GUIDs in base, because NodeMeta
   lives in `lumacore-base`.
8. **No Tags section under Show.** Tags are given in the **one EditSetting overlay of every show
   object** (`<luma-settings>`, 12b2's tabs from the declared sections):
   - first the object's own tabs, then **Meta** (Name, Note, Colour, Icon, Locked, Hidden, from
     12e's `META` props, plus read-only GUID, tree path and "used by") and **Tags**;
   - Tags is a picker tree with + / −, search, "New tag here" and the ≡ menu, with tri-state boxes
     over several objects (`EditSetting Fixture 101 Thru 112`); each change is one Assign /
     Unassign line.
9. **The Tags pool window is the overview,** in two panes. On the left, the tree with + / −,
   colour, counts and the Action badge. On the right, the picked tag's members by kind, Direct |
   Deep, with Run action.
   - Its window settings hold the default Action and Direct | Deep, and each tag has its own Action
     that overrides the default. Today the only action is Select fixtures; Go+ / Off / Toggle are
     planned for playback.
   - A tag with children folds open. Hold opens the ≡ menu; dragging a tag re-parents it.
   - The tree can be organised both here and in EditSetting's picker.
10. **A Tags cell in every `<luma-grid>`** (chips plus a pick list that can open the EditSetting
    Tags tab).
11. **The patch sidebar's Tags mode** (with MVR tree and MVR class): the tag tree with + / − and a
    **Direct | Deep** switch. Deep shows a parent's fixtures plus its children's, marked "from
    Right". A fixture with two tags shows under both. An "Untagged" row lists fixtures with no tag.
    Dropping fixtures on a tag runs Assign.
12. **Importing another show's data** asks Tags: Bring | Leave (roadmap: the import dialog for every
    kind of data).

**The lum.md hard discussion (2026-10-17), and what it changed:**
- **Dotted ids are declared per object.** Each object reads a dotted id one of three ways:
  Decimal (Cue: `1.10` = `1.1`), Parts(n) (Master, DMX, Window), or Path (Tag: `1.10` ≠ `1.1`,
  any depth). An object that nests itself defaults to Path. The resolver reads ids only through
  the declaration, and a change is visible in the reference and the golden record.
  - Rejected: "a path for every nesting object" by shape alone. It leaves DMX, Cue and Window
    implicit, a structural change would silently re-read old lines, and the reference can't show
    it.
- **`Thru` on a path walks one parent's children.** `Tag 1.2 Thru` = 1.2, 1.3, 1.4, …; ends under
  different parents are refused (E-SYN; join them with `+`).
- **A range of tags:** verbs that reach members reach each listed tag's members; the Tag object's
  own verbs (Store, Label, Delete, Copy, Move, Assign / Unassign, EditSetting, List, Info) touch
  only the listed tags.
- **Depth comes from `/R`, revising points 2 and 4.** A member verb reaches the tag's own objects
  (the current level) by default; `/R` adds every tag under it. This holds for every object and
  command that reaches into a tree (the existing `/r` option, "what is below too").
  - At reaches every kind that takes it, at the current level unless `/R` is given.
  - The pool window's and the sidebar's Direct | Deep are `/R` on the lines they run.
  - Membership still follows the tree (fixture 5 tagged Lx1 is part of All); `/R` decides how
    far a line reaches.
- **`Assign Tag 2 At Tag 1` tags tag 1 with tag 2, revising point 5.** Assign always adds a tag.
  Nesting is the door's `Move Tag 2 At Tag 1` (with its subtree, renumbered there); moving to the
  top level is a Move too.
lum.md updated: the Targets rule for dotted ids, `Tag` in the object list with a "Tags"
paragraph, and `/R` under Options. The examples are a `text` block, which becomes a tested `lum`
block when tags are built (PENDING only shrinks).

Mockups: `docs/dev/ui-kit/mockups/2026-10-17-tags/`; round 2 **locked** 2026-10-17 ("Go").

**Tags part 1 built (2026-10-17): the data.**
- `crates/model/lumacore-tags` holds the tree:
  - types: `Tag` (NodeMeta, `action`, `children`), `TagAction` (WindowDefault | SelectFixtures),
    `TagPath` (parts, never a decimal; leading zeros refused) and `TagTree`;
  - operations: `find`, `level`, `all`, `path_of`, `named` / `named_one` (unique among siblings,
    E-TAG-006 when ambiguous), `siblings` (Thru on one parent, typed order, E-TAG-005 across
    parents), `insert`, `remove`, `move_under` (GUID kept, next free number, E-TAG-002 into
    itself), `lift_children` (a taken name gets " 2"), `subtree`, and `members` (read from the
    identity walk's `Tag` links);
  - the E-TAG-001…007 catalog.
- `NodeMeta.tags: Vec<Uuid>` (saved only when non-empty) is walked as links of kind `Tag`, so the
  reference check covers them; `SequenceObject` now walks its meta's links too.
- `ShowData.tags` is saved as `Tags.json`, with `ShowSubtree::Tags` (undone, replicated, Oops
  category Show).
- Show format 28, SESSION 19; the demo regenerated, and the library's format-27 shows backed up
  first.
- Not yet: the objects that carry only a bare `guid` (users, views, screens, UI keys, station rows,
  …) get their NodeMeta in part 1b.

**Tags part 2 built (2026-10-17): every object declares how it reads a dotted id.**
- `lumacore_registry::object::IdReading` (None, Whole, Decimal, Parts(n), Path) sits on
  `LumShape.reading`, with constructors `LumShape::ids(n)` (Whole or Parts(n)), `decimal()` and
  `path()` (`id_parts` = `u8::MAX`, any depth).
- Cue is Decimal; Master, Preset, ViewButton and Window are Parts(2). The reference shows
  `ids decimal (1.10 = 1.1)` / `ids 2 parts` on them.
- The catalog test `an_object_that_nests_itself_reads_a_path` holds the default.
- The resolver's Path members come with the Tag object (part 3), which declares Path.

**Tags part 3a built (2026-10-17): the Tag object and its words.**
- `crates/commands/objects/lumacore-cmd-tags` declares `Tag` (reading Path, owned by another Tag,
  default verb SelectFixtures) with these verbs:
  - `Store Tag [<path> | <owner> Tag "name"] ["name"]`;
  - `Label Tag` (unique among siblings);
  - `Delete Tag [/WithChildren | /KeepChildren]`, with one question naming the child tags and
    what carries the tag; objects only lose their links;
  - `Move Tag … At Tag <path> | Root | <n>` (nests with the GUID kept, or renumbers);
  - `Assign` / `Unassign Tag … At <objects>`, any objects with a NodeMeta, a tag included;
  - `List Tag [<path>]`, `Info Tag [/R]`;
  - the bare `Tag 1.1 [/R]`, which selects the tag's fixtures by the selection rule.
- The tree node `Root.ShowData.Tags` (Node::Tags, the top tags).
- The one resolver walks a Path object through a `PathDef` catalog row that the object's crate
  adds:
  - numbers are paths below the owner, names are looked for among one parent's children
    (anywhere without an owner; E-TAG-006 when ambiguous), and Thru walks one parent's children
    (E-TAG-005 across parents);
  - `resolve::path_slot` is Store's slot rule for such objects.
- `ObjectDef::new(decl)` and its const builders (`.verbs`, `.tree`, `.delete_ask`, `.complete`,
  `.props`, `.pool`) are the door for a new object, so the copy ratchet stays at 42.
- Tag links are edited through the show's JSON (every `meta` with the GUID), then the show is read
  back and its unsaved pools rebuilt. The roadmap has a typed walk instead.
- Not yet (3b): the member verbs (Off, On, Go+, … on what a tag carries, with `/R`); Tags as a
  live column under a patch claim. Today tagging a fixture while someone else holds Patch is
  refused like any patch edit.

**Tags part 3b built (2026-10-17): verbs on a tag reach what it carries.**
- `On`, `Off`, `Toggle`, `Go`, `Goto`, `Pause` and `Tag … At <value>` run on the objects tagged
  with the tag (the current level); `/R` adds those of every tag under it.
- Each kind that takes the verb runs its own handler: a direct call, inside the kind's DataPool
  (`AppState::in_datapool`, the shared door tidy names, now built) and gated by the kind's own
  rights. The rest are skipped with a summary; when nothing takes the verb it is E-CMD-009. One
  Tag line is one statement, so one Oops step.
- lum.md's tag examples are now a tested `lum` block, and the syntax crate's test vocabulary has
  Tag.
- **Moved to 34g4:** Tags as a live column under a claim. It belongs to the Devices claim
  (§34 "34g4 patch hardened" 8). Today's Patch draft swaps the live patch out, so a live column
  there would be thrown away. Until then, tagging a fixture while someone else holds Patch is
  refused like any patch edit (E-PATCH-008).

**Tags part 4a built (2026-10-17): EditSetting's Meta and Tags tabs.**
- `EditSetting` of any show object whose row has a `meta.guid` gets a **Meta** tab after its
  own tabs: the NodeMeta fields (Name, Paint, Note, …) plus read-only facts (GUID, Path, Used
  by). A UserProfile keeps its META fields in its own sections and still gets the facts.
- A **Tags** tab comes from a `SettingsTab` row (`lumacore-cmd-tags`): one field with
  `FieldInput::Tags` (the tree as `TagNode { path, name, colour, state, count }` plus four line
  templates). The page draws it with the kit's new `<luma-tree>` in pick mode. A tick is
  `Assign Tag <path> At <object>`, an untick `Unassign …`, and New tag here is `Store Tag …`
  (under the highlighted tag or at the top). The tree keeps its folds across fresh fields.
  `CLIENT_PROTOCOL` 19.
- `<luma-tree>` is in the Showroom (browse and pick). Browse mode, with the selected row's ≡
  menu, is for the pool window (4b).
- **Moved to 34g4:** the Tags cell in grids goes with the patch window, the first grid that
  needs it.

**Tags part 4b built (2026-10-17): the Tags pool window.**
- Window type 10 `tags` (Pools), drawn by the kit's `<luma-tags-pool>` from the `tags` feed
  (`lumacore_views::tags::view`). It has two panes, as in the locked mockup. Left is the tree
  (`<luma-tree>` browse) with colours, counts and the Action badge. Right are the picked tag's
  members in a read-only `<luma-grid>` (No, Kind, Name, Tagged = direct or "from Right"), with
  the kinds counted.
- Its window settings are `Tag` (a Link by GUID), `Members` Direct | Deep, `Default action`
  (Select fixtures | None) and `Counts`. The bar holds the last three. A tap on a tag is
  `Set Self "Tag" 1.1`. Direct | Deep is `Set Self "Members" …`. Run action runs the tag's own
  Action, or else the window's default (`Tag 1.1 [/R]`). The picked tag's ≡ menu has Settings,
  Label, New child tag, Move to the top and Delete, each the station's line. Folds and the search
  stay in the tree, like a scroll position.
- **A tag has its own settings** (`EditSetting Tag 1.1`): `Action` (Default | Select) on its own
  tab, plus Name, Note and Paint on Meta, Tags and the facts. They are keyed properties.
  - A Path object's row is its GUID, never its last number, because a move renumbers it
    (`set::reads_a_path`).
  - EditSetting names the row as typed.
  - A sibling's name is refused (E-TAG-003), the same check as Label.
- The identity walk records each object's NodeMeta name (`ObjectEntry.name`, from
  `NodeMeta::walk_links`), so a view names members of any kind without per-kind code.
- A window link to a kind without its own lookup resolves through the one resolver. A tag pick
  list answers with the item's GUID (`Pick.by_guid`), because a tag's number is a path.
  `CLIENT_PROTOCOL` 20.
- Not built yet: a kind filter on the members pane (the mockup's "All kinds ▾"), dragging a tag
  to re-parent it, Expand all, and Go+ / Off as actions (they come with playback).

**Tags part 1b hardened (2026-10-17): NodeMeta in levels, on every node.**
Part 1b grew from "NodeMeta for the objects with a bare guid" into the meta model for the whole
tree, so later areas (MVR devices, 3D, plugins) add a level instead of another shape.

1. **Two levels.** Every node under Root has an **Identity** block: `guid` and `name`. Every
   ShowData object also has an **Annotation** block: `note`, `paint`, `icon`, `locked`, `hidden`
   and `tags`.
   - Tags exist only in Annotation. A tag is a saved link, and Runtime is never saved.
   - Everything in ShowData gets the full meta, its parts included: a cue part, a macro line, an
     effect-graph node, a station row.
2. **Building blocks, one flat `meta`.**
   - `Identity { guid, name }` and `Annotation { note, paint, icon, locked, hidden, tags }`.
   - A ShowData object holds `meta: NodeMeta`, where `NodeMeta` is `{ identity, annotation }`
     with both blocks flattened in serde. The file keeps one flat `"meta": { guid, name, note,
     tags, … }`.
   - Runtime, StationSettings and Resources nodes hold `meta: Identity`.
   - A future level is one more block in a new composite (for example a device block for MVR).
   - One trait, `Meta`, reads guid and name from any level.
3. **The name is the display name, optional.** A name that is an identity stays a required field
   of its own: `User.login`, `UiKey.id`, an Attribute's GDTF name, a known station's id.
   - A node is shown by its display name, falling back to its key.
   - Renaming the display name never breaks a login, a GDTF match or a key binding.
4. **No number in the meta.** `NodeMeta.id` goes: the pool's key is the number, its one home.
   - A move or renumber changes only the key.
   - An object in a list (a macro line, a cue part) has its place as its number.
5. **Properties come from the level.** Each block declares its properties once: Identity has
   Name; Annotation has Note, Paint, Icon, Locked and Hidden.
   - An object only says which meta it has (`Props.meta: Identity | Node`). Set, EditSetting's
     Meta tab, the grid columns, the Lum words and the reference read the level's list.
   - The per-object `META` lists go away.
6. **GUIDs outside ShowData.**
   - **Derived** where the node is stable: `Guid::named(kind, key)` for code-built and
     hardware-keyed nodes (window types, built-in UI keys, interfaces, stations). These are the
     same on every station and every start, so a show may link to them.
   - **Fresh** where the station makes the node at run time (a login, a Shell, a claim). It lives
     as long as the node.
   - **Kept** in the station files for station-local saved nodes (screens).
7. **Enforced by the type system and one test.**
   - `Identified` is implemented only through `Meta`: a type gives `meta()`, and its GUID and name
     come from there.
   - `guid_identified!` and `identified!` are removed, so a show type without meta doesn't
     compile.
   - A catalog test walks the demo show. It fails on a ShowData object whose meta is only an
     Identity, and on a Resources node whose GUID isn't derived.
8. **One part.** Built in one commit: the blocks, the trait, properties by level, every ShowData
   type on NodeMeta, Identity on Root's other branches, and the test.
   - Show format 29, with the library's shows backed up first and the demo regenerated.
   - The station files move forward too (no migration).

**Tags part 1b built (2026-10-17): NodeMeta in levels, on every node.**
- The blocks and their trait:
  - `lumacore_base::{Identity, Annotation, NodeMeta, Meta}`.
  - `NodeMeta` is Identity + Annotation, flattened.
  - An `Identity`'s default is a fresh GUID, so a derived `Default` never shares one.
  - Annotation always writes its fields, because a declared property writes into the object's
    JSON by field. Only an empty tag list is left out.
- `Identified` is `meta() -> &NodeMeta` plus a kind; `guid_identified!` and `identified!` are
  gone.
  - About 36 types moved from `guid: Guid` (and `name`, `no`, `id`) onto `meta: NodeMeta`. Among
    them: users, views, screen configs, screen contents, window presets, UI keys, station rows,
    known stations, asset roots, assets, images, cues, cue parts, timing and curve presets,
    executors, effects and their graph, timecode tracks, clips, regions and markers, attributes,
    feature groups, activation groups, rules, encoder bars, macro lines, FID namespaces,
    endpoints, spatial objects and priority orders.
  - The walk names every object from its meta (`ObjectEntry.name`).
- Identity keys stayed as their own fields:
  - `User.login` (renamed from `name`; folder labels and Import Show's same-name rule read it);
  - `UiKey.id`;
  - an Attribute's and an ActivationGroup's GDTF `name`;
  - an AssetRoot's `name` (each station maps the root by it);
  - a known station's `station_id`.
- `NodeMeta.id` and the duplicate `no` / `id` fields are gone; the pool key is the number.
  - The door's `renumber` hook went with them.
  - A revision's number is its place in the list.
  - Undo and replication snapshot a DataPool, Profile or User entry as `[slot, object]`, so a
    restore puts it back in its slot. This is `SESSION_PROTOCOL` 20.
- Properties come from the level: `Props.meta: MetaLevel` (None | Identity | Node).
  `IDENTITY_PROPS` (Name) and `ANNOTATION_PROPS` (Note, Locked, Hidden, Paint, Icon) are
  declared once.
  - They sit at the lowest rights level, so the object's own Set level decides.
  - The per-object `META` copies are gone: groups, fixtures, universes, sequences, lines,
    networks and presets now read the level, and FixtureType gained it.
  - Paint is `meta.paint`.
- Outside ShowData:
  - window types and core UI keys have derived GUIDs (`Identity::named`);
  - runtime users (their session id) and claims have fresh ones;
  - live screens keep theirs, with their display name, in the station files;
  - devices have fresh ones.
- Two catalog tests enforce it:
  - `every_object_with_a_node_meta_has_its_level`: a ShowData object with a NodeMeta has its
    level's properties.
  - `every_resource_has_a_derived_guid`: every Resources node's GUID is derived from its key.
- Show format 29. The library's format-28 shows were backed up to
  `.autosave/*_before-format29_*` and the demo regenerated.
- Found on the way: the `station` feed rebuilt every second, because `started_ms` was recomputed
  from the uptime and wobbled by a millisecond. It is now computed once. The feed's rebuilds had
  been resetting a theme preview.

---

## 13. Playbacks

> **Decided 2026-10-10 (step 4 hardening):** playback state is **saved** — it lives in
> `ShowData.Playbacks`, one flexible subtree per engine kind (`Playbacks.Sequences`, later
> `Playbacks.Effects`, …): current cue, started by, fader, rate, paused. As on established consoles, where the
> Sequence carries `CURRENTCUE` and `USER` in ShowData. Replicated and journaled, **not**
> undoable (Oops never undoes a Go). The loaded/next cue stays in Runtime. On show load every
> running sequence **resumes snapped** at its saved cue (tracked values asserted, no fades).

Playbacks live in `ShowData.Playbacks` (saved with show file).

- On show load, playback state is recalled to where it was
- Tracks what's running, what's paused, what's killed
- References Sequences by ID + Priority pool assignment

---

## 14. Commands Decided

| Command | Description | Phase |
|---------|-------------|-------|
| `Label` | Name any object: `Label Fixture 1 "Front Wash"` | 2 |
| `Set` | Set property: `Set Fixture 1 Name "Front Wash"` | 2 |
| `Pawtern` | Distribution tool (replaces `Distribute`, see §3) | 2 |
| `Scene` | Activate stateful logic configuration | 3+ |
| `Freeze` | Mode object: programmer above playbacks (see §16) | 2 |
| `Count` | Show child count for any object | 1 |

### Abbreviation System

All commands accept shortest-unambiguous abbreviations:
- `Pa` = Patch, `Fix` = Fixture, `Li` = List, `Inf` = Info, etc.

### Context-Aware `cd ..`

`cd ..` goes to the **parent** context, not always root:
- Patch → ShowData → Root
- Universe → Patch → ShowData → Root

Navigation becomes a stack, not a flat switch.

---

## 15. One Registry, One Namespace (2026-10-10)

- `CommandDef` (verbs) and `ObjectDef` (objects) stay **separate structs** but share **one
  namespace**: every keyword, alias and abbreviation is unique across both, enforced by a test.
- `CommandDef` carries its **handler**, argument signature, undo/replication info and help
  family. Dispatch is a registry lookup — no string `match` on verbs.
- **No rogue lists.** Dispatch, completer, hint, `List`, `Info`, `language.rs` families, undo
  subtrees, TUI hotkeys/abbreviations and help all read the registry. `legacy_help.rs` goes.
- A **tidy test** fails on any keyword string literal outside the defs, and on any hotkey,
  UiKey or abbreviation whose command line targets an unknown command.
- A command may be published with `status: Planned`: help and completion mark it, running it
  replies "not yet implemented". Half-working commands are still forbidden.
- **As few keywords as possible.** When a command and an object are one concept, it is one
  word: an object that can be called. `watch clock` calls the Watch object (starts a watch),
  `List watch` lists running watches — no separate `Watches` keyword. Objects are no longer
  also registered as commands; the resolver looks a word up in both registries.
- Short forms are registry aliases (the TUI's expand-on-space reads them); there is no
  separate abbreviation table. A typed prefix of a name also resolves (`registry::resolve`, like
  e.g. `Fixtu 2`, `Fi 5`; ties go to the word whose name is a prefix of the others'), and a stored
  line (keys, later macros) is saved with the full words (`registry::full_words`) so a new
  registry word can't change it (R1 Q11, D7). As on established consoles, every unquoted word that is a
  unique prefix of a registry word is expanded — a name too (probe: `Lab Gro 9998 Sto` →
  `Label Group 9998 Store`); quoted text stays. So a name in a key line that is also a keyword
  prefix must be quoted (`Label Group 3 "Sto"`) (R1 review 11, issue 07).
- Settled clashes (2026-10-10): `u` = Update (also the PRG-mode `u` hotkey);
  `type` = FixtureType; `history` = command history (the undo list is Oops/Undo);
  `shortcuts` = the KBD/PRG/NAV mode switch.

### Quoting (2026-10-13, issue 07, console probe — built)

- **A quoted word is always a value; a bare `/word` is always an option.** The lexer's `quoted`
  mark goes through dispatch: handlers get a quoted word with its quotes (`"Wash Test"`,
  `"/nc"`, `"All"`) and take them off when they store it (`tokens::unquote`). So `Label Group 3
  "/nc"` names it `/nc`, `Group "All"` is a group named All (never the keyword), and every
  `starts_with('/')` option check leaves quoted text alone.
- **Names:** one word may go unquoted (keywords win over unquoted names); a name with spaces
  must be quoted (`Label Group 3 Wash Front` → E-CMD-006).
- **`;` inside quotes doesn't split** the line (one statement, the text as typed; Label drops
  it like established consoles, it is no name character). A later `Lua "…"` needs `"…"` and `'…'` nested.
- **Filenames** a line names are relative to their library folder (`drive::library_file`):
  `Import "x.gdtf"` → `library/gdtf/`, `Export Fixture 1 "x.json"` → `library/export/`,
  `Import Image "a.png"` → `library/media/`, `Import|Export UserProfile "x"` →
  `library/userprofiles/`; sub-folders are allowed (`"test/hej.json"`, created on export); an
  absolute path, a drive letter or `..` is refused (E-CMD-008), and a bare `/word` is an option,
  never a file (E-CMD-006). Show names stay plain names in `library/shows/`. Elsewhere on the
  disk: the tree shell (Drives). Code that builds lines or imports by itself (the demo builder)
  calls the function, not a line.
- Variables (`SetUserVar`, `$name`; unquoted `$name` expands, quoted is text) and a syntax
  hardening pass: roadmap, after the step 12 UI work.

## 16. Mode Objects — Blind, Highlight, Freeze, Solo (2026-10-10)

- Modes are **objects** (`ObjectDef`) with the verbs `On`, `Off`, `Toggle`. Grammar is
  verb-first like everything else: `On Blind`, `Off Highlight`, `Toggle Freeze`. A bare
  `Blind` toggles (its default verb, like pressing the key). No postfix `Blind Off`.
- Mode state is **per user** (`Runtime.Users.{User}`), not saved in the show (see §10).
- Blind: keyword + state + status/feedback now; suppressing programmer output comes later.
- Bare `On` / `Off` (no object) act on **the current selection** — fixtures today; selected
  groups, sequences and other objects later.

**Step 3 hardening (2026-10-10, a reference console's manual + live console)** — built in step 3
(`terminal/objects/modes.rs`, `lumacore_runtime::Modes`, `control::knock_in/knock_out`):

- Modes in step 3: **Blind, Highlight, Freeze, Solo** (Lowlight not now). `Black` is **not** a
  mode and goes away as a command later: blackout is the **grand master** switched off (below).
- Verb-first only. The postfix `Blind On` is an error that suggests `On Blind`.
- Step 3 builds the objects, per-user state, status display and key LEDs only. The **effects**
  (Blind suppresses the programmer, Freeze, Highlight/Solo following the user's **live
  selection**) are built in the playback engine (step 4, §27 merge order). The old
  Highlight/Solo overlay keeps working unchanged until then.
- Per-user runtime state moves from `Runtime.Sessions.{User}` to **`Runtime.Users.{User}`**
  (§18: "session" means the network mesh); modes live at `Runtime.Users.{User}.Modes`.
- `On` / `Off` on fixtures follow knock in / knock out:
  - `On` / `On Fixture 2` **activates** the selection's (or named fixtures') programmer values.
    In step 3 it only activates values already in the programmer; pulling values in from
    cues/presets (KnockIn) comes with the playback engine.
  - `Off` / `Off Fixture 2` **knocks out**: removes those fixtures' values from the programmer.
    Bare `Off` also deselects (`Off Selection`).
  - Today's dimmer zero/restore behaviour of `On`/`Off` is removed.
  - Later: `Off Sequence 1`, `Off Executor …` stop; `Toggle` uses the object's on/off state.
- **Masters (decided now, built later):** Established consoles have Highlight, Lowlight, Solo and Blind as
  grand masters (pool 2, Toggle mode). LumaCore keeps the modes per user **and** gives each
  user mode-masters that switch on like console masters (Toggle / AlwaysOn / Disabled) and set
  the effect's strength. The grand master (`Master 2.1`) is global, applied after the merge
  *(Note 2026-10-11, step 11a: the grand master is `Master 1` in code and §27.)*
  and before DMX. **Blackout = grand master in Toggle mode switched Off** (fader level kept).

## 17. Keys and Keymaps (2026-10-10)

- Layers: **keyboard shortcut → UiKey → command lines**. Wings, web UI and keyboard
  all trigger the same UiKeys (core / plugin / user).
- A user's keyboard shortcuts and user UiKeys live in the **user profile in ShowData**.
- A UiKey holds a **full command line per gesture** (a key works like firing a macro).
  Gestures: `Press`, `Release`, `Hold`, `ReleaseAfterHold`, `DoublePress`.
- Modifier key: **Meow** — Meow+key runs a different command line.
- Key state LEDs: a UiKey shows live state from mode objects (Blind lit when on).
- Command lines are parsed when saved and re-checked on show load (unknown or planned
  commands are flagged in the hotkey editor). The editor lists only hotkey-able commands.

**Step 3 hardening (2026-10-10)** — built in step 3 (`lumacore_tree::ui_keys`,
`terminal/keys.rs`, TUI `KeyCache`, web `data-key` gestures):

- **One key system.** A keyboard binding names only a UiKey id; command lines live only in
  UiKeys. `Bind F5 "List Fixture"` makes a user UiKey (`user.…`) and binds F5 to it. Bindings
  take effect immediately (no more "active after restart"). The old `KeyBinding` key→command
  map (vanilla / debug / user layers) becomes key→UiKey.
- **Meow** is a held key, configurable per profile (default Option/Alt). On-screen and web
  keys get a latching Meow button. Meow+key runs the UiKey's Meow command lines (a second
  set of gestures).
- **Gestures:** all five (`Press`, `Release`, `Hold`, `ReleaseAfterHold`, `DoublePress`) in the
  model and the app/web clients. The TUI can't see key release on most terminals: it fires
  `Press` (and `DoublePress` by timing) and says the other gestures need the app or web UI.
- **LEDs** read the per-user mode state (`Runtime.Users.{User}.Modes`), so each user sees
  their own Blind/Highlight/Freeze/Solo lit.

## 18. Naming (2026-10-10)

- **Session** means the network mesh only (stations joined together).
- **Station** is a running server that joins a session; **Client** is one attached device or
  window (CLI, app window, browser tab) — as in `docs/dev/networking/network-sessions.md`.
- `ShellSession` → **CommandLine**: one command line's input, history and
  path, owned by a user; several clients can attach to and mirror the same one.
- `UserSession` / `Runtime.Sessions.{User}` → **`Runtime.Users.{User}`** (who is logged in
  now, with their command lines and screens). Done in step 3: `RuntimeState::users`,
  `RuntimeUser`, tree `Root.Runtime.Users.{no}.Modes|ViewSessions`.
- `ViewSession` → **Screen** (`Runtime.Users.{User}.Screens.{n}`).

## 19. Network Security (2026-10-10)

- Every request on UDP command, TCP shell and WebSocket carries a **login token**; the server
  checks the user's rights. Stations get a station token when they join a session.
- **Password hashes never go over the wire** (snapshots, replication).
- Admin stays passwordless for now — lighting networks are mostly isolated. External access,
  web origin checks and CORS are decided together later.
- `hissme` stays as is.

### Step 5 hardening (2026-10-11)

LumaCore lives mostly on isolated, open lighting networks: security is about **who did what and
with which rights**, not about keeping attackers out. This replaces the token-on-every-request
and no-hashes-on-the-wire points above.

- **UDP command port stays open**: requests name their user, no token. Health, snapshots,
  operations and Shutdown need no login.
- **Station to station**: trusted by source IP (no station token); members keep the show's
  password hashes so they can check logins themselves.
- **One rights gate**: the `Rights` check (Guest / Programmer / Admin / Internal) runs in
  dispatch for every transport — Shell, UDP, HTTP, OSC, keys. OSC runs as its input line's
  `User` (Admin when none is set).
- **Remote Login holds**: a client-reported address (`client_addr` in the shell hello) is
  believed only when the TCP peer is loopback (the HTTP shell bridge); otherwise the real peer
  counts. In-session `Login` checks Remote Login too.
- **Passwords are masked at the source**: a command line is masked once before it reaches the
  journal, sysmon, replication ops or history; quotes are kept.
- **HTTP stays open** as today (public reads, CORS); `POST /api/logout` really ends the token
  (revoked until it expires).

**Built (2026-10-11, step 5):**
- `terminal/rights.rs` is the gate; `dispatch_single` calls `rights::check` for every statement
  (E-RIGHTS-001). `remote/auth.rs` is gone; the command port only turns a refusal into a failed
  reply. `Help <word>` shows "Needs:". Session and look-around words a Guest needs are Guest:
  Login, Logout, Exit, Disconnect, WhoAmI, cd, ls, Root, Tree, Cat, Head, Tail, Grep, Find, Wc,
  Du, Stat, Dump, Env, History, Shortcuts, Diff, Menu, ReloadUI, ListClaimed, Unwatch.
- OSC lines with no User run as Admin; a named user that doesn't exist still doesn't listen.
  *(Superseded by §30 "12d": a line with no user is refused and doesn't start; who the user
  may be is §31 "OSC line users".)*
- Remote Login: `remote::shell::bridged_client` believes `client_addr` only from a loopback peer;
  token hellos are checked too; Shell clients carry `remote` (`AppState::remote_client` while
  their command runs) so in-session `Login` refuses (E-RIGHTS-002, `passwords::may_log_in_from`).
- Masking: `passwords::mask_secrets` replaces only the secret's span (quotes kept, every `;`
  statement); `dispatch` masks once for journal, undo description, replication op and sysmon
  change reports; the command port masks its envelope, events and sysmon lines; OSC log lines too.
- Logout: `LocalSettings.revoked_tokens` (signature → expiry, pruned, saved; kept by CleanStart),
  published in `StationView` and checked by `verify_token_with`; `ConsoleBridge::logout`
  republishes the station view at once.

## 20. Show File Versioning (2026-10-10)

- Shows carry `format_version: u32` plus the **commit hash** of the build that saved them.
  Loading runs migration steps per version; new fields always get `#[serde(default)]`.
  *Step 6 hardening (§21): no migrations for now — only forward; newer formats are refused.*
- **Two decided load-time rewrites** (R1, not migrations — they follow the build, not the
  format, and say what they did in the System Monitor): a fixture whose type's footprint
  changed takes the new footprint where the slots are free, else stays as it was and is
  listed (Q20, `patch::check::follow_footprints`); windows of a type this build doesn't know
  are dropped with a note, on a show load and on `Import UserProfile` (Q32,
  `UserProfile::drop_unknown_windows`).

## 21. Undo / Replication Coverage (2026-10-10)

- The subtree list is **generated from ShowData's fields** with an exhaustive match, so adding
  a field fails to compile until undo and replication handle it.

### Step 6 hardening (2026-10-11)

Step 6 is split: **6a** groundwork (this section), **6b** the link (master executes, results
pushed in `(epoch, seq)` order, TCP station stream, Runtime replication), **6c** heartbeat,
IdleMaster, election, epoch fencing, **6d** merge on rejoin. Each is hardened, built,
live-tested and committed on its own.

Across step 6:
- **Undo lives on the master and is replicated.** The master files the frame on the sending
  user's stack; stacks travel with the user's runtime state, so a newly elected master can
  still undo. `Oops` from any station goes to the master like any command.
- **Undo is not saved** in the show (session-only). Stacks live in
  `Runtime.Users.{User}.Undo` — Runtime is "replicated, never saved", so the type puts them in
  the right place.
- **Station states**: `Standalone`, `Master`, `Member`, `IdleMaster` (the 4d menu words plus
  IdleMaster for a member that lost its master). These replace §10's
  `Connected`/`GlobalMaster` and the code's `LocalMaster`.

6a:
- **One subtree per ShowData field**, pools by id (`DataPool n`, `User n`, `Profile n`, …),
  generated from the fields with an exhaustive destructure: a new field fails to compile until
  it is listed. Everything is undoable except `Playbacks` (Oops never undoes a Go). Creating
  and deleting users and profiles becomes undoable and replicated (a missing id = absent).
- **Programmer and selection move to `User`** (§10), keyed by user number in undo and
  replication; the profile keeps views and settings. Old shows: programmers **start empty**
  (no conversion).
- **Versioning only looks forward** (§20): shows carry `format_version` and the commit (from
  git at build time, `-dirty` for uncommitted builds) that saved them. **No migrations and no
  backward compatibility** for now: the existing load fix-ups (old users, old key bindings,
  old cue layout) are removed; "a show always has an Admin" stays as a rule. A file with a
  **newer** `format_version` than the build knows is **refused** (naming the version and
  commit that saved it); others load as written.
- `Clear Programmer User …` files the undo frame on the **affected** user's stack (§10).
- Stale docs are fixed in 6a (network-sessions, session-engine, multi-station, runtime,
  object-tree, objects, control, crate-map, engines) to match §10/§18/4d.

**6a built (2026-10-11):**
- `ShowSubtree` (lumacore-core `undo.rs`) has one variant per ShowData field; `change::subtrees`
  destructures `ShowData` exhaustively; `oops::{write_subtree, restore_subtree}` match every
  variant (a missing pool entry = `null` = removed). `User(n)` is the user without its
  programmer (`UserIdentity`, also an exhaustive destructure); `Programmer(n)` is
  `users[n].programmer`; users restore before programmers.
- `User.programmer` (saved); `UserProfile` lost `programmer` and `undo_stack`. Accessors:
  `ShowData::{user_no, programmer_of, programmer_of_mut}`, `AppState::current_programmer`.
  The playback engine's layers are per user (`UserLayer.user_no`, `Source::Programmer(user)`).
  `store.rs` now stores the acting user's selection (the old FIXME is gone).
- Undo: `RuntimeUser.undo`; `AppState::{undo_stack, undo_stack_mut}`; tree
  `Root.Runtime.Users.{n}.Undo` (Runtime.Users is numbered like ShowData.Users).
  `Oops`/`Redo` are `Untracked` (they change the show and replicate, no frame of their own).
  `Clear` is tracked, and `Clear Programmer User` files each cleared programmer's step on its
  user (`AppState::undo_to_owners`).
- Versioning: `ShowSettings.{format_version, saved_with_commit}`, `lumacore_tree::FORMAT_VERSION
  = 1`; `crates/lumacore-station/build.rs` sets `LUMACORE_GIT_SHA` (`-dirty`); every file load goes
  through `app_state::parse_show` (E-SHOW-001 for a newer format). Removed: `ensure_users`,
  `migrate_keybindings` + `KeyBinding.command`, `CueObject::migrate` + its legacy fields;
  `ShowData::ensure_admin` is the remaining rule.
- Not yet: join snapshots don't check the format (6b), replication of undo stacks (6b).

**6b hardening (2026-10-11)** — the link:
- **Stream:** a new TCP port **47658** (LumaSession stream), served under the per-NIC
  LumaSession service. The member opens one connection to the master; it carries join, forwarded
  commands, results, gap fill and snapshots. UDP keeps only small messages (discovery, Health,
  Login, Invite / Dismiss / Member).
- **Encoding:** length-prefixed JSON frames, zstd-compressed; the first frame carries the stream
  protocol version.
- **Join checks:** the stream protocol version and the show `format_version` must match, or the
  join is refused with an error code; a different build commit only warns (join reply and sysmon).
- **Everything waits for the master** in 6b, programmer and selection too (an isolated LAN
  answers in about 1 ms). §10's "programmer instant, corrected later" is postponed and built only
  if waiting is noticeable.
- **Pending outbox now** (pulled forward from 6c): forwarded commands stay in an ordered outbox,
  marked *pending*, and are resent until the master confirms. The 5 s timeout, IdleMaster and
  election stay 6c.
- **Undo travels as steps:** each result carries what happened to the sending user's stack
  (frame filed / Oops / Redo); members replay it, so every stack matches the master's. Modes,
  claims and variables travel as small snapshots in the same result.
- **Runtime split:** session-wide and replicated are each user's modes and undo, who is logged in
  on which station, claims (moved from `AppState` into Runtime) and global variables.
  Screens follow the thin-client decisions (`handoffs/2026-10-10-thin-client.md`): screen
  configs are show data on the profile and live window changes write through to them, so a user
  on two stations with the same screen config sees the same layout because the profile replicates.
  The write-through (`screens::sync`) becomes a change that goes to the master like any other.
  Clients, watches, which screens are open, scroll and monitor positions stay per station.
- **Old UDP pieces:** `SubmitOperation`, `RequestOperations` and `ShowDataSnapshot` are replaced by
  the stream in 6b (§26 already moved big data to TCP); `JoinNegotiation` / `JoinPolicy` stay
  until 6d's merge prompt replaces them.
- **Two servers on one machine:** `lumacore-server --station <name>` keeps that station's local
  files (station settings, sACN CID, revoked tokens) in `library/stations/<name>/` while shows
  and GDTF stay shared; `--port-base N` sets cmd N+3, shell N+4, HTTP N+6, stream N+8;
  discovery stays 47655 for both, bound with SO_REUSEPORT so they hear each other. No flags =
  today's behaviour.
- Proxmox testing is dropped (scripts, doc, lumacore-scripts entries); multi-station live
  tests run as local servers.
- **Priority on join:** a joining or invited station always joins as a member and takes the
  session's show. If its priority is higher than the master's, it gets a **prompt to take over**
  as master. Taking over is a master handover with an epoch bump, so the prompt
  and handover are built with the election in 6c. The reference show choice for that case (keep
  session's / keep mine / merge) belongs to 6d's merge.
- **Join download is atomic** (a reference console: leaving mid-download falls back to the previous show): the
  master's show arrives into a staging copy and replaces the station's show only when complete
  and format-checked. On failure the station stays standalone with its old show and output
  unchanged.
- Reference probe (live + manual): members need the same streaming version (first
  three numbers); a lone session station reports `IdleMaster`, and IdleMasters there send DMX only
  with "Send sACN / Art-Net If IdleMaster" (our output when alone); stations listed in the show's
  network list are re-invited automatically (6c: the known-stations list lives in the show);
  health is a 0–255 flow-control level and NACK counts (1 / 5 / 10 min, total) per station on the
  master (6c, fed by 6b's outbox resends); session keys are not used (§19: IP trust on isolated
  networks).

**6b built (2026-10-11):**
- `remote::stream`: TCP :47658 (`--stream-port`), frames = u32 length + zstd(JSON `StreamMsg`);
  `Hello` (protocol `STREAM_PROTOCOL = 1`, `format_version`, commit, station, user, resume) →
  `Welcome` / `Refused` (E-LUMA-001 protocol, 002 format, 003 not a master); `Command` /
  `Reply`, `Op`, `Snapshot`, `Logins`, `Changes`, `Resync`, `Bye`. Master: `spawn_stream_server`
  (LumaSession service per NIC), a reader + writer thread per member. Member: `spawn_link`
  (connect, Hello, show parsed and format-checked off the state thread, then install; reconnect
  with back-off 0.5–5 s, unanswered commands resent in order).
- `remote::link` (state thread): members forward lines whose `registry::line_scope` is Master
  (`dispatch` before anything runs; objects declare `ObjectDef::scope`, `Set Station|Interface`,
  `Set Network … Interface`, windows, screens, views and paths under StationSettings/Runtime stay
  local; `Clear` became Master). Pending outbox = `AppState::forwards`; the reply goes to
  `ReplyTo` (the user's Shell output, a waiting UDP request ≤ 1.5 s on its own thread, HTTP ≤ 10 s,
  else sysmon); "Pending" after 1 s; `Info LumaSession` shows Master, Members (users per
  station), Applied, Pending. A member's own show changes (screen write-through, station-local
  commands) go as `Changes` and the master files them (`commands::record_change`).
- Master `flush` after every job: the outbox (plus `UndoStep`s and the `SessionRuntime` when it
  changed) committed to the `ReplicationJournal` (`journal` id + `epoch` 1 + `seq`) and sent to
  every member; a show load restarts the journal and sends a `Snapshot`. Members apply in order
  (`apply_op`); a gap or another journal → `Resync` (missed ops, else full snapshot).
- Session runtime: modes, claims (moved to `RuntimeState::claims` / `claim_order`), global
  variables, members (with `connected` and `users`), master's users. Undo stacks mirror via
  `UndoStack::apply_step`. Screens follow the profile (`screens::follow_profiles`).
- Removed: UDP `ShowDataSnapshot`, `SubmitOperation`, `RequestOperations`, `Member`,
  `ReplicationPacket`, `RemoteReply::show_data_snapshot`, `ShowDataScope`; Proxmox scripts/doc.
  `Invite` carries the stream port; discovery beacons too.
- `--station <name>` (files in `library/stations/<name>/`, logs too; default station name) and
  `--port-base N`.
- Not yet (6c): heartbeat / 5 s timeout, IdleMaster, election, epoch bumps, health counters; a
  member keeps reconnecting until Leave. `Invite` / `Dismiss` still do UDP I/O inside the command.

**System monitor research → 6c / 6d (2026-10-11)** (dumps in gitignored `intel/`; manual
2.5 release notes 1.9 / 2.2; live tree: `LivePatch` 14.9 replicated, `Patch` 14.10 the edit copy
with `IGNORENETWORK`):
- **Patch version (6c, used by 6d):** `ShowData.patch_version` rises on every live patch change
  (Keep Patch, Live Patch `Set`, Import); it travels with the show and is remembered at a split.
  6c's rejoin compares it; 6d blocks Merge All when both sides changed the patch since the split
  ("master patch version has changed" → renegotiation).
- **Show status (6c):** NoShow / Loaded / Downloaded (taken from the master) / Saving /
  Negotiating, in the discovery beacon, the LumaSessions menu, `Info Show` and the status bar; a
  join reads Negotiating until the show is installed.
- **Output distribution (6c):** whenever the send plan changes (show, NIC mapping, role: new
  master, IdleMaster, held output) sysmon logs which station now sends which lines/universes;
  `Info Station` shows the plan ("output distribution changed").
- **Link down (6c):** the NIC carrying the stream losing its link or address cuts the station
  off at once (commands pending, "no cable on en1"); the 5 s timeout still decides IdleMaster so
  a quick re-plug doesn't flap ("no cable").
- **Reference check (6d, then everywhere):** after merge, load, join and undo restore every link
  (GUID or number) must resolve and nothing may contain itself; broken links are listed (sysmon,
  `Info Show`), never silently dropped. Later steps run it after Store too (Total Reference
  Update).
- **Ownership (6d):** §10's edit locks are built with the merge: editing claims automatically,
  direct and indirect (a macro whose line is edited), `List Claims` shows them, taking a claimed
  object asks its owner with a 10 s countdown, an Admin can force (Ownership,
  `ListOwnership` / `DropOwnership`).
- **Multicast (a reference console sends session data by multicast from a Multicast Base, needs IGMP
  snooping, falls back to "TCP only"):** session data stays on each member's TCP stream —
  results and the 6c heartbeat — reliable, ordered, no IGMP or router needs; a session has a few
  stations, so per-member fan-out is cheap ("TCP only", always). The **discovery group is
  configurable (6c):** `Set Station MulticastBase Default|Alternative` (Default
  239.255.76.67, Alternative 239.255.76.68; port 47655), station-local, shown in the LumaSessions
  menu; stations on different groups don't see each other, join by address still works.
- **Later:** an overload warning when frames overrun 22.7 ms goes to step 9 (render thread);
  licence parameter limits don't apply. **Skipped:** a startup macro and an agenda.

**6c hardening (2026-10-11)** — health, master loss, election:
- **Heartbeat:** master and members beat on the stream every **500 ms** (also measures
  latency); **warn after 1.5 s** without one (`Info Station`, status icon), **lost after 5 s**.
  Named constants; debug builds take `--session-timeout <s>` (a hidden disable-timeouts switch),
  release builds don't. Link down on the stream's NIC counts at once (pending, "no cable on
  en1"); the 5 s still decides.
- **Losing the master — election first, IdleMaster only when alone** (resolves §10): after 5 s
  every member computes the **same winner** from the replicated member list, counting only
  stations still in discovery: highest **priority** → **most results applied** (fewest lost) →
  **online time** (±5 s) → lowest stable station id; `Never` can't win. The winner becomes
  Master with a **new epoch** (before any output) and keeps the session's show; the others
  connect to it and **resend their pending commands there** (nothing diverges). A winner that
  can't be reached within 5 s is dropped and the next one computed. A result the winner never
  had is named in sysmon. A station becomes **IdleMaster** only when no winner is reachable (cut
  off, or the last one left): its pending outbox is applied locally and everything since the
  split is journaled (§10), output only with "output when alone".
- **Diverged IdleMaster finds the session again (6c, before merge):** nothing automatic — it stays
  IdleMaster and the operator decides (`Join LumaSession` takes the session's show with an
  autosave copy of its own first, or keep working alone). Not diverged → silent fast-forward.
  6d replaces this with the merge prompt.
- **Two masters meet** (old master back, split healed): beacons and `SessionRuntime` carry the
  epoch; the **lower epoch is fenced at once** (stops DMX) and its operator gets a prompt:
  **Join <new master>** (autosave mine first) / **Keep mine** (become master again with a new
  epoch; the other side's stations take my show, each autosaving its own first) / **Stay alone**
  (IdleMaster, no output) — worded as Use Master / Use Mine / Cancel (see below). The higher-epoch master's operator gets a notice. No timeout until 6d's
  ShowSettings default/timeout; 6d adds Merge All. The prompt is a notice to the station's users
  plus a UI pop-up whose buttons run the commands (`Join LumaSession "<name>"`,
  `Claim LumaSession`, `Leave LumaSession`).
- **`Claim LumaSession`:** takes the master role. A higher-priority station that joins is told it
  outranks the master and that `Claim LumaSession` makes it master; an Admin on any member can use
  it any time: the old master hands over cleanly (new epoch), everyone moves to the new master,
  nothing lost.
- **Known stations:** every station that ever joined
  is listed in the show (name, station id, last IP, priority, joined time), so every station knows
  who was in the session; Leave and Dismiss keep it. Listed stations that reappear in discovery
  with Invite on are **re-invited** by the master (back-off: stop after 5 failures a minute).
  `Delete Station "FOH-2"` removes one (master, Admin; refused while connected — Dismiss first).
  `List Station` shows In Session / Lost / Known.
- Also 6c (above): patch version, show status, output-distribution log, link-down, discovery
  Multicast Base Default / Alternative. Online time counts from when the station's LumaSession
  went online; the stable id is `station_id`.
- **System-test research (2026-10-11):** the session prompts are native generic
  message boxes; the reference negotiation mode is Cancel / Use Master / Merge / Use Mine;
  Station Control holds Data Merge Default Mode / Timeout (`ShowSettings.GlobalSettings`) and a
  per-station "Include Playbacks" (`PlaybackDataMerge`); the Oops list has SessionTime and an
  InMySession filter. Decided:
  - **Prompt words = the reference modes** for 6c's two-masters prompt and 6d's merge: **Use Master**
    (take the session's show, mine autosaved) / **Use Mine** (my show wins, the others autosave
    theirs) / **Cancel** (stay alone: IdleMaster, no output); 6d adds **Merge**. Same words in the
    pop-up, sysmon and the ShowSettings default mode; the buttons run `Join LumaSession` /
    `Claim LumaSession` / `Leave LumaSession`.
  - **Show status = the fuller set:** NoShow, Loaded, Downloading, Downloaded, Uploading (the
    master sending a joiner the show), Saving, Negotiating Master / Member / Waiting for new
    master (no media states yet).
  - **Undo frames record their station and time (6c):** shown in `List Undo` and
    `Runtime.Users.{n}.Undo`, filterable to this station.
  - **6d acceptance tests copy a reference data-negotiation system test:** new presets on both
    sides, deletes on both sides, a move on one side (Merge keeps both copies, Use Master drops the
    move), cue inserts and deletes while disconnected, the session stays valid after.

**6c built (2026-10-11):**
- `remote::election`: `BEAT` 500 ms, `WARN` 1.5 s, `lost_after()` 5 s (`--session-timeout`, debug
  builds). `elect` (members list + discovery: priority → applied → online ±5 s → station id;
  `Never` can't win; `Alone` when nobody else is seen), `become_master` (new epoch, journal
  restart, pending commands run here), `become_idle` (IdleMaster, split mark in the journal,
  pending run here, diverged once it changes the show), `tick` (once a second: health warnings,
  link down → socket closed at once, two masters → `fence` + decision, IdleMaster found its session
  → silent rejoin or decision, re-invites), `hand_over` / `handle_yield` / Use Mine, `cmd_claim`,
  `cmd_delete_station`, `status_json`. `Role` (Normal / IdleMaster / Fenced), `ShowStatus` (the reference
  set), `Decision` (Use Master / Use Mine / Cancel → `Join LumaSession <ip:port>` /
  `Claim LumaSession` / `Leave LumaSession`).
- Stream: `Ping` / `Pong` heartbeat both ways, the writer closes a silent connection after the
  timeout; `Claim` (member asks for the master role), `NewMaster` (master → members: follow
  `station` at `host:port`, epoch, autosave). Member reconnects wait 3 s for a Welcome (a master
  that accepts but doesn't answer must not hold the election up); after the timeout the member
  elects and follows the winner, dropping one it can't reach for 5 s. `Hello` carries station id,
  priority, stream port; `Welcome` the master's priority (a joiner that outranks it is told it may
  `Claim LumaSession`).
- `ShowData.session` (`SessionData`: `patch_version`, `stations` = `KnownStation`s),
  `ShowSubtree::Session` (replicated, never undone), `FORMAT_VERSION = 2`. A Devices change raises
  `patch_version` in `dispatch::commit`. Admit remembers the station; Bye marks it as left (no
  re-invite); re-invites go to Standalone / IdleMaster known stations with Invite on (every 10 s,
  stop after 5 a minute); two standalone stations: only the higher-ranked one invites. A
  diverged IdleMaster or fenced master refuses invites.
- Discovery: beacons carry group, epoch, applied, online time, show status; both groups are
  joined, the other group's beacons ignored; `Set Station MulticastBase Default|Alternative`
  (239.255.76.67 / 239.255.76.68). `List LumaSession` shows each station's show status;
  `List Station` this station, members (users, lost) and the known list.
- Undo frames carry station + time (`List Undo [/ThisStation]`, `Runtime.Users.{n}.Undo`);
  `SessionRuntime.epoch`; `MemberRuntime.{station_id, stream_port}`; output distribution logged
  on change (`Output: distribution changed: …`, `Info LumaSession` Output); `Info LumaSession`:
  Link (health, latency, last heard, resends), Epoch, Show Status, Decision. Status bar: a
  LumaSession icon (role, health, latency, pending) and the decision pop-up (a message box).
- Fixed on the way: a joining member's runtime was marked sent, so earlier members never learned
  the new member list (their election missed it).

**6d hardening (2026-10-11)** — identity, merge on rejoin, reference check, ownership. One step,
one commit. Read for it: §4, §10, a reference console's manual (session join,
`network_station_control`) and `system_test_data_negotiation.lua`. Code today: GUIDs only on
`NodeMeta` objects (fixtures, types, universes, networks, lines, datapools and their contents,
profiles, macro lines); most links are numbers or names; nothing is kept at a split; claims are
text-keyed; `JoinPolicy` / `JoinNegotiation` are dead code.
- **Every object has a GUID, strictly:** every collection in the tree holds a type with
  `NodeMeta` (an `Identified` bound, so a new object kind doesn't compile without one), and a
  test walks a full demo show and fails on any object without a GUID or with a duplicate one.
  A show with duplicate GUIDs is refused on load. This covers sequences, cues, parts, executors,
  masters, effects, timing/curve presets, users, screen configs, views, UI keys, 3D objects, FID
  namespaces, feature/activation groups, effect engines and known stations.
- **Every stored link is a GUID**: executor → sequence, route → line, line → network, user →
  profile, cue value → preset, parked channel → universe, spatial patch → fixture, attribute →
  feature group, … Numbers and names are only what commands and lists show, so a move or a
  renumber never breaks a link. A test checks every link field resolves in the demo show.
  Command text (macro lines, key gestures) stays text.
- **Old shows:** `FORMAT_VERSION = 3`. Forward only: an **older** format is refused too
  (E-SHOW-002, naming both versions); gen-demo regenerates the demo show. No converter.
- **Show identity:** `ShowSettings.show_id` (UUID) made on New Show, kept on Save, new on Save As.
  Only the same `show_id` can Merge; otherwise the prompt offers Use Master / Use Mine / Cancel.
  Lineage = `show_id` + journal id + seq at the split.
- **Split base:** at the split the station keeps its show as of the last applied (journal, seq):
  in memory and as an autosave file (reason `split-base`), with the `patch_version` it had. On
  rejoin the merge is **3-way**: base vs the session's show vs mine.
- **Merge rules** (smallest GUID object is the unit: a cue part, a macro line, a preset, a group,
  a fixture):
  - Added on one side → added. Both sides added in the **same slot** (different GUIDs) → the
    master's keeps the slot, mine goes to the **next free slot**, listed.
  - Deleted on one side and untouched on the other → deleted (the reference test: both sides' deletes
    apply). Deleted on one side, **edited on the other → the edit wins**, listed.
  - Changed on both sides → **the master's version wins**, listed (mine is in the autosave copy).
  - A **move** is a slot change of the same GUID, so the move applies and nothing is duplicated
    (a reference console duplicates, and its "keep master" test drops the move). This replaces the "Merge keeps
    both copies" note in the system-test research above.
  - **Running playbacks** (`ShowData.Playbacks`) always come from the master; no Include Playbacks
    toggle.
  - **Patch changed on both sides** (patch_version vs the base) → Merge is greyed out with the
    reason shown; a timeout with default Merge falls back to Cancel.
- **The prompt:** Merge / Use Master / Use Mine / Cancel (6c's prompt plus Merge), listing what
  differs (added, deleted, conflicts, slot moves). `ShowSettings.merge_default` (default **Merge**)
  and `merge_timeout` (default **60 s**; 0 = no prompt, the default applies at once; Unlimited =
  wait), Admin to change. Clicking the pop-up pauses the countdown. Every losing side is
  autosaved first.
- **Autosave names say why:** `.autosave/{show}_{reason}_{station}_{time}.luma`, reason
  `before-merge` / `use-master` / `use-mine` / `invite` / `split-base`; listed in sysmon and the
  merge result; kept until deleted.
- **Reference check** after merge, load, join and undo restore: a link to a missing GUID is
  **kept** (a later merge or undo can heal it) and shown as `missing <kind> <guid>`; sysmon and
  `Info Show` give the count, `List Reference /Broken` lists them. Something
  containing itself is refused when written and reported if found on load.
- **Ownership (§4, §10):**
  - **Opening an editor claims** (`Edit`, `Enter`, UI editor windows) — indirectly too: editing
    a macro line claims the macro, a cue part its sequence. Leaving releases it. Plain commands
    (Store, Set, Label) don't claim but are refused on another user's claimed object.
  - **`cd` never claims** (removes today's auto-claim on Patch, DataPools, ShowSettings…). The
    Claims object is a place: `cd Claims` then `List` shows every claimed object.
  - **Taking a claimed object:** the owner gets "X wants Sequence 3" Allow / Deny with a 10 s
    countdown; no answer, or the owner logged out / their station gone → handed over; Deny
    refuses. An Admin's `Claim … /Force` skips the prompt (this is §4's "Admins can unclaim
    anyone"). *(As built the buttons are **Hand over / Keep**; §30 "12b hardening" adds
    "Remember for this session".)*
  - Claims are keyed by **GUID** (a move keeps the claim). `List Claims` replaces `ListClaimed`
    (§15, no alias). `Claim Patch` still starts the draft.
- **Removed:** `JoinPolicy`, `RemoteRequest::JoinNegotiation`.
- **Acceptance tests** copy a reference data-negotiation system test: a cue inserted while not in
  session; new presets on both sides; deletes on both sides; a move (Merge applies it, Use Master
  drops it); cue inserts and deletes while disconnected; the session stays valid after.
- **Stale docs fixed with 6d:** §1 says the undo stack is saved in the show (it's Runtime, never
  saved); §8 says `.luma` is a zip container (it is plain JSON, see `show-files.md`).
- **Distribution is strict about GUIDs too** (asked during the build: "when merge and other types of
  session reference object distribution system is used, guids needs to be used strict"): replication
  ops, undo frames and members' `Changes` name pool entries by GUID, never by slot.

**Identity research (2026-10-11, live GET probe + manual corpus):** a reference console's GUIDs are 16 bytes
written as spaced hex (not RFC UUIDs), read-only, with no lookup by GUID. Cue (only its Part),
Executor, DMXMode, Attribute and PresetPool have **no** GUID there. Links are handles/addresses in
memory and name paths in XML (`Object="ShowData.DataPools.Default.Sequences.Default"`); a cue's
preset dependency exports `RelAddr` plus the preset with its GUID; groups and phasers name fixtures
by FID. `ListReference` lists references; deleting a referenced preset copies its values into the
cue. The session merge only runs for the same show file and doesn't say how it matches (a move
duplicates — slot/address). So LumaCore goes further on purpose: GUIDs on every object, every link
a GUID, merge by GUID (moves don't duplicate). Kept from the reference: the merge prompt and its four modes,
default + timeout, patch changes block Merge, `List Reference`. Not copied: tombstones of deleted
objects (`ShowDeletedData`) — the 3-way split base tells "deleted" from "never existed".

**6d built (2026-10-11):**
- Identity: `lumacore_core::identity` (`Guid` — `Default` = fresh v4, `Guid::named` = v5 of a
  standard name; `Identified`; `Walk` + `Walker` + `IdentityReport`); `lumacore_tree::identity`
  (every show type walks with an exhaustive destructure; `ShowData::identity()`). New `guid` fields
  on sequences, cues, parts, assignments, masters, effects, timing/curve presets, executors, users,
  views, screen configs, screens (`ScreenContent`; windows are its content), FID namespaces, known
  stations (`KnownStation::guid_for(station_id)`), spatial objects, endpoints, priority orders,
  attribute definitions, feature/activation groups, fixture groups (removed in step 7, §22), timecode tracks/clips/
  regions/markers, effect-engine objects, UI keys (`Guid::named("UiKey", id)`).
- GUID links: executor ↔ sequence, line → network / user (`PropKind::Link` resolves names on
  `Set`), route → line, user → profile / default screen config, view buttons, selected datapool /
  preset pool, fixture → revision / mode (`FixtureType::mode_of`), FID membership, attribute →
  feature / activation group, spatial patch → fixture, parked channel → universe, `Value.preset`,
  running playbacks keyed by sequence GUID (`started_by` = user GUID), show icons. `Store Input …
  /User X` refuses an unknown user (a missing user never means Admin).
- `FORMAT_VERSION = 3`; `parse_show` refuses older (E-SHOW-002) and newer (E-SHOW-001) formats
  and missing/duplicate GUIDs (E-SHOW-003). `ShowSettings.{show_id, merge_default, merge_timeout}`
  (`Show /MergeDefault`, `Show /MergeTimeout`, Admin). Demo show regenerated.
- `ShowSubtree::{DataPool, Profile, User, Programmer}(Uuid)`; `oops::restore_subtrees` removes
  first, then places each object at the slot its data names, refusing an occupied slot.
- Reference check: `terminal::references` after load / join / merge / Oops / Redo; `List
  Reference [/Broken]` (Reference object), `Info Show` (the loaded show: id, format, references,
  merge settings).
- Merge: `lumacore_tree::merge` (`three_way`, `can_merge`, `MergeReport`); `election::SplitBase`
  kept by `become_idle` and by `become_master` when taking over (autosave `split-base`); stream
  protocol 2: `Hello.{peek, merge}`, `Preview`, `Welcome.merged`; `link::{preview, merge_in,
  common_base, merge_offer}`; E-LUMA-005. The decision has Merge (`Join LumaSession … /Merge`),
  details (refreshed while it waits), `merge_blocked`, default + deadline, `Pause LumaSession`
  (new object verb); `tick` runs the default when time is up. Autosave names
  `{show}_{reason}_{station}_{time}`. Removed `JoinPolicy`, `RemoteRequest::JoinNegotiation`.
- Ownership: `Runtime.{claims: BTreeMap<Uuid, Claim>, claim_order, claim_asks}` (replicated);
  `terminal::claims` (label → GUID; areas by name); `Edit` claims (released on exit); E-CLAIM-001
  in dispatch for changing commands on another user's object or anything inside it; `Claim` asks
  the owner (`Unclaim` / `Claim … /Keep`, 10 s, logged-out owner → at once, `/Force` Admin);
  `ListClaimed` removed (`List Claims`); `Root.Runtime.Claims` (`cd Claims`, `List`). UI: the merge
  pop-up lists differences, greys Merge out, counts down, click pauses; a Keep / Hand over pop-up.
- Not yet: owner notices reach only the master's own Shells (members see the pop-up from the
  replicated asks); Timecode and other objects without a tree definition can't be claimed; no cue
  editor yet (claiming a sequence already protects its cues).

## 22. Groups (2026-10-10)

- Groups live only in the **DataPool** (`Group`). The old `FixtureGroup` /
  `ShowData.fixture_groups` is removed; HTTP group endpoints run real commands.
- A group stores **order + grid positions** of its members.
- Members: fixtures only for now (subfixtures are a later discussion).
- Colour and icon come from `NodeMeta` like every object (no separate appearance pool).
- An empty `pawtern` slot is reserved (§3).

### Step 7 hardening (2026-10-11)

Asked round by round (`handoffs/2026-10-10-decisions-qa.md` "Step 7 hardening"). A reference console was read
(corpus 2.5.0.3 + live `localhost:3333`) and, with the user's leave, probed by running commands
on the live console (selection counts read from `UserProfiles.<n>.Environments.1.1
COUNTTOTALSELECTED`; the operator's selection was saved to a temporary group and restored).
One step 7 (not split).

**Findings that shaped it**

- A group stores only the selection — `SELECTIONDATA = [{sf_index, grid{x,y,z,inv,x_inv,y_inv,
  z_inv}}…]`: array order = selection order, independent of the grid; no values. Pools are
  per DataPool (`Group 3` is pool-relative). Groups also carry MODE (group master), MOVEGRIDCURSOR,
  PRESERVEGRIDPOSITIONS, an (unused) Pawterns block, Appearance, Lock.
- **Selection rule (probed):** plain selects add (`Group 4` 0→66, `Group 5` →98) until a value is
  set; the first plain select after a value replaces (`At 50`, `Group 4` → 66); the next plain one
  adds again. `+ Group 3` always adds (98→441); `Group 3 + Group 4` after a value replaces with the
  union (409). Manual: `keyword_selectfixtures`, `qsg_control_fixtures`.
- Two groups selected in a row: the second block lands further along X (AppendX grid cursor), each
  keeps its own layout. `Store … /Merge` + `/GridMergeMode AppendX|Off`.
- Group masters: Mode None / Positive (HTP limit) / Negative (LoTP limit, wins on overlap) /
  Scaling (multiplies) / Additive (adds, HTP); applied after the merge, after grand and world
  masters; fixtures' `Master React` Grand / Group / None. Reference group masters snap.
- Pool windows: every pool is one `AllPoolWindow`/`AllPoolLayoutGrid` with a `Pooltype`, one
  base `PoolSettings` (show title, font size, show empty, columns, datapool, first index, pool /
  empty / For All / For Some / For None colours, 2-finger edit, object action) plus a few per-pool
  extras (Pool Action enum per pool, preset display mode / preset mode / cue part, sequence
  executor style). Windows are `ViewWidget`s (display, x, y, w, h + settings + appearance +
  scroll) in the profile's ScreenConfiguration; Views snapshot them. A tap completes a pending
  keyword (`Delete`, tap → `Delete Group 3`) or runs the pool action.

**Decisions**

1. **Model.** `Group { meta: NodeMeta, members: Vec<GroupMember>, master: GroupMasterMode,
   preserve_grid: bool, move_grid_cursor: Linked|None|AppendX|NewLine, pawtern: Option<Uuid>
   (reserved, None) }`; `GroupMember { fixture: Uuid, x, y, z: i32, rotation: f32, mirror_x,
   mirror_y: bool }` — order is the Vec order, the grid is independent (§28 needs rotation /
   mirror). Colour / icon from NodeMeta. Fixtures only. `FixtureGroup`, `ShowData.fixture_groups`,
   `ShowSubtree::FixtureGroups` and the unused `GroupItem` trait go. Format bump (forward only).
2. **Selection rule — exactly as probed, for every selection** (Fixture, Group, Selection, lasso,
   window taps): the user's programmer has a "value set since the last select" flag; a plain
   select adds while it is clear, the first plain select after it replaces (and clears it); `+`
   always adds; `A + B` replaces with the union after a value. `Group N - Fixture 3` deselects.
3. **Full grid.** The user's selection carries grid positions; a per-user grid cursor with
   None / AppendX / NewLine (profile default; the group's `<Linked>` uses it); `Grid x/y[/z]` moves
   the cursor; PreserveGridPositions per group (off = gaps and offset removed on recall). Store
   keeps positions.
4. **Store over an existing group is refused** with a hint: `/Merge`, `/Overwrite`, `/Remove` (the
   window shows them as buttons). `/GridMerge AppendX` (default) puts merged members after the
   group's last X; `/GridMerge Off` keeps their selection-grid positions. Nothing is overwritten
   silently. Overwrite keeps the GUID.
5. **Datapool:** `Group 3` means the user's selected datapool — store, recall, list and window
   alike (today's recall searching every datapool is a bug); `DataPool 2 Group 3` addresses another.
6. **Fixture gone:** unpatched fixtures still render in the group; deleting a fixture leaves every
   GUID link in place (members keep GUID and grid spot, recall skips them, the reference check lists
   them). Later: a references view per missing object ("these cues, presets, groups point at
   deleted fixture X") with **Resurrect** or **Delete links** — recorded, not built in step 7.
7. **Delete a referenced group** is refused, naming the links; `Delete Group 3 /Force` deletes and
   leaves them broken (listed).
8. **Group masters are built now.** Mode on the Group (show data, undoable: `Set Group 4 Property
   Mode Positive`). The live level and on/off live in `ShowData.Playbacks.group_masters[group
   GUID]` — replicated and saved, never in undo frames (like running sequences); deleting the group
   drops its level. Commands: `Group 4 Master At 50 Fade 5` (the group's `Master` child; masters
   **fade**, unlike the reference, which snaps) and `Set Group 4 Property Master 50` (`Fade 0` = snap, the
   default). `Group 4 At 50` stays as established: select the group, set values. The playback engine applies
   them after grand and world masters (§27): Negative beats Positive on overlap, Scaling
   multiplies, Additive adds HTP; masters never touch Park or DMX input.
9. **Fixture `Master React`** `Grand | Group | None` (default Grand): None ignores every master and
   Solo (house lights); with Group, Additive group masters skip the grand master.
10. **Group recall writes the selection** — its undo class is PROGRAMMER, not ReadOnly (today's
    bug: the selection change was neither undone nor replicated). Knock in / out on groups stops
    round-tripping through `with_keyword`.

**Pool numbers** are 1–9999 for **Groups** (`pools::MAX_NO`, R1-11; other pools are not
capped yet): Store, Copy and Move refuse a **new** number above it; a group an older show
already has above the cap is named, labelled, moved down and deleted like any other.
A pool window **pinned** to a DataPool sends owner-first lines (`Delete DataPool 2 Group 3`; the
station runs the Group verb in that DataPool, `AppState::datapool_override`), and a pinned DataPool
that is gone shows missing — never the user's selected one (R1-55, Q30). The owner stays with
the line: a question the Group verb asks carries it in its buttons (`Store DataPool 2 Group 3
/Overwrite`), `Edit DataPool 2 Group 3` edits that pool's group, and a panic clears it.

**Unified pool window** — one window for every pool, so presets and later pools only add their
extras:

11. **`ObjectDef.pool: Some(PoolDef)`** — the collection, actions + default action, settings
    extras and a `cell_extras` hook. The server builds generic cells from the published show and
    runtime: `no`, `name`, colour / icon, lock / owner (claimed by someone = the red "edited by"
    lock), `referenced`, `applicability` (All / Some / None), `bar` (kind + value + text), badges.
    Group fills in the master bar + mode badge and applicability (All = every member selected by
    this user, Some = part, None); presets add theirs later. The frontend has **one PoolWindow
    component** for every pool.
12. **Live feed:** one `/ws/pools` — clients subscribe per (pool, datapool); after each change the
    server pushes changed cells (new objects, master levels, claims, references). `/api/groups`
    goes; the old client-only group taps go.
13. **Gestures — every gesture sends a real command line** (rights, claims, undo, replication
    apply):
    - Tap: completes a pending keyword (`Delete` / `Edit` / `Label` / `Store` then tap →
      `Delete Group 3`; `Move` / `Copy` wait for the `At` target), else runs the pool action
      (Group: select, by the selection rule).
    - **Hold, drag, release = full-screen selector:** hold still for the profile's `hold_ms` and
      the whole pool opens full-screen with large readable buttons; dragging hovers over objects,
      releasing over one selects it (runs the tap) and closes the selector. Hold + release without
      dragging opens it and does nothing; Esc / tapping outside closes it. Small windows stay
      usable with precision. No swipe menu.
    - **Drag moves, Ctrl-drag copies** (the Windows / CAD convention; Ctrl does not collide with
      Meow = Alt): a press that moves before `hold_ms` drags; dropping sends `Move Group 3 At 7`
      (occupied target refused) or `Copy …`. Profile setting `pool_drag: Off | Move` guards touch
      screens (no Ctrl there; Copy by keyword).
    - Right-click: Store, Edit, Label, Delete, Move, Copy, Settings.
14. **`Edit Group 3`** (and right-click → Edit) claims the group (6d) and opens the members + grid
    editor: ordered member list (reorder, remove) beside a grid with each member's x / y and
    rotation / mirror, edits by commands; leaving releases the claim. In the Shell it enters the
    group (`List` shows members).
15. **Window settings:** pool settings are window-type settings on the window instance
    (`Window.settings`, already a ViewWidget: settings + appearance + scroll) in the user's current
    screen (`UserProfile.screen_configs…screens…windows`) and are snapshotted into Views, so a view
    recalls windows and their settings exactly. Common pool settings: show title, font size, show
    empty, columns (auto / fixed), datapool (follows the user's or fixed), first index, pool /
    empty / For All / Some / None colours, action; per-pool extras come from the `PoolDef`.

### Step 7 built (2026-10-11)

- **Model:** `datapool::{Group, GroupMember, GridCell, GroupMasterMode, GridCursorMove}`;
  `FixtureGroup` / `ShowData.fixture_groups` / `ShowSubtree::FixtureGroups` / `GroupItem` and the
  old `/api/groups` are gone; FORMAT_VERSION 4 (demo shows regenerated). `Fixture.master_react`
  (`MasterReact`, `Set Fixture n MasterReact None`). `ShowData.Playbacks.group_masters[group
  GUID] = MasterLevel {from, to, start_ms, fade_ms}` (`lumacore_tree::now_ms`).
- **Selection** lives on the programmer: `selected_fixtures` (order) + `grid` (spot per fixture) +
  `grid_cursor` + `value_since_select` (set by `Programmer::set_value`).
  `Programmer::apply_selection(fixtures, cells, SelectMode::{Plain, Add, Replace}, preserve,
  cursor)` is the one rule; `AppState::{select_fixtures, select_members, user_groups}`. A cleared
  or replaced selection empties the grid and puts the cursor home; `Grid x/y` survives an empty
  selection. Selection expressions keep the typed order (`Fixture 3 Thru 1`, no more sorting) and
  `Fixture` works inside them (`Group 5 - Fixture 2`).
- **Commands** (`terminal/commands/groups.rs`, the Group object): `Group …` (terms split at `+` /
  `-`, groups by number, range or name, all pool-relative), `Plus` / `Minus` (`+` / `-`), `Grid`,
  `Group n Master [At v [Fade s]]`, `Store Group [n] [name] /Merge|/Overwrite|/Remove /GridMerge
  AppendX|Off` (bare Store Group = first free), `Copy … At … /Merge|/Overwrite`, `Update`,
  `Delete … [Thru] [/Force]` (refused with the linking paths, E-GROUP-002), `Set Group n
  [Property] Name|Mode|Master|PreserveGrid|MoveGridCursor` and `Set Group n Fixture f
  X|Y|Z|Rotation|MirrorX|MirrorY|Order v …` (the editor's commands), `On` / `Off` call
  `control::knock_{in,out}_fixtures` directly (`with_keyword` removed). Group recall is
  `SELECTION` (undoable, replicated). Errors E-GROUP-001/002/003. `Set … Master` snaps; fades are
  `Group n Master At v Fade s`.
- **Engine:** `playback::render_console_at(input, now_ms)` applies group masters after the merge
  (Scaling multiplies, Negative-else-Positive limits, Additive HTP; Master React Grand / Group /
  None, None also skips Solo); the engine keeps rendering while a master fades
  (`masters_fading`) and once after.
- **Pool windows:** `ObjectDef.pool: Option<&PoolDef>` (`terminal/pools.rs`: window type,
  tap actions as command templates, `cells`, optional `detail`); `PoolCell` = no, guid, name,
  colour, icon, owner (another user's claim only), referenced, locked, applicability, bar, badges.
  `StationView.claims` publishes claims. `GET /api/pools/{pool}[/{no}]`, `/ws/pools`
  (subscribe per window, pushed on change, 100 ms). A pool tap is the shell's `tap` packet with
  `action` (the server completes a pending command; Move / Copy / Assign take the object, then
  wait for the target slot). Window settings: `window_types::pool_settings` (action, datapool,
  first index, slots / columns / rows / size, font, title, show empty, colours) on the window
  in the screen / view. Frontend: one `buildPoolPanel(win, pool)`; tap, hold → full-screen
  selector (drag + release taps; hold + release only opens it), drag = Move, Ctrl-drag = Copy
  (`pool_drag` Off disables), right-click menu, the group editor (member list with drag-to-
  reorder, grid with drag / rotate / mirror / remove; `Edit Group n` claims, close sends `exit`).
- **Known gaps:** the tree views (`Set` by path, `held_elsewhere`, `cd`) still show the first
  datapool's groups; recipes / effects don't link groups yet, so nothing blocks a delete today;
  `Join LumaSession <host>` by address did not join in the live test (invite worked) — not
  investigated in step 7.
- **Tests:** `groups_tests.rs` (selection rule, grid cursor / preserve, store options, copy GUID,
  pool-relative, Oops, deleted fixture link, master on the frame + fades, member grid, list /
  knock out, pool cells), `pools::tests` (every pool has its window type and actions),
  e2e `tests/e2e/tests/pool-window.spec.ts` (cells, tap, pending command, drag / Ctrl-drag,
  hold selector, editor). Live: demo on two stations — group mode / master fade / selection
  from a member ran on the master, Oops from the member, DMX faded 255 → 0 over 2 s.
- Demo show: fixtures 101–106 ("renumber the fixtures in the demo showfile to 101-and up so we
  can see order vs FID"); groups All (101–106), Odd stored 105, 103, 101, Even; Group 1 is a
  Scaling master.

## 23. Code Structure (2026-10-10)

- Servers run the terminals; clients (CLI, app, web) only stream them.
- First a small `lumacore-proto` crate (wire types, ports, key codes, discovery) so clients
  stop compiling the server; then `engine/` (state, registries, dispatch, undo, replication)
  and `terminal/` (per-connection REPL and rendering) split inside the server with a layering
  rule. Separate crates only if build times demand it.

### Step 8 hardening (2026-10-11)

Read first: this section, the March docs (`architecture.md`, `software-init.md`,
`cli-terminal.md`, `engines.md`, `client-types.md`, networking docs), the thin-client handoff,
and the code survey below. Reference comparison: a live probe of a reference console and its manual
(session, web remote, terminal app and update chapters).

**What we found.**
- Every client compiles the server. `lumacore-cli` is a binary of `crates/lumacore` (179
  deps: ratatui, gdtf, zstd, dmx). `lumacore-app` links `lumacore` with `http` (286 deps)
  because `--server` runs inside the app binary.
- The wire types are tangled. Shell packets import `RemoteShellKey` / `UiEvent` from the
  ratatui module (`repl/tui_types.rs`), and the beacon carries `engines::EngineInfo`. Status
  is untyped JSON. `lumacore-http`'s `/ws/shell` re-declares the shell packets and the key
  JSON by hand. The CLI imports the command registry just to spot `clear`.
- There are cycles in the server crate:
  - `terminal ↔ remote` (dispatch → `link::forward`, AppState holds link/election handles,
    Invite/Dismiss send UDP inside a command)
  - `terminal ↔ protocols`, `remote ↔ server`, `terminal → app`
  - `dispatch` returns `repl::DispatchResult`, and 54 handler files build ANSI tables via
    `renderer`.
- `lumacore-dmx` → `lumacore-http` under `perfetto` (for the trace collector only).
- `build.rs` watches `.git/index`, so every `git add` recompiles the 52k-line crate.
- **Reference console:**
  - The launcher starts a separate engine process, which holds the whole engine and is
    parametrised by HOSTTYPE.
  - `app_terminal` is a small separate client.
  - The web remote is a pixel stream: the host renders, and the browser is a canvas plus input.
  - Two version numbers: VersionBig must match for anything that calculates or joins;
    VersionSmall for listen-only nodes. The station list has a "Wrong Version" filter.
  - Every category-A device (console, desktop software, processing unit) runs the full engine. Nodes only consume.
- **March docs:**
  - "`lumacore-cli` should never become a second copy of the console engine." "The server
    remains authoritative when the CLI is outdated."
  - Raspberry Pi / bridge nodes want DMX without a UI.
  - "Engine" already means a long-running subsystem (`engines/`, playback engine), so §23's
    `engine/` module name would clash.

**Decisions.**
- **Words.**
  - *Station* = one running server.
  - *Shell* = one connected terminal or window on it.
  - *Engine* keeps its meaning: a long-running subsystem (playback, HTTP, Perfetto).
  - The one-per-station layer that owns state and runs every command is **`core/`**, not
    `engine/`. `kernel` stays reserved for later Linux work.
- **`lumacore-core` → `lumacore-base`**, the foundation types crate. This frees "core".
  Inside `crates/lumacore` the module is always written `crate::core::…` (and std as
  `::core::…`), because Rust has a built-in `core` crate (E0659 ambiguity).
- **`lumacore-wire`** (replaces the planned `lumacore-proto`) is the client↔station
  contract, compiled by both sides, depending only on serde. It holds:
  - ports and multicast groups, the mDNS service name, the discovery beacon (+ `EngineInfo`)
  - command-port `RemoteRequest` / `RemoteReply`
  - shell packets, key codes (`RemoteShellCode` / `Key`), `UiEvent` / `UiFocus` / `Viewport`
  - a typed `Status`
  - `StationKind`, both protocol numbers, the E-WIRE error codes.

  `lumacore-http`'s shell bridge uses these types instead of its copies. The CLI learns
  about `clear` from the station's reply (`RemoteReply.clear_screen`), not from the
  registry. Station↔station
  stream frames are not in wire: they carry show types and stay in `lumasession/`.
- **Versioning (two numbers, big / small).**
  - `SESSION_PROTOCOL` (station↔station; today's `STREAM_PROTOCOL`) must match to join.
  - `CLIENT_PROTOCOL` (CLI, app, web shell) must match to attach.
  - Both go in the beacon, Health and the hellos.
  - A client mismatch is refused with a coded error (E-WIRE-001 "client protocol n, station
    speaks m — update the client"). The CLI returns to its connector, and the app's server
    list shows "Wrong version" before connecting. A browser can't mismatch, because the
    station serves its own page.
  - Bump on any wire change (a typed `Status` field counts). Forward only, no shims.
- **`StationKind`** goes in the beacon and Health, like a host type: `Server` (a full
  station: show + engine; desktop, rack or headless), with `Node` reserved for DMX-only nodes.
- **Clients live under `apps/` and depend only on `lumacore-wire`** (+ their own UI crates):
  - `apps/lumacore-cli` (ratatui/crossterm for its raw terminal)
  - `apps/lumacore-app` (wry/tao/muda)
  - the scripts launcher.

  Nothing under `apps/` depends on `lumacore`. `crates/lumacore` is the station:
  `lumacore-server`, `gen-demo`, and the local in-process `lumacore` REPL. `systemtest`
  keeps linking the station (in-process tests).
- **The app spawns `lumacore-server`** (shipped next to the app binary; `target/<profile>/`
  in development) as its child, like a launcher → engine process. `lumacore-app --server` goes
  away; attach, shutdown and restart keep working over the command port.
- **Layers inside `crates/lumacore`, one direction:**
  `server/` (startup, wiring) → `terminal/` (per Shell) · `net/` (I/O) → `lumasession/` →
  `core/` → crates.
  - **`core/`**: AppState, registries, objects and commands, dispatch, change path, undo,
    rights, claims, references, props, patch draft, programmers, pools, keys, VFS, and
    `core::text` (tables, colours, `strip_ansi`). Command output stays text, the same for
    every client, like console command-line feedback (§31: typed inside, rendered at the edge).
    `DispatchResult` and the UI-event queue move here. Core never imports terminal/net/lumasession/server.
  - **`terminal/`**: REPL editing, completion and hint UI, TUI drawing (ratatui), frames,
    keys → command lines.
  - **`net/`** (was the I/O half of `remote/`): shell server, command port, discovery
    beacon, mDNS, sysmon. It hands LumaSession requests to `lumasession/`.
  - **`lumasession/`**: link, election, merge, stream (its own TCP), invite / dismiss /
    member.
  - **Core never calls `lumasession/`.** Commands (`Join LumaSession`, `Invite Station`,
    `Claim LumaSession`, forwarding a line to the master) put a request in core's
    **outbox**; lumasession/ and net/ act on it after the job. This ends the
    Invite/Dismiss UDP-in-a-command exception. Core reads LumaSession status from
    `Runtime.LumaSessions`.
- **Enforcement:** a tidy layering test (source scan, like the keyword test) fails on an
  import against the direction and on a bare `core::` path. There are no cargo feature gates:
  a DMX-only node build becomes possible later because core is clean. Modules now; separate
  crates only if build times demand it (moving is mechanical once the layers are clean).
- **`lumacore-trace`:** TraceCollector / PerfettoGuard leave `lumacore-http`. Both dmx and
  http depend on trace; dmx never depends on http.
- **`build.rs`** reruns on a new commit (`HEAD`/refs) or a source change, not on staging.
- **Docs** corrected in the step:
  - `architecture.md`, `crate-map.md`, `software-init.md`, `cli-terminal.md`, `engines.md`
    (vocabulary)
  - the planning docs' "Tauri runs the engine in-process"
  - dead links in `network-sessions.md`
  - CLAUDE.md.
- **One step 8** (not 8a/8b).
- **The hub (asked while building).** `ServerContext` and the state thread hold every
  layer's state: core, terminals, LumaSession peers, discovery. So the chain above can't be
  strictly one-way without splitting the hub. Decision: **strict at the bottom**: `core/`
  imports nothing above it and `terminal/` only core. The hub lives in `server/`, and the top
  tier (`server/`, `net/`, `lumasession/`, `protocols/`, `engines/`) may use each other. The
  user added: "We need it to be strict but also flexible enough so we can do thin build for
  small systems", so `lumasession/`, `protocols/` and `engines/` never import `terminal/`.
  Only the shell server and its HTTP bridge (`net/shell.rs`, `net/console_bridge.rs`,
  `net/sync.rs`) host terminals.

### Step 8 built (2026-10-11)

- **Crates**
  - `lumacore-core` → `lumacore-base` everywhere (history in this file keeps the old name).
  - New `lumacore-trace`: `TraceCollector`, `PerfettoGuard` and the JSON export, with its
    tests. `lumacore-dmx` `perfetto` = `dep:lumacore-trace`, and http depends on trace.
  - New `lumacore-wire`, depending on serde, serde_json and socket2. socket2 is for the
    client socket helpers: `send_request_to`, `try_health`, `bind_discovery_listener_at`.
  - The wire crate holds `SESSION_PROTOCOL` 3 (was `STREAM_PROTOCOL` 2) and
    `CLIENT_PROTOCOL` 1. `ShellClientPacket::Hello` / `MonitorHello { protocol }` default
    to 0, so a client older than the field is refused with E-WIRE-001 and not misread.
  - `RemoteReply` and the beacon carry `kind` (`StationKind::Server`), `client_protocol` and
    `session_protocol`, and `RemoteReply` also `clear_screen`. mDNS TXT gets `kind` and
    `client`.
  - `Status` is typed: `ClaimStatus`, `ClaimAsk`, `ShowStatus` (the `Info Show` /
    `backup::show_info` data), `LumaSessionStatus`, plus `Decision` and `StartupShow`. The
    JSON is unchanged for the frontend.
- **Clients**
  - `apps/lumacore-cli`: wire, clap, crossterm and nu-ansi-term. **51 deps, down from 179.**
  - `apps/lumacore-scripts`: crossterm and ratatui.
  - `apps/lumacore-app`: wire, wry, tao, muda and mdns-sd. **145 deps, down from 286.**
  - The app starts `lumacore-server --web-root <frontend>` found next to its own binary, and
    names the build command when the server is missing. `--server` is gone.
  - The home server list greys out "Wrong version" stations (beacon / mDNS client protocol).
  - `lumacore-http`'s `/ws/shell` bridge sends and reads `lumacore_wire` packets; its
    `ServerMsg` copy and hand-built JSON are gone.
  - The e2e sysmon helper sends `protocol` too, as `CLIENT_PROTOCOL` in
    `tests/e2e/support/server.ts`.
- **Station layout** (`crates/lumacore-station/src`)
  - `core/`: was `terminal/` minus the REPL, plus `luma`, `errors`, `panic_report`,
    `session_log`, `station_config`, `feedback` and `sysmon`.
    - `text` was `renderer`.
    - `build_info()` and `DispatchResult` moved here.
    - New `protocol_plan`: `desired`, `line_states`, `LineState` (pure; read by `List
      Output` / `Info Output`). The driver runtimes stay in `protocols/`.
    - New `session_link`: role, `ShowStatus`, split base, `LinkHandle` whose `tx` is the
      **`LinkOut` outbox**, forwards, `ReplyTo`, `LinkRequest`, `SessionRequest`, the status
      words and `forward` / `submit_changes` / `leave`. Also the commands `Claim` / `Pause
      LumaSession` and `Delete Station`, and the autosave copy.
    - The key-name grammar `keys::parse_key_name` uses wire key codes; the TUI maps them to
      crossterm.
    - The fixture and group completers moved into their objects.
  - `terminal/`: `repl/`, `completer`, `session_manager`.
  - `net/`: shell server, command port, discovery, mDNS, menu views, console bridge,
    handler, sync.
  - `lumasession/`: `mod.rs` (invite, dismiss, `ServiceAccess`, `Deferred`), `link`,
    `election`, `stream`, `replication`. The stream writer turns `LinkOut` into `StreamMsg`.
  - `server/`: `app` (CLI flags), `context` (`ServerContext`, plus `sysmon(ctx, …)` and
    `drain_sysmon_pending`), `state`, `mod`.
  - `tidy.rs` is at the crate root.
  - The `remote::` re-export shims are gone, and every path names its module.
- **Invite / Dismiss**: the command queues `SessionRequest::Send`. After the job,
  `lumasession` sends it on its own thread, and the other station's answer goes to the
  line's client (`ReplyTo`). This closes the "network I/O in a job" exception.
- **Tidy**: `layers_only_point_down` covers the layer table, the terminal hosts and bare
  `core::`. `scanner_finds_crate_modules` tests the scanner.
- **build.rs** reruns on `HEAD` / refs / src only.
- **Tests**
  - Wire 3, trace 18, http 31, engine registry 5 — all pass.
  - Station lib: 386 pass. The only failure is the known `net::tests::shutdown_command_sets_should_exit`.
  - Live on a `--station A` server:
    - Health, `Info Show` and `clear` work from the new CLI.
    - The shell and monitor hellos without a protocol, or with protocol 9, are refused with
      E-WIRE-001.
  - e2e: 53 pass. 3 fail, and they fail on `c2b99bb` too:
    - the Shutdown-menu Logout tiles
    - `List UIKey` output
    - feedback "Highlight On" — that test still uses the pre-step-3 syntax and the
      station-wide HI status.
- **Docs**: `crate-map.md`, `architecture.md` and CLAUDE.md.

### App ↔ local server hardening (2026-10-11)

What went wrong: a pre-step-8 `lumacore-app --server` was still running. The new app found
something answering on `127.0.0.1:47656` and attached to it without checking anything. The
current frontend then talked to the old backend, so the group window stayed empty while the
terminal still listed the groups. Shutting it down from the app didn't work either.

Retested on the current build, Shutdown works in under a second on every path: the command
port, `/ws/shell` and with the demo show loaded. The real gaps are the unchecked attach and a
server that ignores signals: `kill` or a closed terminal stops it with no goodbye and no
autosave.

- **The app never attaches to a local server silently.** `/api/station` adds
  `client_protocol`, `pid`, `exe` and `started_ms`. The local server counts as **another
  version** when any of these holds:
  - its client protocol differs, or is missing (a server from before step 8);
  - its `build` differs from that of the `lumacore-server` next to the app
    (`lumacore-server --version`);
  - it runs that same file, but the file was rebuilt after the server started. The build id
    doesn't change between uncommitted rebuilds, so this check is needed too.
- **Pop-up when it is another version:** "Another LumaCore server is running on this
  machine (build X, pid Y)".
  - Buttons: **[Shut it down and start mine]**, **[Use it]** and **[Cancel]**. [Use it] is
    offered only when the protocol matches.
  - Shutting it down asks Save / Don't Save when the show has unsaved changes, like the home
    screen's stop.
- **The server handles signals.** SIGTERM, SIGINT and SIGHUP work like `Shutdown /NoSave`:
  - unsaved changes are autosaved first (`.autosave/{show}_signal_{station}_{time}`);
  - outputs send their goodbye;
  - every thread joins against `SHUTDOWN_DEADLINE`.

  A second signal exits at once. A terminal Ctrl-C now stops the server cleanly too.
- **Hung stop:** if the server is still answering 5 s after `SHUTDOWN_DEADLINE`, the app
  shows "The server did not stop" with **[Force stop]** and **[Keep waiting]**.
  - Force stop sends SIGTERM, then SIGKILL after 5 s.
  - It applies only to a pid the server reported itself, on this machine, whose executable is
    a LumaCore server.
  - The app never kills a process without that click.
- **Damage it did:** the old server saved over `library/shows/demo.luma` as format 4. The new
  build refused that file (E-SHOW-002) and started empty. The file was regenerated with
  `gen-demo` (2026-10-11).
- **Save guard:** before writing a show file, the server reads the existing file's
  `format_version`.
  - If that format is newer than its own, it refuses with a new E-SHOW code and saves to
    `{show}_format{N}_{time}.luma` instead, so nothing is lost and the newer file stays intact.
  - A prompt then asks whether to **overwrite anyway**. That needs an explicit "really
    overwrite"; the one-shot form is `/Overwrite`.
  - Binaries from before this guard don't have it; the stale-server prompt covers them.
- **Demo test:** gen-demo's builder moves into the library (`demo::build()`); the `gen-demo`
  binary only writes its result. One test:
  - builds the demo in-process;
  - saves it to a temporary file;
  - loads it through the real load path (format check, GUIDs, references, `after_load`);
  - checks that the committed `demo.luma` loads and matches the generated show, apart from
    GUIDs and timestamps. Otherwise the test fails with "demo.luma is stale — run `cargo run
    -p lumacore-station --bin gen-demo`".

  It replaces `identity_the_demo_show_loads_clean`, which only parsed the checked-in file.

### Step 9b built (2026-10-11)

- **Station**
  - `/api/station` gives `client_protocol`, `pid`, `exe` and `started_ms`.
  - `lumacore-server --version` prints the build id.
  - `server::watch_signals` is a `signals` thread using `signal-hook` (unix only).
    - On the first signal, `shutdown_on_signal` runs on the state thread:
      `autosave_copy(…, "signal")`, a Critical "Server shutting down: by SIGTERM" message
      and `shutdown_requested`.
    - On the second signal, it exits with code 130.
- **Save guard** (`backup::save_show_as`)
  - It peeks at the existing file with `file_format`. A newer format gets the copy
    `{name}_format{N}_{YYYY.MM.DD_hh.mm.ss}UTC`, which becomes the loaded show, so autosave
    and Quick Save don't pile up copies. The message carries E-SHOW-004.
  - `SaveShow` then asks "Overwrite Show": a pop-up in a UI, yes/no in a terminal.
    `SaveShow "x" /Overwrite` writes over the file at once.
- **Demo** (`core::demo`)
  - `build`, `check`, `normalized` (GUIDs → `guid#n` in order of appearance) and
    `repo_root` / `demo_path`.
  - `gen-demo` is a thin writer that fails when a command line complains or a check fails.
  - The test `the_demo_show_builds_loads_and_is_committed` replaces
    `identity_the_demo_show_loads_clean`.
  - It found that `Login LD` no longer creates a user (§19), so the demo now runs
    `Store User "LD" /Role Programmer`. `demo.luma` was regenerated with the LD user.
- **App**
  - `local_server::other_version` checks protocol, build and rebuilt binary.
  - `start_local` asks via `home.confirmOther`: Cancel, Use it (only when the protocol
    matches), and Shut it down and start mine (Save / Don't Save when the show has unsaved
    changes).
  - Opening `127.0.0.1:47656` from the list goes through the same check.
  - Every stop of the local server (home Shutdown, Quit from a screen, replace) runs
    through `stop_local_then`. If the server is still there after `STOP_WAIT` (10 s),
    `home.confirmHung` offers Cancel, Keep waiting and Force stop.
  - `local_server::force_stop` checks the process name with `ps`. It sends SIGTERM, then
    SIGKILL after 5 s.
- **Login tokens** (found by the e2e Logout-tile test)
  - Tokens were `v1.<user>.<expiry s>.<fp>.<sig>`, so a login in the same second as a
    Logout got back the revoked token and every call was refused (401). The page then
    hung with a shell but no status.
  - Tokens are now `v2.…<nonce>.<sig>` (UUID v4 nonce); v1 tokens no longer verify. The
    security test that slept 1.1 s to step around this now checks a same-second re-login.
- **Leftovers**
  - `scripts/unix/start_developer` and `start_server` build `-p lumacore-station --bin
    lumacore-server` and `-p lumacore-cli` separately, and judge the build by cargo's exit
    status, so a stale binary no longer passes. `start_server` builds with `http`.
  - e2e:
    - Highlight is per user (`On Highlight`; only that user's UI shows HI).
    - `List UIKey` / `Info UIKey` use the `Press:` / `DoublePress:` names.
    - The reconnect test loses the server with `Server::crash()` (SIGKILL), since SIGTERM
      is now an announced shutdown.
    - Token tests expect `v2.`.
- **Tests**
  - Station lib 389/389, app 2, http 31, wire 3. Clippy is clean.
  - e2e 68 passed, 0 failed (before: 53 + 3 stale failures + 12 that didn't run).
  - Live on `--station B`:
    - `/api/station` fields.
    - SIGTERM with unsaved changes → autosave + clean stop in 0.8 s.
    - SIGINT, SIGHUP.
    - A second SIGINT → immediate exit.
    - Save over a format-99 file → copy + E-SHOW-004; Quick Save goes to the copy;
      `/Overwrite` replaced it.
  - The app's dialogs (another version, hung stop) are covered by unit tests of the
    version check. The GUI flow itself is for the operator to try.

## 24. Render Thread Rules (2026-10-10)

- Audio-style strictness in the 44fps tick: no allocation, no locks, no logging (checked with
  `assert_no_alloc` in debug). Show changes compile into a flat render plan off-thread;
  frames are preallocated; clock stats use atomics; the thread is supervised.
  *Rate superseded by "Step 9 hardening": the engine renders at 60 Hz.*

**Step 9 hardening (2026-10-11)** — researched against the code (render path survey), a reference console
(manual, live tree, System Info windows: `intel/Screenshot 2026-10-11 at 19.02.34.png`), the
Art-Net 4 spec and audio/game engine practice. One step.

What the reference console does:

- A realtime loop at ~60 Hz (`Root.REALTIMEITERATION` +60/s); DMX leaves at ≤ 30 Hz, an
  unchanged universe may slow to 1 Hz.
- The patch compiles into dense tables on change (`LivePatch.RTChannels` 35,451 rows with
  resolved addresses, default/highlight/lowlight, curve; `DmxAddresses` 1024×512; dense
  0-based `ATTRIBUTEINDEX` with `…INDEXOLD` kept so running state survives a rebuild). The
  rebuild (`LASTTRUDURATION` ≈ 169) runs only on change.
- System Info "Realtime" lists each stage's ms per frame on a frame timeline: Start,
  DMXDecompress, DMXMix, Jobque, SendRTStream, LUA, GroupMaster, Timecode, Playback,
  Highlight, **CalcChannel 2.69 ms** (35k parameters ≈ 76 ns each), Wait1, DMXCompress,
  DMXTransfer, PostRealtime, Wait2 (idle 4.85 ms), ChannelCompress, LED. "Timing" is the
  separate UI loop. Overload lowers the DMX rate and shows "Overload" in the status bar.
- Protocol pacing is per protocol/row (Packet Delay between universes, Output Delay 0–30 ms).
- Art-Net 4: resend unchanged data every 800–1000 ms (to match sACN); nodes report their max
  rate in `ArtPollReply.RefreshRate`; ArtSync after a frame for synced output.

Decided:

- **Render rate: fixed 60 Hz** (like a reference console's loop). The tick is 16.67 ms on an absolute deadline
  (`spin_sleep`, highest thread priority — §31: real `SCHED_FIFO` 70 where the limits file grants
  it, `Info Health Threads` says what each thread got).
- **Rates live on the Line.** Each output line (protocol + network + target, show data,
  replicated) gets `Rate` (Hz, 1–60, **default 30** = every 2nd frame, even sampling; above
  44 allowed for pixel/LED nodes but never above a node's reported RefreshRate), `Send`
  (`Full` default | `Changes`: changed universes at once, keep-alive every 1 s, 3 repeats
  before idle), `PacketGap` (µs between universes, for old nodes; 0–2000, and the gaps of all
  lines on one driver never take more than half of the fastest line's period, R1-54 Q21) and
  `Sync` (ArtSync / sACN
  sync after the frame). The engine renders once; each line's driver has its own timer and
  takes the newest frame (§31: lines whose Rate divides 60 lock to the engine's frames after a
  doubled/skipped counter has measured it) — a line can never be faster than the render. Other protocols plug
  in as lines with their own rates.
- **Render plan: a compiler thread, two tiers.** A `Compile` thread takes each published
  show (bursts coalesce, newest wins):
  - *channel tier* — patch → dense tables (fixture/attribute indices, DMX channel slots with
    8/16/24-bit layout, defaults, highlight, curves, universe offsets) plus an old→new index
    map so running state (fades, effect phase) survives; rebuilt only when the patch (or
    fixture types) changes;
  - *value tier* — programmers, running cues' tracked values, park, masters, modes,
    group-master fades, all addressed by those indices; rebuilt cheaply per change.
  - The engine swaps plans at frame start (ArcSwap) and never frees one: retired plans go
    back to the compiler thread to be dropped. The state thread never waits on a compile.
    Plans carry the show generation; a stale plan is discarded.
- **The tick** only walks arrays: no maps, no strings, no allocation, no lock, no logging,
  no `SystemTime` (time = the session clock). Frames, owners/winners and the output snapshot
  are preallocated (double-buffered); fades are evaluated per tick from the plan.
- **Commands read the engine's output snapshot** (final value + winning source per
  attribute, published each frame). A command that must see its own change waits for the
  frame carrying its show generation (≤ 17 ms). `AppState::resolved_output` /
  `published_frame`'s state-thread renders go: the merge runs only on the engine.
- **Panic:** each tick runs in a panic guard. On a panic the lines keep sending the last good
  frame (no blackout), the panic is reported like a command panic (report #, file:line,
  sysmon, `Health.panics`) and the engine restarts with a fresh plan. A second panic within
  10 s leaves it down, holding the last frame, and the status says so.
  The **plan compiler** (R1-66, Q39 — after OTP restart intensity, Chromium's GPU fallback, Ardour's
  graph swap): a panicking compile is caught, the last plan stays on stage (never a blackout), that
  input isn't retried, `stale_since` latches the first change the stage doesn't show (Health, System
  Monitor) until a compile works; 3 panics within 60 s stop the compiler until a show load or a restart.
- **Overload** (a frame > 16.67 ms): never catch up and never slow time — the next frame
  renders at the real session-clock time, lines keep their rate with the newest frame. Late
  and skipped frames are counted per stage; "Overload" shows in the status bar and sysmon
  once per episode.
- **Clock: session clock over the stream.** One monotonic engine clock (`Instant`). Members
  learn the master's clock offset from the LumaSession stream heartbeat (500 ms, round trip
  measured, smoothed) and render in session time; a new master continues the session time
  it already had (no jump). Fades, group-master fades and (§28) effect anchors are stored in
  session-clock ms, never wall clock. Standalone = local monotonic clock.
  *Refined by "Session clock (researched)" below: the pings move to UDP and the clock is
  rate-disciplined.*
- **Session clock (researched 2026-10-11)** — after a study of Ableton Link, Snapcast, nqptp,
  PipeWire, FPP MultiSync, RFC 5905, chrony, ntpd-rs, statime, linuxptp and game netcode
  (Unity NGO, Mirror, Source):
  - **Rate overlay, never backwards** (statime `OverlayClock`): session = base + (Instant −
    anchor) × (1 + frequency + correction), re-anchored on every change so it stays
    continuous. Correction = −offset / 2 s, clamped to ±0.5 % (up to 2 % while more than
    20 ms off); frequency (drift) clamped to ±500 ppm. A member that is ahead slows down —
    session time never goes backwards while anything plays. One step only before the first
    render after joining, or beyond 1 s (panic, with a sysmon line).
  - **Pings on the UDP command port** (:47653, per-NIC filtered like every request): a small
    Time request/reply with four timestamps in session time, stamped at the socket (t2 on
    receive, t3 just before the reply). Never queued behind snapshots; the stream heartbeat
    stays for liveness and election only.
  - **Filter:** a burst of 16 pings 20 ms apart on joining or on a new master (locked in about
    0.5 s), then one every 500 ms. 32 samples kept; samples with RTT above min RTT + max(0.5
    ms, ½ min RTT) are dropped, and a single 5σ RTT outlier (the second in a row is kept, as
    in ntpd-rs); offset = median of the survivors; frequency = RTT-weighted linear regression
    over the window (chrony). A step, a gap or 3 outliers in a row start a new burst.
  - **Handover / holdover:** on master loss every station free-runs at its last frequency
    (Holdover); the elected master keeps its time; members drop their samples, keep their
    frequency, burst and slew onto it (linuxptp keeps its servo across a grandmaster change).
  - **States and stats in PTP/NTP terms**: Unlocked / Acquiring / Locked / Stable (10 samples
    within 1 ms) / Holdover; offsetFromMaster, meanPathDelay, jitter, frequency (ppm), skew,
    correction (ppm), samples used / rejected, last step (time, size), steps, master and
    epoch — per station and, on the master, per member (members report in their pings). In
    `Info Clock`, `List Health` and the Realtime window.
  - For effects (§28, another session): Ableton Link's timeline `{tempo, beat origin, time
    origin}` with phase per quantum keeps the beat continuous across tempo changes and
    rejoins — noted for §28, not built here.
- **Leftovers go:** `LumaRuntime` (lanes, envelopes, plan/commit metrics), `HealthReport`,
  `MasterClock`/`SharedClock`, the beacon clock sync (`ClockSyncWire`, `precision_clock`)
  and `benches/luma_runtime.rs`. `Runtime.Health` is the only health source.
- **Health for nerds** (`Runtime.Health`, atomics only):
  - `Render`: per-stage µs per frame (Inputs, Plan swap, Playback, Fades/Effects, Merge,
    Masters, Overrides, Channels, Input merge, Publish, Idle), frame time, jitter,
    late/skipped frames, headroom, panics, plan generation and compile time.
  - Per line driver: send time, actual rate, packets, errors.
  - Clock per member (`Runtime.LumaSessions`): offset, round trip, jitter, drift, last sync,
    state — PTP-style from our stream sync.
  - **Real IEEE 1588 PTP** read-only from linuxptp when present (ptp4l management socket:
    offset, path delay, grandmaster, port state); elsewhere "not available". Never configures
    PTP.
  - Shown by `List Health`, `Info Health Render`, and a **Realtime window** (window type
    `realtime`, `/ws/health` at 4 Hz: stage bars on a frame timeline, frame time, headroom,
    late frames, line rates, clock offsets).
  - The March developer attach stays: **Tracy** (frame marks, a zone and plot per stage, clock
    offset plots) and **Perfetto** (:47657). `trace_zone!` is fixed (its guard dropped at once,
    so it measured nothing).
- **Enforcement:** `assert_no_alloc` in debug builds around the tick and the drivers'
  per-frame send (an allocation panics → the panic path); a criterion bench + test: a
  generated 35k-parameter show with running cues, fades and programmer renders < 3 ms per
  frame in release; a tidy rule fails on `Mutex`/`RwLock`/`log!`/`format!`/`String`/
  `Vec::new`/`HashMap` in the render and driver per-frame modules
  (`// tidy: render-ok (<reason>)` escape).
- **Drivers** follow the same rules: preallocated packet buffers, no per-frame collections,
  report to `Runtime.Health`.

**Step 9 built (2026-10-11).**

- **lumacore-dmx:**
  - `plan` (`ChannelTier` / `ValueTier` / `Plan`; the channel tier reused while
    `EngineInput::patch_generation` stands);
  - `compiler` (the `plan_compiler` thread: coalesces, builds a `Bundle` of plan + 6 frame
    buffers, drops `Retired` plans / frames / input maps the engine hands back);
  - `render` (`RenderedFrame`, `render_into`: Channels with group masters fading by the session
    clock, grand, Solo; Park; InputMerge — allocation-free; `render_now` for engine-less use);
  - `engine` (60 Hz, `PlaybackHub::publish` / `wait_rendered`, panic hold + restart / down after
    two in 10 s, late frames skipped and counted, stage timing; warm-up of ArcSwap's
    per-thread slot before the checked loop — the engine test caught that allocation);
  - `lines` (per-line `Rate` / `Send` / `PacketGap` / `Sync`, node `RefreshRate` cap, `LineStats`,
    driver thread health; sACN and Art-Net packets rewritten in place, E1.31 sync to universe
    63999, ArtSync);
  - `bench_show` + `benches/render.rs`.
  - `master_clock.rs` is gone; `playback.rs` keeps only the merge.
- **lumacore-base `clock`:** the seqlock overlay clock (rate ±5 %, `set_rate_ppb`, `step_us`).
- **lumacore-runtime `render_stats`:** `RenderStats`, `LineStats`, `ClockSync` (process-global),
  `MemberClock`, `PtpStatus`. `HealthBoard::view()` (`HealthView`).
- **lumacore-wire:** `RemoteRequest::Time`, `TimeReply`, `ClockReport`. The removals bumped
  `CLIENT_PROTOCOL` → 2 and `SESSION_PROTOCOL` → 4; `ClockSyncWire` is gone.
- **Station:**
  - `AppState::{output_frame, output_plan, patch_generation, engine_generation}`; the
    patch generation is bumped in the change path (`Devices`), loads, undo/replicated restores
    of `Devices` and Discard.
  - `RenderWatch` / `ClockWatch` on the state thread.
  - `lumasession::clock` (Time pings answered in the UDP thread, the filter, the discipline,
    holdover).
  - `net::ptp` (pmc reader).
  - `Info Clock`, `List Health` / `Info Health Render|Lines|Clock|Threads`;
    `PrecisionClock` removed.
  - Output props `Rate`, `Send`, `PacketGap`, `Sync`; group-master fades on the session clock.
  - The debug allocator in `lumacore-server`.
  - The tidy render rule.
  - Window type 6 `realtime` + `/ws/health` + the Realtime window.
  - `trace_zone!` fixed in both crates (zones on state jobs and plan compiles).
  - Show format 5; demo regenerated.
- **Leftovers removed:** `LumaRuntime` and friends, `HealthReport`, the beacon clock and
  precision clock, `benches/luma_runtime.rs`, `tools/benchmark_luma.sh`,
  `tools/check_metrics_regression.sh`. `CommandClass` moved to the registry.
- **Tests:**
  - station lib 389/389 (the old `shutdown_command_sets_should_exit` expected a text
    `shutdown` over UDP to stop the station; it asks for confirmation since step 3, so the
    test is now `shutdown_command_over_udp_asks_first`: refused, nothing stops);
  - dmx 22 (+1 release-only: the 35k frame under 3 ms passes);
  - base 53, session 5, tree 58, wire 3, http 31, engine registry 5;
  - tidy 8 (incl. the render rule).
  - e2e 53 passed + the same 3 known-stale failures as step 8 (Highlight feedback syntax,
    Shutdown-menu Logout tiles, `List UIKey` text), 12 not run.
  - Clippy is clean.
- **Live (two stations, port bases 48660 / 48670):**
  - Rendering:
    - 60 frames/s with 15.7 ms headroom;
    - stages listed;
    - sACN on loopback: 30 Hz per universe at Rate 30, 44 at Rate 44, Changes = repeats +
      1 s keep-alive, Sync = one sync packet per frame to 63999.
  - Session clock:
    - B joined A and was Stable in 3 s (offset +63 µs, path delay 59 µs, no step);
    - after 17 s the frequency was +1 ppm (skew 0.9), offset −17 µs;
    - A listed B's report;
    - A killed → B in Holdover, then master; no step, session time continuous, no late frame.
- **Known gaps:**
  - the input receive path still allocates per packet (only the send is checked);
  - the Realtime window was syntax-checked and its feed built from the same view as
    `List Health`, not looked at in a browser;
  - a member's Time pings go to the master host from the link target (an unresolvable name
    gives no sync);
  - PTP readout untested on Linux here.

## 26. Engine Ownership, Errors and Robustness (2026-10-10)

### One engine thread owns state

- A single engine thread owns show and runtime state. Shell, UDP, HTTP, OSC and replication
  send it commands through a queue and get replies; readers use published snapshots
  (ArcSwap). No global `Mutex<ServerContext>`, and no network I/O inside the engine — a slow
  client or HTTP request can no longer freeze the server. Inputs feed the queue directly (no
  250 ms read-timeout delay).
- Each command runs inside a panic guard: a panic rolls that command back, the engine keeps
  running, and the reply points at the code:
  `Internal error in <file>:<line> — <message> · command · user · build · report #n`
  (full backtrace in the log under that report number). `hissme` keeps its deliberate crash.

### The tree: read everything, write through one door

- Everything in the tree is **readable** by users, plugins and HTTP — LumaCore shows its
  internals. In code, every **write** goes through one change path (`root.change(…)`) that
  notifies, records undo and replicates automatically; the hand-written `affected_subtrees`
  list goes away.
- `Set <tree path> <property> <value>` works on any property the tree marks writable (same
  path: rights, claims, undo, replication). Read-only and derived properties show in `Info`
  and refuse `Set` with a reason.
- Live internals in the tree: the change log/events (`Watch Events`, `List Journal`), every
  user's undo stack, and engine/thread health (frame time, missed frames, queue depth,
  outbox, resends). ID pool counters stay internal.
- `ShowData::after_load()` rebuilds ID pools on every load path (file, join, merge, restore).

### Step 2 details (hardened 2026-10-10)

- **Threads.** The *state thread* owns show and runtime state and runs commands one at a
  time. It is not the DMX thread: rendering/output keeps its own threads, HTTP its own tokio
  runtime, and shell / UDP / OSC / discovery do their own I/O and send jobs to the state
  thread.
- **Readers.** After every change to ShowData the state thread publishes one frozen
  `Arc<ShowData>` (ArcSwap). HTTP reads only that snapshot and never waits on the state
  queue; only HTTP commands and logins go through the queue, and they wait at most 10 s (then
  503 / E-CMD-017; a shell waits 30 s and queues its keys, lines, taps and answers in order;
  a watchdog warns after 5 s on any turn of the loop — R1-67, Q40). Programmers will get their own
  small published snapshot so an `At` does not republish the whole show (wired up with the
  playback engine, §27; not built yet — first step of §31's snapshot work).
- **Change path = automatic diff.** Dispatch fingerprints every ShowData subtree before and
  after each mutating command; exactly the changed subtrees get the undo frame, the
  replication op and the change notification. Handlers are not rewritten; the hand-written
  `Undo::subtrees` lists go away (the registry keeps only "mutating + undo category").
- **Panics** roll back ShowData only (the changed subtrees, from the change path's
  pre-image); Runtime/station state keeps what the command did. Every user sees the full
  one-liner (file:line, command, user, build, report #); the backtrace goes to the log under
  the report number. A repeating panic is just reported again (no quarantine).
- **`Set`.** Each tree node type declares its writable properties (name, type, range,
  required right) in one table next to the type; everything else is read-only and `Set`
  refuses with the reason (derived, read-only, needs right).
- **Journal.** `Runtime.Journal`: ring of the last 1000 changes — time, (epoch, seq), user,
  station, command line, changed subtrees, result (ok / error code / internal error report
  #). Not saved; the sysmon log keeps the full history on disk. `List Journal`,
  `Watch Events`.
- **Health.** `Runtime.Health.Threads.{State, Render…, Http, Shell, Udp, Osc, Discovery}`:
  alive, last activity, loop/frame time, queue depth, panics. Undo stacks readable under the
  user.
- **Shutdown deadline** 5 s: stop in order (inputs → state thread → outputs say goodbye →
  HTTP → listeners), join with a shared deadline, name anything still running in the log.

### Errors

- Typed errors per crate (`thiserror`), each with a stable code (e.g. `E-PATCH-012`) and the
  source location; the terminal shows message + code, `Info Error <code>` explains it.
- Format `E-AREA-NNN` (area = crate or subsystem, numbers never reused), e.g.
  `Error: Address 0 is not a DMX address (1–512) [E-PATCH-012]`. Step 2 builds the code
  catalog, `Info Error`, codes on the crate errors (Patch, DMX, GDTF, Output, Universe) and
  on internal errors; handler string errors convert as their areas are touched.

### Bugs and robustness

- Server startup: one `ServerBuilder` for `lumacore-server` and the app's embedded server;
  HTTP becomes a normal `Engine`; one shutdown path.
- Shutdown: sACN sends stream-terminated packets, Art-Net stops; engines stop in order,
  threads are joined with a deadline, listeners really close.
- OSC (and any input) may shut down if its configured user has the right; the ignored
  `break` is fixed.
- Art-Net: one sequence per universe, a failing universe never skips the others, fixed send
  order, unicast to ArtPoll-discovered nodes with broadcast as fallback.
- Discovery listener stays blocking (no busy-spin). Patch address 0 is rejected (DMX 1–512)
  and the renderer uses checked arithmetic.
- The TCP shell trusts a client-reported address only from loopback bridges, so
  `remote_login` is enforced. No connection limits (isolated networks).
- UDP carries only small messages; snapshots, op batches and merge data go over a
  station-to-station TCP stream (length-prefixed, compressed, versioned).
- Command line: one tokenizer with spans, unterminated quotes are errors, completion handles
  quoted names, ambiguity checks fold aliases of the same command.

### Cleanup

- One `cargo fmt` commit, clippy warnings fixed (no reason-less `allow`), committed temp
  shows removed, `inspiration/` and `issues/` ignored, dependency versions moved into the
  workspace, the `unsafe localtime_r` replaced with chrono, the broken `lumacore-http` tests
  and Art-Net sequence fixed, CLAUDE.md and `crate-map.md` corrected.

## 27. Playback Engine (2026-10-10)

Researched against a reference console (manual + live console): tracking, cue parts,
priorities, phasers, programmer layers (`Temp/ProgLayers`), runtime state
(`Sequence.CURRENTCUE`, `Temp/LoadedPlaybacks`).

- **Pipeline.** State thread publishes the show snapshot (ArcSwap) → one **playback engine**
  thread at 44 fps (tracking, fades, phasers, priority merge, masters, Highlight/Solo, patch
  to DMX channels, Park) → one frame per universe → sACN / Art-Net output threads only send.
  It replaces today's three renderers. The engine also publishes a read-only output snapshot
  (final value + winning source per attribute, like a reference console's cue and output layers) — derived,
  never saved or replicated.
- **One value type.** Presets, programmer and cues store the same `Value`: normalized /
  physical, with steps (1 step = static, ≥2 = phaser: absolute, relative, width,
  transition, accel, decel; per value speed, phase, measure) and an optional preset
  reference. DMX bytes exist only at patch output. *Superseded 2026-10-11 by §28: the phaser
  fields go, a value is static; dynamics are effects.*
- **Merge order** (a reference console's documented order): external input (sACN/Art-Net/DMX In) >
  DMX tester > Park > Super playbacks > frozen programmer > playbacks Swap > HTP > Highest >
  High > LTP > Low > Lowest > unfrozen programmer. Within a band: HTP = highest intensity,
  everything else LTP (latest wins). Then output modifiers grand → world → group masters,
  then selection overrides Highlight/Lowlight/Solo (above Super and the programmer, never
  above Park). Blind removes the programmer layer only.
- **Tracking** (manual `cue_tracking`): a cue stores the programmer's active values; one equal
  to what already tracks into the cue is kept as a *blocked* value (it doesn't change the
  output, but it stays when an earlier cue changes). A value tracks until a new value or a
  Release (cue column; the OffCue releases by default). Released attributes go to lower
  sources, else to defaults.
  Tracking off = tracked values become releases. Cue Zero holds defaults. Goto asserts
  tracked values (cue `Assert`: None/Assert/X-Assert decides whose timing).
- **Cues have parts** (timing and values live on the part); presets can route into a part.
- **Programmer** (manual `operate_programmer`): three levels — selection, active values,
  inactive values; both active and inactive values output, only active ones store. Clear
  steps through them. Freeze lifts it above all playbacks except Super. Program Time fades
  programmer changes (ignored entering/leaving Blind). Preview is a separate programmer
  with no output.
- **Timing precedence**, low → high (manual `cue_timing`): cue time → feature group time →
  individual attribute time → executor time → rate.
- **Running playbacks** record who started them and what triggered them (manual
  `executor_running_playbacks`); readable and filterable per user.
- **Runtime playback state** (current/loaded cue, started by, faders, rate, paused) lives in
  Runtime: session-wide, replicated, not saved.
- **Answered 2026-10-10:** programmer levels as established (inactive values still output, only
  active values store). Cues have parts from the start (skeleton uses part 0). Preview
  programmer later, **one per user** (not one shared programmer). Priorities stay the **§7
  Priority pool** (`DataPool.Priorities`, user-defined and reorderable) — the reference's fixed bands
  are the default content of the pool, not a hard-coded list.
- **First version (step 4):** the engine skeleton with static cues + tracking (Go/Goto,
  release), programmer layers of every logged-in user, Freeze/Blind, Park, Highlight/Solo.
  Fades, timing, phasers come after; programmer effects get their own phase later.

**Step 4 hardening (2026-10-10, a reference console's manual + live console)** — step 4 is split in two:

- **4a — engine:**
  - **Value**: the full shape now — `steps` (only step 1 renders), absolute
    Normalized/Physical, relative, preset reference, release — in lumacore-tree. Programmer
    (`ProgValue` u8), cues (`CueValue` u8) and presets (`AttributeValue`) all switch to it.
    Physical/normalized everywhere; DMX bytes only at patch output, 16-bit from the start.
  - **Model:** Sequence → Cue → Part (part 0 always exists; values and timing on the part).
    The old `SequencePart` and `SequenceObject.tracks` go.
  - **Commands:** `Store Cue` (every active value; one equal to what tracks in is stored as a
    *blocked* value — R1-31, Q16 b), `Go`, `Go-`,
    `Goto`, `Off Sequence` (snap, no fades), `At Release` in cues (OffCue releases all).
    Executors later.
  - **Playback state** in `ShowData.Playbacks.Sequences` (§13), resumed snapped on load.
  - **Programmers** of every user render, LTP with each other and with playbacks, values
    move between programmers (§4); inactive values output too. Freeze as decided (§7).
  - **Masters:** grand master (`Master 1`, level + Toggle on/off = blackout, level kept),
    applied after the merge; the `Black` command goes. Group masters in step 7 (§22);
    speed/mode masters later.
  - **Solo** per user from their live selection (others' dimmer 0, after masters, never above
    Park). **Highlight later** — it uses the GDTF highlight values; `On Highlight` only sets
    the mode. The old global overlays go.
  - **Park** stores Values per fixture attribute: bare `Park Fixture 3` parks every attribute
    at its current output; `Park Fixture 3 At 50` parks Dimmer (At = intensity);
    `Park Fixture 3 Pan At 30` one attribute. Raw channel park (`Park DMX 1.101 At 255`,
    for unpatched channels) **stays** alongside fixture Park, saved with the show. Park is
    not related to the DMX tester (a separate, later feature).
  - One engine replaces the three renderers (sends with today's defaults until 4b).
  - `Clear Programmer User …`; fixture `connection` slot + `Runtime.Devices` reserved (§11).
  - Small items: the command bar's history button is renamed **PURR** (Meow is only the
    modifier); `list_tree.rs` overviews are generated from the real tree.
- **4a built** (2026-10-10): `lumacore_core::Value`, `lumacore_playback` (Sequence → Cue → Part,
  tracking, `PlaybackStates`, `ParkData`, grand master), `lumacore_dmx::playback` (the merge,
  pure — commands call it too) + `lumacore_dmx::engine` (`PlaybackHub`, one 44 fps thread;
  `PlaybackEngine` registered first, sACN/Art-Net attach), `terminal/programmers.rs` (LTP,
  transfer, claims, `Clear Programmer User`), `commands/playback.rs`. Known gaps: the engine
  reads the whole published show (no separate programmer snapshot yet); the frame is
  recomputed on the engine thread with allocation when the input changes (step 9); a sequence
  without `Sequence <n>` is Sequence 1 (selected sequence/executors later); all sequences are
  LTP (Priority-pool bands, HTP, Super later); Store Cue merges into an existing cue (the reference
  asks); Clear Programmer User's undo frame goes to the clearing Admin; sequence fader/rate
  not applied; `DMX … At` overrides are stored, not output (DMX tester later). sACN now binds
  an ephemeral local port (the fixed 5569 let only one sender per machine start) and both
  output engines open their socket in `start`, so a failure shows as "skipped".
- **4b — protocols (§11, "Step 4b hardening"):** networks (show names, station mapping),
  output lines + routes, reconcile runtimes after every show change (nothing at boot),
  who sends (standalone/master/IdleMaster; replaces the `dmx_output_enabled` hack), Art-Net
  ArtPoll unicast + directed broadcast, stable sACN CID, OSC input line with user, ranges
  dropped, line status, Live Patch `Set` rules. Input lines and merge are modelled.
- **4b built** (2026-10-11): `lumacore_tree::output` (Network, OutputLine with `target`,
  InputLine, Route, MergeMode) + `OutputOps` (routes); `lumacore_dmx::{nics, lines}` and a
  hand-encoded E1.31 sender (`sacn::SacnSource`: binds the NIC and sets `IP_MULTICAST_IF` —
  the `sacn` crate couldn't, so multi-NIC multicast would leave on the wrong card; data,
  universe discovery, stream-terminated) and Art-Net (`ArtNetSource`: ArtDmx to a 15-bit
  Port-Address, ArtPoll every 3 s, ArtPollReply nodes, directed broadcast);
  `crate::protocols` (`desired` = pure plan, `ProtocolRuntimes::reconcile` on the state thread
  after every show change / station change / each second; OSC listeners per input line with
  `InboundEvent::UserCommand`). The sACN/Art-Net/OSC registry engines are gone; only
  `playback`, `perfetto`, `http` start at boot. Objects `Network`, `Output`, `Input`
  (`objects/protocols.rs`); `Assign`/`Unassign`; `Set Network … Interface` (station file);
  `Set Fixture <n> Address <u.a>` (Live Patch). Ranges, Split, Merge removed. Slot occupancy
  is rebuilt on load (was lost, allowing double patching). Known gaps: driver threads don't
  report to Runtime.Health; renaming a network doesn't rename it in lines; IdleMaster isn't
  detected (step 6) so "output when alone" is stored but unused, and has no Set yet; fixture
  Pan/Tilt offset/invert and position aren't modelled for Live Patch Set; sACN discovery sends
  one page (≤ 512 universes); NICs are re-read every 5 s.
  *(Note 2026-10-11, step 11a: IdleMaster detection and output-when-alone were built in 6c/4d; network renames follow GUID links since 6d. Still open: driver threads in Runtime.Health, Live Patch offsets, sACN discovery paging.)*
- **4c — input + DMX window (§11 "Step 4c hardening", one step):** one Line type, sACN/Art-Net
  input lines, per-channel merge after render (`Universe.merge_mode`, input bands mapped onto
  today's layers; Park wins, masters/Solo leave input alone), E1.31 source rules and 2.5 s
  loss, ArtPollReply, sources + `DMX In`, the `dmx` window with a WebSocket feed.
- **4c built** (2026-10-11): `lumacore_tree::Line` (+ `InputPriority`, `MergeMode` labels) and
  `LineOps` (formerly `OutputOps`, every operation by `Direction`; inputs get their own ID
  pool; one guest universe may feed several local universes); `lumacore_dmx::input`
  (`InputMerger`: sources by (protocol universe, CID/IP), E1.31 priority then HTP, target
  filter, 2.5 s loss, stream-terminated, per-channel change stamps → `InputLayer`s),
  `sacn::receiver` (`SacnReceiver`, parse, preview/start-code/own-CID filtered), Art-Net
  `poll` (ArtDmx in, ArtPollReply for input Port-Addresses, own IPs ignored; the Art-Net
  socket now binds `0.0.0.0:6454` pinned to the NIC), `nics::listen` (wildcard bind pinned with
  `IP_BOUND_IF`/`SO_BINDTOIFINDEX`, else filtered by the NIC's subnet), one `NetworkDriver` per
  (protocol, NIC) for both directions, `PlaybackHub::{inputs, console, winners}`,
  `playback::{render_console, merge_inputs}` (`Owner` per channel, `Winner`).
  `crate::protocols` plans `drivers` per (protocol, NIC); input line states "receiving n/m" /
  "no source"; `Runtime.Network.Sources`. Commands: one handler set for Output/Input,
  `Set Input <n> MergeMode`, `Set Universe <n> MergeMode`, `InputPriority` (new `PropKind::Choice`),
  cross-direction settings refused with E-SET-001, `Assign/Unassign … At Input`, `DMX In`,
  `Info Input` with sources, status icon "⇣n". Window type `dmx` (no. 5) + `/ws/dmx`
  (`ConsoleBridge::dmx_universe/dmx_patch/dmx_selected_universe`) + the web UI sheet.
  Live-tested: guest sACN on lo0 merged and re-sent as sACN on en0, two sources by priority,
  loss, Park over Super input, MergeMode Off/Prio, `/ws/dmx`, clean shutdown. Known gaps: the
  browser sheet was checked for syntax and its feed with a script, not visually; Art-Net
  input can't come from this machine (own IPs are ignored, also a second LumaCore here); no
  E1.31 sequence-order check or sync universe; input is drained once per engine tick (44 Hz);
  no reference-console input limits (20 multicast / 128 total); the DMX tester bar and selected-fixture
  highlight in the sheet come later; status says "no source" (it doesn't remember a lost one).

- **4d — menus, LumaSession, staged patch (§11 "Menus, LumaSession and staged patch") built**
  (2026-10-11): `terminal/patch_draft.rs` (draft on Patch claim, live view published,
  owner's patch edits not filed until Keep, others refused E-PATCH-008, Keep / Discard /
  Cancel prompt, `Unclaim Patch /Keep|/Discard`; Unclaim Patch no longer clears undo stacks —
  Keep is one undo step); protocol objects need no claim; LumaSession rename (objects
  `LumaSession`, `Shell`, `Interface`; verbs `Join`, `Leave`, `Invite`, `Dismiss`;
  `JoinSession`/`LeaveSession` gone, no aliases; `Root.Runtime.LumaSessions` with
  Stations/Members/Listeners/Connections); station settings (`Set Station Name|LumaSession|
  Location|Priority|Invite|Online|HoldOutput|OutputWhenAlone`, saved in the station file);
  `remote::lumasession` (status Standalone/Master/Member, Invite/Dismiss/Member requests,
  deferred join/leave after the job, autosave copy before an invited join, `ServiceAccess`);
  discovery per NIC with station id, LumaSession name, location, status; members known to
  the master; a member no longer sends (`is_authority` false; was never set before), leaving
  or dismissal holds output; `Set Interface … Yes|No` enforced live on the command port,
  discovery, Shell and HTTP (axum filtering listener); `GET /api/network`,
  `/api/lumasessions`; the three menu overlays. Live-tested with two servers on one
  machine: discovery, invite (Invite on/off), dismiss (held output), join by name, leave,
  per-NIC HTTP/Shell refusal on en0 with loopback served, LumaSession off/on, the endpoints,
  clean shutdown. Known gaps: the menus were syntax-checked and their endpoints exercised,
  not looked at in a browser; the LumaSession link after a join is still today's one-shot
  snapshot (replication is step 6); the patch has no Patch menu yet (the draft is used from
  the terminal); invite runs the join as the station's first logged-in user (else Admin);
  `remote::tests::shutdown_command_sets_should_exit` already failed before this step.

## 25. Guards and Work Order (2026-10-10)

- Guards: registry tidy test (§15), cargo-fuzz on OSC / wire / command parsers, proptest
  round-trips for `.luma` and replication ops.
- Order: 0 quick fixes + cleanup baseline (§26: fmt, clippy, http tests, Art-Net, discovery
  spin, OSC break, address 0, hygiene) · 1 registry · 2 engine thread + change path + typed
  errors (§26) · 3 mode objects + keys · 4a playback engine skeleton (§27) · 4b protocol
  lifecycle + output routing (§11) · 4c DMX input + merge · 5 security ·
  6 replication + undo coverage + show versioning · 7 groups · 8 proto crate + module split ·
  9 render thread · 10 fuzz/property tests.

### Step 10 hardening (2026-10-11)

The older docs plan nothing beyond the line above; they do promise invariants (§10/§21
replication, undo, identity, merge, election) that step 10 turns into properties.

- **Two kinds of test, run differently.**
  - **Fuzzing is a long run only** — never part of per-step tests ("it would take to long to
    test stuff like this during fast development"). No stable decoder smoke tests in `cargo
    test`.
  - **Property tests are ordinary fast `cargo test`s** (proptest, a dev-dependency in
    `[workspace.dependencies]`), run per step when their area changes; all of step 10's
    together finish in about 10 s (debug).
- **Fuzz targets — every decoder on the open LAN plus the lexer.** OSC packet, sACN data
  packet, Art-Net ArtDmx and ArtPollReply, command-port `RemoteRequest`, shell packets,
  discovery beacon, `Time` ping/reply, LumaSession stream frame (length prefix + zstd + JSON
  `StreamMsg`), the command-line tokenizer/lexer + selection parser (+ dispatch on a fresh
  state, unguarded, so a panic shows), `passwords::mask_secrets`, token verify. GDTF import
  comes later (with the §28 importer fix); `.luma` load is covered by the property tests.
  - The rule each target checks: **no panic, no hang, on any bytes** (a decoder returns
    `None`/`Err`). Codecs that also encode check decode(encode(x)) = x.
  - **`fuzz/`** is its own cargo-fuzz crate **outside the workspace** (nightly only:
    `cargo +nightly fuzz run <target>`); the stable build never sees it. Private decoders get
    a `#[doc(hidden)] pub` entry point for it — no `arbitrary` derives on wire or tree types.
- **The long run** is a script the user starts by hand (overnight): `scripts/unix/fuzz_long
  [minutes per target]` runs every target in turn with `+nightly`, logs to
  `target/fuzz-report/<time>/` (per target: runs, coverage, crashes with their inputs) and
  prints a summary. No CI, no scheduler (§25, Q&A "Tooling guards").
- **Findings are reported, fixed later.** The build runs each target ~5 min once and records
  what it finds (here + roadmap) as follow-ups; crash inputs are kept under `fuzz/artifacts/`.
- **No input size caps** (isolated networks, §19): the stream's 1 GB frame limit, unbounded
  zstd decompression and unbounded shell lines stay as they are; fuzzing looks for panics
  and hangs, not memory use.
- **Random shows come from random command sequences** run through real dispatch from the
  demo show (lines generated from the registries: patch, store/delete/move groups, cues,
  presets, set properties, users, Oops/Redo …) — the real change path, no `Arbitrary` on
  tree types.
- **Properties.**
  - `.luma`: save → load → save gives the same bytes; the loaded show passes the identity
    and reference checks.
  - Replication: the ops a sequence produces, applied on a fresh member in order, give the
    master's show; applying an op twice = once; a gap or a lower epoch is refused.
  - Undo: after any command `Oops` restores ShowData exactly; `Oops` then `Redo` is the
    identity; a member's undo stacks match the master's.
  - Merge (asymmetric by design — master wins, not commutative): merge(base, x, x) = x;
    merge(base, base, mine) = mine except Playbacks (the master's); no duplicate GUIDs or
    slots after a merge; an edit beats a delete.
  - Election + clock: `elect` picks the same winner for any input order; the clock filter
    never runs backwards except one step (first lock after joining, or more than 1 s off) and keeps frequency within ±500 ppm (R1-04, Q1).
  - Identity + masking: no duplicate GUIDs and no new broken links after any sequence;
    `mask_secrets` never leaks the password and is idempotent.
- **Command sweep — every command, keyword and property in the nightly** ("all commands and
  all keywords needs to be tested … all combinations"; "it needs to be dynamic, so if we add
  new features down the line we dont need to update it").
  - **Dynamic, never a list.** Everything is read at run time from the registries
    (`CommandDef` names/aliases/flags/`args`, `ObjectDef` keywords/verbs/pools, `TreeDef::write`
    props with type/range, the completion engine, the error catalogs). A new verb, keyword,
    flag or writable property is swept the next night with no test edit. Anything it can't
    generate (an `Args` shape or `PropKind` without a generator) is reported as **unreachable**
    in the run summary — the one place a new kind of thing asks for code.
  - **Hands off the tree shell and the drive** ("it should NOT touch the unix-like commands
    and drive access"). Every `CommandDef` (and object verb) declares a required
    `reach: Show | TreeShell | Drive` — the compiler forces it on a new command, so the sweep
    stays dynamic. The sweep runs only `Show`, and lists what it skipped in its report.
    - `TreeShell`: pwd ls cd rm cat head tail stat tree find grep du wc diff whoami env
      history root.
    - `Drive`: anything that reads or writes disk — import export saveshow loadshow newshow
      deleteshow create. A tidy test fails on a handler using `std::fs` that isn't `Drive`.
    - Process and LumaSession commands stay in (Shutdown/Restart end that sequence and reset
      the state; Join/Invite only queue outbox requests nobody sends).
  - **Isolated in process:** an `AppState` with no server, no protocol runtimes, no engine
    threads and no sockets (the plan is compiled and one tick runs in process); `Assign … At
    sACN` only edits the show. Its only files are its report, `run.json` and its own temp dir
    for the save → load check.
  - **1) Exhaustive sweep:** every verb × every object keyword × every flag/option × every
    writable property, with ids (none, 1, a `Thru` range, a missing one, a name) and values
    (min, max, out of range, wrong type). **2) Deep random:** hours of random stateful
    sequences built by walking the completion engine word by word.
  - **Checks after every line:** no panic / no E-INTERNAL / nothing over 1 s (a hang);
    no duplicate GUIDs, no new broken links, a refused line changes nothing, `Oops` restores
    the exact pre-image; `Set` on a writable property reads back or refuses with a code, a
    read-only one is refused and unchanged. **Every N lines:** save → load round trip, an
    in-process member fed the ops equals the master, the plan compiles and a render tick runs.
  - **Every role:** each line as Admin, Programmer and Guest (+ a second user holding claims);
    a refusal is E-RIGHTS-001 / E-CLAIM-001 and changes nothing; allowed lines behave as for
    Admin.
  - **Runner:** a stable, seeded `lumacore-sweep` binary (sweep + random), plus a cargo-fuzz
    `commands` target whose bytes choose completion paths (coverage-guided); `fuzz_long`
    runs both and the decoder targets.
  - **Resumable.** `target/fuzz-report/run.json` holds status, pid, build id, phase (sweep /
    random / each cargo-fuzz target), cursor, seed, last line and a heartbeat; checkpoint
    every 100 lines. On start: completed → a new run; Ctrl-C/SIGTERM → saved as `stopped`,
    continue; status running but the pid gone → `crashed` (OOM, abort, power): logged with
    its last lines, that line skipped and kept as a finding, continue. A panic is caught per
    line and is a finding, not a crash. A **new build** continues the run (build id recorded
    per segment) and sweeps the combinations it added first; `fuzz_long --fresh` starts over.
  - **Findings don't stop the run:** each is logged (seed, role, the line history that led
    to it, the failed check, a replay command), the demo show is reloaded and the run goes
    on; a repeat (same check + same verb/keyword) is counted, not logged again.
- **Docs.** `docs/dev/testing.md` gets the property tests and the long run;
  `docs/dev/build/vibecode-workflow.md` loses "full pipeline on every change" for the
  per-step rule (full suite and the long run only on request); stale numbers fixed
  (`FORMAT_VERSION` in show-files.md / crate-map.md, `SESSION_PROTOCOL` in replication.md).

### Step 10 built (2026-10-11)

- **Reach.** `CommandDef::reach` (`Show | TreeShell | Drive`) on all 100 commands and
  `ObjectDef::reach: fn(Verb) -> Reach` (`show_reach` / `drive_reach`: `Show`, `Library`);
  `registry::line_reach`. `create` stays `Show` (it is an alias of Store, no disk).
- **One door to the disk.** `core::drive` wraps every file call of `core/`; the sweep closes
  it per thread and gets what knocked. Tidy `core_reaches_the_disk_only_through_drive`
  (`// tidy: drive-ok (<reason>)` for the sysmon log writer).
- **Sweep** (`crates/lumacore-station/src/sweep/`: `vocab`, `lines`, `world`, `run`; binary
  `lumacore-sweep long|replay`; profile `sweep` = release + debug assertions). The save →
  load check runs in memory (no temp dir needed). The plan is ~19,700 cases × 3 roles (about
  a minute); random sequences run ~1,200 lines/s. Its own layer in the tidy table (`sweep`).
  Resume, crash skip (`kill -9` mid-run → logged and skipped) and new-build-first tested.
- **Fuzz:** `fuzz/` (13 targets: osc, sacn, artdmx, artpoll_reply, remote_request,
  time_reply, shell_client, shell_server, beacon, stream_frame, command_line, token,
  commands) over `lumacore_station::fuzzing`; builds on nightly; a short run (~25 s each) found no
  crash. The 5-minute runs were skipped ("i dont want long tests now").
- **Property tests:** `sweep/property_tests.rs`, 14 pass in ~2 s; `lumasession::link::op_order`
  and `election::best` extracted so they test the code the server runs.
- **Script:** `scripts/unix/fuzz_long` (build, sweep with restarts, property tests with a
  random seed). Docs: testing.md, vibecode-workflow.md, stale numbers fixed.
- **Findings (reported, fix later)** — from a full sweep + 5 min random (474,831 lines):
  1. `Programmer.values` is a `HashMap`: save → load → save changes order (not BTreeMap).
  2. `Reset`, `ResetAll`, `ResetSelection`, `ResetActive`, `Watch`, `Meow`… change the
     programmer while declared `ReadOnly`: no undo, no replication (members drift).
  3. `Bind` gives two profiles' UIKeys the same GUID → the saved show is refused (E-SHOW-003).
  4. A Guest line refused (E-RIGHTS-001) in its second statement still changed the patch via
     the first (`Label …; …`), and Oops after `Label` leaves Devices/DataPool different.
  5. `quicksheet`, `List Drives`, `Enter`, `Watch Show` reach the disk though `Reach::Show`;
     `Set Station …`, `CleanStart`, `Restart`, `Shutdown` write station settings / autosave.
  6. Refusals without an E-code: LD `Store User`, `CleanStart`, `Unclaim Group`,
     `Clear Programmer`; `Set User 1 default_screen_config 1`.
  7. `Delete Fixture` leaves group members pointing at it (broken links).
  8. Election: `ranks_above` isn't transitive (±5 s online tie), the winner depends on the
     order (`every_order_elects_the_same_winner` ignored with this finding).
  9. Never accepted: `add`, `remove`, `goback`, `debug` (and bare aliases).
  10. `systemtest` doesn't compile (still uses `SharedClock`, removed in step 9) — older.

## 28. Effects (2026-10-11)

Researched against three generations of a reference console's effect engines (manuals, release
notes, user forums, a live console (sequence/cue/part
settings), and the GDTF 1.2 spec. Verdict: the first generation got the structure right (a cue *calls* a living
effect: base ± size, forms, fast), the second got per-fixture layers right, the third's phasers ("everything
is a phaser", sines built from accel/decel steps, no clean stomp, breaks on patch changes) are
what we avoid.

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
  *(Note 2026-10-11, step 11a: the crate is `lumacore-base` since step 8: `lumacore_base::Value`, `crates/lumacore-base/src/value.rs`.)*
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
- **Running.** `Effect 5` turns the effect on, as on early consoles: a call in the user's programmer on
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
  use the same timing. Sequence off fades every call by the **OffCue** timing (as established; also
  Kill/Swap). Not needed from the reference: a phaser sync (cue anchors keep calls in sync; rejoining fixtures drop
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
  *(Note 2026-10-11, step 11a: wrong — step 7 built `GroupMember { fixture, cell: GridCell { x, y, z, rotation, mirror_x, mirror_y } }` (`lumacore-tree/src/datapool.rs`), so this order item is done.)*
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
  `Rate`, `Disable`, `Enable`, `Stomp`, `Unstomp`, `IfStomp`, `Shuffle`) are published via
  the registries, `status: Planned` until built (Stomp, Unstomp, IfStomp, Disable, Enable and
  Shuffle are, R1-D6); errors in an `E-EFX-NNN` catalog. **`Lo` / `Hi` are effect-line
  properties, not registry words** (`Set Effect 5.2 Lo 20`; `hi` stays Highlight's alias —
  R1 Q44).
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
  - **Effects in the sequence sheet.** Each cue shows its effect calls like recipe lines,
    drawn as squares stacked in a list. Selecting one gives its controls (size,
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

## 29. Cleanup — docs, findings, Oops categories (2026-10-11, step 11, hardening)

After steps 0–10 an audit compared every md file with the code (three read-only passes:
handoffs, meta docs, dev/user docs). Nothing outside `feedback-system`, `thin-client`,
`ui-keys` and §28 Effects is left as is. Data is kept: superseded docs are archived, never
deleted.

- **Steps.** 11a docs restructure → 11b code fixes (step 10 findings, Planned, gaps) → 11c
  Oops categories + UI Oops → 11d screenshots (before the UI work). Each hardened, built on go, committed on commit.
- **Docs.**
  - **One plan:** `docs/meta/roadmap.md` is the single living list (Done with commits, Now,
    Next, Later, Ideas; each item links its decisions § and handoff). `TODO.md`,
    `requirements.md`, `project.md` and the open items of the handoffs merge into it.
  - **Archive:** superseded files move unchanged to `docs/archive/<area>/` with
    `docs/archive/README.md` (file, why, replaced by, date); finished handoffs to
    `handoffs/done/`.
  - **Command docs:** a reference generated from the registries
    (`docs/user/reference/commands.md` + `objects.md`: syntax, examples, aliases, rights,
    status) with a test that fails when it is stale (like the demo show); hand-written docs
    keep concepts and guides only and link to it — no hand-kept syntax or abbreviation
    tables. `command-reference.md` archived.
  - **Networking:** `session-engine.md`, `multi-station.md`, `network-sessions.md` merge
    into one current `docs/dev/networking/lumasession.md`; `replication.md` and
    `network-engine.md` fixed and kept; the three archived.
  - The per-file plan (keep / fix / merge / archive) is reviewed with the user before 11a.
- **Findings (step 10) → fixes in 11b.**
  - **All or nothing:** the rights gate checks every statement of a line before any runs;
    one refused → nothing runs, the reply names it (E-RIGHTS-001).
  - **Delete keeps links (§22 stands):** links are GUIDs, kept and listed; things work again
    when the object comes back. The sweep stops counting broken links as a finding
    (duplicates, nil GUIDs, wrong kinds, loops still are).
  - **Election:** online time compared in whole 5 s buckets, then station id — transitive.
  - **Bind:** a user key's GUID is `Guid::named` from (profile GUID, key id).
  - **Codes everywhere:** every refusal has its code (rights E-RIGHTS-001, claims
    E-CLAIM-001, others fitting codes) and a test that no `Error:` reply lacks one;
    `add`/`remove`/`goback`/`debug` each checked: editor-only says so with a code, unbuilt
    becomes Planned.
  - **Planned:** commands that pretend to work (Executor, Macro, Effect, Timecode calls,
    Pause, Flash, `DMX … At`, OSC AttributeSet/Transport …) get `Status::Planned`
    (`ObjectDef` gains a per-verb status): running one replies `Error: … isn't built yet
    (planned) [E-CMD-…]` and changes nothing; Help, hints, the generated reference and the
    sweep show or skip them; unbuilt OSC events log once to sysmon.
  - **Disk:** `Root.Drives` is entered only by tree-shell words (Quicksheet, List, Enter,
    Watch never walk into it unless the path names it). The sweep runs in a **sandbox**: a
    throwaway copy of `library/` as its working directory; the drive door lets paths inside
    it through and refuses (a finding) any path outside (absolute, `../`). Drive words
    (SaveShow, LoadShow, Import, Export, Set Station, autosave …) are then swept too; the
    tree shell stays out.
  - Also: `Programmer.values` → BTreeMap (stable save bytes); `systemtest` fixed and kept
    (clock test, `build.rs` watching moved files); §18 renames (ViewSession → Screen,
    ShellSession → CommandLine); every datapool's groups in the tree; help text without
    removed Range / `Show … /save`; web keyboard Release/Hold forwarded (keyup); GDTF cell
    import bugs (geometries kept, footprint, resolver keyed by geometry + attribute);
    scripts still building the old package names (`connect.sh`, `dev_init.sh`, `smoke_test`,
  `./lumacore-scripts`, the Windows `.ps1` set) and the `--monitor` comment in
  `tests/e2e/support/server.ts`;
  playback/sACN leftovers (cue fades and timing, selected sequence, Highlight output,
    E1.31 sequence check + sync, no allocation per received packet) — each hardened in its
    step before it is built.
- **Oops (11c) — the UI too.**
  - Oops-able: windows (open, close, move, resize, settings), views and screens (store,
    recall, delete, split/merge, layout, ViewButtons), user settings, selection and
    programmer (incl. Reset*, Clear). A drag or resize is one step (filed on release).
  - **Categories** label every step: Programmer, Selection, Show (pool objects), Patch
    (fixtures, types, universes, networks, lines, routes, Keep Patch), UI, Users, Settings,
    Keys, Station. One Oops list per user; **Oops always undoes the newest step** of any
    kind (order never broken); categories are for labels, colours and `List Undo /<cat>`.
  - **Station** steps (interface services, network→NIC mapping, Online, name, priority)
    undo only on the station where they happened (never replicated).
  - **Every statement of a line is its own step** (console probe, R1-62, Q10): `a; b` runs
    statement by statement after the rights gate checked them all; each files its own step,
    is confirmed on its own (OopsConfirmation) and goes to the master on its own. A statement
    that asks (Delete, Oops with confirmation) stops the line; the answer runs it and then the
    rest (`Ask::then_run`). Delete asks once, in the station, before the statement runs or is
    forwarded (`/NoConfirmation` rides along); a stored key or macro line never asks, nor do
    `Delete Show`, `Delete ViewButton` or a planned Delete.
  - Oops / Redo of a step that would change another user's claimed object is refused before
    the step moves (E-CLAIM-001); the step stays.
  - Running playback (Go, Off, faders, masters, Solo) is still never undone.
- **Docs plan details** (`handoffs/done/2026-10-11-docs-plan.md`, reviewed): vision-only docs
  (design/*.md notes, effects/effect-engines.md) move to `visions/notes/` (ideas also in the
  roadmap's Ideas); `docs/archive/` holds superseded docs only. `decisions.md` stays one file
  with a table of contents (status per §: built / partly / not built) and stale lines
  annotated in place. The command reference is generated Markdown (`gen-docs`) with a test
  that fails when it is stale.
- **Screenshots (step 11d, before the UI work)** — "we also want to implement screenshotting
  before the ui work": first for development and tests, on the one path that later serves
  view thumbnails (view button pictures) and the `Screenshot` command.
  - **Captured in the client, asked by the server:** the server sends a capture request to
    a connected client (web, app window, headless Playwright); the client renders its screen
    to PNG itself (DOM + canvas, incl. the xterm terminal) and uploads it
    (`POST /api/screenshots`).
  - **Stored as files:** `library/screenshots/<station>/<time>_<user>_screen<n>.png`
    (station-local, not in the show). View thumbnails later go into the show as small
    images through the same capture.
  - **Dev:** `scripts/unix/screenshots` starts a test station with the demo show, opens
    every window type, menu and overlay headless, captures each through the real path into
    `target/screenshots/` — to look at the UI while building it.
  - **Command now, minimal:** `Screenshot [Screen n] [User x]` asks that user's connected
    client(s) to capture and saves the files (rights Programmer, reach Drive). View
    thumbnails stay Planned.
- **11a built (2026-10-12).** `docs/meta/roadmap.md` rewritten as the one plan (TODO,
  requirements, project merged in; every open item placed; findings from the audit added to
  11b); 17 docs archived under `docs/archive/` with an index, 6 vision notes in
  `visions/notes/`, 6 finished handoffs in `handoffs/done/`; `gen-docs` + `core::reference`
  write `docs/user/reference/commands.md` + `objects.md` (test `the_reference_is_current`);
  `docs/dev/networking/lumasession.md` merged; `docs/dev/api/http-api.md` covers every route;
  user docs rewritten as guides linking the reference (lum.md absorbs targets/values/
  functions; new pages for tree shell, keys, health and clock); dev docs, READMEs, CLAUDE.md
  and active handoffs fixed; this § got a contents table with status per §. Every relative
  link outside the archive resolves (archived files keep their old links unchanged).
- **11b hardening (2026-10-12)** — a reference console's manual (Select, Collect,
  `keyword_collection`, `keyword_off`, `keyword_highlight`, `keyword_solo`, `masters_grand`,
  `cue_sequence` pool actions, `keyword_editsetting`). Built as **one** step (all items
  below plus the §29 findings list), go then commit.
  - **Selections (per user, `ShowData.Users`: saved, replicated, Oops-able in the Selection
    category of 11c).** Like an established `Select` ("selects objects as default objects"), apart from
    the fixture selection:
    - **Fixture selection** — as today (`Fixture 3`, group recall, `apply_selection`).
    - **One-only selections: DataPool, Sequence, Timecode** (later Page, World, Layout,
      Timer, Camera when their features come). `Select Sequence 3`, `Select DataPool 2`,
      `Select Root.ShowData.DataPools.2` (by tree path), `Select Timecode 1`; `Deselect
      Sequence` etc. clear one. `selected_datapool` moves from the profile to the user.
      `cd` never selects. A `Select` on any other kind (`Select Group 3`, `Select Fixture
      3`) is a coded error naming the kinds Select takes.
    - **Defaults:** the selected sequence is the target of `Go`, `Off Sequence`, `Goto Cue 3`
      and `Store Cue 4` when no sequence is named; the selected datapool is where
      `Group 3`, `Preset 4.1`, `Sequence 5` resolve. Nothing selected → `Error: no sequence
      selected — Select Sequence <n> [E-PLAY-…]`, nothing runs (never guess Sequence 1).
    - Deleting the selected DataPool / Sequence / Timecode **clears** that selection.
    - **Bare `Off`** stays the fixture knock-out + deselect (elsewhere `Off` needs an object to
      stop a sequence). Clear never touches the one-only selections.
    - **Sequence pool window** (with the UI kit): pool actions through `PoolDef.actions`
      — Select (default), Go, Goto Top, Toggle, Off, None; per-object action later.
    - A **many-object selection** was discussed and dropped. Idea instead: a
      `Collect`-like keyword (own name, not Collect) that gathers several tree objects per
      user (`ShowData.Users`) for one command.
  - **Cue timing.** A cue part is like a stacked cue inside one cue: every part runs its own
    timing from the same Go. Established rule per attribute: dimmer up and every non-dimmer attribute
    use the in fade/delay, dimmer down the out fade/delay (`CuePart` gets `delay_in` /
    `delay_out`); GDTF `LogicalChannel@Snap` attributes jump at the end of their delay. Fades
    run on `clock::session_ms()`. A Go during a fade starts each attribute from where it is
    (the old fade is dropped). No per-value (individual) timing yet.
  - **Highlight output.** Values come only from the fixture's GDTF (`DMXChannel@Highlight`);
    a fixture without them doesn't change — no hard-coded dimmer fallback. Each user's
    `On Highlight` covers that user's fixture selection (several users: all theirs). Merge:
    above programmers and sequences, below park; grand/group masters still scale the dimmer.
  - **Highlight and Solo masters now** (the §16 per-user mode masters, reference masters
    `2.5`): in `Runtime.Users.{User}` (replicated, never undone), 0–100 %, mode Toggle /
    AlwaysOn / Disabled. Highlight master: proportional from the current value to the
    highlight value on every highlighted attribute; Solo master: proportional on the dimmer
    of the fixtures not selected. Blind and Lowlight masters later.
  - **E1.31 input:** per spec — a packet 0 to −20 behind the last sequence number is dropped
    (late/duplicate), a bigger jump is accepted (source restart); a universe with a sync
    address holds input until the sync packet, falling back to no-sync after 2.5 s.
  - **GDTF native.** The original `.gdtf` is the source of truth and travels **in the show**:
    `.luma` becomes a container (zip: `show.json` + `fixturetypes/<sha256>.gdtf`, later
    images/screenshots), `FORMAT_VERSION` 6 (the selection move needs it too; demo show
    regenerated). The `FixtureType` is a derived view rebuilt from the file, now carrying the
    geometry tree + cells (GeometryReference, Break offsets, Relation Multiply = master
    dimmer, per-mode cell layout), every ChannelFunction with DMX/physical ranges and
    ChannelSets, `DMXChannel@Highlight`, `LogicalChannel@Snap`, wheels and physical
    descriptions (emitters, filters). Import stays **warn only** (refuses nothing it can
    read; the report in `Info FixtureType`). The resolver keys by (geometry, attribute).
    Editing a fixture type (writing a new `.gdtf` revision) is not decided — roadmap.
  - **Planned:** one code `E-CMD-…` "isn't built yet (planned)" naming the word. Status on
    **each verb entry** of an `ObjectDef` (`Built | Planned`; a Planned verb has no handler);
    help, hints, completion (dimmed), the reference and the sweep read it; a test fails if a
    Planned verb changes the show.
  - **Show word:** `SaveShow` / `LoadShow` stay; `Show "x" /Save` and `Show "x" /Load` are
    other spellings of them (same handler and guards). **`EditSetting <object>`** (alias
    `EditSettings`) for every object: in the web/app it opens the object's settings
    pop-up, in a terminal it lists the writable settings (`TreeDef::write`) with the `Set`
    lines to change them; `EditSetting Show` = ShowSettings; no settings → coded error.
  - **Small ones:** `--tls-cert` / `--tls-key` dropped (isolated networks; TLS is an idea);
    `MEOWNITOR <ip>` replies Planned (idea: the System Monitor window follows other stations,
    a remembered station list saved in the show, reconnecting); `If` is registered as a
    Planned word (keyword needed later) so the `I` shortcut stays; `core/language.rs` is
    archived to `docs/archive/code/` (not compiled). Sweep sandbox as decided above.
- **11b built (2026-10-12).**
  - **Findings:** rights checked for every statement before any runs (`dispatch_inner`);
    election online time in whole 5 s buckets (`every_order_elects_the_same_winner`
    un-ignored); a user key's GUID is named from (profile GUID, key id); `Programmer.values`
    is a `BTreeMap` (legacy `active_values` read dropped); the sweep no longer counts kept
    broken links; it answers confirmations through `dispatch_confirm` and skips the
    read-only check for a replaced show; `SaveShow` is `Untracked`.
  - **Codes everywhere:** every error reply goes through `core::errors` (`render_code`,
    `render`, `reply` for passed-on messages — no more "Error: Error:"); general codes
    E-CMD-006…015, E-PLAY-003; tidy `error_replies_carry_a_code`; the sweep reports
    `uncoded-error` on every line; session notices are `text::render_warning`.
  - **Planned:** `ObjectDef::planned` (verbs without a handler): Executor / Macro / Effect /
    Timecode / EffectEngine Call / Store / Info, Preset (and the preset types), DataPool Call
    / Info; commands `Flash`, `If` (new, so the PRG `I` shortcut stays); `Pause` other than
    `Pause LumaSession`; `DMX … At` (its unused override map removed). One reply,
    E-CMD-003 "isn't built yet (planned)"; help, quick hint, completion and the reference
    show them; test `planned_words_reply_planned_and_change_nothing`. Planned OSC events
    (`/lumacore/fixture/…`) note once in the System Monitor.
  - **Disk:** `registry::line_reach` counts an object argument (`Watch Show`) and a path
    naming `Root.Drives`; `ls` shows `Drives/` without reading the disk; `Restart`,
    `Shutdown`, `CleanStart` are `Reach::Drive`. The sweep runs Drive lines in its sandbox
    (`target/fuzz-report/sandbox/`, a copy of `library/` without `stations/`,
    `station_settings.json`, `logs/`; `drive::set_sandbox`; `drive-escape` findings).
  - **Selections:** `ShowData.Users.{n}.selected` (DataPool / Sequence / Timecode GUIDs;
    `selected_datapool` left the profile), `Select` / `Deselect`
    (`core/commands/selections.rs`), the playback words use `AppState::selected_sequence`
    (E-PLAY-003), dangling selections are cleared after every line.
  - **Cue timing:** `CuePart` delay_in / delay_out; `Tracked::part`; `Resolved::timing`;
    `plan::Fade` per channel started by the compiler from the previous plan at the session
    clock (`Plan::compile_after`); `Set Cue <n> Fade|OutFade|Delay|OutDelay <s>`.
  - **Highlight and Solo:** `ChannelFunction::highlight` from GDTF only; per-user
    `ModeMaster` (level %, Toggle / AlwaysOn / Disabled) in `Modes`; `UserLayer` carries
    strengths; `Master Highlight|Solo At …|Mode …` (as established). `SESSION_PROTOCOL` 5.
  - **E1.31 input:** sequence check (0…−20 behind dropped), sync packets parsed, synced
    data held until its sync (no-sync after 2.5 s), `SourceName` + reused packet buffers
    (no allocation per received packet); the sACN fuzz target also parses sync packets.
  - **GDTF native:** `lumacore_gdtf` derives the whole model (geometry tree, cells from
    GeometryReferences at their break offsets or sibling geometries, every channel
    function with ranges and channel sets, highlight, snap, relations, wheels, emitters,
    filters; footprint = highest slot of break 1; resolver keyed by (geometry, attribute);
    `refresh_revision` keeps GUIDs). The render plan places a channel in its GDTF break.
    `core::show_file`: `.luma` is a zip (`show.json` + `fixturetypes/<sha256>.gdtf`),
    `FORMAT_VERSION` 6, demo regenerated (it carries the AuraXB file); Import reads through
    the drive door and keeps the file in `library/fixturetypes/` (gitignored).
  - **Words:** `EditSetting <object> <n>` / `EditSetting Show` (terminal list + the web
    `EditSettings` pop-up); `Show "x" /Save|/Load` = SaveShow / LoadShow; help `lum` pages
    rewritten; Shortcuts doc; Preset List reads `PresetGroups`.
  - **Renames (§18):** `ViewSession` → `Screen` (`ScreenStore`, `live_screens`, tree
    `Runtime.Users.{n}.Screens`, `/api/screens`), `ShellSession` → `CommandLine`;
    `CLIENT_PROTOCOL` 3. Keyboard Release / Hold from the web UI (`key_hold` / `key_up` →
    `ShellClientPacket::KeyGesture`).
  - **Tree:** every datapool by number (`Root.ShowData.DataPools.<n>.Groups`,
    `TreeDef::datapool`); `Set` / `Claim` by path use it; `cd` takes a number beside named
    children.
  - **Cleanup:** TLS flags dropped (`tls.rs` and `core/language.rs` archived in
    `docs/archive/code/`, `CommandFamily` moved to the registry); interactive `MEOWNITOR`
    replies planned; scripts build `-p lumacore-cli` / `-p lumacore-scripts`; stale
    comments fixed; systemtest compiles (session clock check, `build.rs` paths) and builds
    its demo with `core::demo`.
  - Not built here: the sequence pool window (with the UI kit); systemtest's failing
    `head` / `tail` steps (roadmap).
- **11c hardening (2026-10-12)** — read: §21 (undo + replication), this §, the decisions Q&A,
  the thin-client handoff, `replication.md`, `control.md`, `runtime.md`, `show-files.md`;
  a reference console's manual (the Oops keyword and overlay, user settings, release
  notes 1.9), its Oops menu, a root dump and a live probe.
  Built as one step, go then commit.
  - **Don't reinvent Oops** ("we only want categorys so the user can have it coloured and see
    them in the lists"): one list per user (`Runtime.Users.{n}.Undo`, replicated), Oops
    always undoes the newest step. The **category is only a label** (colour, `List Undo
    /<cat>`): Programmer, Selection, Show (pool objects), Patch, UI, Users, Settings, Keys,
    Station — replacing today's four (`Programmer`, `Selection`, `ShowData`, `Create`).
  - **Station steps are ordinary steps** in that list (category Station: interface
    services, network → NIC, Online, name, priority); Oops puts the old station settings
    back **on the station where it happened** (station settings themselves are never
    replicated).
  - **UI steps:** open, close, split, resize, move, window settings, views, screens (store,
    recall, delete, layout, ViewButtons) — one step each; a drag or resize is one step (the
    web client already saves once, on release). **Switching the active tab and scrolling are
    not steps.** The write-through (`screens::sync`) files them (today `Untracked`).
  - **Watch / Unwatch** are UI steps: watches live per user in `Runtime.Users.{n}`
    (session-wide), so they undo from any station.
  - **Programmer and selection:** `Reset`, `ResetAll`, `ResetSelection`, `ResetActive`,
    `Meow`, `Clear` file steps (Reset*/Meow are `ReadOnly` today — finding 2).
  - **Not undone:** running playback (Go, Off, faders, masters) and the mode toggles (Blind,
    Highlight, Freeze, Solo) with their Highlight/Solo masters — run state.
  - **Selection has its own subtree** (console probe below): `ShowSubtree::Selection(user guid)`
    = the fixture selection + the one-only selections (DataPool / Sequence / Timecode);
    `Programmer(guid)` = values only; `User(guid)` = login, role, profile link. A programmer
    step and a selection step restore independently. `SESSION_PROTOCOL` bump; no show
    format change.
  - **Reference extras, all four:**
    - `Oops <n>` undoes the newest n steps (the reference Oops menu runs `Undo <n> /NoConfirm`);
      `List Undo` numbers the entries. The Oops window (tap an entry = Oops up to it) comes
      with the touch UI kit (roadmap).
    - `/NoOops` (`/NU`) on any command: it runs, replicates and is journaled, but files no
      step.
    - **Create filters**: `CreateOops`, `OopsViews` (UI), `OopsProgrammer`,
      `OopsSelection` (Yes/No; CreateOops off greys the others out).
    - **`OopsConfirmation` Always / Main / Never** (default Never; the confirm pop-up, a
      yes/no prompt in a terminal). **Main** = the list by what the step did: create
      (Store of a new object, Patch, Create), delete (Delete, rm, Unpatch, Remove), Move,
      Insert, a file action (Import, load from a file), Layout (UI layout). Store over, Set,
      programmer and selection don't ask. Each `CommandDef` says whether it is Main.
    - The five settings live on the **UserProfile** (the current profile in the
      Oops menu), category Settings.
  - **Show load clears** every user's list (Load, New Show, taking the session's show), like
    the reference. Patch changes stay undoable (the reference clears on a patch change; we keep Keep Patch = one
    step).
  - **Console probe (live, profile 2, `Oops /NoConfirmation`):** `Fixture 1 Thru 3` → 3
    selected; `At 50`; OopsSelection off; `Fixture 5` → 1 (not filed); Oops → the value went,
    **the selection stayed 1**; Oops → the `Fixture 1 Thru 3` step went → 0. The console restores the
    selection separately from the programmer. The operator state was restored (selection
    via a temporary group; programmer values cleared by `ClearAll`).
- **11c built (2026-10-12).**
  - **Categories:** `OopsCategory` = Programmer, Selection, Show, Patch, UI, Users,
    Settings, Keys, Station (the old four removed); `OopsKind { category, main }` on
    `Undo::Tracked` (`registry::tracked` / `main`; `STORE_MAIN`, `PATCH_MAIN`, `KEYS`, `UI`).
    `dispatch::step_category` labels generic steps by what changed (Set on a fixture →
    Patch, on a user → Users, a station setting → Station; a programmer word that only
    moved the selection → Selection). `List Undo [/<category>] [/ThisStation]`: numbered,
    coloured label, `Main` mark, station and time.
  - **Selection subtree:** `ShowSubtree::Selection(user)` (`User::selection` /
    `set_selection`: fixture selection, grid, grid cursor, one-only selections);
    `Programmer(user)` = values + reset state (`User::programmer_values`); `User(user)`
    without both. A reference add/replace flag is in neither (either way
    one kind of step would roll the other back).
  - **Steps now filed:** Reset, ResetAll, ResetActive, Meow (Programmer), ResetSelection
    (Selection), Watch / Unwatch (UI), the screen write-through (`screens::sync`: UI, Main;
    a pane showing another tab files nothing — `only_tabs_moved`; an undone layout lays
    the live screens out again).
  - **Station-local state in a step:** `UndoFrame.local` (`LocalSnapshot`: Station
    settings — `station_config::StationImage` — or one user's watches, before + after,
    with the station id); dispatch diffs them around every tracked line; Oops puts
    `before`, Redo `after` back only on that station (`oops::restore_local`; members do it
    when the step reaches them in `apply_replicated_op`). A member's station step goes to
    the master as `Changes` with its `local` part (`SESSION_PROTOCOL` 6).
  - **Reference extras:** `/NoOops` / `/NU` stripped in dispatch (`AppState::no_oops`: runs,
    replicates, no step); profile `preferences.oops` (`OopsSettings`: `create_oops`,
    `oops_views`, `oops_programmer`, `oops_selection`, `confirmation`), set with `Set
    CreateOops|OopsViews|OopsProgrammer|OopsSelection Yes|No` and `Set OopsConfirmation
    Always|Main|Never`; the confirmation is asked in dispatch before the line runs or goes to
    the master (`oops::confirm_first`: UI pop-up or terminal yes/no, re-run with
    `/NoConfirmation`). `Oops <n>` as before.
  - **Show load clears** every list (`AppState::replace_show_data`).
  - The sweep skips the Oops check for a tracked line that filed no step. Tests:
    `oops_tests.rs` (categories, the console selection probe, /NoOops and filters,
    confirmation, station step, watches, layout + tab switch, load) and
    `a_members_station_step_is_undone_there_by_an_oops_on_the_master`. No show-format
    change (the profile settings have defaults); demo show and reference regenerated.
- **11d hardening (2026-10-12)** — read: this § ("Screenshots"), the roadmap (11d, views,
  DBG-08), the decisions Q&A, `tests/e2e/README.md`, `testing.md`, the frontend (xterm from a
  CDN, DOM renderer); a reference console's manual (screenshot options,
  `wvm_store_recall`), the system-test helpers (`system_test_helping_functions_images.lua`)
  and a live probe. Built as one step, go then commit.
  - **Reference console (researched):** Print Screen / F11 captures every screen as
    `YYYYMMDD_hhmmss_displayX.png` with a green flash; `Store Image 3.5 /Screen "1"
    [/ScreenOnly "No"] [/XResolution …]` stores a screen into the image pool
    (`ImageStoreSource` ScreenShot / NDI), `Export Image` writes the file; a view can store
    a screenshot as its appearance. The system tests loop displays 1–7 (`Store … /Screen=i
    /ScreenOnly='No' /NC`, then Export), hide the progress bar first, and compare against
    reference PNGs with `CompareImages` (limit 0.0001 × 255 × 3 × w × h, diff image
    exported). Live: `Store Image 3.999 /Screen=1 /NC` stored a ~79 KB PNG in the image
    object; an unquoted `/ScreenOnly=Yes` became the image's name. Probe image deleted.
  - **Capture in the client** (as decided): a vendored DOM-to-image library
    (modern-screenshot: SVG foreignObject → canvas → PNG) renders the screen as seen —
    nothing hidden — in every client (Chromium, Firefox, Safari/WKWebView in the macOS
    app, WebKitGTK on Linux, WebView2). **Vendored with xterm and its fit addon**: the
    frontend loads nothing from the internet (isolated rigs).
  - **`Screenshot [Screen <n>] [User <x>]`** (rights Programmer, reach Drive): bare = every
    open screen of the issuing user on every client showing them (the reference: all screens).
    Replies at once ("asked N clients"); each saved file is a System Monitor line + a notice
    with its path; no client showing that screen → coded error; a client silent for 10 s → a
    warning. **F11 and Print Screen** are default UI key bindings for `Screenshot`, with a
    short green flash on each captured screen after the capture.
  - **Files** `library/screenshots/<station>/YYYYMMDD_hhmmss_<user>_screen<n>.png`
    (station-local; a second one in the same second gets `_2`). An Image pool
    (`Store Image … /Screen`, view thumbnails) comes later from the same capture path.
  - **Dev script** `scripts/unix/screenshots`: a test station with the demo show; every
    window type, menu and overlay taken **from the registries** (new ones are picked up),
    opened by real commands in headless Playwright (a spec skipped in the normal e2e run)
    and captured through the real Screenshot path into `target/screenshots/`.
  - **Findings:** I run it and look at every capture (DMX sheet, Realtime window, the three
    menus, app home flow included); what's wrong goes on the roadmap for the UI kit; only
    crashes and blank windows are fixed in 11d.
  - **Later (roadmap):** visual regression like a reference image compare (reference PNGs, pixel
    diff, diff image) after the UI kit settles the look.
- **11d built (2026-10-12).**
  - **Capture board** in the shared `ScreenStore` (station-local): `viewer(no, joined)` —
    a browser counts as a viewer of its live screen while its `/api/events?screen=<n>`
    stream is open; `ask_capture` (ids, expected uploads per screen, listeners → SSE
    `{"event":"capture","id","screens"}`), `capture_arrived`, `overdue`.
  - **`Screenshot [Screen <n>] [User <x>]`** (`core/commands/screenshot.rs`; rights
    Programmer, reach Drive, scope Station): no open screen / no viewing client →
    E-CMD-010; replies "asked from N clients". `POST /api/screenshots?id=&screen=` (login,
    PNG only, ≤ 64 MB) → `ConsoleBridge::save_screenshot`: checks on the state thread that
    it was asked for and is the uploader's own screen (409 otherwise), writes the file on
    the HTTP worker through the drive door (`screenshot::write`), then a "Screenshot saved:
    <path>" message (sysmon + notice for whoever asked); `screenshot::tick` (housekeeping)
    warns after 10 s.
  - **Client:** vendored `apps/lumacore-app/frontend/vendor/` (xterm 5.3.0 + fit 0.8.0 +
    modern-screenshot 4.7.0, MIT, unchanged from npm, `README.md` lists them) — the UI
    loads nothing from the internet. `captureScreen` renders `document.documentElement` at
    the viewport size and device pixel ratio, uploads, flashes green (`.shot-flash`).
  - **Keys:** `RemoteShellCode::PrintScreen` (`CLIENT_PROTOCOL` 4; the page sends Print
    Screen on keyup, where Windows delivers it); vanilla bindings F11 and PrintScreen → core
    UI key `screenshot` (`Screenshot`); F11 no longer `cd output`.
  - **Dev script** `scripts/unix/screenshots` → `tests/e2e/tests/screenshots-all.spec.ts`
    (skipped unless `SCREENSHOTS_OUT`): demo show, every window type
    (`/api/window-types`), every menu a UI key opens (Press `Menu …`) and every overlay a
    menu tile opens, `EditSetting Show`, all through the real path; the app home flow
    (countdown, servers, users, quit) captured by Playwright directly. 17 files in
    `target/screenshots/`. e2e `screenshot.spec.ts` (vendored, capture + file + flash,
    F11, no client, unasked upload, layout reaches the page).
  - **Fixed (found by the captures):** the page listened for `live_screen_changed`, the
    station has sent `screen_changed` since the 11b rename — layout changes from the command
    line (`Store Window`, recalls) never reached a browser until a reload.
  - **Findings → roadmap (UI kit):** Realtime window Clock row shows `undefined µs` /
    `undefined ppm` (round trip, drift) on a standalone station; the DMX sheet's 20th column
    is cut off at the right edge; Backup menu AutoSave Interval says Off while the header
    counts down ("Autosave in 14:45") and the show setting is 15; Backup tile icons 🗁 🗋 ⭳
    have no glyph in the system font (bars; 🗑 draws) — the kit needs its own icons; home
    page "Clean start…" sits on the "On the network" heading.
- **After step 11** (roadmap Next): UI first — "touch friendly, implement all elements needed
  for future development": a touch-first UI element kit the later screens (playback,
  pools/editors, patch, feedback) are built from. Then §28 Effects and the active handoffs.
  This became step 12, [§30](#30-touch-ui-kit-2026-10-12-step-12-hardening).

---

## 30. Touch UI kit (2026-10-12, step 12, hardening)

"read all related md files, new and old … probe [the reference console's] docs and analyze [its]
systemtest lua files and [its] ui test plugin for what ui elements we are missing so we
can improve our ui and make it more flexible and make a good standard so it is easier to build
plugins and windows later … multiplatform and touch … window setting windows and prompt
dialogs or the network/lumasession windows." Research (five reports, kept unchanged):
[handoffs/ui-kit/research/](../../handoffs/ui-kit/research/) — LumaCore's UI today, a reference console's
elements (its layout files, manual), its systemtests + API showroom, open-source touch UIs and
web touch practice, theme editors (REAPER and others). Handoff: [handoffs/ui-kit.md](../../handoffs/ui-kit.md).
Q&A: [decisions-qa](../../handoffs/2026-10-10-decisions-qa.md) "Step 12 hardening".

**Way of working.** "we want to make this our way a little bit": after the decisions, **mockup
artifacts** of the elements and changed windows (element gallery; window chrome; dialogs +
settings; menus + theme editor) for the user to see before any go. While building, every UI
change is checked in Chrome and with the screenshot feature (`Screenshot`,
`./scripts/unix/screenshots`) against the approved mockup.

**Findings (why).** The data side is server-driven and good; the view is one hand-written
4600-line `index.html` (about 56 `innerHTML` templates, 61 raw hex colours, ~10 button families,
four tab and five confirm implementations, four ways to write a setting, emoji icons). Touch
breaks: split resize is mouse-only, right-click is the only way to split / settings / close /
pool menu (iOS never fires `contextmenu` on a long press), close × only on hover, 18–26 px
targets, 8–11 px fonts, no `@media`, the soft keyboard covers the command line bar. EditSetting
gets typed props flattened to text (`SettingField.kind` a string). Menus poll every 1 s with
ad-hoc JSON. A reference console builds most of its UI from property-bound cells (about 2200 uses vs 786 plain
buttons) whose editor follows the property type, one MessageBox for every prompt, one pop-up
layout for 77 pick lists, one setup-menu layout, ≥ 40 px targets, no hover.

**Decided.**

- **Step 12, parts 12a–12g**, each hardened → mockup → go → commit:
  12a foundations · 12b dialogs · 12c settings + window bar + `Self` · 12d menus ·
  12e touch layout · 12f controls · 12g theme editor (roadmap Now).
- **Client tech: plain custom elements** as ES modules in `apps/lumacore-app/frontend/kit/`
  (`luma-button`, `luma-swipe`, `luma-cell`, `luma-fader`, `luma-encoder`, `luma-keypad`,
  `luma-keyboard`, `luma-list`, `luma-popup`, `luma-dialog`, `luma-window-bar` …). No framework,
  no build step, nothing from the internet; updated in place by id (never re-rendered under a
  finger). One **pointer core** for every gesture (Pointer Events, pointer capture, one entry per
  pointer, `pointercancel` / `lostpointercapture`).
- **Shape from the registries.** The server sends typed **descriptors** built from what already
  declares them (`TreeDef::write` props, `WindowType` settings, `PoolDef`, `UiEvent` dialogs): kind,
  id (GUID or path), label, value, range / steps / choices, unit, rights (disabled, not hidden, for
  a Guest), gestures → command lines. The editor follows the type (auto-created from properties):
  choice → swipe button / pop-up, bool → toggle, number → keypad (calculator), link → pick-list
  pop-up, text → text input. Every gesture is a command line; no client-only state. Plugins
  publish descriptors, not DOM code.
- **Look: ours, proven sizes.** LumaCore's dark + amber identity and cat names; established proven sizes
  (50 px title rows, 60 px action rows, ≥ 40 px targets, 44–48 px on coarse pointers) and dialog
  patterns. Own **SVG icon set** (a vendored MIT/ISC line-icon subset as one sprite + our cat
  glyphs); no emoji in chrome.
- **The look follows the user's own console concept** (`inspiration/`, analysed in
  [inspiration-style.md](../../handoffs/ui-kit/research/inspiration-style.md)): "the general ux
  concept and ui design is good. But it looks a little bit too much vibecoded and is lacking some
  of the original design i have from inspiration … Dont change the design too much." Matte,
  flat, pure neutral greys (window `#121212`, strips `#1c1c1c`/`#262626`, cells `#303030`,
  lines `#424242`, text `#d1d1d1` / `#9c9c9c` / `#787878`), **olive gold as the one accent**
  (`#7c6200` dot, gold lanes, frames, encoder rings), status colours only as small marks (green
  online dot, red "changed" corner dot); no shadows, no gradients, radius 4–8 px, large regular
  type, sentence-case labels. **Every tab carries a gold dot** (bright on the active tab, dark on
  the others) instead of a colour per window type. The **window bar looks like the inspiration's
  option row**: small square checkboxes and `Label: value ▾` drop-downs on the window background,
  not boxed buttons. This palette is the **Dark** preset.
- **Touch rules.**
  - One finger scrolls lists, sheets and pools (native momentum); faders, encoders and XY
    capture the finger at once; two fingers also scroll anywhere.
  - **Hold = "show me more"**, one meaning everywhere (long press with a progress ring, small
    movement slop; right-click stays for the mouse): hold on a pool object → the full-screen
    selector (today's feature) with an **action row** on top (Edit, Label, Store, Delete, Move,
    Copy, Settings: tap the action, then the object — command-first like the command line); hold
    on a tab / window header → the window menu. No swipe-out actions.
  - **The tab dot is the window's header button**: tapping the dot opens the window menu (a
    44 px hit area round the 7 px dot) — no extra header chrome, to save space.
  - **Faders and encoders: established rules.** A tap never jumps a fader, a drag moves it relative;
    double-tap or hold opens the keypad; several faders with several fingers; encoders are drag
    style (drag opens a pop-up fader with ± and fine), no circle-rotate; one finger sliding across a
    bank only where the window allows it.
- **Settings apply at once**: each change runs its `Set` line, is one Oops step, the dialog
  stays open; no OK / Cancel.
- **Keyboards.** OS keyboard for text on phones and tablets (the command line bar kept above it
  through `visualViewport`), our own **keypad / calculator** for every number, and our own
  **built-in on-screen keyboard** ("we also need a vk for consoles": console and kiosk screens
  have no OS keyboard). A per-screen setting `Keyboard: Auto | OS | Built-in | Off` picks (Auto =
  built-in on console / kiosk stations and touch screens without one, OS on phones and tablets);
  the VK button on the command line bar toggles the chosen one.
- **Window settings shell (12c)**: one shell for every window type — title, tabs from the schema's
  sections, cells generated from the schema, Delete Window, Close; applies at once. **Load / Save
  presets** of a window's settings (on the user) for other windows of the same type.
- **Window bar** (12c): a row of buttons **inside the window, below the tabs — never next to the
  tabs**. Items: **pinned settings** (live value; a tap edits in place) and the window type's own
  **actions** (command lines with `Self`, e.g. sysmon Pause / Clear, DMX universe ±) — they replace
  today's hand-built sysmon and DMX toolbars, so there is one way to edit. Pinned from the
  settings shell (pin per cell; a Window bar tab orders and removes), defaults declared per
  window type in its registry, and by command. Stored as ordinary window settings
  (`WindowBar On|Off`, `WindowBarItems` = ordered ids): in views and presets, `Set Window n …` /
  `Set Self …`, Oops-able as UI steps. Shown per window (type default, a profile default for new
  windows); **auto-hidden** when the window is too small (shorter than about 4 rows, narrower than
  the bar's minimum); items that don't fit go into a `…` at the end; while hidden, the window menu
  (tab dot) lists the bar items.
- **`Self`** (12c, with the window bar; one registry word): the object the command line belongs
  to — a macro line → that macro, a window-bar or window-menu action → its window, a UI key → its
  window or the key, a plugin → the plugin; typed on the command line → the focused window; no
  owner → a coded error, never a guess. Resolved when the line runs. Later (with ShowVar / UserVar,
  §5): property reads `Self.<Property>`, e.g. `Set $macroname Self.Name` in a macro.
- **Menus (12d): a setup layout, ours.** Network / DMX Protocols / LumaSessions / Backup share
  one layout: status in the title bar, tabs on the left, the content (station grid, line sheet) in
  the middle, a function block of 60 px action buttons + bound setting cells, a big state-coloured
  power toggle; pushed over the existing SSE / WS (no 1 s polling); every button stays a command.
- **One standard dialog** (12b, the message box shape): title, text, inputs (with keypad kind),
  toggles, selectors, command buttons, optional countdown; returns all answers together. Replaces
  `openAsk`, `openDecision`, `openClaimAsk`, `confirmAction`, `promptText`. One **pick-list
  pop-up** (filter, None, Add New) for every link.
- **Themes** (tokens in 12a, editor in 12g): REAPER-style, not a reference console's 1166 colour entries. **14
  editable palette colours** (background, surface, raised, line, text, text-dim, accent, info, ok,
  warn, alert, programmer, blind, park) + **5 adjusters** (brightness, contrast, saturation, tint,
  accent); every other colour (hover, pressed, alt row, focus, terminal colours) is derived. Pick an
  element on screen to find its colour, reset per colour, A/B, import / export JSON; built-in
  presets Dark, Daylight, Night. Stored **on the user** (`ShowData.Users.{n}`, only the changes;
  replicated, saved with the show); a drag previews in the client and files one Oops step (UI) on
  release. Tidy rule: no raw colour outside the theme module. **Station override never touches a
  user's theme**: a station-local setting and a hold / toggle button let anyone at a desk see the
  normal theme everyone is used to, for that station's screens only.
- **Tests.** A registry-driven **Showroom** window type (every kit element in every state; also the
  screenshot set and the plugin reference). Playwright projects **tablet** (touch, 1024×768) and
  **phone** (touch, 390×844) next to desktop, plus a **WebKit** project; kit elements carry
  `data-kit` / `data-id`; `tap()` checks the element is really hit (an exact click target).
- **Fixes folded in (12a)**: the 11d findings (Realtime `undefined µs/ppm`, DMX last column,
  Backup AutoSave Off, Backup icons, home "Clean start…" overlap).

**Design locked (2026-10-12).** "Yes lock the design and store it in the repo for reference as
artifact html … so we can change out mind later if needed and have good documentation of all
this." The 14 approved boards are kept in
[docs/dev/ui-kit/mockups/2026-10-12/](../dev/ui-kit/mockups/README.md): the canvas sources
(`*.dc.html` + `canvas.json`), a small offline runtime (`support.js`) so they open and work in any
browser from the repo, and PNG snapshots (`snapshot.mjs`). A later change copies the folder to a
new date and records why here; the locked folder stays unchanged.

**Strict rule — everything is built from kit elements (2026-10-12).** "Make it a strict rules
that all windows etc needs to use elements. So if i want a new type of window that is missing
elements we need to design the elements first with artifacts." Every window type, menu, dialog,
pop-up, bar and plugin UI is composed only of kit elements (`frontend/kit/`); no hand-built
controls or markup outside the kit. A window that needs something the kit lacks waits: the new
element is designed first as a **mockup artifact**, approved by the user, added to the kit and the
Showroom, and only then used. Checked by a tidy rule (no control markup, `innerHTML` templates or
raw colours outside `kit/`; windows only compose `luma-*` elements) once 12a lands; existing code
is exempt only until the part that owns it replaces it.

**12a hardening (2026-10-12)** — after "Yes this is a very good start" on the restyled mockups:

- **Migration: foundations everywhere, elements per part.** 12a adds `frontend/kit/` + the
  pointer core, moves every colour to tokens (tidy rule: no raw hex outside the theme module),
  replaces emoji with own icons, lays the touch baseline (`touch-action`, `dvh`, safe areas,
  coarse-pointer sizes, `:focus-visible`, `@media`), hold ring + tap-the-dot window menu, test
  hooks and the 11d fixes. Old dialogs / menus / settings keep working until the part that owns
  them replaces them (12b dialogs, 12c settings, 12d menus, …); after 12a the UI wears the
  inspiration palette.
- **Icons: Lucide** (ISC), one SVG sprite with only the icons we use, plus our own cat glyphs
  (paw, PURR) in the same stroke.
- **Font: vendored Source Sans 3** (OFL, Myriad tradition) for the UI and **JetBrains Mono**
  (OFL) for the command line and terminals — same look on every platform and in screenshots.
- **Hold time: the profile hold time, default 500 ms** — one timing for keys, pools and menus;
  the ring fills over it.
- **Theme storage: `Set` on tree paths.** `ShowData.Users.<n>.Theme` = `Preset`, `Adjust`
  (Brightness, Contrast, Saturation, Tint, AccentHue), `Palette` (only changed colours), declared
  in `TreeDef::write`: `Set User Admin Theme Preset Night`, `Set User Admin Theme.Accent
  "#9a7a08"`, `… Default` resets. No new keyword. Show format 7 (forward only), demo regenerated.
- **Station override**: station-local `ThemeOverride Off | Dark | Daylight | Night` (never
  replicated, never writes a user) and a UI key `NormalTheme` (hold = momentary, tap = toggle, this
  station's screens only). Default binding: "We should think about screenlock as well with this so
  default would be screenlock with a F key and meow-F is normal theme" → an F key locks the screen,
  Meow + that F key shows the normal theme; a reference desk lock researched before deciding (below).
- **DeskLock** (researched: a reference desk lock is a separate full-screen overlay — Pause / F9 or the
  Shutdown menu's Lock, unlock with the same key or the four-corner tap pattern, no password, per
  station, UI only; elsewhere a `Lock` keyword locks pool objects): a mode-style object `DeskLock`
  (`On DeskLock`, `Off DeskLock`, bare toggles), station-local, never replicated, not an Oops step;
  `Lock` / `Unlock` stay free for an object lock. **F9 and Pause** toggle it; **Meow+F9 =
  NormalTheme** (hold = while held, tap = toggle) and works while locked; today's F9 `cd fixture`
  shortcut is dropped ("Drop it"; typing `cd fixture` still works). Unlock with the same key or the
  corner taps (top-left, bottom-right, top-right, bottom-left). Scope: every screen of the
  station; playback, DMX, the shell and other stations keep running. Built in 12a (mockup first).
- **One Mac-style surface for connect, login and lock** ("A fullscreen lockscreen. Image stored on
  the user as background … the lockscreen should have switch user capability. Mac style. The thin
  client connect to server needs to be unified with the lockscreen and we need a artifact for this
  design"): full screen over the user's background image — big clock and date, station and
  LumaSession / show, the current user large with a password field (if the user has one) or "tap to
  unlock", the show's other users as round avatars (initials + their accent, rights badge),
  `Servers…` bottom-left in the app (discovered servers, address, start local server — today's home
  page content in the same style), a small status line (DMX output, LumaSession role, unsaved
  changes). **Switch user** = this station's screens log in as the picked user and unlock (macOS
  fast user switching); the previous user's work stays on their user, their other stations are
  untouched. **Lock-screen image** on the user: `ShowData.Users.<n>.LockScreen` {Image, ShowClock,
  Text}, the image inside the `.luma` container (`images/<sha256>.<ext>`, like GDTF files), linked
  by GUID, replicated; set from the lock-screen settings or `Set User Admin LockScreen.Image
  "file.png"`; show format 7 (same bump as the theme). No image (or no user yet) → a built-in
  background from the user's console concept.
- **Token names**: CSS `--lc-bg`, `--lc-surface`, `--lc-raised`, `--lc-line`, `--lc-text`,
  `--lc-text-dim`, `--lc-accent`, `--lc-info`, `--lc-ok`, `--lc-warn`, `--lc-alert`,
  `--lc-programmer`, `--lc-blind`, `--lc-park`; derived `--lc-hover`, `--lc-pressed`,
  `--lc-alt-row`, `--lc-focus`, `--lc-on-accent`, `--lc-disabled`, `--lc-gap`. In the show:
  `Theme.Palette.Background`, `Surface`, `Raised`, `Line`, `Text`, `TextDim`, `Accent`, `Info`,
  `Ok`, `Warn`, `Alert`, `Programmer`, `Blind`, `Park`.
- **One surface replaces all three logins**: the desktop app's home page (servers → users), the
  web page's own login and DeskLock; the old `home.html` and login markup go.
- **12a is split**: **12a1 foundations** (kit/ + pointer core, tokens + fonts + Lucide, touch
  baseline, hold ring + tab-dot menu, Showroom, test hooks + tablet / phone / WebKit projects,
  tidy rules, 11d fixes) and **12a2 user surface** (Theme + LockScreen on the user, format 7, demo
  regenerated; station override + NormalTheme; DeskLock; the one lock / login / connect surface).
  Each is built on go and committed on its own.
- **Pool action row (12e)**: as the mockup — Edit, Label, Store, Delete, Move, Copy, Settings.
- **Tests**: the Showroom window type exists from 12a and grows with each part
  (`./scripts/unix/screenshots` captures it); Playwright tablet (1024×768) and phone (390×844)
  touch projects run the specs a part touches; the WebKit project is added in 12a (asks before
  `npx playwright install webkit`).

**12a1 built (2026-10-12).**

- `frontend/kit/`: `tokens.css` (the theme — 14 palette colours per preset Dark / Daylight /
  Night, derived roles by `color-mix()`, sizes larger under `(any-pointer: coarse)`), `theme.js`
  (presets, tokens as concrete colours, xterm theme), `pointer.js` (`hold` with the ring, `drag`
  with pointer capture), `icon.js` (`<luma-icon>`, sprite inlined), `button.js`
  (`<luma-button>`), `menu.js` (the window / context menu), `showroom.js`, `kit.css`. The page
  boots after `window.LumaKit`.
- Vendored Source Sans 3 + JetBrains Mono (latin woff2, OFL) and a Lucide sprite of the names in
  `vendor/lucide/icons.txt` plus own glyphs (paw, showroom), built by
  `scripts/unix/lucide-sprite.mjs`.
- `index.html` on tokens: no raw colour left; flat (no shadows), windows separated by the dark
  gap, gold frame on the focused window; fonts ≥ 12 px; every tab carries the gold dot — tap =
  window menu (Split right / down, Add window…, Change type…, Rename…, Settings…, Close), hold on
  a tab or the header = the same menu; × only on hover for a mouse; split handles drag with the
  pointer core (wider band on touch); pool buttons: a ring while held, one finger scrolls (drag
  to move stays the mouse's); view bar hold uses the kit's hold; status, mode and feedback icons
  are kit icons; the page follows `visualViewport` (soft keyboard), safe-area insets,
  `viewport-fit=cover`; small screens hide the view bar. Window types carry kit icon names; pool
  For All / For Some colours default to the theme (Ok / Warn). New window type **Showroom**
  (no. 7, Tools). The home page takes the same tokens (the app puts `tokens.css` in).
- Tidy: `frontend_colours_only_in_the_theme`, `frontend_icons_come_from_the_kit`.
- Tests: Playwright projects desktop / tablet / phone / webkit; `kit.spec.ts` (kit loads, tab dot
  menu, hold menu + ring, Showroom + presets, split drag saves, touch sizes); `tap()` / `hold()`.
- 11d findings fixed: Realtime `undefined µs/ppm` (— when missing), DMX columns fit the window,
  Backup AutoSave drawn from the status at once, Backup icons, home "Clean start…" overlap.
- **Found while verifying:** the user's Chrome runs Dark Reader, which recolours pages —
  LumaCore draws its own themes, so both pages carry `<meta name="darkreader-lock">`; the
  screenshot capture lost `<use>` icon styles and `<select>` values (icons now carry stroke
  attributes, the capture marks chosen options); a hold must swallow the next click but a
  right-click must not (it ate the menu's first click). On a phone the one-row command line
  shows nothing while the prompt is longer than the bar (→ 12e).
- **Layout description ("our layout files")** asked mid-build: "[The layout format] needs to be decided and what
  format to use" → "Research other projects on github that have something like [it] and give me
  options and reasons. We want something that is easy for the users to understand and easy to
  understand later when doing lua plugins" → research in
  [layout-format.md](../../handoffs/ui-kit/research/layout-format.md) (recommendation: KDL files
  on disk, JSON on the wire, Lua tables for plugins; fallback all-JSON); decided in the **12c
  hardening** with a mockup.

**12a2 / 12a3 hardening (2026-10-12)** — "Lets do 12a2, load and find all md files that has
features related to it … then we harden it". A sweep of every md file (old and new) found the
lock image inside the `.luma` against §8 (media outside), stale format numbers, no rights rule for
a user's own theme, and no answer to which screens a lock covers or how files reach other
stations. A reference console's library and show file were researched (one folder per object kind, media
embedded in the show with a GUID, profile exports inline every dependency — once per reference).
**Supersedes** in "12a hardening" above: theme and lock screen on the *User* (now the profile),
"station override for this station's screens" (now the device), the 12a2 scope (now split).

- **Split**: **12a2 = the show format** (format 7), **12a3 = the user surface**. Each built and
  committed on its own.
- **App architecture.** "the desktop app needs to be multiplatform … so we can port it later as a
  ios and android app. But also http available" → "desktop apps/mobile apps are just basic
  clients with server discovery and connection that gets lockscreens etc that opens the http
  page. A webbrowser gets the http directly." The station serves the lock / login surface (and
  everything else) over HTTP from `frontend/`; the apps only discover, connect and open that
  page. Before any connection the app shows a **small bundled connect page** built from the same
  kit (background, clock, the server list: discovered, address, start local server, Clean start,
  the version prompts of §23). Platform work goes through a small host bridge
  (`window.LumaHost`); a browser lacks it. The shell stays **wry / tao** (both run on iOS and
  Android); Tauri 2 is decided when the mobile port starts. Phones run no local server.
- **`.luma` layout (format 7)** — "a .luma file contains a folder system that represents the
  showdata tree and each image has a manifest for its GUID". A zip:
  `manifest.json` first (format, `show_id`, commit, every entry with sha256 and size — read before
  anything else), then one file per top-level ShowData field, a **folder per pool member** (data
  pool, user, profile, fixture type, image) and **inside a data pool one file per pool kind**
  (`DataPools/1 Default/{datapool,Groups,Presets,Sequences,Macros}.json`). Folder names are
  `<no> <name>` for reading; the loader trusts only the number and the GUIDs inside. Files are
  pretty JSON (the layout-format question of 12c is about window layouts, not the show).
- **Media in two tiers** (rewrites §8). **Embedded**: images, icons and GDTF files live inside the
  `.luma`, ≤ **16 MB a file**, ≤ **256 MB embedded per show** (import refuses past it with a code
  that points to asset drives). Each embedded file sits **beside its manifest** (`image.json`:
  GUID, name, sha256, type, size, w×h, source name); the station keeps one copy per sha256 in
  `library/media/` and in memory, so only the zip may hold a picture twice (a profile stays
  self-contained). **Asset drives** for large show-critical media: the show names `AssetRoots`
  (GUID + name), each station maps a root to a folder / drive like a network to a NIC
  (station-local), an asset manifest is root + relative path + sha256 + size; missing assets are
  listed, never a crash. 12a2 builds roots, mapping, manifests and the missing check; preload into
  memory and §8's operator-approved sync come later.
- **Files reach other stations by hash**: the tree replicates manifests; a member missing a file
  fetches it from the master by sha256 over the LumaSession stream (chunked) into its store;
  browsers read `GET /api/media/<sha256>`. GDTF files travel the same way (members can save).
- **GDTF identity**: "we need both our own guid and the gdtf guid for later use when for example
  downloads a global update". Links keep LumaCore's own GUID; the fixture type also stores the
  file's `FixtureTypeID`; a re-import (or a later downloaded update) of the same FixtureTypeID adds
  a **Revision** to that fixture type instead of a second type. In the tree:
  `Devices/FixtureTypes/<no> <name>/fixturetype.json` + `Revisions/<n>/source.gdtf`;
  `library/fixturetypes/` stays the station's cache.
- **The profile carries the look**: Theme (Preset / Adjust / Palette), LockScreen (Image,
  ShowClock, Text, AutoLock minutes, default Off) and two pools, **Icons** and **Images** ("we want
  to be able to imprt icon-sets later so we want two pools"), so "import the userprofile into
  another show and it should look the same without the need to import any more objects". Profile
  UI links only to its own pools. The User keeps identity (name, password, rights, programmer,
  selection, which profile). A user who wants their own look has their own profile. **Rights**:
  users of that profile and Admin may change it (one Oops step, category UI).
- **Icons**: the show has **no icons** — the ShowData icon pool goes. A **standard icon set ships
  with LumaCore** (Lucide + cat glyphs), the same on every station; names are never removed, so a
  name is a stable identity for show data and profiles. Custom icons and imported icon sets live
  in a profile's Icon pool. Custom pictures for show data go into the show's **Image** pool.
  Copying works both ways (`Copy UserProfile Icon 3 At Image 10`, new GUID).
- **Syntax rule: owner first**, like `DataPool 2 Group 3` and the tree paths:
  `Import Image "stage.jpg"` (show pool), `Import UserProfile Image "bg.jpg"` (bare
  `UserProfile` = your own), `Delete UserProfile "Bob" Icon 3`. Files come from `library/media/`;
  a browser upload lands there and runs the same line.
- **Profile export / import** (in 12a2): `Export UserProfile "Bob"` →
  `library/userprofiles/Bob.lumaprofile`, a zip with the same layout (manifest +
  `UserProfiles/<n>/` with its pools); `Import UserProfile "Bob"`: same GUID = replace (asks),
  else a new profile. *(Built as "needs `/Overwrite`"; the ask comes in 12b, §30 "12b
  hardening".)*
- **12a3 surface**: one surface for lock, login and connect (the locked Lock / LockPhone boards).
  **DeskLock locks the device it's pressed on** (every window of that app, or that browser tab);
  `On DeskLock` from a shell locks the client it came from, `On DeskLock Station` every client of
  the station. Screen 1 shows the full surface, other screens the lock image blurred. Unlock: a
  user without a password with F9 / a tap / corner taps; with one, their password. "**admin:admin
  always works**, this is because if the LD/operator leaves, anyone needs to be able to login that
  is a poweruser" — a fixed fallback login on every station (isolated networks, §19). The
  **built-in keyboard** moves from 12b into 12a3 (the surface needs it on consoles). **F9 / Pause
  are core keys** (Bind may add keys, never take these) plus a Lock tile on the touch sidebar and
  in the Shutdown menu. Normal theme (Meow+F9, `ThemeOverride`) = the **factory preset**, nothing
  of the profile; it also applies per device. Start: the surface for the last server with the last
  user picked; no password → logs in after the 5 s countdown; Logout → the surface, nobody picked;
  no automatic Guest. Servers… in a browser lists the stations the serving station sees.
  **Auto-lock** after idle minutes is a LockScreen setting (default Off).

**12a2 built (2026-10-12)** — the show format; no UI change (the look is applied in 12a3).

- **Format 7** (`FORMAT_VERSION` = 7, `core::show_file`): the `.luma` zip holds `manifest.json`
  first (format, kind, show_id, commit, every entry + SHA-256 + size), one JSON file per
  top-level field (`FILES`), folders per pool member (`FOLDERS`: Devices, DataPools, Users,
  UserProfiles, Images; `<no> <name>`, only the number is trusted), data pools one file per
  pool kind, users `user.json` + `Programmer.json` + `Selected.json`. A changed or missing JSON
  part is E-SHOW-005; a damaged picture is left out; format 6 (one `show.json`) is refused with
  its number (E-SHOW-002). `decode` puts the tree together in the show's field order.
- **Media** (`lumacore_tree::media`, `core::media`): `MediaFile` manifest (GUID, no, name,
  sha256, ext, size, width × height from the header, source); `ShowData.images` (the show's
  pool; the ShowData icon pool and `ShowIcon` are gone, `IconRef::Image` links a show picture);
  `UserProfile.{theme, lock_screen, icons, images}`; store `library/media/.store/<sha256>.<ext>`
  (gitignored), import folder `library/media/`; 16 MB a file, 256 MB per show (E-MEDIA-001…004).
- **Words** (`core/commands/media.rs`, owner first): objects `Image` (show pool), `Icon` (your
  profile's), `AssetRoot`, `Asset`; `UserProfile [<n>|"name"] Image|Icon …` routes through the
  UserProfile object; new object verbs `Import` / `Export` (plain `Import "x.gdtf"` and
  `Export Fixture 1 "f.json"` keep their forms; Import's undo is refined from what changed).
  `Set UserProfile [<n>|"name"] Theme.Preset|Theme.<Adjuster>|Theme.Palette.<Colour>|Theme
  Default|LockScreen.Image|ShowClock|Text|AutoLock|LockScreen Default` through `TreeDef::write`
  (`PropKind::Colour`; `WriteDef::may`: users of the profile and Admins). Adjusters:
  Brightness, Contrast, Saturation, Tint −100…100, AccentHue −180…180.
- **Profile files**: `Export UserProfile ["name"]` → `library/userprofiles/<name>.lumaprofile`
  (same layout, kind `userprofile`); `Import UserProfile "name"` keeps the GUIDs, the same GUID
  needs `/Overwrite`, a new profile needs an Admin, GUID clashes refuse (E-SHOW-003).
- **GDTF**: `FixtureType.gdtf_fixture_type_id` from the file; the same FixtureTypeID imported
  again adds a Revision (the same file: "already has this file"); files in
  `Devices/FixtureTypes/<n>/Revisions/<r>/source.gdtf`.
- **Asset drives**: `ShowData.assets` {roots, files}; `StationSettings.asset_roots` (name →
  folder, station-local, in `station_settings.json` and station Oops steps); `Set AssetRoot
  "Video" Folder …`, `Import Asset "Video" "clips/intro.mov"` (SHA-256 read in pieces on the
  one background worker, never the state thread; the result is a follow-up line
  `Import Asset <root GUID> "path" "name" /Hash <sha> /Size <n>` run as the user — a late
  one imports anyway; R1-70, Q42), `List Asset [/Missing] [/Verify]` (runs on the station
  that asks, also a member: it checks its own drives; `/Verify` hashes in the background and
  reports to the System Monitor). Preload and the approved sync: later.
- **Files travel by hash**: `SESSION_PROTOCOL` 7 — a member asks `FileWant` for pictures and
  GDTF files its stores lack (once each, after a join and after ops touching Images, profiles
  or the patch); the master answers `File` (base64, checked by SHA-256, written on the stream
  thread). HTTP: `GET /api/media/<sha256>` (open, immutable), `POST /api/media?name=` (login),
  `/api/me` carries the profile's `look`.
- **Tests**: `show_file` (tree round trip, pictures, damaged part, old formats, folder names),
  `media_tests` (images, look and rights, profile move, assets, Oops), link / stream file
  exchange, e2e `media.spec.ts`; demo regenerated; reference regenerated (`Export` visible).
- **Uploads on a member** ("a imported image always goes to the master and distributed on the
  lumasession network. Imported assets like big movie files gets uploaded the the http server
  we are connected to"): `POST /api/media` saves on the station the page is open on; a member
  also sends the file to its master (`Upload` frame, ahead of the forwarded `Import` line on the
  same stream), the master keeps it in its `library/media/`, runs the line, and every station
  fetches the picture by hash. Master lost meanwhile: the Import fails, run it again by hand
  ("Fail, retry by hand"; the file stays on the member).
- **Every station keeps everything embedded**: "gtdf files and patch and all embedded files is
  a strict need for any server to work. Only asset linked services/items gets disabled/offline
  so it does not render it if assets is not present." Members fetch every picture and GDTF file
  at once (as built). Asset uploads over HTTP into the open station's root and taking
  asset-linked items offline are roadmap items ("put this on roadmap … just do basic work now
  so we can continue on ui").
- Not done here: the store keeps no in-memory copy yet.

**12a3 hardening (2026-10-12)** — "Lets do 12a3. Load and analyse all md files related to it and
also old md files so we can harden". A sweep of every md file, the Lock / LockPhone / Keyboards
boards, the e2e specs and the code found: no notion of a device (one connection per window), a
reload that logs straight back in, pre-login data that is login-only or missing (looks, presence,
status), no Pause key and no core-key guard against `Bind`, every e2e spec relying on instant
auto-login, and stale wording (station-wide lock, theme on the User, "Admin stays passwordless").
**Supersedes**: F9 `DeskLock` *with* F1–F10 free (below), "Admin stays passwordless" (§19), the
avatar colour from the profile accent.

- **Device**: the app gives one device id for all its windows (kept in `client.json`), a browser
  tab one per tab (`sessionStorage`); every connection names its device. The station keeps each
  device's **lock** and **normal theme** state (station-local, never replicated), so a reload or
  an app restart comes back locked. `On DeskLock` locks the device the line came from, `On
  DeskLock Station` every device of the station; the CLI is not a device.
- **Admin**: "Admin user always has admin as password and cannot be changed. It always asks for
  it." `Set User Admin Password` is refused (coded); the surface always asks Admin's password; a
  fresh station waits for it. A LumaSession password is a roadmap item.
- **Pre-login data**: `/api/station` and an open event stream carry the users (name, role, password
  yes/no, remote login, colour, profile name, where they are logged in — "A user can be logged in
  on multiple stations at the same time"), the station's status line and each user's lock picture.
  Members pass their logins to the master, the master shares them with every station.
- **Countdown everywhere**: browser and app show the surface with the 5 s countdown for a last
  user without a password; a tap on the user, Enter or Space logs in at once, Esc or a tap elsewhere
  stops it. Tests skip it with `?autologin=now`.
- **Switch user**: the whole device switches (the app re-lays its screens out to that user's
  config); the previous user's token on this device is revoked, their other logins stay, their
  work stays on their user.
- **While locked**: nothing is shown over the lock; asks run to their timeouts and defaults; the
  surface shows a small "n waiting" mark and unlocking shows what is still open. A Shutdown of
  this station shows its countdown on the surface. The page forwards only the lock keys and the
  station refuses any other line from a locked device (coded); shells, other devices, OSC and
  playback run on.
- **Built-in keyboard** (a kit element, the Keyboards board): a keyboard button in the password
  field opens it; a device marked **Kiosk** (app setting in `client.json`) opens it at once. QWERTY
  as the board (command-word row, 123 page, Esc, Hide); password mode hides the word row and masks.
  National layouts and the per-screen `Keyboard` setting stay in 12b.
- **Normal theme**: a mode-style word `NormalTheme` (`On` / `Off` / bare toggle; held key = while
  held), kept per device on the station; `Set Station ThemeOverride Dark` is the default for every
  device of the station; it shows the factory preset, nothing of the profile.
- **Keys — "Only F keys. No normal letters for desklock and keyboard shortcuts, like [established consoles] … F1 to F10
  is reserved for executors/user assignments later … F9-F12 core features, desklock, screenshot,
  hide/show all sidebars … Lets define the core buttons now"**: F1–F8 (any modifier) stay free;
  core keys, never unbound by `Bind`:

  | Key | Does |
  |---|---|
  | F9 / Pause | DeskLock (lock / unlock) — as established |
  | Meow+F9 | Normal theme (held = while held) |
  | F10 | Shortcut mode KBD / PRG / NAV — as established, F10 toggles keyboard shortcuts |
  | Meow+F10 | View bar only (was Meow+V) |
  | F11 / Print Screen | Screenshot — as established |
  | Meow+F11 | Shutdown menu (was Meow+X) |
  | F12 | Hide / show all sidebars and the view bar of the window it's pressed in (nothing else) — *since 12b2: also the top bar and the command line bar (full screen of the windows), see "12b2 hardening"* |
  | Meow+F12 | Native OS full screen of that window on its display (macOS native full screen, its own Space; Windows / Linux their real full screen, never just maximize); in a browser the browser's full screen. A panel filling its window stays on hold of its tab (no key) |

  Researched in a reference console's manual: F9 / Pause
  desk lock, F10 shortcuts on / off, F11 / Print Screen screenshot are system-wide and not
  editable; F1–F5, F7, F8 open menus (At filter, menu selector, command, masters, playback,
  custom master section, encoder bar); F6 and F12 are unused; Shift is the modifier key; everything
  else is per-profile Keyboard Shortcuts (with executor index). "Research [the reference] F buttons. I want
  F12 as hide show sidebar/fullscreen" → the map above.

  Dropped defaults: F1 help, F2 list, F3–F5 list fixture / universe / output, F9 cd fixture, F10 cd
  universe, Shift+F10, Meow+B / V / X (typing the words still works). Ctrl+L / Ctrl+H stay
  (terminal).
- **Avatars**: the user's **own colour** (new `User.Color`, show format 8; a new user takes the next
  of 12 fixed calm colours, Admin the gold accent; `Set User "Bob" Color "#758ba0"` by that user or
  an Admin) and a badge with the **profile's name** ("LD"). Initials are the first two letters.
- **Servers… in a browser**: picking a station opens its page and its own surface (no login carried
  over); the last user is remembered per station.
- **Screens**: the open window with the lowest screen number gets the full surface; the others show
  the blurred picture and the clock; tapping one moves the surface there.
- **Dialogs**: the approved standard dialog (Dialogs board) is pulled into 12a3 as a kit element,
  first used by the connect page and the surface (Clean start, §23 prompts, wrong password); 12b
  moves the old prompt patterns onto it.
- **Mockups first**: one new dated board set for what the locked boards don't show (countdown,
  nobody picked, wrong password, remote-login-off users, the keyboard on the surface, a blurred
  second screen, the Lock tile, Clean start and the §23 prompts, the key list), approved before go.
- **Screens as OS windows** ("The desktop apps needs to be able to open screens as individual os
  windows that we can resize however we want and move them to any display on local computer"):
  kept — one OS window per screen, any size, any display. Each screen remembers its **display**
  (the display's name and position) and its size and place on it, per server and user
  (`client.json`); a missing display → the window opens on the main display scaled to fit, and moves
  back when the display returns.
- **Overview** ("I also like the overview from the artifact so we can see multiple screens"):
  both — in the app a **Screens overview** (Window › Screens) shows every screen of the session as
  a live picture: tap to bring its window forward, open or hide it, or send it to a display; and a
  **tiled mode** where one window or browser tab shows several screens side by side, live and
  usable (one display, a browser). Mockup in the 12a3 round.
- **Full screen** ("Only hides sidebars. Fullscreen with meow-F12? … And on macos native
  fullscreen. The desktop clients needs to do this natively per os and not just maximize"): see the
  key table (F12 / Meow+F12).
- Defaults taken (say if not): corner taps within 3 s, 64 px corners; AutoLock is measured by the
  page per device (any pointer or key), only while logged in; Guest is an ordinary user when the
  show has one; the phone's top line shows the station and LumaSession.

**12a3 design locked (2026-10-12)** — "Approved": six boards (Connect, Surface, Screens, Prompts,
Keys, Overview) kept in [docs/dev/ui-kit/mockups/2026-10-12-12a3/](../dev/ui-kit/mockups/README.md)
beside the unchanged 2026-10-12 set. 12a3 is ready for go.

**12a3 built (2026-10-12)** — the user surface, as hardened and locked above.

- **Wire**: `CLIENT_PROTOCOL` 5 — `Hello.device`, `RemoteShellCode::Pause`, `UiEvent::Page {what, on}`
  (the device's page actions: sidebars, view bar, full screen), `Status.device`
  (`DeviceStatus {id, locked, normal_theme, theme_override}`), and `wire::shell::modifiers`
  (SHIFT 1, ALT 2, CTRL 4). The TUI maps those bits explicitly (`tui_types::to_crossterm`); the old
  read as crossterm bits had Alt and Ctrl swapped.
- **Devices** (`core/devices.rs`): the station keeps `{locked, normal_theme}` per device id
  (`AppState::devices`, station-local, never saved). Objects `DeskLock`, `NormalTheme` (Station
  scope), `Sidebars`, `ViewBar`, `FullScreen` (Client scope, a `UiEvent::Page` to that device);
  `On DeskLock Station` locks every device, and a device that connects while it is on starts locked (until
  the last device is unlocked, `AppState::station_lock`, R1-42); a locked device's lines are refused in dispatch before
  forwarding (E-LOCK-001) except the lock words (bare or after On / Off / Toggle, `Station` only for
  DeskLock, one statement); DeskLock by a user with a password unlocks only with it (`POST /api/unlock`),
  without one by the corner taps (E-LOCK-002 names the case); only the user who locked it or Admin unlocks
  (`devices::may_unlock`, R1-42).
- **Admin**: `passwords::ADMIN_PASSWORD` = `admin`, checked by `check_password` everywhere
  (HTTP login, `Login`, unlock); `Set User Admin Password` → E-USER-001.
- **Keys**: `keybindings::CORE_KEYS` (F9–F12, Pause, Print Screen with and without modifiers) never
  resolve from a profile and `Bind` refuses them (E-KEY-005); the Meow layer is `ui_keys::CORE_MEOW`.
  Dropped defaults: F1–F5, F9 cd fixture, F10 cd universe, Shift+F10, Meow+B / V / X.
- **Show format 8**: `User.color` (12 `USER_COLORS`, Admin gold); `Set User … Color` (own user or
  Admin). `Set Station ThemeOverride Off|Dark|Daylight|Night` (station-local).
- **Station view**: `/api/station` adds users' role, colour, profile, look and where they are logged
  in (`presence`: this station + `LumaSessionsRuntime.master_users` from the master + members), and
  `status {dmx, role, lumasession, unsaved}`; `GET /api/station/events` streams it (SSE, 500 ms check).
- **Kit**: `<luma-surface>` (connect / login / locked / blurred), `<luma-avatar>`,
  `<luma-keyboard>` (text / password, QWERTY), the standard dialog (`openDialog`),
  `<luma-overview>` (overview / tiled), `theme.applyLook` (presets, adjusters, palette, normal and
  override); Showroom sections for each. Icons added to the sprite.
- **Station page**: the old login overlay is gone; the surface logs in (5 s countdown, Esc,
  `?autologin=now`), locks (lowest screen full, others blurred, pop-ups queued as "n waiting"),
  unlocks; `?overview` and `?mirror` pages for the overview's tiles; Lock tile on the sidebar and in
  the Shutdown menu; AutoLock from the profile.
- **App**: `connect/connect.html` bundled (`build.rs` → `assets.rs`, served at
  `lumacore://localhost/`); `home/` removed. Picking a server opens the station page without a
  token; the page logs in and calls `LumaHost.loggedIn` → the app loads the user's layout.
  `client.json` keeps the device id and Kiosk; screen windows keep display + place per server and
  user; Meow+F12 = tao borderless full screen (macOS native space); Window → Screens Overview….
- **Tests**: `devices_tests` (8), TUI Meow+F9, e2e `kit-surface` (4 projects), `connect` (was
  `home`), `app-mode` LumaHost tests, `login-sysmon` on the surface; reference and demo regenerated.
  The native app wasn't run by hand (roadmap).

**12b hardening (2026-10-12)** — "Lets do 12b, read and analyze all related md files with new
and old discussion so we can harden this". A sweep of every md file, the locked Dialogs /
Elements / Keyboards / Prompts boards, a reference console's message box / popup input / calculator / keyboard
files and the code found: not five but three server paths for a question (`UiEvent::Confirm`
re-running the line with `/NoConfirmation`; a terminal-only yes/no for Unclaim Patch, `Debug`,
`Shutoff` that the web never sees; Status fields for the merge decision and claim asks, only the
first ask shown) and more client patterns (menus' "click again", object settings, Store view,
window settings); decision and claim asks not queued while locked and not shown again after an
unlock; a queued countdown restarting from full; `At + 3` / `At * 2` silently setting **0 %**
(an unparseable value becomes 0); a label that can't be cleared; `lumacore-cli` ignoring UI and
Status packets; only two real link props (`LockScreen.Image` is text); stale names (§10 "Merge
All", claim ask "Allow / Deny", `Import UserProfile` "asks" vs built `/Overwrite`) and two terminal
docs that disagree.

- **One server-held dialog.** Every question the station asks (line confirms with
  `/NoConfirmation`, OopsConfirmation, SaveShow overwrite, Load / New / Delete show, CleanStart,
  Shutdown / Restart, Unclaim Patch Keep / Discard / Cancel, `Debug`, `Shutoff`, claim asks, the
  merge decision, the password prompt, text prompts such as Label / Move / Copy) becomes one
  `UiEvent::Dialog` with an **id**, kept by the station until answered or timed out. Shape (the reference
  MessageBox, named answers, never 0-based indexes): title, meta, text, rows, details list,
  **inputs** (name, value, kind: text / password / number with range + unit / link with its pool),
  **toggles** (states), **selectors**, **buttons** (label, kind primary / danger / plain / please,
  disabled + reason, each a **command-line template** filled from the inputs, e.g. `Label Group 4
  "{name}"`), optional **countdown** with deadline and default on the **server** (clients only
  show it; a touch pauses it, like `Pause LumaSession`). The merge decision and claim asks leave
  Status and use it too.
- **Answer path.** Browsers get it over the WebSocket the page already uses for its shell (as
  `UiEvent::Confirm` today); a reload or reconnect gets the open dialogs again. A tap sends an
  **answer packet** `{id, button, inputs, toggles, selectors}` (`CLIENT_PROTOCOL` bump), never a
  free line. The station **first closes the dialog on every device** (`DialogClosed {id, by}`),
  then builds the line from the button's template and runs it as that user — journal, rights and
  Oops see an ordinary command. A later answer gets "already answered by Bob on desk2".
- **Who gets it**: a line's confirm → the device that ran the line; a claim ask → every device of
  the owning user; the merge decision → every device of the station. First answer wins.
- **No timeout = it waits.** No invented timeouts: an ask without a countdown stays open until
  answered, or until its device logs out / is gone for good (counts as Cancel). Asks with a
  countdown run to their default also while locked. Locked devices show them as "n waiting"
  (decision and claim asks included) and a queued countdown keeps its real deadline.
- **Terminals** get the same dialog as a **centered window** over the dimmed shell ("is there a
  way to spawn better prompts with the tui that is centered as a window in the terminal?"): the
  station draws the TUI with ratatui (clear the area, a bordered box in the accent with title and
  countdown in the border, text, details list, input fields with a cursor — passwords masked —,
  a countdown bar, buttons in a row; ←/→ or Tab choose, Enter, number keys, Esc = Cancel, any key
  pauses a countdown). `lumacore-cli` only shows the station's frames, so it gets the window with
  no dialog code of its own; the answer is the same as any other path.
- **The TUI wears the user's profile theme** ("the terminal tui styles should follow the users
  theme settings" → "userprofile theme"): the whole TUI (shell, command line, completion, dialog
  window), not only dialogs, is drawn from the logged-in user's **UserProfile Theme** (Preset,
  Adjust, Palette) — the same 14 colours and derived roles as `tokens.css`, resolved on the station
  (one Rust copy of the derivation, checked against `theme.js` by a test) and sent as truecolor;
  the theme's background is painted too, so a Daylight palette stays readable in a dark terminal.
  `NormalTheme` / `ThemeOverride` apply to a TUI on the station's own console like on a device.
  The ~49 hard-coded named / indexed colours in `tui_draw.rs` / `command_line.rs` go; tidy rule:
  no raw colour in `terminal/` outside the theme module. A theme change redraws open shells. One-shot / remote command paths keep answer-as-flag (`/NoConfirmation`, `/Keep`, …) and
  the refusal names the flag. *(Settles the conflict between cli-terminal.md and ui-framework.md.)*
- **Relative numbers on the server**: `Set … + 3`, `- 3`, `* 2`, `/ 2` and `At + 10`
  are parsed by the station against the current value (per fixture for `At`); the keypad sends the
  typed expression as is. An unparseable value is **E-CMD-008**, never 0.
- **Keypad / calculator** (`luma-keypad`): the locked 4×5 grid (7 8 9 ⌫ / 4 5 6 Clear / 1 2 3 + /
  0 . − * / Min Max / Please), the old value shown ("was 15") and preselected (the first digit
  replaces it), range and unit from the descriptor, plus a **unit swipe** only when the descriptor
  has units (seconds ↔ fps, % ↔ DMX 8 / 16 bit). Thru, Specials and presets come with faders (12f).
- **Pick-list pop-up** (`luma-popup`): filter field always shown (Enter picks the first
  match), **None** only for optional links, the items, **Add New** = `Store <Kind>` at the next
  free number and picks it (no editor; hidden without the right or for a kind that can't be
  stored); a broken link (missing GUID) shows as a top row "missing …" until changed.
- **Real links**: `PropKind::Link` covers every GUID link a setting holds — Line Network, OSC
  User, a User's Profile, the profile's `LockScreen.Image` (its own Images pool), icon refs, the
  selected DataPool / Sequence — each naming its pool so the pick list knows the items.
- **Keyboard setting per device, per screen**: `Keyboard: Auto | OS | Built-in | Off` is kept by
  the device (app `client.json` per screen window; browser tab `localStorage`), default Auto; the
  app's Kiosk toggle becomes "Keyboard: Built-in" for all its screens. **VK button** on the
  command line bar: tap shows / hides the chosen keyboard (built-in, or focus for the OS one),
  hold opens Auto / OS / Built-in / Off. The built-in keyboard types into the xterm command line
  too (word row from the registry). **Layouts: English (US) only** in 12b; layouts stay data files
  (Value / Shifted / AltGr per key, a common keyboard-file shape) so others can be added later.
- **More asks on the dialog**: Store over a used slot asks Merge / Overwrite / Remove / Cancel in a
  UI (terminals keep the flag hint); `Import UserProfile` with the same GUID asks Overwrite /
  Cancel; critical feedback and OK / Confirm feedback pop-ups use the dialog (the Message Center
  window stays later). The menus' "click again" and Delete without use stay as they are
  (`confirmAction` still moves onto the dialog).
- **Claim ask "Remember for this session"** (kept from the board): on + Hand over → later claims by
  the same user hand over at once; on + Keep → refused at once; until the owner logs out
  (`Unclaim Group 5 /Remember`, `Claim Group 5 /Keep /Remember`; `List Claims` shows it).
- **Label prompt**: recent-name chips = your last names in that pool kind (client-kept, up to 6);
  an empty answer clears the label (`Label Group 4 ""`).
- **Plugins / macros**: `UiEvent::Dialog` maps 1:1 to a reference message box (commands, inputs with
  kind, states, selectors, timeout + timeout result); Lua `MessageBox` / `TextInput` /
  `PopupInput` and a macro wait come with the plugin runtime (roadmap).
- **Mockups first**: a dated board set for what the locked boards don't show (dialog with
  inputs + keypad kind, the answered-elsewhere notice, "n waiting", Store over existing, the
  terminal form, pick list with a broken link / Add New, keypad with the unit swipe, VK button
  hold menu, the keyboard over the command line), approved before go.

**12b design locked (2026-10-12)** — "approved": five boards (Asks, Inputs, PickList, VK,
Terminal) kept in [docs/dev/ui-kit/mockups/2026-10-12-12b/](../dev/ui-kit/mockups/README.md)
beside the unchanged earlier sets (the Terminal board redrawn twice on request: a centered window,
then the profile theme). 12b is ready for go.

**12b built (2026-10-12)** — the dialogs, as hardened and locked above.

- **Core** (`core/dialogs.rs`): `Ask` (title, meta, text, rows, details, inputs with a kind, toggles
  with a flag, selectors with texts, buttons = command-line templates, `or_when_empty`, cancel,
  countdown, topic, the one-shot flags); `DispatchResult::Ask` (replaces `Confirm` and
  `dispatch_confirm`) — a text handler leaves it in `AppState::asking`, dispatch turns the line's
  result into it. The **board** (`AppState::dialogs`, station-local) keeps each open dialog with its
  audience (`Device`, `Client` for a CLI, `Local`, `User`, `Station`) and asking user; another user
  logging in on the same device doesn't see it. `for_client` adds the derived ones: claim asks (to
  the owner) and the merge decision (every client). `answer` closes first, then fills the template
  (text loses `"` and `;`, numbers only digits and `+ - * / .`, E-CMD-008) and returns the line;
  `take_due` presses the default at the deadline (`server::context::dialog_tick`, housekeeping);
  `forget_gone` drops one whose client is gone 30 s.
- **Transport** (an implementation choice; what it does is as decided): the dialogs ride in each
  client's `Status` (`dialogs`, `dialogs_closed`) instead of a separate `UiEvent::Dialog` — the
  status is already recomputed per client every ~50 ms and sent when it changes, so a reload, a
  reconnect and "closed everywhere" come for free. `ShellClientPacket::DialogAnswer` /
  `DialogPause`; WS `dialog_answer` / `dialog_pause`. `Status.claim_asks`, `UiEvent::Confirm` and
  `UiEvent::PasswordPrompt` are gone. `CLIENT_PROTOCOL` 6.
  R1-61 (2026-10-13): `UiEvent::StoreView` is gone too — Store View Options (`views::store_ask`: name +
  a toggle per screen) and Rename (`Label Window|View|ViewButton n` without a name asks; `/Clear`) are
  station asks. `CLIENT_PROTOCOL` 9.
  R1-61 / Q35: **Delete** is asked by the station before the line runs or is forwarded
  (`dispatch::delete_first`; `/NoConfirmation` / `/nc` answers and is taken out before the handler);
  the page's own Delete confirms are gone.
- **Asks**: backup confirms (Save / Don't save / Cancel with unsaved changes; Shutdown / Restart
  10 s, topic `power:…`, a remote client gets "not this computer"), OopsConfirmation (the steps as
  rows), Unclaim Patch (Keep / Discard / Cancel, waits), `Debug` and `Shutoff` (`/NoConfirmation`
  now answers them), `Login "x"` with a password (masked input; also closes the CLI roadmap item),
  Store over a used group (Merge / Overwrite / Remove / Cancel), `Import UserProfile` same GUID
  (Overwrite / Cancel), `Label Fixture|Universe|Group <n>` without a name (the current name, empty
  → `/Clear`: an empty `""` never reaches a command), `Move Fixture|Group|Universe <n>` / `Copy Group
  <n>` without `At` (a number). Claim asks: Hand over / Keep + "Remember for this session"
  (`Unclaim … /Remember`, `Claim … /Keep /Remember`; `AppState::claim_remember` on the master,
  dropped when the owner logs out).
- **Relative numbers**: `At + 10`, `At - 10`, `At * 2`, `At / 2` per fixture from the user's
  programmer value, else the fixture's current output (R1 Q8), kept in 0–100 % (`-5` glued is absolute, clamped); `Attribute … At` the same; `Set … + 3` on any
  whole-number property (`- 3` with a space is relative, `-3` a negative value). An unparseable value
  is E-CMD-008 (was 0 %).
- **Pick lists**: `PropKind::Link` carries `items`, `optional` and the keyword Add New stores;
  `PropKind::OwnLink` for a link into the object's own pool (a profile's `LockScreen.Image`). Built
  for Line Network (Add New = `Store Network <next>`), OSC User (optional), LockScreen.Image. A User's
  Profile, the selected DataPool / Sequence and icon refs aren't settings yet — their pick lists come
  with 12c's descriptors (roadmap). `SettingField.pick`; EditSetting shows the name, not the GUID.
- **TUI**: `tui_dialog.rs` (the centered window: rounded accent border, title and countdown in the
  border, text, rows, details, inputs with a cursor, toggles, selectors, a countdown bar, numbered
  buttons; keys as decided), drawn only for terminals (a UI client draws the kit's). `core/look.rs`
  (presets and adjusters as `tokens.css` / `theme.js`, checked by tests reading those files; golden
  adjuster values taken from theme.js's own code) and `tui_look.rs` (roles + the paint pass: xterm's
  slot map, ground = `--lc-gap`). The prompt may now use all but 16 columns of the command line
  (was half), so a long station name isn't cut on a narrow bar.
- **Kit**: `dialog.js` (inputs, toggles, selectors, details, tones, Please kind, Enter = primary,
  a stack so a page's question covers a station's; `showDialog` for station dialogs, updated in
  place keeping the caret), `keypad.js` (`<luma-keypad>`, `openKeypad`), `popup.js` (`<luma-popup>`,
  `openPick`), `keyboard.js` (layouts from `kit/layouts/*.json`, English (US); the Keyboard mode
  store; a sink target for the command line), Showroom sections. The keypad button uses `grid-3x3`
  (no `calculator` in the sprite without a Lucide re-pack).
- **Page**: `renderStationDialogs` (from the Status, "answered by …" notices, nothing over the
  lock, "n waiting"), power answers tell the app (`power`, `power_cancel`; the "Server shutting
  down" message does too), `confirmAction` / `promptText` on the kit dialog, the pool menu's Label /
  Move / Copy send the bare line (the station asks), EditSetting link fields open the pick list,
  critical feedback pop-ups on the kit dialog, the VK button (`#cb-vk`) with the Keyboard mode kept
  in this browser's / app's local storage per screen (so it stays with the device whoever logs in),
  the app's Kiosk toggle = "Keyboard: Built-in on every screen". The EditSetting pop-up scrolls (it
  was taller than a tablet).
- **Tests**: `dialogs_tests` (7), `relative_tests` (2), `look_tests` (3), the TUI window test, tidy
  `terminal_colours_only_in_the_look`, `frontend_asks_only_through_the_kit_dialog`,
  `keyboard_words_are_registry_words`; e2e `kit-dialogs.spec.ts` (desktop, tablet, phone, WebKit)
  and the old specs moved to the kit dialog; lumacore lib 461, e2e 154 passed (7 skipped).
- Not done here (roadmap): pick lists for the links that aren't settings yet; a member's forwarded
  line that the master would ask about is refused with the flags; the keyboard's word row is a
  fixed list (checked by tidy) until the registry is on HTTP.

**Theme: background hue + saturation (12g, asked 2026-10-12)** — "one thing i noticed now is
that the theme editor needs a background color hue and satuaration slider. ask me more to
harden". Two adjusters on the profile's `Theme.Adjust`: **BackgroundHue** 0–360° and
**BackgroundSaturation** 0–100, on top of the preset (0 = the preset's own neutrals, so Night keeps
its red-black); they move **all neutral greys together** (Background, Surface, Raised, Line,
blended toward the hue at their own lightness), text / accent / status colours untouched; a
contrast readout warns when text or text-dim gets too low. Built with the theme editor in 12g
(show format bump then, forward only; the TUI follows like any theme value). **Tint is replaced**
("A tint replaced", after testing both on the `ThemeBg` board, canvas row 8): the adjusters are
Brightness, Contrast, Saturation, BackgroundHue, BackgroundSaturation, AccentHue (`Theme.Tint`
goes with the format bump, no migration). The test board is the 12g starting point for the
Theme board.

**12b2 hardening (2026-10-12)** — before 12c: "when i have the servers list after i have shutdown
the server, it still shows as a green status … we need to create a better window settings artifact
… the datapool follow setting … needs to be a better list instead of a int … when i press shift on
my mac, that should be meow … the commandline bottom bar has a dark line … research and probe the
userprofile edit menu and create an artifact for it … when i drag in the group window it selects
the slot numbers as text … selecting text we need to harden where it is allowed", then "pressing
F12 (fullscreen) should also hide the bottom and top bars". **One step now** ("Everything in one
step now"): the fixes plus both editors; 12c keeps the window bar, `Self` and the layout format.
Research: the connect page and discovery code; a reference console's window settings (its layout files,
its manual) and its profile grid from the corpus and
GET reads on the console (nothing changed there).

- **Servers list** (found: the app keeps an entry alive while its mDNS record lives — ~120 s, the
  server never unregisters it and sends no last beacon; the dot is always green): on a clean
  shutdown the server **withdraws its mDNS service and sends a `leaving` beacon** (gone at once);
  the app lets the **beacon decide** — no beacon for 4 s → gone even if mDNS still lists it (a
  killed server); mDNS only finds stations whose beacon can't reach. The dot: **green** = beacon
  fresh, **amber "not answering"** = late (> 1.5 s), removed after 4 s; wrong version stays grey.
- **Meow = Shift** by default on every platform (as established; Shift+F9…F12 are free on
  macOS; Cmd and Ctrl are taken there — Cmd+F5 VoiceOver, Ctrl+F1–F8 keyboard navigation,
  Ctrl+click). Shift with a letter on the command line still types a capital. `Set MeowKey`
  stays per profile. The sidebar Meow button **lights while the key is held** (and stays latched
  on a tap, as before) — today it only showed the latch.
- **Mac F-keys**: without fn (or "Use F1, F2, etc. keys as standard function keys") macOS keeps
  F9–F12 for media / Show Desktop and the page never sees them. LumaCore touches no OS setting:
  on macOS the F10 shortcut sheet says "hold fn, or turn on standard function keys", and a hint
  shows once when Meow is used and no F-key ever arrived.
- **F12 = full screen of the windows** (replaces "F12 hides the sidebars and the view bar"): hides
  the sidebars, the view bar, the **top bar and the command line bar**; per screen, as today. Back:
  F12 again, the `Sidebars` command, or a **small corner handle** (a kit element: visible on touch
  screens, faded on desktop until the pointer comes near). **Typing while hidden floats a command
  line in** over the windows (kit element, like a command line pop-up); Please / Esc hides it,
  the bars stay hidden. Later it grows into a **command palette like Raycast** ("the floating
  commandline, we need to think of this in bigger terms, we want it to be more like raycast later
  … start aiming for it, so we need to do some foundations"; "in normal mode, commandline at the
  bottom, but fullscreen it should behave more like raycast"): the bottom bar stays the command
  line; in full screen the floating line is the palette. **Suggestions also at the bottom** ("that
  means that we need suggestions when it is also at the bottom"): one suggestion source serves
  both — a list above the bottom bar while typing, the palette in full screen. Researched (Raycast,
  PowerToys Command Palette, Flow Launcher, Albert, Ulauncher, Script Kit, cmdk, kbar, ninja-keys,
  VS Code quick pick, Zed, Warp, Fig specs, fish / Nushell completions, nucleo; no lighting console
  has a palette). **Foundations in 12b2** ("1 and artifacts"):
  - **One suggestion engine in `core/`** (the completer core moves down from `terminal/`; the TUI,
    the sweep and HTTP call it): `suggest(line, cursor, surface: Bar | Palette)` → the span to
    replace, the hint, and **typed items** (kind, title, subtitle, icon, insert text, GUID, actions
    as command lines, preview ref, Planned / no-rights state with its code, score). One query /
    reply message on the page's socket (a sequence number drops stale replies) — never a per-kind
    endpoint.
  - **Picking never runs code in the client**: it writes the command line or runs an item's
    command-line template; the palette's breadcrumb is the command line. Input the palette can't
    take inline is the 12b `Ask` (keypad, pick list) — no palette-only forms.
  - **Ranking on the station** with **nucleo-matcher**: grammar first (only what's valid at the
    cursor, from `registry::Args`), then the fuzzy score, then usage; a fixed tie-break (kind, then
    number) so it is deterministic and testable. The client never ranks.
  - **Bottom bar**: the list opens **above the line** after a short pause, plus grey ghost text
    (history / hint); profile setting **Suggestions: Auto | Tab only | Off** (Users & Profiles →
    Command line). **Enter runs the line as typed** — Tab or a tap takes the selected item; Up /
    Down move in the list only while it is open (else history); Right at the end takes the ghost
    text; Esc closes. The TUI gets the same list as a themed overlay.
  - **Full screen palette** (starts in 12b2 as the floating line with the same list; global search,
    item actions, Tab-into pages and the preview pane come next): Enter = primary action, Meow+Enter
    = the second, Tab opens an item, Esc / Backspace on empty goes back.
  - **Usage** ("per user and goes with the showfile so we can learn what users do, we later might
    want other data so the user learns more what is used the most"): `ShowData.Users.{n}.Usage`
    (per GUID or command template: count + last use, decayed rank) — **saved with the show and
    replicated, never in undo frames** (its own `ShowSubtree`, like `Playbacks`); fed from
    dispatch. Show format bump (forward only). Later: more usage data and a "what is used most"
    view (roadmap).
  - One kit element **`luma-suggest`** (`mode="inline|palette"`: rows with icon / title / subtitle
    / shortcut chip, sections, breadcrumb = the command line, preview slot); mockups (row 9) first.

**12b2 design locked (2026-10-12)** — "approved": eight boards (Menu, UsersProfiles, WindowSettings,
FullScreen, Fixes, SysmonList, Realtime, Suggest) kept in
[docs/dev/ui-kit/mockups/2026-10-12-12b2/](../dev/ui-kit/mockups/README.md); WindowSettings replaces
the 2026-10-12 Settings board's window part. 12b2 is ready for go.

**12b2 built (2026-10-12)** — as hardened and locked above; built in three parallel tracks
(servers list, client feeds, suggestions) plus the rest, merged into one step.

- **Servers list**: `mdns::Advertisement::withdraw` on a clean shutdown (unregister + daemon
  shutdown, ≤ 500 ms each) and a last beacon with `leaving` (`SessionDiscoveryWire.leaving`,
  `SESSION_PROTOCOL` 8); a station's own listener drops a leaving peer at once. The app
  (`apps/lumacore-app/src/discovery.rs`, `Entry::listed`): a station that ever beaconed is listed
  only while its beacon is < 4 s old, late (> 1.5 s) = `not_answering_s` (amber dot, kit
  `.lc-sf-dot.warn`); mDNS-only stations as before.
- **Meow = Shift** (`default_meow_key`, `Set MeowKey` usage, the TUI fallback); the sidebar button
  gets `.held` while the profile's key is down (window keydown / keyup / blur). The macOS F-key
  hint is a feedback card, once per device, the first time Meow is held when no F-key ever came
  (there is no F10 "shortcut sheet": F10 is the shortcut mode — the hint is only the card).
- **F12**: `sidebars.full` (per screen, stored as before) → `body.fullwin` hides the top bar,
  both sidebars and the command line bar; `body.fullwin.typing` floats the bar (its own xterm, so
  the same line and suggestions) over the windows; Esc / Please sends it away. The kit corner
  handle (`.lc-corner-handle`, Showroom) brings the bars back; `Sidebars` / `On|Off Sidebars` docs
  updated.
- **Command line bar ground**: `tui_look::paint(…, Ground::Bar)` for an input-only view paints
  empty cells in `--lc-bg`; terminals keep `--lc-gap`.
- **Text selection**: one kit rule (html / body none; inputs, `.xterm`, `[contenteditable]`,
  `.lc-selectable` text; `-webkit-` prefixes, no touch callout); the old partial rules removed;
  tidy `text_selection_only_in_the_kit_rule`. Dialog text, System Monitor rows and the settings'
  last line are `.lc-selectable`.
- **Typed settings**: `lumacore_wire::FieldInput` (Text / Toggle / Number with named values /
  Choice / Link / Colour) and `SettingField.section` + `hint` replace the free-text `kind`
  (`CLIENT_PROTOCOL` 7); `Pick.follow` (the "Selected DataPool" row). Window settings:
  `SettingKind::Int { named }`, `Link { kind, follow, by_guid }`, `Colour`, `SettingDef.section`,
  `label_of`; pool windows Pool · Layout · Colours, DMX Display · Mask, Terminal / System Monitor
  one section; font size Default (0) + sizes; `datapool` = `"selected"` or a GUID
  (`windows::resolve_window_link` turns a typed number or name into the GUID before the coerce;
  the pools feed resolves it, `pools::resolve_datapool`). `EditSetting Window n`
  (`windows::window_fields`), `EditSetting User "x"` (`users::user_fields`: Role, Profile pick
  list, Colour, RemoteLogin; the password is its own action) and every object's EditSetting draw
  in the kit's `<luma-settings>` (`settings.js`: tabs, a cell per input — toggle flips, choice and
  named numbers the pick list, number the keypad, link the pick list, colour / text the dialog —
  every change one line, the station answers with fresh fields, updated in place). `Set Window n
  <key>` with no value clears a text setting (an empty `""` doesn't survive the line's words).
  The window popover and the EditSetting overlay are gone. Load / Save presets of window settings
  stay with 12c (window bar) — not built here.
- **Users & Profiles**: Menu tile next to LumaSessions → a floating window with `<luma-people>`
  (users with avatar / role / profile / where logged in, profiles with who uses them; the picked
  one's `EditSetting` in an embedded `<luma-settings>`; actions New user, Password…, Delete / New
  profile (`Copy UserProfile n At m`), Export, Import, Delete; Theme editor waits for 12g). The
  list comes from `GET /api/station` (now with `profiles` and each user's `no`). Thirteen
  `UserPreferences` fields became `UserProfile` props (MeowKey, HoldTime, DoublePress, GridCursor,
  PoolDrag, the five Oops ones, ShowDescriptions, ClearBuffer, NotificationMode;
  `media::profile_section` puts them on the tabs). The palette colours stay out of the window
  (the theme editor's, 12g).
- **Menu icons**: every tile a Lucide icon above its label, the menus' titles the same icon
  (sprite: plug, users, wand-sparkles, timer, arrow-left-right, box, command, history,
  circle-help, hash, maximize, minimize).
- **Client feeds** (`/ws/feeds`, `websocket/feeds.rs`; the old `/ws/monitor|feedback|health|dmx|
  pools` removed): `sub` / `unsub` per window, period ≥ 16 ms, envelope `{feed, kind, id, epoch,
  from_seq, to_seq, gap}`, `since` + `recovered`, a send stalled 2 s closes. Feeds `health.render`
  (samples: every frame from `RenderStats::FrameRing`, 640 frames, seqlock slots written outside
  the tick), `health` (state), `sysmon` (the whole history in one batch, then lines per tick; a
  lagging broadcast is refilled from the history, else a gap), `feedback`, `dmx` (binary: 20-byte
  header, 3080 bytes per universe, only changed universes), `pools`. Kit `feeds.js` (FeedClient:
  reconnect with `since`, epoch reset, hidden page → 250 ms; `SampleRing`; one rAF `drawLoop` per
  window), `<luma-vlist>` (System Monitor), `<luma-graph>` (Realtime). The Realtime window: eight
  graphs (frame, six stages, headroom) at 16 ms + the tables at 4 Hz; the DMX window at 60 Hz.
  Tests: the router has only `/ws/feeds` and `/ws/shell`; tidy `frontend_feeds_ride_one_socket`;
  the envelope test. The HTTP engine's stop now closes `/ws/feeds` and the SSE streams
  (`HttpState::stopping`, `stopped()`), so an open page no longer holds the shutdown to the 5 s
  deadline (found while testing; the SSE part was older).
- **Suggestions** (`core/suggest/`: `grammar.rs` = the old `terminal/completer.rs`; `mod.rs`
  `suggest`, `ghost`, `record_usage`): typed items (`lumacore_wire::suggest`), grammar first,
  `nucleo-matcher` 0.3 ranking, decayed usage, a fixed tie-break, history first in the bar, an
  object keyword also offers its objects, Planned / no-rights items marked; palette items carry
  actions. Shell packets `Suggest {seq, line?, cursor?, surface}` / `LineEdit {start, end, text,
  expect?}`, reply `Suggestions`. `UserPreferences.suggestions` (`Set Suggestions
  Auto|TabOnly|Off`). Kit `<luma-suggest mode="inline|palette">` above the bar (220 ms pause,
  Tab / tap takes, ↑ ↓ only while open, Enter runs the line as typed, Esc closes, Right takes the
  station's ghost text); in F12 full screen it sits above the floating line. TUI: typed items in
  the overlay, Enter runs as typed. **Usage** `User.usage` (`lumacore-tree/src/usage.rs`: GUID or
  `word:<Label>` → count + last use, half-life 7 days, ≤ 500), `ShowSubtree::Usage(user)`: saved
  (`Usage.json` per user), replicated, never in undo frames; recorded for successful changing
  lines on the master. `FORMAT_VERSION` 9, demo regenerated. Not done: global palette search,
  item actions on Enter / Meow+Enter, Tab-into pages, preview (roadmap). Admin-role users get no
  history items (the TUI never recorded Admin history — unchanged).
- **Tests**: lumacore lib 476, lumacore-http 36, tree / wire / session / dmx / app pass; new
  `window_settings_tests` (4), suggest tests (8), feeds tests; e2e `kit-12b2.spec.ts` (Meow held,
  F12, text selection, Menu + Users & Profiles), `feeds.spec.ts`, `kit-suggest.spec.ts`, the
  window-settings test in `kit-dialogs.spec.ts`; specs moved to the new behaviour (Shift as Meow,
  F12 full screen, the settings window, suggestions in `<luma-suggest>`). **Parity** (asked: "we need a test
  to verify that the suggestion in the cli tui is the same as the commandline in ui"): the Rust test
  `the_terminal_and_the_bar_show_the_same_suggestions` (a terminal's live list read off the rendered
  screen vs the `Suggest` packet's answer, eight lines of the demo show) and e2e
  `suggest-parity.spec.ts` (a `lumacore-cli`-like client over the shell port, its frames drawn by the
  vendored xterm.js, vs the page's `<luma-suggest>`; helper `Cli` in `support/server.ts`).
- **Command line bar ground**: the line's terminal paints its empty cells in the bar's own colour
  (12b painted `--lc-gap`, a dark strip on the `--lc-bg` bar); a full terminal keeps the gap ground.
- **Text selection — only where you read or type**: nothing is selectable app-wide (no iOS callout
  or magnifier on a hold either); allowed only in text inputs, the command line and terminal
  (xterm), System Monitor rows and help / info text. One kit rule with the `-webkit-` prefixes
  (the app's WebKit needs them) and a tidy check.
- **Window settings** (pulled forward from 12c; the 2026-10-12 Settings board was the start): one
  kit settings window for every window type; **tabs from the settings' sections**, the tab strip
  hides itself at one tab; changes apply at once (one `Set` line each); Load / Save presets, Delete
  Window and Close in the title bar. Pool windows: **Pool** (action, DataPool, first index),
  **Layout** (columns, rows, size, font size, show title, show empty), **Colours**.
- **Named choices instead of magic numbers** (typed descriptors, also from 12c): **DataPool** is a
  pick list — **"Selected DataPool"** (follows the user's selected datapool)
  plus every datapool by name, stored as a **GUID** (the slot number broke strict identity);
  **font size** = Default + sizes; **columns / rows** = Fit + numbers; colours = the colour picker;
  the DMX window's **universe** = a pick list of the show's universes. The window settings get the
  props' kinds (`Link`, `Colour`, `Choice`) instead of their own `SettingKind`.
- **Users & Profiles** (a reference console has only a grid, one column per property): a **floating window like
  LumaSessions**, a new Menu tile next to LumaSessions. Left: users and profiles (who uses which,
  logged in where); right: the selected profile's tabs **Look · Lock screen · Input & keys · Oops ·
  Command line** (a Readouts tab comes with the readout defaults), and the selected user's settings (role, password, profile link —
  its pick list). Store / Delete / Import / Export run commands. **Every `UserPreferences` field
  becomes a declared prop** (Meow key, hold / double-press ms, grid cursor, pool drag, oops, verbosity,
  descriptions, language …) — no hand-made `Set` paths. Only what we use now; the reference readout
  defaults (value / speed / preset / time / DMX readout, single digit input, encoder style …) go to
  the roadmap until a window reads them. `OopsConfirmation` stays **Never** by default (the reference:
  Always).
- **Menu icons**: every Menu tile gets a **Lucide icon above its label** (Network, DMX Protocols,
  Backup, Quick Save, LumaSessions, Screens, Users & Profiles, Settings, Patch …); the same icon
  heads the window it opens; unbuilt tiles keep their icon, dimmed.
- **Client feeds — one rule** (asked during the hardening: "when adding the systemmonitor window,
  it streams in all the lines from the beginning line by line … it needs to … be snappy" and "the
  realtime window updates pretty slow … run at the native 60fps"; found: `/ws/monitor` sends its
  history one message per line and the page adds one row per message; `/ws/health` ticks every
  250 ms, `/ws/dmx` 50 ms, `/ws/pools` 100 ms): every feed to a JS client sends **one batch when a
  window opens** (the state or the whole history), **then changes in batches per tick** — never
  one item per message ("i think this is how most of all data should be sent to clients"); a tidy
  rule / test holds it. The monitor and feedback feeds move to it now.
- **System Monitor**: the stream opens with the window (not from login: "a batch of data when
  loading windows makes sense so the clients can get up to speed faster"), the **station's whole
  sysmon history in one batch**, then live lines per tick; the page keeps the same number of lines
  as the station. The list is a **virtual list** (a new kit element: only the visible rows exist;
  filter and search run over the lines in memory; visible text stays selectable).
- **DMX window at 60 Hz while visible**: only changed universe values per frame, drawn on
  `requestAnimationFrame`; idle universes send nothing.
- **Realtime window at the native 60 fps** "so nothing important gets lost, we will later add more
  important data that the client should show with good performance". Researched (Centrifugo
  history + recovery by offset / epoch, Foxglove WS protocol, rosbridge throttle / queue, OBS
  WebSocket's opt-in high-volume events, game snapshots, Netdata; tokio `broadcast` lag): a feed is
  either **State** (latest wins: each batch replaces, a slow client gets the next newest — DMX
  values, pools, the line / clock / thread tables; `tokio::sync::watch`) or **Samples** (every item
  matters: each has a `seq`; the station keeps a ring per feed, ~10 s of render frames; each batch
  carries **every item since the client's last `seq`**; what the ring no longer holds arrives as an
  explicit **`gap`**, never a silent loss; a server restart changes the `epoch` — render frames,
  sysmon lines, journal / Oops events). The send period suits the client, the sample rate never
  changes: the Realtime window subscribes at **16 ms** (native 60 fps), a hidden tab drops to
  250 ms and still gets every frame. A send stalled > 2 s closes the socket; the client reconnects
  with `since`.
- **One `/ws/feeds` per page** (old per-feed endpoints go, no shims): `{"op":"sub","id","feed",
  "since"?,"period_ms"?}` / `{"op":"unsub","id"}`; high-rate feeds only while a window shows them;
  one envelope `{feed, kind, epoch, from_seq, to_seq, gap, …}`; a kit **FeedClient** (reconnect,
  `since`, epoch check, gaps, typed-array rings per feed, one `requestAnimationFrame` loop per
  window; network and drawing apart).
- **Encoding**: JSON, **columnar** (`{"t_ms":[…],"frame_us":[…]}`), for control and small feeds;
  **binary frames for wide arrays** (DMX universes: a little-endian header feed / seq / time /
  count, then raw columns read straight into typed arrays). No new dependency.
- **Realtime window UI**: each render stage's number plus a **10 s canvas graph** (one point per
  frame, late / skipped frames marked, a gap breaks the line) — a kit graph element; mockup
  (Realtime board, row 9) before go.
- Mockups first (canvas row 9), then go.

**12c hardening (2026-10-13)** — "lets do 12c hardening, read all docs with any kind of related
features to it new and old so we can get all input needed, also probe [the reference console's] docs".
Read: §30 (window bar / presets / `Self` above), the 12b2 parts, `layout-format.md`,
the elements research, the Q&A log, the 2026-10-12 Chrome / Settings boards. A reference console probed (GET reads
+ one `List ScreenContent`, nothing changed): title bars (`NormedTitleBar`: context button, live
title, dynamic chips, `TitleButtons` right to left), `VisibleTitleButtons` per settings object,
the Edit Title Bar mode, `StorePreferences.Views.<Collect>` presets on the profile (copy in / copy
out, `PreferenceHandle`), no `Self` keyword (focus-dependent targeting), Views / ViewWidgets
(half-cell grid) vs its layout files (read-only firmware files bound to a settings class by the menu
manifest, never in a show, no plugin loader). This replaces the earlier wording where it differs.

- **Window bar** (as above, made exact): items are **any typed setting** (pinned) plus the window
  type's **declared actions** — `WindowType.actions: [{id, label, icon, line}]` (the line uses
  `Self`), `bar_default: [ids]`, a minimum size. `WindowBarItems` is the ordered id list (settings
  and actions share the id space; unknown ids dropped on load). **Every item is a `Set` / command
  line** — rights, Oops and journal stay one path: sysmon Pause = `AutoScroll`, Clear = a view-local
  "since" mark (never deletes the station's history), Wrap and Time = settings; DMX `Universe ±` =
  `Set Self Universe + 1` (**relative numbers on `Set Window`**, like tree props). The hand-built
  sysmon / DMX toolbars and their `PUT layout` settings writes go; splits / resizes / tab moves keep
  the layout path. **Overflow**: what doesn't fit goes into a `…` menu at the right; below the
  type's minimum height the bar hides and the window menu lists its items (the kit decides, no
  setting). Not a reference clip at 50 px, not its Edit Title Bar mode: pins in the settings cells and a
  plain ordered list (drag) on the Window bar tab.
- **One state rule through `Self`**: a bar action or UiKey declares `state: "Self.AutoScroll"`
  (lit / checked while true) and optionally `shown_when: "Self.Filter"` (an **auto chip** that
  appears only when relevant, e.g. × clear filter); a window type declares a **live title** as
  text with reads (`"U {Self.Universe}"`, `"{Self.DataPool.Name}"`, sysmon "12 errors"), shown on
  the tab / left of the bar. Key LEDs get `{Self}` next to `{User}`. The station resolves the
  reads per window and pushes changes on the feed; `Self.<Property>` in macros later reuses the
  same resolver.
- **Resets, layered** ("we need a careful system to reset to defaults if a user deletes or modify
  too much"): one setting (↺ on the cell, `Set Self <setting> Default`), one window (`Reset Self`:
  type defaults incl. bar items), the bar (Load defaults on the Window bar tab), and a profile's
  presets / bars of one type. Each is one Oops step; it asks first (12b dialog) when it touches
  more than one window.
- **`WindowPreset`** (not "Preset" — the lighting word — and on the **profile**, not the user:
  12a2 put the UI on the profile, as established): named + numbered per window type,
  `UserProfile.WindowPresets` (show format bump, forward only). `Store WindowPreset "Big DMX"`
  saves `Self`'s window; `WindowPreset 2` loads into `Self`; Label / Delete / List; Load / Save
  buttons in the settings window run these lines. A **copy, not a live link** (the window remembers
  which one it came from to preselect it). It holds **everything**: settings, bar, label, colour,
  scroll. No automatic first preset on new windows. Goes with `Export UserProfile`.
- **`Self`** = the line's owner, now **windows**: typed → the focused window; window bar / window
  menu / settings-window lines → that window; a UiKey → the window it sits in, else the focused
  window (never the key itself). The line context carries an **owner** (window / key / macro /
  plugin) so macros and plugins plug in when their runtimes exist; no owner (a terminal without a
  focused window) → a coded error (E-CMD-016), never a guess. **Resolved on the station where the
  line starts**, before forwarding to the master (it can't see a member's focus); journal, Oops and
  sysmon show the resolved `Window <screen>.<n>`; macros, keys and Usage keep the `Self` text.
- **Two models** (asked: "probe how [the reference] does it … i need an easier explanation"): the
  **floor plan** — screens / views (our split tree, ratios + tab stacks) with each window's type,
  label, colour, scroll and typed settings — is show / profile data, **JSON in the `.luma`** and
  JSON when exported. **How a window type is built** (bar actions, defaults, live title, chips,
  minimum size, settings tabs) ships with LumaCore and is never in a show — an update can redesign
  a window without touching a show. It is **declared in the Rust registry** next to
  `window_types.rs` (compiler- and test-checked, like commands) and **sent to clients as JSON**; no
  hand-written file format now (KDL and layout-XML files dropped; `layout-format.md`'s
  recommendation is superseded). Plugins later describe their UI as Lua tables → the same JSON.
  Every drawn element gets a **`data-id`** (`<window type>/<setting or action id>`, pool cells by
  GUID) for tests.
- **Export / import as JSON** ("anything should be able to be exported and imported as json"):
  12c builds `Export|Import View`, `ScreenConfig`, `WindowPreset` into **`library/ui/<kind>/`**
  (`views/`, `screenconfigs/`, `windowpresets/`, through `core::drive`); every other object →
  roadmap (generic object export).
- **Pick lists left from 12b**: a bare `Select DataPool|Sequence`, `Store User "x" /Profile`, icon
  refs ask through the 12b dialog with a link input (one ask path).
- Mockups next (Design canvas row 10): the bar on sysmon / DMX / pool / Realtime (normal, `…`,
  hidden + window-menu list, live title, chip), the settings window's pins + Window bar tab +
  Load / Save preset + resets, `Self` and its error, export / import. Then go.

**12c design locked (2026-10-13)** — "approved": four boards (WindowBar, BarSettings, Presets,
SelfExport) kept in [docs/dev/ui-kit/mockups/2026-10-13-12c/](../dev/ui-kit/mockups/README.md);
they replace the window-bar parts of the 2026-10-12 Chrome and Settings boards. Command forms taken
from the boards: Clear = `Set Self Since Now`, the search chip = `Set Self Search ""`, one setting =
`Set Self <setting> Default`, the bar = `Set Self WindowBarItems Default`, a profile's windows of a
type = `Reset UserProfile "x" Window /Type <type>` (asks), windows named `Window <screen>.<n>`.
12c is ready for go.

**12c built (2026-10-13)** — as hardened and locked above; station side first, then the web
frontend against its wire contract, then checked in Chrome on a test station (T12, nothing mapped,
no DMX sent) against the boards.

- **Window types** declare `actions` (`{id, label, icon, line, state?}`), `bar_default`, `bar_min`
  (px), a `title` template and `chips` (`{id, label, shown_when, line}`); every visible type gets
  `WindowBar` + `WindowBarItems` (`SettingKind::List`, ids checked against settings + actions,
  unknown ids refused on `Set`, dropped on load). Sysmon adds `Wrap`, `Time` (Session / Wall),
  `Since` (`Now` stores wall-clock ms, so the mark survives a restart); Realtime `Graph`, `Pause`.
  Setting keys match ignoring case and `_`. `GET /api/window-types` carries all of it.
- **`Set Window` / `Set Self`**: relative `+ - * /`, `Default`, `Toggle`, `Now`.
- **`Self`** (`core/self_ref.rs`): one registry object; the shell packets `Command` / `KeyAction`
  carry `window`; resolved in dispatch before a forward, so journal, Oops, sysmon and the master see
  `Window <screen>.<n>` (Usage keeps `Self`); macro / plugin owners answer E-CMD-016 until their
  runtimes exist. CLIENT_PROTOCOL 8 (SESSION_PROTOCOL stays 8).
- **Resets**: `Verb::Reset` on objects (bare `Reset` stays the programmer reset): `Reset Self` /
  `Reset Window`, `Reset UserProfile "x" Window /Type t` (asks when > 1 window).
- **`WindowPreset`** (`commands/window_presets.rs`) on `UserProfile.window_presets` (type → number);
  the window keeps `Window.preset`; bare Store / call ask through the 12b dialog (name + overwrite
  pick list; the board's list layout became the standard dialog). FORMAT_VERSION 10, demo
  regenerated.
- **JSON export / import** (`commands/ui_files.rs`): View, ScreenConfig, WindowPreset into
  `library/ui/<kind>/` through `core::drive`; imports into your own profile, new GUIDs on a clash.
- **`windows` State feed** (`core/window_state.rs`): per window title, bar, items, states, chips,
  values, `picks`, `Self` key LEDs, preset.
- **12b pick lists**: bare `Select DataPool|Sequence|Timecode`, `Store User "x" /Profile` ask.
- **Web**: kit `<luma-window-bar>` (`kit/windowbar.js`, Showroom; checkboxes, `Label: value ▾`,
  steppers `‹ 1 ›`, chips, `…` overflow, hidden below `bar_min` → listed in the window menu;
  `data-id="<type>/<id>"`); the hand-built sysmon / DMX toolbars and their layout writes are gone
  (the layout PUT carries only splits, resizes, tabs; the sysmon search box became the `Search`
  setting + chip); live tab titles; `<luma-settings>` pins, ↺, the Window bar tab, preset / reset
  header buttons; window menu Settings… / Window bar / Reset window… / Rename = `Label Self`.
- **Known limits**: sysmon Wrap cuts long rows with "…" (the virtual list keeps fixed rows);
  Realtime's "60 fps" title is the page's own reading; `Reset Self` keeps the programmer `Reset`
  rights; keys placed in a window bar aren't offered yet (no UI places them).
- Tests: `lumacore` filtered 218 + window_bar 13 + tidy 18, `lumacore-http` 37, `lumacore-tree` 64,
  `lumacore-wire` 8, `lumacore-runtime` 4; e2e full run 224 passed / 27 skipped / 0 failed
  (new `kit-12c.spec.ts`, port base 49000).

**12c2 hardening (2026-10-13)** — after 12c: "in the users & profiles menu, we need a better new
user and new userprofile system and also delete prompt, it needs to be safe. the menu with network,
backup, quick save etc, the buttons are now too big with the icons, analyze and make sure it all
works multiplatform, same for all other windows, menus, settings etc so it all works
multiplatform." Research: the Users & Profiles code paths, a reference console (user / profile setup layouts,
`LOCK "PL"` default users and profile, the import "used by …" text; GET reads only) and a
Playwright audit on phone 390, tablet 1024, laptop 1280, desktop 1920 and WebKit (17 findings).
Found broken: the window's New profile (`Copy UserProfile`) and Delete profile both answer
E-CMD-006; Delete user runs at one tap; a deleted user keeps Admin rights through the same-named
profile's role (`role_for_user` fallback) and stays logged in; claims stay held.

- **Default is a template only** (locked): UserProfile 1 "Default" ships in
  every show, can't be edited, renamed or deleted, and **no user holds it**. A new user gets **their
  own profile, a copy of Default named after them**, or shares an existing profile you pick ("all
  users have their own profiles from the start, but can share profiles as well").
- **New user** = one station ask (bare `Store User`; the window's button runs it): Name, Role (Admin /
  Programmer / Guest, default Programmer), Profile (pick list: first row "Own, a copy of Default",
  then existing profiles to share), Password (optional, masked), Colour (next free). Works the same
  in terminals.
- **New profile** = an ask: **from Default / copy of … / import file** — never blank ("a blank copy
  should never be possible"). `Store UserProfile "name" [/CopyOf n]`, `Copy UserProfile n At m`
  (fixed; new GUIDs); duplicate names get the next free ("Bob 2").
- **Delete user**: always a station ask that says what goes — "Delete User Bob? Logged in on
  Station B. Their programmer (14 values), selection and 2 claims go with them. Profile 'Bob' is
  used by nobody else — delete it too [✓]." Delete (danger) / Cancel, `/NoConfirmation`. A
  logged-in user is logged out everywhere (shells and tokens ended, claims released). One Oops step
  brings user + programmer + profile back (not sessions / claims). Refused, coded: yourself, the
  last Admin, Admin itself.
- **Delete profile**: refused while any user uses it (coded, names them); Default refused; otherwise
  an ask, one Oops step.
- **Rights only from the User**: the role on the profile and the name fallback go (§21: a profile is
  only views, screens, keys, settings); Admin-only refusals answer E-RIGHTS-001.
- **Every prompt in the window is a station ask** (New user, Password…, Import, Rename): no page
  dialogs; `Set User "x" Name "y"` (not Admin; the GUID keeps every link).
- **Menu tiles**: one kit **tile grid** for Menu, Shutdown and Backup — compact tiles (icon 24 px,
  one-line label, ~96 px high, ≥ 44 px), columns auto-fill (2 on a phone, 4–5 on a desktop), the box
  never larger than the screen and scrolls inside.
- **Multiplatform contracts in the kit** (all audit findings now; the Network / DMX Protocols /
  LumaSessions / Backup rebuild stays 12d with a 44 px stop-gap): overlays ≤ screen − 32 px with a
  fixed title row and an inner scroll; the title row never pushes × off (actions wrap or go into `…`
  below ~600 px, container queries); a 44 px floor for every interactive element on coarse pointers
  (closes, tabs, steppers, round buttons); fonts only through `--lc-fs*` tokens (≥ 12 px); safe-area
  insets as `calc(base + env(…))` on every edge bar and fixed element; layouts sized by container
  (Users & Profiles, settings) not the viewport; the phone command line as the Phone board (PURR ·
  line · VK, status in one overflow) and suggestions clamped to the screen; pool cells Fit on touch;
  every hover / right-click action has a tap or hold equal (DMX cell info, view-button menu), F11 /
  F12 reachable by touch.
- **Guards**: a tidy rule (no raw px font or grid sizes outside the kit tokens) and a Playwright
  spec that opens every window, menu, settings window and dialog on phone, tablet, laptop, desktop
  and WebKit and fails on touch targets < 44 px, content off-screen, horizontal scroll, text < 12 px.
- **A desk view on a phone or tablet** ("we also need an artifact that shows a view that was made for
  desktop and how it scales to a phone. this is a big thing that we need to discuss properly").
  Researched (LumaCore keeps every ratio today; established consoles never reflow — fixed grid, scale / scroll, Web
  Remote streams pixels, a Remote user with its own config; Grafana, Home Assistant, Node-RED,
  react-grid-layout, Gridstack, VS Code, TouchOSC, Companion, Eos apps). Options A–E (collapse to a
  stack, per-size layouts, fold to fit, own config per device, scale down; board ResponsiveViews)
  were all **not taken**: "i think we should try to still keep the panes on the phone but utilize
  the hold-fullscreen feature … its ok if a group for example would be 1x1 pixel, it should not be
  tapable at that size, but hold to fullscreen here is key" (board HoldFullscreen, tried):
  - **Same view everywhere**: a phone or tablet shows the desk's screen with every pane in its place
    and proportion; "we want to have the same view as on the desktop so if we change anything on
    the phone it should happen in the desktop app" — live screens stay shared, nothing is local.
  - **Miniature**: each pane draws its content scaled down (the desk, smaller); anything under the
    44 px floor is **not tappable**; a quick tap there says "Too small — hold to open <window>".
  - **Hold a pane (≈ 0.4 s, a ring fills) = that window full screen** at real size, everything
    tappable; ‹ › to the neighbour pane (tree order), × back. Hold on a cell big enough to tap keeps
    its own hold (the pool's full-screen selector); hold on something too small opens the pane.
  - **Does the desk maximise too?** A profile setting (UserProfile prop, default: yes — the desk
    follows); off = the full-screen pane is only on that device.
  - **Tablets: the same rule.** Each device still picks its screen config (device setting "Screen
    config: Follow user / <n>", default Follow user).
- Mockups first (canvas row 11), then go.

**12c2 design locked (2026-10-13)** — "approve": UserAsks, MenuTiles, PhoneFixes, HoldFullscreen
(+ ResponsiveViews as the record of the options not taken) in
[docs/dev/ui-kit/mockups/2026-10-13-12c2/](../dev/ui-kit/mockups/README.md). The PhoneFixes board's
single Groups frame is a one-pane view (or a held pane); the settings and Users & Profiles frames
hold as drawn. 12c2 is ready for go.

**12c2 built (2026-10-13)** — show format 11, `CLIENT_PROTOCOL` 9, `SESSION_PROTOCOL` 9 (the role left
the profile).
- **Profiles:** 1 Default is a locked template (`UserProfile.locked`, `ensure_default_profile`); a
  dispatch backstop refuses any line that changes it (E-USER-002), except a line that replaces the
  whole show (`AppState::show_loads`: LoadShow, NewShow, CleanStart bring their own Default). The
  demo is 1 Default, 2 Admin, 3 Debug, 4 LD. Rights come only from the User
  (`ObjectDef.rights`); the old same-named-profile role fallback is gone.
- **Station asks:** `Store User` (own copy of Default or a shared profile), `Store UserProfile`,
  `Copy UserProfile n At m` (fixed), `Import UserProfile … /As`, `Label UserProfile`,
  `Set User "x" Name "y"`, `Delete User … [/WithProfile]` (says what goes, logs out on every
  station, revokes tokens, releases claims, one Oops step; Oops restores users and profiles
  before programmers). Refusals E-USER-003…009; a profile in use can't be deleted. Loading a show
  without a logged-in user logs that user out.
- **Phone / tablet desk:** a pane under its type's `bar_min` draws in miniature (kit `miniature()`;
  under 44 px not tappable, a tap says "hold to open"); hold 0.4 s = full screen with
  `<luma-maxbar>` ‹ › ×. Profile `HoldFollows` (default On) mirrors it as `Set Self Maximized`
  (`Screen.maximized`, in the windows feed); a desk can Maximize from the window menu.
  `Set Device ScreenConfig <n>|Follow` (`Status.device.screen_config`) wins over local choice.
- **Kit:** `<luma-tiles>` (Menu, Shutdown, Backup), `toast()`, `<luma-maxbar>`, `miniature()`;
  overlay contract `.lc-scrim/.lc-box/.lc-box-title/.lc-box-body/.lc-close` (50 px ×, body
  scrolls, box ≤ screen − 32 px); 44 px floor on coarse pointers; every font size a `--lc-fs*`
  token (≥ 12 px); safe areas; container queries (Users & Profiles three / two / one column);
  phone command line PURR · line · VK · ⋯, suggestions clamped; tap on a DMX cell = info, hold a
  view button = its menu; F11 / F12 in the window and Shutdown menus.
- **Guards:** tidy `frontend_font_sizes_are_tokens`, `font_and_grid_scanners`
  (`tidy: px-ok` / `grid-ok`); e2e `support/fit.ts`, `multiplatform.spec.ts` (five screens),
  `kit-12c2.spec.ts`.
- **Left for later:** the New user ask stacks its fields in one column (the board shows two);
  the phone keeps the left sidebar; pool Delete and window / view Rename still use page dialogs
  (outside Users & Profiles).

**12d hardening (2026-10-13)** — "lets harden 12d, read all md files with any related features to
setup, new and old". Research: the docs and code sweep, a reference console's setup menus (corpus + live reads),
eight big server–client projects' push protocols (Home Assistant, Grafana Live / Centrifugo,
Phoenix / Supabase, Meteor DDP, Kubernetes watch, OBS WebSocket 5, graphql-ws, Syncthing), the
rights levels across every old and new doc + a reference console, and backup / recovery screens in 14 projects
(reports in `handoffs/ui-kit/research/12d-*.md`). Decided:

- **Menus in scope:** Network, DMX Protocols, LumaSessions, Backup + a new **Settings** menu.
  Patch stays its own step.
- **Shape: a `MenuDef` registry → JSON** (the 12c model): tabs, content kind, function-block actions
  (command-line templates), bound settings, the power toggle and rights; one kit `<luma-setup>`
  draws any menu. `Menu "<name>" [/Tab x]` opens one (names from the registry, completion; the same
  line again closes it — never a stacked copy); `List Menu` / `Info Menu "x"` publish them.
- **Overlay**, never larger than the screen; full screen on a phone (left tabs become a top strip,
  the function block a 2-column block at the bottom, power first; container queries).
- **Layout (a setup frame, ours):** title row with status chips and ×; left tabs; content; function
  block of 60 px cells = command lines or bound `Set` cells (at most 4 cells; every other setting on
  the menu's settings tab, drawn by `<luma-settings>` — one home per setting); a big power toggle.
  The last tab per menu is kept on the profile (not an Oops step).
- **Power toggle, per menu:** Network = `Set Station Online` (asks before Off: it stops Shell, HTTP
  and DMX), LumaSessions = Join / Leave this station's LumaSession, DMX Protocols = output held /
  sending. Backup and Settings have none. Three looks: on, off, on but failing — with a line
  inside saying what a press does. (Fixes the 12 board's `Set Station LumaSession On|Off`.)
- **Colours:** only the power toggle is filled; rows, cells and tabs carry a dot or stripe plus
  words. The 12 Network board's green fills are corrected.
- **Rows:** tapping a row highlights it in the menu (client only) and the block's line templates get
  its name — every line names its row; nothing hidden on the server.
- **Dangerous actions are station asks:** Online Off, Leave, Dismiss, Claim master, delete line /
  network, HTTP off on the NIC this page uses, Load / New show. Click-again is gone.
- **Network:** tabs Interfaces (each NIC, link, services LumaSession / Shell / HTTP) · Networks (the
  show's networks and this station's NIC for each). Stations move to LumaSessions.
- **DMX Protocols:** left tabs = the show's networks with a stripe for what is really sent (grey off,
  green out, orange in, cyan both — computed on the station); inside Lines · Routing · Discovery; the
  block holds the selected line's settings. A member shows the master's live line states ("sent by
  FOH") plus its own network→NIC map.
- **LumaSessions:** Stations (a card per member, health from the master) · Known (every station that
  ever joined; re-invite) · This station. New `Claim LumaSession Station "X"` on the master hands the
  master role over.
- **Settings:** Station (name, location, priority, ThemeOverride, MulticastBase, startup show) ·
  Show (autosave, merge default + timeout, description) · Assets (AssetRoot per drive) · About
  (build, protocols, paths, disk). It must be clear where each lives: every tab and cell shows
  "This station · never in the show" or "In the show · every station".
- **Typed props:** Station, Interface services, the network→NIC map and the show's autosave / merge
  settings become declared props (`TreeDef::write`, kinds, rights) — `Set`, `EditSetting` and the
  cells use them; the hand parser of `Set Station` goes.
- **Backup:** tabs Show · Library · Recovery.
  - The timed autosave writes a **recovery copy** (untitled shows too); the show file changes only
    when someone saves. **Every save keeps the previous file as a version**; a Snapshot is a pinned
    version with a note. A `before-load` copy is written before Load / New / CleanStart replaces
    unsaved work. Opening a copy loads it as unsaved changes to its show.
  - Cleanup is a station setting: all from the last hour, 1 per hour today, 1 per day for 7 days,
    1 per week to 30 days, a size cap (500 MB); signal / merge copies ≥ 30 days; pinned never.
    Delete moves a show to `.trash` for 30 days.
  - **Startup recovery:** each station marks itself running; after a crash, if a recovery copy is
    newer than the saved show, one station ask (Open copy / Open saved show / Show Recovery…). It
    never loads by itself and never asks twice.
  - **Manifest facts (format 12):** saved_at, reason (save / timed / signal / merge / before-load /
    snapshot), station, user, note, counts. Lists read only manifests; one sanitised name scheme.
    Show header: saved time, "Autosave in …", free disk (warn < 15 GB, alert < 5 GB). Library:
    search as you type, no hidden filters; Import refuses a newer format before copying. Partial
    load (pick parts of another show by GUID) is later.
- **Everything live on `/ws/feeds`** (the 12b2 socket): new State feeds `network`, `lumasessions`,
  `shows`, `station` (users, presence, show status), `screen` (views / layout / ReloadUI / capture),
  the pool editor. `/api/events` and `/api/station/events` are retired, and every poll goes; fetch
  stays only for one-shot loads.
  - The socket opens without login; feeds marked `open` in the feed registry work before login (the
    lock screen); `auth` on the socket keeps every subscription.
  - **Shared builds:** one build per (feed, params), only while subscribed, rebuilt when the
    station's own counters move, serialized once and sent to every subscriber. Small states are
    replaced; big keyed lists send `set` / `del` against a revision. A reconnect sends its revision:
    the same → `unchanged`, else a full snapshot.
  - **Limits (Home Assistant):** a server `hello` with the protocol number and the feed list; every
    `sub` is answered `subbed` or `nosub` with an `E-FEED-…` code; batches due in one tick go as one
    frame; a slow client keeps only the latest State, and a queue cap or 5 s over peak closes it with
    a code; 64 subscriptions per page; the `screen` subscription counts as presence; ReloadUI and
    capture are Samples, so none is lost.
- **Rights: one level per user, an established ladder** — Guest < Playback < Presets < Program < Setup <
  Admin (+ Internal). The user property is `Rights` (Guest, not View, and Program, not Programmer:
  both are object words). Only Admin grants levels.
  - Playback may select and set values, never store; Presets may update existing presets, not create
    objects.
  - Setup: Claim Patch / the full patch, networks, lines, routes, `Set Station` / `Interface`,
    LumaSession Join / Leave / Invite / Dismiss / Claim / Delete Station, merge and autosave
    settings, LoadShow, Shutdown / Restart, Clear Programmer User. Admin keeps users and profiles,
    `/Force`, NewShow, CleanStart, Debug.
  - Every check is declared (the handler-side Admin checks move into `rights`); generic verbs take
    their object's or property's level; a Guest keeps the core keys (DeskLock, full screen) and
    their own password, colour and look.
  - An OSC line without a user is refused (it never runs as Admin by default; Admin named
    explicitly is fine, §31 "OSC line users"). Macros and keys run as the
    pressing user, all or nothing. Controls below your level are greyed with the reason.
  - The sweep runs one user per level.
- **One grid for the whole system: `<luma-grid>`** ("we need grid cells and columns and rows like
  [established consoles]", "this is a big an important thing we need to unify in the system so all grids looks the
  same"; research `handoffs/ui-kit/research/12d-grid-*.md`). An object-grid model, our
  gestures:
  - Rows are keyed by GUID; columns are declared in the registry (prop, kind, width, priority,
    rights); every gesture ends as one command line naming its rows — never a hidden `UiGrid`.
  - **Selection** (cells, rows, columns; a set of GUIDs, so it survives scrolling, sorting and live
    updates): a tap or lasso replaces the selection (a lasso that starts on a selected item too);
    the row-number gutter selects rows, a header tap the column on the selection, twice the whole
    column. **`+` / `−` / `Thru`** buttons in the selection bar add, remove and make a range (tap A,
    Thru, tap B). **`+` and `−` latch** on the first press and stay on until pressed again (or the
    other is pressed) — "so i can select multiple things easier"; `Thru` is one-shot. Mouse: Ctrl = `+`,
    Alt = `−`, Shift = `Thru`. A sticky bar shows "n selected (k hidden) · All · None · Invert".
    Where order matters (patch) the order shows as small numbers.
  - **One-finger drag** is a device setting `Drag: Auto | Lasso | Scroll`: Auto = lasso with
    two-finger scroll and visible scroll bars on a desk or tablet (≥ 600 px, or a mouse / pen),
    one-finger scroll on a phone. Hold-then-drag lassos in every mode, and the selection bar has a
    lasso toggle. A lasso starts after 6 px or ~150 ms; a second finger before that scrolls and
    puts the selection back, after it is ignored; selection lands on finger-up; edges auto-scroll.
    The same selection surface later serves pools and layout canvases. Pinch does nothing in grids
    and pools (font size is the window's setting) and zooms layout canvases.
  - **Edit:** a two-finger tap (fingers not moving) edits the selection on touch — the safe main
    gesture; one finger reaches the function block's **Edit** cell (with the count) or an Edit chip
    that appears beside the selection after a pause, so a scroll never edits; toggles never flip on
    a plain tap. Desktop: double-click, Enter / F2, Ctrl+Enter, right-click, or typing. Editors are
    kit elements by kind (keypad, pick list, toggle, text, colour): a popover ≥ 600 px, a bottom
    sheet with OK / Cancel on a phone (placed by `visualViewport`), headed "Rate · 3 lines".
  - **The edit card** ("when i multi select rows in the grid and press edit, it should open a dialog
    so i can see a unified card with tabs and <different values>, this way it will be easier to edit
    on the phone as well. so we can have both the grid and the edit dialog card"): Edit (slab cell,
    chip, two-finger tap) opens one kit dialog for the whole selection — tabs from the object's
    declared settings sections, one cell per field (the `<luma-settings>` cells), each showing the
    shared value or **‹different values›** with the range ("25–44 Hz" / "Full / Changes"). Changing a
    field sets it on every selected row that has it, as one line and one Oops step (rows without the
    field are skipped and named); per-row fields (Name) are read-only with several rows. Cells
    selected in one column open the card on that field. No OK — × closes. On a phone the card is
    full screen: tabs on top, one column, the value picker as a bottom sheet. Both stay: the grid for
    direct cell edits, the card for many fields on many rows.
  - **Multi-edit:** editing with several selected edits that column on all of them as
    one line (`Set Output 1 Thru 3 Rate 44`) and one Oops step; `21 Thru 30` spreads, `+ − × ÷ %`
    are relative; mixed values show "Mixed 25–44"; Fill / Fill series and Paste over a range are
    actions. Refused rows are named with their codes, their cells marked, the selection kept; with a
    filter on, an edit applies to every selected row and says "(1 hidden)".
  - **New-object line** last ("+ New Output"; on/off per grid on the profile). **Columns:** a header
    menu (⋮ or hold) — sort asc / desc / none, hide, move, width; a yellow filter row (text or pick
    cells, AND, a funnel on filtered headers); column sets Condensed / Full / Extended; factory
    columns from the registry, each user's changes per grid on their profile. Leading columns (No,
    Name) are frozen; on a phone low-priority columns fold into an expandable row and the rest scroll
    sideways.
  - **Looks (established meanings, our tokens):** editable cells have a cell box, read-only ones sit on the
    grid background; selected cells a selection fill, selected rows an accent border, focus a 2 px
    frame; invalid rows alert, warnings warn text; cut-waiting-for-paste an alert tint; a send /
    receive flash on the name; rows 48 px on touch, 40 compact; nothing re-sorts under an open editor
    or a finger. Virtualized beyond ~200 rows.
  - **Function block = the fixed slab** in every menu (a 2-column block at the bottom on a
    phone): left, in this order, on the selection: Select · New · Copy · Cut · Paste · Delete · Oops ·
    Import · Export (+ Edit); right: the tab's actions, up to 4 bound cells and the power toggle.
    Labels carry the count ("Delete 3"); actions that don't apply are greyed and say why; deleting
    several asks.
  - 12d uses it in the setup menus, Users & Profiles and the show lists; a tidy rule forbids other
    table markup; every later grid (patch, sheets, editors) uses it.
  - **Slab icons and colours** ("the slab needs minimal icons and standard colors"): each edit action
    has one minimal icon in a standard colour, the same in every menu — Select (pointer, accent),
    New (plus, ok), Copy / Paste (info), Cut (scissors, warn), Delete (trash, alert; its label turns
    alert only when it applies), Oops (undo, warn-amber), Import / Export (arrows, teal), Edit
    (pencil, accent). The icon carries the colour, labels stay plain; greyed actions grey the icon.
  - **Every setup list is a `<luma-grid>`** ("network i need to be able to grid select for example
    all shell cells and edit them"): Network › Interfaces (services as Yes / No cells; selecting the
    Shell column on several interfaces + Edit = one `Set Interface en0 + en1 Shell No` line),
    Networks, DMX lines, routing, discovery, LumaSessions stations (with a card view) and the show
    lists. A grid whose rows come from the hardware has no new-object line and greys New / Cut /
    Delete with the reason.
- **The slab is the kit's one action block** ("i think we also need the slab in some other menus now,
  like the user and userprofile menu add and delete, make sure to unify this in the UI as a tidy
  requirement"): every window or menu that lists objects (setup menus, Users & Profiles, later
  editors) puts its add / delete / copy / … actions in the kit's function block (`<luma-slab>`) with
  the standard icons, colours and order, on the selection of its `<luma-grid>` — on a phone the same
  slab as a 2-column block at the bottom. Users & Profiles moves onto it: a Users grid and a Profiles
  grid (name, rights level, profile, colour, password set, remote login, logged in · no, name, used
  by, screens, views, locked), New = the station ask, Delete = the safe delete ask (12c2), Edit =
  the selection's cells. Tidy: no hand-made add / delete / copy buttons or other action rows outside
  the slab, and no table markup outside `<luma-grid>`.
- **Approved boards:** SetupNetwork (Network on the grid), SetupDmx (the `<luma-grid>` board), PowerToggle
  (look A) and SetupUsers (Users & Profiles on grid + slab, desktop + phone, one selection — the
  model every 12d board now follows; each board carries its own phone layout).
  Also approved: SetupSessions (LumaSessions: Stations grid | cards, multi-station edit) and
  SetupSettings (kinds shown with neutral badges, home marks, rights levels).
- **Station settings home: the station split (approved)**: rig wiring per station — network→NIC, services
  and later timecode (LTC / MTC) and DMX ports — lives in a **station table in the show** (one GUID
  row per station, replicated in the LumaSession, pre-configurable while a station is offline;
  each station applies only its own row and keeps a last-applied copy). Identity — name, LumaSession,
  priority, location — stays **on each station** and is edited remotely through the master. The
  LumaSessions › Stations grid shows both; selecting several stations + Edit = one
  `Set Station "FOH" + "Tower" …` line. This changes §10 for the wiring part (output stations
  vs the session socket, from the 12d stations research).
- **Load show = one dialog with parts** ("i dont like load parts, we rather have it say load show then
  show the load parts dialog so we can get partially things from a luma file"): Backup › Library ›
  Load… opens the Load show dialog: the file's facts (from its manifest), then its parts — Show data
  (groups, presets, sequences, macros, pools) · Patch · DMX protocols · Users & profiles · Station
  table — each ticked by default, tri-state, and openable (Pick…) to take single objects. All ticked
  = `LoadShow "x"` (replaces the show; asks first about unsaved changes). Anything less =
  `Import Show "x" <objects>` — taken into the current show by GUID (same objects updated, new ones
  added), one Oops step; unticked parts keep what is loaded now. This replaces the separate "load
  parts" and a partial-show-read wizard (the GUID merge of `lumacore_tree::merge`). Snapshots stay open
  (versions with Pin in the meantime).
- **Show history = one row per save** ("we also need the showfile history like [established consoles](only a list of save
  dates basically, maybe other good data?)"; from the 12d history research).
  A reference show history only adds a row when version / name / description change and has no user,
  station or reason; ours merges history and versions: Backup › Show · History is a grid with Saved,
  By, Station, Reason (Save / Quick / Save As / Recovery / Before load / Lost merge / Signal / Format
  / Restored), Note, Changes (counts against the save before: "+12 fixtures · −1 sequence") and Kept
  (current / version / ★ pinned / pruned — a pruned row stays, its file is gone). Rows come from each
  file's manifest; the manifest also carries the last ~20 history rows so a copied file still shows
  where it came from. A row: Open version (as unsaved changes), Take parts… (the Load show dialog),
  Pin, Delete. Library › History… reads a show's history without loading it. Reading needs no right.
- **History matrix + coloured show actions** ("save show, load show, info and history buttons need
  distinct colors, show history is good but we need more like a object matrix because a lot can
  change during a save"): Show · History switches List | Matrix. The matrix has a row per save and a
  column per object kind (Fixtures, Patch, Groups, Presets, Sequences, Cues, Macros, Lines, Users,
  Profiles); each cell shows +added ~changed −removed, compared by GUID with the save before; tapping
  a cell lists the objects ("~ Sequence 1 (5 cues retimed) · + Sequence 2 Cue 14 Thru 16"). The
  manifest keeps, per kind, a GUID → content-hash index so the diff needs no full load; the phone
  shows each save with its changed kinds. Show actions carry one icon and colour each, the same
  everywhere: Load show (folder, the gold primary), Save show (save, ok green), Save as (save +,
  green), New show (file +, neutral), Info (info, blue), History (clock, violet) — a faint tint of
  the colour on the cell.
- **Show actions are fixed** ("i am now missing save show as well. the phone is missing load and new
  show"): Library and Show · History both carry Load show… (primary) · Save show · Save as… · New
  show… in the same place. On a phone a tab's main actions come first in the slab (Load show, Save
  show, New show; LumaSessions: Invite, Hand master), then the standard edit actions.
- **Settings show their kind** ("i want i to be clear what is a string, int, bool and a list of drop
  down, it needs to be more visually clear"): every settings cell and edit-card field carries a kind
  badge — **Aa** text (an underlined field), **123** number (right-aligned value + unit, − / +
  steppers, keypad on tap), **I/O** switch (a toggle), **≡** list (a ▾ box, pick list on tap) — each
  in one neutral outline ("the color chips was too much": no colour per kind); grid columns of kind list show ▾. Read-only cells grey the badge.
- **Power toggle look A** (approved from the PowerToggle board): the tile — a big power icon
  centred, the state word (ONLINE / OFFLINE / SENDING / OUTPUT HELD), what a press does under it, and
  a 5 px state-coloured bar along the bottom; "on but failing" fills the tile dark alert with the
  reason as the line.
- **12d design locked (2026-10-13)** — every board approved: SetupDmx (`<luma-grid>`), SetupNetwork,
  SetupUsers, SetupSessions, SetupBackup, SetupSettings and PowerToggle (A), in
  [docs/dev/ui-kit/mockups/2026-10-13-12d/](../dev/ui-kit/mockups/README.md) (canvas row 12). Still
  open, not blocking: named snapshots — pinned versions with a note stand in until decided.
- **Tests:** Rust (feed protocol, MenuDef + rights per level, autosave / versions / cleanup /
  recovery, manifest) and Playwright per menu (opens by command, every cell runs its line, no fetch
  loop, the lock screen on the feed socket, phone fit).
- **Mockups after all decisions** (a new canvas row): the setup frame, each menu, the power toggle's
  looks, Backup's tabs and the recovery ask, Settings' home marks, phone.

**12d part 1 built (2026-10-13): everything live on `/ws/feeds`** — the feeds bullet of "12d
hardening" above, nothing else of 12d yet (menus, rights, grids, backup are later parts).

- **Socket**: opens without a login; `hello` (`client_protocol`, `epoch`, `user`, the feed
  registry `FEEDS` with kind / open / params, `max_subs`); `auth` → `authed` (a token on the
  upgrade still logs in at once); every `sub` → `subbed` or `nosub` with E-FEED-001…004
  (catalog `lumacore_http::errors`, listed in `core::errors::CATALOGS` with the `http`
  feature; 005 slow, 006 token refused, 007 not an op). Logout / a revoked token ends the
  login feeds with `nosub` E-FEED-002, the FeedClient keeps them and sends them again after
  the next `authed`; another user's `auth` restarts them as theirs. `CLIENT_PROTOCOL` 10.
- **New feeds**: `station` (open), `network`, `lumasessions`, `shows`, `screen` (samples:
  views / screen changed, ReloadUI, capture, 256 kept, a fresh sub starts at the newest; the
  sub is the Screenshot presence), `pool_editor`. `/api/events` and `/api/station/events`
  are gone; the overlays', the pool editor's, Users & Profiles' and Backup's polls and
  after-command fetches are gone (tidy `frontend_has_no_event_stream_or_fetch_poll`; the
  router test also refuses an SSE route).
- **Shared builds** (`websocket/hub.rs`): one producer per (feed, user, params) while
  subscribed, woken by `lumacore_http::Revisions` (state thread `FeedRevs`: show publish,
  station view / show status / claims / menu views / revoked tokens compared at each
  publish, `drive::writes()` for the library, plus the screen-event seq for `windows`) —
  `station`, `network`, `lumasessions`, `shows`, `pools`, `pool_editor`, `windows`. The
  state thread builds the Network / LumaSessions views (and their NIC list) only while
  watched (`FeedSignals`); the one-shot `GET /api/network` builds once on the state thread.
- **Revisions**: `rev` = a 53-bit FNV fingerprint of the build's JSON, not a counter
  (deviation: the same state has the same rev, so `unchanged` also works across a restart;
  the `epoch` isn't needed for states). Keyed diffs `{list, key, fields, set, del, order?}`
  against `base` for `shows`, `pools` (cells by `no`), `pool_editor` (members by `fixture`)
  when under half the data; `station` is replaced whole.
- **Limits**: due batches of one tick go as one JSON-array frame; a writer queue of 256
  (full, or ≥ 64 for 5 s → close 4008 E-FEED-005); States wait while the queue is behind;
  2 s send stall; ping 15 s, silent 45 s; 64 subs; jittered reconnect 0.5–8 s.
- **Known limits**: `health` and `dmx` stay sampled each period (atomics / engine frames,
  no counter); `shows` re-sends each second while the autosave countdown runs (it is in
  the status); a pool with a fading group master is rebuilt every 100 ms (`moving`); no
  per-sub `gen` yet (an in-flight batch of replaced params can still arrive); no cap on
  feed sockets per device; the NIC list for `ServiceAccess` is still read every second
  (it decides what each interface serves). Roadmap: "Feeds follow-ups".

**12d part 2 built (2026-10-13): rights levels** — the rights bullet of "12d hardening" above.

- **One level per user**: `lumacore_tree::UserRights` (Guest < Playback < Presets < Program
  < Setup < Admin) on `User.rights` (replaces `role`; show format 12, `SESSION_PROTOCOL` 10,
  `CLIENT_PROTOCOL` 11 — the station's users carry `rights`). `registry::Rights` is the same
  ladder + Internal. `Store User "x" /Rights Program` (default Program), `Set User "x" Rights
  Setup` (Admin only; the user Admin is always Admin, E-USER-005; last-Admin E-USER-004
  stays), the New user ask picks the level (six choices), `EditSetting User` a Rights choice.
- **Declared, one gate**: a line naming an object takes the object's level for the verb
  (`ObjectDef.rights`, now the full level, not an extra; shapes in `core::rights`:
  `content`, `presets`, `fixture`, `setup`, `screen`, `playback`, `anyone`, plus User,
  UserProfile, Show and the device words' own); otherwise the word's `rights`, raised by the
  new `CommandDef.arg_rights` (`Claim Patch` / any area → Setup, the datapools Program,
  `/Force` → Admin; `Clear Programmer …` → Setup; `SaveShow` with arguments → Program). `Set
  <object> …` also takes the property's level (a tree path is checked by `props::set`). The
  handler-side Admin checks are gone (CleanStart, Debug, Store / Delete / Set User,
  profile import, `/Force`, Claim LumaSession, Delete Station, `Show /MergeDefault|
  /MergeTimeout`, Clear Programmer User). E-RIGHTS-001 names both levels ("'Claim Patch'
  needs Setup — you are Program."); Help "Needs:" lists an object's levels per verb; the
  reference has a rights fact per object and the property levels.
- **Counts** (generated reference): commands Guest 39 · Playback 27 · Presets 1 (SaveShow) ·
  Program 23 · Setup 10 · Admin 3 (CleanStart, NewShow, Debug) · Internal 1; the 75 objects
  by their highest verb: Guest 29 · Playback 14 · Program 21 · Setup 9 · Admin 2 (User,
  UserProfile).
- **OSC**: `Store Input … /Protocol OSC` without `/User` is refused (E-RIGHTS-003, new); a
  line without a user doesn't start (state "no user", System Monitor once per episode); an
  untagged inbound command is refused, never run as Admin.
- **Sweep**: `ROLES` = one user per level, generated from `UserRights::ALL` (Admin first, the
  reference), plus Owner (Program). Not run long here.
- **UI (kit only)**: Users & Profiles shows each user's level and greys New user / Rename /
  Delete / another's Password and the profile actions with "Needs Admin — you are Program";
  the lock surface's avatars carry a level badge (`luma-avatar level=`); suggestions dim with
  the same text.
- **Deviations**: ownership rules stay in handlers through `rights::require` (another user's
  claim → Admin, another user's password / colour → Admin, a profile you don't use); `Claim
  LumaSession` is Setup and the station-priority bypass for lower levels is gone; an OSC line
  may name a user up to your own level when stored, but `Set Input n User` stays Admin;
  `Set Self` / window settings are Playback, so a Guest's window bar is refused.
- **Limits**: settings cells (`EditSetting`) aren't greyed by level yet (the Set line is
  refused); `Store Preset … /Merge` / `Update Preset` are declared Presets but the preset
  verbs are still planned; no macro / key minimum level (they run as the pressing user,
  all or nothing). Roadmap: "Rights follow-ups".

**12d part 3 built (2026-10-13): the grid, the slab, the setup frame and the MenuDef registry**
— the kit and registry foundations of "12d hardening" (one grid, the slab, kinds, power toggle
look A, `MenuDef`, typed props); part 4 rebuilds the menus on them, part 5 does Backup.

- **Kit** (`apps/lumacore-app/frontend/kit/`, Showroom section "12d", `showroom12d.js`):
  - `<luma-grid>` (`grid.js`): rows keyed by `row_key`, columns from descriptors; selection =
    row keys + cells, kept through scroll / sort / feed updates; the gutter (a declared `no`
    column is the gutter) selects rows, a header the column on the selected rows (twice or with
    nothing selected: the whole column); `+` / `−` latch (the other one unlatches), `Thru`
    one-shot (A, Thru, B = the rectangle; Shift), Ctrl = `+`, Alt = `−`; All / None / Invert and
    "n selected · k cells (h hidden)"; one-finger drag per the device setting `Drag: Auto |
    Lasso | Scroll` (`localStorage lc.grid.drag`, the bar's "Lasso:" button cycles it; Auto
    lassos with a mouse / pen or ≥ 600 px, scrolls with one finger on a phone, two fingers scroll
    in lasso mode), hold-then-drag lassos, start after 6 px / 150 ms, a second finger before the
    start scrolls, the selection lands on finger-up, edges auto-scroll, pinch does nothing; Edit
    by a two-finger tap, the Edit chip (after 0.7 s beside the selection), double-click, Enter /
    F2 / Ctrl+Enter, right-click, typing; header menu (⋮ or hold): sort, hide, move, width
    (keypad), the yellow filter row (contains, AND, funnel on the header), column sets
    Condensed / Full / Extended (by `priority` 1 / 2 / 3), the new-object line on / off, reset;
    frozen leading columns, a phone folds priority > 1 into an expandable row; looks per the
    board (boxed editable cells, read-only on the ground, selection fill + frame, accent gutter
    on whole rows, 2 px focus, invalid / warn / cut / flash), 40 / 48 px rows, virtual beyond 200
    rows, order frozen under a finger or an open editor; a Grid | Cards view on the same
    selection. Every change is ONE line naming its rows (`rowsWord`: `1 Thru 3 + 5`, `en0 + en1`,
    rows of another object word grouped as `…; Set Input 1 …`); column state lines are `Set
    GridColumns "<key>" Columns|Widths|Sort|ColumnSet|NewLine|Default …`.
  - The edit card (`editcard.js`): tabs from the columns' sections, settings-like cells with the
    kind badge, the shared value or ‹different values› + range ("30–44 Hz", "Full / Changes"),
    per-row fields read-only with several rows, rows without the field skipped and named
    ("2 of 3 have it", the last line), greyed below the viewer's level; a number typed relative
    on the old value (`25 + 5`) goes as `+ 5` for every row; no OK, × / Esc; full screen on a
    phone.
  - `<luma-slab>` + `<luma-power>` (`slab.js`): the fixed left half (icons + standard colours as
    tokens `--lc-act-*`, counts, greyed = grey icon + the reason as a toast), the tab's actions
    (show-action tones with a tint, Load the gold primary), ≤ 4 bound cells, the power toggle
    look A (5 px bar, failing = dark alert fill + reason); a 2-column block on a phone (power
    first, the tab's `phone` actions, then New · Delete · Edit · Copy · Oops · Import); a tablet
    stacks icon over label.
  - `<luma-setup>` (`setup.js`): title row (icon, chips, Filter row / Columns / Grid | Cards,
    the home mark, ×), left tabs with a sub line (a top strip on a phone), content by kind
    (grid / cards / settings / custom / empty), the last line ("lines name their rows: …"),
    the slab; ≤ screen − 32 px, full screen on a phone. `{object}` / `{rows}` / `{at}` /
    `{value}` fill the templates (`{value}` asks with the kit dialog).
  - `<luma-settings>`: the kind badge (Aa / 123 / I/O / ≡, neutral outline, grey when read-only),
    text underlined, numbers right-aligned with unit and − / + steppers (`+ 1` lines), a switch, a
    list box with a chevron; `home` marks (long / short by width), `rights` greyed below
    `spec.level` with "Needs Setup — you are Program" (the 12d part 2 follow-up), `readonly`
    with its reason; `spec.note`.
- **Registry / station** (Rust): `core::menu_defs` — 8 MenuDefs (Menu, Shutdown = page; Settings,
  Network, DMXProtocols, LumaSessions, Backup, Users = setup) → `GET /api/menu-types`
  (`ConsoleBridge::menu_types`); columns take kind / range / options / rights from the declared
  props, slab cells from the verbs the object really has (planned → "… is planned"), power
  `{feed, path, invert, off_values}`. `Menu` is now an object (Call / List / Info; the command
  entry went): `Menu "<name>" [/Tab x]` sends `OpenMenu { menu, tab, toggle, grids }`; `/Tab`
  stores the last tab on the profile (Untracked, a locked profile keeps nothing). `GridColumns`
  object on the profile (`UserProfile.ui: ProfileUi { menu_tabs, grids }`, Untracked).
  `core::settings_props`: typed keyed settings for Station (Name, Location, LumaSession,
  StartupShow (new), Priority, ThemeOverride, MulticastBase, Online, Invite, HoldOutput,
  OutputWhenAlone), Interface (LumaSession / Shell / HTTP), Network › Interface and Show
  (Description, Autosave, MergeDefault, MergeTimeout) with home / unit / section / rights —
  `Set` and `EditSetting` use them; the hand parsers (`set_station`, `set_interface_services`,
  `set_interface`) are gone. `commands::set_rows`: one `Set` line for many rows (`+` / `Thru`,
  `a Thru b` spreads, relative per row, refused rows skipped and named with codes).
  `SettingField` gains `home`, `unit`, `rights`, `readonly`.
- **Page**: `open_menu` → `openMenuEvent` (toggle / tab rules, `legacy` menus keep their old
  overlay); `openSetupMenu(name)` draws any MenuDef live (rows from the tab's `feed` + `rows`,
  settings from `EditSetting <target>`, bound Station cells, the power state from its feed).
  Menu › Settings is live now (the tile was "planned"). Fix found on the way: a station
  setting change refreshes `Root.Runtime`'s network view and publishes the station view at once
  (on an isolated rig Menu → Network only followed at the next discovery beacon).
- **Numbers**: `CLIENT_PROTOCOL` 12, show format 13 (demo regenerated), `SESSION_PROTOCOL`
  stays 10. Icons +15 (`mouse-pointer-2`, `scissors`, `clipboard-paste`, `undo-2`,
  `arrow-up-from-line`, `funnel`, `square-dashed`, `ellipsis-vertical`, `minus`, `share-2`,
  `save-plus`, `file-plus-2`, `arrow-up-down`, `columns-3`, `text-cursor-input`).
- **Guards**: tidy `frontend_tables_only_in_the_grid` (no `<table>` / `role="row|grid"` outside
  grid.js; 9 old tables carry `tidy: table-ok (…)`) and `frontend_actions_only_in_the_slab` (no
  New / Copy / Cut / Paste / Delete / Import / Export buttons or action-row items outside the
  slab; 14 old ones carry `tidy: slab-ok (…)` until part 4 / 5); Rust `menus_tests` (11);
  Playwright `kit-12d.spec.ts` (8 tests × desktop / tablet / phone / WebKit); fit.ts checks
  `.lc-setup` / `.lc-card` as overlays.
- **Deviations**: Station Priority stays the Never…High choice and MulticastBase
  Default / Alternative (the board shows 100 and an IP); a direct cell edit opens the edit card on
  that field with its editor at once (the kit's keypad / pick list / dialog), not a separate
  popover / bottom sheet; Select with no select line latches the lasso; Copy / Cut keep a
  client clipboard and Paste runs `Copy|Move {object} {rows} At {first selected}`; the filter
  row isn't stored on the profile; a phone grid bar hides All / None / Invert and the Lasso
  button; Network, DMX Protocols, LumaSessions, Backup and Users stay on their old overlays
  (`legacy`) from the Menu tiles and `Menu "x"` until part 4 / 5 — `openSetupMenu` and the
  Showroom ("(registry)" buttons) draw them already.
- **Limits** (roadmap "12d part 3 follow-ups"): power presses run their line without the station
  ask (Online Off / Leave / HoldOutput asks are part 4); LumaSessions' power Join names no
  session; the feeds lack some declared fields (link, rate, …) — part 4 fills them; refused
  rows aren't marked in their cells yet; deleting several doesn't ask yet; no Fill / Paste over a
  range action; Recovery cap waits for Backup (part 5).

**12d part 4 built (2026-10-13): the menus on the frame and the station table** — Network,
DMX Protocols, LumaSessions, Users & Profiles and Settings on `<luma-setup>` per the locked boards;
the station table in the show (the approved station split); the station asks of the part 3 follow-ups.

- **Station table — where each field lives (§10 changed for the wiring part).** `ShowData.Stations`
  (`lumacore_tree::StationWiring`, `ShowSubtree::Stations`, `Stations.json`, `core::stations`):
  one row per station — GUID (`StationWiring::guid_for(station id)` when a station makes its own row,
  so both sides of a split make the same object; a fresh one for a row made in advance), number,
  name, station id (empty = made in advance, claimed by the station of that name), **network → NIC**
  (`BTreeMap<network GUID, NIC spec>`, Walk links) and **services per NIC** (LumaSession / Shell /
  HTTP). **In the show (replicated, undone, saved):** the wiring. **On the station (never in the
  show):** identity — name, location, priority, LumaSession, Online, Invite, HoldOutput,
  OutputWhenAlone, ThemeOverride, MulticastBase, StartupShow — and asset folders, the sACN CID,
  tokens. Each station applies only its own row (`apply_own_row`, on every show change: load,
  replication, Oops) into `Root.StationSettings` and saves that as its **last-applied copy**
  (`station_settings.json`); a show without a row for it changes nothing (standalone boots, other
  shows, a joining member until it wires itself). Maps for networks a show doesn't have are kept.
  The own row is made on the first wiring change from the last-applied copy. Interfaces are their
  hardware's own identity (names, not GUIDs).
- **Lines:** `Set Interface en0 + en1 Shell No` / `Set Network "Main" Interface en1` write this
  station's row (a member's goes to the master as `Changes`, like any station-local show write);
  `Store Station "Tower"` makes a row in advance (Setup, runs on the master); `Set Station "Tower" +
  "FOH" Network "Main" Interface en1` / `Set Station 1 Thru 3 Interface eth7 HTTP No` wire several
  rows in one line (one Oops step; a member or discovered station gets its row at once);
  `Set Station "Tower" Priority High` (identity) is **sent to that station's command port** and
  runs there as the user (offline → E-CMD-010 naming what does change here); `Delete Station`
  forgets the known station and its row. `List Station` / `Info Station "x"` show the table.
- **`Claim LumaSession Station "X"`** (master only; asks): hands the role to member X
  (`SessionRequest::HandTo` → `election::hand_over`); on X itself it is the plain Claim. Help /
  reference published through the LumaSession object's Claim verb.
- **Station asks** (one dialog path, `DispatchResult::Ask`, `/NoConfirmation` for one-shot paths):
  `Set Station Online Off`, `Set Station HoldOutput On` (and their Toggle), `Leave LumaSession`
  (a member), `Claim LumaSession [Station "X"]`, `Delete Output|Input|Network|Station` even for one,
  and deleting several Users / UserProfiles (one ask, one line `Delete User A /NoConfirmation;
  Delete User "B C" /NoConfirmation`) — `set_rows::delete_rows`; rows that can't go (a network in
  use, a missing line, a member in session) are refused before any ask.
- **Menus on the frame:** `legacy`, `npOpen`, the Network / DMX Protocols / LumaSessions / Users
  overlays, their CSS and every `tidy: table-ok` / `slab-ok` escape for them are gone; the Menu
  tiles run `Menu "x"`. MenuDef gains `left` (tabs from a feed: DMX Protocols' networks with the
  out / in / both / off stripe; the menu's tabs become a top strip), `Grid.left_filter`, `Grid.store`
  (New's line: `Store Output /Network {left}`, `Store Network {value}` asks), `Grid.locked` (Default
  profile: Cut / Delete / Edit / row actions greyed with the reason), `Col.tone`, `Col.expand`
  (`networks`: one "Main → NIC" column per show network on the Stations grid, prop `Network "Main"
  Interface`), `Action.mine` (your own password needs no level). Hardware rows (interfaces, routes,
  found nodes) grey New / Copy / Cut / Paste / Delete / Import with the reason (the part 3 Chrome
  finding "Delete 2"). Bound cells naming the grid's object show the selected rows' value and set
  them in one line (DMX: Rate / Send / Sync / Packet gap of the selected lines). Templates over
  rows of mixed objects become one statement per object (`Delete Output 1; Delete Input 1`).
  Chips per menu from the feeds; the LumaSessions power Join names `{station.lumasession}`.
- **Feeds** carry every declared column: interfaces (no, link + tone), networks (guid, state + tone,
  lines, counts, flow, note), lines (guid, object, rate_hz, send, sync, packet_gap, priority,
  target, universes text, network, tone; on a member "sent by FOH: …" from the master's line states,
  new `SessionRuntime.lines` → `luma_sessions.master_lines`), `routes`, `nodes`, `asset_roots`;
  lumasessions `members` (this station, the master, members, table rows: role, priority, location,
  `nic:<guid>`, rtt, users), `known`, `networks`; station users (guid, password, online), profiles
  (guid, used_by_text).
- **Settings:** Station · Show (unchanged) · **Assets** (a grid of the show's asset drives with this
  station's folder — `Set AssetRoot "Video" Folder …` is now a keyed setting; the hand parser
  `set_asset_folder` went) · **About** (`EditSetting Server`: build, client / session protocol, show
  format, station + id, show / station folders, uptime — read-only keyed settings, no home mark).
- **Fixes:** bound cells right after login (the `EditSetting` waits for the shell's first status
  and is asked again twice, 1.5 s apart, while the answer is missing); `Info Station` with no row
  in the runtime list; the grid header's "New-object line" toggle relabelled "Show the new-object
  line" (tidy false positive).
- **Numbers:** show format **14** (demo regenerated), `SESSION_PROTOCOL` **11** (the station
  table in snapshots, the master's line states), `CLIENT_PROTOCOL` stays 12 (no client packet
  changed; the page and the feeds come from the same station).
- **Deviations from the boards:** Users' "Log out" action isn't built (no command logs out another
  user) and "Rights…" opens the user's settings window (the Rights column edits in place);
  identity edits of another station go straight to its command port (the decision said "through
  the master"; **decided 2026-10-13: direct, hardened** — see "Remote station edits" below); a phone's Users / Profiles slab adds Password… / Settings… first (no other way to a
  profile's typed tabs there); DMX discovery's "Last seen" says "now" (the runtime keeps no time);
  Free disk is not in About yet (part 5's disk warning); Station Priority stays the Never…High
  choice; Settings labels are the setting names (`ThemeOverride`, not "Theme override").
- **Tests:** Rust `stations_tests` (7: own row + apply + Oops, rows in advance + multi-station
  line + rights + Info / List, claim by name + save / load, remote identity send, the asks,
  hand-over, Assets / About), `core::stations` (3), `menus_tests` (hardware greys every row
  action); the four stale tests fixed to current truth (`show_file` profile 1 = Default, `net` rights
  wording). Playwright `kit-12d.spec.ts` + 6 tests × desktop / tablet / phone / WebKit (one per
  menu through the real frame, the station ask of each power toggle, bound cells right after
  login); `feeds`, `kit-12b2`, `kit-12c2`, `kit-rights`, `multiplatform`, `screenshots-all`
  migrated from the old overlays.
- **Limits** (roadmap "12d part 4 follow-ups"): a joining member's row isn't added to the session's
  table until it wires itself; no "Add present / Remove absent"; no device-config export of the
  rig; line's NIC column is text (no pick list of a remote station's NICs); refused rows still
  aren't marked in their cells; no Fill / Paste over a range; mixed Output + Input delete asks twice.
- **Remote station edits (decided 2026-10-13, researched in
  `handoffs/ui-kit/research/12d-remote-station-edits.md`, 12 projects):** an edit of another
  station's **identity** (name, location, priority, LumaSession, Online, …) goes **straight to that
  station's command port**, not through the master. The station that owns the value writes it,
  checks rights and logs it; this works with no master and reaches standalone neighbours, which
  matters most for Priority (fixed exactly when a session is leaderless or split). Projects do the
  same for node-local settings (Consul / Nomad agents, Elasticsearch static settings,
  Syncthing, ETC GCE); shared wiring stays in the show's station table. Rejected: through the
  master (it stores nothing, so it can't validate, undo or queue, and nothing is editable without a
  master) and identity in the show (a show load could rename stations, move their LumaSession or
  change election priority). **Hardening (to build):** wait for the target's reply (timeout +
  retries, reply shows old → new); the grid cell is "pending" until discovery / the member list
  reports the new value; rights checked only on the target, with a declared level per setting
  (Name / LumaSession / Online at Admin); asks happen on the sender, the line then goes with
  `/NoConfirmation`; both stations journal it (passwords masked); Oops doesn't undo it — the reply
  offers the reverse line; last write wins, serialised on the target; an offline station stays
  refused (E-CMD-010); a Priority change takes effect at the next election and the reply says so.
  Preset identity on rows made in advance (applied once when the station claims the row) only if
  asked for.
  - **Built (2026-10-13):** `settings_props::remote_station_set` sends the line absolute
    (`Set Station Priority High /NoConfirmation`) as `SessionRequest::EditStation`; the state
    thread's tick hands it to a thread of its own (`election::edit_station`: 1 s per try, 3 tries,
    `send_with_retries`) and the answer comes back as a job (`edit_answered`): the line's client
    gets `Station "Tower": Priority Normal → High — takes effect at the next election (Claim
    LumaSession hands the master role over now)` + `Put back: Set Station Tower Priority Normal`,
    the target's refusal as it came (`refused: … [E-RIGHTS-001]`), or `didn't answer (3 tries …)
    [E-CMD-010]`; a UDP / HTTP request waits for it like a forwarded line (`forward_waits`). The
    line's own output is "sent, waiting for its answer". **Pending:** `AppState::remote_edits`
    (station-local, never saved, `core::stations`): a sent edit is pending until answered and —
    for what a beacon carries (Priority, Location, LumaSession, Invite; Name = a station of the
    new name is seen) — until discovery reports the value (30 s at most); a refusal marks the cell
    for 10 s. The `lumasessions` feed's Stations rows carry `pending` / `refused` (field → value /
    why); the kit's `<luma-grid>` draws any row's `pending` (the sent value in italics, a blinking
    warn dot, a tooltip with the reported value) and `refused` (alert outline, the reason as the
    tooltip) — a generic grid state, in the Showroom. **Rights:** `Station` declares its own levels
    (`objects::system::station_rights`): `Set Station <rows> …` passes the gate (Guest), the target
    checks with its dispatch; this station's own rows still check their declared level in
    `settings_props::apply`. Declared: Name, LumaSession, Online **and MulticastBase** at Admin
    (deviation: the research note puts MulticastBase there too — it hides the station like
    Online Off), the rest Setup. **Asks** on the sender (`station_ask`): Online Off, HoldOutput On,
    Priority Never on the LumaSession's master — for this station and others, several rows ask
    once; the button runs the line with `/NoConfirmation`. **Journal:** the target logs `command
    request from 'Anna@FOH'`, `[Anna@FOH] Set Station … → ok` and `Station: Anna@FOH: Priority
    Normal → High` (the sender found by its address in discovery, else `udp <ip>`; masked as
    before); the sender logs `Answer from "Tower" (Priority): …`. **Values:** `Toggle` and
    relative numbers need the station's value, so only a reported one takes them (Invite Toggle
    yes, HoldOutput Toggle / RecoveryKeepDays + 5 → E-CMD-008). Own-station replies now read the
    same way (`Priority Normal → High — …`). Last write wins on the target's state thread (every
    request is one job). No wire change: `SESSION_PROTOCOL` stays 11, `CLIENT_PROTOCOL` 12.
    Tests: `lumasession::election::remote_edit_tests` (5: old → new + put back + pending until
    reported + Oops sends nothing; rights only on the target at each level; asks on the sender,
    the line without one; absolute values, offline / silent refused; retry on a socket of its
    own), `rights_tests` levels updated, Playwright `kit-12d` "pending until it reports it".
    **Limits:** a UDP command request waits 1.5 s (`UDP_WAIT`) and then reads "Pending: the
    master hasn't confirmed yet" for a station edit still retrying (the answer still reaches the
    log); two stations on one machine are told apart by address only (the first discovery match
    names the sender); settings a beacon doesn't carry settle on the answer, not on a report.

**12d part 5 built (2026-10-13): Backup** — the Backup, Show History, Load show and show-actions
bullets of "12d hardening" on the SetupBackup board; snapshots stay open (pinned versions with a
note stand in).

- **The stores** (`core::backups`, station-local, everything through `core::drive`): the show file
  changes only when someone saves; **every save keeps the file it replaces** in
  `library/shows/.versions/<show>/<stamp>.luma` (UTC `YYYYMMDD-hhmmss`, `-2` on a clash; moved
  back if the write fails). **Recovery copies** in `.autosave/{show}_{reason}_{station}_{stamp}.luma`
  (one sanitised name scheme; the manifest is the truth): the timed copy (the show's `Autosave`
  minutes, counted from the last save or copy, untitled shows too — **decided 2026-10-13: the
  timer stays the show's `Autosave` setting**, only cleanup is a station setting — `autosave_tick` never writes
  the show file any more), `before-load` (Load / New show / CleanStart over unsaved work, also
  after "Don't save" — the ask says so), `shutdown`, and the existing signal / merge copies
  (`session_link::autosave_show` now writes through `backups::recovery_copy`). **Delete** moves
  to `.trash/<stamp>_<file>` for 30 days (`DeleteShow`, several shows `"A" + "B"` ask once,
  `Delete Version|Recovery`). **Pins** in `library/shows/.pins.json` (path → note).
- **Cleanup** (`plan_cleanup`, pure + unit-tested; runs after every save / copy): per group (a
  show's versions, a show's copies) all of the last hour, 1 per hour today, 1 per day for 7 days,
  1 per week up to `RecoveryKeepDays` (30); signal / merge / format copies ≥ 30 days; pinned never;
  then the cap `RecoveryCap` (500 MB) over all of them, oldest plain first, lost work last; the
  trash empties after 30 days. Both are **station settings** (`Set Station RecoveryKeepDays|
  RecoveryCap`, section Cleanup, Setup). **Deviation (safety):** a file whose manifest doesn't say
  why it was written (older formats, copies made by hand such as the user's
  `demo_user-save-before-*`) is never removed. `Delete Recovery /Cleanup` asks with what goes.
- **Manifest facts (format 15)**: `saved_at`, `reason` (quick / save / saveas / new / restored /
  format / timed / before-load / shutdown / signal / merge words), `station`, `user`, `note`
  (`SaveShow /Note "x"`), `description`, `counts`, the per-kind GUID → `[hash, label]` `index`
  (Fixtures, Patch = a fixture's addressing, Groups, Presets, Sequences, Cues, Macros, Lines,
  Users without their programmer / selection / usage, Profiles; plus FixtureTypes, Networks,
  Stations for the pick lists) and the last 20 `history` rows (each with its `changes` per kind
  `[added, changed, removed]` against the save before and the objects' names). The loaded file's
  history and index ride on `AppState::saved` (the next save's Changes; Save As keeps the
  history — a copied file still shows where it came from). Manifests are read alone and cached
  (`backups::manifest`, path + size + mtime).
- **Startup recovery**: the station writes `<station dir>/running` at start and removes it after
  a clean shutdown; after an unclean stop, when a recovery copy of the startup show is newer than
  its file, ONE station ask (`Audience::Station`): Open copy (`LoadShow /Recovery "<copy>"`) ·
  Show Recovery… · Open saved show. Never loads by itself; the marker is rewritten whatever is
  chosen, so it never asks twice.
- **Commands**: `LoadShow "x" /Version <stamp>` and `LoadShow /Recovery "<copy>"` (the copy names
  its show) open a copy as **unsaved changes of its show** (the next save is "Restored" and keeps
  the file as a version); `Import Show "x" [/Version|/Recovery …] <parts | objects>` (the Show
  object's Import verb, Setup, reach Drive, one Oops step, `core::show_import`): parts Data
  (datapools, priority orders, sequences, effects, timings, curves, images) · Patch · Protocols ·
  Users (users + profiles) · Stations; single objects by the file's numbers or names (`Sequence 1
  + 3`, `DataPool 2 Group 1`, `Preset 4.1`, `User "LD"`, `Network "Main"`, …) — merged **by GUID**
  (same GUID updated in place, a new one added at its number or the next free; a user, profile,
  network or station row whose name another object holds here is skipped and named; broken links
  kept and counted by the reference check; refused while Patch has a draft). New objects
  **`Version`** (List = the history, also of a show not loaded; Info; Set Pinned / Note; Delete →
  trash) and **`Recovery`** (List, Info, Set, Delete, `/Cleanup`), both station-scoped, reach
  Drive; reading is Guest, pins Program, deletes Setup.
- **Menu (`MenuDef` "Backup", `legacy` gone everywhere)**: tabs **Library** (first; the loaded show
  selected when it opens) · **Show · History** · **Recovery**, each a `<luma-grid>` on the `shows`
  feed (now `backups::library_view`: current + facts, shows with format tone / versions / saved by,
  `history` of the feed param `history` — the loaded show unless Library › History… picked another
  —, recovery rows with what the rules keep them for, usage of the cap, free disk via `df`, warn
  < 15 GB / alert < 5 GB). The fixed show actions with one icon and tone each (Load show… gold
  primary, Save show + Save as… green, New show… neutral, Info blue, History… violet) on Library
  and History; a phone puts Load · Save · New first. Library: New = `NewShow {value}` (Admin,
  `Grid.new_rights`), Delete = `Delete Show`, Import = the Load show dialog. History: List |
  Matrix (`Grid.matrix`: a frozen Saved column and a column per kind with `+a ~c −r` and a tone; a
  tapped cell's objects in the last line), Note / Pinned edit in place (`Set Version "show"
  <stamp> …`, `Grid.owner` puts the show into every line), the current file is locked, Open
  version, Take parts…. Recovery: Open copy, Take parts…, Pin, Clean up now, Cleanup rules. Chips:
  the show and unsaved / saved, the next recovery copy, free disk. The History caption is the
  board's banner (saved by / on, format, counts, the recovery interval and next copy, use).
- **Load show dialog** (the kit's dialog — no new element): the file's facts (saved, reason /
  note, size + format, description; a refused format greys Load and says why), the five parts as
  All / Pick / None selectors with their counts (from `GET /api/show-parts`, a one-shot load of
  the manifest), Pick… → which part → a toggle per object. Everything All = `LoadShow "x"` (the
  station's unsaved-changes ask follows); less = ONE `Import Show "x" <parts> <objects>` line.
- **Kit**: setup.js gains the `matrix` view (List | Matrix switch beside Grid | Cards), `owner`
  (`{object}` = `Version "demo"`, `{owner}`), a host selection (`extra.select`, once), the
  cancelable `slab-act` and the `action` events to the host (`onAction` / `onSlab`); the touch
  switch buttons are ≥ 44 px wide.
- **Numbers**: show format **15** (demo regenerated); `CLIENT_PROTOCOL` stays 12 and
  `SESSION_PROTOCOL` 11 (no packet changed: the facts ride on the `shows` feed and the station
  view, served by the same station).
- **Deviations from the board / hardening**: the recovery-copy interval stays the show's
  `Autosave` (Settings › Show, as hardened) — only cleanup is a station setting; the Library has
  no "Pinned" column (a show isn't pinned; its versions are) but Saved by / Station / Versions;
  Recovery isn't grouped by show and day (a Show column instead); the Load dialog's primary button
  is always "Load" and the line preview is a note (the kit dialog doesn't redraw while ticking);
  Import / Export of `.luma` files into / out of the library is not built (the Library's Import
  takes parts of the selected show); the matrix is a second view of the same `<luma-grid>` (a
  phone scrolls its kind columns sideways beside the frozen Saved column); tab subs are fixed words.
- **Tests**: Rust `backups_tests` (8: stamps, versions + manifest facts + history + `List
  Version`, the timed copy never touches the show + the startup ask + Open copy → Restored,
  before-load copy + trash, cleanup tiers / pins / lost work / cap / foreign files, pins and
  notes, `Import Show` parts + single objects + one Oops step + refusals + the dialog's parts
  view, the feed); Playwright `kit-12d.spec.ts` + 3 tests × desktop / tablet / phone / WebKit
  (Library + Load show dialog → one Import Show line, History list + matrix cell, Recovery + the
  crash ask once); `feeds`, `kit-12c2`, `multiplatform` migrated from the Backup overlay.
  Screenshots `target/screenshots/12d-part5/`.
- **Limits** (roadmap "12d part 5 follow-ups").

- **Merge of bugfixes (decided 2026-10-13):** `origin/bugfixes` (the R1 bug-report fixes, sweep
  issues 04 / 05 / 06 / 07 / 09 / 20, quoting issue 07) merged into main after 12c2, 12d and the
  remote station edits; where the two collide, main's design stays and the branch's fix is folded in.
  - **Backups: main's Versions win.** The branch's `library/shows/backups/<show>.backup_<UTC>.luma`
    (newest 10) is dropped; the `backups/` folder maps to Versions (§9). Its hardening lives in
    `core::backups`: `keep_version` **copies** the replaced file by the OS through a temporary
    name (the named show stays whole; a failed save drops the copy again), never over a version
    from the same second (`-2`, `-3` …), only a missing file is "nothing to keep" (a read error
    stops the save); recovery / trash names are cut to fit 255 bytes (`sane` ≤ 100 bytes); a
    recovery copy reports why it couldn't be written (`try_recovery_copy`), so LumaSession copies and
    "Don't save" before a load refuse to replace the show (E-CMD-012) instead of losing work. Show
    names keep `drive::check_file_name`.
  - **Delete asks: one ask, in dispatch.** `dispatch::delete_first` is the only question for every
    Delete (`/NoConfirmation` / `/nc` skips it and is taken off before the handler; never inside a
    stored key / macro line; Show and ViewButton exempt). An object may shape it
    (`ObjectDef::delete_ask` → `DeleteAsk::Ask | Refuse | Handler | Plain`): the row objects
    (`set_rows::RowDelete`: Output, Input, Network, Station, User, UserProfile) name the count and
    the rows — "Delete 3 lines (Output 1, 4, 7)?" (title "Delete 3 Outputs") — refuse the rows that
    can't go before asking (a network in use, this station, …; the rest are asked about with a
    "Not deleted" note), and the button runs the rows as **one statement** (`Delete Output 1 + 4 + 7
    /NoConfirmation`, one Oops step); rows follow the one range rule (`4 Thru`, `Thru 5`, missing
    numbers skipped). `Delete Version|Recovery` (and `/Cleanup`) ask there too. A single User /
    UserProfile keeps its handler's richer question (what goes with the user; `Handler`: dispatch
    asks nothing and leaves the flag). `set_rows::delete_rows` never asks: a grid delete shows one
    pop-up (test `a_grid_delete_asks_once_naming_its_rows`). Other station asks (`Set Station Online
    Off`, `HoldOutput On`, `Leave` / `Claim LumaSession`) stay as 12d built them.
  - **Start markers: both, woven.** The branch's `.starting` marker (R1-17, E-SHOW-006) skips the
    startup show once after a start that died while loading it; main's `running` marker offers the
    recovery ask after an unclean stop. A skipped startup show counts as an unclean stop
    (`AppState::skipped_startup_show`): the show is skipped first, then the recovery ask offers its
    newer copy (the file's own `saved_at` from its manifest; "Not now" instead of "Open saved show").
    `library/.starting` and `library/stations/*/.starting` are gitignored next to `running`.
  - **Protocol numbers:** both sides changed the wire, bumped once past main: `SESSION_PROTOCOL`
    **12**, `CLIENT_PROTOCOL` **13**. `FORMAT_VERSION` stays **15** (ShowData and its
    serialization are unchanged against main; the demo show loads and matches).
  - **Busy coverage** (R1-67's waits on main's new state-thread paths): a feed's menu view built on
    demand waits like an HTTP read (`call_within` 10 s; busy = no view yet, the state thread
    publishes it later); a command line on the UDP command port — a client's or another station's
    remote edit — waits at most 10 s, then answers E-CMD-017 and never runs late (station-to-station
    requests still wait); the sender of a remote edit shows a busy answer as E-CMD-017, not
    "refused"; the shell's suggestions wait 1 s at most; behind queued keys the newest waits and is answered once they ran (2026-10-19: skipping it lost the list after a taken suggestion — `LineEdit` and the next `Suggest` land in one read, and no keystroke follows to ask again), a taken
    suggestion (`LineEdit`) waits in the key queue, a resize at most 30 s; the recovery ask's answer
    and every menu / setup line ride the shell's queue (keys, lines, taps, answers in order, never
    dropped); the live `station` feed shows `busy` when the watchdog's episode starts and ends
    (`StateHandle::on_busy_change` moves the station counter).
  - **Layout PUT / Store View:** the branch's removal of `UiEvent::StoreView` and `POST
    /api/views/store` stands (Store View / Rename are station asks); main's 12d frontend and kit call
    neither; the page keeps main's kit UI with the branch's behaviour changes only (no client-only
    Delete confirm, the pool title escaped, owner-first pool lines).
  - **Syntax rules everywhere:** 12d's handlers follow the branch's quoting (quoted words keep their
    quotes until `tokens::unquote`: `Delete User "A" + "B C" /NoConfirmation`, `Set Station "Tower"
    Location "Side Wing"`), the one range rule (row deletes), and statements as own steps (the grid
    delete's answer is one statement).
  - **Rights wording:** where the branch coded an Admin refusal by hand (E-CLAIM-001 for Unclaim of
    another's claim), main's declared gate stays (`rights::require`, E-RIGHTS-001 naming both
    levels).

**12e hardening (2026-10-13)** — "lets do 12e analyze it and all md decisions old and new and
harden". Research: every old and new decision on touch layout (§22 pools, §29 UI Oops, §30 12a1 →
12d, the R1 bug report, the 2026-10-12 PoolHold / Phone / Tablet boards), the code as it stands, and
a reference console (corpus: manual + layout files + systemtests) with other consoles and touch engineering
(Android / iOS gesture constants, VS Code sash, react-resizable-panels, the WAI-ARIA window
splitter, dnd-kit, SortableJS, readline). Findings that change earlier research: the reference pool
actions open on a **swipe out**, not a hold (3×3 — Store / Edit /
Assign · Label · Delete / Move / Copy, Edit *or* EditSetting per pool type); it **stores on a hold
on an empty pool cell** (a systemtest); it orders group members in the
Selection Grid's Setup mode (+ Linearize / Remove Gaps tools); Dockview's long-press drag is
unverified (`online-touch.md` §3). Already built and kept: split drag on the pointer core (12a1),
hold on a tab / header → the window menu (12a1, kit `menu.js`), the phone desk in miniature with
hold = full screen (12c2). Decided:

- **Command line on a narrow bar — fixed on the station** (one fix for the web bar and the TUI):
  the prompt shrinks in steps (`[user@host] scope>` → `scope>` → `>`; the `>` is never cut), the
  line scrolls sideways to keep the cursor in view with a margin of about 4 cells (readline's
  `horizontal-scroll-mode`, on by itself for a one-row terminal), `‹` / `›` mark clipped edges;
  the "Terminal too small" guard never blanks an input-only view; the two prompt builders
  (`tui_draw.rs`, `tui_commands.rs`) become one. TestBackend tests at 20–30 columns.
- **One hold, one owner.** Every hold — pane, tab, header, pool cell, key, grid lasso, row — uses
  the profile's `HoldTime` (default 500 ms, 8 px slop, ring after 140 ms) through
  `kit/pointer.js`; minipane's own 400 ms, the grid's lasso timers, the UI-key and pool timers go.
  The **innermost target big enough to tap owns the press** (12c2's size rule): a press arms one
  hold, never two (fixes a tab inside a miniature pane opening both the full screen and the window
  menu). `hold()` also ends on `lostpointercapture`. Tidy: no hand-made hold timer, no `draggable`
  / HTML5 drag events, no window-level pointer listeners outside `kit/`.
- **Tab hold → window menu** stays as built; 12e adds touch tests (`tap()` / `hold()` helpers) in
  the tablet and phone projects.
- **`<luma-pool>` — the pool as a kit element in 12e** (R1 Q34 c): virtual cells, updated in place
  cell by cell (never re-rendered under a finger, R1-60), keeps its scroll, auto-fill capped; one
  element for the cells, the hold selector, the action row and Move / Copy target picks; every
  gesture a command line (owner first, `poolObject`). **Two designs:** the desk cell as today, and
  a **compact phone design** — a tight matrix, almost no padding or gap, smaller cells, the number
  and name only (the user: "a smaller matrix with almost no padding on mobile"). Both on the
  mockup before go. `buildPoolPanel`, `makePoolBtn`, `poolPointer`, `.pool-selector` and the ghost
  are replaced (the e2e selectors move deliberately: `pool-window.spec.ts` runs in the tablet and
  phone projects too).
- **Pool hold — decided on the artifact** (the user tests it first). Candidates: (A) the locked
  PoolHold board as is (hold → full-screen selector of every cell, action row Edit · Label · Store
  · Delete · Move · Copy · Settings, command-first: action, then object; Move / Copy then a target
  tap); (B) A + a hold on an **empty** cell stores there at once; (C) a per-object sheet
  (object first, the row at the bottom). Settings needs `EditSetting <obj> n` for pool objects
  (missing for Group today) or opens the window's settings — decided with the board. Label sends
  `Label <obj> n` with no name (the station asks, merge of bugfixes); Delete goes through
  `dispatch::delete_first`.
- **Split handle — decided on the artifact.** Candidates: (A) the hairline with a 44 px hit band on
  coarse pointers; (B) A + a grip pill on touch only; both with double-tap = 50 / 50, arrow keys
  (`role=separator`, `aria-valuenow`), one Oops step on release (still the layout PUT with its
  revision — the accepted exception, §30 "12c built"); the window type's minimum size clamps.
- **Group editor:** the member list becomes a `<luma-grid>` with its actions in the slab; each row
  has a 44 px **grip** that drags on the pointer core (`touch-action: none` on the grip only, rows
  scroll freely; the pattern of `settings.js`), plus **↑ / ↓** and the established tools (Linearize by
  number / left to right / top to bottom, Remove gaps), each one command line (`Set Group g Fixture
  f Order n`; a tool is one statement, one Oops step). The grid canvas moves cells on
  `pointer.drag` (capture, cancel). On a phone the list stacks above the grid.
- **Phone layout** (12c2's miniature desk stays): at ≤ 600 px the **left sidebar folds** into the
  bar's `⋯`; a full-screen pane **swipes left / right** to the next / previous pane (the same as
  `<luma-maxbar>`'s ‹ ›, `Set Self Maximized` when `HoldFollows`); **no landscape layout** for now.
  Shown on a phone board before go.
- **Window scroll (R1 Q31):** the page reports the first visible row as a **quiet `scroll`
  message** on the page's shell socket `/ws/shell` (built: `ShellClientPacket::Scroll`, beside
  `focus` / `live_screen`; `/ws/feeds` only takes subscriptions) when a scroll ends — station-local, no echo, no Oops step, not
  replicated (like a tab switch; the deliberate exception to "every gesture is a command line").
- **Mockups:** the 12e artifact (pool hold A / B / C to try, `<luma-pool>` desk + compact phone,
  split handle A / B to drag, phone sidebar fold + swipe, the group editor, the narrow command line)
  — approved before go; then copied to `docs/dev/ui-kit/mockups/2026-10-13-12e/`.

**12e artifact round 1 (2026-10-13)** — the user tried the first 12e artifact
(https://claude.ai/artifact/KVgctMjNoZbRuuM41Hn8GP, private). Decided:

- **Pool hold = C, the object sheet.** Hold on an object opens one sheet for that object (preview:
  number, name, facts, holder) with the action row at the bottom (Edit · Label · Store · Delete ·
  Move · Copy · Settings); Move / Copy close the sheet and arm a target pick in the pool (a banner
  "Move Group 3: tap the target" with ×; the next tap sends `Move Group 3 At 12`). The locked
  board's full-screen selector (A) and B (hold on an empty cell stores) are not taken; §22's
  drag-through selector goes with A.
- **Split handle = B, but the hit area is only around the grip pill** — the hairline itself is not
  a touch target on coarse pointers (no 44 px band along the whole divider); the pill (≥ 44 px
  hit area) is what you grab. Mouse keeps the thin band along the divider.
- **Phone sidebar folded into `⋯`** (as the board).
- **Miniatures carry content:** a miniature pane shows miniature text in lists, and a miniature
  `<luma-pool>` shows the object's text and its group colour; slot number and full text don't
  matter on a phone until the pane goes full screen, which shows the complete pool.
- **Group editor list, grip, ↑ ↓: good.** The tool row (By number · Left → right · Top → bottom ·
  Remove gaps) is **not** — many more tools come later; the tools need another home (asked and
  researched in round 2).

**12e round 2 (2026-10-13)** — research: a tool home for editors (a reference selection grid toolbar
16 tools in 6 groups, each `Grid "<Tool>" ["<Arg>"]`; its align bar;
Pawterns sections; Onyx Grouping Tools window; Eos array pop-up; Figma Actions, Blender F3,
Procreate / Affinity strips) and a live console probe (pool-object properties, Appearance, small
windows). Findings: the reference has no "NodeMeta" — the shared layer is `Object` (tier 1: Lock, No, Name,
Note, …) plus an "identified" block on 97 classes (GUID, Scribble, Appearance, Note, Tags,
PreviewCopy); colour is never on the object but an `Appearance` link (BackRGBA, Image, ImageMode);
each pool type has a theme colour; sheets never drop columns by themselves (column sets, font
size, line height, adjust columns, scroll); no "mini" window variant exists. Decided:

- **Colour = `NodeMeta` only, forever.** Every object's `NodeMeta.color` and `icon` (already in the
  tree and drawn by the pool cell) become declared props on every pool object (`Set Group 3 Color
  …` with the palette pick list, `Icon` with the icon pick list); no Appearance pool, ever. An
  object without a colour draws in its **pool type's theme colour**.
- **Miniature `<luma-pool>`:** colour only (NodeMeta colour, else the pool type's), dense matrices
  of 30 × 30 and more; the name appears by itself once a cell is big enough to read (about 40 px),
  so a larger pane shows text without a setting; numbers only full screen.
- **Miniature lists: a declared mini view.** Each window type declares in the registry what it
  shows in miniature (sequence: cue number + name + active mark; fixture sheet: fixture + dimmer;
  …); the window settings can pick another column set for miniature; full screen shows every
  column. Never a scaled copy of every column.
- **Tools: one form, `Grid <owner> "<Tool>" ["<Arg>"]`** (owner first: `Grid Group 3
  "Linearize" "LeftToRight"`), one statement, one Oops step, computed on the station; tool names
  are quoted values (no new keywords). ↑ ↓ and drag keep `Set Group g Fixture f Order n`.
- **Tools are safe from a mis-tap:** a **Tools** button opens a list; a tap on a one-shot tool
  (Remove gaps) runs it and closes the list, × closes without running; a tool with a parameter
  always asks first (its options + Run). Where the button and list live is tried on the round-2
  artifact (Tools… sheet with categories / search / pins vs a tool rail vs a context bar).
- **Swipe end** (wrap vs stop) is tried on the round-2 artifact.

**12e artifact round 2 (2026-10-13)** — the user tried round 2 (same link, version 2). Decided:

- **Pool approved:** the object sheet with NodeMeta / pool-type colours, the phone full-screen
  matrix, the 30 × 30 miniature with names from 40 px.
- **Split handle approved:** the grip pill is the only touch target (44 × 64), the mouse drags the
  thin line.
- **Phone: good overall**, but the miniature lists need **more columns** and a real **table / grid
  look** (header row, cell lines, as `<luma-grid>`) for the sheet and the sequence list — in
  miniature and full screen. Shown again on round 3.
- **Tools: the Tools sheet — reshaped.** No drill-down on a desk: **Tools opens a bottom toolbar
  docked in the editor**, tools **grouped** side by side (Order · Gaps · Align · Transform · Scale ·
  Pawterns, a label over each group), so the set grows sideways; on a **phone** the tools come up
  **at the bottom**, in thumb reach (a low bottom sheet of the same groups). A tap on a one-shot
  tool runs it, × closes; a tool with a parameter asks first (unchanged).

**12e artifact round 3 (2026-10-13)** — https://claude.ai/artifact/GcXNCnqoL8LMe9dDzptPm6
(private). Decided:

- **Group editor tools approved:** the desk's grouped bottom toolbar (Order · Gaps · Align ·
  Transform · Scale · Arrange · Pawterns, no drill-down, grows sideways) and the phone's low sheet
  at the bottom; **every tool needs an icon** (Lucide, declared with the tool in the registry).
- **Miniature lists:** the table look is what the user wants, but it looked cheap as a small
  table inside a pane — a miniature list is **never a small list floating in a pane**, it is the
  **whole list designed in miniature**, filling the pane. Suggestions asked for (round 4).

**12e round 4 (2026-10-13)** — the miniature `<luma-grid>` bench
(https://claude.ai/artifact/WGbWBfSfndWwRw9iqmnXP5, private: A scaled, B dense text, C visual
summary, D hybrid). Decided: **the style is per window type** — the **fixture sheet** miniature is
**D, hybrid** (key columns text; levels as bars, colour as a swatch, …), the **sequence**
miniature is **B, dense text** (all text at mini density). Each window type declares its mini
style and each column its mini renderer (`text | bar | level | swatch | icon`). Refined on round 5.

**12e round 5 (2026-10-13)** — https://claude.ai/artifact/1FTf3WsyH1vCjaSt8r7KVi (private).
**Approved:** the sequence miniature (dense text: done cues dim, running cue gold with its fade
progress along the bottom, next cue marked at the left edge, times monospace with faint zeros)
and the fixture sheet miniature (hybrid: Dim a level bar with the value on top, programmer colour
when the value is in the user's programmer, Color a swatch, selected fixtures a blue edge). Both:
columns keep a declared minimum width (headers never squeezed), a narrow pane shows the first
columns and fades at the right edge, the pane fills edge to edge and keeps the live row in view.

**12e round 5 questions (2026-10-13), one at a time:**

- **Swipe past the last pane: stop** — it springs back ("Last pane"), ‹ / › grey at the ends.
- **The desk tool toolbar stays open** after a tool (Tools or × closes it); the phone's low sheet
  closes after each tool.
- **Tool icons approved** (Lucide, declared with each tool, one per choice for a tool that asks:
  Linearize `list-ordered` (choices `hash` / `arrow-right` / `arrow-down`), Reverse
  `arrow-down-up`, Shuffle `shuffle`, Remove gaps `fold-horizontal`, Remove offset `move-up-left`,
  Align `align-horizontal-justify-start|center|end`, Distribute `align-horizontal-space-around`,
  Transpose `move-diagonal`, Rotate `rotate-cw|ccw`, Flip `flip-horizontal-2|vertical-2`, Multiply
  `maximize-2`, Divide `minimize-2`, Symmetrical `columns-2`, Line `minus`, Circle
  `circle-dashed`, Grid `grid-3x3`, Pawtern positions `grid-2x2-check`); a tool button is a fixed
  grid — one width, icon box 24 px, label line, note line (`asks` / `planned`) — so icons and
  labels line up across the toolbar (the user: "not aligned properly" on the first try).
- **A tool that opens a menu** (asks for a choice or a number first) shows a **yellow `===` mark**
  in its note line instead of the word "asks" (the same mark wherever a button spawns a menu);
  planned tools keep "planned".

**12e design locked (2026-10-13)** — "lock it", in `docs/dev/ui-kit/mockups/2026-10-13-12e/`
(rounds 1–5 as plain `.html` boards + PNGs). The narrow command line approved as on the round-1
board. One addition with the lock:

- **The list button is a standard kit element:** a button that opens a list or menu (a tool that
  asks, a pick, any "choose from" button) carries the **yellow `===` mark** in its note line — the
  kit's one way to say "this opens a list"; drawn by the kit (a `menu` / `list` variant of the
  button and the tool button), shown in the Showroom, used everywhere such a button exists (never a
  hand-made marker or the word "asks").

12e waits for go. Build order when it comes: the station's narrow line; the one hold; `<luma-pool>`
(+ NodeMeta `Color` / `Icon` props, pool-type colours, `EditSetting` for pool objects, the object
sheet, miniature); the split grip pill; the tool registry (`ToolDef`, `Grid <owner> "<Tool>"`), the
grouped toolbar / low sheet and the list-button mark; the group editor on `<luma-grid>` + slab;
the declared mini views (`<luma-grid>` mini density, per-column mini renderers); the phone desk
(folded sidebar, swipe); the quiet `scroll` message; tidy rules; e2e on tablet + phone.

**12e part 1 built (2026-10-13)** — the station's narrow line, one hold, the split pill, the scroll
report:

- **Narrow command line** (`terminal/repl`): one prompt builder `fit_prompt` / `prompt_for`
  (`[user@host] scope> ` → `scope> ` → `‹end> ` → `> ` — `‹` like the line's own cut mark; a `…` drew blank in the web terminal's font, at least 12 columns or half the line left
  for typing); `input_offset` scrolls the line to keep the cursor 4 columns from the right edge
  (readline's horizontal-scroll-mode); `window_spans` draws the window with dim `‹` / `›`; a
  view of only the command line is never "Terminal too small". The TUI and the web bar share it.
- **One hold, one owner** (`kit/pointer.js`): `claim(e)` marks a press owned — the innermost hold
  takes it, holds further up leave it; inside a miniature only a `.lc-tap-ok` target keeps its own
  hold (in practice a miniature's inner elements take no pointer at all, so the pane gets every
  press). The miniature's 400 ms and the grid lasso's 450 ms cap are gone — every hold is the
  profile's HoldTime; `hold()` ends on `lostpointercapture` too; UI keys claim their press. The
  old pool's own timer goes with `<luma-pool>` (part 2); the tidy rule against hand-made hold
  timers / HTML5 drag / window-level pointer listeners outside `kit/` comes when the last ones go
  (part 3).
- **Split handle = a kit element** (`kit/split.js`, `LumaKit.split.handle`, in the Showroom):
  4 px line, a mouse grabs a ±5 px band, a finger only the grip pill (44 × 64 hit area; the line
  takes no touch); double-tap = 50 / 50; `role="separator"`, arrows 5 %, Home / End, Enter; panes
  never below 120 px; drag / reset / key = one layout step (`saveLayout`). The page's `.dhandle`
  and `attachDrag` are gone.
- **Window scroll** (R1 Q31): `ShellClientPacket::Scroll { window, row }` on `/ws/shell`
  (`{"type":"scroll", window, row}`), kept by `ScreenStore::set_scroll` quietly (no revision, no
  listener — a scroll never redraws a page or conflicts with a layout PUT); the page's
  `trackScroll(win, scroller, pitch, prepare)` reports the first visible row 250 ms after scrolling
  stops and puts it back once when the window is built; used by the pool window (which now keeps
  its scroll across a redraw). `CLIENT_PROTOCOL` **14**.
- Tests: repl `the_prompt_shrinks_on_a_narrow_line`, `the_line_scrolls_to_keep_the_cursor_in_view`,
  `a_narrow_bar_shows_the_prompt_and_the_end_of_a_long_line`; live_screens
  `a_scroll_report_is_kept_quietly`; e2e `kit-12e.spec.ts` (held tab in a miniature, the phone
  bar, pool scroll kept), `kit.spec.ts` split drag on the pill + double-tap reset.

**12e part 2 built (2026-10-14)** — `<luma-pool>`, the object sheet, NodeMeta colour:

- **NodeMeta look on every object** (`core::props`): `META` grows Color (`PropKind::Rgba`, written
  `#rrggbb`, kept as RGBA bytes, `none` = the pool type's colour) and Icon (`PropKind::Icon`, a
  standard icon's name); `props::shown` reads them back for `Set` replies and `EditSetting`.
  Groups get a writable tree (`group_write`; a prop write now carries the object's datapool —
  `props::Apply` takes `Option<u32>`), `Set Group 3 Color|Icon|Note|Locked|Hidden` go to the
  generic props from `set_group`; `EditSetting Group 3` lists them. One Oops step each (the change
  path). Test `a_group_takes_a_colour_and_an_icon`.
- **`<luma-pool>`** (`kit/pool.js`, in the Showroom): only the rows in view exist (cells placed in
  a sizer as tall as every row), a cell is repainted only when what it shows changed (R1-60),
  new data keeps every drawn cell, the scroll stays (R1 Q31: `onScrollRow` + `scrollToRow`, a saved
  row waits until the data is there). Designs by place: desk (number, name, lock, link, badges, fit
  edge, bar; the window's columns / rows / size), phone (≤ 600 px on touch: 1 px matrix, cells
  ≥ 62 px, number + name), mini (inside a miniature: colour only, the name once a cell is drawn
  ≥ 40 px). Colour: the object's `--lc-obj` (NodeMeta) or `--lc-pool-<kind>` (tokens:
  `--lc-pool-groups`, `--lc-pool-default`). Gestures: tap → the window's tap action; hold or
  right-click on an object → the page's sheet (an empty slot: nothing — B not taken); a mouse drag
  → `Move|Copy <obj> n At m` (Ctrl / ⌘; macOS's Ctrl-click context menu is ignored there); one
  finger scrolls; `armPick(text, src, onTarget)` puts the Move / Copy banner above the cells
  (never over a target) and the next tap is the target. Keyboard cursor: `step(cur, dir)`,
  `reveal(n)`.
- **The object sheet** (`kit/sheet.js`, `openObjectSheet`, `POOL_ACTIONS`): bottom-anchored,
  ≤ 640 px, preview (number in its colour, name, facts, holder) and the action row Edit · Label ·
  Store · Delete · Move · Copy · Settings (60 px, icons; Delete in the delete colour); each action
  the page's command line — `Edit` opens the editor, `Label <obj> n` / `Store <obj> n` /
  `Delete <obj> n` (the station asks), Move / Copy arm the pool's pick, `EditSetting <obj> n` (off
  for a window pinned to another DataPool). Tidy: `sheet.js` is an action file like the slab.
- **The page:** `buildPoolPanel` is a `<luma-pool>` in the pool window; `makePoolBtn`,
  `poolPointer`, `poolCtx`, the full-screen selector and its CSS are gone; e2e
  `pool-window.spec.ts` → `kit-pool.spec.ts` (all four projects: sheet + Move pick, colour,
  right-click → Edit; the old editor's grid drag stays a desk check until part 3).
- The narrow prompt's cut mark is `‹` (a `…` drew blank in the web terminal's font).

**12e part 3 built (2026-10-14)** — the tools, the list button, the group editor:

- **Tools** (`core::tools`, `TOOLS`): the selection grid tools as one registry — id, label,
  Lucide icon, toolbar group (Order · Gaps · Align · Transform · Scale · Arrange · Pawterns),
  question (`ToolAsk::Choice` with an icon per value, `ToolAsk::Number`), status, transform.
  Ready: Linearize Number / LeftToRight / TopToBottom, RemoveGaps, RemoveOffset, Align Left /
  Center / Right (each row to the left edge, the middle or the right edge of the widest row),
  Transpose, Rotate Right / Left, Flip X / Y, Multiply n, Divide n (2–16). Planned (listed,
  greyed, refuse E-CMD-003): Reverse, Shuffle, Distribute, Symmetrical, Line, Circle, Grid,
  Pawtern positions. `Grid [DataPool d] Group n "<Tool>" ["<Arg>"]` runs one on a group's members
  (order + spots) — one statement, one Oops step; the grid cursor forms stay; `Grid` with an
  object needs Program (`arg_rights`). `GET /api/tool-types` (open, static). Icons added to the
  sprite (lucide-static 1.55.0 re-pack, `icons.txt`). Test
  `grid_tools_change_the_group_layout_one_step_each`.
- **The list button** (`kit/button.js`): `listMark()` — the yellow `===` — and
  `<luma-button list>`; the slab draws it on an own action with `list: true`, the tool buttons on
  every tool that asks. Applying it to the existing pick buttons (settings links, window-bar
  choices) is a follow-up.
- **`kit/tools.js`**: `toolbar(spec)` — the desk's grouped bottom toolbar (each group labelled,
  side by side, scrolls sideways, × closes, stays open after a tool); `openToolSheet(spec)` — the
  phone's low sheet (each group one row; closes after a tool); `ask(tool)` — the question (choices
  with icons or − n +, the line it sends, Cancel / Run). Tool buttons are one fixed grid (96 × 72:
  icon 24 · label · note). Above the editor's layer.
- **`<luma-grid reorder>`**: a 44 px grip per row (the pointer core's drag; the rows still scroll
  with one finger) and ↑ ↓; one `reorder { key, from, to }` event per move.
- **The group editor** (`kit/groupeditor.js`, `openGroupEditor`): the member list on
  `<luma-grid reorder>` (`Set Group g Fixture f Order n`), `<luma-slab>` (Remove = the edit slab's
  Delete → `Remove Fixture …`, Oops, Rotate 90°, Mirror X / Y, Tools with `===`; a narrow slab
  shows only Delete and Oops of the standard six — slab `phone_edit`), the canvas (members on
  their spots, drag on the pointer core → `… X x Y y`; it never redraws under a finger) and the
  tools (desk: the docked toolbar; phone ≤ 600 px: the list above the canvas, Remove · Tools at
  the bottom, the low sheet). The page's hand-made editor (HTML5 drag, window listeners, `.pe-*`)
  is gone.
- **Tidy** `frontend_gestures_only_in_the_kit`: outside `kit/` no HTML5 drag, no window- /
  document-level `pointermove`, no hand-made hold ring (`tidy: hold-ok (<reason>)` — the UI keys'
  own Press / Release / Hold gesture carries one).
- e2e `kit-pool.spec.ts` covers the editor on all four projects (↓ reorder, the toolbar's
  Linearize question, a canvas drag on the desk; the phone's low sheet and its closing).

**12e part 4 built (2026-10-14)** — the miniature list, the phone desk:

- **`<luma-grid view="mini">`** (`renderMini`): the whole list drawn small to fill its pane —
  the window type's `mini_columns` (else the shown ones), per-column renderers `c.mini`: text
  (default; a frozen column is the key, in full text), `num` (right, monospace, a zero faint),
  `level` (a bar with the value on top; `c.tone` = a row field that paints it in the programmer
  colour), `swatch`, `bar` (`c.max`), `icon` (`c.icons`). A row's `mini` state: `run` (gold,
  `progress` along the bottom), `next` (an edge mark), `done` (dimmed), `sel` (blue edge); the
  live row stays in view; columns keep `c.mini_min`, past the pane's edge they fade. Drawn at
  `--lc-fs-small` with 16 px rows and scaled to `spec.mini_row` (like every miniature — no raw
  font size). In the Showroom: the sequence (dense text) and the fixture sheet (hybrid) as
  approved in round 5. **No window uses it yet:** there is no sequence or fixture-sheet window
  type today (window types: terminal, groups, dmx, sysmon, realtime, showroom, empty); each
  declares its mini view when it is built (12f's Sequence pool / sheet, the fixture sheet).
- **Phone desk:** at ≤ 600 px the left sidebar folds too (with the view bar); `⋯` lists its keys
  first (Shutdown, LOCK, BLND / HI / FRZ / SOLO lit as on the bar, Meow, Menu, AT — each runs as
  its button) and the stored views, then the status. A full-screen pane swipes (kit
  `pointer.swipe`: touch / pen, sideways past the slop follows the finger, past 20 % of the width
  = next / previous, else springs back; vertical stays the content's scroll); ‹ › and the swipe
  **stop at the ends** ("Last pane", ‹ grey on the first, › on the last — `<luma-maxbar>` takes
  `index`).
- e2e `kit-12e.spec.ts`: the folded sidebar in `⋯`, the held pane's ‹ at the first pane and a
  touch swipe to the next; the Showroom's miniature lists.

**12f hardening (2026-10-14, in progress)** — "harden 12f, read all related decisions and docs".
Research: every decision on faders, encoders, sheets, Oops and sequences; the code (no fader /
encoder element yet; masters refuse relative input; the grand master files an Oops step; executors,
pages and Flash planned only; the sequence fader / rate stored but unused; sequences have no
NodeMeta and live show-wide); a reference console (manual + a live read of its masters), Eos, MagicQ,
Onyx, Hog, TouchOSC, Logic, Photoshop. The reference masters: 1 Selected (Master,
XFade, XFadeA / B, Temp, Rate, Speed, Highlight, Lowlight, Solo, Time — follows the selected
executor) · 2 Grand (Master, World, Highlight, Lowlight, Solo, Rate, ProgramTime / XFade,
ExecutorTime / XFade, Blind) · 3 Speed (16) · 4 Playback (50) · 5 Timing (50), each a normed value
0–100 (Speed with a scale). The reference Oops menu: newest at the bottom, a tap selects it and every newer
step, the button reads "Oops n Actions". Decided so far:

- **Scope: masters now, executors later.** 12f's faders drive what exists — the grand master,
  group masters, Highlight / Solo masters. Executors, pages, Flash, fader functions (X-fade, Temp,
  Rate, Speed), speed masters come with "Playback depth" as their own step.
- **Masters are never undone** (§29): no master move files an Oops step (a test checks it); the
  grand master's level leaves the undoable `ShowData.Playback`.
- **Masters live in `ShowData.Playbacks.Masters`, in master groups** — saved, replicated, never undone:
  1 Selected (planned: follows the selected sequence) · 2 Grand (2.1 Master, 2.2 Highlight,
  2.3 Lowlight planned, 2.4 Solo, 2.5 Blind planned) · 3 Speed (planned, §28) · 4 Playback
  (planned, executors) · 5 Timing (planned) · Groups (the group masters move in, by group GUID;
  `Group 3 Master At …` stays). Addressed `Master 2.1` (the established grand master address; our numbering
  inside Grand is our own, as listed), names too (`Master "Grand"`). Your own Highlight / Solo
  masters stay per user in `Runtime.Users.<u>.Modes` (they scale your mode; the Grand ones cap
  everybody). Masters take relative input (`Master 2.1 At + 10`).
- **The gesture channel** (not a `/Quiet` option — anyone could hide any line with it): continuous
  controls send `gesture { id, line, phase: move | end }` on the page's shell socket. The line is a
  real command (`Master 2.1 At 63.5`) — same rights gate, dispatch and replication; the channel
  makes it quiet: no echo, no command history, no Usage; while the station is busy only the latest
  move of a gesture is kept (stale moves dropped, never the end); the System Monitor and journal get
  ONE entry per gesture at its end ("Fader Master 2.1: 100 → 63.5 · Admin") under a hideable
  "Faders" filter. Only kit controls send it; the same line typed stays a normal command. OSC /
  MIDI faders take the same channel later.
- **Fader touch:** no double-tap on a fader; a tap never jumps; hold = "show me more" (the master's
  sheet). The rest is tried on the 12f artifact.
- **Oops window: select, then "Oops n".** Newest at the bottom; a tap selects that entry and every
  newer one, the button becomes "Oops 3 steps", a second tap runs it (OopsConfirmation still
  applies); undone steps stay visible dimmed below a line with "Redo n"; rows show the line, its
  category chip, station and time; filter chips = the profile's Oops filters; never a one-tap undo.
- **To try on the artifact:** the fader, the encoder (the user's console concept `inspiration/A1
  Whole Low Res.png` as the look: hardware-like faders with a yellow cap line, an encoder bar of
  "Attribute / Value" cells under each screen), the Sequence pool cells (and whether sequences
  get NodeMeta and stay show-wide).
- **The sequence window** joins 12f's sheets, from the user's references `inspiration/sequence
  idea.jpg` (a vertical cue sheet: expanding a cue shows per-feature-group timing — Fade ·
  Delay · Path, inherited values in brackets — and the fade curve) and `inspiration/Horizontal
  Sequence 1.jpg` (a timeline: cues as columns, a row per timing with its line over the seconds,
  zoom, "+ add timing" as Attribute × Type, a tap on a cue's head for its options). Both layouts on
  the round-1 artifact (https://claude.ai/artifact/Ttf9BSeXogUiw9eA4RENrP, private).

**12f artifact round 1 (2026-10-14)** — https://claude.ai/artifact/Ttf9BSeXogUiw9eA4RENrP. Decided:

- **Fader look B** (the locked Elements board's filled bar; not the concept's groove + cap).
- **Encoder B** (the encoder bar + the pop-up band fader while dragging); on a desk the **mouse
  wheel** over a cell changes the value (one resolution step a notch); on touch a **hold spawns a
  vertical slider** (the finger keeps dragging it; lift closes it). Tap = the keypad.
- **Oops window as shown** (select, then Oops n; undone steps dimmed with Redo n).
- **Sequence pool in executor style** (each cell a short cue list): a **yellow background behind
  the active cue's name**, gone (no yellow) when the sequence / executor is off; the established **blue
  active indicator** for the running sequence / cue; **no green running dot** (it looks bad).
  Checked with a live screenshot (the captures are too blurred to read the cue marks; the blue
  follows the user's console knowledge and the corpus: executor-style cells centre the active cue on
  blue).
- **Sequence window: both layouts** (vertical sheet and horizontal timeline — a window setting).
  Vertical: the curve shows **every feature group's line, dimmed, behind the active one**.
  Horizontal: **tap a timing field to edit it with a slider**.

**12f artifact round 2 (2026-10-14)** — https://claude.ai/artifact/ViEpEbuowyZJpqtWzXU3No. Decided:

- **Fader: no hold either** — no double-tap, no hold sheet; a fader only drags (relative, one
  finger each, a second finger fine). Its settings come from the window (bar / settings).
- **Encoder B with wheel + touch hold-slider: better** (kept). **Oops window: ok.**
- **Sequence pool:** executor style ok; the active cue's yellow is **too bright** — a muted gold.
- **Timing is edited with a vertical slider, everywhere** (vertical sheet and horizontal timeline):
  a **tap** on a timing field opens the vertical slider (the one of 5B, turned vertical) with
  − / + and a **Keypad** button; **tap-hold-and-drag** on a field edits it at once (the slider
  appears under the finger, lift to close) — as easy as the encoder's touch hold.
- **Vertical sheet:** switching the feature group must not make the window jump (keep the
  scroll); the curve view shows **splines, lines and curves** — each feature group's path shape
  (linear, S-curve, ease in / out, snap …), the others dimmed behind the active one; the Path is
  picked from a list.

**12f artifact round 3 (2026-10-14)** — https://claude.ai/artifact/1hTLrhzXi53s9aBMZc8b4R. Decided:

- **Faders approved** (look B, relative drag, one finger each, a second finger fine, no hold, no
  double-tap). **Oops window approved.**
- **Encoders use the same vertical pop-up slider** as the timing fields: tap = the slider panel
  (− / +, Keypad), hold-and-drag = the slider under the finger; the desk's mouse wheel stays.
- **Sequence pool: hold = Edit** (opens the sequence) — and pool windows get a **setting for what a
  hold does** (e.g. `HoldAction Sheet | Edit`), the object sheet staying the default elsewhere.
- **Timing edits show live:** while a field is dragged, the lines, curves and horizontal spans
  redraw in real time; every timing field drags the same — feature-group fields (Dimmer,
  Position …) too, not only the cue's own Fade / Delay (round 3's mockup lost them).

**12f artifact round 4 (2026-10-14)** — https://claude.ai/artifact/2PuKrUE8P1QjWbcswN78c6. Decided:

- **Encoder: no hold delay** — press and drag adjusts at once (the slider appears as the finger
  moves); a tap without moving opens the slider panel. (The encoder bar takes the finger; it does
  not scroll.)
- **Sequence window B (horizontal timeline) approved** — tap a timing field = the vertical slider
  panel, hold-and-drag = the slider under the finger, lines / spans redraw live.
- **Sequence window A:** a live edit must not glitch the graph (redraw only the curve and the
  changed value, never rebuild the sheet under the finger); **tapping a feature group's name above
  its values highlights that column** (and makes it the active curve).
- **The graph edits too** (round 5 → 6, https://claude.ai/artifact/KAEVXNG9nhxpbj5vzG883M): the
  active feature group's curve has **handles** — the end of its delay and the end of its fade —
  dragged sideways to change that time; the time scale holds still while dragging, the table's
  values follow live, one Oops step at the end.
- **The graph's scale never jumps** (round 7, https://claude.ai/artifact/TdYdAnbssZeFkM7ZJyv4Dj):
  the time axis always shows rounded headroom past the longest delay + fade (at least **2 s**,
  +20 %, rounded to a whole step), the same before and during a drag; grabbing a handle keeps
  the grab offset (no snap to the pointer); the scale grows smoothly only when a handle is dragged
  past its end — also when the dragged time is the one defining the width.
- **Sequence window A approved** (round 7).
- **Sequences live in DataPools** (like Groups): `Root.ShowData.DataPools.<n>.Sequences`, with
  NodeMeta (name, colour, icon); the Sequence pool shows the user's selected DataPool (or a GUID
  setting); `DataPool 2 Sequence 3` names another pool's; Copy / Move work across pools; the
  user's selected sequence stays a GUID link. Show format bump (forward only, no migration; the
  demo show is regenerated). Pages / executors follow the same rule later.
- **The encoder bar is a per-screen bar** along the bottom (like the command line bar): on for
  Screen 1 by default, each screen turns it on / off in the profile's screen config (a `Set` line);
  on a phone it folds away like the sidebar.
- **Encoder bar research (a reference console's encoder bar,
  its layout files and manual):** a 240 px row under the command
  line on one display (`ShowEncoderBar` per display); feature-group strip (`EncoderBankSelector`),
  per-profile Encoder Bar pool → bank → page → ≤ 5 encoders (inner/outer); resolution Coarse /
  Fine / Increment / Native, changed on screen only by holding a modifier and tapping; a tap = calculator,
  drag out = an H/V fader pop-up; timing is a hidden **layer** switch (Absolute / Relative / Fade /
  Delay / Speed / Phase …); around the 5 cells sit Sync, SingleStep, Align, Readout, Selection,
  Phaser, Pawterns, ProgFade, ExecFade, Link, Preview, the Grand Master — about 15 buttons, column
  widths tuned to the hardware knobs. **The user's verdict: that encoder bar is bad** — too many
  buttons, timing hard to adjust and **no visual indication of timing** (see
  `inspiration/Dynamic Presets v2.0.png`: Delay / Fade / Value as visible rows per feature).
  **Requirements:** shapers and the colour picker one tap away; good control over effects (§28:
  call size / Rate, line Form / BPM, per-fixture Lo / Hi with `Thru` fans, Stomp).
- **Round 8** (https://claude.ai/artifact/7pifevZdmQyDU1coc6ACF9): three concepts — A tabs + cells
  (the reference baseline), B smart cells, C chip row → slider bank. B: three controls besides the cells
  (one resolution button cycling Coarse → Fine → Ultra, ⏱ Timing, ∿ Effect); the strip starts with
  Effects, Colour, Shapers; Pan + Tilt one XY cell; Colour a hue/saturation picker (a dot per
  fixture); Shapers the beam with draggable blades; gobo / shutter channel sets as chips; a spread
  line with a tick per fixture; **timing and effects always visible** (badges `◢ 2s` / `∿ 5.1`,
  the effect's live output as hollow ticks / XY dots over its swept band); ⏱ adds a delay → fade
  graph per value with the sequence graph's handles; ∿ adds Lo / Hi / Form (Meow = fan); the
  Effects tab: calls (size, Rate, Off, Stomp) and lines (live wave, Form, BPM). Sheet columns
  shown as Compact / Full reference / Minimal. Awaiting the user's pick.
- **Picked (round 8):** concept **A** — "a little more like [the reference] encoder bar but more modern and
  smarter": **four fixed encoder sections** (pages ‹ › per feature), with some smart cells: channel
  functions and **channel sets from the GDTF** (wheel slots as images / colours), the spread line,
  timing / effect markers; the user's `Dynamic Presets v2.0` layout (Delay / Fade / Value rows per
  encoder, Value Layer / Effect Layer n of m, per-encoder resolution). Colour picker and shapers
  stay one tap away. **The vertical sequence sheet's default columns = the full established set** (Cue ·
  Name · Trigger · Trig time · Fade · Out fade · Delay · Out delay · Command · feature groups ·
  Info); every column can be turned off in the window settings.
- **Round 9** (https://claude.ai/artifact/BkWsmoRj2to9p2H47qWY7N): the encoder bar as A — the
  feature strip numbered 1–8 (programmer corner mark, ∿ for a running effect; a second tap knocks
  the feature in); left column: feature + its picture (Colour: a swatch per fixture +
  **Picker**; Shapers: the beam, blades drag; Position: a mini XY pad; Gobo: the current slot) and
  **Value Layer / Effect Layer n of m**; four encoder sections with **Delay / Fade / Value** rows
  always visible (Effect Layer: **BPM / Hi / Lo** of that effect's lines, size / Rate / Off / Stomp
  in the left column, Form in the attribute bar); each section's attribute bar: name, timing and
  effect badges, the GDTF channel function (tap cycles, === lists) and its own Normal / Fine /
  Ultra; wheel attributes as an image carousel (drag rolls, tap = every channel set + channel
  function); right: Page n/m and Program Time. Bar below the command line by default.
- **Round 9 feedback (decided):** the bar sits **above** the command line; the effect indicators
  are good; Delay / Fade rows took too much height. **Round 10**
  (https://claude.ai/artifact/25Ue5CLTXgQqN3VngUtAta, proposed): **Layer** = *what the encoders
  turn* for each attribute, in **sets**: Values (Value · Fade · Delay) and Effect n (Hi · Lo · BPM ·
  Phase — room for Width etc., not tied to timing). The set's rows stack above the four encoders;
  the **chosen layer's row is enlarged**, the others are slim rows that keep showing their numbers;
  a **Layer** button cycles (=== lists, a row label picks); Values / Effect n of m switches the set.
  Indicators: **horizontal** (number + spread, drag sideways) vs **vertical** (a fader per encoder,
  ticks per fixture, drag up / down) on desk — to pick; the **phone always uses vertical faders**.
  `Layer` would be a new published word (`Layer Fade`, `Layer Next`; not registered yet).
- **Round 10 feedback (decided):** **vertical indicators only** (horizontal dropped); Value / Fade /
  Delay sit **side by side** per attribute (three vertical faders; an effect: Hi · Lo · BPM · Phase),
  the chosen layer's fader widens; the phone has **no pages — swipe** sideways through every
  attribute of the feature; the programmer-value red is **harder** (`--lc-programmer` #ff2e4d,
  non-programmed values grey); the feature strip is **taller** (≥ 58 px) with a smart mini
  indicator per feature group. **General UI rule (all pop-ups, everywhere): a tap anywhere outside
  closes it** (a slider panel keeps its value, a pick list changes nothing); **dimming the interface
  behind a pop-up is a user profile setting** (`PopupDim`). **Round 11**
  (https://claude.ai/artifact/JgrLvLJRr6jabz3URHgTgR): that layout; minis — Dimmer: live level bars
  per fixture, Position: dots on a tiny stage, Gobo: each fixture's gobo, Colour: swatches, Beam: a
  cone (zoom width, iris brightness), Focus: a blurring dot, Control: lamp state, Shapers: the beam
  with blades; marks: red corner = programmer, moving ∿ = effect, ◢ = timing.
- **Encoder bar modes** (asked 2026-10-14; round 12, https://claude.ai/artifact/Gp94wDjVPirgZvV1CeV6yj,
  proposed): a **Mode** button above Layer switches what the whole bar is for (a tap opens the mode
  list; per screen): **Attributes** (rounds 9–11); **Cue timing · vertical** (strip = the selected
  sequence's cues with a mini timing graph; encoders = the cue + its feature groups, Fade · Delay ·
  Out faders side by side, inherited values in brackets); **Cue timing · horizontal** (each encoder
  a timeline with delay / fade handles dragged sideways, one shared scale frozen while dragging;
  Layer = In / Out); **Masters** (Grand, Highlight, Solo, Program time, speed masters with Learn,
  group masters — gesture channel, never undone); **Effects** (running calls: Size · Rate, Off,
  Enter). Suggested: **Pawtern** (Interleave / Blocks / Groups / Wings), **Selection** (Previous /
  Next / All, Highlight width), **Executors** (once they exist), **Stage** (Pan/Tilt on the stage
  plan). Per-feature-group cue timing (`Set Cue 3 FeatureGroup "Dimmer" Fade 2`) is new data — the
  open "per-feature timing in 12f?" question.
- **Encoder bar concept approved (2026-10-14, round 12):** concept A with four encoders; **Mode**
  above **Layer** (a tap opens the mode list); Layer sets Values (Value · Fade · Delay) / Effect n
  (Hi · Lo · BPM · Phase) as vertical faders side by side, the chosen one widened; the taller feature
  strip with smart minis; GDTF channel functions / sets (wheels as image carousels); colour picker
  and shapers one tap away; the phone swipes (no pages); the bar above the command line, on / off
  per screen; tap outside closes every pop-up, `PopupDim` a profile setting. Modes: Attributes, Cue
  timing (vertical / horizontal), Masters, Effects; suggested modes still to be scheduled.
- **Per-feature-group cue timing is in 12f** (asked 2026-10-14): a cue part holds Fade / Delay /
  OutFade / OutDelay per feature group (unset = the cue's); new show data, format bump.
- **Attributes: two layers that never mix** (asked 2026-10-14; collision found: `ShowData.attribute_defs`
  links each attribute to one `FeatureGroup`, while the old tree puts `EncoderBarPool` on the
  UserProfile — "per-user encoder pages"):
  - **Show layer (patch):** basic **attribute definitions in the patch**; each attribute belongs to
    exactly one Feature Group (seeded from GDTF, movable to another group). **Cue timing and the
    sequence sheet** read only this.
  - **Presets stay open:** they do **not** follow the show's attribute definitions / feature groups.
  - **User layer (profile):** each user's **encoder bar is editable** (which attributes on which
    strip button, page, encoder). It is **auto-updated from GDTF**: when a patched fixture brings an
    attribute unknown to the show's attribute definitions, it lands in the definitions and is
    **pushed to every user's encoder bar — each user gets a dialog every time** to decide where it
    goes on their bar. So a profile keeps its own setup and stores presets its own way.
  - **Open presets = store what's active:** a preset stores every active programmer value whatever
    its feature group (a preset can mix Dimmer, Colour, Position); no preset pool is tied to a
    feature (an "All" pool); filters are only what the user picks at store time (their encoder
    bar's current strip button, or a chosen filter).
  - **New-attribute dialog:** names the attribute and the fixture type that brought it and
    suggests a spot from its GDTF feature ("Gobo → page 2, encoder 3"); buttons **Put it there ·
    Pick a spot** (the bar opens to tap a free encoder) **· New strip button**; several new
    attributes from one patch = one dialog, a row each. **There is no "leave it off"**: every
    attribute of the show's definitions is placed into a group on every layout — no hidden
    attributes and no "Other" button.
  - **Who / when:** queued **per user profile** and **must be answered by the user** (no
    auto-answer, no expiry). Logged-in users get it at once (after a patch edit is kept, never in
    the middle of it); logged-out profiles at their next login; it waits like other station asks
    ("n waiting") and never blocks playback. Until answered the attribute waits **at its suggested
    spot, marked new** (no "Unplaced" bucket). **An imported UserProfile** (`Import UserProfile`) is checked against the show's
    attribute definitions: every attribute its bar doesn't place gets the same dialog to resolve.
  - **Moving an attribute between feature groups (show layer):** Setup rights + the Patch claim
    (staged patch; Keep files one Oops step), e.g. `Set Attribute "Gobo1Pos" FeatureGroup "Gobo"`;
    cue timing stored for the old group stays with the cue and is listed by the reference check,
    never dropped. Grouping is **automatic** when attributes come into the patch; the user prefers
    the user's **"FX" grouping** — the standard attribute definitions and the user's own loaded ones
    are probed (localhost:3333) before the grouping rules are decided.
  - **Encoder bar layouts: a pool per profile, one active per screen** (an encoder bar pool):
    e.g. "Busking", "Programming", "LED wall"; `Set Screen 1 EncoderBar "Busking"`; the
    new-attribute dialog places the attribute in every layout of the profile (a row per layout,
    the same suggestion prefilled).
  - **A new profile's bar** = one layout "Default" **built from the show's feature groups** (a strip
    button per group, attributes in order) — an ordinary, editable copy (later show changes don't
    move it); the locked UserProfile 1 Default carries it too.
  - **Editing a layout:** an **Encoder bar tab under the UserProfile** (Users & Profiles): a grid
    of strip buttons × pages × encoders, attributes picked from lists; **`Edit EncoderBar` opens
    that tab** (window / menu). Every change is a `Set` line, one Oops step; kit elements only.
  - **Slots:** each strip button (attribute group of the layout) has **pages of 4 slots**; attributes
    are **dragged and dropped into the slot wanted** (both an ordered list and fixed spots: pages
    grow as slots fill). Dropping an attribute **onto a group, not a slot, places it at the
    bottom** (the first free slot at the end). The new-attribute resolve window takes **bulk
    drags**: select several (tap +, **lasso**) and drag them together, or press **"Assign to
    EncoderBar group …"** (a pick list of the layout's groups).
- **Attribute definitions probe (2026-10-14, read only):**
  a reference console's standard attribute definitions (346 attributes, 9 feature groups Dimmer ·
  Position · Gobo · Color · Beam · Focus · Control · Shapers · Video, each with Features, e.g. Color =
  RGB / Color / HSB / CIE / ColorCorrection / HSBC_Shift / ColorKey; every attribute carries an
  explicit `Feature="Group.Feature"`, Pretty, Special, unit, readout, encoder resolution). Unknown
  attributes get the fixture file's Feature string and the console **creates missing groups from file
  strings** (no rule, no fallback — the user's show has empty shells Lamp / Follow / CrowdLED / SFX,
  and third-party GDTFs put junk such as "Pan Invert" under Dimmer). **The user's show (858
  attributes, 15 groups):** only the Feature changed on standard attributes — **FX** = all of
  Gobo.Gobo + Beam minus shutter/strobe (Iris, Frost, Prism, Effects, BeamShaper) + Focus.Focus
  (Focus, Zoom) + effect / animation extras; **Shutter** = the shutter / strobe attributes +
  StrobeMode; Gobo keeps only Media / Object. The console writes the show's grouping into exported GDTFs
  (`Feature="FX.FX"`). Its per-profile attribute preferences hold only readout / encoder
  resolution / press factor / IgnoreChannelFunctions — no per-user grouping. **LumaCore today:**
  `attribute_defs` / `feature_groups` exist but **nothing is seeded** (demo show empty); no Feature
  level, no Special; the GDTF importer ignores `Feature` and its name matching misses `Shutter1`,
  `Focus1`, `Prism1` …
- **Show grouping = templates** (asked 2026-10-14): New Show offers **"LumaCore FX"** (Dimmer ·
  Position · Color · Shutter · FX · Control · Shapers · Video — the user's own grouping) and **"GDTF
  standard"** (the nine); the show keeps the one picked, editable afterwards. **LumaCore FX is a
  "smart" auto resolver** that places new attributes well (rules to be designed; suggestions asked).
  The reference standard set is probed again in full before the resolver rules are written.
- **Patching opens the same attribute editor** (asked 2026-10-14): when a patch / fixture-type import
  brings attributes the show has never seen, the person patching gets **the same attribute editor
  as the UserProfile's new-attribute window** — here for the **show layer** (which feature group
  each new attribute belongs to): the smart resolver's placements prefilled with their reason /
  confidence, bulk select (+, lasso), drag onto a group, "Assign to group …". Kept with the patch
  edit (one Oops step); only then are the users' encoder bar dialogs queued. One editor element,
  two targets (show feature groups / a profile's encoder bar layouts).
- **The UserProfile keeps two things apart** (asked 2026-10-14):
  1. **User attribute definitions** — the user's own grouping of every attribute (their groups,
     e.g. Dimmer / Position / FX …) plus per-attribute readout / encoder resolution (as established,
     UserAttributePreferences). The new-attribute dialog places attributes **here**, once.
  2. **Encoder bar layouts** (the pool) — only how the user's groups are laid out: which group gets
     a strip button, in what order, and the 4-slot pages; a newly placed attribute appears at the
     bottom of its group in every layout showing that group.
  **Store filters live on the UserProfile too** and use the user's attribute groups ("store Shutter
  only"). The show layer (feature groups for cue timing) stays separate.
- **Reference standard set, full probe (2026-10-14):** notes kept outside the repo
  (never committed): 346 attributes (the manual's "366" is
  stale), 9 groups / 20 features / 20 activation + 2 deactivation groups, 91 sub-attributes
  (MainAttribute), no `*_Fine`; unchanged 2.3.1 → 2.5.1 but for PresetMode / Special tweaks and one
  rename. Families cover all 346 names when matched most-specific-first with the Control words
  (`*MSpeed`, `*Reset`, `*Mode`) before Gobo / Color; a show only re-homes whole families (the
  user's FX / Shutter); GDTF scan of 50 files: 185 non-standard names — `ColorAdd_*` / `ColorSub_*`
  = aliases of the RGB family, `ColorRGB_Red…` = Color.Indirect, copies n ≥ 2 join the n = 1 family,
  vendor names with spaces / ALLCAPS route by keywords, the file's Feature only as a last resort.
- **The resolver is data-driven** (asked 2026-10-14): LumaCore ships the 346 standard names (Pretty,
  Special, unit, activation group, MainAttribute) and an **ordered family-rule list** (pattern →
  Group.Feature, most specific first; keyword rules for vendor names). A template ("LumaCore FX",
  "GDTF standard") = which group each family goes to. Rules are **show data** — editable,
  exportable, and they learn ("Apply to similar names?" after a Setup user moves one). Every
  placement records its step and confidence (exact · family · parent · context · hint · guess);
  hint / guess placements land in the patch's attribute editor for the person patching.
- **Each profile has its own family rules** (asked 2026-10-14): a copy of the show template's at
  first; the user's new-attribute dialog is **prefilled by the user's rules** (the show's grouping
  shown beside it for reference) and every answer can be learned ("apply to similar names?").
- **Scope:** this attribute work is its own step **12f-a "Attributes"**, built before 12f (roadmap);
  mockup artifacts first (attribute editor for patch + profile, Encoder bar tab, store filters,
  resolver preview).
- **12f-a artifact round 1** (https://claude.ai/artifact/8Y84Zmvm7Cm8b9B39mDrby): the attribute
  editor for the patch (12 unknown attributes from a GDTF, placed by the resolver with step +
  confidence chips, tap / lasso select, drag the selection onto a group, Assign to group ===,
  "Apply to similar names?" → a learned rule; Keep = one Oops step with the patch edit) and for the
  profile (your groups, prefilled by your rules, the show's group beside it, must be answered,
  "n waiting"); the Encoder bar tab under Users & Profiles (layouts, strip buttons reordered by
  grip, 4-slot pages, drag to swap / free slot, drop on a strip button = move into that group at the
  bottom); store filters (named sets of your groups, `Store Preset 4.2 /Filter "Shutter only"`);
  the show's resolver rules (ordered, first match wins, test box, learned mark).
- **Round 1 feedback:** the Encoder bar tab and Rules & store filters are good; the new-attributes
  table is confusing — it should be **visual like the encoder bar editor, drag and drop**, with
  Assign to group kept. **Round 2** (https://claude.ai/artifact/Vm8CytEwRRAWeogd2i1C4K): four
  variations on one drag engine (tap selects, drag moves the selection, lasso on empty space, drop
  on a group = bottom, on a slot = that slot; Accept suggestions; Assign to group ===; patch or
  profile target): **A Tray** (new tiles above group cards), **B Inbox in the encoder bar editor**
  (an Inbox strip button; suggestions as dashed ghost tiles in slots, tap accepts), **C One at a
  time** (a big card + large group buttons, the suggested one gold), **D Board** (every group a
  column, new cards under a "new" line, auto-scroll while dragging).
- **D (Board) picked** (2026-10-14) for placing new attributes; hardening follows.
  - **One board everywhere:** a kit element `<luma-attr-board>` with a **target** (show / a profile)
    and a **filter** (All / New / Hints & guesses) is the profile's Attributes tab, the patch's
    attribute definitions and the new-attribute dialog (opened with the New filter).
  - **Fit:** desk / tablet — columns shrink to fit every group (min ~110 px; shorter tile names, the
    full one on hover / hold), sideways scroll only past that; phone — two columns at a time with
    snap, a dot map of all groups on top, auto-scroll near the edge while dragging.
  - **Old tiles:** with the New filter each column folds its existing attributes into "+12 existing
    ▾" (tap expands, e.g. to place between old ones); with All everything shows; **page lines every
    4 tiles** in both (the encoder pages).
  - **Order:** a column orders the group (the attribute definition); a newly placed attribute lands
    there in the group and **at the bottom of the group on every layout**; each layout keeps its own
    slot arrangement (Encoder bar tab) — the board never reshuffles an arranged layout.
  - **Tile gestures (proposed, to see on an artifact):** tap selects, **hold opens a detail sheet**
    (what the GDTF says, why it was placed, fixture types using it, channel functions, this
    profile's readout / resolution); the === mark on the tile.
  - **Round 3** (https://claude.ai/artifact/KCyzmaRbsyN7gP6m5B9yft): the board as the dialog (New)
    and as the Attributes tab (All), profile or show; fold "+n existing", page lines, a gold drop
    line, drop on a header = bottom, lasso, chips All / New / Hints & guesses, search highlights,
    "+ New group" column (asks the name; Setup for the show), learning toast with an editable
    pattern, "show: X" compare marks on the profile, **hold → details** in three presentations
    to pick (side panel / bottom sheet / popover: group pick, why here, GDTF says, fixture types,
    channel functions, your readout / resolution, make a rule, show on my encoder bar); phone = two
    columns with snap + dot map.
  - **Details = popover** (picked, round 3): hold a tile → a popover next to it (no dimming).
  - **No live fixture context on the board** (asked: touch-to-test, highlight users, usage counts —
    none); the board stays a pure organising tool.
  - **Sharing a grouping (all four):** **export / import** (`Export UserProfile Attributes "mine"`:
    groups, rules, layouts to a library file; importing places what it knows and opens the board for
    the rest) · **copy from another user** ("Copy grouping from …", one Oops step) · **reset to the
    show's grouping** — **with a warning ask** (keeps layouts' slot arrangement where it can) ·
    a **template library** beyond LumaCore FX / GDTF standard (e.g. "Busking minimal", "Theatre"),
    for a profile or a show — **applying one also asks first** (it regroups everything).
  - **A group carries colour + icon** (NodeMeta, Lucide) — used on the board, the encoder bar strip
    and the sequence sheet's feature-group columns; no default timing, no store behaviour on groups.
  - **The profile gets smarter over time — nothing is deleted:** user attribute definitions and
    rules **keep every attribute ever placed**, also when no patched fixture uses it any more; an
    unused attribute **disappears from the encoder bar** (its slot is kept for when it comes back)
    but stays in the profile's groups. When it is patched again it returns to its group and slot —
    **no new-attribute dialog** for an attribute the profile already knows.
    On the board unused attributes are **hidden behind an "Unused n" chip** (faint tiles, still
    movable); never on the encoder bar.
  - **A new show: the profile's knowledge wins** — the profile's memory places every attribute it
    knows, silently; only never-seen attributes get the dialog (prefilled by the user's rules). The
    knowledge travels with the profile (`Export` / `Import UserProfile`).
  - **The show layer learns only inside its show** — a new show starts from the shipped template it
    picks (no station library memory).
  - **Users answer independently** — each profile its own board; nobody waits for anyone, the show's
    Keep never waits for users, and one user's answers never change another profile.
  - **Speed with many new attributes:** a **"By fixture type"** view (new tiles grouped under the
    fixture type that brought them) and **"Select all <family>"** in the popover (e.g. every
    `Shutter2*` tile in one tap). Keyboard use (arrows / Space / 1–9) waits for a **separate step on
    keyboard tabbing and window focus** (roadmap, Later → UI leftovers).
  - **When show and profile disagree:**
    - The **encoder bar follows the user's grouping**, with an option to **view the show's
      grouping** instead (a toggle on the bar / layout).
    - **Sequences and the sequence sheet are strict show data:** their feature-group columns and
      cue timing always use the **show's** attribute groups; so does the encoder bar's **Cue timing
      mode**.
    - **Individual timing per value is loose:** fade / delay (and other timings) stored **per
      value** (the "individual attribute time" rung of the precedence) may be set **through the
      user's own groups** — e.g. Fade on your "Beam" group in the encoder bar's Fade layer writes an
      individual time on each value — because values are loose in tracking.
    - **Store filters move to later** (roadmap), still on the UserProfile when built.
    - Individual times are set **per attribute only** (encoder by encoder); the user's groups only
      decide which encoders are seen — no group-wide Fade on a strip button.
  - **Tidy groups:** a **context menu on a group's header** (hold / right-click) with **Sort by name ·
    Sort by family · Main attributes first · Merge into … (asks) · Split by family**; **sub-attributes
    follow** — moving a main attribute (Gobo1) offers "Move 4 with it?" (Gobo1Pos, Gobo1PosRotate …);
    a **"+" at the end** of the board creates a new attribute group. New artifacts for all of this.
  - **Everything on the board must be possible by syntax too** (command lines: moves, sort, merge,
    split, new group, colour / icon, rules, export / import / copy / reset / template, readout /
    resolution) — the syntax is hardened **after the board artifact is approved**.
  - **Round 4** (https://claude.ai/artifact/ATM5AiriLFjBKYGfU7UPnB): header menu (hold / right-click), sub-attributes
    follow ("Move n with it?"), "+" column, group colour + icon, Unused chip, By fixture type view
    with a drop rail, popover details with "Select all <family>*", ⋯ menu (export / import / copy
    from / reset to the show's (asks) / apply template (asks)).
  - **Round 4 feedback → round 5** (https://claude.ai/artifact/VXCNvhZWiEUPTArbYKGFYX): **By fixture
    type** = a **mini board per fixture type** (that fixture's attributes in the group columns, old ones
    as small chips, new ones as tiles; drag within a fixture type between groups); **groups the fixture
    doesn't use are hidden**, "All groups (+n)" per type or "All groups for every type" shows them as
    empty drop columns; every new tile shows its **destination chip** (→ icon + colour + group, tap =
    pick list). **By group:** the fold control sits **right below the existing attributes** ("+n
    existing ▾" / "fold ▴"), the new ones under it.
  - **Round 5 feedback → round 6** (https://claude.ai/artifact/E3DXZ9hdU2SrHKpTrvsvsf):
    - **A simple dialog comes before the board** (show and user): **Follow standard attribute
      grouping · Edit attributes · Cancel** — the easy path for normal users (questions on it follow).
    - **By fixture type always shows every group** (no show / hide toggles), **no Accept per type**;
      all columns fit the width (no scrollbar per fixture type on desk); dragging inside a type
      keeps the tile (it stays a tile, marked ✓ placed, in both views — it never disappears).
    - **Phone scroll indicator = a group scrubber**: a segment per group (icon in its colour, a gold
      dot for new attributes), a gold window over the visible columns with their names under it; tap
      jumps, drag scrubs; in By fixture type it scrolls every type's row together.
  - **Cancel in the patch's first dialog** goes **back to the patch**: the patch stays claimed (staged),
    nothing is saved, nothing is sent to the network / replicated yet.
    **Other users' first dialog** (after the patch is kept) has **Later** instead of Cancel: the
    attributes wait at their suggested spots marked new ("1 waiting" in the sidebar) and the dialog
    returns at the next login or from that mark; only Follow or Edit answers it.
    **The person patching gets one first dialog with two lines** — "Show grouping: Follow standard /
    Edit" and "Your encoder bar: Follow standard / Edit" — plus **Follow standard for both** (one
    tap) and Cancel (back to the patch).
    **"Standard" for a user** = the profile's own knowledge and rules first, then the template the
    profile uses (LumaCore FX by default); the dialog names it ("your rules + LumaCore FX").
  - **Round 7** (https://claude.ai/artifact/Y5A1TwGoJeLBpvf21fyxkZ): those first dialogs; **every pick
    list / group window has a close (×) and a Cancel button** besides tap-outside (asked on the phone:
    hold → details → assign to group); on a phone the details popover is a sheet along the bottom.
- **12f-a syntax hardening** (asked 2026-10-14: "the syntax for group is pretty bad, we cannot use
  group … harden the syntax first"; `Group` is the DataPool fixture group; `Attribute` is a command;
  Dimmer / Position / Color / Gobo / Beam / Focus / Control / Shaper / Video are objects):
  - **One word, owner first:** **`FeatureGroup`** (GDTF). A bare `FeatureGroup "FX"` is the
    **show's**; `UserProfile FeatureGroup "FX"` is **yours**; `UserProfile "Bob" FeatureGroup "FX"`
    is Bob's (the owner-first rule, as `UserProfile "Bob" Icon 3`). E.g. `Set Attribute "Zoom"
    FeatureGroup "FX"` · `Set UserProfile Attribute "Zoom" FeatureGroup "Beam"` · `Store FeatureGroup
    "Haze"` · `Store UserProfile FeatureGroup "Busk"` · `List UserProfile "Bob" FeatureGroup`.
  - **`Attribute` becomes an object** (no longer a command): its **Call** keeps `Attribute "Dim" At
    50` (programmer, as today); it gains **Set / List / Info / Edit** for the definition —
    `Set Attribute "Zoom" FeatureGroup "FX"` · `Set UserProfile Attribute "Zoom" Resolution Fine` ·
    `Info Attribute "Pan Invert"` (why it is where it is) · `List Attribute /New` · `Edit Attribute`
    (the show's board) · `Edit UserProfile Attribute` (your board). Owner first as for FeatureGroup.
  - **Several attributes:** `+` joins names and a quoted `*` is a pattern (case-insensitive, whole
    name); no Thru (attributes have no numbers); a pattern matching nothing is E-CMD-007 —
    `Set Attribute "Zoom" + "Iris" + "Frost1" FeatureGroup "FX"` · `Set Attribute "Shutter2*"
    FeatureGroup "Shutter"`.
  - **Position:** `Set … FeatureGroup "x"` lands at the **bottom**; order is **`Move … At`** like pool
    objects — `Move Attribute "Gobo2PosShake" At 3` (inside its group) · `Move Attribute "Zoom" At
    FeatureGroup "FX" 3` (another group, slot 3) · `Move UserProfile Attribute "Zoom" At FeatureGroup
    "Beam" 1`; a drop between tiles sends the Move.
  - **Group lines (no new verbs):** `Store FeatureGroup "Haze"` (new, at the end) · `Label FeatureGroup
    "FX" "Effects"` · `Set FeatureGroup "FX" Color "#b07be6" Icon "sparkles"` · `Set FeatureGroup
    "Beam" Sort Name | Family | MainFirst` · `Set FeatureGroup "Gobo" Merge "FX"` (asks) · `Set
    FeatureGroup "Beam" Split Family` (asks) · `Move FeatureGroup "Shutter" At 2` (column / strip
    order) · `Delete FeatureGroup "Haze"` (only when empty); the same with `UserProfile ["Bob"]` first.
  - **Rules and templates:** an object **`AttributeRule`** (numbered in order, first match wins) —
    `Store AttributeRule "Pixel*" FeatureGroup "FX"` (learned, before the keyword rules) · `Move
    AttributeRule 15 At 3` · `Delete AttributeRule 15` · `List AttributeRule` · `UserProfile ["Bob"]
    AttributeRule …`; the template is a **prop** — `Set ShowData AttributeTemplate "LumaCore FX"` /
    `Set UserProfile AttributeTemplate "Busking minimal"` (both ask).
  - **Answering, as established** (console probe: no Ok / Accept keyword; a question is skipped by an option on
    the asking line, `Delete … /NoConfirmation`; `Attribute` is an object keyword with Call;
    `FeatureGroup` is a keyword; `Default` resets to defaults): the patcher answers on the asking
    line — `Unclaim Patch /Keep` (asks the first dialog) · **`Unclaim Patch /Keep /Default`** (follow
    the standard grouping, show + yours, no ask — one idea, one word, as the `Default` keyword) ·
    **`Unclaim Patch /Keep /Edit`** (keep, then the boards); answers by
    **`Default`** (the established keyword): `Default Attribute /New` (show: place the new ones the standard
    way) · `Default UserProfile Attribute /New` (yours: your rules + template) · `Edit UserProfile
    Attribute /New` (your board); `/NoConfirmation` skips the asks (Reset, Template, Merge, Split);
    `Export / Import UserProfile Attribute "x"` · `Copy UserProfile "Bob" Attribute At UserProfile`.
  - **Encoder bar lines:** `EncoderBar` is a profile-owned pool object — `Store UserProfile EncoderBar
    "Busking"` (a copy of the current) · `Set Screen 1 EncoderBar "Busking"` · `Set Screen 1
    EncoderBar Off | On` · `Move UserProfile EncoderBar "Busking" FeatureGroup "Shutter" At 2` (strip
    order) · `Move UserProfile EncoderBar "Busking" Attribute "Zoom" At 5` (slot) · `Set UserProfile
    Attribute "Zoom" Readout Physical | Resolution Fine` · `Edit EncoderBar` (the tab). The
    show-grouping view is **an option, not `EncoderBarGrouping`** (word being chosen).
    Tentative: **`/Global` (the show's grouping) · `/Profile` (yours)** — `Set Screen 1 EncoderBar
    /Global`; `Self` is taken. The user wants these words checked **in a bigger perspective** (all
    decisions and md files) so the same scope words work for other systems later — analysis running.
    **User's proposal (2026-10-14): name the core objects** — **`/ShowData`** (or `/Show`) for the
    show's layer and **`/UserProfile`** for yours: `Set Screen 1 EncoderBar /ShowData` · `Set Screen 1
    EncoderBar /UserProfile`; to be checked against the analysis (options already reuse object words,
    e.g. `/Screen`, `/User`).
  - **Scope options = the owner keywords (decided 2026-10-14, rule for §15):** **`/ShowData`** (the
    show's shared layer, the tree root the data lives in) · **`/UserProfile ["x"]`** (bare = yours) ·
    later **`/Station ["x"]`** (replacing `/ThisStation`). `Set Screen 1 EncoderBar /ShowData |
    /UserProfile`; the template prop is `Set ShowData AttributeTemplate "LumaCore FX"` (was `Set Show
    …`). Analysis (scope vocabulary across decisions, roadmap, reference): `/Global` is taken three
    ways (GlobalVar = the volatile Runtime tier §5, the planned preset mode `/Global`, GlobalMaster);
    `/Local` = station tier, `/Own` = OwnLink, `/Mine` = Use Mine, `Self` = the window, `/Profile`
    takes a value — all rejected. "Global" stays for the Runtime tier and the preset mode. The same
    switch is needed later for variables, readouts, store filters, keys, look, WindowPreset, Usage,
    Oops per station.
  - **Filters and views as options** (the same options open the board in that state and filter a
    List): `List Attribute /New | /Unused | /Hint` (hints + guesses) · `List UserProfile Attribute
    /Unused` · `Edit UserProfile Attribute /New /FixtureType` (the By fixture type view) · `List
    Attribute FixtureType "ACME Pixel Line"` · `Info Attribute "Pan Invert"` (group, rule, confidence,
    GDTF Feature, fixture types) · `List AttributeRule /Learned` · `List FeatureGroup` (the show's) ·
    `List UserProfile FeatureGroup` (yours).

**12f / 12f-a design locked (2026-10-14, "approved"):** mockups in
`docs/dev/ui-kit/mockups/2026-10-14-12f/` (README "Boards (2026-10-14-12f)"). Build order: **12f-a
Attributes** first (standard table, resolver, GDTF importer fix, show + profile attribute
definitions, `<luma-attr-board>`, Encoder bar layouts, per-feature-group cue timing data, the
syntax above), then **12f controls** (masters in `ShowData.Playbacks.Masters`, gesture channel,
fader, encoder bar with modes, Oops window, Sequences in DataPools + Sequence pool, sequence window A /
B). The open questions are answered below. **Building waits for "go".**

**12f open questions answered (2026-10-14):**
- **"Top" = no new word:** the pool's Top action / sheet button sends `Goto Sequence 3 Cue <first>`
  (labelled "Top" in the UI), with the sequence's Goto timing.
- **The Oops window opens** by **holding the Oops key** (sidebar button or key; a tap still undoes one
  step) and by **`Menu "Oops"`** (a MenuDef: the same line closes it, `List Menu` lists it); a pop-up,
  tap outside closes. Later also in the TUI and elsewhere (roadmap).
- **Suggested encoder modes (Selection, Pawtern, Stage, Executors) are not in 12f** — each gets **its
  own step / phase** to be designed (roadmap). 12f builds Attributes, Cue timing (V / H) and Masters;
  the Effects mode's controls follow the effects build (§28), shown as planned until then.
- **`PopupDim` defaults to On** (a UserProfile setting; anyone can turn it off).
- **"Move n with it?"** follows the profile setting **`SubAttributesFollow Ask | Always | Never`**
  (default **Ask**); the dialog's "Remember" tick sets Always / Never; on a line `/WithSubs` or `/Only`
  answers it and `/NoConfirmation` means Always.
- **What moves into DataPools with Sequences** (asked 2026-10-14 at the 12f go): **Sequences, Timing
  presets and Curve presets** move in the same format bump (the sequence window uses the presets);
  **Effects and Executors stay show-wide for now** and move with their own steps (§28, Playback depth).
- **Build defaults (12f go, 2026-10-14):** parts 1 masters + gesture channel · 2 fader · 3 encoder bar
  (Attributes, Cue timing, Masters) · 4 Oops window · 5 Sequences / presets in DataPools + Sequence pool ·
  6 sequence window A · 7 sequence window B. The grand master's address is `Master 2.1` (`Master 1`
  goes, no alias). An encoder drag on programmer values files one Oops step at its end; master moves
  file none.

**12f against §32 (asked 2026-10-14, after the second merge of bugfixes):**
- **Masters live in `Playbacks.Live.Masters`** (picked over a third branch or `ShowData.Masters`): §32's
  registry is `Playbacks` = `Live` (saved, resumed on load: Sequences, later Effects, **Masters** — Grand,
  Groups by GUID, the planned groups) · `Echoes` (replicated, never saved). Never undone, as decided.
- **12f builds §32's echoes** (picked over "Goto Top now, Top later"): every Sequence gets its fixed
  **CueZero** and **OffCue** (type invariant, can't be deleted / moved / renumbered, load refuses a sequence
  without them; format bump); `Top On|Off Sequence n` with `Playbacks.Echoes.Sequences`, `MaxEchoes` (8) /
  `EchoMax` (30 s), a holder that goes away = `Top Off`; the pool's Top action holds (press On, release
  Off). Effects' echoes come with the effects build (§28). Nothing hardcodes "cue 1": the top is CueZero,
  then the first cue (the old "Goto Sequence 3 Cue <first>" Top is replaced).
- **DataPools:** §32 names the destination (Sequences, Effects, Executors are DataPool objects); 12f moves
  Sequences + Timing / Curve presets, Effects and Executors move with their own steps (answer above).

**12f echoes, asked before part 2 (2026-10-14):**
- **How echoes merge:** **intensity HTP** across the live run and its echoes (overlapping ramp-downs
  stack into a sparkle tail; a new echo never pulls a brighter one down), **every other attribute LTP**
  (the newest run wins), like two sequences today; programmers and other sequences merge with the result.
- **CueZero = the established model** (probed: a live desk's sequence settings, its manual's sequence settings /
  cue tracking): an automatically created cue you can store into (`Cue 0`); the sequence
  setting **`CueZeroMode Off | AllUsedAttributes | OnlyUsedDimmers`** fills it with defaults (Off: nothing ·
  every attribute of the sequence's fixtures · their dimmers); **default Off**. So with Off an echo
  takes non-intensity attributes from where the output stands and fades them into the first cue; its
  intensity rises from nothing (HTP). **OffCue** holds the Off timing; Release is on by default.

**Empty pool objects by Store (asked 2026-10-14, after part 7b; decided, not built — waits for go):**
`Store Sequence 1` did nothing useful (E-CMD-006 "Store which cue?"), `Store Group 3` without a
selection refused (E-CMD-009). Any built pool object can now be created empty with Store:
- **`Store Sequence <n>`:** creates the sequence if missing (CueZero + OffCue, no cues). With
  values in the programmer it also stores them as the **next cue** (Cue 1 in a new sequence, else after
  the last cue); with an empty programmer it only creates it. An existing sequence with an empty
  programmer is left as it is and says so.
- **`Store Group <n>` with nothing selected** creates an **empty group** (filled later with Store /Merge
  or the group editor). An existing group with an empty selection stays as it is and says so (no
  silent wipe).
- **New object words `TimingPreset` and `CurvePreset`** (DataPool objects, owner first like Group):
  Store (empty init), Label, Delete, List / Info, in the tree. Their pool windows come **later**, with
  what the presets do (a timing preset on a cue). Preset, Macro, Layout, World, Filter, Timecode … stay
  planned until their own steps (an empty one would do nothing yet).
- **A quoted name labels it in the same line:** `Store Sequence 3 "Main"`, `Store Group 4 "Front"`,
  `Store TimingPreset 2 "Slow"`; no name = unnamed (shown as "Sequence 3"); storing into an existing
  object renames it only when a name is given.

**Store Cue (decided 2026-10-14, Q&A with the user, one question at a time).**
- **The order of a sequence:** CueZero is Cue 0 and nothing goes before it (a cue number below 0 is
  E-CMD-008); OffCue is always last and nothing goes after it (it stores no values, E-CMD-010).
  Every new cue lands between them.
- **Cue is Store's default object**: in a Store line a bare number is a cue — `Store 2` runs as
  `Store Cue 2`, and the line completes it while typing (`S 1` → `Store Cue 1`, the suggestion
  engine's grammar fills in "Cue" before the number). A bare **`Store`** = `Store Cue`.
- **`Store Cue` without a number** (and bare `Store`): the next cue = the last cue + 1, before
  OffCue (2.5 → 3.5; Cue 1 in an empty sequence) — with the programmer's values, or an **empty cue**
  when the programmer is empty.
- **`Store Cue <n>` for a new number** with an empty programmer creates the empty cue (today it is
  refused, E-PLAY-002); with values it stores them as now.
- **Storing over an existing cue asks** (a station dialog, `core::dialogs`), the full established set:
  **Merge** (today's store: the programmer's values go in, the rest stays) · **Overwrite** (the cue
  becomes exactly the programmer; with an empty programmer: an empty cue) · **Remove** (the
  programmer's attributes leave the cue) · **Create Second Cue** (a new cue right after it, before
  the next one) · **Cancel**. With an empty programmer the ask offers **Merge / Overwrite / Cancel**
  (Merge keeps it as it is). Declared flags skip it: `/Merge` `/Overwrite` `/Remove`
  `/CreateSecondCue`; a one-shot path without a flag is refused with the flag named (the dialogs rule).
- **No selected sequence:** `Store Cue`, `Store Cue <n>`, `Store <n>` and bare `Store` create a new
  sequence at the number after the highest used one, **select it** and store there (the reply says
  so); never Sequence 1 by default.
- **Bare `Store Sequence`** (no number): a new sequence at the number **after the highest used one**
  (1, 2, 5 → 6), then as `Store Sequence <n>` (the programmer's values as Cue 1, else empty).
- **Open (decide while building, said in the build note):** Create Second Cue's number (assumed: the
  next free point number after it — 2 → 2.1 — before the next cue); a Store line inside a key / macro
  without a flag (assumed: refused with the flag named, as one-shot paths).

**Store Cue built (2026-10-14).**
- `playback::store_cue_line` / `store_cue(state, target, StoreMode, name)`: `StoreMode` Ask · Merge
  · Overwrite · Remove · SecondCue from the flags (`flags::STORE_CUE` = /Merge /Overwrite /Remove
  /CreateSecondCue, declared on Store, Cue and Sequence). The ask is a station dialog (E-PLAY-004
  names the flags; a key / macro line without a flag is refused with them). **Only a cue holding
  values asks** — an empty cue (an empty CueZero, a cue made empty) just takes what is stored, since
  Merge and Overwrite would do the same. Overwrite clears every part first; Remove takes the
  programmer's attributes out of every part; Create Second Cue = `second_key` (1 → 1.1, 1.2 …, then
  hundredths, before the next cue; no room = E-CMD-010). A quoted name names the cue.
- An empty programmer stores an **empty cue** ("— empty"); no number = `next_cue_key` (last + 1).
  No selected sequence: `next_sequence_no` (after the highest in your DataPool), created and selected
  ("No sequence was selected: Sequence n created and selected."). A bare `Store Sequence` takes
  `next_sequence_no` too.
- `CommandDef::default_object` (Store: `"Cue"`): `run::store_default` runs bare `Store`, `Store <n>`
  and `Store "name"` as Store Cue (any other word is still "Store what?"); the suggestion engine
  offers `Store Cue <n>` first when a number follows the word (`S 1`), the items replacing from the
  line's start (`suggest::default_object_line`; no wire change).

**12f part 8 built (2026-10-14): sequence window B (the timeline).**
- The Sequence window (type 9) has **Layout `Sheet | Timeline`** and **Zoom** (20–400 %, 100 = 60 px a
  second) — window settings, on its bar by default (`Set Self Layout Timeline`). One element:
  `<luma-cue-sheet>` with `layout = 'Timeline'` (Showroom has both), from the same `sequence` feed.
- A column per cue (CueZero … OffCue): its head (No · Trig, name; running / current marked as on the
  sheet), a ruler in seconds, a row per timing with its span over the seconds — the cue's Fade and
  Delay (Out fade / Out delay when set), then each feature group that has timing of its own (the keys
  it sets, its Path); a fade is drawn in its Path's shape, inherited values in brackets.
- Fields edit like the sheet's: tap = the slider panel, press and drag = live with the gauge; the
  column's spans and ruler redraw in place, the column only grows while dragging (the cues to its
  right hold still); one gesture = one Faders entry and one Oops step.
- A tap on a cue's head = its options (kit menu): Go to this cue (`Goto …`), Add timing…, and per
  feature group with its own timing "<group>: the cue's timing" (one line: `… Fade Cue; … Path Cue`).
  **+ add timing** = feature group (or the cue's own) → type (Fade / Delay / Out fade / Out delay,
  Path…) → the slider panel on that field (its first change adds the row) or the shape list.
- The slider panel's Escape closes only the panel (it reached the page before).
- Not yet: the mockup's other cue options (Trig, Mode, Default path, Snap percent, Info text) wait for
  their `Set Cue` words (roadmap "Sequence sheet leftovers"); pinch zoom (Zoom is the setting).

**Empty pool objects by Store built (2026-10-14).**
- `Store Sequence <n> ["name"]` (`playback::store_sequence`): created unnamed (or named) with CueZero +
  OffCue; active programmer values go in as the next cue = **the last cue + 1** (2.5 → 3.5; Cue 1 in a
  new sequence); an existing one with an empty programmer is E-CMD-009 (or labeled when named).
  `Store Sequence <n> Cue <m>` stores that cue as before. A sequence made by `Store Cue` is unnamed too.
- `Store Group <n>` with no selection creates an empty group; an existing group with an empty
  selection is E-CMD-009 (labeled when named), never wiped.
- `TimingPreset` / `CurvePreset` (`core/objects/presets.rs`): Store (empty init; a CurvePreset starts
  Linear), Label (the Label dialog), Delete (the station's ask, the range rule), List / Info; tree
  nodes `DataPools.<n>.TimingPresets | CurvePresets` are Collections now; owner first works
  (`Store DataPool 2 TimingPreset 5`).

**12f part 7b built (2026-10-14): sequence window A.**
- Window type 9 **Sequence** (Sheets): its `sequence` setting is a `Sequence` link (`"selected"` follows
  your Select Sequence, or a GUID; the pick list holds every DataPool's sequences). `Edit Sequence <n>`
  opens the same sheet as a pop-up (`UiEvent::SequenceSheet`, scope Client; **CLIENT 18**); the pool's
  hold sends it.
- The `sequence` feed (`core::sequence_view`): CueZero (No 0) first, the cues, OffCue last; each cue's
  part-0 timing, values, parts, the running cue; per feature group its own Fade / Delay / Out fade /
  Out delay / Path (null = the cue's). `line` per cue — owner first (`Set DataPool 2 Cue 1 Sequence 4`)
  when the sequence isn't in your selected DataPool; the owner-first form now takes every object that
  lives in a DataPool (its `TreeDef::datapool`) and Cue, not only Group.
- Kit `<luma-cue-sheet>` (`kit/cuesheet.js`, in the Showroom) as approved in round 7: `+` opens a cue
  (feature-group chips, the timing grid with inherited values in brackets, a Path cell picks the shape,
  a column name or chip = the active curve without a jump); the canvas draws every group's shaped
  curve dimmed behind the active one with its delay-end and fade-end handles; the scale is
  `niceTotal` (≥ 2 s, +20 %, rounded), held during a drag, growing only past its end. A tap on a time
  = the slider panel, press and drag = live with a gauge under the finger; values and the curve
  redraw in place and the sheet re-renders only after the drag (scroll kept). Every edit is a
  `Set Cue …` line; a drag rides the gesture channel (one "Faders" entry and one Oops step).

**12f part 7a built (2026-10-14): a feature group's fade Path.**
- `GroupTiming.path: Option<FadePath>` (`Linear` default · `SCurve` · `EaseIn` · `EaseOut` · `Snap`),
  one shape for its in and out fades. **Format 23.** `Set Cue <n> [Sequence <m>] FeatureGroup <g>
  Path <shape> | Cue` (`S-curve`, `ease in` … parse too; a Path without a feature group is E-CMD-006).
- The engine: `Timing.path` (from the group's timing in the part), `plan::Fade.path`; the tick shapes
  `t` with `FadePath::at` (no allocation); `Snap` jumps at the end of its delay like a GDTF snap
  channel. An echo's fades take the value's path too.

**12f part 6b built (2026-10-14): the Sequence pool.**
- Window type 8 **Sequences** (one pool window, `buildPoolPanel`, fed by the `pools` feed; DataPool
  `"selected"` or a GUID). Executor-style cells: name, then the previous / current / next cue lines,
  the current one framed (gold while the sequence runs, plain while off); a phone shows only the current
  line. A running sequence has the blue mark, the selected one the gold frame.
- Tap actions (`SEQUENCE_POOL`): **Select** (default), Go, Toggle, Off, **Top** (held: `Top On` at the
  press, `Top Off` at the release, §32), Edit, None. `PoolAction.release` is the line a release sends;
  a cell whose action has one is press-and-release in `<luma-pool>` (no hold, no drag). The window's
  `hold_action` setting (Edit | Sheet) is what a hold does; `Edit Sequence` stays planned until part 7.
- `Label Sequence <n> ["name" | /Clear]` (the selected DataPool's, the Label dialog without a name),
  through the shared `label_or_ask`.

**12f part 6a built (2026-10-14): Sequences, Timing and Curve presets in the DataPools.**
- `Datapool { …, sequences: BTreeMap<u32, Arc<SequenceObject>>, timing_presets, curve_presets }`;
  `PlaybackData` keeps Effects, Executors and Park. A sequence's `guid` + `name` are a `NodeMeta`
  (name, note, colour, icon …). **Format 22**; the show file writes `DataPools/<n>/Sequences.json`,
  `TimingPresets.json`, `CurvePresets.json` (the generic per-pool files).
- `ShowData::sequences()` (every pool), `sequence_by_guid` → (datapool, number, sequence),
  `sequence_mut(dp, n)`; `AppState::selected_sequence()` → (datapool, number). Commands take a typed
  `Sequence 3` in the selected DataPool (or the owner-first line's, `DataPool 2 …`); bare words use the
  selected sequence wherever it is. The tree: `Root.ShowData.DataPools.<n>.Sequences | TimingPresets |
  CurvePresets`; the Sequence object's settable properties are NodeMeta's plus Tracking, CueZeroMode,
  MaxEchoes, EchoMax. Select / the pick lists / the import / the backups index follow.
- **Not built (there were no such words before):** Copy / Move / Delete of a Sequence (Delete stays
  planned) — roadmap.

**12f part 5 built (2026-10-14): the Oops window.**
- `Menu "Oops"` (a page menu, alias `undo`; the same line closes it) and a core **`oops` UI key** (tap =
  `Oops`, hold = `Menu "Oops"`) with its sidebar button (OOPS); on a phone (the sidebar folded) the line or
  a key opens it. `Redo [<n>]` redoes the newest n undone steps (stops at the first that can't).
- **`oops` State feed** (per user): the station view's `undo` (each logged-in user's steps oldest first —
  line, category, station, time, Main — the newest 200, and the undone ones next-redo first; counter
  `Revisions.undo`), the profile's number, PopupDim and its four Oops filters.
- `kit/oops.js` (`LumaKit.oops.openOops`): a pop-up (tap outside closes, PopupDim dims), newest at the
  bottom; a tap selects a step and every newer one → **"Oops n steps"** (`Oops n`; never a one-tap undo,
  OopsConfirmation still asks); undone steps dimmed below a line, a tap selects them → **"Redo n steps"**;
  the chips are the profile's Oops filters (CreateOops, OopsProgrammer, OopsSelection, OopsViews — each a
  `Set UserProfile <n> <filter> Yes|No`; the round 7 board's chips filtered the view, §30 said the
  profile's filters — the filters it is).
- e2e: hold the key (a phone: the line), Oops 3 steps, Redo 2 steps, a chip, tap outside (desktop, tablet,
  phone, WebKit). The Oops window in the TUI stays on the roadmap.

**12f part 4c built (2026-10-14): the encoder bar's Cue timing modes.**
- `encoder_view::cue_timing`: the strip = the selected sequence's cues (the current one marked, a mini of
  each cue's delay and fade); `group` 0 = the current cue (`EncoderMode CueTiming…` sets it; its first cue
  when off); the encoders = the cue's own part timing, then every show feature group (In Delay / Fade, Out
  Delay / Fade; an unset group time shows the cue's in brackets), four a page; lines `Set Cue <c> Sequence
  <s> [FeatureGroup "g"] Fade | Delay | OutFade | OutDelay {value}`.
- **Vertical:** F · D · Out faders in seconds (inherited values dimmed). **Horizontal:** an In and an Out
  timeline per encoder on one shared scale (≥ 2 s, longest delay + fade +20 %, held while dragging), the
  delay-end and fade-end handles dragged sideways. Both drawn (round 12 had a Layer In / Out switch for
  the horizontal one: showing both rows keeps the Out timing in sight).
- **One Oops step per drag, any subtree:** `gesture::remember` keeps the first `before` of every subtree
  a gesture's moves change (from the change path), and its end puts them back first — so a timing drag
  (the Playback subtree) is one step like a programmer drag; the 4a Programmer-only snapshot is gone.
- e2e: opens on the current cue, a vertical F drag and a horizontal fade-handle drag each file one Faders
  entry (desktop, tablet, phone, WebKit).

**Cue timing mode opens on the current cue (asked 2026-10-14, round 12's open point):** the encoder bar's
Cue timing modes start on the cue the selected sequence stands on (its first cue when it is off); a strip
tap picks any other cue.

**12f part 4b, the page half (2026-10-14): `<luma-encoder-bar>` and the slider panel.**
- `kit/encoderbar.js` above the command line bar (`#encbar`, hidden by `EncoderBar Off`, F12 and the
  mirror): the strip (a button per layout group, its colour line, the red programmer corner, a mini of
  per-fixture levels; tap = `Set Screen EncoderGroup n`), the left column (feature, **MODE** — a list,
  `Set Screen EncoderMode …`, Effects marked planned — and **LAYER** n of 3, `EncoderLayer Next`), four
  encoders each with **Value · F · D** vertical faders side by side (the layer's one wide; Value red when
  programmed, `lo…hi` readout, a tick per fixture; F / D `2s` | `cue` | `mixed`), the attribute row (name,
  timer badge, attribute, resolution), Page n/m ‹ ›. Masters mode = a `<luma-fader>` bank.
- **Gestures:** press and drag adjusts at once (Value 0–100 %, F / D 0–10 s), one gesture per drag (one
  Oops step, part 4a); a tap on the wide fader opens the **slider panel** (`kit/sliderpanel.js`: − / +,
  Keypad, a drag in it; a tap outside closes it, dimmed by the profile's PopupDim — the feed's
  `popup_dim`); a tap on a narrow fader picks its layer; the wheel steps (Coarse 1 / Fine 0.1 / Ultra
  0.01; one gesture until 400 ms of rest). Phone (< 600 px): no pages — `all` of the feature swipe.
- The feed follows a new **`modes`** counter (`Revisions.modes`, `FeedRevs.modes`) so your own Highlight
  / Solo masters show at once. Touch targets ≥ 44 px on coarse pointers (narrow F / D faders, page buttons).
- e2e (`kit-12f.spec.ts`, desktop / tablet / phone / WebKit): strip, a Value drag = one Oops step and one
  Faders entry, the slider panel and tap-outside, MODE → Masters with your own master following, off.
- **Not yet (roadmap):** the smart minis beyond levels (position dots, gobo pictures, colour swatches,
  the beam cone, shapers), GDTF channel functions / channel sets and wheel carousels, the colour picker and
  shapers one tap away, the second strip tap = knock in, Prog time, the Effect layers (§28).

**12f part 4b, the station half (2026-10-14): the encoder bar's state, words and feed.**
- **State:** what a screen's bar shows lives on the live screen, runtime only (`Screen.encoder`:
  `EncoderState { mode, group, page, layer }`, `EncoderMode` Attributes | CueTimingVertical |
  CueTimingHorizontal | Masters | Effects, `EncoderLayer` Value | Fade | Delay) — switching is never an
  Oops step, like a tab. Whether it shows and which layout is saved on the profile's screen config
  (`ScreenContent.encoder_bar: EncoderBarSetting { on, layout }`; `on` None = on for Screen 1; kept across
  the live → saved copy; **format 21**).
- **Words:** `Set Screen [n] EncoderBar On|Off|Toggle|"layout"` · `EncoderMode <mode>` · `EncoderGroup <n>|Next|
  Previous` · `EncoderPage <n>|Next|Previous` · `EncoderLayer Value|Fade|Delay|Next` (no number = the screen
  the line comes from). Stepping is `Next` / `Previous`, not `+ 1` (a `+` reads as a row range).
- **Feed `encoder`** (`screen` param; rebuilt on show, screens and station moves): `core::encoder_view` —
  Attributes: the layout's strip (group name, colour, icon, programmed, a mini of per-fixture levels), the
  current button's four encoders per page (readout or `lo…hi`, per-fixture levels, programmed, Fade /
  Delay `{text: "2s" | "cue" | "mixed"}`, resolution, the value / fade / delay lines); Masters: 2.1, 2.2, 2.4,
  your own Highlight / Solo (the station view's `mode_masters`) and the group masters of your DataPool;
  Cue timing → part 4c, Effects → §28 (planned).

**12f part 4a built (2026-10-14): value timing, one Oops step per drag.**
- `lumacore_base::value::ValueTiming { fade, delay }` (seconds, `None` = the cue's) on `ProgValue` and
  `CueValue` (**format 20**); Store Cue keeps it; the engine's `playback::value_timing` = the part's, its
  feature group's, then the value's own (`Timing::with_value`, In and Out alike) — for live runs, echo
  offers and echo intensity. Live programmer changes still snap (the timing only travels into cues).
  Word: `Attribute "Dimmer" Fade 2 | Delay 0.3 | Fade Cue` on the selection's programmer values (none →
  E-CMD-009).
- **One Oops step per drag:** `GestureLine::Move { id } | End { id, summary }`; `gesture::before_line`
  (dispatch, where the line runs — the master for a member) keeps the user's Programmer subtree when the
  gesture begins and puts it back before its end runs, so the end's step spans the drag; moves are
  Untracked. Station-local `AppState::gesture_starts` (at most 64 open).

**Value timing, asked before part 4 (2026-10-14): build it in 12f.** The encoder bar's Value · Fade · Delay
faders (rounds 10–12: "2s" or "cue") are individual timing: a programmer value may carry its own Fade
/ Delay (unset = the cue's); Store Cue keeps them on the cue value; the engine uses a value's own timing
over its feature group's and its part's. New show data (format bump). Part 4 = 4a value timing + one Oops
step per encoder drag · 4b the encoder bar (Attributes, Masters) · 4c the Cue timing modes.

**12f part 3 built (2026-10-14): `<luma-fader>`.** `kit/fader.js` (Showroom entry: the grand group and
your Highlight as live faders): round 7's look B (the filled bar, the cap line in the derived
`--lc-cap` role, value on top, name + address below), 300 px travel (`--lc-fader-height`, 400 px by
default). Drag only and relative (a tap never jumps, no hold, no double-tap); each finger owns one
fader; a second finger on it = fine ×0.1; snaps within 2 of the ends; the bubble shows the line while
dragging; moves go out at most every 33 ms. Every move emits `gesture` (`id, line, phase, label, from,
value`) and the page sends it with `wsGesture` (a document listener; a window control that needs `Self`
sends its own). e2e: a real drag moves `Master 2.1` and files one "Faders" entry, a tap sends nothing
(desktop, tablet, phone, WebKit).

**12f part 2 built (2026-10-14): CueZero, OffCue, echoes.**
- **Sequence:** `cue_zero` and `off_cue` are fixed `CueObject` fields (no serde default: a file without
  them is refused; **format 19**); `tracked()` starts at CueZero (`CUE_ZERO` = 0; `SequenceObject::cue`).
  Words: `Cue 0` / `CueZero`, `Cue OffCue` (`playback::cue_ref`); `Store Cue 0` stores into CueZero, the
  OffCue refuses values (E-CMD-010) and takes timing (`Set Cue OffCue … OutFade 2`); neither is a Goto
  target; `List Cue` shows both. `CueZeroMode` fills **CueZero part 1** with fixture-type defaults (the
  used dimmers or every used attribute; part 0 = what you stored, which wins), refilled on every Store and
  mode change. Sequence settings are props now: `Set Sequence n Tracking | CueZeroMode | MaxEchoes |
  EchoMax`.
- **Echoes:** `Playbacks.echoes.sequences: guid → [Echo { id, cue, started_ms, stamp, holder, started_by,
  off_ms }]` — replicated, stripped from the file, cleared on a file load. `Top On|Off Sequence n` (Playback
  rights, Untracked); `Off Sequence` ends them. The housekeeping tick (`playback::echo_tick`, the station
  that owns the show) starts the OffCue past `EchoMax` or when the holder (the client) is no longer
  attached on an unlocked device, and removes an echo once its OffCue has faded — filed with
  `record_change` (Untracked), so it replicates.
- **Engine:** echo offers join the LTP merge stamped at Top On, except intensity (`resolve_cached`'s
  `is_intensity`); intensity is `ValueTier::echoes` (`EchoLevel { channel, to, fade }`, sorted): rising from
  CueZero's level (else 0) by the cue's In timing, or from where it stood to 0 by the OffCue's Out timing;
  the tick takes the max with one cursor before masters (no allocation; Park still wins).
- **Not built (recorded):** echoes don't advance through Follow / time cues (live runs don't either yet),
  `Kill` doesn't exist yet, the live run's Off still snaps (the OffCue times echoes only), non-intensity
  attributes hand back with a snap when an echo goes, readers (`resolve`) see an echo's intensity as LTP.
  Effects' echoes come with §28.

**12f part 1 built (2026-10-14): the Playbacks registry, masters, the gesture channel.**
- **Registry:** `PlaybackStates { live: LivePlaybacks { sequences, masters } }` (§32's `Live`; `Echoes` come
  in part 2); tree `Root.ShowData.Playbacks.Live.Sequences | Masters` (Live rows). The undoable grand master
  pool (`PlaybackData.masters`, `PlaybackMaster`, `MasterType`) is gone; **format 18**.
- **Masters:** `Masters { grand: GrandMasters { master, on, highlight, solo }, groups }` — `Master 2.1`
  (`Master "Grand"`, Off = blackout), `Master 2.2` / `2.4` cap everybody's Highlight / Solo (the engine
  gets each user's own strength × the cap), `Master Highlight|Solo` stay yours; `Master 1`, `2.3`, `2.5`,
  groups 3–5 reply planned (E-CMD-003; `Master 1` points at 2.1). Relative input everywhere a master takes
  a level (`playback::level_after_at`: `+ 10`, `-10` spaced, `* 2`, `/ 2`, Full, Zero; group masters too).
  The `Master` object is `Untracked`: a test checks no master move files an Oops step.
- **Gesture channel:** `/ws/shell` `{"type":"gesture", id, line, phase, window?, label?, from?, value?}` →
  `ShellClientPacket::Gesture` (**CLIENT_PROTOCOL 17**) → the shell's `run_gesture` (`AppState::gesture`,
  `core::gesture::GestureLine`): no echo, history, Usage, journal or `Cmd` line per move; while the shell
  queue waits only the newest move of a gesture stays (a dropped move hands on its `label` / `from`); the
  end files one journal entry + one System Monitor line under **Faders** ("Fader Master 2.1: 100 → 63.5 ·
  Admin"); a refusal is a notice once per gesture. A member forwards the mark (`StreamMsg::Command.gesture`,
  **SESSION_PROTOCOL 14**), so the master stays quiet too. Page: `wsGesture(g)` (kit controls only).
- Tests: masters never undone + relative input, planned masters, the grand caps, one entry per gesture;
  e2e `kit-12f.spec.ts` (gesture end to end, a refusal told once).

**12f-a built (2026-10-14, parts 1–4b: 71a0401, debe880, f6dee42, 8a14189, part 4b):**
- **Data** (part 1): the 346 standard attributes and the resolver's rules / templates are data files
  (`crates/lumacore-tree/data/`); `ShowData.attribute_rules`, `ShowSettings.attribute_template`,
  `CuePart.group_timing`; the GDTF importer keeps each revision's `<AttributeDefinitions>`
  (`Revision.attributes`); **format 16**. A patch change outside a draft defines its new attributes in
  the same step (`change::finish` → `attribute_resolver::sync`); that sync's write into the locked
  Default profile is the sync's, not the line's (never E-USER-002).
- **Syntax** (part 2) as hardened above; `Attribute` is an object (its Call keeps `Attribute "Pan" At
  120`), `Default` is a command. **Deviation:** changing the show's attribute definitions needs Setup
  but **no Patch claim** (the board opens after Keep, when the patch is no longer claimed).
  **Deviation:** a **learned rule wins before the exact standard name** (a user's "apply to similar
  names" must beat the shipped table, or learning a standard family would never take).
- **Profile layer** (part 3): `UserProfile.attributes` (`ProfileAttributes`: template, groups, rules,
  `placed` — every attribute ever placed, `encoder_bars`) replaces the old overrides; the resolver
  works on either layer (`AttributeLayer`). New attributes reach every profile at their suggested spot,
  marked new (the locked Default unmarked); each user is asked at login and after a Keep (Follow
  standard / Edit / Later); the patcher gets the one two-line dialog (`Unclaim Patch /Keep {show}
  /Mine {mine}`). `PopupDim` (On) and `SubAttributesFollow` (Ask; a new "Attributes" tab of the
  profile's settings). **Not built:** the "Remember" tick on "Move n with it?" (the setting itself and
  `/WithSubs` / `/Only` are).
- **UI** (parts 4a / 4b): `<luma-attr-board>` (`kit/attrboard.js`) fed by the **`attributes`** State
  feed (`core::attribute_board`, `profile` = a number or `"mine"`), opened by the **`attribute_board`**
  UI event (`Edit [UserProfile] Attribute [/New|/Hint|/Unused] [/FixtureType]`, `Unclaim Patch /Keep
  /Edit` — the show's board, then yours; **CLIENT_PROTOCOL 15**); a terminal gets the list. Every
  gesture is a line: drop on a header / below the last tile = `Set … FeatureGroup` (bottom), between
  tiles = `Move … At FeatureGroup "G" n`, "+" = bare `Store [UserProfile] FeatureGroup` (the station
  asks the name), the learning toast = `Store … AttributeRule`, the header menu = `Set FeatureGroup …
  Sort | Merge | Split`, `Move FeatureGroup … At n`; **Delete stays a command line** (no hand-made
  Delete outside the slab, tidy). The **Encoder bar tab** is a `Content::Custom` tab of the Users &
  Profiles menu drawn by `<luma-encoder-bars>` (`Edit EncoderBar` opens it; your own layouts — another
  profile's by its lines); bare `Store UserProfile EncoderBar` asks the name. **"n WAIT"** in the left
  sidebar (`Status.attributes_waiting`, tap = `Edit UserProfile Attribute /New`). Nine Lucide icons
  for the groups (sun, move, palette, zap, sparkles, sliders-horizontal, crop, video, focus …) and the
  menu's sort / merge / split icons are in the sprite. Open items (sharing a grouping, a screen's
  layout, the board for another profile, the TUI board) are on the roadmap.

**Users & Profiles follows storage (asked 2026-10-14, after 12f-a part 4b: "the Encoderbar tab needs to
live under the userprofile, all data that is nested in the Users Profiles menu needs to be logical where
it is stored"):** the menu shows data where it lives in the tree (`Root.ShowData.Users.<n>`,
`Root.ShowData.UserProfiles.<n>.…`). The top-level "Encoder bar" tab (12f-a part 4b) goes.
- **Shape (picked):** the **Profiles grid stays the list**; a picked profile opens **its detail** with
  its parts as tabs: Settings · Attributes · Encoder bars · Keys · … (two levels).
- **Detail opens by drilling in (picked):** the same menu swaps its content to the profile — a
  breadcrumb `‹ Profiles › Admin` (‹ back), the part tabs under it; one window, the phone the same.
- **Part tabs by what they are (picked):** Settings (the preference sections inside) · Look (theme, lock
  screen, images, icons) · Attributes (your groups + rules, the board) · Encoder bars · Keys · Screens &
  views · Window presets.
- **Users drill in the same way (picked):** `‹ Users › Bob`: Account (name, rights, password, colour,
  remote login) · Profile (the link, Open profile ›) · Work (programmer, selection, selected objects —
  views + Clear) · Usage.
- The user asked for **artifacts of the variants of questions 1–3** (structure, where the detail opens,
  the part tabs) to choose on before locking.
- **Round 1** (https://claude.ai/artifact/6GdXA1Wecmpwho9sFA4Fxo) **picked: 1 B (grid, then detail), 2 drill in,
  3 Settings / Content** (two groups: Settings = the preference sections; Content = Attributes, Encoder
  bars, Keys, Window presets, Images, Icons, Screens). Good on the desk; **too cramped on a phone** →
  round 2 = phone variants.
- **The tree follows the same split:** `ShowData.UserProfiles.<n>.Settings` and `….Content` (Attributes,
  EncoderBars, Keys, WindowPresets, Images, Icons, Screens …) — the menu and the tree name the same
  things in the same places (a show format bump when built; forward only).
- **Later:** more categories on the user, e.g. **External logins** (web servers, APIs) — roadmap.
- **Round 2** (https://claude.ai/artifact/HLdwfLdZHvSsvnozE3RsnR): **P1 approved — a screen per level on a
  phone**: Profiles (Users / Profiles switch) → the profile (two big tiles Settings / Content) → the
  group's parts as rows → the part's editor, each the whole phone; ‹ one level up, crumbs jump further;
  the encoder bar editor on a phone = layouts and strip buttons as scrolling chips, 2 × 2 slots per page.
  The desk keeps round 1 (left Users / Profiles tabs, the grid, drill-in with `‹ Profiles › name`, the
  Settings / Content tabs with their parts as second tabs).
- **Locked 2026-10-14** (mockups `docs/dev/ui-kit/mockups/2026-10-14-users-profiles/`); building waits for go.
- **Built 2026-10-14 (go; part 1 88f854f, part 2):**
  - **Tree:** `UserProfile` = meta + locked + `settings` (`ProfileSettings`: preferences, theme, lock
    screen, prog lang / alias depth, descriptions, clear buffer, verbosity, permissions, menu state) +
    `content` (`ProfileContent`: attributes with encoder bars, keybindings, UI keys, views, view buttons,
    screen configs, window presets, images, icons, variables, the selected preset group). The show file
    follows (`UserProfiles/<n> <name>/profile.json · Settings.json · Content.json ·
    Content/Icons|Images/…`, **format 17**); the tree lists `Root.ShowData.UserProfiles.<n>.Settings |
    Content` and their parts (named tree folders now resolve by name). Profile props write
    `settings.…` paths.
  - **Menu:** a grid declares its row **detail** (`menu_defs::Detail`: owner / path / edit templates, a
    `detail` feed kind, groups → parts; a part = settings cells of a section, a page element, or a
    read-only list). `<luma-grid open>` ends each row with ›; `<luma-setup>` swaps the grid for the new
    kit element **`<luma-detail>`** (desk: `‹ Profiles › Busking`, the stored path, group tabs, part
    tabs; under 600 px **P1**: tiles → rows → part, ‹ one level up, crumbs). The page answers with the
    object's `EditSetting` fields, the **`detail`** State feed (a profile's Keys / Window presets /
    Images / Icons / Screens, a user's Work and Usage) and, for a profile, `<luma-encoder-bars>` on that
    profile's `attributes` feed (any profile now, not only yours). Users: Account (all the user's
    settings) · Profile (the link + Open profile) · Work (+ Clear programmer) · Usage.
  - **`Menu "x" [/Tab t] /Open "<row> [<group> [<part>]]"`** drills in (`OpenMenu.open`,
    **CLIENT_PROTOCOL 16**); `Edit EncoderBar` = `Menu "Users" /Tab Profiles /Open "<yours> Content
    EncoderBars"`. The top-level Encoder bar tab is gone.
  - **Not yet:** the Content lists are read-only (no slab: add / delete of keys, images, presets still
    by their lines); the Look part shows the profile's theme props as they were (a whole-object "Theme"
    cell, long palette names) — tidy them when the theme editor is designed.

**Second merge of bugfixes (2026-10-14, asked: "merge bugfixes"):** `origin/bugfixes` (§31 built: declared
flags, typed replies, typed `TreePath`, two-lane state queue, `Arc` parts; §32 decided) merged into main after
12e, 12f-a and Users & Profiles. What main's newer code had to follow:
- **The profile's halves are typed tree nodes:** `UserProfiles.<n>` takes the schema `UserProfile` (Settings,
  Content, each part a declared node: `…Settings.Theme`, `…Content.Attributes`); a collection whose entries
  have a schema takes that schema's children below an entry (`Node::entry_schema`, `TreePath::join`). The old
  nodes straight under `UserProfiles` (Views, KeyBindings, Preferences …) are gone (golden updated).
- **Flags declared:** `Attribute` (`/New /Unused /Hint /FixtureType /WithSubs /Only`), `AttributeRule`
  (`/Learned /Keyword`), `Default` (the attribute flags), `Unclaim` (`/Default /Edit /Mine <Default|Edit>`),
  `Menu` (`/Open`); a test runs the 12f-a / Users & Profiles lines and finds no E-CMD-018.
- **Profiles are `Arc`:** writes go through `profile_mut` / `Arc::make_mut` (the resolver only when something
  new arrives). The plan compiler's sequence cache keeps per-feature-group timing and is cleared when the
  attribute → feature group links change.
- Kept main's: `Default` = the attribute word (12f-a), Grid group tools (12e), the datapool argument of a
  property writer (12e). Clippy clean again; `cargo fmt --check` was not clean on either branch (not done here).

**Open (asked in each part's hardening):** element list per part, exact token names, the icon
set, pool action-row order (12e: decided on the artifact), window-bar minimum sizes, how `Self` reaches a plugin, the Oops
window and Sequence pool (12f), keyboard layouts beyond English (data files ready).

---

## 31. Code review hardening (2026-10-13)

A read-only review of the whole codebase (core, DMX path, server / network / LumaSession / HTTP,
workspace and build, frontend and clients), checked against this file and researched against
well-known projects (QLC+, JACK / PipeWire, PostgreSQL, Git, rust-analyzer, nushell,
git's parse-options, Electron / Tauri, Anthropic's CLAUDE.md guidance). Every choice below was
asked and answered on 2026-10-13. Nothing here changes behaviour a user sees except Health
lines and the CLI no longer hanging. Status: **decided, not built** — the work list is the
roadmap's "Code review (§31)".

**Real time**

- **Line timing: measure, then lock** (amends §24 "each line's driver has its own timer").
  First each output line counts frames it sent twice and frames it skipped, shown in `Info
  Health Lines`. Then a line whose `Rate` divides 60 (60, 30, 20, 15, 12, 10, …) sends every Nth
  engine frame: the engine bumps a frame sequence when it publishes, the driver waits on it —
  no doubled or skipped frames on slow fades (video's lesson: whole-number ratios don't
  judder; a reference console runs a ~60 Hz loop and sends ≤ 30 Hz). Other rates keep their own timer; the
  timer also stays as the fallback while the engine is down.
  Step 1 built (`LineStats.doubled` / `skipped`, `Info Health Lines`); first numbers (LumaCoreA,
  30 Hz, no real-time priority): idle 0 / 0, fully loaded 0 sent twice and 1 skipped of ~1,380
  sends. Step 2 waits for numbers from real shows and slow fades.
- **Render budget stays 3 ms** (§24, a reference console's own channel calculation is ~2.7 ms for ~35k parameters).
  Benchmarks added for the paths the current bench misses: input merge with 64 layers, running
  fades, `compile_after` one programmer change on a 500-cue tracking sequence, a channel-tier
  rebuild, the published-show clone. Baselines are stored; a run 20 % slower than its baseline
  warns. 1 ms is a goal, never a failing test.
- **Thread priority: real `SCHED_FIFO`** (§24's "highest thread priority", which today doesn't
  happen: `ThreadPriority::Max` keeps normal scheduling and only asks for nice −20, which fails
  without rights, logged once). The engine asks for `SCHED_FIFO` at a modest priority (~70, below
  audio / IRQ threads), output-line drivers a little lower, through `RLIMIT_RTPRIO`; RTKit is the
  fallback on desktops. An install file for `/etc/security/limits.d/` grants it (JACK's way).
  `Info Health Threads` shows the policy and priority each thread got, or why not.
  Built: `lumacore_dmx::realtime` (`ENGINE_PRIORITY` 70, `DRIVER_PRIORITY` 65),
  `scripts/unix/realtime.conf` for the group `lumacore` (`dev_setup.sh --realtime`), the `scheduling`
  field on each thread's health; RTKit is not built yet.

**Data and LumaSession**

- **Show snapshots: programmer snapshot first, then `Arc` parts.** §26 "Step 2 details"'s
  programmer snapshot ("so an `At` does not republish the whole show") is built first. Then
  `ShowData`'s fields become `Arc` parts one at a time (`Arc::make_mut` on write), so a publish
  copies only what changed. The saved form doesn't change (`FORMAT_VERSION` stays). The change
  path passes its set of changed subtrees on: in `EngineInput` (the plan compiler redoes only
  those) and to the feed hub's revisions (a pool window follows its datapool, not the whole
  show). Feed revisions stay content fingerprints, never counters (§30 "12d part 1").
  *Built 2026-10-14:* `Arc` per sequence, fixture type and user profile (copy-on-write; the
  saved form unchanged); the merge keyed by the tier's attribute index; the compiler's
  `SequenceCache` (a running sequence's offers kept while its `Arc` and cue stay). The order
  was asked again 2026-10-14: this before typed replies (the benchmarks put the cost here).
  Typed replies, when built: `DispatchResult::Output` carries a `Reply { text, error }`.
- **Session stream: keep JSON + zstd, stop the waste** (§21 "6b" stands). Embedded shows
  (`Welcome` / `Preview` / `MergeOffer`) pass through as raw JSON instead of JSON inside a JSON
  string; snapshot bytes (`SubtreeSnapshot.data`, `LocalSnapshot`) travel as base64, not number
  arrays; the master encodes each broadcast once and sends the same bytes to every member.
  `SESSION_PROTOCOL` is bumped. Binary (postcard) only if measurements still point at the
  stream.
  Built (protocol 13, `lumacore_base::embedded_json`): snapshot bytes that are JSON travel as
  that JSON (better than base64: zstd sees the text), other bytes as `{"b64": …}`; the shows
  as raw JSON (`embedded_json::text` / `text_opt`, the fields stay `String`). `StreamMsg` is
  adjacently tagged (`kind` + `body`): an internally tagged enum buffers its fields and can't
  carry raw JSON. `broadcast` encodes once for two or more members (`StreamMsg::Encoded`, local
  only, `encode_frame`).
- **Replication journal: a byte cap.** 1000 entries **or 64 MB**, whichever comes first; a
  member that falls behind the trimmed range gets a full snapshot, as today (§10, §21 "6b
  built").
- **State job queue: two lanes.** A shell's keys, lines, taps and answers (and a logout's
  revoke) keep their guaranteed, unbounded lane — never dropped (§26). Machine senders (HTTP,
  OSC, feeds, the UDP command port, station requests) go to a bounded lane; a full lane refuses
  with E-CMD-017 / HTTP 503. Its depth is shown in Health. (Fred Hebert's "queues don't fix
  overload": block or shed; a person typing can't outrun the state thread, an OSC flood can.)
  *Built 2026-10-14:* `server::state::Lane` — `StateHandle::machine()` for HTTP + feeds
  (`ServerConsoleBridge`), the `udp-requests` worker and discovery; the LumaSession stream, the
  election threads and signals stay on the shell lane with the shells (their order matters and
  their own threads bound them). The machine lane holds 256 jobs; full → `CallError::Full`
  (E-CMD-017, HTTP 503), never run. The state thread serves the shell lane first. Health: the
  State row's `queue` is everything waiting (both lanes + OSC's channel); every thread row has
  `refused` (the machine lane, the command port's 64, OSC's 256).
- **Threads stay one per concern** (§26 "Step 2 details"; no tokio outside HTTP, the DMX-only
  build stays possible). Fixes: the election's one-off threads get names and Health entries
  (`spawn_named`); the UDP command port's receive thread never waits on the state thread — it
  answers `Time` pings itself and hands every other request to one `udp-requests` worker
  (bounded at 64 waiting; beyond that the port answers E-CMD-017 at once), which runs them in
  arrival order on the state thread, so a ping never queues behind a Health or Login; a reply
  over 65,507 bytes is refused with E-WIRE-002, never silently dropped (§21: UDP keeps small
  messages). Health still runs on the state thread (it notes the check in the System Monitor).

**Code structure**

- **Replies are typed inside, text at the edge** (amends §23 "Command output stays text"):
  what reaches every client stays the same text, like console command-line feedback; inside, a
  handler's reply carries ok / error and its code, and the text is rendered at the transport.
  Dispatch, the journal, `set_rows`, sysmon and the TUI read the type, never the text
  (`journal_result` stops parsing `Error:` and `[E-…]`). Step by step: first a tidy check that
  every code literal is in a catalog, then the reply type; issue 03's code placement applies to
  the rendered text.
- **Flags are declared and a line is parsed once — one project, now** (before the command set
  grows). `CommandDef` / `ObjectDef` declare their flags (`/Force`, `/Keep`, `/NoConfirmation`,
  … with value or not, rights, doc); completion, help, `gen-docs` and the sweep read them; an
  unknown flag is refused in dispatch. Each line is lexed once into statements (word resolved,
  args, flags, spans) that rights, scope, reach, undo and the handler all read; the name lookup
  is one index built at startup, keeping §15's prefix and tie rules. Handler signatures change;
  the duplicate flag helpers go.
  **Asked 2026-10-14** (a reference console probed through its API: `/nc` and `/NoConfirmation` are
  accepted on any line; an unknown option is silently ignored — `Store Group 9999 /Foo` stores,
  `Delete Group 9999 /Foo` deletes — so a typo like `/Overwite` changes nothing visible):
  - **Global flags** valid on every line: `/NoConfirmation` (`/nc`), `/NoOops` (`/NU`), `/Verbose`
    (`/v`), `/Debug` (`/d`), `/Watch` (`/Watch=ms`); a word's own flag wins (`Tree /d` = depth).
  - **Any other unknown flag is refused** with the new **E-CMD-018** ("'/Foo' isn't an option of
    Store Group — it takes /Merge, /Overwrite, /Remove, /GridMerge"), for every statement before
    any runs (like rights), before forwarding to the master, the Oops ask and the Delete ask, and
    when a key / macro line is saved — better than the reference, which ignores it.
  - **Handlers: hybrid, then migrate.** One mechanical change gives handlers an `Input` that still
    reads as today's `&[&str]` (output byte-identical) plus typed `flag(&FORCE)` / `value(&PORT)`;
    then each family moves over and its hand helpers go (one commit per family).
    *Asked again 2026-10-14 (step 6): no signature change* — ~150 named handlers and their test
    calls would move. Handlers read typed flags from their own args (`flags::has(args, &FORCE)`,
    `value`, `rest_value`, `positional`): not from a statement kept on AppState, because handlers
    call each other with args they build (`store_group(state, &[raw, "/Merge"])`), where the
    outer line's statement would be wrong.
  - **A flag takes the next word only when declared with a value** (`Store Input … /Protocol OSC
    /Port 8000`); today's "every `/x` swallows the next word" in `store_args` goes.
  - A quoted `"/x"` is always a value (§15 Quoting) — also inside `a; b` (today the multi-statement
    rights path treats it as a flag), and a bare `nc` is never `/nc` (four handlers accept it today).
  - **Built 2026-10-14:** `core::commands::flags` (FlagDef, the globals, `has` / `value` /
    `rest_value` / `positional` / `without`), `core::commands::line` (Line / Statement), the name
    index, E-CMD-018, help Options, the reference, completion after `/`, the sweep; no `/word`
    string comparison left in `core/` outside `flags.rs`.
- **Tree paths are typed**: one `TreePath` built from `TreeDef` / `ObjectDef.tree`; navigation,
  the VFS, `cd` / `ls` and their aliases read one table instead of comparing
  `"root.showdata…"` strings in ~10 files (supports §15 "no string match").
  Plan (2026-10-14): `core/tree_path.rs` holds one `Node` enum whose exhaustive `def()` gives
  each node its name, parent, aliases, kind (static, reserved, a `TreeDef` collection, live
  children, a numbered schema such as `DataPools.<n>`, the disk), reach, scope, claim, REPL
  context, ls meta and List renderer — children are derived from the parent links, so two
  tables can't disagree. `TreePath` = steps (node, numbered entry, disk name); one parser for
  `/`, `Root.`, `..`, `.` and `/` separators. cd / enter keep their lenient match (exact,
  prefix, substring); List, Set and selections match exactly. Aliases (`patch`, `claims`)
  resolve the same in cd, enter and ls (today only some take them); List still refuses them.
  A new ShowData field fails until placed: `ShowSubtree::node()` is exhaustive (`None` needs a
  reason), and tests check every subtree and every `TreeDef` is reachable and only Drives
  reaches the disk. Built behind a golden test of every path's output; the only intended
  differences are the bugs listed in the roadmap.
  Built (2026-10-14, steps 0–7): `core/tree_path.rs` (`Node`, `Kind` Static / Collection /
  Live / Numbered / Schema / Disk, `TreePath`, `ALIASES`, `dot` / `slashed` / `names_drive` /
  `strip_root`); `TreeDef.node`; the VFS, ls metas, claims, contexts, List, selections, scope
  and completion match on `Node`; cd / enter / ls share `AppState::resolve_target`; the
  golden record is `crates/lumacore-station/testdata/tree_paths.golden.txt`
  (`core/tree_path_golden_tests.rs`, `LUMACORE_BLESS=1`); tidy `tree_paths_are_typed`.
  `tree` passing Drives from above never reads it; a `tree` inside Drives runs on the state
  thread, so it stops after 300 ms or 2,000 lines and says so (was ~6 s at depth 5).
- **Registry entries state only what differs**: `const CommandDef::BASE` + `..CommandDef::BASE`
  (and `..Doc::BASE`); every entry still names `verb`, `doc`, `run` and the four that decide what a
  line may do and where — `undo`, `scope`, `rights`, `reach` — checked by tidy
  `every_command_states_what_decides_it` (struct update can't require a field; BASE holds the
  strictest values for them). Still one `COMMAND_REGISTRY` (§15). Built: 888 default lines gone.
- **Later:** a `Walk` derive — only if a field without `#[walk(pool|link|data)]` fails to
  compile (§21 "6d": classification stays compile-time). **Gradually:** `AppState`'s fields
  grouped into sub-structs, one area when it is next worked on (session link and change
  bookkeeping first); every field keeps its one home (§10).
- **Crates: §23's condition stands.** Build profiles first, then measure (`cargo build
  --timings`); if builds still hurt, the sweep and the terminal become leaf crates. The small
  data crates (effects, timecode, playback, session) stay — they will grow; `lumacore-tree`'s
  one-line re-export shims go.

**Build, checks, docs**

- **Build profiles** (a 3 GB box; `target/debug` had grown to 60 GB): `[profile.dev] debug =
  "line-tables-only"`, dependencies at `opt-level = 1` without debug info; `.cargo/config.toml`
  with `jobs = 2` and a fast linker (lld, mold where installed); `test = false` on the station's
  binaries; unused dependencies removed (checked with `cargo machete`). (`zip` keeps its default
  features: `gdtf` asks for them, so deflate-only would save nothing.)
- **Checks stay local** (§25 "No CI" stands): `[workspace.lints]` (`unsafe_code = "forbid"`, a
  short clippy list), a pinned `rust-toolchain.toml` (stable; nightly only in `fuzz/`), one
  `./scripts/unix/test check` (fmt, clippy with warnings as errors, every crate's tests),
  optionally a pre-push hook.
- **`CLAUDE.md` is trimmed** to the core invariants and links (Anthropic: under ~200 lines);
  area rules (UI kit, render thread, backups, …) move to path-scoped `.claude/rules/` files.
  `decisions.md` stays one file (§29).
- **Git hygiene** (§26 "Cleanup" again): `library/shows/.autosave/`, `.versions/`, `.trash/`
  are ignored; the committed autosave and `NewShow_*` files are untracked; `demo.luma` stays.

**Clients**

- **Desktop app navigation:** the webview may load only `lumacore://localhost` and the
  connected station; anything else opens in the system browser, new windows are refused (the
  login token rides an initialization script that runs on every page; Electron's checklist,
  Tauri). IPC messages become one typed enum; the app's HTTP replies are size-capped. §19's web
  origin checks for the station stay open.
  *IPC built 2026-10-14:* `apps/lumacore-app/src/ipc.rs` (`Ipc`, tagged by `type`: connect
  page, station screen, overview); a message that doesn't parse is ignored and logged at debug.
- **CLI:** end of stream is a disconnect (it hung forever when the station died); the terminal
  is restored on panic; sentinel strings become an enum.
- **Frontend: not now.** The findings (`<luma-grid>` rebuilds rows with `innerHTML` against §30's
  "updated in place by id"; Realtime graphs force a layout per graph per frame; the 242 KB
  `index.html` script; 18 copies of `esc`; no `@ts-check`) are on the roadmap's UI leftovers.

**OSC line users** (asked 2026-10-14; refines §11 "an Input line names a user" and §30 "12d"):

- Every OSC line names the user that receives its commands: `/User` is required when it is
  stored (E-RIGHTS-003), a line with no user or a gone user doesn't start (state "no user").
- **Any user may be a line's user, Admin too** — chosen explicitly, within the storer's own
  rights level (only an Admin can name Admin). "Never Admin" means never as a fallback.
- **The line always runs**: its user doesn't need to be logged in anywhere.
- **Commands use that user's own programmer and selection** (one user, one programmer, §21),
  shared with anyone logged in as that user.
- **Deleting a user a line names asks first and names the lines** ("used by Input 3 (OSC)");
  going on stops the line ("no user") and the broken link shows in `List Reference /Broken`.
  *Built 2026-10-14:* the one-user question has an "Input lines" row; deleting several users
  names it per user (`RowDelete::affects`); the reply says which lines stopped.

**Not asked yet** (on the roadmap as open): DMX-path speed-ups found by the review — the merge
keyed by attribute index instead of `(Uuid, String)`, reusing retired frame buffers (changes
§24's "drops `Retired` … frames" wording), a structure-of-arrays tick layout, bounded per-member
stream queues (§25 "No input size caps"), socket buffer sizes / DSCP / multicast loop, input on a
blocking receive.


## 32. Binary study hardening (2026-10-14)

Source: an earlier study of a reference console's behaviour (object model, documented scripting
API, the manual) read against this repo; findings checked in code, console
behaviour probed on the live desk (scratch Sequence 99, removed after). Asked one at a time.

**Playback objects and the Playbacks registry** (restates §1 / §13 / §28; the code drifted):

- **Sequences, Effects and Executors are DataPool objects** (§1 tree). Today they sit in one
  global `PlaybackData` saved as `Playback.json` — that is the drift to fix.
- **`ShowData.Playbacks` is the registry of what runs** — the playback engine's one source of
  truth. **Per engine kind** (a category per kind, so later engines add one), split at the top
  by what is saved:
  ```
  Playbacks
    Live          (saved; resumed snapped on load, §13)
      Sequences.{guid}
      Effects.{guid}
    Echoes        (replicated, never saved; a load starts with none)
      Sequences.{guid}[]
      Effects.{guid}[]
  ```
- **One live run per object**: a sequence or an effect is never "running twice" as a
  controllable playback.

**Every sequence has CueZero and OffCue** (as established: created with every sequence, probed):

- Fixed parts of every Sequence (a type invariant, not pool entries): can't be deleted, moved
  or renumbered; Copy / Store / import keep them; load refuses a sequence without them.
- CueZero holds the start state (defaults per a CueZeroMode), OffCue the off timing + release.
- Nothing hardcodes "cue 1": starting from the top means CueZero, then the first cue.

**Top = the echo multi-caller** (new; the reference has no echoes — probed: Top there jumps the one
playback to the first cue):

- `Top On Sequence 3` / `Top Off Sequence 3` (and `Effect 5`); a bare `Top` is a usage error.
  A held executor button / pool object sends On at press, Off at release.
- `Top On` starts an **echo**: a run from CueZero → the first cue with cue timing, follow cues
  advancing; the live playback is untouched (still takes Go / Off). `Top Off` = that echo's
  OffCue with its timing; it is gone at 0. Several holds = several overlapping echoes (the
  sparkle ramp-down).
- Echoes are not controllable playbacks: no Go / Goto / fader; Off on the object fades them
  with the live run, Kill cuts them.
- Any sequence and any effect can be Top'ed; no mode setting. Limits on the object:
  `MaxEchoes` (default 8; 0 = Top refused) — **a full list refuses the newest `Top On`, never
  drops an old echo**; `EchoMax` (default 30 s) starts the OffCue of an echo still running;
  a holder that goes away (client gone, device locked, logout, session loss) = `Top Off` at once.
- **Temp** (probed): runs the sequence itself with its timings as a normal call; release
  = Off with off timing. **Flash**: the first cue after CueZero with no timing, master
  at full while held; release restores the state before.

**Effect targets** (refines §28 "The effect stores no fixture list" — replaced):

- **Fixtures live inside the Effect, per line**: a line's target is either a **raw fixture
  selection** (order + grid spot / rotation / mirror — the same members type and code as a
  Group) or a **pointer to a Group** (GUID, live). Per-fixture Lo/Hi and Disable stay sparse
  overrides on the line (§28).
- Set by `Assign Fixture 1 Thru 5 At Effect 5.2`, `Assign Selection At Effect 5.2`,
  `Assign Group 3 At Effect 5.2`; each replaces the line's target.
- **Layout per line**: `Grid` (the target's grid) · `Order` (stored order) · `FID` (by fixture
  ID); the effect holds the default the lines inherit. Editing an effect line's order / grid is
  the group editor (one window, UI later).
- **A cue stores the call** (effect, size, Rate, timing, pause) — not fixtures.
- **Words**: `Effect 5` selects its fixtures, like `Group 5`; `Go Effect 5` starts it as a
  programmer call (§28's anchor rules unchanged); `Off Effect 5`; `Top On|Off Effect 5`. Tap
  once / tap again (select, then start) is the effect pool window's tap action.
- **Idea, not now**: effect cues — an Effect with its own CueZero / cues (Go, Follow, basic
  timing) / OffCue, each cue holding layered lines (roadmap Ideas).

## 33. Lum syntax crate hardening (2026-10-14)

Why: the 2026-10-14 code review found the command language read in many ways — five range
parsers, about seven `Set` routes, three relative-value parsers, about ten bool parsers,
handlers matching raw tokens (`Run = fn(&mut AppState, &[&str])`), the line re-lexed 4+ times
per dispatch, prefix expansion only on the first word, quote rules not kept. The fix is one
grammar, one parser, one typed statement.

**Decided (asked one at a time):**

1. **Our own grammar.** Established consoles are loose inspiration, not the rule: its irregular parts (a
   different default function per object, `At`'s three roles, space-sensitive minus, prefix
   expansion of names, dot as both decimal and hierarchy, Thru reusing the last range) are not
   copied.
2. **`docs/user/syntax/lum.md` is the binding spec.** It keeps its teaching text and gains one
   fenced `grammar` block; the reference (generated from the registries) lists words, lum.md
   says how they combine. **Changing the syntax or lum.md is a hard discussion of its own**
   (alternatives, what it breaks, which edge case it adds), decided before lum.md changes, and
   lum.md changes before the code.
3. **Enforcement:** the parser crate prints its own grammar; a test fails if it differs from
   lum.md's `grammar` block. Every ` ```lum ` example in lum.md must parse; an example marked as
   an error must fail with its E-code. Handlers never see the raw line.
4. **Handlers get a derived typed struct** per verb × object (`#[derive(Args)]` in a
   proc-macro crate): the struct is the signature; parse, completion, hints, help, the generated
   reference, rights and the sweep read it; a duplicate flag or two forms that can't be told
   apart is a compile error. Static `CommandDef` / `ObjectDef` tables stay (`run: typed::<C>`).
5. **The parser crate is hand-written with no parser dependencies** (`lumacore-syntax`; its one
   dependency is `lumacore-base` for the shared error-catalog type `ErrorInfo`, asked 2026-10-15): a lexer and a
   recursive-descent parser driven by a runtime vocabulary built from the registries; total —
   never fails, never panics, records coded diagnostics with spans (`E-SYN-…`) and what it would
   accept at each point (completion reads that, no second grammar); `parse`, `at_cursor`,
   `highlight`, `print` (the canonical line for forwarding, keys, macros, journal),
   `grammar_listing` (golden). Like nushell and rust-analyzer.

6. **A bare object = its type's default verb.** Every object type declares one `default_verb`
   on its `ObjectDef` (a registered verb: `Group` → `SelectFixtures` (decision 26), `Blind` → `Toggle`, …);
   `Group 3` is exactly `SelectFixtures Group 3` — no separate Call handler, no special parse. The default lives
   only in code; changing one is a lum.md hard discussion. `print` writes the verb out.

7. **Quoted = text, never a keyword; property names are always quoted**:
   `Set Fixture 3 "Name" "Spot SR"`, `Set Input 1 "Network" "Rig"`. A bare keyword after an
   object only continues the path (`DataPool 2 Group 3`). Whether a quoted word is a name, a
   value or a property name comes only from the verb's declared signature (`Set` = target,
   "property", value; `Label` = target, "name") — never guessed. A bare property name is
   refused with a hint showing the quoted form (one spelling); every existing `Set` line (docs,
   keys, demo macros, tests, UI line templates) is rewritten once.

8. **`At` is both value and destination; the verb's signature decides** (established typing kept):
   `Fixture 1 Thru 4 At 50`, `Master 2.1 At 80` (value) · `Copy Group 1 At 2`,
   `Move Fixture 1 Thru 4 At 10`, `Assign Universe 1 At Output 1` (destination). A signature
   declares its `At` slot as `Value` or `Destination`, never both; nothing is guessed from what
   follows.

9. **Owner first only.** A nested object is written in tree-path order: `Goto Sequence 2 Cue 3`,
   `Store Sequence 4 Cue 2`, `List Sequence 4 Cue`, `Clear User 3 Programmer`,
   `Master Group 3 At 50`, `Park Fixture 1 Attribute "Pan" At 30`; a child alone uses your
   selected owner (`Cue 3` = your selected sequence). Child-first is refused with a hint showing
   the owner-first form. Probed on a reference console (2026-10-14, read only): it nests only
   owner first too (`Sequence 1 Cue 2`, `DataPool 1 Sequence 1`); its child-first `Cue 2
   Sequence 1` is a list of two objects and `Group 1 DataPool 1` silently keeps the last — not
   copied: a keyword after a path that is neither a declared child nor a slot of the verb's
   signature is refused with a code. A word naming another object kind inside a line
   (`Park Fixture 1 Attribute "Pan" At 30`, `Screenshot Screen 1 User "LD"`) is a declared
   helper slot of that signature, never a path.

10. **An id is a number or quoted text — nothing else.** A number is digits with optional dots
    (`Fixture 3`, `Cue 1.5`, `Master 2.1`, `DMX 1.001`, `Window 1.4`); the object declares how
    many parts its numbers take (a wrong count is refused with a code). Everything else is
    quoted: named members (`Master "Grand"`, `Master "Highlight"`, `Cue "CueZero"`), names
    (`Group "Front"`, `Interface "en1"`, `Network "Rig"`), codes (`Info Error "E-PATCH-002"`),
    hosts (`Join LumaSession "10.0.0.5:47653"`), stamps (`Set Version "20261013-113000"`). A
    dot never means nesting. No wildcards (`Delete Window 2.1 Thru 2.9`, not `2.*`).

11. **Operators: a sign glued to digits is part of the number; a sign standing alone is an
    operator** (one lexer rule; the established manual rule): `Attribute "Pan" At -120` absolute ·
    `At - 10`, `At + 10`, `At * 2`, `At / 2` relative · `Group 1 + 3 - 2` set operators ·
    `+ Group 3`, `- Fixture 5` at the start add to / take out of the selection · `/` followed by
    a letter is an option. What an operator means comes from its slot in the signature. Stricter
    than the reference (probed 2026-10-15: it reads `Fixture 1+3` as 1 and `1 Thru 5-2` as 1–5 —
    silently): a glued operator inside a list (`1+3`, `5-2`, `1-3`) is refused with a hint to
    space it; `At +` / `At -` with no number is refused (a "+1" not copied); `*` is only
    multiply (no wildcard, no "all"). A range keeps its typed order (`5 Thru 2` = 5, 4, 3, 2).

12. **One name per word; typing is fuzzy, lines are exact.**
    - **No built-in short forms**: every registry alias goes (`Seq`, `S`, `Gr`, `Profile`,
      `Colour`, `Plus` / `Minus`, …); each verb and object has exactly one name. (`+` / `-` stay
      operators, `?` stays Help's hint.) Existing lines (keys, macros, demo, docs, tests) are
      rewritten once to full words.
    - **No prefix guessing in a line**: a line names words exactly; `full_words` expansion goes.
      A new word never changes what an old line means.
    - **Typing is fuzzy, Raycast-like** (the suggestion engine, `core/suggest`, nucleo): while a
      line is typed each word is matched fuzzily and the full line shows as ghost text
      (`str gro 3` → `Store Group 3`). **Enter runs the line shown**: when every word matched
      uniquely it is expanded on the typing station and that full line is what runs, is
      forwarded, journalled and kept in history; an ambiguous or unknown word (`cu 2` → Cue 2 |
      CurrentUser 2) shows the choices and Enter is refused with a code. Never in stored or
      machine lines (keys, macros, cues, OSC, HTTP, UDP): those are exact (`"str gro 3"` in a key
      is refused on save). The profile's `Suggestions Off` means exact typed input. Replaces
      "Enter runs the line as typed" (§30 "12b2").
    - Why (probed live, 2026-10-15): the reference is prefix + hand-picked shortcuts — `Cu 2` gives
      CurrentUser, `Se 1` SpecialExecutor, `Fix 1` nothing (another keyword), no fuzzy or typo
      tolerance; its 2.3 notes fix shortcut clashes by hand ("typing C now enters Cue"). Its
      release notes up to 2.5 add no fuzzy input.
13. **User aliases: personal, typed only, marked with `!`.** Each user's profile holds aliases
    (a word → a command line; Unix-like; "alias" means only this — the registry short forms are gone, decision 12): `Store Alias "St" "Store Cue 1"`, then `!St` runs
    `Store Cue 1`; `List Alias` shows yours, `Delete Alias "St"` removes one — `Alias` is an object
    with the normal verbs (asked again 2026-10-15; no `Alias` / `Unalias` verbs). The reference has nothing like it (only Macros and `$`
    variables). Safeguards:
    - expands **only in a line typed on a command line** (terminal, TUI, a page's command bar) —
      never in macro / key / cue lines, OSC, HTTP, the UDP command port or any stored or machine
      line;
    - expanded on the typing station before anything else; the line that is forwarded, journalled
      and filed in Oops is the expanded one written out (`print`) — nothing downstream sees an
      alias;
    - **words typed after an alias are appended** (Unix): `Store Alias "sg" "Store Group"`, then
      `!sg 3 "Front"` → `Store Group 3 "Front"`, `!St /Merge` → `Store Cue 1 /Merge`; no
      placeholders (`$` stays for variables). An alias line with `;` gets the words appended to
      its last statement; an alias line can't name another alias (no `!` inside, checked when it
      is set), and every word in it must be an exact word (checked when it is set; the full line is
      parsed when it runs, with the typed words appended);
    - the `!` marker keeps aliases apart from every built-in word: no clash now or after a new
      word is added (`St` alone is just an unknown word). `@` stays free (At for keypad users),
      `$` stays for variables (§5).

14. **One on/off value kind, parsed once:** `On` / `Off` / `Yes` / `No`, plus `Toggle` where the
    property declares it (`Set Output 1 "Sync" On`, `Set Recovery "x" "Pinned" Yes`,
    `Set Self "Maximized" Toggle`); anything else (`1`, `true`, `pinned`) is refused naming the
    accepted words; `print` writes `On` / `Off`. The ~10 handler-local bool lists go. `On` / `Off`
    at the head of a statement stay verbs (`On Blind`, `Off Sequence 1`) — position, from the
    signature, tells verb from value.

15. **Hold verbs take a press / release slot right after the verb** (the established form for Flash,
    Temp, Swap): `Top On Sequence 3` / `Top Off Sequence 3`, `Flash On Sequence 1` /
    `Flash Off Sequence 1`; a hold verb declares the slot (one rule for the family); the bare form
    (`Flash Sequence 1`) is refused naming `On|Off`. Noted, not decided here: the reference `Top` means
    "jump to the first cue" (`Top Executor 105`), not an echo — a §32 naming question.

16. **`Set` always names its target; a bare owner keyword = your own / this one** (the owner
    rule): `Set UserProfile "ClearBuffer" On` (yours), `Set UserProfile "LD" "ClearBuffer" On`,
    `Set Station "Name" "Luma01"` (this station), `Set Station "Tower" "Priority" High`,
    `Set Interface "en1" "HTTP" No`. Today's target-less `Set ClearBuffer On` goes; one `Set`
    shape (the ~8 `handle_set` routes become one).
17. **Trailing slots bind from the right.** A signature's trailing slots (value, then the quoted
    property before it; Label's name) are filled from the end of the statement; the ids take what
    is left between the object keyword and them — so a quoted id never collides with a quoted
    property (`Set UserProfile "ClearBuffer" On` = your profile's ClearBuffer even if a profile
    named "ClearBuffer" exists). A value is one expression (`21 Thru 30`, `* 2`, `"Rig"`).
    Every `Set` has a value: the value-less password ask becomes `Set User "LD" "Password" Ask`
    (`Ask` a declared value word of password properties).

18. **The tree shell is a fenced Unix sub-grammar, decided by the word.** These words always
    parse with Unix rules for their statement (`-n 5`, `--type T`, `--all`, `..`, `/` = root):
    `cd`, `pwd`, `ls`, `cat`, `tree`, `find`, `grep`, `head`, `tail`, `wc`, `du`, `stat`, `diff`;
    every other word is Lum. `root` (= `cd /`), `rm` (= Delete), `whoami` (= Info User), `env`
    and the `history` stub go. **Allowed in any user-written line** — typed, macros, keys,
    commands in sequence cues — **never in LumaCore's own code**: Rust handlers, registry lines,
    the UI kit's line templates and menu definitions, generated lines, the demo-show builder
    (tidy rule + a test that every line the code emits parses as pure Lum). The lum.md grammar
    block specifies both parts; the Unix part is the fence's own small grammar.

19. **Option values: `/Name value`** — the next single word or quoted text, no `=`
    (`/Port 8000`, `/Tab "Lines In"`, `/Watch 500`, `/Note "before the show"`); multi-word values
    are quoted (the "Rest" form goes); whether an option takes a value is declared on it, so
    `/Overwrite 3` never takes the 3.
20. **One list of things a statement acts on; every extra carries a `/label`.** Files, sources
    and descriptions are options; an embedded command line is quoted text, parsed as its own line
    when used: `Import Patch + Users /From "Festival"`, `Import Sequence 1 + 3 + Group 2 /From
    "x"`, `Bind "F5" "List Fixture" /Description "Lists all fixtures"`,
    `Watch "List Fixture" /Every 500`, `Import Asset /Root "root" /File "path" /Name "name"`.
    No verb keeps its own positional sub-shape.

21. **Default object** (keeps issue 02's decided design, `issues/README.md` 2026-10-13; the established
    `Fixture Please`): an object keyword **alone** (`Group`, nothing after it) sets your default
    object — saved per user (`ShowData.Users`), shown in the prompt (`Group>`); a **bare number**
    uses it (`3 At 50` = `Group 3 At 50`; default `Fixture`); an object keyword **with ids** is
    its type's default verb (decision 6). A typed bare number is written out in full before it
    runs, is forwarded or journalled; in a stored line (macro, key, cue command) a bare number is
    refused on save — it can't know whose default it will run with. Profile start place and
    `cd` alone as in issue 02.

22. **Routing names the line, the network universe is `/Universe`** (the protocols' own word —
    sACN "universe", Art-Net "Port-Address" Net.SubNet.Universe): `Assign Universe 1 At Output 1
    /Universe 101`, `Assign Universe 1 Thru 6 At Output 1 /Universe 101` (101 … 106),
    `Assign Universe 2 At Output 3 /Universe 0.1.2` (Art-Net), `Assign Universe 5 At Input 1
    /Universe 7`, `Unassign Universe 1 At Output 1`. The protocol shortcuts (`At sACN 5`,
    `At ArtNet 0.1.2`, `Net … Subnet … Universe …`) go: the line knows its protocol.

23. **`If` has exactly the established three shapes**; planned until
    built (replies "planned", E-CMD-003): (1) verb at the start — `If Group 5` deselects what is
    not in group 5, `Group 3 If Group 5` keeps the overlap; (2) a filter clause at the end of a
    statement — `Delete Cue 3 If Fixture 4`, `Delete Cue 3 If Fixture 4 Attribute "Pan"`,
    `On FeatureGroup 2 If Filter 42`, `Label Group Thru If Appearance 20`, Clone's scope
    `Clone Fixture 1 At Fixture 2 If Sequence 1`; (3) `EndIf` for an If in the middle —
    `Store If Group 5 EndIf Preset 1.1` runs as `Store Preset 1.1 If Group 5`; `print` writes
    the If at the end (one spelling). No comparisons: today's half-built `> < = ==`
    filter in the selection parser goes.

24. **Words ignore case, text keeps it.** Keywords, enum values and on/off words are
    case-insensitive (`store group 3` = `Store Group 3`, `"MergeMode" htp` = `HTP`); quoted text
    is stored as typed; a lookup by name ignores case (`Group "front"` finds "Front"); two objects
    of one kind whose names differ only in case are refused at the second; `print` writes
    canonical case. (Case-sensitive values are not copied.)

25. **A verb's own default object comes first** (keeps §30 "Store Cue"): a bare number
    after a verb uses the verb's declared `default_object` (Store → Cue: `Store 2` = `Store Cue 2`,
    `Store` = the next cue), else your default object (decision 21), else `Fixture`
    (`3 At 50` = `Fixture 3 At 50`; `Group>` → `3 At 50` = `Group 3 At 50`).

26. **Two verbs: `Select` and `SelectFixtures`** (the established Select / SelFix split). `Select` makes
    an object your selected one (DataPool, Sequence, Timecode — §29 "11b" unchanged);
    `SelectFixtures` selects fixtures (Fixture, Group, Cue, Preset; through
    `Programmer::apply_selection`). Default verbs follow: `Group 3` = `SelectFixtures Group 3`,
    `Fixture 1 Thru 4` = `SelectFixtures Fixture 1 Thru 4`, `Sequence 3` = `Select Sequence 3`;
    `Select Group 3` is refused naming `SelectFixtures`.

27. **A keyword alone: counted objects set the default, one-of-a-kind objects act.** An object
    that takes ids (Fixture, Group, Cue, Universe, …) typed alone sets your default object
    (`Group` → `Group>`, decision 21); an object without ids (Blind, Highlight, Freeze, Solo,
    Clock, …) typed alone runs its default verb (`Blind` = `Toggle Blind`, §16 kept). The
    object's declaration says which kind it is — nothing guessed.

28. **The cat names become ready-made personal aliases.** The registry keeps one name (`Oops`,
    `Shutdown`); every new user profile starts with the aliases `hiss` → `Oops`, `unmeow` → `Oops`,
    `turnoff` → `Shutdown` (typed `!hiss`); a bare `hiss` is unknown and the hint names
    `!hiss`. `hissme` stays its own verb.

29. **Watching is the `Watch` verb only:** `Watch "List Fixture" /Every 500` (the line quoted,
    decision 20), `Unwatch 2`; the global `/Watch` option goes (`Clock /Watch`, `List … /Watch`),
    and so does the object form `Watch Clock` (→ `Watch "Info Clock"`). The tree word's depth is
    inside the Unix fence (`tree -d 2`); the global `/Debug` keeps its meaning everywhere else.

30. **NAV keys are profile key data.** Every new profile's NAV UiKeys carry `cd ..` (Press) and
    `cd /` (Hold) as editable key lines; the defaults — like the cat aliases (decision 28) — come
    from a shipped default-profile **data file**, not from strings in Rust or JS, so decision 18
    holds with no exception (the code never emits a Unix-style line; the tidy rule checks code).

31. **A `Root.` path is a Lum target — the one place a dot nests.** A word starting with `Root.`
    is a tree path, resolved through the typed tree (`core::tree_path`): `Select
    Root.ShowData.DataPools.2`, `Set Root.ShowData.Protocols.Outputs.1 "Priority" 120`. Anywhere
    else a dot only joins the parts of a number (decision 10). `print` writes the path in canonical
    case. The written exception to decision 10.

32. **Reads are part of Lum now: `Self.<Property>` fills a value slot** (`Set Group 3 "Name"
    Self.Name`, `Set Self "Title" Self.DataPool.Name`), resolved when the line runs on the station
    where it starts (12c). A read is one dotted word: a word starting with `Root.` or `Self.` is a
    path or a read, the **only** place a dot nests (widens decision 31); the property names after
    the dot are bare. Reads start from `Self` only. **Declared conditions** — a window-bar item's
    `state` / `shown_when` in the registry — say "not" with the word **`Not`** (`shown_when:
    "Not Self.Filter"` = show the chip only while no filter is set); `!` means only an alias
    (decision 13), nowhere else (today's `!Self.X` conditions are rewritten). Title templates (`"U {Self.Universe}"`) use the same
    read.

33. **Built in parts on one branch, merged when done** (like 12d / 12f): (1) the
    `lumacore-syntax` crate + lum.md's `grammar` block and its tests; (2) the `#[derive(Args)]`
    boundary + a temporary shim for unmoved handlers; (3) handlers moved area by area (a tidy
    counter of shim uses may only go down); (4) every line rewritten — docs, keys, macros, the
    demo, UI templates, menu_defs, tests (format bump); (5) the old lexer, `parse_selection`, the
    bool lists, `full_words`, `tokens::unquote` copies gone. main never has two parsers: the
    branch merges when the shim count is 0. Needs an explicit go.

34. **The parser owns `E-SYN-…`.** Everything refused before a handler runs gets an E-SYN code
    with its span (caret under the word): unknown word (with the fuzzy "did you mean"), missing
    slot, glued operator, child-first, unquoted property, bare number in a stored line, unknown
    option, Unix word in code. E-CMD-005 / 006 / 018 retire into E-SYN; errors that need the show
    stay (E-CMD-007 no such object, 008 value out of range, 010 not possible now, E-RIGHTS-…).

35. **`Call` is a registered verb: bring a stored thing back here** (the established word; decision 6 only
    forbids `Call` as a per-object catch-all): `View 3` = `Call View 3` (onto a screen),
    `ViewButton 5` = `Call ViewButton 5`, `Call WindowPreset 2` (into the window), later
    `Call Preset 4.1` (onto the selection).
36. **`Preset 4.1` = `SelectFixtures Preset 4.1`**; applying is `At Preset 4.1` or
    `Call Preset 4.1`. TimingPreset / CurvePreset follow when they apply.
37. **`Sequence 3` = `Select Sequence 3`** (then a bare `Go` runs it); its cues are
    `List Sequence 3 Cue`.
38. **`Menu "Network"` = `Toggle Menu "Network"`**; `On` / `Off Menu "Backup" /Tab "History"` open /
    close explicitly; no new Open / Close verbs. The ⚙ button's line is `Toggle Menu`.
39. **Every counted object can be your default object** (widens decision 27; the reference allows only patch
    ID types — Fixture, Channel, Universal, … — manual 2.5 "Change the Default Keyword"): any
    object that takes ids typed alone sets it (`Group` → `Group>`, `Station` → `Station>`,
    `Menu` → `Menu>`); a bare number **or a bare quoted name** then names one of it
    (`Station>` `"Tower"` = `Info Station "Tower"`, `Menu>` `"Network"` = `Toggle Menu
    "Network"`), always written out in full before it runs (decision 21). One-of-a-kind objects
    typed alone run their default verb (`Blind` = `Toggle Blind`).
40. **Default verbs** (from each object's `Call` today, research 2026-10-15): Fixture, Group, Preset
    → `SelectFixtures` · DataPool, Timecode, Sequence → `Select` ·
    Cue → `Goto` · Macro → `Go` · View, ViewButton, WindowPreset → `Call` · Menu and every mode /
    device toggle (Blind, Highlight, Freeze, Solo, DeskLock, NormalTheme, Sidebars, ViewBar,
    FullScreen) → `Toggle` · Network, Output, Input, FixtureType, User, UserProfile, Image, Icon,
    AssetRoot, Asset, AttributeRule, LumaSession, Device, Clock, Station, Interface, Client,
    Shell, Error, UIKey, Reference, Version → `Info` · Selection, Claims, Undo, Journal, Library,
    Settings, KeyBindings, GlobalVar, GridColumns → `List` · Programmer → `Dump`. Then decided:
    Universe, Window / ScreenConfig, Master (41–43), Attribute / FeatureGroup / EncoderBar (44),
    Executor (45), Show (46), Recovery / Version (47) — complete.

41. **`Universe 3` = `Info Universe 3`**; creating is only `Store Universe 3` — a bare line never
    creates.
42. **`Window 1.4` = `Select Window 1.4`** (focus = your selected window), `ScreenConfig 2` =
    `Select ScreenConfig 2` (this client uses it), `Self` = `Select Self`.
43. **`<target> At <value>` is the `At` verb on that target** — one rule; `At` declares the effect
    per target kind; a default verb applies only when there is no `At`. Fixtures, groups, presets:
    **selected, then set** (as today): `Fixture 1 Thru 4 At 50` selects 1–4 and sets 50, a
    following `At 70` sets 1–4 to 70. Attribute: that attribute on your selection
    (`Attribute "Pan" At 45`). Master: its level (`Master 2.1 At 80`; bare `Master 2.1` =
    `Info Master 2.1`). `At 50` alone acts on your selection.

44. **`Attribute "Pan"`, `FeatureGroup "Position"`, `EncoderBar "B"` = `Select …`** — your
    selected attribute / feature group / encoder bar (the encoders follow); with `At` it is the
    `At` verb (decision 43).
45. **`Executor 201` = `Select Executor 201` = select what is assigned to it; the Master Section
    follows your selected sequence** (as established: an executor's default function is Select,
    "selects the object assigned to the executor"; "the selected sequence is the target for all
    sequence-related commands"; the master section's Go+ / Go- / Pause and master faders 1 / 2 act
    on the selected sequence). Each user has one selected sequence or none (§29 "11b"); selecting
    an executor makes its sequence the selected one, **an empty executor clears it** — the Master
    Section goes empty and a bare `Go` is refused (E-PLAY-003, never Sequence 1). `Sequence 3`
    (decision 37) sets the same selection: one target however it was chosen.

46. **Show:** an option never says what a line does (decision 20), so `Show`'s action / setting
    options move — `/Save` / `/Load` → `SaveShow "demo"` / `LoadShow "demo"`; `/Autosave`,
    `/Description`, `/Startup`, `/MergeDefault`, `/MergeTimeout` → `Set Show "Autosave" 5`, ….
    `Show` is counted (show files): typed alone it sets `Show>` (decision 39, no exception);
    `Show "demo"` = `Info Show "demo"`; `Info Show` = the loaded show.
47. **`Recovery "x"` / `Version "…"` = `Info …`** (the copy's facts); opening stays explicit
    (`LoadShow /Recovery "x"`, `LoadShow "demo" /Version "20261013-113000"`) — a bare line never
    replaces your show.

48. **`Patch` and `DMX` are objects only** (one name, one meaning). `Patch` = the patch:
    `Claim Patch`, `Unclaim Patch /Keep`, `Info Patch`. Patching is Assign:
    `Assign FixtureType "LED Par" At DMX 1.001 /Count 4 /Mode 2`; its pair `Unpatch 3` becomes
    `Unassign Fixture 3` (the fixture loses its address). `DMX` = a DMX address (`universe.address`):
    `DMX 1.001 At 255` (the `At` verb, decision 43), `Park DMX 1.001`, `Info DMX 1.001`. The
    `Patch` and `DMX` verbs go.

49. **Nesting has two forms, one job each — dots only in a tree path** (probed live,
    2026-10-15, changing destination from Root through the console's API): the reference accepts keyword nesting
    with spaces (`DataPool 1 Sequence 1` → Sequence 1, `Sequence 1 Cue 1` → cue 1,
    `Page 1 Executor 201` → Page 1.201) and a dotted path of **tree node names**
    (`ShowData.DataPools.1.Sequences.1` → Sequence 1); it refuses a dotted *keyword* path
    (`DataPool.1.Sequence.1`, `Sequence.1.Cue.1`, `Page.1.Executor.201` → Failed) and ignores
    spaced tree names (`ShowData DataPools` stays at ShowData). So:
    - **operator form = keywords, spaced, owner first** (decision 9): `DataPool 2 Group 3`,
      `Set Station "Tower" + "FOH" Network "Rig" "Interface" "en1"` (a station's wiring row:
      `Network` is a declared child of a station in the tree);
    - **tree form = node names, dotted, from `Root.`** (decision 31): `Root.ShowData.DataPools.2.
      Groups.3`, `Set Root.ShowData.Stations."Tower".Networks."Rig" "Interface" "en1"`; a part is
      a node name as stored (plural collections), a number, or quoted text (the only place quotes
      sit inside a word); we keep the leading `Root.` (the reference starts at the destination) so a path is
      never confused with a word;
    - not forms: a dotted keyword path (`DataPool.2.Group.3`), a mixed `DataPool 1.Sequence 1`,
      numeric child positions (`DataPool 1.7.1`, `Page 1.201`) — refused with a hint; `print`
      writes the operator form.

50. **Tree paths are relative to where you are; a part is the node's own name; typing is fuzzy**
    (probed inside ShowData: `DataPools.1.Sequences.1` and `datapools.1.sequences.1` → OK,
    `datapool.1.group.1` / `DataPool.1.Sequence.1` → Failed, a pool's name `DataPools.Default` →
    Illegal index). `cd DataPools.1.Groups.1` inside ShowData, `cd ..`, `cd 3`; `Root.` makes a
    path absolute from anywhere. A part is a node name as stored (plural collections), a number or
    quoted text — the object keyword (`datapool`) is not a path part (one name, decision 12). The
    fuzzy typing of decision 12 covers paths and fence lines too: `cd datapool.1.group.1` (or
    `cd dp.1.gr.1`) shows the ghost `cd DataPools.1.Groups.1` and Enter runs that; an ambiguous
    part (`s.1`) shows the choices; a stored line is exact (`cd datapool.1.group.1` in a macro is
    refused on save).

51. **`Go-` is the back verb's one name** (the established keyword; `goback` / `go-` go): a word may end
    with **one glued `+` or `-` when a space, `;` or the line's end follows** — the only sign a
    word carries (`Go-`, `Go- Sequence 2`); `Go` stays `Go`. Everywhere else decision 11 holds.
52. **`lumacore-shutoff` folds into `Shutdown`**: it is Shutdown with a confirmation, which is
    Shutdown's own station ask (`Shutdown /NoConfirmation` skips it). No word has a hyphen inside.

**Part 1 built (2026-10-15, branch `lum-syntax`, 9d820d8):** `crates/lumacore-syntax` — lexer, total
parser, `print`, `at_cursor`, `highlight`, `grammar_listing`, E-SYN catalog; lum.md's grammar block,
examples and error examples are tests; property tests and the fuzz target `syntax_parse`.

**Part 2 built (2026-10-15, branch `lum-syntax`):**
- **The vocabulary comes from the registries** (`core::lum`): verbs by their one name (no aliases,
  decision 12; the grammar's own words `Thru` / `If` / `EndIf` / `Self` are never registered),
  `CommandDef::hold` (Top, Flash) and `CommandDef::fence` (the 13 Unix words, lowercase, their flags
  as `-x` / `--name`), objects with `ObjectDef::lum` (`LumShape`: id parts, 0 = one of a kind, and
  the owner-first children as `&ObjectDef`s — `DataPool` → Group, Sequence, Preset, TimingPreset,
  CurvePreset; `Sequence` → Cue; `Master` → Group; `User` → Programmer; `UserProfile` → Icon, Image;
  `Station` → Network, Interface), the helpers `At` / `Fade` / `Delay`, value words (levels, on/off,
  `Ask`, every declared choice / named number / on-off word, the window types' names), property names
  (a name that is another word keeps that word), every flag as an option. Golden record
  `testdata/lum_words.golden.txt`.
- **lum.md runs against it** (`core::lum_tests`): the 15 lines whose words part 3 adds (`/Universe`,
  `/Count`, `/From`, `/Every`, the `Alias` and `Users` objects, the fence flags) are listed in
  `PENDING`; a listed line that parses fails the test until it is taken off. The crate keeps its
  own rule vocabulary for its parser tests until `PENDING` is empty.
- **Binding** (`lumacore_syntax::bind`): a `Signature` (target never / optional / required, plain
  slots Property / Name / Value, helpers with `At` as Value or Destination, options, phase, If);
  trailing plain slots bind from the right and take the target's last single id when short
  (decision 17); a verb without a target reads bare ids as its slots (`Redo 3`); unknown helpers /
  options, a value `At` where a destination is declared, a bare word where a property is declared
  are refused with E-SYN codes; `OnOff` / `Switch` / `u32` refuse with E-SYN-014.
- **`#[derive(Args)]`** (`crates/lumacore-syntax-derive`, a proc-macro so `lumacore-syntax` keeps
  one dependency): the struct is the signature; `Signature::check` runs at compile time (two optional
  plain slots, a helper or option declared twice — compile errors, tested as `compile_fail`).
- **The boundary** (`core::commands::typed`): `Command::run(self, &mut AppState)`, registered as
  `run: typed::<C>`; until dispatch hands over the parsed statement (part 3) `typed` writes the words
  back into a line, parses and binds it. `Redo [<steps>]` is the first typed verb (`Redo x` →
  E-SYN-001, `Redo 1.5` → E-SYN-014, `Redo 0` → E-CMD-008). The shim counter
  (`the_shim_count_only_goes_down`) is 447 and may only go down.

**Part 3a built (2026-10-15, branch `lum-syntax`): the tree-shell fence.** The 13 Unix words run as
`fenced::<C>` (`commands::typed::FenceCommand`: the word, how many paths it takes, its handler on a
`lumacore_syntax::FenceLine` — positional paths / texts and the declared flags). The flags are
declared on the words as Unix flags: `ls -r`, `tree -d 2`, `head` / `tail -n 5`, `find … --type T`,
`diff a b --all` (`ls /r`, `tree /d=2` go). A path is dotted (decision 49): a slash path
(`cd ShowData/Devices`) reads as several fence arguments and is refused with E-SYN-003 naming the
dotted form, never silently cut. `cd .` is refused (lum.md's `TREE-PATH` has `..` and `/`, not `.`).
`root` (= `cd /`), `rm` (= Delete), `whoami` (= `Info User`), `env` and `history` are gone; usage
errors are E-SYN-002. Found, left for 3f: `resolve_target` still prefix-matches node names in a
line (`cd show` → ShowData), which decision 50 gives to fuzzy typing only.

**Root (asked after part 3a, 2026-10-15; probed live).** On the reference: `Root` is an object
keyword (`Root [root-index].[sub-index]`); every node has a `No`, Root's 23
children are numbered 1–23 (`Root 3` = Interfaces, `Root 14` = ShowData); `ChangeDestination Root 3`,
`ChangeDestination 3` from Root, `ChangeDestination Root 2.1` work and `ChangeDestination Root
"ShowData"` lands on `Root 14`; `List Root 3` lists Interfaces; `Root` alone and `Root 3` alone reply
"Not implemented"; `Root` alone doesn't change the default keyword (only ID types and `Group` do —
`DEFAULTTOKEN` on the profile). The manual warns that root indexes change between releases.

53. **`Root` is a counted object** (decision 39 widened past the reference: anything can be your default
    object): `Root` alone sets `Root>`; its default verb is **List** — `Root> 3` = `List Root 3`,
    `Root 3` = `List Root 3` (a "Not implemented" not copied). `List Root` / `Info Root` name
    the top. Its ids are tree numbers with any number of parts (`Root 1.10`) or quoted names
    (`Root "ShowData"`).
54. **Every node has a number, given by its declaration order** — no number is written in the
    code (the tree-path / metanode declarations hand them out); a collection's children keep
    their member numbers (`DataPools.2` = datapool no. 2). One rule: **a number in a path is the
    child's `No`**. `Root 3`, `Root.3`, `Root.Resources` name one node; `cd 3` / `cd Root 1.5`
    work as a change of destination. Narrows decision 49 (numeric child positions are refused
    only where a node has no number) and decision 10 (Root's ids take any number of parts).
    **53–54 built (2026-10-18, round 3 step 0's finding):** `Node::no` / `nth_child`; a number in
    a path below a declared node (or a numbered entry's schema: `DataPools.1.2`) is its child's
    No, `cd` / `ls` included (`cd 1` under Devices = Dmx), `cd Root 1.5` = `cd Root.1.5`; the
    `Root` object (`objects/root.rs`: List / Info, default List, a `PathDef` of the declared
    tree, Root.Drives left to the tree shell) — so `Move Tag … At Root` runs.
55. **A stored line may hold tree numbers** (macro, key, cue command): accepted that after an update
    that declares a node in between, such a line may point at another node (the established behaviour; no
    refusal on save).
56. **`print` keeps a tree path's numbers or names as typed** — the echo is the execution
    confirmation: `List Root 1.10` stays numbers, `List Root.ShowData.Users` stays names. Only
    this: everything else stays canonical (case, spacing, fuzzy expansion, bare numbers written
    out, If moved — decisions 12, 21, 24), because ghosting and fuzzy matching are autocorrect while
    a number is a hard path.

**One resolver (asked after 3b1, 2026-10-15).** A survey found five resolver families with
different grammars (`parse_selection` + `resolve_parsed` for fixtures, `number_list` for numbered
maps without names, `set_rows::split` without `-` or open ranges and losing typed order,
`playback::parse_target` without names or ranges, per-object owner / address parsers) and about
20 hand-written "number or name" lookups (UserProfile ×5, Station ×4, Group ×3, DataPool ×3 …),
plus six private `unquote` copies — so `Group 1 Thru 4` and `Output 1 Thru 4` followed different
rules.

57. **One resolver turns every target into objects — strict** (`core::resolve`, built before 3b2):
    the parser gives a `Target`; `resolve` gives each item's object kind, its owner and its members
    (number, GUID, name) in typed order, by the one range rule (`3`, `1 Thru 4`, `1 Thru`, `Thru 4`,
    `+` / `-`, typed order kept, missing numbers skipped), names case-insensitively (24), from what
    the registries declare (`ObjectDef::lum`, `TreeDef`); fixtures are a typed view on it
    (`Group 1 + Fixture 7 - Group 2`); nothing matched is one refusal (E-CMD-007). **No handler,
    command or module resolves ids or names any other way**: the old resolvers are counted by a
    tidy ratchet that only goes down and are gone in part 5; a new resolver or lookup copy fails.

**Decision 57 built (2026-10-15, branch `lum-syntax`):** `core::resolve` — `resolve` (each item:
object, datapool, named, members `(no, guid, name)` in typed order), `resolve_one` (one kind,
items joined), `fixtures` (Fixture / Group items, `+` / `-`, group members in their order), and
the bridges `words`, `fixture_words`, `target_of` for handlers that still get raw words (they
parse the words as Lum first). Members come from `TreeDef::children_in` (the DataPool owner or your
selected one) — `DynChild` now carries the stored `name` and `guid`, and a test checks every
collection does —, DataPools from the tree's own list (a numbered node, not a collection).
`Root.` tree paths resolve through the typed tree. Not yet: owners other than DataPool
(UserProfile, Sequence → Cue, Station → Network …) and multi-part ids (Cue 1.5, Master 2.1, Window
1.4) — they join the resolver as their areas move (3d), never as a handler-side lookup. The tidy
ratchet `old_resolvers_only_go_down` counts 19 old resolvers / lookups (104 uses; 102 after groups'
`resolve_fixtures` moved).

**One property registry (asked at 3c, 2026-10-15).** The survey of `Set` found three property
systems behind it — `TreeDef::write` (`core::props`: name, JSON field, kind, rights), the keyed
settings (`core::settings_props`: Station, Interface, Network, Show, AssetRoot, About — get / set,
home, section, unit) and 18 objects with their own `Set` parser (Cue, Group, User, Window / Self,
Screen, Version, Recovery, GridColumns, FeatureGroup, Attribute, Device, DataPool, ShowData,
UserProfile …) — about ten on/off parsers with different words (one turns any unknown word into
false), a target-less form (`Set ClearBuffer On`) duplicating five UserProfile properties, and two
`Set` handlers nothing reaches (Interface, Show: the row router took them first).

58. **Every object declares its properties once, on its `ObjectDef`** — name, kind (Text, Int,
    Choice, OnOff with or without Toggle, Colour, Icon, Link …), rights, home (show / station),
    section, unit, doc, how it is read and written — and `Set`, `Info`, `EditSetting`, the
    settings cells, the menus' grid columns, suggestions, the rights gate, help and the reference
    read only that list. `TreeDef::write`, the keyed settings and the hand-written `Set` parsers
    become entries in it. `Set` is one typed handler (`Set <target> "<Property>" <value>`): the
    target by the one resolver (57), the property by its quoted name (7), the value parsed once by
    the property's kind (14's On / Off / Yes / No, Toggle where declared; relative input; a
    `Thru` value spread over the rows), every row one statement of the line (one Oops step). The
    target-less forms go (16: `Set UserProfile "ClearBuffer" On`). Forms that are not "a
    property and a value" — moving an attribute into a feature group, cue timing per feature
    group — get their own question when their area moves.

**3c hardening (after 3c1, 2026-10-15), one question at a time.**

59. **A negative value right after an implicit owner's quoted property stays refused** (E-SYN-007,
    decision 11 as built: `Group 1 +3`): `Set UserProfile "Theme.AccentHue" -30` reads the glued
    `-30` as a list operator. Name the owner (`Set UserProfile "LD" …`) — and with decision 60 the
    property follows a child keyword, so the case doesn't arise for nested settings.
60. **A property is never nested inside its quotes** — no dotted names (`"Theme.AccentHue"`,
    `"LockScreen.AutoLock"`, `"Theme.Palette.Accent"`). A group of settings is a **one-of-a-kind
    child object**, written owner first like `Clear User 3 Programmer` (no grammar change:
    `segment = OBJECT [ ids ]`): `Set UserProfile Theme "AccentHue" -30`,
    `Set UserProfile "LD" LockScreen "AutoLock" 10`, `Set UserProfile Theme Paint "Accent"
    "#9a7a08"`. New registered words (one namespace): **Theme** and **LockScreen** (children of
    UserProfile), **Paint** (the theme's colour group, a child of Theme — so theme settings group
    into categories before more are added). Each new object is hardened on its own (verbs,
    default verb, rights, what it holds).
61. **`Reset` puts defaults back; `Default` stays the fixture-value word** (console probe 2026-10-15:
    the reference `Default` is `Fixture 1 At Default` / `Store Default` only — `Set Sequence 1
    "Priority" Default` never completes on the desk; its `Reset` clears an object's settings
    owner first, `Reset Selection Pawterns`): `Reset <target>` puts all of an object's
    properties back (`Reset UserProfile Theme`, `Reset UserProfile Theme Paint`, like today's
    `Reset Self`), **`Reset <target> "<Property>"`** one property, for every object of the
    registry (`Reset Output 1 Thru 4 "Rate"`, `Reset Station "Priority"` — our way; the reference has no
    per-property form). Every property declares its default (the settings cells' reset reads
    it). The value-`Default` properties `"Theme"` / `"LockScreen"` go.
62. **Two kinds of colour, by role, with two words.** **`Color`** is light — the feature group
    (GDTF), stored as CIE 1931 xyY (device-independent, like GDTF's wheels, emitters and
    filters) and converted to each fixture's emitters. **`Paint`** is screen — the theme's colours
    and an object's own pool colour, sRGB with alpha written `"#rrggbb"`; the `NodeMeta` property
    `"Color"` becomes `"Paint"` (`Set Group 1 "Paint" "#3c8f3c"`). A property declares which kind
    it takes. How a light colour is written and handled (`At Color …`, CIE → emitters, presets,
    Prowl blending) is its own § (roadmap), hardened later; 3c needs only Paint.

63. **The theme bases keep their names** — Dark, Daylight, Night (cat-coat names were offered and
    declined); the property is **`"Base"`** (not `"Preset"`, the lighting word):
    `Set UserProfile Theme "Base" Night`. A one-of-a-kind settings object typed alone runs
    **EditSetting** (`Theme` = `EditSetting UserProfile Theme`; `LockScreen`, `UserProfile Theme
    Paint` alike).
64. **The property's kind decides what its value slot takes** — the same word can mean different
    things in different properties, like `Off` (verb at the head, value in a slot). A Choice takes
    its declared words: in `Set Fixture 1 "MasterReact" Group` `Group` is the word, not a group (a
    bare object keyword without ids is the value word when the property declares it). A **Link**
    takes a target of the linked type through the one resolver (57) — `Network 2`,
    `Network "Rig"`, a bare quoted name or number meaning that type, or a `Root.` tree path —
    and is stored as the object's GUID, so it follows the object when it moves; the hand-written
    link lookups (`network_link`, `user_link`) go. On / off, numbers, text and Paint as declared;
    a word the kind doesn't take is refused, naming what it takes.
65. **A property may point at another object's property — and that is always a live link**
    (the value follows the source; no one-time copy form): stored as GUID + property name,
    class-matched by kind. The ultimate case the design must allow: a preset's Dimmer value
    driving a user's theme Paint `"Background"`. How kinds that don't match connect (declared
    conversions, a per-link mapping), how a link is typed on the command line ("it must be easy to
    type"), loops, replication and the render rules — in hardening, informed by how node tools
    (Node-RED, TouchDesigner, Blender, …) do it, with a flexible node network later in mind.

66. **`Link` is an object** (a new registered word) — a stored live link from a source property to
    a target property, made with the existing verb **Assign** (like `Assign Universe 1 At Output
    1`): `Assign Preset 1.4 "Dimmer" At UserProfile Theme Paint "Background"` → `Link 7` (source
    first — decision 78); `Unassign UserProfile Theme Paint "Background"` removes it. Its conversion is its settings
    (`Set Link 7 "OutFrom" 20`, `Set Link 7 "Gradient" "Warm"`) — established remotes carry their
    mapping the same way (`InFrom` / `InTo` / `OutFrom` / `OutTo`, `TriggerOn` / `TriggerOff`).
    Researched 2026-10-15 (Node-RED's Range node, TouchDesigner's references / Bind, Blender's
    typed sockets and implicit table, Unreal / Vuo auto-inserted converters, Chataigne's
    per-mapping filter chain): **a declared conversion table per kind pair** (Exact / Lossy —
    shown as such / Refused with a coded error at link time, never a silent guess) **plus a
    per-link mapping** (range in / out with clamp, curve, gradient Number → Paint, threshold,
    invert); a link that would close a loop is refused. Details in hardening.
67. **Links live in three homes**: `ShowData.Links` (the show's), `UserProfile.Links` (a
    profile's — exported with it) and `DataPool.Links` (a song's — a DataPool brings its links
    when it is imported).
68. **A link's ends are stored as GUID + address** (`Group 2`, `UserProfile Theme Paint`, where
    "the user's profile" is a role, not a GUID), so an end can be found again where the GUID
    isn't there.
69. **Importing a DataPool reconnects its links**: an end inside the DataPool takes the imported
    copy's GUID; an outside end with the same GUID connects; one found only **by address is a
    proposal the user reviews, approves and confirms** (a station ask listing each link —
    nothing reconnects by address on its own); one found by neither is kept broken and listed
    (the reference check: never dropped).

70. **A new link lands in the home its line names, owner first** (`Assign DataPool 2 Link …` → the
    song, `Assign UserProfile Link …` → your profile — the exact form follows decision 78's
    direction, in hardening); with no owner it lands with what it
    writes — a profile setting in that profile, anything else in the show; the reply names where
    (`DataPool 2 Link 3`).
71. **Links follow at 60 Hz with the main engine.** A stored source (a preset's Dimmer) follows on
    the change path when it is edited; a live source (`Master 2.1 "Level"`, a playback fader, a
    fixture's output) follows every frame: a link that ends in DMX (`Master 3.1 "Level"` ←
    `Sequence 5 "Level"`) is compiled into the render plan and computed in the 60 Hz tick (same
    frame as its source; the render rules hold — fixed arrays, no allocation, no lock); a screen
    target (theme Paint ← the grand master, the FOH screens dimming with a blackout) rides the same
    frames out to the clients' feeds. A live value never writes show data or an Oops step.
72. **A link crate, and the one resolver for its ends.** A new crate holds the pure parts — the
    link graph, the conversion table per kind pair, the mapping math (range, curve, gradient,
    threshold), loop detection and the compiled per-frame link plan for the tick — with no tree
    and no station (testable alone, like `lumacore-dmx`): **`lumacore-links`** (plain, like the
    other crates; cat names were offered and declined), its station side `core::links`. Finding a link's ends (GUID → object,
    the address re-match on import) goes through `core::resolve` behind a trait the crate
    declares — still one resolver (57), which gains a "link end" mode; `core::links` keeps
    Assign / Unassign, the change path and the import review.

73. **A link's mapping settings are named like Blender's Map Range node** (compared 2026-10-15
    with Node-RED's Range node, TouchDesigner's Math CHOP, Unreal's Map Range, Max's `[scale]` and
    established remotes): `"FromMin"` / `"FromMax"` (the part of the source used), `"ToMin"` / `"ToMax"`
    (where it lands in the target, in the target's units), `"Outside"` Clamp / Wrap / Drop
    (Node-RED's three; Clamp the default), `"Curve"` Linear / Stepped / Smooth / Smoother
    (Blender's interpolations), plus `"Invert"`, `"Gradient"` and `"TriggerOn"` / `"TriggerOff"`
    (→ On / Off, with hysteresis). `Set Link 7 "FromMax" 50`.

74. **Setting a linked property by hand asks** (a station ask, the one dialog path): the value
    follows Link 7 (Master 2.1 "Level") — **Remove link** (Unassign, then set) · **Pause link**
    (`"Enabled"` Off, then set) · **Cancel**. Never asked inside a stored key / macro line (like
    Delete): refused there, naming the link (an E-LINK code). The settings cell shows a linked
    value with its link; one owner per value — nothing is silently overwritten next frame.

75. **The first conversion table**: Number ↔ Number (through From / To; exact) and Number ↔ On / Off
    (to On / Off by `"TriggerOn"` / `"TriggerOff"` with hysteresis; from On / Off as `"ToMin"` /
    `"ToMax"`); every other pair only to its own kind (Text, Choice, Paint, Link) until decided.
    **Numbers drive a Paint two ways** (Blender's Separate / Combine Color and Color Ramp,
    TouchDesigner's per-channel binding, Chataigne's ColorRemap): a link drives one **part** of a
    paint (`"Part"` Hue / Saturation / Brightness / Alpha, or Red / Green / Blue — several links on
    one paint, one owner per part), or a **gradient** maps one number to a whole paint.
76. **A link is a typed node.** Link types are declared in the Rust registry (like window types,
    §30 12c — sent as JSON, never stored in a show): each declares its inputs (fixed, or growing —
    one more appears when the last is assigned) and its own properties. Inputs are child objects,
    owner first — **`Inlet`** (Max / Pure Data's word; `Input` is the DMX / OSC line and `Port`
    its property): `Assign Link 8 Inlet 2 At Master 2.1 "Level"`, `Unassign Link 8 Inlet 3` —
    each assigned At a source; a plain `Assign … At …` makes the simplest type (one
    number, From / To / Curve). First types to harden: the mapping one, Gradient (number → paint),
    Combine (H S B A → paint, `"Mode"` HSB / RGB), Threshold (number → On / Off), Math (Add /
    Multiply / Min / Max over N inputs). The start of the node network.

77. **Links chain, with Outlets from the start.** Every link type declares its results as
    **`Outlet`** children as well as its `Inlet`s, so a link's result feeds another link
    (`Assign Link 8 Outlet 2 At Link 9 Inlet 1`) and a type with several results (a Separate that
    splits a paint into H, S, B, A) is possible now; a link that would close a loop is refused.
78. **Assign reads source At destination, one way only** — like every Assign
    (`Assign Universe 1 At Output 1`, like `Assign Sequence 1 At Executor 101`): what flows comes
    first, where it flows to after `At` (decision 8: Assign's `At` is a destination).
    `Assign Preset 1.4 "Dimmer" At UserProfile Theme Paint "Background"` → Link 7,
    `Assign Master 2.1 "Level" At Link 9 Inlet 2`, `Unassign Link 9 Inlet 1`. Both directions
    aren't allowed: one spelling per meaning, and `Assign Link 8 Outlet 2 At Link 9 Inlet 1` would
    be ambiguous.
79. **A quoted property sticks to the target before it** in a verb whose signature takes property
    references (Assign of a link): `<target> "P"` is one property reference, on either side of
    `At` — the binder reads it from the slot order the parser already keeps (no grammar change;
    lum.md gets the rule sentence). Where an end needs a property and none follows its target, the
    target's last quoted id is taken (decision 17's right-binding, per side): `Assign Station
    "Priority" At …` = this station's Priority. `Set` is unchanged (its property binds from the
    right).

80. **Links are only ever made by a user.** A clean show (NewShow / CleanStart), the locked
    UserProfile 1 Default, the shipped data files, the demo show (`gen-demo` — built by the code)
    and the code itself never contain or create a link; a test checks each has none.

81. **Making a link needs the destination property's level** (what a `Set` of it needs: a Theme
    Paint on your own profile — Guest; Output `"Rate"` — Setup); reading the source needs nothing
    (everything in the tree is readable). Making, changing or removing a link is an ordinary Oops
    step (its Links subtree, replicated); the values a live link writes never are (like running
    playback).

82. **A link never overwrites its destination's stored value** — it drives an output on top of it,
    like a master over a fader. Removed, paused or undone, the destination shows its own stored
    value again; the settings cell shows both (stored value · `← Link 7`); a show saves the stored
    value, never a driven one. (Setting the stored value by hand while linked still asks —
    decision 74.)

83. **A link is computed where its destination is used.** Links are show / profile / DataPool data,
    replicated like the rest; each station computes the ones whose result it needs — a link ending
    in DMX on the station that sends (the master or a standalone station), in its render tick; a
    link ending on a screen (theme Paint, a window) on each station showing it, from the
    session-wide values it already has (masters, playbacks). Driven values never travel between
    stations; only the links do.

84. **Links are command-line only at first** — `Assign` / `Unassign`, `Set Link 7 "…"`,
    `List Link`, `Info Link 7`, `Reset Link 7`; no window, grid or node editor yet (a node editor
    is a new kit element: research, mockup and approval first — its own roadmap item).

85. **Assign places objects; a Link carries a value from property to property** — so links get
    their own verb, **`Wire` / `Unwire`** (source first, like Assign): `Wire Master 2.1 "Level" At
    UserProfile Theme Paint "Background"` → Link 7, `Wire Link 8 Outlet 2 At Link 9 Inlet 1`,
    `Unwire UserProfile Theme Paint "Background"`. **This replaces Assign / Unassign in decisions
    66, 70, 77, 78 and 79** (their examples read `Wire …`; the property-reference rule 79 is
    Wire's). Assign stays objects into objects (`Assign View 3 At ViewButton 5`, `Assign Universe 1
    At Output 1`, `Assign Attribute …` below).
86. **An attribute goes into a feature group with Assign, owner first per layer** (the two layers of
    12f-a never mix): the show layer — cue timing per feature group — `Assign Attribute
    "Gobo1Pos" At FeatureGroup "Gobo"` (Setup + the Patch claim; a GDTF that put a gobo's index
    under Position no longer crawls with a Position fade), `Assign Attribute "Gobo1Spin" +
    "ColorWheelSpin" At FeatureGroup "FX" /WithSubs`; your own grouping — encoder bar, store
    filters — `Assign UserProfile Attribute "Gobo1" At UserProfile FeatureGroup "Beam"`. Today's
    `Set Attribute … FeatureGroup …` and the attribute board's line go.

87. **A cue's timing for one feature group is that group as the cue's child**, owner first, with
    the cue's own timing properties: `Set Sequence 2 Cue 3 "Fade" 2` (the cue's own),
    `Set Sequence 2 Cue 3 FeatureGroup "Position" "Fade" 4`, `Set Cue 3 FeatureGroup "Gobo"
    "Delay" 0.5` (your selected sequence); `"Fade"` `"Delay"` `"OutFade"` `"OutDelay"` `"Path"`;
    `Reset Sequence 2 Cue 3 FeatureGroup "Position"` goes back to the cue's own ("unset = the
    cue's"). Today's child-first `Set Cue 3 Sequence 2 FeatureGroup "Position" Fade 2` goes; the
    cue sheet and its keypad write these lines.

88. **`Set` only sets properties; an action is a verb.** A feature group's one-shot actions
    (today `Set FeatureGroup "x" Sort|Merge|Split …`): **merge** is objects into objects —
    `Assign FeatureGroup "Strobe" At FeatureGroup "Shutter"` (asks: its attributes move, the group
    goes, cue timing set for it is listed as broken); **`Sort`** and **`Split`** are **two new
    commands** (neither exists on the reference) taking `/By`: `Sort FeatureGroup "Beam" /By
    Name|Family|MainFirst`, `Split FeatureGroup "Effects" /By Family` (asks). General verbs — each
    object declares the `/By` keys it takes (later `Sort Group 3 /By Name`, `Split Group "All
    Spots" /By FixtureType`). The attribute board's buttons write these lines. A group member's
    grid spot is its member child's properties, one per statement:
    `Set Group 3 Fixture 101 "X" 4` (the multi-pair `… X 4 Y 2` form goes).

89. **An attribute template is applied with `Call … At …`** — a template is a built-in grouping
    recipe (feature groups + rules; shipped: "GDTF standard", "LumaCore FX"), now a listed object
    (`List AttributeTemplate`); `Call AttributeTemplate "LumaCore FX" At ShowData` regroups the
    show layer (cue timing groups), `Call AttributeTemplate "GDTF standard" At UserProfile` your
    own grouping — once, with today's ask (learned rules stay), like `Call WindowPreset 2`. Today's
    `Set ShowData|UserProfile AttributeTemplate "…"` goes.

90. **Levels are set only with `At`** — `Master Group 4 At 50 Fade 5` (owner first: the Master
    object owns the group masters, as lum.md has it), `Master 2.1 At 80`, like
    every value; `Set` on a group keeps its properties (`"Mode"` Positive / Negative / Scaling /
    Additive, `"Name"` …) and refuses `Set Group 4 "Master" 50` with a hint to the At form (the old `Group 4 Master At …`
    is child-first, E-SYN-004). A level
    is still a declared **live property** (`"Level"` of a master, a playback, a group's Master) —
    readable, a `Wire` source and destination (decisions 71, 85) — but never written by `Set`.

91. **A command line given as a value is written in backticks — strict, in lum.md and both
    lexers.** Since every property is quoted (7), stored lines hold quotes themselves, and Lum's
    text has no escapes; so a line inside a line is `` `…` ``: ``Bind "F5" `Set Self "AutoScroll"
    Toggle` ``, ``Store Alias "au" `Set Self "AutoScroll" Toggle` ``, ``Watch `List Fixture`
    /Every 500``. One spelling: a line value is always backtick-quoted and `"…"` is always text,
    never a line (today's `Bind "F5" "List Fixture"` is refused with the backtick hint); a
    signature declares which slots take a line (Bind's, Alias's, Watch's, a macro line, a cue's
    command). Backticks mean nothing else in Lum; a backtick inside a line is refused (no escapes).
    (Asked 2026-10-15 while moving Window's `Set Self "AutoScroll"` lines; alternatives — doubled
    quotes, backslash escapes, no inline lines — declined.)

92. **`List`, `Info` and `Dump` are one command each, reading the registered objects** (asked
    2026-10-15 at 3d: about 40 List, 30 Info and 6 Dump handlers, one per object, each with its
    own output). The command owns the logic, the objects only declare data: the resolver finds the
    rows, the columns and fields come from the declared properties and the tree collection, Dump
    is the node's JSON. An object whose information isn't stored rows (Health, Clock, a
    station's network runtime) declares its own view function on its ObjectDef; nothing else
    keeps a per-object List / Info / Dump handler. Outputs change shape (one look for every
    object). (Alternatives — a typed handler per object keeping today's outputs, or deferring the
    read verbs to part 4 — declined.)

93. **What `List` shows** (asked 2026-10-15). One rule everywhere: one row per object, **No ·
    Name**, then **every declared property its own column** (`List Output`: Note, Locked, Hidden,
    Network, Target, Priority, Rate, Send, PacketGap, Sync, Enabled); the grids in the menus may
    still show fewer columns for the screen. (A first answer, the grid's columns, was replaced by
    this one rule.) A bare `List` lists the **children of where you are** (the node
    `cd` put you on): at `Root.ShowData`, every ShowData object. Where you are scopes only the
    bare `List`: a named kind resolves as always (your selected DataPool, or the owner written
    first) — inside `DataPools.2`, `List Group` is still your selected DataPool's. `cd` never
    selects (§29), and it never redirects a line either. A bare `List` follows the prompt: with a
    default object set (`Group>`, §33 21) it lists that kind (`List Group`: your selected
    DataPool's groups); on a path `cd` put you on (`Root.ShowData.DataPools.4.Groups>`) it lists
    that path. When both are set (`Root.ShowData.DataPools.4 [Group]>`) the path wins — `cd` is
    the more explicit place — and a written `List Group` is still your selected DataPool's. At
    `Root` with no default object, `List` lists Root's children. Children of different kinds
    (`Root.ShowData`) show **No · Name · Kind · Count**, then **every property its own column**
    (the union of the rows' declared properties; a row without one leaves it blank).

94. **What `Info` and `Dump` show** (asked 2026-10-15). `Info <target>` per object: its identity
    (No, Name, GUID, tree path), **every declared property** with its value and kind, then the
    **facts** the object declares (read-only runtime: counts, state, a line's send numbers — a
    `facts` function on the ObjectDef, the same place as decision 92's views) and its **links**
    both ways (what it links to and what links to it, from the reference index). `Dump <target>`
    is the node's JSON as stored, one object or an array. A bare `Info` / `Dump` follows the same
    prompt rule as a bare `List` (93): the path you are on, else your default object's kind.
    (2026-10-15: built later, a roadmap item of its own — §33 continues with the core and syntax.)

**Part 3d1–3d2 built (2026-10-15, branch `lum-syntax`): Label, and the patch area typed.**
3d1: one typed `Label <target> ["name"] [/Clear]` (`commands::label`) for every object that
declares a `"Name"` and has no Label of its own: the resolver finds the rows, `set::write` writes
them, several rows number on from the name keeping the spacing, one row without a name asks; an
empty text now clears an optional text property (none), a required one keeps `""`. 3d2: Move
(Fixture, Group, Universe — `move_cmd::plan`: the resolver's members, destinations keep the
spacing, no `At` asks for one source), Delete (Fixture = unpatch, Universe) and Assign / Unassign
Universe are typed. Decision 22 is built: `Assign Universe 1 Thru 6 At Output 1 /Universe 101`
(Art-Net `0.1.2`); without `/Universe` the routes start at the first universe's own number (the
DMX Protocols menu's Assign… lines send none). The protocol shortcuts `At sACN 5` / `At ArtNet
…` / `Net … SubNet … Universe …` and `At Output 1 Universe 5` are gone, so is E-OUTPUT-004 (the
ambiguous-line error only the shortcut raised); the demo stores its sACN Output 1 first. Shim 394
→ 380, old resolvers 96 → 94. 3d3: Groups — Store, Delete, Copy, Update typed. Store names a
**slot** (`core::resolve::slot`, the resolver's Store rule): a number that may not exist yet, a
name an existing one has (its number), a new name (the next free number, named so), or none (the
next free); a range is refused (one at a time); owners in front scope it. Copy uses
`move_cmd::plan` (the spacing kept, no `At` asks); a destination past the last number is a value
error (E-CMD-008). A typed word's option has one name: `/o` is refused (E-SYN-006), `/Overwrite`
runs. `/GridMerge` declares its words (AppendX, Off). Shim 380 → 376. 3d4: TimingPreset / CurvePreset — Store (the slot
rule) and Delete typed, `"Name"` declared (so the one Label takes them), their tree rows carry
name and GUID. Shim 376 → 368, old resolvers 94 → 93. 3d5: Output / Input / Network — Store (the slot rule;
`/Protocol sACN|ArtNet|OSC` declared words, `/Network "Rig"`, `/User "LD"` quoted names,
`/Port n`) and Delete (the resolver's rows through `set_rows::delete_resolved`: rows that can't
go refused first, a missing row is the resolver's E-CMD-007) typed; `Create Output|Input|Network`
is gone (one name per word: Store). The setup menu's New ask quotes a name (only a number stays
bare). Shim 368 → 359. 3d6: `Store Universe [<n>] ["name"]` typed on the slot rule; the hidden
`Create` command (an alias of Store, against "one name per word") and every object's `Create`
handler are gone. Shim 359 → 355. 3d7: the resolver knows **scoping owners**
(`resolve::resolve_owned` / `with_owners`: an owner whose Props declare `within` — Sequence, Cue,
UserProfile, Station — is resolved to one row and its scope entered while the next segment
resolves; the rows come back with their owner chain) and the Store slot under owners with ids in
parts (`resolve::slot_owned`: `Store Sequence 4 Cue 2.5`, `Slot::Key`; a Store may name an owner
that doesn't exist yet by one number, `Store Sequence 9 Cue 1`). `Goto [Sequence <n>] Cue <m>`,
`Store [Sequence <n>] Cue [<m>]` and `Store Sequence [<n>]` are typed, owner first only: the
child-first `Goto Cue 3 Sequence 2` / `Store Cue 1 Sequence 2` are refused (E-SYN-004) and every
line, ask button, test, doc and kit line was rewritten. CueZero / OffCue by name are quoted
(`Cue "OffCue"`); a number starts with a digit (`0.5`, not `.5`). Shim 355 → 351, old resolvers
93 → 91. On / Off / Toggle / Edit Sequence typed (351 → 347). 3d8: **Master** on the
registry — keyed rows 2.1 (`"Grand"`), 2.2, 2.4, `"Highlight"`, `"Solo"` (other numbers reach the
master code, which says planned or none), its `"Mode"` a property (`Set Master "Highlight" "Mode"
AlwaysOn`, decision 88), and the Master owns the group masters (`Master Group 4 At 50 Fade 5`,
decision 90). `Master … At` and On / Off / Toggle Master are typed; a typed `At` takes its line as
`<target> At <value>` (decision 43). Gone: `Group 3 Master At …` (child-first), `Master Highlight
Mode …`, bare `Master Highlight` (now `Master "Highlight"`, decision 10). A level that isn't a
number is refused as an unknown word (E-SYN-001). Shim 347 → 343.

**Part 3c3 built (2026-10-15, branch `lum-syntax`): every `Set` line is the registry's.**
`commands::set::route` is the Set command's handler and takes every line: a target, a quoted
property, a value (decision 16). The target-less forms are gone, and the reply names the owner-first
line (`Set ClearBuffer On` → E-CMD-006 "Set names its target: Set UserProfile "ClearBuffer" On").
A bare `Set` is a usage error. An object without properties says so (E-CMD-015), and a line naming
none of its object's properties lists them (E-SET-001 / 002). Gone: `settings::handle_set` and its
arms, `set::legacy_words`, `set_rows::route`, the objects' own `Set` handlers (UserProfile,
DataPool, Self), `users`/`windows` `cmd_set_*` window paths, `vfs_runtime::set_window_setting`.
`Suggestions` and `Verbosity` are declared profile properties. `"ClearBuffer"` / `"ShowDescriptions"`
now write the fields the station reads (`ProfileSettings`); the dead copies in `UserPreferences` are
removed (format 25, demo regenerated). The terminal's tree pop-up edits with registry lines
(`Set Station "Priority" High`, `Set UserProfile 2 "ClearBuffer" On`). Shim 397 → 394.
**Open:** `Set Notifications <category> <level> <surface>` went with the target-less forms and has
no home yet. A `Notification` object is wanted, but on the User rather than the profile, and
together with how a notification is sent globally (to everyone). That is in hardening; the e2e
"important events can not be muted" test is `fixme` until then.
**Notification hardening (in progress, 2026-10-15).** N1 (decided): a notification sent
globally reaches the **whole LumaSession**: every station and every logged-in user there
(replicated through Runtime, never saved). Narrower audiences are named: `At Station "FOH"`,
`At User "LD"`. N2 (liked, shape open): a `Notify` word to send one (`Notify "Doors in 5"`).
N3 (decided): the choices of how messages show are a **category × level grid** (8 categories ×
Info / Warn / Error / Alert → None / Card / Popup; only Normal importance follows it, Important and
Critical reach everyone). N4 (decided): the grid lives on the **UserProfile** — a profile is how
the system and the UI look and behave, a User is the account (name, password, rights, programmer,
selection); users sharing a profile share it. The audience of a notification names the account
(`At User "LD"`), the profile decides how it shows: `Set UserProfile Notification "Network" "Info"
Card` (your own: `Set Notification "Network" "Info" Card`).
N5 (decided): notifications and chat are **two systems**. Notifications come first. Chat (local,
groups, the whole LumaSession, across LumaSessions later) is a later roadmap item that **hooks into
the notification system**: a chat line can raise a notification for a user or a group. So the
notification audience is designed for that from the start (a user, a group of users, a station, the
LumaSession), even before chat exists.
N6 (decided): a group of users is a new object **`Team`**: a named set of users (by GUID), saved
in the show (ShowData, replicated). `Store Team "Crew" At User "LD" + "Video"`. Notifications
address it now; chat later lets you pick which Team (or other audience) a line goes to. N7 (decided,
shape of the audiences): `Notify "<text>"` with no audience = the LumaSession; `At User "LD"`,
`At Team "Crew"`, `At Station "FOH"` narrow it. The wider idea (a user known beyond one show, so
people elsewhere can reach them) is a backbone note in `visions/notes/global-users.md`, not a
decision.
N8 (decided): `Notify` takes `/Level Info|Warn|Error|Alert` (default Info) and `/Importance
Normal|Important|Critical` (default Normal). Normal follows each profile's grid; Important
(nobody can mute it) needs Setup; Critical (a pop-up to everyone) needs Admin — the rights gate
reads it from the declared option. A person's notifications are category Show; the system keeps
its own categories. Sent notifications are not objects (N2 shape): Notify only sends, the System
Monitor keeps the log, and the profile's grid object is `Notification` (one row per category, its
properties the four levels).
N9 (decided): `Store Team "Crew" [At User "LD" + "Video"]` makes one, `Assign User "Bob" At Team
"Crew"` adds, `Unassign User "Video" At Team "Crew"` removes (decision 85: Assign places objects),
`Label` / `Delete` (asks) as for any object; changing Teams needs Setup, anyone may read them.
N10 (follows from §10, not a new path): a `Notify` line runs on the session master like any line
whose scope is Master, and reaches the members as session runtime in `link::flush` — never a
second sync path. It is not an Oops step and is never saved. The new words (`Notify`, `Team`,
`Notification`, `/Level`, `/Importance`) are vocabulary in the registries; lum.md's grammar is
unchanged, and its examples get them when they are built.

**Part 3c2b built (2026-10-15, branch `lum-syntax`): Theme, Paint and LockScreen are objects.**
A profile's look is its children, owner first (decision 60): `Theme` (`"Base"` Dark / Daylight /
Night, `"Brightness"`, `"Contrast"`, `"Saturation"`, `"Tint"`, `"AccentHue"`), its `Paint` (the 14
screen colours, sRGB `"#rrggbb"`, none = the base's; 62) and `LockScreen` (`"Image"` — a number or
name of the profile's own Images becomes its GUID —, `"ShowClock"`, `"Text"`, `"AutoLock"`).
One-of-a-kind: the row is the `UserProfile <x>` in front (its `within`), else your own; their
fields live in the profile's JSON (`Props::view`), so a field under a scoping owner is allowed
where the object declares a view. Typed alone each runs its default verb `EditSetting` (63; new
`Verb::EditSetting`, typed `EditTheme` / `EditPaint` / `EditLockScreen`); `EditSetting UserProfile
[x]` shows the children's cells too, each line owner first (`Set UserProfile 2 Theme Paint
"Accent" {value}`; Paint on the Look tab, the Users & Profiles window leaves it to the theme
editor), and `EditSetting UserProfile 2 Theme` scopes the same way. `Reset [UserProfile x] Theme
[Paint "Accent"]` puts the declared defaults back; the `"Theme"` / `"LockScreen"` reset words and
every dotted name are gone. The pool objects' NodeMeta `"Color"` is `"Paint"`, and so are the
pool windows' colour settings (`empty_paint` … `for_none_paint`, `"EmptyPaint"`; show format 25,
the demo show regenerated). An option may declare its value words (`FlagDef::choices`: `/By Name |
Family | MainFirst`; one that is a property's name stays that). Shim stays 397.

**Part 3c2 built (2026-10-15, branch `lum-syntax`): the hand-written `Set` parsers on the
registry.** The registry grew what the areas needed, each declared on the object: `Access::Named`
(one get / set for a list declared as data, told the property's name), `PropKind::Declared` (the
object's type checks the value — a window setting's Toggle, Now, relative input, links),
`Props::dynamic` (every window type's settings, built once), `Props::scoped` (properties that exist
only under an owner — a group member's grid spot, a feature group's timing in a cue),
`Props::within` used down a whole owner-first path (`Set DataPool 2 Sequence 3 Cue 2 FeatureGroup
"Dimmer" "Fade" 4`: a DataPool owner in front, then each scoping owner's rows), `Props::hint`
(`Set Group 4 "Master" 50` → `Master Group 4 At 50`, the At form; the owner-first Master form is
3d's), and `Property::default` for `Reset <target> ["<Property>"]` (decision 61; `Default` as a
`Set` value is refused with the Reset hint). Moved: Version / Recovery, GridColumns (ids), Screen
(`"EncoderBar"` On / Off / Toggle and `"EncoderLayout"` — one kind per property), Device, User
(`"Password"` / `"Name"` `Ask`, `""` = no password; the user's colour is `"Paint"`, 62), Window /
Self (CamelCase names of the type's keys, their choice words in the vocabulary), Group (+ members),
Cue (+ feature groups), Fixture `"Address"`, Input `"MergeMode"`, Attribute / FeatureGroup (86, 88:
`Assign` into a group and to merge, typed `Sort` / `Split` with `/By` — new verbs, a property
word works as an option's value), `AttributeTemplate` (89). A bare object keyword without ids is
the value word unless the property links an object, and a helper word (`Fade`) a signature doesn't
take is a value word (64). Backtick lines (91) are in lum.md's grammar, the parser (`Value::Line`,
`CommandLine`) and the station's lexer (`Kind::Line`); `Bind` takes a line only in backticks. The
station's lexer keeps a quoted `""` (an empty text), except that Label still asks on one (§30 12b).
Shim 408 → 397. Not yet: Theme / LockScreen / Paint objects and the `"Color"` → `"Paint"` rename on
pool objects and window settings (3c2b), the target-less forms (3c3); `Goto Cue 2 Sequence 7`
(child-first) is 3d's.

**Part 3c1 built (2026-10-15, branch `lum-syntax`): the one property registry.** `core::props` is
the registry: one `Property` (name, kind, named words, display words, rights, home, section, unit,
doc, `Access::Field` — a field of the object's JSON, written through `Props::apply` — or
`Access::Keyed` — get / set by row), `PropKind::OnOff` (On / Off / Yes / No) and `Switch` (also
Toggle), and `Props` on `ObjectDef::props` (the lists, keyed rows, the implicit row of a bare
owner, labels, apply / may, after, read-only, and the station's own write, asks and wiring scope).
`TreeDef::write` and `settings_props`' `KeyedDef` / `Setting` are gone; Fixture, Universe,
Network (+ its station-local `"Interface"`), Output, Input, Sequence, Group, UserProfile (implicit:
your own), Image, AssetRoot, Show, Server (About), Station and Interface declare there. `Set` on
them is one typed handler (`commands::set::SetProperty`: target, quoted property, value): rows by
the one resolver — keyed rows (interfaces, stations, the show) resolve there now, a name or
number by the object's own row lookup —, `Station <rows> Network|Interface …` sets the child
within each station's wiring row (`Props::within`), the value parsed once by the kind, every row
one statement of the line. The binder takes a trailing `+ n` / `- n` after an implicit owner's
quoted property as the relative value (`Set Show "Autosave" + 10` — decisions 11, 17). Lines the
station writes quote ids and properties (asks, `Put back:`, menu templates, the remote edit sent to
another station; the kit's `idWord` quotes every id that isn't a number). Not yet: the
hand-written `Set` handlers (3c2 — a quoted property reaches them bare until then), the
target-less forms (3c3). **Open:** a negative value right after an implicit owner's quoted
property (`Set UserProfile "Theme.AccentHue" -30`) parses as a glued list operator (E-SYN-007).

**Part 3b2 built (2026-10-15, branch `lum-syntax`): the default object.** `User.default_object`
(the keyword, None = Fixture; show format 24, `SESSION_PROTOCOL` 15) — in the User subtree:
replicated, never an Oops step (a keyword alone is `Undo::Untracked`, runs on the master, needs
no rights). A counted keyword alone sets it (decision 39: `Menu` → `Menu>`, so the ⚙ key is
`Toggle Menu` — decision 38 — and the window bar's "Load preset" `Call WindowPreset`). A bare
number, quoted name or `Thru` at the start of a statement is written out with your default
object, and a number right after a verb with the verb's own `default_object` (Store → Cue,
Go / Go- → Sequence, Goto → Cue) or yours when it takes that verb (`Delete 3` = `Delete Group 3`)
— in `dispatch` before anything reads the line, and again for nested lines — so it takes the
change path, journal, forwarding and Oops of the full line (issue 02). The old Fixture /
Universe `context_number` shortcut is gone. A key line starting with a bare number is refused on
save (E-SYN-008). The terminal prompt shows it: `[LD@host] Root [Group]> `. Not yet: the
web command bar (a UI change — mockup first), the profile's start place (issue 02), `Root>` (3e2).

**Part 3b1 built (2026-10-15, branch `lum-syntax`): default verbs.** Every `ObjectDef` declares
`default_verb: Verb` (decision 40's table; the objects it didn't name took the obvious one: List
for list-only objects — WindowType, ShowData, UIEngine —, Info for Server, Health, Effect,
EffectEngine, Screen, TimingPreset, CurvePreset; the preset types follow Preset). A bare object
runs `ObjectDef::bare_verb`: the At verb when the line has `At` and the object takes it
(Fixture, Group, Master, Attribute — decision 43), a named owner-first child's own bare verb
where the owner passes it on (`DataPool 2 Group 3`, `UserProfile "Bob" WindowPreset 2`), else the
default verb — run by the object's handler, or by the verb's own command (`Sequence 3` =
`Select Sequence 3`). New verbs `SelectFixtures` and `Call`; `Select`, `Goto`, `Go` route objects
that take them and keep their own form otherwise; `Select Group 3` names SelectFixtures. The old
catch-all Call handlers became real verbs or went: a bare `Universe 3` no longer creates (41,
`Store Universe 3`), `Clock` is `Info Clock`, `Show`'s `/Save` `/Load` `/Description` `/Autosave`
`/Merge…` `/Startup` are gone (46: `SaveShow`, `LoadShow`, `Set Show …`, the station's
StartupShow). Interim until 3b2: a keyword alone runs its default verb with no ids (lists where
that verb isn't built). `Watch` keeps Call as its default (it starts a watch) until 3e makes
Watch a verb — **open: decision 29 makes `Watch` a verb while `Watch` is also an object (`List
Watch`); one name can't be both (decision 12) — to ask at 3e.** The object reference lists each
default verb.

**Follows from the decisions (no question needed):** a line stored in a show (keys, macros, cue
commands, demo) that §33 rewrites means a **format bump** — forward only (§20): older shows are
refused, the demo is regenerated, no load-time rewrite. Inside the tree-shell fence `/` is the
root, so `ls /r` becomes `ls -r`. Machine lines (OSC, gesture channel, HTTP, UDP) are exact: no
fuzzy, no aliases; an OSC address is not a Lum option. Password masking works per slot on the
printed line. Every template line the code emits (ask buttons, slab, menu_defs, `Put back:`) is
rewritten and tested as pure Lum. Superseded by §33 once it is decided: §14's "shortest-unambiguous
abbreviations", §15's prefix / tie rules and "short forms are registry aliases" and "one word may
go unquoted", §31's "no signature change" (`rest_value` / `positional`), §30 "12b2"'s "Enter runs
the line as typed", issue 02's `<Keyword> Please`.

**Conflicts with earlier decisions:** found 2026-10-15, all decided above (25–52).

**Next:** lum.md gets the `grammar` block and the rewritten examples (the spec), CLAUDE.md the
strict rule, the roadmap the build (decision 33) — then the build waits for an explicit go.

---

*Decisions recorded: 2026-03-10; §3, §10 (rewritten), §11, §14 revised, §15–§27 added and §7/§13 annotated 2026-10-10; §28 added 2026-10-11, §29 2026-10-11, §30 2026-10-12, §31 2026-10-13, §32 2026-10-14, §33 2026-10-14*
*Sources: Design discussion, reference console v2.3.2.0 root dump analysis, COMMAND-REFERENCE.md*

## 34. Code structure and file headers (2026-10-15)

Asked 2026-10-15, one question at a time, after a research round: the 40 longest files (31 over
1,000 lines; `lumacore` is 96k of the workspace's 140k lines), every existing structure rule
(§23 layers are firm; nothing on file shape), 13 families of near-duplicate code, and how
rust-analyzer, nushell, ripgrep, helix and zed lay out crates and files. The rules are in
[../dev/architecture/code-structure.md](../dev/architecture/code-structure.md); this section
records what was decided and why.

**Headers (every `.rs` file), after a reference console's systemtest Lua files** (
ANSI Shadow banners for parts, `****` boxes for sections, `----- Step -----` inside functions).

1. **File banner**: the module's name in ANSI Shadow letters, then a tag line `// FILE: <name>`.
2. **Module doc** (`//!`) under it, a fixed shape: one sentence of purpose, then `Owns:`,
   `Doesn't own:` (and where that lives), `Entry points:`, `Decisions:`.
3. **Parts**: ANSI Shadow banners for the categories of a file, a fixed vocabulary in a fixed
   order — TYPES, INIT, REGISTRY, COMMANDS, LOGIC, VIEWS, IO, HELPERS, TESTS — each followed by
   its tag line `// PART: LOGIC`. A file uses the Parts it needs, never others. Titles are short
   (≤ 12 letters) so the 6-line banner fits 100 columns; the plain tag line keeps them greppable.
4. **Sections** inside a Part: a star box — a rule of `*`, the title, an optional 1–3 line
   purpose, a rule. They replace today's `// ── X ──` lines.
5. **Steps** inside a function over ~40 lines: `// ----- Checks -----` (dashes).
6. **Banners are generated, never drawn**: one font table in `crates/lumacore-banner`,
   `./scripts/unix/banner "Logic"` prints one, and tidy re-renders every tag line and fails if
   the art above it differs.

**Structure.**

7. **Crates**: the station is cut in three, one direction only (a tidy rule checks the crate
   edges): `lumacore-state` (AppState, the change path, Oops, props, resolve, errors, dialogs,
   drive — `core/` today minus the objects and dispatch) → `lumacore-objects` (the registry
   tables, `OBJECTS`, every object, dispatch — 16) → `lumacore`, the station (server, net, terminal,
   lumasession, protocols, engines). (Not `lumacore-core`: §23 retired that name for
   `lumacore-base`. Alternatives — objects behind a `Station` trait, or one crate for now —
   declined.)
8. **One folder per object, one file per verb** (nushell's commands, rust-analyzer's assist
   handlers): `objects/group.rs` holds the REGISTRY Part (the ObjectDef, its Props);
   `objects/group/{store,delete,copy,master,select,views,tests}.rs` hold one typed command each
   with its logic. The registries stay explicit lists (no inventory / linkme).
9. **Size**: a file should stay under ~500 lines; over 800 fails tidy unless listed with a
   written reason (a generated table, one state machine). Today's large files are on that list
   as a ratchet that only shrinks. Functions: clippy `too_many_lines` (100) as a warning.
10. **Modules**: `foo.rs` + `foo/`, never `mod.rs` (`clippy::mod_module_files` once none is left;
    a ratchet until then).
11. **Tests**: a module's unit tests in `foo/tests.rs` through `#[cfg(test)] mod tests;` — one way,
    replacing today's three (sibling `*_tests.rs`, `#[path]`, inline modules); line-in / reply-out
    behaviour tests in one `tests/it/` binary per crate with a `check(line, expect)` helper.
12. **Shared code has one door each**, named in the document next to `drive` and `resolve`, and
    tidy bans the copy patterns (as ratchets while copies remain): the next free number, not-found
    and exists replies, the DataPool scope (`AppState::in_datapool(dp, |s| …)`, also panic- and
    `?`-safe, which today's hand-written save / restore isn't), the label / name / delete asks,
    on/off words (the four, no `true` / `1`), `unquote`, colour and percent text, the plain
    Store / Delete / Label / Move / List steps (a `Collection` trait), list reordering.
13. **The next free number is the first unused one, everywhere** (Groups 1, 2, 4 → Store Group
    makes 3). This replaces §30 "Store Cue"'s "a new sequence after the highest": a
    Store Cue with no sequence selected fills the gap too. (Today 8 places use first free and 14
    "after the last".)
15. **The registry's types go down, its contents up** (asked 2026-10-15 at 34b, after measuring 172
    references from `core/` into `commands/` and `objects/`): `lumacore-state` keeps what knows no
    registry content — AppState, the change path, Oops, props' types, errors, dialogs, drive,
    show_file, backups, pools — plus the registry **types** (`ObjectDef`, `CommandDef`, `Verb`,
    `Doc`, `Rights`, `FlagDef` …); the modules that read the registry's **contents** (`OBJECTS`,
    `find`, `DATAPOOL` / `FIXTURE` / `STATION`) — resolve, rights, claims, lum, menu_defs, devices,
    settings_props, show_import, self_ref — move up into `lumacore-objects` beside the tables.
    Tests that run lines through dispatch move up with them. (Alternatives — the tables installed
    into state at startup, or postponing the cut — declined.)
16. **Dispatch goes down with the objects** (asked 2026-10-15 before 34c; handlers run nested
    lines — macros, keys, an ask's rest — and need the result at once: every statement its own
    Oops step, an ask stops the line, an error stops a macro): `dispatch` moves into
    `lumacore-objects` beside the tables, one re-entrant function as in Vim's `do_cmdline`,
    Emacs' command loop and nushell's `nu-engine` (handlers call it and get the reply). What only
    the station can do leaves as today — the outbox (`session_link::LinkOut`, `core::background`)
    or a `DispatchResult` the server acts on after the job (gpui's `defer`, an indirect command:
    only for work whose result nobody waits for). Nested depth gets a cap with its own code (Vim:
    200). (Alternatives — a `lumacore-engine` crate below the objects, nushell-style, which needs
    the tables as data filled from above; or dispatch left in the station behind a `fn` hook,
    rustc's `Providers` — declined; one hook only if a single call can't move.)
17. **Every crate is `lumacore-<category>`** (asked 2026-10-15): the folder under `crates/` and the
    package name are the same, `lumacore-` plus what it is — a tidy rule checks both. The two that
    weren't: the station `lumacore` → **`lumacore-station`** (its binaries keep their names), and
    `lumacore-session` → **`lumacore-runtime`** (it holds `Root.Runtime`'s types; it never held
    LumaSession code, and "session" alone is no name of ours). The LumaSession code
    (`lumasession/`) stays a station module until it is untangled from `ServerContext`; the station
    is researched as a whole to plan its split into crates (roadmap).
18. **Every workspace crate has a README.md** (asked 2026-10-15): every `crates/lumacore-<category>`,
    `apps/lumacore-*`, `systemtest` and `fuzz`. It is the crate's specification page, quick to read:
    - **decisions.md stays the source**: the README never restates a decision; its Decisions
      section lists the §s it follows, one line each, linked;
    - **seven sections, in this order** (tidy-checked): `# lumacore-<category>` + one sentence ·
      Purpose · Owns / Doesn't own · Public surface · Rules · Edges · Decisions · Testing;
    - **the README is the crate's text**: `lib.rs` keeps its ASCII banner and one `//!` line pointing
      at the README (tidy's header rule accepts that for a crate root);
    - **every rule names its check**, in brackets: a test, a tidy rule, the compiler, or `unchecked`
      written out — tidy verifies a named test or tidy rule exists;
    - **Public surface is hand-written** (what other crates should use, one line each); **Edges are
      checked**: they equal the crate's EDGES row and its Cargo.toml dependencies;
    - **one pass per crate**: its README, every file's banner, `//!` labels and Part order, tests in
      `foo/tests.rs`, `mod.rs` → `foo.rs` where cheap — one commit, smallest crates first (wire,
      base, syntax … http); objects and station as their §33 3d areas move. HEADERLESS goes down
      crate by crate.
19. **The station splits into crates** (asked 2026-10-15 after researching all of it): edges only
    point down, each new crate gets its README and headers as it moves (18). In steps:
    1. no untangle — `lumacore-tidy` (the source rules; paths from the workspace root),
       `lumacore-terminal` (REPL, TUI), `lumacore-engines` (the `PublishedShow` alias moves into it),
       `lumacore-protocols` (`join_until` moves down);
    2. small untangles — `lumacore-net` (the leaf I/O: socket, mDNS, PTP, wire, menu views),
       `StateHandle` generic in the state crate, the `Services` alias beside `ServiceAccess`;
    3. the LumaSession untangle — the net files that sit on the hub (shell, command port, handler,
       console bridge, sync) move to the server; LumaSession's fields bundle into one struct on
       `ServerContext`; a narrow `SessionHost` trait in `lumacore-lumasession`, implemented by the
       server (34b's pattern); then `lumacore-lumasession`, `lumacore-server`, `lumacore-sweep`
       (with `fuzzing`, so fuzz builds stop compiling the server);
    4. `lumacore-station` is thin: `main.rs`, the binaries, feature forwarding.
    tidy's `LAYERS` table goes as its modules become crates (EDGES says it); `mod.rs` files become
    `foo.rs` as they move.
    *Its plan is replaced by 20 (hardened the same day); `lumacore-terminal` was built under it.*
20. **The station holds no library code** (hardened 2026-10-15, one question at a time, after
    asking why tidy, playback and the server sit in the station): everything in it becomes a crate
    of its own, and the station is three files. Decided:
    1. **Binaries live with their crate**: `lumacore-server` in `lumacore-server`, `lumacore-sweep`
       in `lumacore-sweep`, `gen-demo` and `gen-docs` in `lumacore-objects`. The station keeps
       `main.rs` (the in-process `lumacore` REPL), its `Cargo.toml` (feature forwarding: http,
       perfetto, tracy) and its README; a tidy rule allows no other file in it.
    2. **The station moves to `apps/lumacore-station`**: `crates/` = every library crate, flat;
       `apps/` = everything you start (station, app, cli, scripts). The clients' rule is worded per
       client: app, cli and scripts compile only `lumacore-wire`; the station is the one app that
       builds the console.
    3. **`lumacore-dmx` splits along its own imports**: `lumacore-dmx` keeps the wire (Art-Net,
       sACN, the input merger, NICs, errors); **`lumacore-render`** gets the plan, compiler, render
       tick, engine thread, realtime, the bench show and the merge (`playback.rs` → `merge.rs`);
       the line drivers (`lines.rs`, which sample frames) go up into `lumacore-protocols`. Edges:
       dmx ← render ← protocols.
    4. **`PlaybackEngine` → `RenderEngine`** (it is the one renderer); "playback" stays for the
       data (`lumacore-playback`) and running playbacks. The wrapper stays in `lumacore-engines`
       with the perfetto and http wrappers: the `Engine` trait's context carries the render hub,
       so a wrapper inside render would be a cycle.
    5. **`lumacore-lumasession`** is the station ⇄ station crate (the object's name; no second word).
    6. **`net/` splits three ways**: the leaf I/O (socket, mDNS, PTP, wire framing) is
       **`lumacore-net`**; the hub files (shell, handler, spawn, console bridge, sync) go to
       `lumacore-server`; the menu rows go to views (7).
    7. **`lumacore-views` is the one source of what any UI shows**, on objects: menus (`menu_defs`
       plus their rows, from `net/menu_views`), pools (`objects::pool_views`), attribute board,
       encoder, detail, sequence and window types (`window_state`). The HTTP feeds and the TUI
       both read it; no client builds its own rows.
    8. **`lumacore-tidy`** is a test-only crate (paths from the workspace root), documented in
       **`docs/meta/tidy.md`**: one entry per rule — name (= the test), what it checks, why (§),
       the escape comment, and for a ratchet its constant and value. Checked both ways: a test
       without an entry, an entry without a test, or a ratchet number that differs from the code
       fails tidy. code-structure.md, testing.md and CLAUDE.md link it instead of repeating rules.
    9. **The trace macros have one door, `lumacore-trace`** (`#[macro_export]`; the copies in the
       station and `lumacore-dmx` go, COPIES counts it). A crate that zones forwards the tracy /
       perfetto features (a macro's `#[cfg]` is read in the caller's crate); render-thread zones
       stay allocation-free.
    10. **`lumacore-sweep`** takes `sweep/` and `fuzzing` (it sits on objects, lumasession and
        protocols, never the server).
    11. **Order, one commit each, every new crate with its README and headers (18)**: 34f1 tidy +
        tidy.md · 34f2 trace macros · 34f3 views · 34f4 dmx | render, RenderEngine · 34f5 engines,
        protocols (+ lines) · 34f6 net · 34f7 the `SessionHost` untangle (19.3) → lumasession,
        server, sweep · 34f8 the binaries to their crates, the station to `apps/` · **34f9 the
        structure to zero**: every file in the workspace (crates, apps, systemtest) has its banner,
        labels and Part order, every crate its README, no `mod.rs` is left; HEADERLESS, READMELESS
        and MOD_RS are pinned at 0, their exemption lists deleted (34g is folded in), crate by
        crate, smallest first.
21. **The catalog: one registry every crate adds to** (hardened 2026-10-16, one question at a time,
    after 34f3 found the menu table pinned in objects — its `Menu` verbs read it — and asked how
    large projects do it: rustc's `provide(&mut Providers)` and `LintStore`, nushell's `add_decl`,
    VS Code's contribution registries, Blender's `register_class` and RNA, Bevy's plugins and
    `TypeRegistry` push at startup; Zed's GPUI and Linux gather by the linker). Decided:
    1. **Push at startup** (rustc / nushell / VS Code / Blender), never gathered by the linker
       (`inventory` / `linkme`: undefined order, rows lost when a crate is dropped, toolchain
       trouble) and no static list in the crate on top.
    2. **Every declaration is a row**: commands, objects (props, pools, tree shape), menus, window
       types, feeds, error catalogs, global flags, tools, settings (keyed props), the built-in UI
       keys. `freeze()` checks across them: one namespace (§15), a menu column names a declared
       prop, a key line resolves, a window's feed exists.
    3. **Declaration and behaviour split** (VS Code's `package.json` + `registerCommand`, Blender's
       RNA + `execute`): **`lumacore-registry`** sits at the bottom (on base, no `AppState`) with
       pure-data rows, the store and `freeze()`; any crate adds rows and reads them (state's error
       lookup too — state stops depending on http for its catalog). Handlers are a separate table in
       dispatch (`lumacore-objects`): verb / word → `fn(&mut AppState, …)`, checked against the
       catalog — a built word has a handler, a `Planned` word has none, no handler without a row.
    4. **A handler names its declaration's static**: `pub static GROUP: ObjectDecl`, then
       `h.verb(&GROUP, Verb::Store, typed::<StoreGroup>)` — a typo doesn't compile; the typed
       handler struct (§33) stays the signature.
    5. **One process-wide frozen catalog**: `registry::install(r.freeze()?)` at start (the server,
       the sweep, gen-docs, each crate's tests through a testing helper that installs its own rows
       and everything below), `registry::get()` everywhere; reading before install panics with a
       clear message. Frozen for the run: no lock, no version. Plugins / words added at run time are
       a later § of their own (a layer checked against this one).
    6. **The catalog stays on the station**: the clients are thin (they draw what the station sends;
       completion, suggestions and help run on the station), so the vocabulary never goes over the
       wire. Every row is serializable; `/api/menu-types`, `/api/window-types` and the feed list
       become dumps of it.
    7. **Order**: it is 34f3, before views (which then becomes a plain move), table by table, one
       commit each: a errors · b flags · c feeds · d window types · e tools, settings, UI keys ·
       f menus · g commands and objects split into declaration and handler entries (mechanical:
       the handlers are untouched, the §33 typing goes on separately). The rest of 34f shifts by
       one (20.11's 34f3–34f9 are now 34f4–34f10).
    8. **A test binary installs its crate's catalog before its first test** (hardened 2026-10-16
       after comparing rustc / Bevy (each test builds its context), the logging crates (an
       idempotent `try_init` per test, `#[test_log::test]`) and `ctor`): Rust's test harness has no
       "before all tests" hook, and a per-test call that a test forgets passes or panics by test
       order. So every crate on the registry has, in its `lib.rs`, a test-only
       `#[cfg(test)] #[ctor::ctor] fn test_catalog() { lumacore_registry::install_with(crate::register) }`
       (`ctor` a dev-dependency; the product never has it and keeps its explicit install). The
       workspace lint `unsafe_code` is `deny`, not `forbid` (ctor's expansion allows it); tidy
       `no_unsafe_code` refuses any `unsafe` written in the source, and
       `crates_on_the_registry_register_and_install` that a crate on the registry has its
       `register` and its hook (and that a crate on state or objects is on the registry).
    9. **g comes before f; a row names another row by its static** (asked 2026-10-16 at 34f3f: a
       menu's grid names the object its rows are): commands and objects become rows first (g1
       commands, g2 objects), then a menu holds `&'static ObjectDecl` like today's
       `&'static ObjectDef` — a typo doesn't compile, the 21.4 rule for rows naming rows too.
    10. **The properties stay beside the row** (asked 2026-10-16 at g3): a `Property`'s `Access`
        (get / set on `AppState`) and its `Props` (keyed rows, apply, ask, after, within, view) are
        behaviour, and every reader of them sits in the objects crate, so they stay on the
        `ObjectDef` (the settings among them); the check that a menu column names a declared
        property is a test (`column_json` for every column; in `lumacore-views` since 34f4, which
        reads them through `def_of`) instead of a freeze check.
        They become rows when something outside objects (a plugin, the views) has to declare or
        read them (roadmap).
22. **Each protocol is a crate that declares itself; `lumacore-protocols` is only the interface**
    (hardened 2026-10-16, one question at a time, before 34f6, after asking how large projects do
    it. OLA has the E1.31 wire in `libs/acn`, the small `olad/plugin_api` (Plugin, Device, Port,
    DmxSource) and `plugins/e131` on those two, loaded by the daemon. QLC+ has one
    `QLCIOPlugin` (`capabilities()`, `openOutput`, `writeUniverse`, `openInput`, per-line
    `setParameter`, `valueChanged`), and `artnet`, `E1.31`, `osc` and `midi` are plugins of it.
    Companion modules extend `@companion-module/base`. sqlx drivers sit on `sqlx-core`, never on
    `sqlx`. rustls providers are installed once in `main`. The common rule is that a plugin sits on
    a small interface below the engine, never on the engine, and the binary wires the plugins in.
    It replaces 20.3's "lines go up into protocols" and 20.6's net timing). Decided:
    1. **Layers**:
       - **`lumacore-net`**: the leaf I/O. It holds NICs and bind / multicast on a NIC (from
         dmx `nics.rs` and the protocols' `socket2` code), plus socket, mDNS, PTP and the wire
         framing (20.6).
       - **`lumacore-protocols`**: the interface, like `plugin_api`.
         - The `ProtocolDef` catalog row (21): name, aliases, kind, directions, default port,
           capabilities, and its declared line props.
         - The driver traits.
         - Everything protocol-agnostic from `lumacore-dmx`: `Packet`, `SourceKey`, `SourceName`,
           `InputMerger`, `InputLayer`, `LinePlan`, `InputPlan` and the errors.
         - **`lumacore-dmx` folds into it and goes away.**
       - **`lumacore-artnet`**, **`lumacore-sacn`** and **`lumacore-osc`**: each holds its wire on
         net + protocols and adds its row in its `register`.
       - **`lumacore-lines`**: the runtime, like `olad`. It holds today's `lines.rs` (frame
         sampling, rate, keep-alive, input layers to `RenderHub`) and the station's `protocols/`
         (`ProtocolRuntimes::reconcile`). It sits on render + protocols, finds a driver by its row,
         and never names a protocol: the `enum Sender` and its `match` arms go.
       - The station's `install_catalog()` registers the protocol crates (sqlx's umbrella,
         rustls' `install_default`). Nothing else lists them.
       - Edges: net ← protocols ← {artnet, sacn, osc}; protocols ← render ← lines. Art-Net and
         sACN never see render. Objects read `nics` (net) and `LinePlan` (protocols).
    2. **Two kinds** on the row:
       - **Dmx**: universes out, input layers merged per channel in the tick.
       - **Control**: OSC today, and later MIDI, MSC and timecode as more crates. Messages
         become command lines run as the line's user, through the `InboundEvent` channel.
       - One `Line` type and one catalog, with a small trait per kind.
    3. **A line stores its protocol by the row's name** (`"sACN"`, `"ArtNet"`, `"OSC"`), like a
       window's type:
       - the tree's `OutputProtocol` enum goes;
       - `/Protocol` values, completion and the reference come from the rows;
       - a line naming a protocol this build lacks is kept, never sends, and is listed by the
         reference check (never dropped);
       - show format 26, forward only.
    4. **A line's settings: common typed, the rest declared.**
       - `Line` keeps typed what the runtime uses for every line of a kind:
         - all lines: network, target, enabled, direction;
         - Dmx: rate, send, packet gap, input priority, and sync (allowed by a `Sync`
           capability);
         - Control: the user, since rights need it.
       - What one protocol alone has (sACN priority, OSC port) lives in `Line.settings` (JSON),
         declared as props on its `ProtocolDef`. This is 12c's window-settings pattern, so `Set`,
         the grid columns and the reference read them through the one property registry (§33 58).
    5. **Defaults, not asked**:
       - Art-Net's ArtPoll nodes are a generic `Discovery` capability: the driver reports
         nodes (ip, name, universes, refresh rate), and the line's rate cap reads them.
       - Error codes keep their codes, each in its crate's catalog.
       - The OSC listener moves as it is.
    6. **Order** (replaces 20.11's tail, everything one commit each with its README and headers,
       18):
       - **34f6**: all of `lumacore-net`.
       - **34f7a**: the crates and the traits, behaviour unchanged (dmx → protocols,
         artnet / sacn / osc, lines).
       - **34f7b**: the show side (`Line.protocol` by name, `Line.settings`, `/Protocol` from the
         rows, format 26).
       - **34f8**: `lumacore-engines`.
       - **34f9**: the `SessionHost` untangle → lumasession, server, sweep (+ fuzzing).
       - **34f10**: the binaries to their crates, `apps/lumacore-station`.
       - **34f11**: the structure to zero.
    7. **A row whose type lives above the registry** (found while building 34f7a): a
       `ProtocolDef` holds its opener (a fn taking protocol types), so it can't be a registry
       type. The registry keeps such rows untyped and hands them back by type —
       `Registry::rows(&'static [T])`, `Catalog::rows::<T>()` — and runs a crate's
       `Registry::check(fn(&Catalog) -> Vec<String>)` at freeze (protocols: one name, one
       protocol; the opener fits the kind). Still one catalog, pushed at startup (21.1); the
       behaviour sits beside its row like an `ObjectDef`'s.
23. **Crates live in folders by domain** (hardened 2026-10-16, one question at a time, from the
    user's sketch, after comparing flat workspaces (Bevy's `crates/*`, Zed, Ruff, nushell) with
    grouped ones: Deno (`cli/`, `runtime/`, `ext/` with one crate per API, `libs/`, `tests/`,
    `tools/`), rust-lang/rust (`compiler/rustc_*`, `library/`) and gst-plugins-rs (`audio/`,
    `video/`, `net/`). Deno also shows the cost of folder names drifting from package names:
    `ext/bundle` holds the package `deno_bundle_runtime`). Decided:
    1. **Eight groups under `crates/`, by what a crate is about, not by its layer.** `EDGES` and
       the crate map keep the layers, and a group can hold crates of several layers:
       - **`foundation/`**: base, registry, trace, wire, syntax, syntax-derive;
       - **`show/`**: tree, runtime, playback, effects, timecode, patch, state, objects;
       - **`formats/`**: gdtf (later luma, cut out of state's `show_file`, and mvr);
       - **`engine/`**: render, engines;
       - **`network/`**: net and the `protocols/` family;
       - **`ui/`**: views, terminal, http;
       - **`server/`**: server, lumasession;
       - **`tools/`**: tidy, banner, sweep.
       What the sketch had that is no crate: commands and verbs are folders inside
       `lumacore-objects` (decision 16), and the window types are registry rows. No group is
       named "runtime" (that is `lumacore-runtime`, the `Root.Runtime` types).
    2. **A family gets a sub-folder: only `network/protocols/`.** A family is siblings on one
       interface (Deno's `ext/`). It holds the interface (`lumacore-protocols`), its runtime
       (`lumacore-lines`), and its plugins by their row's `Kind`:
       - `dmx/` holds artnet and sacn (later rdm, kinet);
       - `control/` holds osc (later midi, msc, ltc).
       There is no `vendor/`: tools read that name as copied third-party code (`cargo vendor`,
       GitHub's linguist), and sACN is an ESTA standard. Two levels (group, crate) everywhere
       else; more only inside a family, one level per meaning.
    3. **The folder is the full package name**: `crates/network/protocols/dmx/lumacore-sacn`.
       `-p`, grep, paths in errors and the README title all read the same.
    4. **Tidy**: `crates_are_named_by_category` becomes `crates_sit_in_their_group`. Every crate
       is `crates/<group>[/<family>…]/lumacore-<name>`, the path listed once in `GROUPS` (with
       each group's purpose, in tidy.md and the crate map). `lumacore-station` is the one crate
       outside a group until it moves to `apps/` (20.2).
    5. **`tests/`** holds systemtest and fuzz too, beside e2e.
    6. **One door to the repo root**: code and tests that read files beside the crates (the
       demo show, lum.md, fixtures, the frontend kit) use `env!("LUMACORE_ROOT")`. It is set in
       `.cargo/config.toml` (`[env]`, relative), never counted as `../..` from a crate.
    7. **Now, as 34f8**, before the crates still to come, so they are born in their group:
       - 34f9 `lumacore-engines`;
       - 34f10 the `SessionHost` untangle → lumasession, server, sweep;
       - 34f11 the binaries, and the station to `apps/`;
       - 34f12 the structure to zero.
       34f7b's place is still open (proposed: with the §33 3d pass of the protocol objects).
24. **The station is the assembler, and its code becomes crates in the groups** (hardened
    2026-10-16, one question at a time, after 34f8; compared with Zed's `crates/zed` and Deno's
    `cli`, each the one crate that assembles the program, and rustc's `rustc_driver` behind a
    `rustc` binary of a few lines). Decided:
    1. **`apps/lumacore-station` builds the program and holds no library code.** It keeps its
       `Cargo.toml` (every crate it builds with, the features http · perfetto · tracy), its
       README, `catalog.rs` (`install_catalog`: the one list of every crate's `register`) and a
       thin `main` per binary in `src/bin/`: `lumacore` (the REPL), `lumacore-server`,
       `lumacore-sweep`, `gen-demo`, `gen-docs`. Each installs the catalog and calls one library
       function. **This replaces 20.1** (binaries with their crate): every binary needs the whole
       catalog, and a crate owning one would have to depend on every crate, a second
       assembler. A build without the terminal or HTTP is then a feature choice there.
    2. **`crates/` keeps 34f8's layout**; the station's code moves into crates in its groups, and
       only `server/` is a new folder (it was in 23.1):
       - `engine/lumacore-engines` ← `engines/`;
       - `network/lumacore-clocksync` ← `lumasession/clock.rs` (the session clock discipline:
         burst, RTT gate, median, regression, corrections, holdover);
       - `foundation/lumacore-lanes` ← the generic half of `server/state.rs`: `StateHandle<C>`,
         the shell and machine lanes, `call` / `call_within` / `send`, `CallError`, the busy
         watchdog (19.2);
       - `ui/lumacore-shell` ← `net/shell.rs` + `net/sync.rs` (the TCP shell port hosting the
         terminals, on a `StateHandle` and a small host trait);
       - `server/lumacore-lumasession` ← `lumasession/` (link, election, stream, replication;
         the `SessionHost` trait, 19.3);
       - `server/lumacore-server` ← `server/` + `net/`'s hub files (handler, spawn, the console
         bridge): `ServerBuilder`, `ServerContext`, the state loop's body, the command port;
       - `tools/lumacore-sweep` ← `sweep/` + `fuzzing.rs` (20.10).
       The `core` re-export goes. Tidy's `UNGROUPED` goes, so every crate under `crates/` is in a
       group.
    3. **The folder moves to `apps/` now**, before it is emptied (replaces 20.2's timing). Until
       34f11 it is the one app that still holds library code; the clients' rule (wire only)
       names it.
    4. **Order, one commit each**:
       - 34f8b: the station's folder to `apps/`;
       - 34f9: engines, clocksync, sweep (moves, no tangle);
       - 34f10a: lanes (`StateHandle` generic);
       - 34f10b: shell;
       - 34f10c: `SessionHost`, then lumasession;
       - 34f10d: server;
       - 34f11: the station is only the assembler;
       - 34f12: the structure to zero.
       The larger cuts outside the station (`formats/lumacore-luma` from state's `show_file`, and
       objects' areas) belong to the §33 3d pass.
25. **The objects crate splits: a command kernel and a crate per domain** (hardening 2026-10-16,
    one question at a time, after 34f11; researched with four read-only agents: BlinderKitten's
    tree; rust-analyzer, rustc, nushell, Zed, Bevy, uv, ruff, Deno, Helix, Servo; Blender, Godot,
    VS Code, LLVM, Chromium, Firefox; and a coupling count of objects, state, tree and http).
    Found: size alone never decides a split (nu-command ~155k lines, ruff_linter ~193k are one
    crate each); splits pay for parallel builds, isolated features with a narrow interface, and
    a minimal subset — and the recipe is a small shared kernel first (nushell's `nu-cmd-base`,
    uv's `uv-command-support`, rust-analyzer's `ide-db`, Blender's windowmanager), then a crate per
    family with one file per verb (Blender's `editors/*/*_ops`, VS Code's `contrib/*`). In
    objects (127 files, 58.7k lines) the domains touch each other thinly (one to four `crate::`
    edges per pair); what blocks a cut are the kernel's edges up into them: `commands/registry.rs`
    (3,748 lines, the handlers of 22 modules), the `OBJECTS` list (statics of 13 modules) and
    about ten special cases in `dispatch.rs`. "Foundation" means knowing no domain (`//base`,
    `vs/base`, LLVM `Support`): objects is the hub above effects, playback and patch (none of
    which depends on it), so it never goes into `foundation/`; the declarations it reads
    (`CommandDecl`, `ObjectDecl`) already sit there in `lumacore-registry`. Decided:
    1. **Split, not only folders** (asked 2026-10-16; the alternative — one crate with category
       folders, ruff's / nu-command's way — stays the fallback if the cut families still need
       each other through dispatch): a kernel crate (dispatch, resolve, rights, claims, the typed
       `Command`, flags, lum, the tree shell's vfs / navigation) and one crate per domain, each
       pushing its own command and object rows in its `register` (`Registry::rows`), dispatch's
       special cases becoming flags or hooks on the rows. Every cut is that area's §33 3d pass
       (typed, moved, headed once), which replaces 34f12's split.
    2. **The command crates are a group of their own, `crates/commands/`** (asked 2026-10-16):
       above the show's data and logic, below `ui/`, `server/` and the station; never in the
       same folder as the data crates. The kernel stays there, above state — not in
       `foundation/`: dispatch touches `AppState` throughout (the change path, forwarding to the
       master, the panic guard, asks, masking, the rights gate against the user, resolving against
       the show); foundation already holds the generic half (the declarations in
       `lumacore-registry`, the grammar in `lumacore-syntax`), as VS Code's `platform/commands`
       holds only the id → handler table.
    3. **Programming and playback are two crates** (asked 2026-10-16, the established split): programming is
       what you set (selection, values, groups, presets, modes, the attribute board), playback is
       what runs (sequences, cues, masters, Go, echoes). Playback reads the programmer through
       programming's API (Store Cue is a playback verb); programming never names a sequence.
    4. **Key bindings are their own crate, below the users** (asked 2026-10-16): `Bind`, the key
       lines' checks on save and load, the core keys. Today's `keys.rs` names only the kernel
       (errors, text, tokens, the registry, `line`, sysmon), so the cut is clean; the users'
       crate may use it, it never uses the users'. The keys' data stays in the tree (`UiKeys`
       on the profile).
    5. **`crates/show/` becomes `crates/model/`** (asked 2026-10-16): what stays there is the data
       and its rules — tree (ShowData and Runtime), state, playback, effects, timecode, patch,
       runtime — no commands; "show" read too much like ShowData (`data/` sounds like files,
       `domain/` is vague, `core/` collides with Rust's and the old re-export).
    6. **Every command crate is `lumacore-cmd-<domain>`** (asked 2026-10-16, nushell's
       `nu-cmd-*`): the prefix says a crate holds commands, and each pairs with what it edits
       (`lumacore-playback` in model ↔ `lumacore-cmd-playback`, `lumacore-patch` ↔
       `lumacore-cmd-patch`, the line runtime `lumacore-lines` ↔ `lumacore-cmd-lines`). Still
       `lumacore-<category>` (17). "Rig" and "wiring" are not names: in lighting a rig is the
       hung fixtures — the patch.
    7. **The names, from what each crate holds** (asked 2026-10-16, checked against the generated
       reference): the kernel is **`lumacore-cmd-dispatch`** (a line → statements → the rights
       gate → resolve → the verb through the change path; the verbs every object shares: Set,
       Label, Copy / Move / Cut / Paste / Delete / Insert / Compact, Edit, EditSetting, Run, Help,
       Info, List, Dump, Oops / Redo, Claim / Unclaim, Clear, Self). The domains:
       - **`cmd-programmer`** — At, Select, On / Off, Full / Zero, Grid, Flash; Group, Preset,
         Attribute, FeatureGroup; Blind, Highlight, Freeze, Solo;
       - **`cmd-playback`** — Sequence, Cue, Master; Go, Go-, Goto, Pause, Stop, Top, Stomp;
       - **`cmd-patch`** — Fixture, FixtureType, Universe, DataPool; Patch / Unpatch, Park /
         Unpark;
       - **`cmd-protocols`** (for now) — Network, Output, Input and the network's Interface
         mapping; Assign / Unassign (a universe on a line), DMX In: what the Network and DMX
         Protocols menus edit ("lines" means nothing to an operator);
       - **`cmd-users`** — User, UserProfile; Login, Logout;
       - **`cmd-keys`** — UIKey, KeyBindings; Bind, Unbind, the core keys, the key lines' checks;
       - **`cmd-windows`** — Screen, Window, WindowPreset, WindowType, View, ViewButton, ViewBar,
         Menu, GridColumns, Sidebars, FullScreen, EncoderBar; Split, Sort, ReloadUI, Screenshot;
       - **`cmd-media`** — Asset, AssetRoot, Image, Icon, Paint, Theme, LockScreen;
       - **`cmd-devices`** — Device, DeskLock, NormalTheme, ScreenConfig;
       - **`cmd-showfile`** — Show, Version, Recovery; SaveShow, LoadShow, NewShow, DeleteShow,
         CleanStart, Import Show;
       - **`cmd-lumasession`** — LumaSession; Join, Leave, Invite, Dismiss, Claim LumaSession;
       - **`cmd-treeshell`** — cd, ls, pwd, cat, tree, find, grep, stat, diff, du, head, tail, wc,
         cp, mv, rm; Enter / Exit.
       "Desk" is no crate: it held three subjects (windows, media, devices). The station's own
       objects (Station, Interface, Clock, Health, Journal, Watch, Client, Shell, Server; Restart,
       Shutdown, Watch / Unwatch, Echo, HissMe) are **`cmd-station`** (asked 2026-10-16;
       singular like the rest, named after its main object, every station's row as `cmd-patch`
       holds every fixture). The splash and the `banner` command live there too (11); the
       terminal calls its `splash()` when the REPL starts.
    8. **No product crate depends on the developer tools' folder** (asked 2026-10-16, Chromium's
       `DEPS` keeping `//tools` and `//testing` out of `//content`): tidy, the header tool and the
       sweep are never a dependency of anything that ships; only the station (the sweep's thin
       main), systemtest and the fuzz crate may name them. A tidy edge rule holds it. The folder
       is **`crates/dev/`** (asked 2026-10-16; chosen over `tools/`, rustc's word, knowing "dev"
       is also Cargo's word for dev-dependencies and the dev profile — the rule, not the name,
       keeps them out of the product).
    9. **`crates/commands/lumacore-suggest`** (asked 2026-10-16): the suggestion engine (grammar
       first from the catalog and the show, nucleo ranking, per-user Usage; the web's
       `<luma-suggest>`, the TUI overlay, the sweep) beside dispatch, not in the domains' folder
       and not `cmd-` (it has no commands; rust-analyzer's completion crate). Dispatch's call to
       `suggest::record_usage` becomes a hook.
    10. **The cuts below the commands** (asked 2026-10-16):
        - **`formats/lumacore-luma`** — state's disk side: `show_file`, `drive`, `backups`, the
          media store, the station config file (~3.3k); state keeps AppState, the change path and
          Oops (24.2's plan);
        - **`foundation/lumacore-sysmon`** — `sysmon`, `feedback`, `panic_report` (~1.2k): the
          System Monitor is a logging system, so it knows no domain and sits in foundation (LLVM's
          `Support`, Chromium's `//base/logging`; asked 2026-10-16). Every crate can log to it —
          lanes' watchdog and clocksync drop their `warn: fn(String)` callbacks; the render tick
          still never logs. Two small moves: feedback's one function that reads a user's
          preferences stays in state, and a panic report's E-code text comes from the catalog
          (`lumacore-registry`) instead of state's `errors`;
        - **`lumacore-fixtures`** — tree's fixture data: `fixture_type`, `fixture`,
          `attribute_def`, `universe` (~1.3k, data only); the Network / Line types in tree's
          `output.rs` go with the protocols, not with it (14: the protocols move below the tree).
    11. **The two "banners" get two names** (asked 2026-10-16): the developer tool that draws the
        ANSI Shadow file and Part headers is **`lumacore-headers`** ("file header", "Part
        header" in the docs; `./scripts/unix/banner` follows); the startup art (the cat and the
        logo) is the **splash** (`splash.rs`). The `banner` command word is decided on its own.
    12. **The layout of `crates/commands/`** (asked 2026-10-16): the kernel
        `lumacore-cmd-dispatch` and `lumacore-suggest` at its top, and the 13 crates that hold
        objects and their verbs in **`commands/objects/`** — the console's own word (each declares
        objects like Cue and Fixture with their verbs; `network/protocols/dmx/` is the same
        pattern). `Go` is then `crates/commands/objects/lumacore-cmd-playback/src/go.rs`. Tidy's
        edges say it: everything in `objects/` stands on dispatch, dispatch names none of them.
    14. **`lumacore-protocols` moves into `model/`, below the tree** (asked 2026-10-16 when 34g2
        found the Line types couldn't go "to the protocols" above the tree — a cycle; researched
        with three read-only agents: QLC+ (`plugins/interfaces/qlcioplugin.h` below the document,
        which saves a plugin by name + parameters), OLA (the port API low, plugins high),
        BlinderKitten (the warning: its show tree holds the live drivers, patch and interface
        include each other), Godot (`servers/` interfaces and `*_types.h` below `scene/`), Linux
        (`net_device` + its ops in `include/`, drivers in `drivers/`), GStreamer, PipeWire,
        Chromium (`public/mojom`), Servo (`shared/*_traits`), Bevy (data crates under
        `bevy_render`), Zed (`lsp` below `language`); Blender alone keeps the engine interface
        above its DNA). The common shape: the contract — data, interfaces and the pure logic around
        them — low, the model holding it (by name), the implementations with sockets and threads
        high. `lumacore-protocols` holds no socket and no thread (the line data, `ProtocolDef`, the
        driver / listener traits, the input merger render's tick reads, `LinePlan`); decided:
        - it moves to **`crates/model/lumacore-protocols`** below the tree (its name kept, it
          pairs with `cmd-protocols`); a crate below the model doesn't sit in the I/O folder;
        - tree's `output.rs` moves into it (Network, Line, Direction, LineSend, InputPriority,
          Route, MergeMode, OutputProtocol until 34f7b) with its `Walk` / `Identified` impls; the
          `*_ops` over `ShowData` stay in tree; tree's `register` calls the protocols';
        - **`Nic` moves into it as plain data**; `lumacore-net` keeps listing NICs from the OS,
          sockets, mDNS, PTP and discovery, and depends on `lumacore-runtime` instead of the tree
          (its only tree use was the runtime's `HealthBoard` / `PtpStatus`); the one `now_ms`
          goes to `lumacore_base::clock` (four copies today);
        - **`crates/network/` means "has sockets or threads"**: net, clocksync and the
          implementations in `network/protocols/` (`lumacore-lines`, `dmx/` artnet · sacn,
          `control/` osc);
        - lines name their protocol by the row's name (34f7b, like QLC+ and Blender); fixtures
          (10) sits on it (a `Universe` holds `Route` / `MergeMode`). Decision 22's "only the
          interface" now reads: the protocol contract — the interface, the line data and their
          pure logic.
        The layers (each row uses only the rows above it; a type defined below the tree is still
        held *in* the show tree — `Root.ShowData.Patch.Fixtures` — as a `Vec<String>` holds
        strings defined below it):
        ```text
        foundation                      base, registry, sysmon, …
        runtime → protocols → fixtures  the types ShowData is made of
        tree                            ShowData (holds fixtures, lines, …)
        patch · gdtf · luma · render    logic and formats on the tree
        state                           AppState, the change path
        commands · network/             (net, lines, artnet, sacn, osc)
        ui · server
        station
        ```
    15. **The whole catalog's tests are a crate of their own, `tests/catalog`** (asked 2026-10-16
        when the tree shell's cut left the kernel's tests seeing only the kernel's words — the
        reference, the lum.md examples and the Lum words record failed, and a dozen row checks
        would have passed checking less with every cut): package `lumacore-catalog-tests`, beside
        systemtest and the fuzz crate, dev-dependencies only, its test binary on the station's
        catalog (`lumacore_station::register`). It holds the checks whose subject is every row
        (one namespace, help examples that parse, reading is anyone's, planned words, objects'
        verbs / properties / tree defs, pool members' names, the shim ratchet), the lum.md
        examples with `PENDING` and the Lum words golden record, and the reference check; a crate's
        tests of its own logic stay beside its code. Chosen over the station's own tests (the
        station only assembles) and over each crate running the row checks on its own rows (the
        same checks copied 14 times). The golden record of every tree path is one of them too
        (34g4 cut 3: it walks every object's tree, the keys' `UIKeys` among them).
    16. **Three homes for a test, by one question each** (asked 2026-10-16 when the station's cut
        broke 14 kernel tests that used Debug and Echo; the user asked whether every test should
        move to `tests/catalog`; researched with three read-only agents — rustc `tests/ui`, cargo's
        one `testsuite` binary (PR #5022: −79 % rebuild after a library edit, 5.8 → 1.1 GB),
        rust-analyzer, nushell, Deno spec tests, helix, ripgrep, wasmtime; VS Code's command
        registry and service tests, kubectl's `TestFactory`, Neovim `test/functional`, fish
        `tests/checks`, Blender, Godot, Kubernetes; LLVM lit vs `unittests/`, PostgreSQL regress,
        KUnit vs kselftest, SQLite, Chromium, Firefox; Google's test sizes, matklad's "Delete Cargo
        Integration Tests" and "How to Test"): no project keeps every test at one level — the
        behaviour tests (line in → reply out) are one central suite on the whole program, grouped
        by feature, the unit tests stay beside the code (private access, each layer tested, no
        rebuild of every layer above), and a registry's own machinery is tested with throwaway
        commands (VS Code registers `foo` / `test`). Measured here: 1,205 tests, 382 through
        dispatch, 307 of the kernel's 439; an edit rebuilds the kernel's tests in ~2 s and
        `tests/catalog` in ~4 s. Decided:
        - **a line that needs words from more than one command crate** is tested in
          `tests/catalog`, in feature modules (`words`, later playback, patch, …) — the kernel's
          dispatch tests move there feature by feature (roadmap), so a cut no longer breaks them;
        - **the kernel's machinery** (dialogs, the rights gate, flags, `;`, Oops) is tested with
          **stand-in words** only its test binary registers (`commands/test_words.rs`: TestAsk
          asks and needs Admin, TestEcho prints, TestInternal is the station's own level) — never
          in the product's catalog;
        - **everything else stays beside its code** (`foo/tests.rs`): unit tests and a command
          crate's own words, on the kernel and that crate. Later, from fish / Deno / LLVM:
          behaviour tests as plain files (a line and its reply) with a bless mode.
    17. **The rig wiring follows the console's two screens** (asked 2026-10-16 when the
        protocols cut found Station, Interface and Network named all over the kernel — Station's
        Lum shape, `settings_props`, three menus, DeskLock Station, Claim LumaSession Station;
        researched with two read-only agents: a reference console's network menu (Stations, Station Control,
        My Interfaces) beside its DMX Protocols menu, Eos's per-device network settings with
        per-NIC services beside its output protocols, Hog 4, MagicQ, QLC+, OLA, Disguise; Evans'
        shared kernel ("keep it small"), Vernon's reference-by-identity, Kubernetes labels,
        systemd-networkd / NetworkManager, Chromium's layered components). Every console keeps
        the station's own network (identity, session, its NICs) apart from the DMX protocol
        lines; nobody else has a show-level Network object. Decided (this changes 25.7):
        - **`cmd-station` holds the station and its network**: Station (the table, its
          identity settings, the wiring lines, remote station edits), Interface (the NIC and its
          services) and Network (the logical network and its NIC map), with their settings and
          the Network / LumaSessions menus;
        - **`cmd-protocols`, on `cmd-station`, holds the DMX lines**: Output, Input, Assign /
          Unassign, DMX In, the DMX Protocols menu, then 34f7b — a line names its Network as
          the reference protocol line sits under the station's network;
        - the "is it the Station?" checks move up with their words instead of a marker:
          **devices is cut first** (on `cmd-station` once Station is there), Claim LumaSession's
          crate sits on `cmd-station`;
        - order: devices → the station's network into `cmd-station` → `cmd-protocols` → 34f7b.
        - (found at the protocols cut) **Assign / Unassign stay the kernel's**: they are shared
          verb words like Store and Delete (a universe on a line, an attribute to a feature
          group), so their rows sit with the verbs every object shares; the Universe's Assign
          finds the lines by their collection (`objects::object_of(Node::Outputs)`, the row's
          declared tree node — a lower layer reads a declared value, never names the static).
    13. **The order** (asked 2026-10-16), one commit or a few per part:
        - **34g1 — renames**: `crates/show/` → `crates/model/`, `crates/tools/` → `crates/dev/`,
          `lumacore-banner` → `lumacore-headers` (and `./scripts/unix/banner`);
        - **34g2 — the cuts below the commands**: a `foundation/lumacore-sysmon` ✓, b
          `formats/lumacore-luma` ✓, c `lumacore-protocols` into `model/` below the tree with
          tree's line types and `Nic` (14) ✓, d `model/lumacore-fixtures` ✓;
        - **34g3 — objects untangled from the inside, no new crate**: `registry.rs` split so each
          module owns its command rows, each module pushes its own object rows (no `OBJECTS`
          list naming 13 modules), dispatch's special cases become hooks on the rows;
          `cargo build --timings` once as the baseline;
        - **34g4 — the domain crates cut out one at a time, easiest first**, each its area's §33
          3d pass (typed, moved, headed, its README — once): suggest, treeshell, keys, showfile,
          lumasession, station (with the splash), protocols, patch, media, devices, windows,
          users, programmer, playback; **last, what remains of `lumacore-objects` is the kernel**,
          renamed `lumacore-cmd-dispatch` and moved to `crates/commands/` (how the station was
          emptied).
        34f12 ("the structure to zero") is folded in: each crate reaches zero as it is cut, and
        the ratchets are pinned at 0 when the last one lands. If a cut shows two families still
        needing each other through dispatch, they stay one crate with folders (1's fallback).
        34f7b (lines by row name, format 26) goes with the protocols cut.
14. **Order**: 34a — this section, the document, the stale crate map and architecture.md fixed,
    the banner tool, the tidy rules (ratchets); 34b — the crate cut; then every §33 3d area is
    typed, moved into its object folder, given its headers and its shared doors in one pass, so
    each handler is touched once.

**Part 34b built (2026-10-15, branch `lum-syntax`): `lumacore-state` is its own crate.** Untangled
in place first (34b1–34b6, a tidy ratchet counting the references from the modules bound for the
state to what sits above them: 53 → 0): the registries' types moved to `defs`
(`command`, `object`, `view`), the flag declarations and the lexer down, Oops's snapshot /
restore half down (`oops`), the device table (`device_table`), the show paths into `backups`,
`LineOwner` into `app_state`; up went what reads the registry's contents — the pool window's
content (`objects::pool_views`), the Station / LumaSession verbs (`objects::station_verbs`),
`objects::default_object`, the after-load checks (`objects::after_show_replaced`, run by
`objects::replace_show` / `objects::load_show`), a setting's pick list, the ask buttons' flag
check; follow-ups the state can't run note what changed instead (`AppState::profiles_touched`,
refreshed above by `views::follow_touched_profiles`). Then 38 files (14,000 lines) moved to
`crates/lumacore-state` (with `build.rs`: the station reads the git stamp through
`build_info()`); the station's `core` re-exports its modules by name, so `crate::core::…` paths
hold. AppState methods that need what sits above (claims, navigation) became extension traits in
the station (`claims::Claims`, `navigation::Navigation`). Tidy's file rules read both crates (the
state's files count as `core/…`); the crate edges are now the compiler's.

**Part 34c built (2026-10-15, branch `lum-syntax`): `lumacore-objects` is its own crate.** `core/`
already named nothing above it (34b's untangle), so the rest of it moved in one piece: 122 files
(~58,000 lines — the registry tables, every object and verb, dispatch (decision 16), resolve,
rights, claims, lum, menus, settings, suggestions, the tree views) and `testdata/` to
`crates/lumacore-objects`, `crate::core::` written `crate::`, the crate root re-exporting the state's
modules by name. The station's `core` is `pub use lumacore_objects::*`. Tidy reads all three
crates under the `core/…` names (four rules had read only the station's `core/` since 34b — the
flags, the disk door, error codes, tree paths — and now read the state's files too); a header's
`use` may span lines. **34d**: nested lines count their depth; past `dispatch::MAX_NESTED_DEPTH` (32 — Vim allows 200, but
one of our nested dispatches costs more stack) a line is refused with E-CMD-019.

**Part 34f1 built (2026-10-15, branch `lum-syntax`): `lumacore-tidy` is its own crate** (decision
20.8). The station's `tidy.rs` (1,374 lines) and `tidy/structure.rs` became one file per area —
`words`, `layers`, `render`, `doors` (the door rules, the old resolvers, COPIES), `frontend`,
`structure`, plus `files` (the paths from the workspace root, the station's names for files, the
shared scanners and the ratchet check) — each with its banner, labels and Parts. The rulebook
[docs/meta/tidy.md](tidy.md) has one entry per rule (Checks, Why, Escape, Ratchet / Copies,
Self-test), and `the_rulebook_matches_the_rules` holds it to the code both ways. 45 tests (the 43
moved + 2); HEADERLESS 369 → 368. `lumacore-terminal` (decision 19) came first.

**Part 34f2 built (2026-10-15): the trace macros live in `lumacore-trace`** (decision 20.9). Its
`macros.rs` has a recording and an empty form of every macro, chosen by the trace crate's own
`tracy` / `perfetto` features, so callers need no feature of their own: the dmx crate lost its
`tracy` / `perfetto` features and `tracy-client`, the station's features forward to
`lumacore-trace/…`, both `telemetry.rs` copies are gone. A render stage's Tracy zone is a
`HeldZone` field set with `held_zone!`, its Perfetto event `record_complete` (never blocks). COPIES
counts `tracy_client::` outside the crate (0). The crate got its README, headers and
`foo/tests.rs` in the same pass (HEADERLESS 368 → 358, MOD_RS 21 → 20, old test files 54 → 51,
READMELESS 22 → 21).

**Part 34f3a built (2026-10-16): `lumacore-registry` and its first table, the error catalogs**
(decision 21). The crate (on base) has the builder (`Registry::enter`, `errors`, `freeze`), the
frozen `Catalog` (`error`, `errors`, `crates`; freeze checks every catalog and the codes across
them) and the one process-wide catalog (`install`, `install_with`, `get`, `installed`). Every crate
with codes (syntax, tree, patch, gdtf, dmx, http, state) has `register` in its `lib.rs`, after the
crates it builds on; objects and the station include what they build on (the station the HTTP
engine's with `http`). `lumacore_station::install_catalog()` runs in the server's start, the REPL,
gen-demo, gen-docs, the sweep and the fuzz entry points; a catalog that doesn't freeze stops the
start. State's `CATALOGS` is gone and so is its `http` feature (and objects': the rest were
dead-code gates); `render_code`'s debug check runs once a catalog is installed. Tidy's code rule
reads the catalog (its `http` feature adds the HTTP engine's rows).

**Part 34f3b built (2026-10-16): the global flags are catalog rows** (decision 21). `Verb` and
`FlagDef` / `FlagValue` moved into `lumacore-registry` (`verb.rs`, `flag.rs`; state re-exports them
where they were); the registry has `global_flags` (freeze refuses two global flags of one name or
short form; `Catalog::global_flags`, `global_flag`); state registers its `GLOBAL_FLAGS` (now
`pub(crate)`), and every reader — the line's flag check, the typed binder, the Lum vocabulary,
`flags::all`, the help's globals line — reads the catalog. Decision 21.8's test hooks went in with
it: every crate on the registry has its `register` and `test_catalog` hook (terminal got its
`register`), the lint is `deny` and two tidy rules hold it.

**Part 34f3c built (2026-10-16): the feeds are catalog rows** (decision 21). `FeedDecl` /
`FeedKind` are the registry's (`feed.rs`; freeze refuses a feed declared twice; `Catalog::feeds`,
`feed`); the HTTP engine registers its `FEEDS` as rows and binds each to its builder by name
(`Feed::name`, a match per feed; `Feed::def` reads the catalog); the `hello` lists the catalog's
feeds. Its tests hold the binding both ways (every `Feed` parses by its name, the `hello` lists as
many feeds as there are builders).

**Part 34f3d built (2026-10-16): the window types are catalog rows** (decision 21). Their types
moved into `lumacore-registry`: `setting.rs` (`SettingDef` and its builders, `SettingKind`,
`NowSource`, `FOLLOW_SELECTED`, `same_key`, `list_ids`) and `window_type.rs` (`WindowType` and its
builders, `ActionDef`, `ChipDef`, `BarMin`, the window bar keys, `section`); the catalog holds them
as built rows by number (freeze refuses a number or a name twice; `window_types`, `window_type`).
`lumacore-tree` keeps the table (`builtin_window_types`, now `pub(crate)`, registered by its
`register`) and the live side (`self_value`, `eval_state`, `render_template`, the view as the
`WindowTypeView` trait); `Root.Resources.WindowTypes`, `/api/window-types`, the live screens, the
window presets, the Lum vocabulary, the window state and the pool check read the catalog.
`window_types.rs` 1,344 → 839 lines.

**Part 34f3e built (2026-10-16): the tools and the built-in UI keys are catalog rows** (decision
21). `Status` moved into the registry (`status.rs`; state re-exports it). Tools: `ToolDecl` /
`ToolAsk` (`tool.rs`) with the toolbar groups (freeze refuses a tool twice or outside a group);
the objects crate registers its `TOOLS` rows and binds each built one to its code by id
(`tools::run_of`; a test holds it both ways); `find`, the toolbar JSON and `Grid … "<Tool>"` read
the catalog. UI keys are show data (`UiKey` lives in profiles and DataPool buttons, with its
identity walk), so the catalog's row is the pure-data `KeyDecl` (`key.rs`; freeze refuses a key
twice): `lumacore-tree` registers its core keys and turns the catalog's into the locked `UiKey`s of
`Root.Resources.UIKeys` (`UiKey::from_decl`, `builtin_ui_keys`). **The settings (keyed
properties) go with the objects in g**: they are the Station / Interface / Network / Show / About
objects' props, so they move with the object rows.

**Part 34f3g1a built (2026-10-16): the command words are the registry's** (decision 21, 21.9).
`lumacore_registry::command` holds `Doc`, `HelpCategory`, `AliasDepth`, `Undo` (with `tracked`,
`main` and the common ones), `Scope`, `Args`, `Rights`, `CommandClass`, `Reach` and
`CommandFamily` (pure data; `Rights::label` spells the levels itself); state's `defs::command`
re-exports them and keeps `CommandDef`, `Run`, `planned_run` and the help's colours
(`HelpColour`, colours stay out of the catalog); `From<UserRights> for Rights` moved to tree.

**Part 34f3g1b built (2026-10-16): the commands are catalog rows** (decision 21). `CommandDecl`
(every field but the handler; `BASE`, `label`, `names`) is the registry's; state's `CommandDef` is
the pair `{ run, decl }` and derefs to its row, so the 115 entries of `COMMAND_REGISTRY` (now
`pub(crate)`) read `CommandDef { run: …, decl: CommandDecl { …, ..CommandDecl::BASE } }`. The objects
crate registers every `decl`; `Word::Command` holds a `&'static CommandDecl`, `all_names`,
`all_words`, `lookup`, `lookup_any`, `visible_commands`, the sweep and tidy read the catalog, and
dispatch runs a row's handler with `registry::run_of` (found by the row's address — a row and its
handler are one `CommandDef`). Freeze refuses two commands of one word. Tidy's stated-field rule
reads `run` beside the row and the row's fields in `decl`.

**Part 34f3g2 built (2026-10-16): the objects are catalog rows** (decision 21). `ObjectDecl`
(keyword, Lum shape, default verb, aliases, help, family, call undo / class, scope, reach, rights,
flags, planned verbs) and `LumShape` (its children are rows: `&'static [&'static ObjectDecl]`)
are the registry's; state's `ObjectDef` is the pair `{ decl, pool, verbs, delete_ask, complete,
props, tree }` and derefs to its row. The 47 `ObjectDef` literals read `decl: ObjectDecl { … }`
beside their behaviour; a child list names `&X.decl`; `objects::def_of(row)` finds a row's
behaviour by address in `OBJECTS`, and `bare_verb` moved to `objects::bare_verb` (a child's own
bare verb needs its handlers). The objects crate registers every row; freeze holds one namespace
across commands and objects (§15) and refuses a child that isn't in the catalog. The properties
(the settings among them) stay beside the row: their `Access` is behaviour, so a `Property`
splits into a row and its get / set before they move (34f3g3, roadmap). `objects/system.rs` (806
lines with the nesting) is in `LONG_FILES` until §33 3d gives each object its folder.

**Part 34f3f built (2026-10-16): the menus are catalog rows** (decision 21, 21.9). `MenuDef` and
its parts (`MenuKind`, `Left`, `Power`, `Look`, `Tab`, `Content`, `Grid`, `Detail`,
`DetailGroup`, `Part`, `PartContent`, `Col`, `Own`, `Action`, `Bound`, with `tab` /
`grid_keys`) are the registry's (`menu.rs`); a grid names its object's row
(`object: Some(&INTERFACE.decl)`). The objects crate keeps the table (`MENUS`, now `pub(crate)`,
and its `const fn` builders) and registers it; freeze refuses a menu twice and a grid whose object
isn't in the catalog; `menu_defs::find`, `grid_keys`, `List Menu`, the menu completion and
`/api/menu-types` read the catalog, and the JSON builders find a grid object's behaviour through
`objects::def_of`. **34f3 is done** (the properties stay beside the row, 21.10).

**Part 34f4 built (2026-10-16): `lumacore-views`** (decision 20.7), on objects, tree, dmx: `menus`
(the menu JSON from `menu_defs`: `menu_types_json`, `menu_json`, `column_json`, `slab_json`),
`menu_rows` (from the station's `net/menu_views`), `pools` (`objects::pool_views`),
`attribute_board` (`view`; `BoardTarget` and `owner_words` stay in objects for the `Edit Attribute`
lines), `detail`, `encoder`, `sequence` (the `_view` files) and `window_state`; each file with its
headers, its tests in `foo/tests.rs`. The station's console bridge and state thread read it. The
view tests that lived in objects moved with it (on the demo show or a small station); the ones that
only drive commands stayed. Nothing changed in what a feed sends.

**Part 34f5 built (2026-10-16): `lumacore-dmx` | `lumacore-render` | `lumacore-protocols`**
(decisions 20.3, 20.4). `lumacore-dmx` keeps the wire (artnet, sacn, input, nics, errors; `artnet.rs`
/ `sacn.rs` instead of `mod.rs`); `lumacore-render` gets plan, compiler, render, engine, realtime,
the bench show and the merge (`playback.rs` → `merge.rs`), the budget tests (`engine/tests.rs`) and
`benches/render.rs`; the line drivers (`lines.rs`) went straight to a new `lumacore-protocols`, so
they move once — the station's `protocols/` joins them in 34f6. `LinePlan`, pure data like
`InputPlan`, stays in dmx (`line_plan.rs`): the objects crate's `protocol_plan` names it, and
objects must not sit on the drivers (34f6 puts `protocols/`, which reads objects, on top of them).
`PlaybackEngine` → `RenderEngine` (`engines/render.rs`, name `render`), its hub `PlaybackHub` →
`RenderHub` (`EngineContext.render`, `Published.render`; 20.4 calls it the render hub), the
render thread's health row `Render.Playback` → `Render.Tick`, its OS thread `render_engine`;
"playback engine" in help and docs is "render engine" (the reference regenerated). Edges: dmx ←
render ← protocols; state sits on render, objects and views on render + dmx. Every file of the
three crates has its headers and its tests in `foo/tests.rs`; each crate its README. systemtest
installs the catalog now (missed in 34f3a) and two of its steps follow lum.md (a key's line in
backticks, the resolver's "No such Fixture"). MOD_RS 20 → 17, HEADERLESS 353 → 333, OLD_TEST_FILES
51 → 50, READMELESS 21 → 20.

**Part 34f6 built (2026-10-16): `lumacore-net`** (decisions 20.6, 22.1). The leaf network I/O
left the station and dmx: `nics.rs` (+ its tests) from `lumacore-dmx`; `socket.rs`, `mdns.rs`,
`ptp.rs` (tests in `ptp/tests.rs`) and `wire.rs` → `discovery.rs` (`DiscoveryPeerState`, now
`pub`) from the station's `net/`. It sits on tree and wire only and has no catalog rows (no
`register`). `socket::send_reply` takes the oversize refusal's text from its caller, so the error
renderer stays in the state crate (the station's `net/spawn.rs` renders E-WIRE-002). Users name
`lumacore_net::…`; dmx, protocols, objects, views, the station and systemtest depend on it, views
no longer on dmx, the station no longer on `mdns-sd`. Edges, READMEs and the crate map follow;
HEADERLESS 333 → 329.

**Part 34f7a built (2026-10-16): the protocol crates** (decisions 22.1, 22.2, 22.7). `lumacore-dmx`
is gone: its input merger, `LinePlan`, errors and E-DMX codes are in `lumacore-protocols`, which
is now the interface — `protocol.rs`: `ProtocolDef` (name, kind, directions, port, capabilities
`Sync` / `Discovery` / `SharedPort`, `Open::Dmx | Control`), `DriverIdentity`, `Node`,
`ControlStart`, the `DmxDriver` and `ControlListener` traits, `all` / `find` and the freeze check.
`lumacore-artnet` (source + `ArtNetDriver`, row `ARTNET`), `lumacore-sacn` (source, receiver +
`SacnDriver`, which now opens and retries its own input socket, row `SACN`) and `lumacore-osc`
(the station's `protocols/osc.rs`, row `OSC`) sit on protocols + net. `lumacore-lines` holds
`lines.rs` (a `Box<dyn DmxDriver>`, no `enum Sender`; the send path, targets, rate caps from
`Node`s and the input loop unchanged) and the station's `protocols/mod.rs` as `runtimes.rs`
(`ProtocolRuntimes` opens every protocol through `protocol::find`; the unpinned rule reads
`SharedPort`, an OSC line is found by `Kind::Control`). Render sits on protocols; `join_until`
moved to `lumacore_base::threads` (19.1). The station registers lines, artnet, sacn and osc; its
`protocols/` module and tidy layer are gone. Left for 34f7b, where the enum goes: drivers keyed by
`OutputProtocol`, `desired`'s `osc` map, `AppState::sacn_line_count` / `sacn_running`, OSC's
default port written in tree, `Runtime.Network.ArtNetNodes`. MOD_RS 17 → 16, HEADERLESS 329 → 327,
READMELESS unchanged (each new crate has its README); systemtest 200/200.

**Part 34f8 built (2026-10-16): the crates sit in their folders** (decision 23). Every crate moved
by its folder (caches included), and every relative path was re-resolved from the old layout: the
`path =` deps of every Cargo.toml, the markdown links, the `crates/lumacore-x` strings in code,
scripts and docs (the history left as it was: `docs/archive`, `handoffs/done`, `issues/`, this
file). systemtest and fuzz are `tests/systemtest`, `tests/fuzz` (the sweep's `--fuzz-dir` too). The
repo root is `env!("LUMACORE_ROOT")` (`.cargo/config.toml` `[env]`, relative): the demo builds in
tests, lum.md, the GDTF fixtures, the frontend kit, tidy's workspace and systemtest's harness and
build script read it instead of `../..` (systemtest's manifest now hashes files that exist: the
objects crate's registry and dispatch, the demo show). Tidy finds a crate through `GROUPS`
(`crate_root`, `crate_roots`); `crates_are_named_by_category` became `crates_sit_in_their_group`.
`.gitignore` reads `crates/**/library/`. No code changed otherwise; systemtest 200/200.

**Part 34f8b built (2026-10-16): the station's folder is `apps/lumacore-station`** (decision 24.3).
A folder move like 34f8 (its paths re-resolved, its tracked library and logs with it); tidy's
`UNGROUPED` is gone, `crate_root` finds an app in `apps/`, and every crate under `crates/` sits in a
group. Its code leaves in 34f9–34f10.

**Part 34f9 built (2026-10-16): `lumacore-engines` and `lumacore-clocksync`** (decision 24.2).
`engines/` is `crates/engine/lumacore-engines`: `registry.rs` (the old `mod.rs`: `Engine`,
`EngineContext`, `EngineRegistry`, `http_ui_engine_state`, and `PublishedShow`, which moved in from
`server/state.rs` per 19.1), `render.rs`, `http.rs` and `perfetto.rs` under the crate's own `http`
/ `perfetto` features (the station forwards its features), `tests/engine_registry_tests.rs`.
`lumasession/clock.rs` is `crates/network/lumacore-clocksync` (`clock.rs`, tests in
`clock/tests.rs`). Its one System Monitor line comes back through a `warn: fn(String)` the station
passes to `spawn_clock_sync`, so it sits on base, tree and the wire only; callers name
`lumacore_clocksync::…`. The station's `engines` layer is gone from tidy's `LAYERS`.
**`lumacore-sweep` waits** (24.4 had it here): the sweep and its property tests call lumasession
(`link::op_order`, `replication::apply_replicated_op`, the election's `best`, the stream's frames),
so it moves after lumasession, as 34f10e. MOD_RS 16 → 15, HEADERLESS 327 → 322; systemtest
200/200.

**Part 34f10a built (2026-10-16): `lumacore-lanes`** (decisions 19.2, 24.2). The handle half of
`server/state.rs` is `crates/foundation/lumacore-lanes`, on base only: `StateHandle<C>` (`call`,
`call_within`, `send`, `machine`, `shell`, `busy_for`, `on_busy_change`), `Lane`, `MACHINE_LANE`,
`Lanes<C>` (the loop's receiving end) with `Busy` (the busy clock: `start`, `start_if_idle`,
`end`), `CallError<P>`, `Job<C>`, `enter_state_thread`, and the watchdog. A context is a `Host`:
`guarded` (its panic guard) and `panicked` (its health entry); a refused machine job and the
watchdog's line go to callbacks the server passes (`health.refused()`, `sysmon_early("State", …)`),
and the busy clock reads `lumacore_base::clock` directly. The server keeps its policy (the waits)
and names its instance: `impl Host for ServerContext`, `type StateHandle =
lumacore_lanes::StateHandle<ServerContext>`, `CallError<PanicReport>` (`PanicReport` got its
`Display`, the old text), so no caller changed. The crate's own tests run a small host (calls, a
panic, a full lane, a late job, a reentrant call); the station's state-thread tests stay with its
loop. systemtest 200/200.

**Part 34f10b built (2026-10-16): `lumacore-shell`** (decision 24.2). `net/shell.rs` is
`crates/ui/lumacore-shell` (`shell.rs`, tests in `shell/tests.rs`; still in `LONG_FILES`, 920
lines). `net/sync.rs` stayed with the server: it builds `Root.Runtime`'s network view and prunes
the discovery cache, not the shell's (24.2 had it here). The context a shell port serves is a
`ShellHost` (`host.rs`): `split()` (the terminals and the `AppState` at once), `refresh_runtime`,
`request_shutdown`, `sysmon`, `drain_sysmon_pending`; the gestures that were `ServerContext`
wrappers of `sessions.X(client, &mut state, …)` (attach, render, keys, key actions and gestures,
UI commands, taps, viewports, focus, live screen, scroll) are its provided methods, and
`ServerContext` lost them. `handle_shell_client` is generic over the host; the shell's own waits
(`SHELL_WAIT`, `SUGGEST_WAIT`) moved with it, `HTTP_WAIT` stays in the server. It builds on objects,
so it is on the registry (`register`, the test hook). HEADERLESS 322 → 321; systemtest 200/200.

**Part 34f10c built (2026-10-16): `lumacore-lumasession`** (decisions 19.3, 24.2). `lumasession/`
is `crates/server/lumacore-lumasession`: `link.rs`, `election.rs`, `stream.rs`, `replication.rs`
as they were; the old `mod.rs` split into `session.rs` (invite, dismiss, `Deferred`,
`run_deferred`, and **`LumaSession`**: journal, deferred, peers, replies, next_conn, sent_runtime,
sent_logins, watch — the session's fields, one struct, `ServerContext.lumasession`) and
`access.rs` (`ServiceAccess`, and the `Services` alias beside it, 19.2; `net::spawn` re-exports
it). **`SessionHost`** (`host.rs`) is what it needs from a station: `state` / `state_ref`,
`session` / `session_ref`, `split` (both at once), the ports, `request_from`, the discovery cache,
`state_handle`, and the station's own work as callbacks — `refresh_runtime`, `request_shutdown`,
`drain_sysmon_pending`, `shell_users` / `push_output` (so it never names the terminals),
`make_reply`, `execute_remote_command`, `handle_request` (the command port's handler stays in the
server). Every function that took `&mut ServerContext` is generic over `H: SessionHost`
(rewritten mechanically; the borrows that took two fields at once use `split()` or shared
`_ref` reads). The crate's pure tests (services, the election's order, the stream's frames) are
in `foo/tests.rs`; the three whole-server tests (link, election, remote edits) stay in the
station as `src/lumasession.rs` + `lumasession/*_tests.rs` until the server is a crate (34f10d).
MOD_RS 15 → 14, HEADERLESS 321 → 317, OLD_TEST_FILES 50 → 49; systemtest 200/200.

**Part 34f10d built (2026-10-16): `lumacore-server`** (decision 24.2). `server/` and the rest of
`net/` are `crates/server/lumacore-server`: `builder.rs` (the old `server/mod.rs`:
`ServerConfig`, `ServerBuilder`, `Server`, `serve`, `restart_process`), `state.rs`, `context.rs`,
`app.rs`, `net.rs` + `net/{handler, spawn, sync, console_bridge}.rs`, and LumaSession's
whole-server tests (`lumasession.rs` + `lumasession/*_tests.rs`); inline tests in `foo/tests.rs`.
**The server never installs the catalog** (24.1): `require_catalog` refuses to start without one,
and every binary of the station calls `lumacore_station::install_catalog()` first (the REPL's
`main` and `lumacore-server` gained the call; gen-demo, gen-docs, the sweep and systemtest had
it). Its `register` lists what it builds on (views, shell, lumasession, lines, http under its
feature); the station's is the server's plus the protocols. The station forwards its features
(`http`, `perfetto`, `tracy`) to the server's. Tidy: the station's `LAYERS` keep only `core`,
`terminal`, `sweep` (`HOSTS_TERMINALS` gone), and **`CUT_OUT` now lists every crate carved out of
the station** under its old folder name (lines and osc `protocols/`, engines, clocksync and
LumaSession `lumasession/`, lanes and the server `server/`, shell `net/`): since 34f7a the rules
that read "the station's files" (doors, words) had stopped reading the moved code — they read it
again, and it passes. MOD_RS 14 → 12, HEADERLESS 317 → 307; systemtest 200/200.

**Part 34f10e built (2026-10-16): `lumacore-sweep`** (decisions 20.10, 24.2). `sweep/` and
`fuzzing.rs` are `crates/tools/lumacore-sweep`: `world.rs`, `vocab.rs`, `lines.rs`, `run.rs`,
`fuzzing.rs`, the property tests in `src/tests.rs` (was `sweep/property_tests.rs`). It sits on
registry, tree, objects, LumaSession and osc — never the server. **The catalog:** the sweep never
installs the station's (24.1); `World::new` refuses without one. The `lumacore-sweep` binary stays a
thin main in the station (`install_catalog`, then `lumacore_sweep::run`), so the overnight run
sweeps the whole program; the property tests and the fuzz targets run with the sweep's own
`register` (LumaSession + osc and what they build on — every command and object is in objects).
The fuzz crate (`tests/fuzz`) depends on `lumacore-sweep` instead of the station: a fuzz build no
longer compiles the server. `scripts/unix/fuzz_long` runs the property tests as `cargo test -p
lumacore-sweep`. Tidy: the station's `sweep` layer is gone from `LAYERS` (only `core` and
`terminal` are left), the sweep is in `CUT_OUT` and `EDGES`. MOD_RS 12 → 11, HEADERLESS 307 → 300,
OLD_TEST_FILES 49 → 48; the sweep's 15 tests pass; systemtest 200/200.

**Part 34f11 built (2026-10-16): the station only assembles** (decision 24.1; talked through
first, compared with rustc — `main.rs` of a few lines over `rustc_driver`, itself an empty re-export
of `rustc_driver_impl` —, Bevy's facade crate and `DefaultPlugins`, Servo's servoshell over
libservo, and cargo, uv, ruff, Zed, Deno and rust-analyzer, whose binary crates keep argument
parsing and startup glue). Decided with it: one assembler and the work in crates, as everyone does;
the binaries keep their argument parsing (cargo / uv style) — thin means "parse, install the
catalog, call one library function", not "only `main`"; no facade for our own crates (Bevy's is
for outside users; inside, every crate names the crate it uses). Built: `lib.rs` → `src/catalog.rs`
(`[lib] path`), `main.rs` → `src/bin/lumacore.rs`, the `core` re-export and `pub use
lumacore_terminal as terminal` are gone — systemtest and the binaries name `lumacore_objects` /
`lumacore_terminal` (systemtest gains the objects edge and gets its catalog from the station). The
station's `Cargo.toml` keeps only what it uses (registry, objects, terminal, server, sweep, the
three protocols, assert_no_alloc, clap, log; 58 dependencies → 11), its features pass to the
server. Tidy: `layers_only_point_down` (§23's station layers) and its bare-`core::` check are
replaced by `the_station_only_assembles` (only `src/catalog.rs` and `src/bin/*.rs`); a crate on
the station needs no registry edge of its own; the station has its README. MOD_RS 11 → 10,
HEADERLESS 300 → 298, READMELESS 20 → 19; systemtest 200/200.

**Part 34g1 built (2026-10-16): the renames** (decision 25.5, 25.8, 25.11, 25.13). `crates/show/` is
`crates/model/`, `crates/tools/` is `crates/dev/`, `lumacore-banner` is `lumacore-headers` and its
script `./scripts/unix/headers`; every path dependency, tidy's `GROUPS`, the docs and the generated
reference follow (the docs say "file header" / "Part header"; tidy's rule names keep theirs). New
tidy rule `dev_tools_never_ship`: no crate outside `crates/dev/` depends on one in it, except the
station, systemtest and the fuzz crate. Tidy 47; systemtest 200/200.

**Part 34g2a built (2026-10-16): `foundation/lumacore-sysmon`** (decision 25.10). State's `sysmon.rs`
(now `monitor.rs`, its items at the crate's root: `lumacore_sysmon::sysmon_note`), `feedback.rs`
and `panic_report.rs` are the crate, on no workspace crate. State keeps what reads the show:
`show_feedback.rs` (`settings_json`, `publish_statuses`). `PanicReport::reply` became `text`;
dispatch renders it with E-INTERNAL-001. Lanes' watchdog and clocksync log directly (their `warn:
fn(String)` callbacks are gone). Callers name `lumacore_sysmon` (objects no longer re-exports the
three modules). HEADERLESS 298 → 295; systemtest 200/200.

**Part 34g2b built (2026-10-16): `formats/lumacore-luma`** (decision 25.10). State's disk side is the
crate: `show_file` (with `parse_show`, moved from `app_state`), `drive`, `background` (the worker
behind the disk door), `backups`, `media`, `station_config`, `build_info` (with the git stamp's
`build.rs`) and the E-SHOW / E-MEDIA codes (its `errors`, registered by its `register`). The eight
functions of `backups` that read `AppState` (`facts`, `recovery_copy`, `try_recovery_copy`,
`rules`, `startup`, `recovery_ask`, `station_facts` and the private `who`) are state's `recovery`.
`with_code` moved to `lumacore-base` beside `ErrorInfo` (state's `errors` names it). State keeps
`parse_show` and `build_info` as named re-exports at its root. Tidy reads the crate as `core/` (its
disk door rule holds). State lost zip, hostname, chrono and base64. HEADERLESS 295 → 289;
objects' 570 tests and systemtest 200/200.

**Part 34g2c built (2026-10-16): `lumacore-protocols` below the tree** (decision 25.14). The crate is
`crates/model/lumacore-protocols`; tree's `output.rs` is its `line.rs` (Network, Line, Direction,
LineSend, InputPriority, Route, MergeMode, OutputProtocol and the labels) with their `Identified` /
`Walk` impls (moved out of tree's `identity.rs`, the exhaustive destructure kept); `Nic` is its
`nic.rs` (net names it). It depends on base, runtime and registry only; tree depends on it and its
`register` calls the protocols' (render's calls the tree's). `lumacore-net` depends on base,
runtime and protocols, no longer on the tree. The nine wall-clock `now_ms` copies are one,
`lumacore_base::clock::now_ms` (lanes' local-clock one stays). Callers name `lumacore_protocols`.
HEADERLESS 289 → 288; every affected crate's tests (objects' 570, the server's 52) and systemtest
200/200 pass.

**Part 34g2d built (2026-10-16): `model/lumacore-fixtures`** (decision 25.10). Tree's `fixture_type`,
`fixture`, `attribute_def` and `universe` are the crate, on base and the protocols (a `Universe`
holds `Route` / `MergeMode`), with their `Identified` / `Walk` impls (`identity.rs`, the exhaustive
destructures kept). The identity macros (`meta_identified!`, `guid_identified!`,
`meta_only_walk!`) are base's now, one copy for every crate with show types. `lumacore-gdtf` sits
on base and fixtures and no longer on the tree. Tree's `identity.rs` left `LONG_FILES` (716 lines).
HEADERLESS 288 → 284; objects' 570 tests, the server's 52 and systemtest 200/200 pass.

**Part 34g3a built (2026-10-16): the command rows in families** (decision 25.13). `registry.rs`'s
one 115-row table is 13 family files in `commands/registry/rows/`, one per crate of 34g4 (dispatch
and inspection for the kernel; programmer, playback, patch, protocols, users, keys, windows,
showfile, lumasession, station, treeshell — `Sort` / `Split` are the kernel's, `Shortcuts` the
keys', `Connect` / `Disconnect` the station's). Each family's `ROWS` go into the catalog as typed
rows (`r.rows`) beside their declarations; every reader asks the catalog (`registry::all()`,
`run_of`), and nothing names the table any more. Tidy's `every_command_states_what_decides_it`
reads every family. Baseline before 34g4: an incremental station build after touching objects
takes 66 s wall, objects itself 6.9 s (`cargo build --timings`). `registry.rs` (656 lines) left
`LONG_FILES`; objects' 570 tests and systemtest 200/200.

**Part 34g3b built (2026-10-16): the objects in families** (decision 25.13). The `OBJECTS` list
(statics of 13 modules) is gone: each family file in `commands/registry/rows/` lists its objects
(`OBJECTS`) beside its commands (`ROWS`) — two new families, `media` and `devices`, hold only
objects; `Library` went to showfile, `Settings` to users, `GlobalVar`, `Undo`, `Error`, `Claims`,
`Reference`, `ShowData` and `Self` to dispatch. `registry::FAMILIES` pushes both kinds of rows
into the catalog; `objects::all()` and `def_of` read it. Objects' 570 tests and systemtest
200/200.

**Part 34g3c built (2026-10-16): dispatch's special cases are hooks** (decision 25.1, 25.13).
`commands/hooks.rs` holds one typed catalog row, `Hook`, and every family pushes its hooks beside
its rows (`Family::hooks`, `HOOKS` in its file); dispatch runs them and names no family any more.
The seven: `Gate` (devices: a locked device runs only its lock keys), `Intercept` (station:
`<line> /Watch`), `Used` (suggest: Usage ranks the words), `Changed` (windows: following the
profiles they show), `PatchKept` (programmer: the attributes follow a kept draft, `/Standard` /
`/Mine`), `DeleteNeverAsks` (showfile: Show; windows: ViewButton) and `SetUntracked` (windows:
GridColumns). The Default profile's lock stays dispatch's rule, so its message (`DEFAULT_LOCKED`)
moved there. The nine old unused imports in objects' `#[path]` tests are gone. Dispatch still
names the kernel's own pieces (claims, the change path, the System Monitor's change report).
Objects' 570 tests, the server's 52 and systemtest 200/200 pass; clippy is clean.

**Part 34g4 cut 1 built (2026-10-16): `crates/commands/lumacore-suggest`** (decision 25.9). Objects'
`suggest/` is the crate, the first in the new `commands/` group, on the kernel (objects) only; its
`Hook::Used` is pushed by its own `register`, which the terminal's calls (so the station's catalog
has it through the server). Objects' tests that called completion moved with it (three tests),
and the Tab completion section of the tree-path golden record is the crate's own golden file
(`testdata/tab_completion.golden.txt`, its lines identical; the tree-path record lost only those
27). The terminal and the sweep name `lumacore_suggest`. MOD_RS 10 → 9, HEADERLESS 284 → 282.
The disk filled during the build (errno 28): `target/debug/incremental` (43 GB) was deleted
again; `target/debug/deps` holds 119 GB. Objects' 553 tests, the suggest's 18, the server's 52 and
systemtest 200/200 pass.

**Part 34g4 cut 2 built (2026-10-16): `crates/commands/objects/lumacore-cmd-treeshell`** (decision
25.7) and **`tests/catalog`** (25.15). The first `cmd-` crate, in the new group
`commands/objects`: the rows of `cd`, `ls`, `pwd`, `cat`, `head`, `tail`, `stat`, `tree`, `find`,
`grep`, `du`, `wc`, `diff`, `cp`, `mv`, `rm`, `Enter` and `Exit` (the last two out of the dispatch
family), the fence words, what they print (objects' 851-line `treeshell.rs` split into `read`,
`tree`, `search`, `count`, `diff`, `print`), `mv` / `cp` / `rm` (`door`, was `door/shell.rs`) and
Exit's handler. Walking the tree stays the kernel's (25.1): `vfs`, `navigation` and
`commands::navigate` (Enter's claim, cd, the `ls` listing the quicksheet shows, `LS_RECURSIVE`).
Dispatch no longer names the tree shell: its `mv` / `cp` / `rm` write-out is the new
`Hook::Rewrite`, pushed by the crate. The terminal's `register` calls the crate's (the shells host
it; the station's catalog has it through the server), as do the sweep's and tidy's; suggest's
tests take it as a dev-dependency (completion takes whatever the catalog holds). Objects' tests
that ran the words moved with them (57, the golden record of the tree among them, byte for byte
apart from its header line); the kernel tests that only used them on the way call the kernel
(`navigate::cmd_enter`, `resolve_target`, `List`) and their tree-shell halves are five new tests
here. The checks over every row, lum.md and the reference moved to `tests/catalog` (18). The
demo builder enters Patch through `navigate::cmd_enter`. OLD_TEST_FILES 48 → 46, HEADERLESS
282 → 279. Objects' 478 tests, the tree shell's 62, the catalog's 18, suggest's 18, the server's
52, tidy's 47 and systemtest 200/200 pass; clippy is clean.

**Part 34g4 cut 3 built (2026-10-16): `crates/commands/objects/lumacore-cmd-keys`** (decision 25.4).
Objects' `keys.rs` (split into `lookup`, `lines`, `bind`), the UIKey object (out of `menus.rs`),
KeyBindings (out of `system.rs`, with `List KeyBindings` out of `list_misc.rs`), Shortcuts (out
of `run.rs`), the keys family's rows and the E-KEY codes (out of state's catalog: the crate's own
`ERRORS`). After a show replaced the last the kernel runs the new `Hook::ShowReplaced` instead of
naming the keys; the crate pushes its check there. The kernel's `listed_object!` is exported, so
KeyBindings is declared by it as before (no new field-by-field `ObjectDef`). The terminal names
`lumacore_cmd_keys` (bindings, find, command_for, parse_key_name) and registers it, as do the
sweep and tidy. Moved tests: the core keys, a key line's flag, a stored line's object, and Bind
as a Keys Oops step (new, out of the kernel's Oops test); `settled_clashes` (a namespace check)
and the tree-path golden record (every object's tree) went to `tests/catalog`, the record byte
for byte apart from its header line. Views' window-bar test makes its key on the profile.
HEADERLESS 279 → 278; system.rs off LONG_FILES (797 lines). Objects' 471 tests, the keys' 7, the
tree shell's 61, the catalog's 20, the server's 52, tidy's 47 and systemtest 200/200 pass; clippy
is clean.

**Part 34g4 cut 4 built (2026-10-16): `crates/commands/objects/lumacore-cmd-showfile`** (decision
25.7). Objects' 1,442-line `backup.rs` split by job — `ask` (the options, a show's name, "save
first?", the confirm), `save`, `load`, `words` (SaveShow, LoadShow, NewShow, CleanStart,
DeleteShow), `show` (the Show object, List Show, autosave, the status), `copies` (Version,
Recovery) — with `show_import.rs` (`import`), the Library object (`library`, by the exported
`listed_object!`), the showfile rows and the Backup menu (`menu`: its three grids and actions,
pushed by the crate's `register`; the grid builders in `menu_defs` are public for every command
crate). **Restart and Shutdown ride here until `cmd-station` is cut**: their "save first?" is this
crate's, and the station's crate will sit on this one and take them. The server names
`lumacore_cmd_showfile` (startup show, Admin user, autosave tick, status, parts view) and
registers it, as do the terminal (the status line), the sweep and tidy. `List Menu` now lists
Backup after the kernel's menus (the page finds menus by name). Moved tests: the backup tests and
backup.rs's own (13), the show-settings and power halves of the kernel's (5, four new); to
`tests/catalog` the rights table (every line's level, the gate, the help) and views' menu checks
(every menu's JSON, every column's prop). HEADERLESS 278 → 275, OLD_TEST_FILES 46 → 45;
menu_defs.rs and backup.rs off LONG_FILES. Objects' 454 tests, the show file's 18, the catalog's
25, views' 13, the server's 52, tidy's 47 and systemtest 200/200 pass; clippy is clean. Not typed
(handlers still take `&[&str]`): the §33 3d typing comes with the rest of part 3.

**Part 34g4 cut 5 built (2026-10-16): `crates/commands/objects/lumacore-cmd-lumasession`** (decision
25.7). The LumaSession words on the kernel's outbox: the rows of Join, Leave, Invite and Dismiss,
Join's parsing (an address or a name through discovery, `/Merge`, what keeps this show first),
Leave's ask, Claim / Pause LumaSession (out of `station_verbs.rs`) and the LumaSession object (out
of `system.rs`, by the exported `listed_object!`). The line is CLAUDE.md's: the kernel asks for
LumaSession work through its outbox, so the primitives stay in its `session` door — `join_at`,
`join_lumasession_at`, `leave_lumasession`, and Invite / Dismiss Station, which are verbs of the
Station object (with it until `cmd-station`) — and the Station's Connect, `Set Station "Online"
Off` and `lumacore-lumasession`'s stream call them. `lumacore-lumasession` registers the crate (the
server and the sweep reach it through it), as does tidy. Moved tests: what Join / Leave / Invite /
Dismiss say when something is missing, the master handing its role on; two new halves (Info
LumaSession, no session). Objects' 452 tests, the crate's 4, the catalog's 25, the server's 52,
tidy's 47 and systemtest 200/200 pass; clippy is clean.

**Part 34g4 cut 6 built (2026-10-16): `crates/commands/objects/lumacore-cmd-station`** (decision
25.7) and decision 25.16. The station's own words: the objects Server, Shell, Client, Clock,
Journal, Health, UIEngine (out of `system.rs`, by the exported `listed_object!`) and Watch (with
`watch.rs`, its `/Watch` hook and `watchable`); Shutoff, Echo, Debug (out of `run.rs`), HissMe,
Banner, Unwatch, Disconnect / Connect's rows; Restart and Shutdown, moved over from the show file
(they ask through its `cmd_power`, the crate sits on `cmd-showfile`); the splash (state's
`banner.rs`), which the terminal shows through `lumacore_cmd_station::splash()`. **Station and
Interface stay in the kernel**: the station table, Network's Lum shape, the Network / LumaSessions
menus, DeskLock Station and the keyed settings name them — Interface is now in the protocols
family (its crate per 25.7), Station in the dispatch family until a later cut takes it.
Disconnect's handler stays with Logout's (`run::leave`), Connect's in the kernel's session door.
The terminal registers the crate (and through it the show file), as do the sweep and tidy. Tests:
the kernel's machinery tests use the stand-in words (25.16); the clock / Watch / journal / About
/ Shell / power-word tests moved to the crate (22); the cross-crate completion test to
`tests/catalog` (`words`). HEADERLESS 275 → 273. Objects' 437 tests, the station's 22, the show
file's 16, the catalog's 26, the server's 52, tidy's 47 and systemtest 200/200 pass; clippy is
clean.

**Part 34g4 cut 7 built (2026-10-16): `crates/commands/objects/lumacore-cmd-devices`** (decision
25.17, step 1). DeskLock (`On DeskLock Station` too) and NormalTheme with the lock gate
(`Hook::Gate`, `allowed_while_locked`), the Device object and its ScreenConfig setting, and the
ScreenConfig object (out of `screens.rs`). The kernel's `devices.rs` keeps the toggle shape — the
exported `device_object!` (its rights now a parameter), `Switch`, `page`, the scopes — and the
window toggles Sidebars, ViewBar, FullScreen (the windows family; anyone's now, as they were
except for an unused `Station` argument); the device table stays the state's. The server and the
terminal register the crate (the lock gate and the core keys), as do the sweep and tidy. Tests:
the devices' moved with them (10), three that weren't about devices went back to the kernel's
users and window-bar tests. HEADERLESS 273 → 271, OLD_TEST_FILES 45 → 44, screens.rs off
LONG_FILES. Objects' 428 tests, the devices' 10, the terminal's 9, the server's 52, tidy's 47 and
systemtest 200/200 pass; clippy is clean.

**Part 34g4 cut 8 built (2026-10-16): the station's network into `lumacore-cmd-station`** (decision
25.17, step 2). Station and Interface (out of `system.rs`), the table's Store / Delete (was
`station_verbs.rs`), Network (out of `objects/protocols.rs`, with Store / Delete Network), the
station's settings and the wiring (was most of `settings_props.rs`: `settings`, `wiring`), and the
Network / LumaSessions menus with their grids. Two kernel special cases became hooks: the rights
gate's "`Set Station \"Tower\" …` is checked on that station" is `Hook::SetCheckedElsewhere`, and
Import Show's table of single objects reads `Hook::ImportByGuid` rows beside its own (Network and
Station are the station's). The kernel keeps `network_id` (lines use it), the shared Store / Delete
helpers (`set_rows::store_slot`, `delete_typed`), `settings_props`' show / asset / About settings
and their small helpers (`on`, `word`, `station_after`, `STATION_WORDS`). Devices and the
LumaSession words sit on the crate now and register it first. Tests: 11 of the station's words
moved to the crate (`wiring/tests.rs`, with the two choices tests), 14 that need lines, users,
patch, the show file or programmer words too went to `tests/catalog` (`wiring`, `lines`,
`import`). HEADERLESS 271 → 270, OLD_TEST_FILES 44 → 43 (lumasession_tests.rs is gone),
objects/protocols.rs and settings_props.rs off LONG_FILES. Objects' 403 tests, the station's 35,
the catalog's 40, the server's 52, tidy's 47 and systemtest 200/200 pass; clippy is clean.

**Part 34g4 cut 9 built (2026-10-16): `crates/commands/objects/lumacore-cmd-protocols`** (decision
25.17, step 3). Output and Input (out of `objects/protocols.rs`: `lines`, and their properties
`props`), DMX In (out of `show.rs`: `dmx`), the DMX Protocols menu with its three grids, and the
protocols family's DMX row; the crate sits on `cmd-station` (a line names its Network) and pushes
Import Show's Output / Input finders (`Hook::ImportByGuid`). The kernel keeps the Store options
and `network_id` the Network and the lines share (`objects/protocols.rs`, now 50 lines), and
Assign / Unassign (25.17's correction above; the Universe's routes find the lines through the new
`objects::object_of`). The server, the terminal, the sweep and tidy register the crate. Tests: 4
to the crate (with its inline one, 6), 13 that need the station's network, the patch, the users or
the demo to `tests/catalog` (`lines`, and `kernel`: the change path, the demo show, the reference
check and the menu list on real words until stand-in words cover them); the station's journal test
journals a Network now. HEADERLESS 270 → 269. Objects' 385 tests, the lines' 6, the station's 35,
the catalog's 53, the server's 52, tidy's 47 and systemtest 200/200 pass; clippy is clean.

**34f7b hardening (2026-10-16, asked with real-world scenarios; replaces 22.3's "a line naming a
protocol this build lacks is kept, never sends"):**
1. **Switching a line's protocol keeps its settings dormant**: Output 1 made sACN with Priority 150,
   switched to Art-Net (`Set Output 1 "Protocol" artnet`), keeps the 150 in `Line.settings`,
   unused and shown as sACN's in Info; switching back brings it back. `Set Output 1 "Priority"`
   on the Art-Net line is refused, naming the protocol that has it.
2. **A show naming a protocol this build lacks is refused at load** (a new E-SHOW code naming the
   line and the protocol), like a newer format — no silent, never-sending line.
3. **Every station of a LumaSession runs the exact same build** (version and commit, the build id
   `lumacore-server --version` prints): a join from another build is refused before the show is
   taken (a new E-LUMA code naming both builds), as established consoles require one software version. A
   rebuilt station is rebuilt everywhere.
4. **Old shows**: opening them matters, but a converter comes later, for major releases once the
   software is mature (roadmap idea); until then the bump backs up the library's format-25 shows
   (`.autosave/<name>_before-format26_<time>.luma`) and regenerates only the demo.
5. **A setting a line's protocol doesn't declare refuses the load** (naming the line and the
   setting): what the show says is what sends; a new protocol setting therefore always bumps the
   show format.
6. **Protocol names are lowercase and must match**: `sacn`, `artnet`, `osc` — stored in the show,
   typed (`/Protocol artnet`, `Set Output 1 "Protocol" sacn`) and completed exactly so, no
   aliases; one namespace across the rows (checked at freeze).
7. **Values a user types** (asked further, lum.md updated — a syntax decision): a declared value
   (a choice, a protocol) is typed bare, in any case — lowercase always works — and the line
   that runs, the journal, history, stored lines and the screens show its declared form
   (`alwayson` → `AlwaysOn`, `htp` → `HTP`); a copied line typed back runs as shown; any other
   spelling is refused with a hint (E-CMD-008 / E-SYN-014). Quoted is always text and keeps its
   case (`"FOH Left"`): a quoted choice (`"high"`) is refused ("type it without quotes"), a bare
   name (`foh` where text goes) too ("a name is quoted"). Verbs, objects and At's own words
   (Full, Zero) stay as they are (`At Full`, `Off Sequence 1`). Two-word choices keep their
   names (`UseMine`, `AlwaysOn`, `LowTP`). Protocol rows are named in lowercase — `sacn`,
   `artnet`, `osc` — so that is their declared form, with no aliases (`Art-Net` gets a hint).
   This replaces 6's "must match" for every value but the protocol names' spelling.
8. **Every declared word is written out as declared** (asked: "the same thing for commands and
   objects"; today a line runs and is journalled as typed — `store group 3`): verbs, objects,
   options, property names inside their quotes and declared values, in the line that runs, is
   forwarded, journalled, kept in history and stored (`set universe 1 "mergemode" htp
   /noconfirmation` → `Set Universe 1 "MergeMode" HTP /NoConfirmation`); free text and tree-path
   node names keep what was typed. Built with §33 part 3's printer.
9. **Protocols and properties are declared in CamelCase**: the protocol rows are `ArtNet`,
   `Sacn`, `Osc` (later `Midi`, `Kinet`) — replacing 6 and 7's lowercase names; typing
   `artnet` / `sacn` / `osc` works and is written out as declared, there are no other spellings
   (`Art-Net` gets a hint). Property names already are (`MergeMode`, `ThemeOverride`).
Still from 22.3 / 22.4: the `OutputProtocol` enum goes; a line keeps typed what every line of its
kind uses (network, target, enabled, direction; Dmx: rate, send, packet gap, input priority,
sync; Control: the user); sACN's Priority and OSC's Port move into `Line.settings`, declared on
their `ProtocolDef` as `lumacore_registry::setting::SettingDef` (the window-settings pattern) and
read by `Set`, the grid columns and the reference through `Props::dynamic`; `/Protocol` values,
completion and the reference come from the rows; drivers keyed by row name; the plan's OSC map,
`AppState::sacn_line_count` / `sacn_running` and `Runtime.Network.ArtNetNodes` become generic
(per protocol, the `Discovery` capability); show format 26, forward only.

**34f7b built (2026-10-16):** `OutputProtocol` is gone. `Line.protocol` is the row's name
(`Sacn`, `ArtNet`, `Osc`), and `Line.settings` (serde default, walked as data) holds what only
one protocol has. The `ProtocolDef` row declares:
- `untargeted` (`Multicast` / `ArtPoll` / "Any source");
- `universes` (`Universes { first, last, label, parse }`: sACN 1–63999, Art-Net's Port-Address
  `0.0.1` 0–32767 — its helpers moved into `lumacore-artnet`; Control rows have none);
- `settings` (sACN `Priority` 0–200, default 100, section `Output`: a setting in section
  `Output` / `Input` is that direction's only; OSC `Port`).

`Line::{setting, setting_u64, declared}` read a value with the row's default. `wire_label` /
`mode_label` read the row. The routes' range, `Assign … /Universe` (the row parses it; a plain
number outside the range reaches E-OUTPUT-005), Info, `List Universe`, the DMX Protocols grid rows
(the settings by their declared names) and the System Monitor all read the row too.

In `lumacore-cmd-protocols`:
- `/Protocol` takes a vocabulary word from the rows (`lum::vocabulary` adds every row's name as
  a value; the flag has no static choices); quoted is E-SYN-014; an unknown name is E-SYN-001;
  a row that doesn't take the direction is E-CMD-008.
- `/Port` goes only to a protocol that declares it.
- The settings are `Props::dynamic` per direction (`Access::Named`; a line whose protocol has
  none by that name is refused naming it, E-SET-001).

The plan keys drivers by `(row name, NIC)` and its Control listeners by line (`ControlPlan`).
`ProtocolRuntimes` keeps them generic. `AppState::protocols_running` (protocol → active output
lines) replaces `sacn_running` / `sacn_line_count`, and the HTTP info / metrics carry it as
`protocols_running`. `Runtime.Network.Nodes` (`NodeRuntime` with its protocol) replaces
`ArtNetNodes`.

`LineOps::check_protocols` refuses a show at load (`show_file::parse_show`) when a line names a
protocol this build lacks (E-SHOW-007) or carries a setting its row doesn't declare
(E-SHOW-008). `Hello.version` joins `commit`: a join from another version or commit is refused
(E-LUMA-006), and the old "another build only warns" note is gone. `FORMAT_VERSION` 26,
`SESSION_PROTOCOL` 17, the demo regenerated; the library's format-25 shows were backed up first.

Tests whose catalog loads the demo show register the three protocol crates as
dev-dependencies: lumacore-objects, lumacore-suggest, and lumacore-tree (sACN only). The sweep
registers them too. Tests that need the rows moved to `tests/catalog` (`protocols`).

Not built yet (roadmap):
- switching a line's protocol with `Set … "Protocol"` (1: settings dormant, shown as the old
  protocol's);
- the `Art-Net` hint;
- the grid's protocol columns from the rows (the Priority column is still declared in the
  menu);
- writing out every declared word (8, with §33 part 3's printer).

**34f7c hardened (2026-10-17): a protocol is an object, a line is one protocol row.** Grew out of
issue 11 (§11 "Issue 11 hardening"), asked one at a time after probing a reference console (universes are
1024 fixed slots; each protocol holds rows `sACNData n` / `Art-Net-Data n` with `LocalUniverse`,
`Amount`, the protocol universe, `Mode`, `Priority`, `InputPriority`; `MergeMode`'s one home is the
universe; protocol-wide settings such as `ArtPollRate` on the protocol). A hard syntax change
(lum.md): alternatives were routes under `Output` / `Input` (decided first, then replaced) and
the reference shape (one object per protocol, network and rate on every row — declined: per-line settings
repeated).
1. **Each protocol row is a counted object keyword; a numbered one is a line.** `Sacn 1`,
   `ArtNet 1`, `Osc 1` replace `Output n` / `Input n`; numbers are per protocol; `/Protocol` and
   the words `Output` / `Input` go. Inside it stays one `Line` type and one set of handlers —
   "never duplicate per direction" becomes "never duplicate per protocol".
2. **A line is one protocol row**: Network, `"Mode"` Output | Input, `"LocalUniverse"` (the first
   slot) + `"Amount"` (established names; a contiguous run), the protocol's first universe
   (`"SacnUniverse"`; Art-Net `"PortAddress"`, typed `5` or `0.0.5`, one field for the Net /
   Sub-Net / Universe / Absolute), `Enabled`, Rate / Send / PacketGap / Sync, `InputPriority` on
   input. More ranges = more lines. No `Route` / `Range` child objects. `Amount` 0 = the line
   sends nothing.
3. **Assign order is the data flow**: `Assign Universe 5 Thru 8 At Sacn 1 [/SacnUniverse 101]`
   sets LocalUniverse 5, Amount 4, Mode Output; `Assign Sacn 2 At Universe 40 /SacnUniverse 9` is
   Mode Input. The run must be contiguous. A line that already has universes asks Replace / New
   line (next free, same network and settings) / Cancel; `Assign … At Sacn 3` on a missing line
   creates it (your only network, or it asks); `Unassign Sacn 1` sets Amount 0.
4. **Overlaps are allowed**: one slot out on several lines (multicast + unicast, two
   networks, sACN + Art-Net); several inputs into one slot merge by the universe's `MergeMode`
   and each line's `InputPriority`; output and input of one slot too (own packets ignored by
   CID / IP). The grid marks an overlap as a hint.
5. **The tree**: `Root.ShowData.Protocols` holds `Networks`, `DMXProtocols` (`Sacn`, `ArtNet`)
   and `DataProtocols` (`Osc`); `ProtocolDef.kind` Dmx | Control becomes Dmx | Data and picks
   the group. `DMXProtocols` / `DataProtocols` are tree nodes only (cd / ls), never words;
   `Sacn 1` needs no group in front.
6. **The bare protocol word is the protocol**: `Set ArtNet "ArtPollRate" 3` (the bare-owner
   implicit row), `EditSetting ArtNet`, `Info ArtNet` (settings, capabilities, drivers, nodes)
   name the protocol; `List ArtNet` lists its lines; typed alone it sets the default object
   (§33 21). A line property on the bare word is refused naming `Set Sacn 1 Thru "…"`. Store /
   Delete need a number. Protocol-wide settings live on the protocol's node, per show; none
   exist today (ArtPoll keeps its 3 s until one is declared).
7. **Declarations**: `lumacore-protocols` declares the settings every line of a kind has, once
   (Dmx: Name, Network, Mode, LocalUniverse, Amount, Enabled, Rate, Send, PacketGap, Sync,
   InputPriority; Data: Name, Network, Enabled, User); each protocol crate declares its own line
   and protocol-wide settings (sACN: SacnUniverse, Priority; Art-Net: PortAddress; OSC: Port),
   and the commands set them through the one property registry. A protocol without a capability
   leaves its setting out (no Sync → no `"Sync"`). Store's options are the line's declared
   settings. A missing property is the ordinary "no such property", not 34f7b's E-SET-001
   refusal.
8. **No protocol change**: Copy / Move between protocols are refused; Store a new line and Delete
   the old. The 34f7b leftover `Set … "Protocol"` (dormant settings) goes.
9. **Crates mirror the tree**: `network/protocols/dmx/` (lumacore-sacn, lumacore-artnet) and
   `network/protocols/data/` (lumacore-osc, from `control/`) stay the runtime; a command crate per
   protocol under `commands/objects/protocols/` (`dmx/lumacore-cmd-sacn`, `dmx/lumacore-cmd-artnet`,
   `data/lumacore-cmd-osc`) declares its object; `lumacore-cmd-protocols` keeps the shared parts
   (the Network object, the one set of line verbs every protocol object plugs into, DMX In, the
   DMX Protocols menu). Tidy's groups and edges follow.
10. **Order**: built next as 34f7c — lum.md first (its `Output` / `Input` lines rewritten when the
    build starts, so its examples stay tests), then tree + crates, objects + verbs, every line
    rewritten (docs, demo, tests, systemtest), format 27. Then 34g4's patch step, which grows
    into a patching overhaul (crate cut, issue 11's draft rules, a patch window, MVR import —
    hardened first).

**34f7c built (2026-10-17):** `Output` / `Input` and `/Protocol` are gone; each protocol row of the
build is an object word, its object in its own crate under `commands/objects/protocols/`
(`dmx/lumacore-cmd-sacn`, `dmx/lumacore-cmd-artnet`, `data/lumacore-cmd-osc`, each registering its
protocol's row first), declared with `lumacore_cmd_protocols::line_object!` (Store, the door's verbs,
typed List / Info, the properties, the tree) and, for DMX, `dmx_line_verbs!` (`Assign <P> <n> At
Universe …`, `Unassign <P> <n>`). OSC moved to `network/protocols/data/`; `Kind::Control` is
`Kind::Data` and picks the tree group (`Kind::group`).
- **Data:** `ProtocolsData` holds `networks`, `dmx_protocols` and `data_protocols` (row name →
  `ProtocolNode { settings, lines, id_pool }`); `Line` has `mode`, `local_universe`, `amount` and
  `universe` (the first protocol universe), its `protocol` is the node's name (not saved, set by
  `rebuild_id_pools`); `Universe.routes` and `Route` are gone. `LineOps`: `create_line`,
  `set_run` (consecutive, existing universes; E-OUTPUT-001 / 005 / 007 new, 003), `take_off` (from
  either end; the middle is E-OUTPUT-007), `routes_of` (the run's pairs whose universe exists),
  `lines_on`, `wire_clash`, `check_range`; `check_protocols` also refuses a protocol in the wrong
  group (E-SHOW-007) and a protocol-wide setting its row doesn't declare (E-SHOW-008).
  `ProtocolDef` gained `protocol_settings` (none declared yet) and `Universes.property`
  (`SacnUniverse`, `PortAddress`).
- **Tree:** `Kind::Named(shape)` and `Step::Name`: `Protocols.DMXProtocols` / `DataProtocols` list
  the protocols by the objects' `TreeDef.name` (a new field, `None` elsewhere); `objects::tree_at`
  / `named_in`; `ls` shows a named entry as a folder. No protocol is named in the state crate.
- **Commands:** `Assign Universe <run> At <P> <n> [/Universe <w>]` (Output) and `Assign <P> <n> At
  Universe <run>` (Input) share `universe::assign_run`: a missing line is made on the show's only
  network (several networks ask which), another run asks Replace / New line / Cancel.
  `Unassign Universe <u> [At <P> <n>]` takes universes off. Data lines take no run (E-OUTPUT-006),
  Osc lines stay inputs. `Info <P>` without a number is the protocol (`info_protocol`).
- **Plan / runtime / views:** the plan walks `all_lines`, Data listeners are keyed by (protocol,
  line), line states match by (protocol, line); the DMX Protocols rows carry `object` = the
  protocol and `mode`; the Lines grid finds its object at run time (`Grid.object_of`, a new field)
  — the first DMX protocol's — and shows what every DMX line shares.
- **Format 27, SESSION 18**, the demo `Store Sacn 1` + `Assign Universe 1 Thru 2 At Sacn 1 /Universe 1`;
  the library's format-26 shows backed up first. The shim ratchet went to 341 (List / Info typed;
  the counter now counts a `line_object!`'s handlers once per object), the ObjectDef copies to 42.

Built differently from the hardening, on purpose:
- Assign's protocol-universe option stays `/Universe` (lum.md decision 22, one option on the
  Universe word for every protocol); Store and Set use the protocol's own `/SacnUniverse`,
  `"PortAddress"` (point 3 named `/SacnUniverse` on Assign too).
- What every line of a kind has is declared once in `lumacore-cmd-protocols::props`, not in
  `lumacore-protocols` (point 7): a `Property` is an objects type, and `lumacore-protocols` sits below
  the objects crate. A protocol's own settings stay on its row in its own crate.
- `Set <line> "MergeMode"` (the bulk setter over a line's universes) is gone: MergeMode's one home
  is the universe (`Set Universe 1 Thru 6 "MergeMode" HTP`).
- The DMX Protocols Lines grid has the shared columns only (no Priority) and no bound cells (rows
  mix protocols); its New asks the protocol word (`Store {value}`).

Not built yet (roadmap): `Set ArtNet "<Setting>"` / `EditSetting ArtNet` on the protocol itself
(point 6 — waits for the first protocol-wide setting; a bare line property is refused naming
`Set Sacn 1 "…"`); a protocol's own columns in the DMX Protocols grid; the `Mode` words are
spelled `output` / `input` in the vocabulary (the DMX window's View values declare them first).

**34g4 patch hardened (2026-10-17): Devices › DMX › Patch, faithful to MVR and GUIDs, with MVR
import and export.** Asked one question at a time, with mockups for the choices, after
researching four sources:
- a reference console's manual and live console tests;
- the user's own patcher plugin for that console;
- the user's screenshots in `inspiration/patcher/`;
- the MVR 1.6 spec (DIN SPEC 15801), its `next` branch, and the open issues and proposals on
  `github.com/mvrdevelopment/spec`.
Not built. Tags (§12) are hardened and built first, then this.

*Research, the facts the decisions stand on:*
- **The reference has two patch trees.** A live patch is what runs; a show patch is the
  staging copy that the Patch menu edits. Leaving the menu asks Save and Exit / Discard and Exit /
  Stay.
- **Its Live Patch applies at once but can't add or delete.** Tested by writing each property of
  one fixture with `Set Fixture n Property "<P>" <v>` and putting it back:
  - accepted: Name, Note, Patch (the address), pan/tilt offsets, DMX / 3D / encoder inverts,
    position and rotation, beam angle, shadow, Follow Target, Selectable / Visible 3D, Master
    React, P1–P4, grid invert;
  - refused: FID, IDType, CID, Layer, Class, Scale, MIB, grid rotation, Tags;
  - fixture type and mode are Patch only.
  Opening either menu needs Setup in the UI; the manual says Program reaches the Live Patch but
  not the full Patch.
- **What its patch holds:** fixture types, attribute definitions, DMX curves, layers, classes,
  12 IDTypes and 1024 fixed universes, plus Stages → Spaces + Fixtures. Fixtures nest: an empty
  Grouping fixture holds children. A fixture has one FID plus one IDType + CID, `PATCH "u.aaa"` and
  Break1–8. Adding fixtures is wizard-only; the commands are thin: `Patch Fixture 2 3.123`, and
  `/PatchOffset` for patching a run with a spacing.
- **It imports MVR through a partial show read.** The MVR's patch is shown beside the live one and
  rows are matched by FID, then CID, then GUID, then Name. You take, keep or drop per row; the
  Patch and Position/Rotation columns can be taken separately. It can't be undone and works only
  standalone or as idle master. Export is `Export Stage /MVR`; MVR-xchange is
  `SendMVR Join|Leave|Commit|Request`.
- **The patcher plugin imports by diff.** Its home screen picks the newest file, with saved settings: match
  key, FID and universe offsets, rig offset / rotation, ignore rotation. It matches by GUID, then
  FID, then Name, with claims so a show fixture is matched once and a NEW row absorbs a LOST twin.
  Rows are OK / DIFF / NEW / LOST, plus red for a clash. Each cell can be ticked, universe and
  address separately. A NEW row's fixture type comes from the show, the MVR or a dummy. It backs
  up, then applies.
- **What the patcher plugin learned:**
  - the MVR UUID equals the console's fixture GUID byte for byte;
  - GDTF modes have no UUID: they are a name inside the type (proposal #309);
  - MVR has no per-fixture `<Layering>` link to match `<Classing>`;
  - the console exports every fixture under one layer, "groupings";
  - converting the MVR matrix to Euler angles loses information on some hangs.
- **MVR 1.6 `<Fixture>` fields:** uuid, name, multipatch, Matrix (4×3 in mm, relative to its
  parent), Classing, GDTFSpec / GDTFMode (by name), Focus, CastShadow, DMXInvertPan / Tilt,
  Position, Function, FixtureID / FixtureIDNumeric, UnitNumber, ChildPosition, Addresses (one per
  break, plus Network: interface, IP, DHCP), Protocols, Alignments, CustomCommands, Overwrites,
  Connections, Color (CIE), CustomIdType / CustomId, Mappings, Gobo, ChildList.
  - CustomIdType values: 1 Fixture, 2 Channel, 4 Houselights, 5 NonDim, 6 Media, 7 Fog,
    8 Effect, 9 Pyro, 10 Marker.
  - Layers are flat (elevation only). GroupObjects nest and are "only there to create a hierarchy
    of objects" (maintainer, #295).
  - AUXData holds Class, Position, MappingDefinition and Symdef.
- **What MVR is likely to add:**
  - already on `next`: `Comment` on every object;
  - as PRs: Color on Layer (#322), Cables linking Connections (#313), non-lighting devices (#320);
  - as proposals: Circuits on Fixture / VideoScreen / Projector (#326), a DMXModeID (#309), a
    Settings object (#141 / #143), tracking as child fixtures (#199), a thumbnail (#285).

*Decided:*
1. **The patch is faithful to MVR and to GUIDs end to end.** It holds only fixtures. MVR's 3D
   objects (scene objects, trusses, supports, video screens, projectors, focus points) belong to
   `Root.ShowData.3DSpatial`, hardened later. A fixture's links to them stay as GUIDs and show as
   "outside the patch"; they are never dropped.
2. **Paths:** `Root.ShowData.Devices.Dmx.Patch.Fixtures.<n>`, with each fixture split into two
   parts:
   - `Fixture.Mvr` has every MVR 1.6 `<Fixture>` field, typed, with links as GUIDs;
   - `Fixture.Luma` has what MVR lacks: MasterReact, grid spot, pan/tilt offsets, encoder inverts,
     Luma settings.
   The code follows `Devices.Dmx`; later device kinds that use other base protocols get their own
   `Devices.<Kind>`. A fixture's GUID is the MVR uuid.
3. **One home per field.** A field MVR defines lives in `Fixture.Mvr`. Properties stay flat to the
   operator (`Set Fixture 101 "Name" "x"` writes `Mvr.Name`); each declared property's home says
   which part holds it, and the parts are tree nodes, not words (lum.md unchanged: no
   `"Mvr.Name"`).
4. **Position and rotation are one stored world matrix** (4×3, mm), exact, with no Euler drift.
   `"PosX"`…`"RotZ"` (metres, degrees) are declared properties that read from it and write into
   it. Export writes local = parent⁻¹ × world.
5. **The selectable numbers are MVR's.** `Fixture 101` is `Mvr.FixtureIDNumeric`; the second ID is
   `CustomIdType` + `CustomId` (`Houselights 4`), from MVR's fixed list of ID types. Our FID
   namespaces go (forward only). FixtureID text and UnitNumber stay as fields you can read and
   filter on.
6. **Fixture type, revision and mode are GUID links in `Fixture.Mvr`.** Export derives GDTFSpec /
   GDTFMode from them and embeds the revision's `.gdtf`. A mode is found by name inside its
   revision (until #309).
   - A fixture always links a type that is in the show. Types reach the show from an MVR's
     embedded GDTFs, the station library, or later an online API.
   - An embedded GDTF with a known FixtureTypeID and other content becomes a new revision (kept by
     SHA-256). Matched fixtures keep their revision, with a Type cell offering the new one; NEW rows
     default to the file's revision.
7. **`Root.ShowData.Devices.Metadata.MVR`** holds Layers, Classes, Positions, MappingDefinitions,
   Groups (the GroupObject tree) and Files, all by UUID. The Patch and 3DSpatial both use it.
   - Classes, Positions, MappingDefinitions and Files arrive and update only through imports,
     matched by UUID (a rename in CAD renames here). You can label them, filter by them and use
     them as an import scope. "Clean up unused" removes entries nothing links to.
   - Layers and Groups are the patch tree (point 12) and can be edited.
   - Files is one record per import: name, SHA-256, provider and version, MVR version, when, who,
     scope, preset, counts and UserData. The `.mvr` itself goes to
     `library/mvr/.store/<sha256>.mvr` and travels between stations by hash.
8. **The claim moves up to `Root.ShowData.Devices`.** It covers every device kind; claiming takes
   the draft (this replaces "Claim Patch"; issue 11's rules still apply, now to the Devices draft).
   Live device edits stay open, like an established Live Patch:
   - Program level, on the columns whose property is declared live (the set above);
   - a live edit goes to the live show and into the claim holder's draft, unless the holder already
     changed that cell there, in which case it is refused, naming them;
   - Keep is one Oops step.
9. **The Patch is a kit setup dialog** (`<luma-setup>`, like DMX Protocols in 12d).
   - The title row reads **Devices › DMX › Patch**. Under it, a rail lists the kinds under Devices:
     DMX, Metadata, and a slot for kinds that come later.
   - The strip has Patch, Fixture types and Universes, and the selection bar.
   - The left side is the sidebar (point 12). The grid is one `<luma-grid>` in three modes:
     - Live: non-live columns are greyed and locked;
     - Draft: every column can be edited;
     - Import review: a Status column, cell marks and ticks.
   - The slab's function block changes with the mode. The bottom-right button is **Exit**, which
     asks "Do you want to save or discard the draft?" (Save / Discard / Cancel).
10. **Importing an MVR:**
    - **The dialog.** Import… → Import MVR has a **File | Xchange** switch. It sets:
      - the scope (point 11) and the match key (Auto = GUID → FID → Name);
      - FID and universe offsets, and the rig offset / rotation;
      - a **None / Take / Offer** switch per field group (Addresses, IDs, Names, Position and
        rotation, Type revisions, New fixtures), defaulting to Offer;
      - LOST: Mark / Delete, defaulting to Mark; and Ignore rotation;
      - all saved as the user's import preset.
    - **Review.** Nothing is written while you review. Taken cells are orange with a tick and the
      live value under them; offered cells are dashed with the file's value under them; new
      fixtures are green and LOST rows red. A clash is never taken and blocks Keep; a blue GUID
      marks the match key. Dots mark the holder's own edits and other users' live edits.
    - **Import ticked.** It asks where new fixtures go, one pop-up per MVR group, with the
      suggestion first and a "Use for all other imported layers" checkbox. Then it writes the
      ticked changes into the draft and returns to it. Cancel import leaves the draft as it was.
11. **Scope** is an MVR layer or group, an MVR class, a tag, or the whole patch. It filters both
    sides, but a GUID is matched across the whole patch, so a fixture is never duplicated.
    - A fixture matched outside the scope shows as **OUTSIDE**, with "Bring into …". Only fixtures
      inside the scope can be LOST.
    - A scope that names a group the file can't know (a tag, or a group made in Luma) takes the
      file fixtures that match its fixtures, plus file fixtures whose MVR group it came from.
12. **The sidebar is the MVR tree, and private sorting is tags.** The patch tree is MVR's own
    Layers › nested Groups (`Metadata.MVR`, editable); a fixture's place is one GUID link,
    `Fixture.Mvr.Parent`. A switch groups the sidebar by **MVR tree, MVR class or tag**.
    - Re-import: new fixtures land in their group by UUID. A fixture the drawing moved is a DIFF
      on its Parent cell. A question is left only when the group a new fixture belongs in was
      deleted in Luma.
    - Each selected node has a **≡ menu**: New sublayer, Rename, Copy, Duplicate…, Move…, Select
      fixtures, Delete….
    - **Duplicate** asks for clean fixtures or cloned data (everything, or chosen kinds: presets,
      groups, sequences, effects, layouts, masters) and a first FID (suggesting the next free
      block, spacing kept). The copies are unpatched, with new GUIDs and no MVR source.
    - **Delete** deletes the node's fixtures too, after one question naming what uses them.
13. **Export MVR** writes what the dialog shows (the holder's draft while held, otherwise the live
    patch): the whole patch, a node or the selected fixtures. GDTFs are embedded, and "Commit to
    Xchange" with a comment is an option.
    - Its Layout switch, By MVR layer | By Luma layer, was decided while Luma layers were planned
      (see the note below). With point 12 the tree is exported as it is.
    - Anything made only for the export gets a UUID once, kept in Metadata so the next export
      reuses it.
14. **MVR-xchange is a station-local service per network interface**
    (`Set Interface "en1" "MvrXchange" Yes`, plus `MvrXchangeGroup`, default "Default"). The Xchange
    side of the import dialog lists the files committed in the group (name, station, comment,
    time). Picking one fetches it into `library/mvr/.xchange/`, then it imports like a file.
15. **Order:** Tags (§12) → this patch overhaul → the programmer → the device sheet (fixture sheet)
    → playback, including the tags' playback behaviours.

Superseded during this hardening (recorded, not decided):
- a separate Luma layer tree (organising only, one layer per fixture, nested, with a source UUID
  per layer);
- "New fixtures go into: Automatic", which built Luma layers from MVR layers and groups;
- the export's By Luma layer layout.
Point 12 replaced all three once tags became the private sorting method. The import was also
briefly decided to write straight into the draft; review → Import ticked replaced that. Mockups,
not locked: `docs/dev/ui-kit/mockups/2026-10-17-34g4/`. The patch dialog page predates point 12
and still draws Luma layers.

**34g4 patch, the parts (cut 2026-10-17, at the go).** Six parts, each committed on its own:
1. **The data** (format 30): `Devices.Dmx.Patch` (FixtureTypes, Fixtures, Universes) and
   `Devices.Metadata.MVR` (Layers, Groups, Classes, Positions, MappingDefinitions, Files); a fixture
   is its meta + `Mvr` + `Luma`; one world matrix with `"PosX"`…`"RotZ"`; `CustomIdType` +
   `CustomId` replace the FID namespaces; the LivePatch placeholder goes.
2. **The Devices claim and draft**: issue 11's rules on the Devices draft, live device edits on
   the declared live columns, Keep = one Oops step.
3. **The Patch setup dialog**: mockups locked first (point 9, 12).
4. **Import MVR**: `crates/formats/lumacore-mvr` (the file), the `.mvr` store, the dialog and
   the review (mockups first; point 10, 11).
5. **Export MVR** (point 13).
6. **MVR-xchange** (point 14).

Naming (2026-10-17): a bare `mvr` names only MVR as a format and MVR-xchange
(`crates/formats/lumacore-mvr`, like `lumacore-gdtf`, depending on `lumacore-fixtures`). Show
data that maps MVR is named for what it maps onto: `lumacore-fixtures` `fixture_mvr.rs` (a
fixture's MVR part) and `devices_meta.rs` (the `Devices.Metadata.MVR` pools).

**34g4 part 1 built (2026-10-18): the data, format 30.** `ShowData.devices` is `dmx.patch`
(FixtureTypes, Fixtures, Universes) + `metadata.mvr` (`lumacore_fixtures::MvrMetadata`: Layers,
Groups, Classes, Positions, MappingDefinitions, Files, each entry a NodeMeta). `Fixture` =
`meta` + `mvr` (`MvrFixture`: parent — a layer, group or fixture —, the world `Matrix`, type /
revision / mode, breaks, multipatch, class, position, the ids, the MVR records) + `luma`
(MasterReact). `"PosX"`…`"RotZ"` are keyed properties on the matrix (R = Rz·Ry·Rx, reading never
changes it, writing keeps scale and offset); `"Function"`, `"FixtureID"`, `"UnitNumber"`,
`"IDType"`, `"CID"` are fields. One link may name several kinds (`MvrLayer|MvrGroup|Fixture`).
Focus points and connections point into 3DSpatial and are data until it is hardened. The tree
has `Devices.Dmx.Patch`, `Devices.Metadata.MVR.<pool>` (static nodes with counts; their words
come with part 3); `List patch` reads the path aliases. The show file keeps `Devices/Devices.json`
and `Devices/Dmx/Patch/FixtureTypes/<n>/Revisions/<i>/source.gdtf` (a revision's folder is its
place in the list again: since 1b dropped `meta.id` every revision had gone to folder 0).
Not yet: selecting by the second id (`Houselights 4`) and the behaviour of DMXInvertPan / Tilt.

**34g4 part 2a built (2026-10-18): the Devices claim and draft, live device edits.** The draft
is the whole `ShowData.devices` (`PatchDraft.live: DevicesData`); `Patch` left the claim areas.
An area is named by its tree path (`Claim Root.ShowData.Devices`, decided 2026-10-18 one question
at a time: the areas never were Lum words — `Claim Patch` parsed only because `Patch` is a verb —
and `Devices` next to the `Device` object would invite typos); `enter patch` and the bare
`Patch` verb still claim it. Live columns are declared per object (`Props::live`; Fixture: Name,
Note, Address, PosX…RotZ, MasterReact; Address down to Program). While a draft is open, another
user's `Set` on a live column and every Oops / Redo run with the live devices swapped in for that
statement (`commands::live_draft::runs_live`; outer statements only), so the normal change path
undoes, replicates and journals them; the draft then follows cell by cell
(`patch_draft::rebase`: a leaf the holder left as it was takes the live value, objects merge key
by key, a leaf both changed refuses the line, E-PATCH-008 naming the holder). Other edits by
others stay refused.

**34g4 part 2b built (2026-10-18): issue 11 points 4–5.** `Move Universe … At …` asks when
input lines stay on a slot it leaves (Move anyway / Cancel; MergeMode goes with the universe);
Universe's one Delete question names the fixtures patched on it (in the draft, and the live
ones while a draft is open) and the lines left on its slot; on yes the universe goes and those
fixtures **lose their address there and stay in the patch** (the established unpatch: groups, presets and
cues keep them; `Delete Fixture` is what removes one); Keep's question names the lines left on
slots without a universe (`objects/patch/universe_ask.rs`).

**34g4 part 3 cut (2026-10-18, round 2 of the dialog locked: `mockups/2026-10-18-34g4/`).** 3a the
`patch` feed and the Patch menu row · 3b the kit: `<luma-setup>` with a tree sidebar (MVR tree /
Class / Tag), the kind rail, the crumb title, Exit, the live columns locked · 3c the MVR tree's
words (a Lum vocabulary question first), the ≡ menu, Duplicate · 3d the Metadata tab.

**34g4 part 3a built (2026-10-18): the `patch` feed and the Patch menu.** The state thread
publishes the open draft with its holder beside the live show (`Published.draft`); the `patch`
feed (per user, rebuilt when the show or the claims move) builds `lumacore_views::patch::view`:
the holder of the Devices draft gets `mode: draft` and the draft's rows, everyone else `mode:
live` and the live devices — fixtures (no, guid, name, note, fixture type, mode, address, pos /
rot as their properties show them, master react, tags, MVR group), the MVR tree (layers and
groups, each counting the fixtures in it and under it), classes, universes, fixture types.
`Menu "Patch"` (`objects/patch/menu.rs`, icon plug) has the Patch, Fixture types and Universes
grids on it, their columns the declared properties. `lumacore-tree` re-exports
`lumacore_fixtures as fixtures` (as it does the tags) so the views need no new crate edge.

**34g4 part 3b built (2026-10-18): the Patch dialog in the kit.** A menu row may declare the claim
area it edits (`MenuDef.area: Area` — the crumb `trail`, the rail of `kinds`, the `feed` with
`mode` / `holder`, the sidebar's `tree` modes, the `claim` and `exit` lines); `<luma-setup>` then
draws Devices › DMX › Patch: the claim chip (Live: "nobody / Bob holds the draft", a tap runs
`Claim Root.ShowData.Devices`; the draft: "Devices draft · Bob"), the rail (DMX on, later kinds
dashed), the sidebar as a `<luma-tree>` (no numbers, no ≡ yet) grouped by MVR tree / Class / Tag
(Direct | Deep), filtering the grid whose rows carry the mode's field, the tabs as the strip, and
Exit in the power slot (`<luma-power>` with an icon; gold = the draft: `Unclaim
Root.ShowData.Devices`, which asks Keep / Discard when something changed; Live: closes). Each
column says whether its property is `live`; in Live the others are locked with the reason (the
grid's column `lock`). Starting or ending a draft republishes (`show_generation`), so the holder's
feed flips at once. Checked in Chrome against the locked round 2 (live, draft, tag grouping, a
sample MVR tree in the Showroom, phone width) and by `tests/e2e/tests/kit-34g4.spec.ts`. Not yet:
the ≡ menu and its words (3c), Metadata (3d).

**34g4 part 3c hardened (2026-10-18): the MVR tree's words and one nesting rule.** Asked one
question at a time (Lum vocabulary: a hard discussion first, lum.md binding):
1. **Words:** `MvrLayer`, `MvrGroup` (numbered pools of `Devices.Metadata.MVR`; a group's place
   is its `"Parent"` — a layer or a group), `MvrClass`, `MvrPosition` (read only: they come by
   import; select their fixtures). A bare `MvrGroup 3` / `MvrLayer 1` / `MvrClass 2` selects the
   fixtures in it (and under it). The `Mvr` prefix names MVR's own concept and can't be confused
   with `Group` (the selection group). Alternatives declined: tree paths only (long, selecting by
   group awkward), `Layer` / `GroupObject` (`Layer` is a common lighting word).
2. **Putting fixtures in is `Assign`** (lum.md: putting one object into another is Assign):
   `Assign Fixture 101 Thru 104 At MvrGroup 4` — a fixture has one place (MVR's one parent), so it
   leaves its old group; `Unassign Fixture 101 At MvrGroup 4` puts it in No MVR group.
3. **One nesting rule:** `Move <node> At <node of a nesting kind>` makes it a child of the
   destination; `Move <node> At <number>` stays the door's renumber (swap). It **always asks**
   first — "MvrGroup 3 'Truss 2' becomes a child of MvrGroup 2 'Truss 1' (2 groups, 88
   fixtures)?" Nest / Cancel; `/NoConfirmation` skips it (stored lines, the dialog's drag). Tags
   follow it too (today they nest silently). A node never goes under itself or its own children.
4. **What nests:** only tree kinds — Tag and MvrLayer › MvrGroup › MvrGroup. Pools stay flat and
   numbered; private sorting of anything is tags. Another nesting kind is a hardening of its own
   ("only tree kinds for now, others might come later"). FeatureGroups don't nest: `Assign
   FeatureGroup "Strobe" At FeatureGroup "Shutter"` merges one group's attributes into another
   (built in 12f-a, kept — it puts members into a container like every Assign).
5. **Tags and the MVR tree** share only what every object has: an MvrLayer / MvrGroup carries tags
   (`Assign Tag "FOH" At MvrGroup 3`), and the sidebar's Tag grouping sorts by them. No sync or
   link between the two trees.
lum.md takes the words, the Assign examples and the nesting rule with the code that adds them
(its examples are tests; the pending list only shrinks).

Read with §12 "Tags part 1b hardened" (decided after this hardening): MVR's `uuid` is the
fixture's GUID and its `Name` the fixture's name, both in the meta (Identity), so point 3's
`Mvr.Name` is the meta's name; the next MVR's `Comment` is the meta's note. `FixtureIDNumeric`
is the pool's key (a number is only its pool's key).

**34g4 part 3c built (2026-10-18): the MVR tree's words, the nesting question, the ≡ menu.** In
the kernel's patch objects (`objects/patch/devices_meta.rs` + a file per verb): `MvrLayer`,
`MvrGroup` (on the object door: Copy / Cut / Paste / Insert / Compact; Store, Delete, Move,
Label by the meta), `MvrClass`, `MvrPosition` (read only), their tree collections
`Root.ShowData.Devices.Metadata.MVR.Layers|Groups|Classes|Positions` (now `Collection`), a bare
one selecting the fixtures in it and under it, List / Info with place and counts. Every write
needs the Devices claim (the draft). `Store MvrGroup` lands in the first layer (none: refused,
store a layer first); `Assign Fixture … At MvrGroup | MvrLayer n` sets the one place,
`Unassign Fixture … [At …]` clears it; `Delete MvrLayer | MvrGroup` asks once naming the groups
and fixtures that go with it and what uses those fixtures, then unpatches them. The one nesting
question is the door's `nest_ask` ("… becomes a child of … (n groups, n fixtures)?" Nest /
Cancel, `/NoConfirmation`): `Move MvrGroup … At MvrLayer | MvrGroup n` and `Move Tag … At Tag`
ask (what can't be done — under itself, a name taken there — is refused before the question);
`At <number>` renumbers, `Move Tag … At Root` un-nests without asking. A layer nests nowhere
and nothing goes under itself or its own groups (E-PATCH-011). The `patch` feed gives each
layer / group its ≡ menu lines (`lumacore_views::patch` `node_lines`), and `<luma-setup>`'s
sidebar opens them on the picked node of the MVR tree mode: Select fixtures, Rename, New group
here (Store + Move /NoConfirmation), Copy (beside it), Move… (a pick of every layer / group but
itself and its own; the station asks), Delete… — edits only in the draft. Duplicate shows off:
its verb is a Lum vocabulary question of its own (3c2). lum.md has the words, the Assign
examples and the nesting rule. Checked in Chrome and by `tests/e2e/tests/kit-34g4.spec.ts` (four
projects).

**34g4 patch entry hardened (2026-10-18): one Patch tile, a claim button.** Asked one question
at a time after 3c ("when i build i cannot enter the patch from the menu"): the Menu's Patch and
Live Patch tiles were still the old disabled placeholders.
1. **One tile.** The Menu has one **Patch** tile (`Menu "Patch"`); the Live Patch tile goes. The
   dialog always opens **Live**; opening claims nothing.
2. **The bottom-right is a claim button, not a power button** (named at the mockup: "Live",
   "Claimed", not lock / unlock). **Live** · "Hold to claim and edit" — a **hold** claims the
   Devices (`Claim Root.ShowData.Devices`), locking the patch for everyone else; a plain press
   only gives that hint. **Claimed** · "Draft · Bob" (gold, yours): a **press** leaves the draft —
   Save / Discard / Cancel when it changed (`Unclaim Root.ShowData.Devices` with `/Keep` /
   `/Discard`; the question "Leave the draft"), back to Live at once when nothing changed. The top
   chip only shows the state; Exit in the power slot goes.
3. **The dialog closes with a normal ✕.** Live: it closes. Claimed (yours): it asks Save / Discard
   / Cancel (nothing changed: back to Live and closes) — never "keep claimed": a draft never
   outlives its dialog.
4. **Claimed by someone else:** the button is greyed, "Claimed · Bob" · "Bob's draft"; a hold
   does nothing (only Bob ends it; an Admin's `/Force` stays on the command line). Declined:
   asking Bob like other claims.
5. **The holder lost from the session** (logout, disconnect, station gone): the claim is
   released and the draft **discarded** — no copy kept. Declined: a recovery copy, saving it into the show.
Mockup locked 2026-10-18 ("approved"): `docs/dev/ui-kit/mockups/2026-10-18-34g4/patch-lock.html`;
it replaces the round-2 page's Exit and claim-chip button.

**34g4 patch entry built (2026-10-18).** The Menu's one Patch tile runs `Menu "Patch"` (Live
Patch gone). `<luma-power>` takes `hold` (a hold fires `hold`, a press shows the hint) and `note`
(its small line as given); the area's power slot is the claim button — Live · pencil · "Hold to
claim and edit" runs `Area.claim`; Claimed · lock · "Draft · <holder>" (gold, `draft`) a press
runs `Area.exit`; "Claimed · Bob" (`theirs`, greyed, dashed, `off`) does nothing. The chip only
shows the state. The ✕ while claimed runs the new `Area.close`, `Unclaim
Root.ShowData.Devices; Menu "Patch"`: the station's one leave question ("Leave the draft", Save /
Discard / Cancel — the press and the ✕ share it, so the mockup's "Close Patch" title is the same
question) and every answer runs the rest, closing the dialog; nothing changed: released and
closed at once. Found on the way: the terminal's line loop split `;` itself and dropped the rest
of a line when a statement asked — it now attaches it (`Ask::then_run`), as dispatch does. A
lost holder (decision 5): the state thread's `patch holder` task (1 s; masters and stations
alone — a member holds a replica) releases the Devices claim and discards the draft once the
holder has been absent from presence (this station's shells, the LumaSession's users) for 5 s, so
a page reload keeps the draft; the System Monitor says so (`Claims::release_lost_devices`). The
open-lock icon isn't in the vendored sprite: Live wears the pencil until it is added (`npm pack
lucide-static@1.55.0`, `lucide/icons.txt`). Checked in Chrome and by
`tests/e2e/tests/kit-34g4.spec.ts` (four projects).

**34g4 patch, round 3 hardening (2026-10-18, in progress; mockups next).** Asked after an audit
of the built patch window (two agents: decisions vs code, the dialog vs the locked mockups) and the
user's own use: "i dont have new line so i can patch a fresh new fixture", "i cannot trigger the
import/export for mvr", "on the universe tab i need to be able to move a universe", "changing fixture
type and mode of a patched fixture is not well built", "when i hold claim button there is no
animation", "claim button needs to be anchored to the bottom right". Inspiration: a reference console's Insert New
Fixtures (picker + form), Fixture Types and Edit Patch screens (`inspiration/patchwindow/`), an older
Defined Attributes clone dialog (`inspiration/Clone.jpg`), a GDTF downloader plugin (revisions
per type, a mode inspector, compare modes), the reference manual (Clone keyword and menu; exchanging a
fixture type converts data by attribute name, no dialog). Found on the way: the old `Patch 6x
"Type" At 1.001` and `Import "x.gdtf"` still add fixtures and types (pre-Lum syntax, no UI); the
hold ring is drawn under every dialog (z-index 100001 < the setup scrim's 100004). Decided so far:

1. **Words** (lum.md first): `Store Fixture 201 Thru 206 /Type 3 /Mode "Standard" [/Revision n]
   [/Address 1.001]` creates fixtures (the one object door); `Patch Fixture <sel> At 2.001 [/Offset
   n]` addresses existing ones (as established); `Unpatch Fixture <sel>` keeps the fixture and removes
   its address. The old `Patch Nx …` and `Unpatch <id>` go.
2. **GDTF into the show:** `Import FixtureType "file.gdtf"` reads `library/gdtf/` (sub-folders
   allowed), like `Import Image`; the Fixture types tab's Import opens a picker of those files (name,
   manufacturer, revisions, modes read from each file; ones in the show marked) plus Upload from this
   device (lands in `library/gdtf/`). `/Type` names a type already in the show. The old
   `Import "x.gdtf"` / `Import library` go. "Be careful so it works with GDTF revisions and is easy."
3. **One file search:** a single library file index behind the one suggestion engine
   (`lumacore-suggest`): each file kind registers its `library/` folder and a reader for what to
   search on (GDTF: manufacturer, name, revision, modes; later MVR, shows, pictures, profiles).
   `/Type <Tab>`, `Import <Kind> "<Tab>` and every picker read the same index and ranking, through
   the drive door. A library file picked where a show type is needed is imported first, then used.
4. **Insert fixtures… (Draft): the established two steps in our kit.** 1 Choose: one search box (the
   suggestion engine), Show | Library | Both, Manufacturer → Type → Revision → Mode with the mode's
   footprint and channel map, a library file that is a newer revision of a show type marked. 2
   Place: type, mode, footprint, name (suggestions), quantity, first FID, IDType + CID, MVR layer /
   group, class, address (Next free offered), offset — every field part of the one `Store Fixture`
   line, shown live; Create runs it.
5. **Revisions:** a type holds its revisions (same FixtureTypeID); `/Type` takes the newest,
   `/Revision n` picks one. Importing a newer revision changes no fixture: the reply names how many
   stay and offers the line (`Set Fixture … "Revision" 2`). Revisions are told apart by the file's
   SHA-256 and revision text, never by name. Changing a revision needs the claimed patch.
6. **Changing type / revision / mode** (claimed patch): `Set Fixture <sel> "FixtureType" | "Revision"
   | "Mode" …`, from the grid's cells through the same picker. The address is kept; where the new
   footprint collides or runs past 512 the address shows red with a warning until the user resolves
   it, no question at change time.
7. **The attribute map (cross-clone, defined attributes made easier):** one station dialog —
   the attributes the fixtures have values for (how many, where), an arrow, their targets in the new
   type: exact GDTF names mapped already, then a suggestion by feature / geometry, the rest marked;
   tap to change, + for several targets (C1 → R, G, B); scope (programmer, presets, cues) and Merge /
   Overwrite. It opens when a type / revision / mode change leaves values without a target; values
   left unmapped are kept unused and listed, never dropped silently. The same dialog serves a new
   **`Clone`** word (`Clone Fixture 1 Thru 12 At Fixture 101 Thru 112 [If Sequence 1 Thru 10]
   [/Merge | /Overwrite]`, `Clone Fixture 1 Attribute "Gobo1" At Fixture 2 Attribute "Gobo2"`; lum.md
   first). Research for it: values are keyed by (fixture GUID, attribute name) in the programmer, cue
   parts and presets, so a remap rewrites the name and converts through `physical_range` where
   ranges differ; `standard_attributes.json` (346 attributes, features, main / activation) and the
   12f-a resolver exist; nothing maps attributes between types today, and no mode-change path
   exists. An attribute / feature dataset (GDTF, community.smidn.com/docs, consoles) is being gathered.
8. **Universes:** a universe moves to another number by editing its No cell (Draft) or Move to… for
   one or several (`Move Universe 3 + 4 At 20`; the door swaps / keeps spacing; input lines ask).
9. **The claim button: the locked mockup done right** — anchored bottom-right (the slab's last cell,
   on a phone too); a hold fills the button gold left → right over the hold time and drains back when
   released early (reduced motion: no animation); the press hint only on a press; the open-lock icon
   for Live. Kit-wide: the hold ring goes above dialogs.
10. **Live vs Draft:** Live edits only the live columns, every other column (and its header) shows the
    lock and says "Claim the draft to change it"; Draft edits every column, each cell its one line —
    FID (the No cell = Move Fixture), Type / Revision / Mode (the picker; the attribute map when it
    asks), MVR group (pick list), Tags, Class / Position. The Note column returns; the gold dot marks
    your draft edits, the violet one another user's live edits.
11. **The function block** (this round): Insert fixtures…, Patch to… (address mode, offset), Next
    free address, Multipatch…, Info, Sort, Clone…
12. **One DMX map element, two zooms:** zoomed out, the Universes tab's Map view (a bar per universe,
    fixture blocks by type, free runs, red overlaps, multipatch hatched; tap a block selects, a free
    slot patches the selection there); zoomed in, the established slot grid of one universe (rows of 20, `u:aaa`
    labels, FID:CID on a fixture's first slot, ← → universe, Columns). An Address cell or Patch to…
    opens Edit Patch: the fixtures left with a cell per break, the slot grid right, tap a slot =
    patch there; bottom: Patch to · Unpatch · Move to this universe · Next free address · Next free
    universe · Skip patched · Address mode · Offset — each a Patch line.

13. **From other consoles' manuals** (four research agents, 2026-10-18: ETC Eos 3.x, ChamSys MagicQ
    1.9.9, Avolites Titan 19 + Hog 4, ONYX / QLC+ / Lightkey / Capture), taken into this
    round: **update fixture types with a diff** (Eos Update Profile, ONYX Patch Exchange, Titan
    Update Personality: one list of the show's types with a newer library revision, what changed,
    tick and apply); **remembered attribute maps** per (old type, new type) in the show (Titan's
    Exchange Mapping), part of the clone / cross-clone crate; **keep output vs keep meaning** on a
    type change (Eos Preserve Native) and **slot-by-slot ranges** for stepped values (Titan Range
    Mapping, ONYX's honest "what can't map") — both decided with the attribute-map mockup;
    **identify / probe / find** (Eos Flash, ONYX Highlight Tool, Titan Find Fixture: an address or
    fixture flashes outside the programmer; step by footprint until the real unit reacts);
    **swap addresses** between two selections and **re-address keeping spacing or bunched up**
    (Titan, Capture's selection order); **renumber — kit-wide:** in every list and grid, lasso a
    selection, type the first number, and the selection is numbered from there in order (one line
    through the object door); **park on conflict** (Titan: an overlapping fixture keeps its address
    and programming but sends nothing until resolved) with **a conflict list** of one-tap fixes;
    **usage bars** (channels used of 512 per universe, the Map view). The user wants an artifact
    for every one of these before deciding the details. To the roadmap (Later): a patch report
    (CSV / PDF per universe), a DIP-switch helper, an address display toggle (Eos), per-fixture
    patch attributes (intensity %, curve, invert / swap pan-tilt, GM exempt — Eos, Hog), an origin
    flag for unofficial GDTF files (Hog), RDM discovery / auto-patch / "repatch the show to the
    fixtures" (MagicQ, Eos, Titan), drag-to-plan patching (QLC+ 5, Lightkey), MVR import modes Add /
    Readdress / Morph / Clone (MagicQ, for part 4).

14. **Mockups, round 3** (`docs/dev/ui-kit/mockups/2026-10-18-34g4-r3/`): `insert-fixtures.html`
    (v3; changes asked and made: Show is the default scope; the channel map sorts by Structure /
    Channels / Feature group; the file's details strip — description, file, size, GDTF version,
    revision, uploader, FixtureTypeID, thumbnail; the DMX sheet beside the Place form, a free slot
    sets the address; the buttons in their own column; the Fixture types tab on GDTF's
    FixtureType fields, a reference console's columns probed live for comparison) and
    `attribute-map.html` ("a good start": the map a type change opens — mapped / suggested /
    converted / kept unused by feature group, a target picker, + for several targets; a wheel
    slot by slot, pre-filled by slot colour or name; meaning vs DMX output with the two files' real
    ranges; remembered maps and Clone; the Update fixture types list with a diff). Decided on it:
    a **remembered map pre-fills and the dialog still opens** every time (marked "from map n");
    **one Meaning | DMX output switch for the whole map, Meaning the default**.

15. **The attribute map, hardened further** ("we need to harden map attributes more"):
    - **Colour as a whole.** Each stored step's Color-group values are read together as one colour
      into CIE (GDTF emitters / filters / wheel-slot colours / kelvin) and reproduced on the new
      fixture with what it has (emitter mix, CMY, nearest wheel slot by ΔE2000, CTO / CTB / CTC by
      mired), out-of-gamut held at the nearest reachable colour and listed; the map shows Color as
      one row "through colour" with before / after swatches. "We need a very good colour science
      dependency": researched — **`palette`** (0.7.7, MIT/Apache, light, `no_std`: XYZ / xyY / Lab /
      Oklab / CAM16, ΔE2000, Bradford) for the perceptual maths, plus LumaCore's own small
      lighting-colour module (xyY↔XYZ, RGB matrices from GDTF primaries at run time, Planckian /
      CCT / mired, the CIE 1931 table, GDTF Measurement / DMX-profile curves, the emitter vertex
      solver with a Quality blend, the CMY / CTO optimiser, gamut clipping, the
      fallback order: emitter colour → dominant wavelength → nominal by name → the ColorSpace);
      `colorimetry` only if spectral precision is wanted later (heavy, young). The ΔE achieved is
      reported per object; a conversion is always called approximate.
    - **One base window, three uses:** the change map (type / revision / mode change in the claimed
      patch), **Defined attributes** (the show-wide relations), and **Clone**
      (outside the patch, unclaimed).
    - **Defined attributes are rules saved in the show** on the show's attributes, like 12f-a's
      AttributeRule: ordered, first match wins, patterns with `*` and `{n}`, an exact rule beats a
      pattern, scoped to all or to a pair of types; the GDTF families are built-in rules at the
      bottom (visible, read-only, overridable). A hand edit on any map writes an exact rule; a
      change map's Remember writes rules scoped to that pair. Basic built-in rules are being
      derived from community.smidn.com's API (vendor spellings → standard names, console ↔ GDTF pairs,
      families, substitutes when a target lacks an attribute).
    - **Clone:** the established scope — from source to destination fixtures line by line, their values in
      cues, presets and the programmer and their places in groups; never the patch; Merge /
      Overwrite; `If` limits it; Program rights, no claim; the map opens only when the types
      differ. **Its own window:** the pairs on top (reorder, add, replace, remove), a tree of
      everything that holds the source fixtures (sequences › cues, preset pools › presets, groups,
      the programmer) with ticks and value counts, the one Clone line built live, Map… / Merge /
      Overwrite.
    - Editing a row by hand (drag to move, remove, add): two working variants to test and choose,
      `attribute-map-editing.html` — A rows + a target palette, B two columns joined by lines.

16. **Fixture parts — open, being rethought.** Mapping geometries raised the question of a
    fixture's parts (a centre and a ring, pixel cells, the heads of a bar). §28 decided "one object
    everywhere, no subfixtures, cells only for effects"; today a mode with the same attribute on
    two geometries can't be programmed apart (the plan keys values by fixture + attribute). The user:
    "we don't want to follow the [established] subfixture system, we need a better system". A proposal was
    drawn (`parts.html`: parts as a facet of the value, roles, a part focus in the programmer) and
    taken further, provisionally: **part kinds as a pool in Devices** (shape Single / Linear n /
    Grid / Circle; a fixture type's geometries link to them by GUID, suggested by rules on import,
    confirmed, editable in the Fixture types tab; a default set per new show), **one part with
    numbered instances** (cells and heads alike), **resample by position** between instance counts
    (First n / Repeat as choices). Map rows by hand: **lines (variant B)** chosen. Then: "this is too
    close to subfixtures; values directly on the main fixture is bad as well somehow — research with
    multiple agents and come up with something easier for the operator and programmer". Three
    research agents (consoles' multi-element models, operators' pain points, models outside
    lighting) run before the next proposal; the word (`Part` is reserved for cue parts) waits.

17. **Fixture parts: Media server → canvases (controlled like fixtures) → mappings → parts**
    (the user, 2026-10-18, after three research agents: other consoles, operators' pain points,
    models outside lighting — cascade, layer, linked channels, range mask, and the user's own idea
    worked out in full). A built-in media server (the §36 renderer) renders content onto
    **canvases**; a canvas is **controlled like a fixture** (selected, given values in the
    programmer, stored in presets and cues, in groups). **Mappings** attach fixture **parts** (the
    Parts pool's kinds: ring, cells, heads — item 16) to canvas regions. The operator never
    programs a part directly: a whole fixture is programmed as before, a part's look comes from
    the canvas it is mapped to, and the canvas is programmed like any fixture (a static "rings
    red" = a canvas at red with the rings mapped to it). Lessons recorded from the research: the
    mapping must sit inside the console's one merge (MADRIX / Resolume beside a console need two
    operators); samples reach the 60 Hz tick as one more input layer, compiled sample points in
    the plan, a pre-allocated hand-off like DMX input (§24 / §27); a missing frame holds the last.
    Details hardened next. **A canvas is a device of a new kind, "Pixelmapper", outside the DMX
    patch, with an FID** in the one fixture number space (selected, programmed, grouped, stored
    like a fixture; no DMX address — the renderer is its output); generative content through the
    node wire system later. Research backing it: operators' top pains on established consoles / Eos / Titan are cell
    order and layout for effects, the master leaking into the cells (a dimmer effect on pixels
    still dimming through the parent; Eos running an effect on master and cells), an opaque
    parent / child structure that depends on the profile, presets that don't carry across cells
    or types; and separate part objects make type changes fail (Hog: no compound → compound; ONYX:
    no multipart swap; elsewhere: multi → single may lose presets). MagicQ's "element 1 recorded, copied
    to the rest unless set on its own" is the closest good model.
    - **The pixelmapper controls a mapped part completely**; the main fixture's programmer and
      cue values never touch it (no master leaking in). The output stage still reaches it:
      through the canvas (it is a fixture: group masters, Solo, Highlight, Blind), grand master
      and blackout scale the output, and Park on the real fixture or part wins over everything.
    - **A canvas holds layers**, their count set when it is patched (default 4, changed in the
      patch like a mode): a master block (Dimmer, Color tint, Transform) and per layer the same
      block (Layer{n}Opacity, Content, Blend, Color, Speed, Transform, the content's parameters)
      — ordinary attributes that store, fade and map like any; the encoder bar gets a Layer page
      (Master · 1 · 2 · 3 · 4). The word for a layer goes through lum.md.
    - **A canvas patched in Devices also appears in a Canvas pool**, where its setup (size,
      layers, mappings) is configured on the fly as live data — no patch claim.
    - **Mapping placement, two settings:** **3D** — fixtures and parts at their real MVR positions,
      the group's grid giving each member's rotation / mirror so the orientation is right;
      **2D** — laid out like the group grid, the mapping expanding each fixture's pixels by
      itself from its part's shape. Anything can be dragged, scaled, rotated in the editor.
    - **A mapping takes its layout from a group three ways:** **Linked** (the group's grid —
      positions, order, rotation — used live; the group changes, the mapping follows; only the
      whole mapping moves in the editor), **Copied** (a snapshot of the group's grid, then the
      mapping's own: members move and turn freely), **Free** (no group). **Self-expanding:** each
      member's part grows from its grid spot along the member's group-grid rotation (an upright
      batten's 12 plates expand downwards), so pixel order and orientation come from the group.
    - **2D and 3D look clearly different:** 2D is the canvas itself, pixel by pixel, with the group
      grid over it; 3D is the rig in perspective (floor, truss, fixture bodies, axes) with the
      canvas as a plane and a line from every part to its pixel — Front, Top, Orbit.
    - **The canvas editor edits mappings only** (no attribute editing): a sidebar of every
      fixture by group / type / part kind with each part's mapping; tools select, move, rotate,
      scale, align, snap.
    - **The encoder bar gets its own Pixelmapper mode** (12f concept A, the approved bar): the
      strip is the selected canvas's master and Layers 1–4 with live pictures; four encoders of
      Value · Fade · Delay faders over their attribute bars; Page 2 for the rest (Blend, Size,
      Direction, Transform); Mode ▸ the list has Pixelmapper as a new entry.
    - Mockup: `pixelmapper.html` v3 (patch kind and Canvas pool "a good start").

18. **The DMX map, reviewed** (`dmx-map.html`, after the compaction):
    - **An overlap parks both fixtures** (not Titan's "the later one"): every fixture with a
      slot shared with another, or a run past 512, keeps its address and programming but sends
      nothing until resolved. Both are red and on the conflict list.
    - **Identify / Probe / Find: Setup and up, any time** (the patching level); the flash goes out
      on the live rig, so the patch dialog shows a warning strip while one runs.
    - **Drag and drop in the DMX map**, both zooms: a fixture's block (Map) or run (Edit Patch)
      drags along its universe or onto another one; the ghost snaps to slots, gold when free, red
      where it would overlap (a drop there is allowed and parks both); a selected fixture takes the
      whole selection (Keep spacing / Bunch up); the drop is one Patch line. One drag engine: tap
      selects, drag on a fixture moves, drag on empty space lassos (Shift adds).
    - Swap addresses and renumber get their own mockup (`address-tools.html`).
    - **Lasso in every list and grid of the UI** (the user, kit-wide): `<luma-grid>` has it since
      12d; every other list, pool, tile set, tree, slot grid, map and sheet gets the same lasso from
      the kit's one drag engine (`kit/pointer.js`), with the same rules (hold-then-drag, the
      device's `Drag: Auto | Lasso | Scroll`, Shift / Meow adds). A list without the lasso is a
      kit gap, not a window choice.
    - **Swap addresses as drawn** (`address-tools.html`): pick A, Swap addresses…, pick B, preview,
      one line; uneven counts pair up to the shorter list and the rest stay where they are.
    - **One context menu for every list, grid and pool** (the user: "a general context menu with
      renumber etc.; it should be possible to change fixture type, modes, revision and all types of
      data a row has in a list, unified in any list / grid in the whole system"): right-click,
      hold without moving, or the selection chip's ⋯ opens one kit menu on the selection —
      the common actions (Edit… = the edit card, Renumber…, Label, Copy, Move, Delete), a **Set**
      part with every declared column of the row's object (its shared value or ‹different
      values›, the column's editor — pick list, keypad, text, colour; greyed with the reason where
      the rights or the claim refuse it), then the object's own verbs. Built from the registry
      (`ObjectDef` verbs + `Props`), never a per-window list; it replaces the pools' object sheet
      and the grid's right-click = Edit. Mockup `context-menu.html`.
      **Both stay**: Set for one field from the menu, Edit… for the edit card (every field in tabs;
      full screen on a phone).
    - **Renumber = one small floating value dialog** (the user, after a docked panel with a number
      map was "way too much": "the context menu › renumber › floating dialog with simple hints and
      good tools, unified to work with other values as well later"): context menu › Renumber…
      opens a small dialog beside the selection (never over it; draggable by its header; a bottom
      sheet on a phone): a big field with − / +, one hint line ("→ 107 Thru 109 · all free",
      "110–120 taken → 121 Thru 123"), up to four **free chips** that fit the selection, the
      value's toggles (Selection / Grid order, Skip taken / Swap), Apply; Enter / Esc / ↑ ↓. Every
      keystroke previews old → new in the list behind it; Apply sends the one line. **One kit
      element for any value**: a value kind brings its parser, step, free index and toggles from
      the registry — Renumber and Address (Keep spacing / Bunch up, free addresses that fit) are
      the first. The free values come from one free-value index per object kind in the suggestion
      engine (which also suggests them after `At`). Mockup `renumber.html` v2.
      **A taken number: Skip taken by default** (the selection jumps over it, nothing else moves),
      Swap one tap away. The dialog as drawn is approved.
    - **The patch window in Live / Draft, approved** (`patch-grid.html`): the claim button in the
      slab's last cell (hold 0.6 s fills gold left → right and drains back on an early release; a
      press in Live only shows the hint, which goes away; a press on Claimed or the ✕ asks Save /
      Discard / Cancel when the draft changed); Live and Bob's draft lock every non-live cell and
      header with the reason ("Claim the draft to change it" / "Bob holds the draft"); a coloured
      bar under the strip says which state you are in; gold / violet dots, red ⚠ addresses, "no
      universe"; Note back, X / Y / Z split, the path in the title, tree guide lines, counts in
      "fixtures", "+ Layer / + Group" greyed in Live. **Slab states:** greyed cells stay in place
      and say why on a tap; in Live New, Copy, Cut, Delete, Import… need the claim (a copied fixture
      is a new fixture), Select, Oops, Export…, Edit (live fields) work; Paste is greyed with an
      empty clipboard and counts what it holds; New = Insert fixtures…. **Function block:** Insert
      fixtures… and Multipatch… need the claim; Patch to…, Next free address (address is live),
      Info, Sort, Clone…, Offset work in Live.
    - **Deleting an MVR layer / group that holds fixtures asks once** (replaces round 2's "Delete
      deletes the group's fixtures too"; the built question's "unpatched" was wrong both ways):
      "Delete MVR group Truss 1? It holds 96 fixtures" with **Keep the fixtures** (the default:
      they move to the parent node, House, or No MVR group for a layer) · **Delete the fixtures
      too** (naming what uses them: groups, cues, presets) · Cancel; one statement, one Oops step;
      nested groups follow the same answer. The two answers' flags are words for lum.md (with
      the round's other words). The code today deletes the fixtures (`lumacore-patch/src/patch.rs`)
      and must follow.
    - **MVR classes and positions can be labelled** (point 7 holds; part 3c's read-only goes):
      `Label MvrClass` / `Label MvrPosition` in the claimed draft; the name goes out with an export;
      a re-import matches by UUID and shows a renamed class or position as a diff cell to take or
      leave.
    - **The `Fixture.Luma` extras and the other live columns are built this round** (the user
      chose this over waiting for the areas that read them): pan / tilt offsets, DMX inverts (pan,
      tilt), encoder inverts, 3D invert, grid spot (position in the group grid) and grid invert,
      beam angle, cast shadow, follow target, selectable / visible in 3D, P1–P4 (free patch
      values — what they mean is checked against a reference console before building) — each a declared property and
      a patch column (in the Full / Extended column sets), **live** where an established Live Patch accepts it
      (all of these but grid spot's rotation, which is Patch only). Nothing reads them
      until their area is built (the programmer and device sheet, the 3D visualizer); the column is
      there to set and keep them.
    - **Clean up unused stays** (point 7): a function-block button on the Metadata tab (claimed
      draft) that names what it would remove ("4 classes, 2 positions, 1 file") and asks once; one
      Oops step.
    - **Insert fixtures v6, approved "for now"** (`insert-fixtures.html`): Choose opens on the **No
      list** (the show's fixture types by number: No, Name, Manufacturer, Revision, modes count +
      footprints, Used; "Library…" jumps to the columns on Library), Details off; the other view is
      Manufacturer · Type · Revision; **one frame** — only the left part changes, the Mode column
      (name + footprint) and the channel map never move. Place: First FID and Address show the free
      chips; **dragging a new fixture moves only that one** (the line gains `; Patch Fixture n At
      u.aaa`), Shift (Meow) + drag, the Address field or a free chip moves the whole block; an
      overlap parks both. A revision is picked in the columns view only.
    - **The attribute map, v3** (the user's review of v2; `attribute-map.html`):
      - **Change map and Clone are one window, "Map & clone"** (From → To, What, the map);
        **Defined attributes is a sub-dialog it spawns**, not a third mode.
      - **A press on any source opens "Connect to…"**: suggestions (same feature / geometry)
        first, every target attribute by group with what already feeds it, a search, Keep unused,
        several targets tickable; the Color row offers how colour converts (through colour, only
        the wheel, only CMY, split C1 → wheel / RGBW → CMY / CTC → CTO). Dragging from the dot stays.
      - **What (object kinds), proposed:** Values (cues by sequence, presets by pool, programmers
        by user, parked values) · Places (groups with order and grid spot, tags; later layout
        positions, pixelmapper mappings) · the fixture itself (offsets, inverts, MasterReact; name,
        note, paint) — never the patch; quick sets All values / Everything / Only cues / Only
        presets. A type change converts what's ticked and keeps the rest unused (listed).
      - **Wheels and gobos: a smart picker for the targeted fixture** — best matches (ΔE2000 from
        the slots' CIE colours; gobos by family and name), every slot with swatches / pictures, a
        search, Mix with CMY instead.
      - **Values are stored native — strict** ("we need to be very very careful"): degrees, Hz,
        kelvin, CIE, slots by GUID, never a DMX byte; DMX is computed from the fixture's current
        GDTF range at every plan compile, so a later revision that changes a range recalculates
        the DMX and never touches the value; a held (out-of-range) value keeps what it wanted, is
        listed, and is retried when a range grows. DMX output converts once at the change.
      - **Update fixture types handles breaking revisions:** items by severity — must resolve (a
        mode in use is gone: move to another mode / keep the old revision) blocks Update; warnings
        (a grown footprint that overlaps, a removed wheel slot in use, a smaller range, renames)
        carry a pre-filled fix; info needs nothing.
    - **Map & clone v4** (the review of v3; `map-clone.html`, a new artifact): the window is
      **compact, in tabs** — What · Attributes · Colour · Wheels & gobos — with a fixed-height body;
      a **decisions bar** carries what changes everything: Values keep their Meaning | DMX output,
      Remember for this pair (type change), and for Clone "the destination already has values":
      Keep theirs (merge low priority) · Take mine (merge high priority) · Overwrite, plus
      **Referenced presets too** (dependencies, on by default; off = none).
      **What, widened from the reference** (probed: its Clone dialog's Sequences, Groups, Presets,
      Worlds, Layouts, Filters; the DataPool): Values (cues, presets, programmers, parked, effects
      later) · Membership and places (groups with order + grid spot, tags, users' selections,
      layout positions later, pixelmapper later) · the fixture itself · Named, not cloned (key /
      macro / OSC lines naming the fixtures are listed to fix; the reference Worlds have no twin — tags and
      If cover them; never the patch). The reference Filters = **Limit to attributes** (feature families).
      **Wheels: a slot may land on any colour source of the target** — wheel 1, wheel 2, CMY, CTO,
      or a wheel slot with a CMY tint — ranked by ΔE2000 with the source tagged, filter chips per
      source. Defined attributes stays a sub-dialog; **while it is built, a second artifact pulls
      community.smidn.com's API data** so the built-in rules are reviewed before they ship.
    - **After v4** ("a good start, we are onto something — still busy; make it easier for
      beginners"): **no pairs list** (Clone shows From → To only); the window must read for a
      beginner first. **Open, not decided:** how parts meet the map — item 16 links a fixture
      type's geometries to part kinds in the Parts pool (per type, in the Fixture types tab), but
      the map has no part dimension yet (the Aura ring's attributes fall to "kept unused" instead
      of mapping to the target's ring by part kind). Research running: real fixtures from
      community.smidn.com (heavy attribute / geometry fixtures, mode changes, type swaps, naming,
      part-kind rules, predicted problems) → `handoffs/2026-10-18-patch-round3/research/community-fixtures/`.
    - **For beginners and power users both: summary first + a short wizard.** The window opens on a
      plain-language summary ("6 fixtures become Viper Profiles · 9 attributes match · 3 need a
      choice · colour converts as a whole, ΔE ≤ 2") listing only what needs a choice, each with a
      suggested answer, and Apply works from there. The details are the wizard's steps — 1 What ·
      2 What doesn't match · 3 Colour · 4 Check and apply — with Next / Back in a fixed order,
      and any step can be jumped to (the steps replace v4's tabs). Mockup v5 after the research.
    - **v5 reviewed:** the summary "looks better"; **parts are hardened on their own first** (the
      parts model and where parts are used — the clone menu among them) — the clone menu carries
      on with the attribute model until then. The steps were too simplified: **1 What** gets the
      full tree back; **2 Doesn't match** must not repeat the summary — it is **the node graph**
      (the lines, Connect to…) filtered to what needs attention; **3 Colour & gobos** holds the
      colour conversion *and* the slot mapping with the cross-wheel picker; **4 Check** is a real
      review (per object kind, attribute totals, worst colours, held / unused values, a before /
      after example, the line). Mockup v6.
    - **v6: "a very good first step"; the node graph's "Needs attention" is liked.** v7: **Everything
      is the default** of what gets converted; **Colour & gobos in sub-tabs** — Conversion ·
      Colour slots · Gobos — one at a time; **a type change and Clone are one window, no switch**
      — the decisions bar is the same (Meaning / DMX output, Remember for this pair, Referenced
      presets too) and Keep theirs / Take mine / Overwrite joins it only when the targets already
      have values.
    - **Map & clone v7 approved as the first build** (2026-10-18; `map-clone.html`, attribute
      model). It replaces `attribute-map.html` v3's first board; v3's native-values and Update
      fixture types boards stay the reference for those parts. Parts join the window after their
      own hardening.
19. **The round's words, hardened** (2026-10-18, after the compaction; lum.md follows once the
    words are through, then code):
    - **No `Patch` / `Unpatch` verbs — §33 48 stands; item 1 is corrected.** Store creates
      (`Store Fixture 201 Thru 206 At DMX 2.001 /FixtureType 3 /Mode "Standard"`, see below), Assign
      addresses (`Assign Fixture 201 Thru 206 At DMX 2.001 /Offset 20`), Unassign takes it away.
      The UI keeps the labels *Patch to…* / *Unpatch*; the line they run is Assign / Unassign.
      lum.md's stale `Assign FixtureType … At DMX … /Count` example goes (creating is Store).
    - **Unassign always names the kind.** A bare kind after `At` = whichever of that kind it is
      assigned to: `Unassign Fixture 201 Thru 206 At DMX` (every address, every break), `At DMX
      2.001` (that one), `At Fixture 201 DMX 2` (break 2; see the swap below — no `/Break`), `At
      MvrGroup` (whichever group), `At MvrGroup 4`, `At Tag`. A bare `Unassign Fixture 201` is refused (E-CMD-006) naming the forms. Edge case
      accepted: a bare `DMX` after `At` is "any DMX address", not the default object.
    - **`/FixtureType`, not `/Type`** (one name with the object and the property). A number = the
      show's slot (the No list); quoted = a loaded type's name; quoted ending `.gdtf` = a file in
      `library/gdtf/`, imported first in the same Oops step (newest-revision rule). `<Tab>`: the
      show's types first, then library files (the one file index).
    - **`/Every n`, not `/Offset`** — the stride, start to start (`At DMX 2.001 /Every 20`: 2.001,
      2.021, 2.041 …); omitted = packed; under the footprint refused (E-CMD-008 naming it).
      `Offset` stays free for position offsets later (quoted properties). Edit Patch's cell is
      *Every*.
    - **`DMX` reads one part as the universe — DMX only** (`Parts(2)` with a one-part form): `DMX
      1` = universe 1's slots, `DMX 1.001` an address. As a destination `At DMX 1` = the first free
      run in universe 1, a bare `At DMX` = the first free run anywhere (in Store `At DMX 3`);
      the whole selection fits in one run or the line is refused (E-CMD-010), never split across
      universes. *Next free universe* has no word: the button writes the number (`At DMX 7`).
      `Unassign Fixture 201 At DMX 2` = their addresses inside universe 2; `Park DMX 2` / `Info DMX
      2` = the universe's slots. No absolute typing (`DMX 513` = universe 513; absolute stays a
      display mode). `DMX 1 Thru 2.010` mixes levels (E-SYN). The other Parts objects keep a fixed
      count (`Preset 4` stays E-SYN-012, `Master 1` → 2.1 stays).
    - **`Universe` is the object, `DMX` its slots:** name, MergeMode, lines, renumbering are
      `Universe` (`Assign Universe 1 At Sacn 1`); values, Park and destinations are `DMX`. `Assign
      DMX 1 At Sacn 1` is refused (E-CMD-006 naming `Universe 1`).
    - **Taken slots: an exact place is exact, a run skips.** One fixture at an exact address goes
      there (taken = an overlap: red, both parked, a warning); several fixtures, `At DMX n` or a
      bare `At DMX` skip taken slots (each takes the next free run after the one before; the reply
      names what was skipped and whose it is). No flag; forcing a run onto taken slots is one
      fixture at a time.
    - **Addresses are always packed** (the user: "when working with DMX it's always packed, but
      working with groups it should keep spacing"): omitted `/Every` = packed, addressed or not;
      `/KeepSpacing` keeps the selection's gaps (the Address dialog's Keep spacing / Bunch up
      toggle); with it the block keeps its shape, so a taken slot inside it moves the whole block
      to the next place it fits. `/KeepSpacing` with `/Every` is refused (E-CMD-006). Pool Move /
      Copy keep their spacing as before (the range rule).
    - **Swap addresses: `Assign Fixture 201 Thru 204 At Fixture 301 Thru 304 DMX /Swap`** (the
      approved mockup's line in Assign; the user: "a DMX word to say it is the DMX we swap"). Pairs
      trade every break, by fixture; uneven counts stop at the shorter list, the rest stay.
      **`DMX` is a child of `Fixture`, owner first:** `Fixture 301 DMX` = its addresses (every
      break), `Fixture 3 DMX 2` = break 2 (replaces `/Break`); `Info Fixture 3 DMX`, `Park Fixture 3
      DMX` (its raw slots). Without `/Swap`, `At Fixture 301 DMX` is an exact place (taken =
      overlap). **`/Swap` = "trade with what is there"**, only here (Move already swaps); the
      playback hold verb `Swap` is another registry (flags are declared apart, like `/Universe`).
    - **Renumber is Move; §35 4 stands (Move onto a taken number swaps, every pool).** The renumber
      dialog's default *Skip taken* writes **`/Skip`** (`Move Fixture 101 Thru 103 At 107 /Skip`:
      101→107, 102→109, 103→110; every pool takes it), its *Swap* toggle the bare line. No `/Swap`
      on Move.
    - **Order: typed, or `/By <column>` — no `/Sorted`.** *Selection order* is the line as typed
      (`Move Fixture 105 + 101 + 103 At 107`: 105→107 …); *Grid order* writes `/By` + the grid's
      sort column (`Move Fixture 101 Thru 140 At 201 /By Address`), any declared column, the flag
      `Compact` already has.
    - **The MVR node delete answers are the Tag pair:** `Delete MvrGroup 3 /KeepChildren` (its
      nested groups and fixtures move to the parent; under a layer, No MVR group) · `/WithChildren`
      (the fixtures go too, the question naming what uses them). Children = nested groups +
      fixtures here, child tags for a Tag — read from each tree. No new flags.
    - **Selecting by the second id: `Fixture <IDType> n`** (`Fixture Houselights 4`, `Fixture
      Houselights 1 Thru 4 + Fixture 101`, `Fixture Channel 12 At 50`). The second id is MVR's
      `CustomIdType` + `CustomId` (decided 2026-10-17, "34g4 patch hardened" 5; built in part 1 as
      the fields `"IDType"` / `"CID"`); one object word, the type is the existing choice word in
      Fixture's id slot (the slot's kind decides — `Fixture Effect 2` is the ID type, not the
      Effect pool); no per-type keywords (Effect and Media would collide). `Fixture Fixture 4` is
      refused (E-CMD-006, "that is Fixture 4": type 1 is the FID).
    - **Clone: `Clone <fixtures> [FeatureGroup … | Attribute …] At <fixtures> [If <objects>]`** —
      a new verb, the established word and meaning; everything is the default (v7). The attribute limit is
      owner first under the source (`Clone Fixture 101 Thru 106 FeatureGroup "Position" + "Color"
      At Fixture 201 Thru 206`, `Clone Fixture 101 Attribute "Pan" + "Tilt" At Fixture 201`); the
      object limit is `If` (the established form: `… If Sequence 1 + 2 + 7`). Uneven counts (6 → 4) and From =
      To are refused (E-CMD-006).
    - **A destination with values: `/Keep` · `/Merge` · `/Overwrite`** (existing flags): no flag =
      one ask (Keep theirs · Take mine · Overwrite · Cancel; a stored line must answer); `/Keep` =
      theirs stay where both have one (merge low), `/Merge` = mine win (merge high, Store's
      sense), `/Overwrite` = theirs go.
    - **The decisions bar: `/KeepOutput` · `/Remember` · `/NoDependencies`** (on both the type
      change and Clone; defaults need no flag: Meaning, not remembered, referenced presets cloned).
      `/Remember` is the existing flag (not `/RememberMap`); `/NoPresets` was turned down (it reads
      as "not into presets", which `If` says).
    - **Maps are objects: an `AttributeMap` pool in Devices, `/Map n` names one.** A map is a pair
      (from type / mode → to type / mode) with its lines; the window creates or edits one; `/Remember`
      makes it the pair's default (pre-fills next time); without it the map is deleted after the
      apply, in the same Oops step. No `/Map` = the pair's remembered map, else the automatic rules
      (Defined attributes). `List` / `Info` / `Delete AttributeMap`; several maps per pair allowed.
    - **Defined attributes are `AttributeRule`s with an `Attribute` target** (12f-a's object, one
      object for both jobs): `Store AttributeRule "Dim*2" Attribute "Dimmer"` beside the grouping
      rules (`… FeatureGroup "FX"`), ordered, first match wins; show-wide only — the per-pair scope
      is an `AttributeMap`'s lines (replaces "scoped to all or to a pair"); `List AttributeRule`
      shows the built-in GDTF families last, read-only, overridable. A profile's rules only group:
      `UserProfile AttributeRule … Attribute …` is refused (E-CMD-006).
    - **A canvas layer is `CanvasLayer`, owner first** (the `Mvr` prefix precedent; a bare `Layer`
      is never a word — MvrLayer, value layers, the encoder's Layer page): `Fixture 900 CanvasLayer
      2 At 50` (its opacity), `Fixture 900 CanvasLayer 2 Attribute "Speed" At 30`, `Copy Fixture 900
      CanvasLayer 1 At CanvasLayer 2`, `Off Fixture 900 CanvasLayer 3`. The name is reserved now,
      built with the pixelmapper. **`Part` stays reserved for cue parts** (fixture parts get their
      word when parts are hardened).
    - **Store takes the address with `At`, no `/Address`** (a flag carries a value, never a
      target; options come last, E-SYN-003, so `At` stands before them): `Store Fixture 201 Thru 206
      At DMX 2.001 /FixtureType 3 /Mode "Standard"` — every
      Assign destination rule as is (exact / run, taken slots, `/Every`, `/KeepSpacing`, bare `At
      DMX`, `At DMX 3`), one statement, one Oops step; without `At` no address. `Store … At` only
      for objects with a place (Fixture today); `Store Group 3 At DMX 1` is refused (E-CMD-006).
    - **Multipatch is the established form** (probed in a reference manual: Multipatch keyword, Add Multipatch
      Fixtures, Assign Multipatch Fixture — a virtual fixture mirroring its primary, its own
      address, name and place in layouts / 3D, not in the selection grid): `Fixture 201 Multipatch
      1`, owner first. `Store Fixture 201 Multipatch 1 Thru 2 At DMX 5.001` creates (the amount as
      numbers; `Store Fixture 201 Thru 206 Multipatch 1 At DMX 5` one each), `Assign Fixture 201
      Multipatch 1 At DMX 5.020`, `Delete Fixture 201 Multipatch 1`; selectable — a value goes to
      the primary, every copy follows (`Fixture 201 Multipatch 1 At 50`); its name and place are
      its own. A global `Multipatch n` (an id that changes with the patch) is not a form.
    - **Flags get sorted into groups later** (roadmap): `/Keep` · `/Merge` · `/Overwrite` one group,
      for the reference and the manuals.

**`DMX` becomes the object (§33 48) in part 1a; `DMX In 1` → `DMX 1 /Input`** (decided
2026-10-19, one question): bare `DMX n` / `DMX n.aaa` is Info (the output sheet, a channel),
`/Input` reads what the input lines bring in on the same slots; `DMX 1.001 At 255` (the tester)
stays planned; `Park DMX 1.001` as before.

**4d4 hardened (2026-10-19, one question at a time, after the phone round; mockups
`docs/dev/ui-kit/mockups/2026-10-18-34g4-r3/touch-move.html`, `map-cards.html`,
`universe-crossing.html`, `ghost-wrap.html`; all four decided, the build waits for the go).** Item 18 decided the desk (tap
selects, a drag on a fixture moves, a drag on empty slots lassos); §37 made the phone's slot grid
52 rows that one finger scrolls, so a drag on a run was taken.
1. **On touch the hold is the press: hold lifts** (`touch-move.html`; the user: A, "A is good but
   it is hard to see under the finger when you hold that it changes to move state"). Hold still on
   a run (the profile's hold time, the ring): the run lifts and follows the finger, the ghost snaps
   to slots (gold free, red where it would park both), let go is the one `Assign Fixture … At DMX
   u.aaa` line; a selected run takes the whole selection (Keep spacing / Bunch up). Hold on empty
   slots is the lasso, as everywhere in the kit; hold and let go without moving is the context
   sheet; a plain finger scrolls (the device's `Drag: Auto` / `Scroll`); in `Drag: Lasso` a plain
   drag on a run moves at once and on empty slots lassos — the desk's rules. Not chosen: a ≡ grip
   on the current run (covers the first slot, only the current run moves), a drag on a selected run
   moving at once (a scroll that starts on the selection moves it), the Drag setting alone (a phone
   is Auto, nobody would move by drag). Edit Patch's hint line says "hold a run to move it"; a tap
   on a run selects it in both zooms (today Edit Patch patches the current fixture over it — the
   engine fixes that).
   **The lift cue** (the user: A1): on the lift the run rises in place — brighter, a shadow, the gold
   outline — and the rest of the grid dims a little; a chip floats above the finger naming the run
   ("Fixture 102 · 1.015") and, as it moves, where it lands ("Fixture 102 → 1.078", red with
   "parks both"); it follows the finger (flips below it near the top edge) and goes on release. A
   phone gives one short vibration on the lift where it has one (`navigator.vibrate`; iOS has
   none). The desk's Map drag shows the same chip under the pointer. Not chosen: the grid dimming
   alone with a gold footer line, the header reacting alone.
2. **The phone map's cards take the selection** (`map-cards.html`; the user: C). The desk's "tap
   a free run with fixtures selected patches them there" is a phone's tap on a card: while
   fixtures are selected (the selection chip — on the Map it sits in the tab strip, there is no
   No cell), every card grows a gold cell naming where they would land, "Patch here · 2.001" —
   the first free run that fits them one after another (the Patch tool's line, `/Every n` when
   the list has an offset); a universe without such a run says "No room" (greyed). The tap is the
   one `Assign Fixture … At DMX u.aaa` line; the card's body still zooms into the slot grid. No
   drag on the phone's Map: the drag lives in the slot grid (another universe from there is
   item 3). Found on the way, fixed with it: a card opens Edit Patch with the **Patch list's
   selection** (today the setup hands it the map's own block picks, which a phone never has, so
   Edit Patch opened empty). Not chosen: the cards as doors only (a phone has no way to another
   universe until item 3), a lifted run dragged up over the universe bar onto a list that drops
   over the grid (one continuous gesture, but a drag that changes screen mid-gesture: the heaviest
   to build, the hardest to find; the same bar is a candidate for item 3), "Move to universe…" in
   the context sheet (a second way beside the context menu's Set › Address).
3. **A lifted run crosses universes by the ‹ › spring** (`universe-crossing.html`; the user: A).
   Edit Patch shows one universe, as established ones do, and reaches another without a drag already (‹ ›
   then a tap on a slot patches the selection there; the function row's *Move to this universe*
   takes its first free run, *Next free universe* the next empty one) — this decides only what a
   **lifted run** can do: held still over ‹ or › for the hold time (the button arms and fills
   like the hold ring), the grid pages to that universe with the run still lifted and the ghost
   re-planned against its runs (the same slot numbers under the pointer; gold / red as always);
   carry on to a slot and let go there, one `Assign` line; let go on the button puts the run
   back. The same ‹ › sit in the phone's bar above its grid of rows of 10. **One kit rule, from
   the pointer core: a lifted thing held still over a pager or a tab springs it** — the setup's
   tabs, a pool's pages, every ‹ › (the desk's folders and every phone's home screen taught it);
   a window that pages without it is a kit gap. Not chosen: paging by holding at the grid's edge
   (invisible, and a phone's edges are the OS's back gesture), the universes dropping as a list
   from the bar's name with hover paging the grid beneath and a drop on an entry landing at its
   first free run (any universe in one hover, but a list that appears mid-drag and a second kind
   of drop target: the most to draw and to teach), Map only (the Map's block would cross and Edit
   Patch's run not: two drags, two rules).
4. **The ghost that wraps is a ribbon, the FID again on every continuation** (`ghost-wrap.html`;
   the user: C, and red per run). The ghost's slots are painted (gold free, red parks; an outline
   alone is faint under a thumb and across the dimmed grid) and each part of a run is one ribbon:
   rounded where the run begins and ends, cut square at the row break with ⤶ at the row's end and
   ⤷ where it goes on, so two parts never read as two runs; the FID sits on the first slot and
   again, dimmer, on the continuation's first slot (⤷ 102), so a band that starts a row is named
   without scrolling back — rows of 10 and 20 alike, a run longer than a row the same way. **Red
   per run**: with the selection lifted, the run that would overlap is red and the rest stays
   gold (parking is per fixture); the chip names it ("… → 1.011 · parks Fixture 103"; one run:
   "parks both"). Today's outline-only `.lc-dmap-slot.ghost` (Insert fixtures' sheet) takes the
   same look when 4d4 builds the engine; a patched run that wraps may take the ribbon later. Not
   chosen: painted slots with the FID once (two bands that could be two runs), the ribbon without
   the second FID, the outline alone, the whole ghost red.
**4d4 built (2026-10-19)**: the pointer core's one drag engine, `lift` (`kit/pointer.js`: tap ·
lift · lasso · context · a plain finger scrolls in Auto / Scroll, two fingers in Lasso · pagers
spring after the hold time, `.lc-sprung` fills), on which `<luma-dmxmap>` drags in both zooms
(`kit/dmxmap.js`: the ghost block on a bar, the ribbon over slots with the FID again, red per run,
the chip, the lasso box, `layout` one / spacing (`/KeepSpacing`) / bunch, `springs`, a redraw
mid-drag keeps the lift); Edit Patch passes ‹ › as springs, takes the drop's line, makes a tapped
run current (added to its list when new) and the lassoed runs its selection; the setup keeps a
grid's selection when its tab is left (`kept`), so the phone's cards show "Patch here · u.aaa"
and a card opens Edit Patch with the Patch list's selection; a hold on a block or run opens the
Patch list's context menu (`mapContext`; Renumber… / Address… preview in the list). Tests
`kit-4d4.spec.ts` on all four projects.

**RunTool / EditTool, hardened (2026-10-19, one question at a time, asked at part 3d "lasso
everywhere": what does a lasso do in a pool, where a tap runs its action?).** The user: "we need a
unified toolset that works within standard pool windows and are in sync across windows". Decided
so far:
- **Two modes for the whole user**, the same in every window, dialog and screen: **RunTool** (the
  default: a tap runs the object's action — Group 3 selects, a sequence goes —, a hold takes the
  window to full screen) and **EditTool** (a tap selects, a drag on an object moves it, a drag on
  empty space lassos, right-click / hold opens the one context menu). Names: Run / Edit with
  "Tool" so neither collides with the Edit verb.
- ~~`EditTool` is a mode object~~ (replaced by the `Tool` object, "The words" below).
  Its one setting is the profile's default tool (below).
- **What EditTool selects is per user, per screen** (the user: "having it individually per window
  will confuse the user"): every window on that screen showing the same kind shares the picks.
- **Every window and dialog follows it**: every pool window, the sequence sheet, the Tags pool,
  the setup grids and every other window and dialog.
- **EditTool is shown everywhere**: the windows' top bars, the sidebars and the top bar turn
  orange while it is on. An artifact comes first, to test.
- **The sequence window is renamed the sequence sheet** (a name only), and it gets the lasso too.
- The artifact: `edittool.html` (https://claude.ai/artifact/QGpFrYycTj3xCNQNtEfi62, v4) — the indicator in three looks, two Group pools sharing picks, the sequence sheet, a setup grid.
- **The indicator is the orange edge line, tinted** (the user, at v1: "best but it is too much"):
  a 2 px line along the bottom of the top bar and every window bar and the inner edge of the
  sidebars, its orange mixed toward the line colour; **only the line changes**, the bars keep
  their fill (the user, at v2); a strength slider settles the mix (55 % to start).
- **Window focus with Tab** (the user, at v1): Tab focuses the next window, Meow+Tab the one
  before; the focused window has a frame (gold, orange-white in EditTool); arrows move a cursor in
  it, Enter taps (runs in RunTool, picks in EditTool), the Menu key opens the menu in EditTool. On
  the command line Tab completes what you typed, on an empty line it goes to the windows.
- **Esc always leads back to normal** (the user, at v2: "the user should be able to always press
  esc a lot to get back to normal state and cmdline selected"): every Esc selects the command line
  and undoes one layer, the top one first — a menu or dialog closes · a full-screen window comes
  back · the picks clear · the tool returns to the user's default. A few presses always reach the default
  tool, nothing picked, the command line selected.
- **A setup grid selects in both tools** (asked, the recommended answer): Patch, Universes,
  Network … are editing lists, so a tap selects rows and cells as today in RunTool too; EditTool
  adds the edge and the same lasso / move / menu rules.
- **Insert toggles EditTool by default** (asked): a built-in UI key (`ui.ToggleTool`, its line
  `Toggle Tool`) bound to Insert — not a core key: the user rebinds or unbinds it (`Bind`), like
  any UI key (§17). Insert is a special key nothing used (fn+Enter on a Mac laptop).
- **Each kind keeps its own picks** on a screen (asked, the recommended answer): groups, cues,
  universes … each hold theirs; the context menu acts on the kind of the window it opens in;
  Esc's "the picks clear" clears every kind at once.
- **One setting: the default tool** (the user: "one setting, default tool, so esc will exit to
  the default user tool"): `Set UserProfile "DefaultTool" Tool "Edit"` (a declared
  link property, `Tool "Run"` by default); you start in it at login, and Esc's last step returns to it
  (not always RunTool).
- **The edge strength is 55 %** (the user): the orange mixed 55 % into the line colour.
- **Hardened (2026-10-19).** It replaces part 3d's plain "lasso everywhere": the lasso, move
  and pick rules are EditTool's. **Built after the patch window** (the user: "put this on
  roadmap after the patch window"; the roadmap item "RunTool / EditTool + the lasso").
- **The words, hardened (2026-10-19, the user: "a better EditTool wording so i can Tool "Edit" or
  Tool "Run""): one object, `Tool`**, replacing the `EditTool` mode object above. Per user, in
  `Runtime.Users.{User}` (never saved, no Oops step), with two built-in entries by number and
  name — `Tool 1 "Run"`, `Tool 2 "Edit"` — that can't be stored, deleted or renamed:
  `Tool "Edit"` / `Tool 2` switches to it (its default verb), `Toggle Tool` flips between the
  two (Insert's line), `Info Tool` shows yours and your default; `Set UserProfile "DefaultTool"
  Tool "Edit"` is a link by any target form (default `Tool "Run"`). A bare `Tool` sets your
  default object (`Tool>`), as every counted object does. Quoted, because a bare `Edit` is the
  verb. Names on screen: "Run tool" / "Edit tool". `Tool` was a free word (the group editor's
  Tools button is only a label). More tools may join later as entries.

**Round 3, the parts (cut 2026-10-18, at the go; each committed on its own, the mockups the
reference):** 1 the words (1a Store / Assign / Unassign `At DMX` with `/FixtureType` `/Mode`
`/Revision` `/Every` `/KeepSpacing`, the one-part `DMX n`, the old `Patch` / `Unpatch` / `Import
"x"` go and the demo builder is ported · 1b `Fixture n DMX`, `/Swap`, Move `/Skip` `/By`, `Fixture
<IDType> n` · 1c the MVR node delete answers, Multipatch) · 2 Import FixtureType, the one library
file index, the GDTF picker, revisions · 3 the kit: the one context menu, the value dialog
(renumber / address), lasso everywhere, the claim button's hold ring, slab states · 4 the patch
window: Live / Draft grid, function block, Insert fixtures v6, the DMX map element (Map view, Edit
Patch, drag, swap, usage bars, park on conflict + list, Identify / Probe) · 5 type / revision /
mode change: native values (strict), red addresses, Update fixture types with a diff · 6 Map &
clone: the AttributeMap pool, AttributeRule → Attribute, Clone, the window v7 (CIE colour, the
cross-wheel picker, the community-data artifact) · 7 the Fixture.Luma extras, the live columns,
Clean up unused, MvrClass / MvrPosition labels.

**Round 3, the left-out items hardened (2026-10-19, one question at a time, after 4c2):**
- **Multipatch…: a bare `Multipatch` under Store is each fixture's next copy** (asked, the
  recommended answer): `Store Fixture 201 Thru 206 Multipatch [At DMX …]` adds copy 2 to a
  fixture that holds one and copy 1 to one that holds none, packed from the address; a numbered
  `Multipatch n` stays exact (the next number only, as today). Multipatch… in the function block
  = the address value dialog, then that line (needs the draft). A bare `Multipatch` in Delete /
  Assign is refused, asking which copy; `Info Fixture 201 Multipatch` lists them as today.
  lum.md follows.
- **The Sort cell**: an artifact first (the user), `patch-sort.html`
  (https://claude.ai/artifact/A49syVA5BEd817XdYuPjjq, v1): A the grid's order (a view cell) · B
  renumber by a column (`Compact Fixture /By …`) · C no Sort cell.
- **Sort is the grid's order (A, the user):** a view cell in the function block showing the column
  the patch grid sorts by; a tap picks another (FID, Name, Fixture type, Address, MVR group) — the
  user's grid state like a header tap (its `Set GridColumns` line), no show change, no Oops step.
- **Offset = spare slots between fixtures** (asked, the recommended answer): a cell of the function
  block the DMX actions use — Patch to…, Next free address, Multipatch… and Insert fixtures pack a
  selection with n spare slots after each fixture (they write `/Every <the largest footprint of the
  selection + n>`; 0 = packed, no `/Every`). Kept on the user's profile, like a window setting.
- **Insert fixtures' Place sets the class** ~~and the second id~~ (asked; the second id dropped
  with "No CIDs" below):
  Class = a pick list of the show's MVR classes (`Assign Fixture … At MvrClass n`); IDType = a
  pick of MVR's types, CID = the first number, counting up like the FID (`Set Fixture … "IDType"
  Houselights; Set Fixture … "CID" 1 Thru 6`), each its own statement after the Store. The user,
  on it: "i dont really like CID, i believe tags will be superior to this" — the second id stays
  MVR's; how tags and second ids relate goes with the second-id hardening on the roadmap.
- **A fixture's class by hand: `Assign Fixture … At MvrClass n`** (asked, the recommended
  answer), like its MVR group — one class, `Unassign Fixture … At MvrClass` takes it out. Classes
  still come only from an import (no `Store MvrClass`); with none, the Class field says so.
  lum.md follows.
- **The thumbnail: in Details only** (asked, the recommended answer): read from the file (a library
  file, or the show's embedded one) on demand; the lists stay text.
- **An upload on a member reaches every station** (the user: "all stations needs to have it when a
  member saves the patch, basically all members has to save the same luma file and be in perfect
  sync"): the member sends the file to the master over the LumaSession stream as media uploads
  do, before its Store / Import line runs there; the type then rides in the show with its file
  (revisions embed their `.gdtf`, files a station lacks come by SHA-256), so every station holds
  it and every save writes the same `.luma`.

- **Reopened for discussion** (the user, later on 2026-10-19: "Update the roadmap to instead discuss
  CIDs" — the roadmap item "Discuss CIDs (second ids) against tags"; nothing below is built). Was:
  **No CIDs: tags instead** (the user, 2026-10-19: "i believe we dont want CIDs anymore and rely on
  tags instead"; asked what stays, then "Keep MVR's data only"). MVR's own `CustomIdType` +
  `CustomId` (a fixture's second number from MVR's fixed list — "CID" was our short name for it)
  stay as **read-only data**: an MVR import keeps them, an export writes them back, `Info` shows
  them. Everything built to use them goes: the id kind in the grammar (`Fixture Houselights 4`,
  lum.md's `segment = OBJECT [ id-kind ] [ ids ]`, `LumShape::id_kinds`, the parser, the
  resolver's second-id walk), `"MoreIds"` (`Fixture.Luma.more_ids`), the writable `"IDType"` /
  `"CID"` properties, and Insert's IDType · CID field (decided above, now dropped). Tags select
  and group what a second id did (`Fixture If Tag "Houselights"`). The roadmap's "several second
  ids — harden" item closes. **Not now** (the user: "dont touch the CID for now"): the roadmap
  item "No CIDs: take out the second-id uses".

- **The violet dot, until you leave** (asked, the recommended answer): the draft remembers which
  cells a live edit brought in and who made it; the grid shows a violet dot ("Bob changed it live")
  until the holder saves or discards; the holder's own gold dot wins on a cell they changed.
- **MVR has one second id per fixture** (the user asked "can a fixture contain multiple CIDs?"):
  an MVR fixture holds one `CustomIdType` + `CustomId`; several were only our `"MoreIds"`, gone
  above. An import takes it in as data (the user: "if MVR has CIDs we should take it in, but we
  still like tags better").

**The left-out items built (2026-10-19), but the CID (not now, the user):** the words (bare
`Multipatch` in Store, `Assign` / `Unassign … At MvrClass`); the function block's **Multipatch…**
(value kind `copies`: the address dialog starting nowhere, the fixtures' own slots taken, its
`template` line), **Sort** (a view cell: a pick of the grid's columns, `<luma-grid>.sortBy`, the
cell shows "Sort · Address") and **Offset** (a view cell on the keypad, kept as the patch grid's
state `Set GridColumns "patch.fixtures" "Offset" n` — `GridState.offset`; Next free address,
value kind `packed`, and the address dialog add `/Every <largest footprint + n>`); Insert
fixtures' **Class** pick list (`Assign … At MvrClass n`; the feed's classes carry `at`) and its
Every from the Offset. Multipatch… is on the phone's slab too. The **thumbnail** (`GET
/api/fixture-type/thumbnail`: the PNG, else the SVG named by the file's `Thumbnail`, for a library
file or a show type's stored file) shows in Details. **A member's upload reaches the master**:
`LinkOut::Upload` / the stream's `Upload` carry `gdtf`, the master keeps the file in its
`library/gdtf/` before the Store / Import line arrives (SESSION_PROTOCOL 21). **The violet dot**:
`PatchDraft.marks` (fixture GUID → row field → who), filled when a live line is rebased into
someone's draft (`live_draft::live_cells`: name, note, address, position, master react),
published with the draft and sent to the holder as `live_changed`; the grid draws it where no gold
dot is, titled "Ann changed it live"; it goes with the draft.

**Round 3 part 4d3b built (2026-10-19): Edit Patch.** `<luma-dmxmap>` got its second zoom
(`zoom: 'slots'`: one universe's 512 slots in rows of 20, `u:aaa` labels, the FID on a run's first
slot, the current fixture gold, the picked outlined, overlaps striped, `flash` slots blinking; a tap
is a `slot` event) and the Map view its **Zoom in** (a button under each universe's name, or a
double-tap on the bar). `kit/editpatch.js` (`openEditPatch`, in the kit and the Showroom): the
fixtures left (a `<luma-grid>`, FID · Name · Address), the slot grid right (‹ › universe), the
tools below — **Patch** (a tap patches the current fixture there and goes on to the next; **Skip
patched**, on by default, goes on only to one without an address; **Layout** One at a time · Keep
spacing (`/KeepSpacing`) · Bunch up (the Offset's `/Every`) moves the whole selection from the
slot), **Probe** (`Identify DMX u.aaa`), **Find** (`Identify DMX <footprint> /As Fixture n`, ‹ ›
a footprint, Patch here), **Identify** (`Identify Fixture …` / `Off Identify`), Unpatch, Next free
address, Next free universe (the first empty one), Move to this universe. A red strip says what
flashes (Stop = `Off Identify`); closing (✕, Esc, the patch dialog going) runs `Off Identify
Fixture Thru; Off Identify DMX Thru` when it started one. The four ways in as decided (the value
dialog's `more` link: Slot grid…, the typed address kept); not on the phone slab (see the
decision). It follows the `patch` feed while open. Setup and up changes, anyone looks. **Found on
the way:** the station's Identify check took client ids for users (every Identify ended at once as
"logged out"): it reads the present users (`state::presence`), with the Devices holder check; and
`<luma-grid bar: false>` still showed an empty selection bar (`.lc-grid-bar[hidden]` was
overridden — Insert fixtures had it too). Not yet: Swap addresses and Multipatch… inside Edit
Patch (the context menu and the function block have them), drag and drop and the lasso (4d4),
another user's Identify in the strip.

**Round 3: the ways into Edit Patch (2026-10-19, `edit-patch-entry.html` v1; the user: "Do the
recommended").** Patch to… stays the small value dialog (3c, approved); Edit Patch — the DMX map
zoomed in, with Find, Identify, Probe and the address tools — opens on the selection from four
places: the function block's **Edit Patch…** cell (beside Patch to…; not on the phone slab — one
more slab row leaves the phone's patch grid no room, so a phone opens it from the context menu or
Slot grid…), the
value dialog's **Slot grid…** link (grows it into Edit Patch on the same fixtures, the typed
address kept), the context menu's **Edit Patch…** (beside Patch to…), and the Map view's **zoom
in** on a universe (a button on its bar, or a double-tap; that universe, the map's selection). The
Address cell keeps opening the value dialog (no double-tap, no ⌗). Opening runs no line; closing
runs `Off Identify Fixture Thru; Off Identify DMX Thru`. Insert fixtures' Place shows the same
slot grid inside its dialog (4c3), not a way in. Supersedes decision 12's "an Address cell or
Patch to… opens Edit Patch".

**Round 3 part 4d3a built (2026-10-19): Identify, the word and the output.** 4d3 is cut in two:
4d3a Identify (server) · 4d3b Edit Patch (the slot grid, the dialog, Find, the warning strip, the
leave's Off lines). In 4d3a: the `Identify` object (`objects::patch::identify`: `LumShape::ONE`
owning `Fixture` and `DMX`, default verb On, Off, `/As`; Setup, reading anyone's; scope Master — it
runs on the station that sends), one per user in `AppState::identify` (`identifying::Identifying`:
targets, what, start), published with the engine input (`EngineInput::identify`; a change
republishes). The render plan compiles the targets into `identify::IdentifyTier` (every break and
copy of a fixture — a conflict-parked fixture too —, a raw run, or a run `/As` a fixture's shape;
Park's slots left out) and the tick writes it over the merged output after DMX input: defaults,
dimmer sine 15 → 100 % / 2 s, hue round 6 s through ColorAdd / ColorRGB R G B and ColorSub (CMY;
**the simple RGB mapping, not CIE yet** — CIE comes with the attribute map, part 6), a wheel-only
fixture's colour wheel stepping its slot sets 1 s each, a raw probe 38 → 255. The station's ends:
`identify::tick` (with the echoes' housekeeping) ends one whose user is gone or that ran 5 min,
`Unclaim Root.ShowData.Devices` ends the holder's — each logged "Identify left on (Admin: Fixture
104) — ended (logged out | 5 minutes | Unclaim)". Never an Oops step. **Found on the way:** an
option's value naming an object with ids (`/As Fixture 104`) was read as the value word `Fixture`
when the word is both; the parser now takes an object followed by ids as a target there.

**Round 3: Identify, Probe and Find hardened (2026-10-19, one question at a time, before 4d3).**
Edit Patch's function row needs only these new words: *Move to this universe* is `Assign … At DMX
u`, *Next free universe* the same line with the first empty universe's number (`At DMX 9` creates
it), *Skip patched* only a tap rule of the dialog. lum.md "Identify" holds the lines.
- **`Identify` is one word** (A, the user): `Identify Fixture 101 Thru 106`, a bare `Identify` =
  your selection, `Identify DMX 1.150` (Probe: an address or a run, patched or not), `Off
  Identify [target]`. One namespace: it is an object owning what it flashes (`Fixture`, `DMX`),
  its default verb On — like `Off Sequence 1`.
- **What it sends** (the user: "GDTF default values + sinus wave on dimmer channel and rainbow
  color sinus wave"): the fixture's GDTF defaults; the dimmer a sine 15 % → 100 % every 2 s; the
  colour round the hue circle every 6 s through its mixing (CIE, as the attribute map); a
  wheel-only fixture steps its slots, 1 s each; no colour, dimmer only. A probed raw address: 38
  → 255 in the same 2 s sine.
- **Above everything but Park**: programmers, playback, group / grand masters (a blackout too),
  Solo and DMX input leave an identified channel alone; a parked channel stays parked. A fixture
  parked by a conflict flashes on its own slots (that is how you find it). In a LumaSession it
  runs on the station that sends; never stored, never an Oops step, never replicated as show
  data. Setup and up (§34 round 3, 18); the warning strip names who runs one, with Stop.
- **Per user, ending itself**: one Identify per user (a new line replaces yours, others' go on);
  it ends on `Off Identify`, when the dialog that started it closes, at logout or a lost holder,
  and after 5 minutes. **Leaving the patch** (✕, Unclaim, the dialog closing; the user) runs `Off
  Identify Fixture Thru; Off Identify DMX Thru` as you, then the station checks that nothing of
  yours is still identified — left over, it is ended and logged to the System Monitor ("Identify
  left on (Admin: Fixture 104) — ended"); a test covers every leave path ("make sure … identity
  is never forgotten on").
- **Find = probe steps, one Assign at the end**: Find probes the current fixture's footprint at
  its address `/As Fixture n` (sent the way that fixture would: defaults, dimmer sine, rainbow),
  ← / → move the probe one footprint along (each line replaces the last), nothing is patched
  while searching; *Patch here* runs one `Assign Fixture n At DMX u.aaa` (one Oops step).
- New option `/As` (Identify's probe in a fixture's shape). The lines are on lum's PENDING until
  built; the syntax crate's own vocabulary knows them (its cursor test now takes a leading `Thru`
  as ids, as lum.md's `Group Thru` always meant).

**Round 3 part 4d2 built (2026-10-19): the DMX map, zoomed out.** The kit's `<luma-dmxmap>`
(`kit/dmxmap.js`, in the Showroom): a row per universe — its name, a usage bar and "70 / 512" —
and a bar of its 512 slots: a block per run (FID or `201·M1` for a copy, its start address;
coloured by fixture type from the palette tokens, a copy hatched), free runs dashed, a parked run
striped red; an overlap shares the bar's height (the run on top in the lower half, the one under
it in the upper half) so both stay in sight. Below, the conflict list with its fixes, or "No
overlaps — every fixture sends". Tap a block selects its fixture (Shift / Meow adds; the last
line names them), tap a free run with fixtures selected runs `Assign Fixture … At DMX u.aaa` at
the tapped slot; fixes and free runs need Setup (greyed with the reason). A grid declares the map
as its second view (`Grid.dmx_map`; the Universes grid: List | Map), drawn by `<luma-setup>` from
the area feed's `runs` / `conflicts` and updated in place when the feed moves.

**Round 3 part 4d1 built (2026-10-19): an overlap parks both.** 4d is cut in four: 4d1 park on
conflict (the server side of the map) · 4d2 the DMX map element zoomed out (Universes › List |
Map: usage bars, blocks by type, free runs, the conflict list) · 4d3 zoomed in (the slot grid,
Edit Patch) · 4d4 drag and drop + lasso in both zooms. Identify / Probe / Find wait for their
words (lum.md first). In 4d1: one door for who sits on which slot, `lumacore_tree::dmx_slots::
SlotMap` (every run — a fixture's breaks and its multipatch copies' —, each overlap once, each
run past 512, the parked fixtures, the slots in use per universe, a shared slot once); the render
plan's channel tier skips a parked fixture's channels (it keeps its address, values and
programming; resolved, it sends again at the next patch generation); the `patch` feed reads the
same map: a row's `parked`, a universe's `used`, `runs` (the map's blocks: fixture, copy, break,
universe, start, count, type, parked) and `conflicts` (overlap / past, both ranges, one-tap
fixes as lines — the later run to the next free address `Assign … At DMX`, `Swap` for two
fixtures, `Unassign … At DMX`; a copy has no Unassign in lum.md, so its fix is `Delete Fixture n
Multipatch m`).

**Round 3 part 4c2 built (2026-10-19): the Insert fixtures dialog** (`kit/insert.js`, as in
`insert-fixtures.html` v6). The Patch tab's function block has **Insert fixtures…** (an Action of
value kind `insert`, `claim`: greyed in Live) and the slab's **New** opens the same dialog. 1
Choose: the No list (a `<luma-grid>` of the show's types: No, Name, Manufacturer, Revision, Modes
with footprints, Used) or Manufacturer · Type · Revision columns; Show / Library / Both; search
over names, manufacturers, revisions and modes; Used only; the Mode list; the channel map (a
read-only grid: DMX offset from 1, attribute, short name, geometry, feature group; sorted by
Structure, Channels or Feature group); Details (the file's facts); **Upload** from this device
(Setup and up; the uploaded file is picked at once). 2 Place: Name, Quantity, First FID and
Address with free chips (the value dialog's `freeNumbers` / `freeAddresses`, now exported),
Every, MVR group (a pick list of the tree's nodes); where the block lands, overlaps or passes
512 is said before Create; Create runs `Store Fixture a Thru b At DMX u.aaa /FixtureType <no |
"file.gdtf"> /Mode n [/Revision n] [/Every n]; Label Fixture … "name"; Assign Fixture … At
MvrGroup n` (each its own Oops step). `<luma-grid>` takes `bar: false` (a grid that picks one
row has no selection bar). On a phone the area's state bar shows only its title. Not yet: the
DMX sheet in Place (with 4d), Class and IDType · CID fields, the thumbnail.

**Round 3 part 4c1 built (2026-10-19): Insert fixtures' data.** 4c is cut in three: 4c1 the
data · 4c2 Choose (the No list, Manufacturer · Type · Mode, the channel map, Details, Upload) ·
4c3 Place (the form and the DMX sheet — the DMX map element of 4d, zoomed in; so 4d comes
before 4c3). In 4c1: `commands::gdtf_library` reads `library/gdtf/` through the disk door
(each file's fixture type and its own facts, `lumacore_gdtf::file_details`: names, description,
FixtureTypeID, GDTF version, revisions with text / date / uploader; cached by name, size and
time); `lumacore_views::insert` builds `GET /api/fixture-types` (the show's types — the holder's
draft's — with revisions, the newest revision's modes and footprints, how many fixtures use it;
the library's files, `in_show` when a show revision is the same file by SHA-256, or the file's
error) and `GET /api/fixture-type?no|file&revision&mode` (one mode's channel map: break, offset,
bytes, attribute, its pretty name and feature group, geometry, virtual; and the details);
`POST /api/gdtf?name=` is Upload from this device (Setup and up, refused unless the importer
reads it, ≤ 16 MB). Not yet: the thumbnail (named in the details, not served), a member's upload
reaching the master's library (an Import runs on the master).

**Round 3 part 4b built (2026-10-19): the function block, and the address is live.** The Patch
tab's function block: **Patch to…** (the value dialog's address kind), **Next free address**
(`Assign Fixture … At DMX`), **Info**; an `Action` declares `claim` (greyed in a claim area's Live
with the reason — for Insert fixtures… in 4c) and `value` (the value dialog its `{value}`
opens). **Found on the way:** `Assign` / `Unassign Fixture … At DMX` asked for the Devices claim,
though the address is a live column (round 3, 18: "Patch to… and Next free address work in
Live"). They now need none; under someone else's draft they run against the live devices and the
draft follows (`live_draft::runs_live`), like a live `Set`. A place in the MVR tree still needs
the claim. The value dialog's Esc is caught on the window, before the setup frame's own Esc.
Not built from the mockup's block: **Multipatch…** (a selection whose fixtures hold different
numbers of copies has no one "next copy" word yet — a word question), **Sort** and **Offset**
(no decided meaning yet), **Clone…** (part 6), **Insert fixtures…** (4c).

**Round 3 part 4a built (2026-10-19): the Live / Draft grid** (`patch-grid.html`). Part 4 is cut
in four: 4a the grid · 4b the function block · 4c Insert fixtures (+ the GDTF picker, Upload) ·
4d the DMX map element. In 4a: a column declares `draft` (`menu_defs::draft`: the FID, Fixture
type, Mode, MVR group) — Live (or someone else's draft) locks it like a non-live property, cells
and header, with the one reason; the patch area's `Area` row now carries its state bars (Live
green, your draft gold, another's violet — `{holder}` named), its tree's add lines (`+ Layer` /
`+ Group`, greyed in Live) and its count unit ("12 fixtures"); `<luma-tree>` draws a guide line
for every level (`guides`); the title shows the path (`Root.ShowData.Devices.Dmx.Patch`, from
the feed's typed `path`) and the picked node. The `patch` feed's rows: `address_tone` `alert`
where a slot is shared or the run passes 512 (both fixtures red), `warn` with "no universe";
the holder's rows list `changed` cells against the live rig (a gold dot). The Note column is
back. Not yet: the violet dot (a live edit that reached your draft: the rebase leaves no trace
yet), editing MVR group / Fixture type / Mode cells in the draft (part 5 and the tree's Move),
the park of overlapping fixtures (4d).

**Round 3 part 3c built (2026-10-19): the value dialog.** One kit element (`kit/value.js`,
`openValue`) as approved in `renumber.html` v2: beside the selection (below it, else above;
draggable by its header; a bottom sheet on a phone), the field with − / +, one hint line, up to
four free values, the value's toggles, Apply; Enter / Esc / ↑ ↓. The registry names the kind an
item opens (`ContextItem::value`, the common Renumber… = `number`, Fixture's Patch to… =
`address`); the kit holds the two kinds: **Renumber** (`Move <obj> … At n`, Skip taken = `/Skip`
by default, Swap = Move's own swap; Selection order = the order rows were picked, `+`-joined when
not ascending; Grid order = as the list shows them) and **Address** (`Assign Fixture … At DMX
u.aaa`, Bunch up by default — DMX packs — or `/KeepSpacing`; an overlap is red and named, past 512
warns). Every keystroke previews old → new in the grid (`<luma-grid>.setPreview`: the gutter shows
both, a narrow cell the new value with old → new on hover; rows in the way red); nothing is sent
until Apply. A pool's Renumber… opens it too (no preview there yet). The patch feed's fixture
rows carry `footprint`. Not yet: the free values come from the list's own rows in the kit, not
from a free-value index in the suggestion engine (which would also suggest them after `At`) —
on the roadmap.

**Round 3 part 3b built (2026-10-19): the one context menu.** `GET /api/object-menus`
(`lumacore_views::context`, open and static like the menu types) gives every object's menu from
the registry: the common actions its verbs take (Edit… · Renumber… · Label… — every object with a
"Name", through the Label command — · Copy · Move… · Delete, each marked `claim` when it changes
more than live columns), Set with every declared property (its FieldInput, level, live), and its
own lines from a new catalog row, `ContextDef` (`ContextItem`: label, icon, verb, line, live) that
the object's crate adds (`r.rows(..)`): Fixture — Patch to…, Next free address, Swap addresses…,
Unpatch, Info; Universe — Info; Group — Select, Edit members, Info; others get Info. The kit's
`context.js` draws it on the selection (greyed items keep their place with the reason under them;
a phone gets a bottom sheet; ↑ ↓ / Esc): `<luma-grid>` opens it on a right-click, a finger held
still and let go (hold-then-drag still lassos), the selection chip's new ⋯ and the Menu key
(Shift+F10 is Meow+F10, a core key); the grid's Set part lists its own columns with their labels
and values. A pool's hold opens it too — **the object sheet (`kit/sheet.js`) is gone**; Move /
Copy pick their target by a tap, Edit members opens the group editor, Edit… is EditSetting. Not
yet: Renumber… opens a plain number ask until the value dialog (3c); Fixture type / Mode /
Revision join Set with part 5; Identify with the DMX map (part 4).

**Round 3 part 3a built (2026-10-19): the claim button, the ring, slab states.** The claim
button is the slab's last cell, bottom-right on a phone too (`power.claim`); Live shows the open
lock (`lock-open`, added to the sprite) and a hold of 0.6 s fills gold left → right and drains
back on an early release (`<luma-power>` `hold_ms`, `pointer.hold`'s `onArm` / `onDisarm`; no
fill with reduced motion). The hold ring is drawn above every dialog, sheet and toast. In a claim
area's Live (or another's draft) New, Cut, Delete, Import and Paste are greyed with the one reason
(`areaWhy`: "Claim the draft to change it" / "Bob holds the draft"), the same the locked columns
give; a word the object lacks keeps its own reason (no `Copy Fixture` yet: Duplicate's word is
still to come). Greyed slab cells are dashed. Paste counts the station's clipboard: each user's
clipboard rides in the station view (`clipboards`, a new `clipboard` revision) and the `patch`
feed carries yours. Not yet: the coloured state bar and the locked headers (part 4).

**Round 3 part 2 built (2026-10-19), the server side: `Import FixtureType "file.gdtf"` and the
library file index.** `Import FixtureType` (the Devices claim; reach Drive) reads library/gdtf/
(sub-folders too); a newer revision of a type in the show changes no fixture and the reply offers
`Set Fixture … "Revision" n` for the ones using it; the bare `Import "x"` / `Import library` /
`Import GDTF` are refused, naming the kinds. `Store Fixture … /FixtureType "x.gdtf"` imports in
the same step (1a). The index: `commands::import::gdtf_files` (through the disk door) feeds
`/FixtureType <Tab>`: the show's types first (number, name), then the library's files. The picker
and its Upload (an HTTP route into library/gdtf/) come with the Insert fixtures window (part 4);
the revision change itself is part 5.

**Round 3 part 1c built (2026-10-19): the MVR node delete answers, Multipatch, `Unassign … At
Tag`.** `Delete MvrGroup / MvrLayer` asks Keep the fixtures (the default, `/KeepChildren`: a
group's nested groups and fixtures move to its parent; a layer's fixtures to No MVR group, its
groups with it) · Delete the fixtures too (`/WithChildren`) · Cancel; a node holding nothing asks
the plain Delete. **Multipatch copies live inside their fixture** (decided 2026-10-19, one
question: `Fixture.Luma.multipatch`, each its own GUID, name, address and place; MVR import /
export will map each to MVR's separate fixture with `multipatch` set): `Store Fixture 201 Thru 206
Multipatch 1 [Thru n] [At DMX …]` (the next numbers only), `Assign Fixture 201 Multipatch 1 At DMX
…`, `Delete Fixture 201 Multipatch 1` (the later ones move down), `Info Fixture 201 Multipatch`;
selecting a copy selects its fixture; the render plan mirrors the fixture's channels on each
copy's address, and a copy's slots are taken. `Unassign Fixture … At Tag [path]` takes that tag
off (every tag bare), no claim (a live column); the Tag object is known by its tree node. Not yet:
naming a copy (`Set Fixture 201 Multipatch 1 "Name"`), a copy's own place, its row in the grid.

**Round 3 part 1b built (2026-10-19): `Fixture n DMX`, `/Swap`, Move `/Skip` / `/By`, the
second id.** `DMX` is `Fixture`'s child: `At Fixture 301 Thru 304 DMX` places each at its partner's
address exactly (an overlap named), `… /Swap` trades every break (or break n with `DMX n`) pair by
pair up to the shorter list, `Info Fixture 3 DMX [n]` lists its addresses, `Unassign Fixture 1 At
Fixture 1 DMX 2` takes one break. Move on every pool: `/Skip` (consecutive, over numbers rows that
aren't moving hold) and `/By <column>` (consecutive in the column's order; the pool's declared
orders) — on the Move row, so every pool's Move takes them. **The id kind is grammar now** (lum.md
`segment = OBJECT [ id-kind ] [ ids ]`, the parser, printer, cursor, golden `kinds:`): an object
declares `LumShape::id_kinds`; Fixture's are MVR's ID types but None and Fixture; `Fixture
Houselights 1 Thru 4 + Fixture 101` resolves by second id through the one resolver (a range walks
the second ids that exist). **A fixture may hold several second ids** (the user, 2026-10-19 — the
simple form now, hardening on the roadmap): MVR's IDType + CID stays the first, more live in
`Fixture.Luma.more_ids` (the `"MoreIds"` property, `"Channel 12, Fog 3"`, Setup, in the draft;
empty isn't saved, so no format change). Not yet: `Park Fixture 3 DMX` (Park's old handler),
`Fixture Fixture 4`'s named refusal. Found on the way (the sweep, its seed shifted by the new
words): a preset-type word called bare (`All 999 - 1`) selected and was declared read-only — it is
a Selection step now; what it should select belongs to the presets work.

**Round 3 part 1a built (2026-10-19): Store / Assign / Unassign `At DMX`, the DMX object.**
`Store Fixture <numbers> [At DMX …] /FixtureType <slot | "name" | "x.gdtf"> [/Mode] [/Revision]
[/Every n]` (`objects/patch/fixture/store.rs`: every number of a range, a taken one refused, a
file found by SHA-256 or imported in the same step); `Assign Fixture … At DMX [u[.aaa]] [/Every n |
/KeepSpacing]`; `Unassign Fixture … At DMX [u[.aaa]] | MvrGroup [n] | MvrLayer [n]` (a fixture
alone refused, a bare kind = whichever) — one placement rule (`fixture/address.rs`: exact = exact
with the overlap named E-PATCH-001, a run skips and names what it skipped, packed unless `/Every`
or `/KeepSpacing`, one universe or nothing, a missing universe created and said, a virtual mode
placed without occupying). `DMX` is the kernel's object (`objects/patch/dmx.rs`: Info by default,
`/Input`, `At` planned — a planned `At` now routes as the At verb); the `Patch` / `Unpatch` verbs,
the old DMX verb and its `DMX In` went; the demo builder stores its fixtures at 101–106 (no stale
"Fixture 1" names: the demo show was regenerated, the stray working copy kept in
`.autosave/demo_user-save-before-round3-1a_20261019.luma`). One `/FixtureType` (it takes a value
now): Attribute's board-by-type switch is `/By FixtureType`. `Unassign … At Tag` waits for 1c.

**Round 3 step 0 built (2026-10-18): the missing tests, and the Live lock on the server.** With no
Devices draft open, `Set` on a patch object's own data that isn't a live column is refused
(E-CMD-010, "changes in the Devices draft: Claim Root.ShowData.Devices first"); it was written
straight to the rig. Not patch data, so still live: the meta level (Name, Note, Paint, Icon,
Locked, Hidden, Tags) and another owner's scoped rows (a group's grid cell). A universe's Name and
Note are live columns like a fixture's; its MergeMode waits for the draft (issue 11). The 5 s
lost-holder grace is `claims::HolderWatch`.

Open in this round: slab states in Draft (New, Import, Paste); deleting an MVR node deletes its
fixtures but the question says "unpatched"; the small layout differences (FID label, X/Y/Z, title,
echo counts, tree bar, guide lines); the dropped or contradicting items (Fixture.Luma extras, Clean
up unused, labelling MvrClass / MvrPosition, the other live columns); the missing tests the audit
listed. Later, not this round: browsing GDTF Share (needs internet and a login; isolated rigs).
Mockups (round 3) next: Insert fixtures, the GDTF picker, the attribute map, the DMX map / Edit
Patch, the grid in Live / Draft with the claim button.

**Build times hardened (2026-10-18).** Asked after a commit alone rebuilt 23 crates in release
(2 min 39 s, measured: only the branch ref touched): `lumacore-luma`'s `build.rs` stamps
`LUMACORE_GIT_SHA` and re-runs on `.git/HEAD` / `refs`, so everything above it recompiles. Real
projects stamp at the top of the graph or outside compilation (ripgrep's and rust-analyzer's
binary crates, Go's `-ldflags -X`, Chromium's one `LASTCHANGE` target, Zed's CI-only stamp,
`vergen` / `built` in the top crate). Asked one question at a time:
1. **The stamp lives in the station.** `apps/lumacore-station`'s `build.rs` stamps the commit
   (`-dirty` kept); `install_catalog` hands it to `lumacore-luma` once (set at startup, read by
   build info, `Version` and every saved show's `saved_with_commit`); a test or tool that never
   goes through the station reads `unknown`. A commit then recompiles only the station (the
   catalog and the mains) and relinks. Declined: a tiny build-info crate (helps only if just the
   station depends on it — this with an extra crate), stamping only shipped builds (local saved
   shows would say `dev`).
2. **A `local` profile for the scripts:** `[profile.local]` inherits release (the render tick keeps
   its speed) with `incremental = true` and `codegen-units = 256`, so changing one crate
   recompiles mostly what changed; `./scripts/unix/server start` and the app's build use it, a
   flag (`--ship`) builds full release. Measured before / after on one crate change and one
   commit. Declined: dev by default (our own crates unoptimised on the render thread), release as
   it is.
3. **Jobs: all cores minus 2** (`jobs = -2` in `.cargo/config.toml`, cargo's logical CPUs − 2;
   the baselines are a MacBook M3 and a Threadripper 1950X). Replaces §31's `jobs = 2`.
4. **Every build ends with its stamp** (asked after: "a compile time stamp when lumacore-script
   has ran"): one end line — when it finished, how long, how many crates (`✔ Built
   lumacore-server · finished 14:32:05 · 1m 12s · 3 crates`), in the script's log too. Declined:
   a build history and a launcher status row.
5. **A fresh benchmark build:** `./scripts/unix/build bench [profile]` (and its launcher entry)
   asks first (it wipes that profile's target dir), runs `cargo clean --profile <p>` and a timed
   full build of server, CLI and app — every crate, dependencies too — and prints and appends one
   row to `docs/dev/build-times.md` (committed): date, machine (CPU, cores, RAM, OS), rustc,
   profile, jobs, crates, time; the M3 and the 1950X compared side by side over time. Declined:
   printed only, a separate target dir.
The crate graph is the third lever (a change in `lumacore-objects` always recompiles the cmd
crates above it); 34g's split shrinks it.

**Build times built (2026-10-18).** The stamp: `apps/lumacore-station/build.rs` (moved from
`lumacore-luma`) → `lumacore_station::stamp()` (called by `install_catalog`, and first in the
server's `main`, whose `--version` reads the build id before the catalog) →
`lumacore_luma::set_git_commit` (a `OnceLock`; `build_info()` reads `unknown` without it).
`[profile.local]` in the workspace `Cargo.toml`; `lib.sh` `lc_profile` builds it unless `--debug`
(dev) or `--ship` (release, `SHIP=1`; the smoke test ships), the scripts pass `--ship` on, the
Windows scripts take `-Ship`, the launcher's default profile is `local`. `jobs = -2`. `cargo_build`
ends with `✔ Built <bin> · finished <date time> · <duration> · <n> crates · <profile>`.
`./scripts/unix/build bench [local|release|dev] [--yes]` (and the launcher's "build bench") cleans
the profile and times server + CLI + app, appending a row to `docs/dev/build-times.md`.
Measured on the MacBook M3 Pro (12 threads), server only, unchanged code:

| | before (release, 2 jobs) | after (`local`, 10 jobs) |
|---|---|---|
| a commit alone | 23 crates, 159 s | 1 crate (the station), 1 s |
| `lumacore-objects` touched / a comment added | 21 crates, 126 s | 21 crates, 4–5 s |
| fresh build of everything (server + CLI + app) | — | 380 crates, 2 m 16 s |

A real code change in the kernel costs more than a comment (its codegen), still incrementally.

**Import MVR hardened (resumed 2026-10-20 after §38, one question at a time, complete the same day;
34g4 part 4; mockups `docs/dev/ui-kit/mockups/2026-10-19-mvr-import/`).** Points 10–12 above stay;
what the drives round changed is folded in here. Built as 38c (the element's phone rule, the File
step) and part 4 proper (`lumacore-mvr`, the preset, the review, Apply, the asks) at the go.
1. **The dialog's shape** (`import-options.html`; the user: "A is good"): the kit's two-step dialog,
   like Insert fixtures — **1 File** is the files element on `library/mvr/` (§38 5: the drives
   rail, the MVR columns scene · layers · fixtures · GDTFs embedded or missing · size · modified,
   This device, a drop), **2 Options** is the import preset form (scope, match key, FID / universe
   offsets, rig offset and rotation, the None / Take / Offer matrix, Lost, Rotation; Reset to Offer
   all); the footer shows the line as it grows; **Review** runs it, the dialog closes and the
   Patch grid is the Import review (Import ticked · n, Cancel import). A bare `Import Mvr` typed,
   and the TUI, get the station's File ask (38b-4) and run with the preset. Not chosen: B the
   station's one ask with the options as its fields (six selector rows, no file facts beside them),
   C the options as chips on the review (the decided order turned around, a re-read per chip).
1b. **The files element on a phone** (`files-phone.html`; asked on 1's reservation "the phone
   version looks cramped" — it is, wherever the element is used): in portrait the drives rail
   **folds into a chip strip** over a full-width grid (id + label, the picked one gold, Out ones
   dimmed, a new stick appends; This device at the strip's end), the crumbs are one scrolling line,
   the grid folds its columns the kit way (the facts under the name), the drop zone is one line,
   the Name row stays; landscape keeps the desk layout with a narrow rail. One rule for Backup ›
   Library, the File ask, Insert fixtures' Library and Import MVR's File step; built with 38c.
   Not chosen: B the drive as a crumbs button with a sheet (a stick put in is not seen), C a
   narrow icon rail (ids alone say little, the grid still folds to one column).
   **1b built (2026-10-20, 38c-1)**: `files.js` sets `data-phone="up"` from `phoneMode()` (and
   follows `onPhoneMode`); the rail renders chips (id + label, the picked one on, Out dimmed, This
   device last) and `kit.css` lays the element out as a column; the grid folds its own columns.
   Checked on the phone project in `kit-drives.spec.ts` (the strip one row, the grid the width).
   **The File step's format built (2026-10-20, 38c-2)**: `crates/formats/lumacore-mvr` (no lumacore
   dependency; quick-xml at the gdtf crate's version): `read` / `read_from` walk the zip and read
   `GeneralSceneDescription.xml` into data — layers, nested groups, every fixture with its parent
   and layer, matrix, GDTF spec and mode, addresses (absolute `513` = 2.001 or `universe.channel`),
   ids, classes, positions, other objects counted —, `head` / `head_from` the file list's facts
   from the scene entry alone (a 200 MB file is never read whole). `lumacore_objects::commands::
   mvr_library::heads_in(dir)` caches heads by path, size and time; the `files` feed's `mvr` rows
   carry scene (provider + version), layers, groups, fixtures, the GDTFs embedded / missing and
   `gdtf_text`; the element's `mvr` columns show them. `tests/fixtures/rig.mvr` (2 layers, 2 groups,
   6 fixtures, AuraXB embedded, one GDTF missing) drives the e2e check. The dialog itself (1, the
   two steps) comes with part 4's words.
2. **The line** (`import-line.html`, a lum.md discussion; the user: A). No grammar change: one
   value per option. **The preset is the profile's**: every setting of the Options step is a
   declared `UserProfile` property, section "MVR import" (`MvrMatch` Auto · UUID · FID · FixtureName
   — `Name` alone is the property word —,
   `MvrLost` Mark · Delete, `MvrRotation` Take · Ignore, `MvrAddresses` / `MvrIds` / `MvrNames` /
   `MvrPosition` / `MvrRevisions` / `MvrNewFixtures` None · Take · Offer (default Offer, New
   fixtures Take), `MvrFidOffset`, `MvrUniverseOffset`, `MvrX` / `MvrY` / `MvrZ` (mm, MVR's unit — whole numbers,
   the one property registry has no decimals), `MvrRotate`
   (°)); the Options step is a settings form, each change one `Set UserProfile` line (no OK), shown
   in Users & Profiles. **`Import Mvr "file" [/Drive "usb1"]`** runs with the preset; its options
   override for this import: `/Scope <target>` (an MvrLayer, MvrGroup, MvrClass or Tag; absent =
   the whole patch — a target as a value, grammar today), `/Match`, `/Lost`, `/IgnoreRotation`,
   `/FidOffset n`, `/UniverseOffset n`, `/X` `/Y` `/Z` `/Rotate`, and the six groups as options
   with the three-word choice (`/Addresses Take`, `/Ids`, `/Names`, `/Position`, `/Revisions`,
   `/NewFixtures`). The dialog builds and runs the line; a macro or the TUI types it. lum.md
   "### MVR import" (PENDING until part 4). Not chosen: B `/Take` and `/Offer` with `+` lists and a
   vector `/Offset` (a grammar change whose edge case — `+` after an option's value is a list today
   an error, in a target the set-op — and a second new value rule aren't worth shorter lines), C a
   named `MvrPreset` pool object (a new object for what a reference console keeps as one dialog memory; the
   profile's preferences are the preset the decision names).
3. **The review is the draft's** (`import-review.html`; the user: A). `Import Mvr "file" …` reads
   the file once (a big one on the background worker, the reply when it is read) and keeps the
   review on the holder's `PatchDraft` — the file's name, the settings used, one row per file
   fixture with its status (Same · Changed · New · Lost · Outside · Clash), its match key, the
   file's value per field and the group's None / Take / Offer; nothing is written. The `patch`
   feed carries it as the grid's third mode, `review`, for the holder (a second screen, a reload
   and the TUI see the same). **The grid's selection is the ticks**: Take rows pre-selected, a
   Clash row refuses a tick, the + / − / All / None / Invert cells and the lasso work. The review's
   slab: Take cell · Offer cell · Bring into scope · **Apply ticked · n** · Cancel import. The
   words, the rows as the target like every line (the mockup's `/Apply 101 + 102` reads
   `Import Mvr 101 + 102 /Apply`): `Import Mvr {rows} /Apply` writes the ticked rows into the draft
   (the holder's own edit of a cell wins, as decided) and closes the review; `Import Mvr /Apply`
   every taken row; `Import Mvr /Cancel` drops it; `Import Mvr 102 /Take "Address"` /
   `/Offer "Address"` one cell; `List Mvr` the review as a table and `Info Mvr` its summary (never
   `List Import`: an object is never also a verb, §15). Keep is refused while a review is open
   (Apply or Cancel first); Discard and a lost holder drop it with the draft; a second file asks
   before replacing the review. Not chosen: B the page's review (the line's reply, a second line
   with `/Only` — no TUI, no second screen, two reads), C written into the draft at once as marks
   (the way of 2026-10-17, replaced then: it breaks "nothing is written while you review").
4. **The placement asks** (`import-placement.html`; the user: A). At `Import Mvr … /Apply`, for a
   ticked New row whose MVR group (or its chain) the show no longer has, the station asks **one
   ask per missing group**, in the file's order, the suggestion first: *Recreate the group* (stored
   again under its parent, the MVR UUID kept so the next import finds it; a missing chain
   recreated up to the layer) · *Into the parent* (the nearest ancestor the show has, the layer at
   worst) · *No group*; a toggle "the same for the other missing groups" makes the answer cover
   the rest. The answers are lines: `… /Apply /Missing Recreate | Parent | None`, with
   `/Group "name"` naming the one group when the toggle is off (the next group asks on); the asks
   chain (`Ask::then_run`) and the Apply writes once at the end; Cancel keeps the review. The
   preset's seventh row, `MvrMissing` Ask (default) · Recreate · Parent · None, silences the asks
   when set (a macro, a rig that always recreates). Not chosen: B never ask (a deleted group comes
   back silently — the one case point 12 wanted asked), C one ask for all missing groups (different
   fates need separate Applies). Built as `/Gone` (part 4b): `/Missing` is `List Asset`'s valueless
   option and an option name means one thing across the registry, so the answer's option is `/Gone`
   (the ask's own word); the preset stays `MvrMissing`.
   **Part 4a built (2026-10-20)**: the preset on `UserPreferences.mvr` (`lumacore_tree::MvrImportPreset`;
   16 `UserProfile` properties, section "MVR import", `media::PROFILE_PROPS`); the review types
   on the state (`lumacore_state::mvr_review`: `ImportReview` with `ReviewRow` / `ReviewCell`,
   statuses Same · Changed · New · Lost · Outside · Clash, `PatchDraft.review`); the `Mvr` object
   (`objects/patch/mvr`: the 22 options declared on its row, typed `Import` / `List` / `Info`),
   `Import Mvr "file" [/Drive] [/Scope] …` reads the file through the disk door (`lumacore_mvr::
   read_from`, only the scene entry; the embedded GDTFs hashed for the type match), resolves the
   scope with the one resolver (an MvrLayer / MvrGroup / MvrClass by uuid, a Tag by its fixtures)
   and builds the review (`review::build`: UUID → FID → name or one key, the cells per group with
   the preset's None / Take / Offer, position within 1 mm and rotation within 0.5°, a clash only
   where the address would change or a new number is taken, Lost = draft fixtures in the MVR tree
   the file lacks); a bare `Import Mvr` asks with the files element; `/Cancel` drops the review;
   `List Mvr` / `Info Mvr`; the `patch` feed carries `review` and `mode: review` for the holder
   (`PublishedDraft` gained it). `/Apply`, a cell's `/Take` / `/Offer` and the placement asks
   reply E-CMD-003 until part 4b; the grid's review mode and the two-step dialog are part 4c.
   Every MVR line of lum.md parses with the registries now (PENDING shrank by 15).
   **Part 4b built (2026-10-20)**: `Import Mvr [rows] /Apply` (`objects/patch/mvr/apply.rs`) writes
   the ticked rows — or the rows named, as every line names its target — into the draft: a
   Changed row's taken cells (name, address with the universe made when the draft lacks it,
   position and rotation, unit and custom id, the type by the embedded GDTF through the one door
   `import::insert_gdtf_bytes`, which `Import FixtureType` now uses too), a New row made at its
   number with its uuid, group, matrix and ids (a GDTF neither in the zip nor in the show skips
   the row, named in the reply), a Lost row deleted when the preset says Delete; the review is
   closed and the draft published. A Clash row refuses the Apply. **The placement asks** (decision
   4): a ticked New row whose group is gone asks per group — Recreate the group (the chain up to
   the layer, uuids kept) · Into the parent (the layer) · No group — with a toggle "Only this
   group" that adds `/Group "name"` (the kit's toggle adds text when on, so the decided "the same
   for the others" is its inverse, off by default = all); the answers ride on the review
   (`ImportReview.answers`) until every missing group has one, then the Apply writes once; the
   preset `MvrMissing` other than Ask answers without a dialog; a layer the draft lacks is made,
   never asked. `Import Mvr 102 /Take "Address"` / `/Offer` tick one cell (Name, Address,
   Position, Rotation, Type, Unit, CustomId). **An open review counts as a draft change**
   (`patch_draft_changed`): leaving asks, Keep is refused (E-CMD-010, Apply or Cancel first),
   Discard and a lost holder drop it.
   **Part 4c built (2026-10-20)**: the page. `Action.when` ("" | `review`): the Patch tab's own
   cells Take cell · Offer cell · Apply ticked · Cancel import show only in the grid's review mode
   (`menu_defs::reviewing`), the others hide then, the standard cells grey ("Apply or cancel the
   import first"); an own action of value `import` (Import MVR…) marks the slab's **Import cell**
   as the page's — its line `Import Mvr` is the fallback (the File ask), the page opens the
   two-step dialog `kit/mvrimport.js`: 1 File = `<luma-files kind="mvr">` (a pick fills the line,
   a double-tap goes on), 2 Options = `<luma-settings embedded>` with the profile's "MVR import"
   section (`EditSetting UserProfile`'s fields reach it through the page's `lc-fields` event, the
   one `edit_settings` path; every change one `Set UserProfile` line; Reset to Offer all) and the
   Scope picked from the draft's MVR tree or classes; the footer shows the line, Review runs it
   and closes the dialog. `<luma-setup>` draws the review when the `patch` feed says `mode:
   review`: the file's rows with Status (a tone dot), Name, Type · Mode, Address, Position,
   Rotation, MVR group, Layer, Clash — a cell reads `take · <file value> (was <live>)` (the grid
   has one text per cell; the decided two-line cell is a kit refinement to come), Take rows
   pre-selected once per review, the selection the ticks (`{rows}`), Take / Offer cell pick the
   field. Bring into scope waits for its word. The `files` element's MVR columns (38c-2) show the
   file's facts in step 1.

## 35. One object door: Store / Delete / Copy / Move / Paste (2026-10-15)

Asked before the rest of §33 3d (2026-10-15): every object still writes its own insert / remove /
rename into its own `BTreeMap`, the next free number is copied 22 times (14 as "after the last"),
and each verb's occupied-slot rule differs by object. Research (established consoles / Eos / Hog / MagicQ,
PowerShell providers, Unix VFS `rename(2)`, Kubernetes `genericregistry.Store` + strategies,
Blender / Godot / Unreal duplicate, nushell): the systems that scaled keep the verb generic and let
the type do the work, check each type's rules once at registration, mint ids only in the generic
layer, never turn a move into copy + delete, and make the occupied destination an explicit rule.

1. **Two layers, one implementation.** A typed **pool door** does every write on every collection
   (`BTreeMap<u32, T>` and the cue key) — Kubernetes' generic store with per-type rules. A thin
   **path front** (`tree::copy` / `tree::move`, `rename(2)`-like) resolves both ends of an
   owner-to-owner line or a tree-shell `mv 3 7` / `cp` to pools and calls the same door; a node with
   no pool refuses. (Alternatives — only the typed door; console / PowerShell-style untyped operations on
   any node with run-time hooks and a deny list — declined.)
2. **Per-pool rules are declared and compile-checked** (`PoolRules` on the object's `TreeDef`):
   `can_delete` (the locked Default profile, E-USER-002), `after_change` (a Fixture bumps
   `patch_generation`), `capture` (what Store records), optional `merge`; its key type (`PoolKey`:
   `u32`, the cue's parts). A pool without its required rules doesn't compile; a verb a pool can't
   take isn't published (no completion, no help), never a run-time "not supported".
3. **Identity lives only in the door**: `insert` and `duplicate` are the only places a GUID is
   minted. Copy / Paste of a copy = new GUIDs for everything `Walk` marks own (pool members, nested
   cues), links kept; Move, Swap, Label and Overwrite keep the GUID.
4. **Move onto a taken slot swaps** (`RENAME_EXCHANGE`; not a shift): `Move Group 3
   At 7` with 7 taken trades the two; a range swaps with a range of the same size, each destination
   on its own (free → moved, taken → swapped). Today's refusal (E-CMD-010) and the pool drag's (§22)
   change to this. (Alternatives — shift to the first hole, refuse, ask each time — declined;
   The shift stays available as Insert, 11.)
5. **Store / Copy / Paste onto an existing object ask once, from the door; flags answer it**:
   Merge (only where the pool declares `merge`: groups, cues, presets) / Overwrite / Remove (Store
   only) / Cancel, plus Create Second for cues (§30 "Store Cue" keeps its buttons);
   `/Merge` `/Overwrite` `/Remove` skip it. A stored key or macro line can't ask (as Delete): without
   a flag it is refused, naming the flags. Today's group refusals and media's `/Overwrite`-only become
   this ask. Overwrite keeps the existing object's GUID, so links to it hold.
6. **Between owners of the same kind** (`Move DataPool 1 Group 3 At DataPool 2 Group 8`, `Copy
   UserProfile "Bob" View 2 At UserProfile "Ann" View 5`): allowed; a Move keeps the GUID (the same
   object, never copy + delete), Swap and the ask (5) apply there too. Another kind (`Move Group 3 At
   Sequence 4`) is refused with a coded error (Unix's EXDEV).
7. **Delete of something in use: the one Delete ask names its users** ("Delete Group 3? Used by 2
   cues in Sequence 1, Executor 1.1"), read by the door's `users_of` from the GUID index
   (`ShowData::identity()`); after a Yes the links are kept broken and listed (`core::references`),
   Oops brings everything back. A pool may later declare one more answer (presets: "Delete, keep
   values in cues").
8. **Next free is the key type's own**: the first unused number for every pool (decision §34 13,
   sequences included — Groups 1, 2, 4 → 3), but **a cue appends** — `Store Cue` in a sequence with
   cues 1, 2, 4, 10 makes 11 (Eos, Hog and others: cues are a timeline). `next_free_block(len)` finds the
   first block a range fits, its spacing kept.
9. **A bare Copy and `Cut` fill a station clipboard; `Paste` finishes them** (revised by 14: the
   cut is its own word, not a bare Move). `Copy Group 1 Thru 5` remembers their GUIDs as *copy*,
   `Cut Group 3` as *cut*; `Paste Group 10` (or `Paste DataPool 2 Group 10`) duplicates with new GUIDs
   or moves with kept ones, keeping the spacing, with 4 – 6 applying. `Copy|Move … At …` stay the one-line forms. The grid slab's Copy / Cut / Paste (§30
   "12d", today a client clipboard) run these lines; the client clipboard goes.
10. **The clipboard holds live GUIDs** (with copy / cut and the pool) on `Runtime.Users.{User}`:
    session runtime — every station of the user shares it, never saved, a show load clears it. Paste
    takes the objects as they are then; a deleted one is skipped and named. (Alternative — a snapshot
    of the data at Copy — declined.)
11. **Whole-pool operations in the door**: renumber (closing holes, or in an order — by name, by a
    property such as a fixture's DMX address; GUIDs kept), move with shift, and finding by a
    name pattern. Each is one Oops step and gets a slab action. Their words are 15 – 18.
12. **The door's functions** (`lumacore-state`, `pools`): `next_free` / `next_free_block`, `spacing`
    (destinations from `At`, today's `move_cmd::plan`), `insert`, `take`, `put`, `rename`, `duplicate`,
    `relocate` (swap on taken), `users_of`, the coded `not_found` / `exists` replies and the one
    existing-object ask; with §34 12's other doors (`in_datapool`, `label_ask` / `delete_ask`, …). The
    rest of §33 3d moves each area's Store / Delete / Copy / Move / Label onto it — one pass per handler
    (§34 14).

**The Lum round (2026-10-15, lum.md written the same day; the words wait in `PENDING` until built).**
A reference console's manual was read for each word: it has `Cut`, `Paste` (the destination
as target; *"the pasted object will have all the edits made to the source after copying"* — 10),
`Insert` (move with shift) and the quoted `*` wildcard; it has no compact or pool sort.

13. **`Paste <target>`: the destination is the target** (as established: `Paste Group 5`), resolved as a slot
    like Store's; owner first (`Paste DataPool 2 Group 10`). `Paste` has no default object of its
    own: `Paste 10` follows *your* default object (`Group>`, §33 21, 39) like `3 At 50`, written out
    before it runs; a default object of another kind than the clipboard's is refused as in 6; a
    stored key line with a bare number is refused (E-SYN-008). A bare `Paste`: a copy goes to the
    first free block of its own pool (8), a cut asks where (the keypad); a stored line with a bare
    Paste of a cut is refused, naming the target. (Alternatives — `Paste At Group 10`; no new word,
    `Copy At …` finishing from the clipboard — declined.)
14. **`Cut` is a word**: `Cut Group 1` fills the clipboard as cut. Move always moves — a bare
    `Move Group 3` (and a bare `Insert`) keeps the keypad ask; every word means one thing.
15. **`Insert` is move with shift**: `Insert Group 5 At 9` puts 5 on 9 and moves the old 9, 10
    … up to the first free number (GUIDs kept, one Oops step); a range places the block; between
    owners of the same kind like Move. No empty-hole verb — Move (swap) and Insert (shift) cover it.
    (`Shift` is a value — `Set MeowKey Shift` — so it can't be the word; `Gap` reads as grid gaps.)
16. **`Compact` renumbers a pool**: `Compact Group` (the whole pool, holes closed, order kept),
    `Compact DataPool 2 Group`, `Compact Group 10 Thru 50` (those rows, from 10), `/By Name`,
    `/By Address` (an order). **`Sort` keeps its one meaning: an object's contents** (`Sort
    FeatureGroup "Beam" /By Family`, later `Sort Group 3 /By Address` for its members).
    (Alternatives — Sort doing both by whether ids are given; `Move Group /Compact` — declined.)
17. **Name patterns: a quoted `*`** — anywhere in a quoted id of a target, every row whose name
    fits, ignoring case, through the one resolver (`Group "Mac*"`, `Fixture "backt*blue"`, `Delete
    Group "Front*"`, `Store Cue "BO*" /Merge`). A value is never a pattern (`Label Group 3 "A*"`),
    names may not contain `*` (Label / Store refuse it, E-CMD-008), a pattern that fits nothing is
    not found — never a new name. No bare `*` id (`Group Thru` is every group); the grammar block
    doesn't change (it is the resolver's rule for a QUOTED id).
18. **No Find verb**: patterns in every target do it (`List Group "Front*"`, `Select`, `Delete`);
    the door's Find is the resolver's pattern match.
19. **The tree shell gets `mv`, `cp`, `rm`**, each path through its provider's door (PowerShell's
    providers): the show tree → the pool door (Move = swap, Copy, Delete; their asks, Oops);
    `Root.Drives` → the drive door (`core::drive`): files have no GUIDs, so a move between drives is
    copy + remove; a taken name asks Overwrite / Keep both ("name 2") / Cancel; `rm` never deletes —
    it moves the file to that drive's `.trash` (30 days, as for shows) and the reply carries a
    `Put back:` line; disk changes aren't Oops steps; Setup rights, reach `Drive` (the sweep runs
    them only in its sandbox). Between `Root.Drives` and the show tree they are refused (EXDEV):
    that is `Import` / `Export`.

**Parts 35a–35b built (2026-10-15, branch `lum-syntax`).** 35a: `pools::door` in `lumacore-state` —
`BTreePool<T>` (its map, `renumber`, `identify`, `PoolRules` with every rule a field), the erased
`PoolOps` the verbs use, `next_free` / `next_free_block` / `spacing`, `renewed` (new GUIDs through
`Walk`, links inside following the copy), `copy` (sources read first; Ask writes nothing and names
the taken rows; Overwrite keeps the target's GUID), `relocate` (swap slot by slot, between owners
too), `insert` (the shift), `compact`, `delete`, `users_of`; `TreeDef.door`. 35b: the verbs in
`lumacore-objects` (`commands/door` + one file per verb; `door_verbs!` makes an object's typed
structs) — `At` takes a number or an owner's object of the same kind (another kind E-CMD-020), the
clipboard on `Runtime.Users.{User}.clipboard` (replicated in `SessionRuntime.clipboards`,
SESSION_PROTOCOL 16, cleared by a show load), the existing-object ask, the where-to keypad, the
Delete ask naming users (`ObjectDef::delete_ask`); `Cut` / `Paste` / `Insert` / `Compact` in the
registry (`/By` takes Number and Address too). Group is the first pool on it: its old Copy / Move /
Delete code went, and E-GROUP-002 (`Delete … /Force` while linked) with it.

**Part 35e built (2026-10-15): quoted `*` patterns.** `resolve::name_fits` is the one match rule
(ignoring case, `*` any characters) wherever a quoted id names rows; `resolve::check_name` refuses
a name holding `*` (Label before its sanitizer, `Set … "Name"`, Store Group's name). A Store's
quoted id: an existing name, a new name — or with `*` the one row it fits; a pattern that fits
nothing is not found, one that fits several is refused for now (storing into each, the established `Store Cue
"BO*" /Merge`, waits until the Store handlers move onto the door).

**Part 35f built (2026-10-15): `mv`, `cp`, `rm`.** On the show tree they are **written out** in
dispatch as the Lum line they mean (`door::shell::write_out`, next to `write_out_bare`: `mv 3 7` in
DataPool 1's Groups = `Move DataPool 1 Group 3 At 7`, `--merge` / `--overwrite` = `/Merge` /
`/Overwrite`, `rm … -n` = `/NoConfirmation`) — before rights, forwarding, the journal and Oops, so the
door does the work and a member forwards an absolute line; a stored line runs the same Lum line
nested. Under `Root.Drives` they run on this station (scope Station, Setup) through the drive door:
`--overwrite` / `--keep` (the next free `name 2.ext`) answer a taken name, a move across volumes is
copy + remove, `rm` moves to the volume's `.trash` (a mounted volume's own, else `library/.trash`)
with a `Put back: mv …` line. Move's alias `mv` is gone (the fence word). The Delete ask follows the
last object of an owner-first path (`Delete DataPool 2 Group 3` asks Group's question). Tidy's test
files are `*_tests.rs` and `foo/tests.rs` alike.

**Part 35g1 built (2026-10-15): the DataPool pools.** Sequence (its Delete was planned; a copy's
cues are new objects through its `Walk`; a deleted sequence's running state and echoes go with it),
TimingPreset and CurvePreset are on the door with every verb; each pool now declares its `name`
reader (a TimingPreset has no NodeMeta). `/Merge` on a pool that can't merge is refused, never an
overwrite; Copy declares `/Merge` / `/Overwrite` like Paste. The shim count is exact now (a helper
named `…_typed::<P>` no longer counts as a typed handler): 345, which it already was. Still to come
onto the door: Fixture and Universe (the patch's rules: draft, addresses, generation), the setup
menus' rows (Output, Input, Network, Stations) — then the grid slab runs Copy / Cut / Paste — and the
rest as their areas move (§33 3d).

**Part 35g2 built (2026-10-15): the lines, and the slab on the station's clipboard.** Output and
Input are on the door (a deleted line takes its universe routes with it; every write rebuilds the
ID pools); their Delete is the door's ask ("Delete 3 Outputs (1 + 4 + 7)?" plus what uses them), and
its answer names the rows found then (`Delete Output 1 Thru 2 /NoConfirmation`), never the words as
typed. A setup grid of an object on the door runs `Copy {object} {rows}` / `Cut {object} {rows}` /
`Paste {object} {at}` (bare `Paste` with no row selected); the page remembers only that something
was put aside. Network and the station table stay on their row rules for now.

**Part 35g3 built (2026-10-15): the patch's pools.** Fixture and Universe are on the door for Move
(swap), Insert, Compact (Fixture also `/By Address`: the first break's universe and address) and
Cut / Paste; every write rebuilds the ID pools and bumps `patch_generation` (the engine's channel
tier). Their own Delete stays (it clears the patch's occupancy) and there is no Copy yet — a copy
would sit on the same DMX addresses; the patch area's pass decides both. Each pool declares its own
highest number (`BTreePool::max`: 9999 for pools, the u32 range but the last for fixtures and
universes). `move_cmd` is gone: every Move is the door's.

---

## 36. WebUI, the thin client and 3D rendering (2026-10-18)

**Hardened 2026-10-18, not built.** Asked after the build bench: a fresh build of the app took
50 s on LumaCoreA (Linux) against 11 s on the M3 Pro, a serial chain of gtk-rs crates (gtk 17 s,
then webkit2gtk, wry); "is GTK not a Unix tool? the app should run on Windows later — something
more modern?" Researched with three agents (webview shells on Linux, how big Rust projects keep
GUI build cost down, what lighting / pro apps use). Findings: wry/tao already use WebView2 on
Windows and WKWebView on macOS — GTK is Linux only; every option that drops GTK on Linux bundles
Chromium (Tauri's CEF runtime: Tauri 3 alpha, ~1 GB first download, no mobile; Electron); Servo
embedding is 0.1; wry's GTK4 move is stalled (wry #1474) and would not shorten the build; a web
UI in a thin shell is what the field does (1Password 8, VS Code, Open Stage Control), and one UI
for app, browser and phone avoids QLC+'s second, smaller web UI. Big workspaces keep the GUI crate
out of default builds (Spacedrive "Exclude Tauri from default builds", Zed, Cap). Measured with
`--timings` (both at `ab7fca5`): per-crate speed of the 1950X VM matches the M3 Pro; the server
links in 0.8 s; the gap is the app's GTK chain (39 s) and the start of the server build (11 s).
A reference console's 3D viewer probed from its manual and window settings.

**The WebUI and the shell**

1. **The WebUI is its own crate: `crates/ui/lumacore-webui`** (group `ui`; "WebUI" in docs, logs
   and the UI — the package is lowercase like every crate). It holds every file the station's page
   and the app's connect page are made of: `web/index.html`, `web/connect.html`, `web/kit/`,
   `web/vendor/` (moved from `apps/lumacore-app/frontend/` and `connect/`), and a `build.rs` that
   embeds them (`include_bytes!`, the list from the folders, like the app's `build.rs` today). It
   has no dependencies and no logic beyond handing out the files (tidy checks).
2. **Embedded, with a disk override.** `lumacore-http` serves the WebUI from the embedded files, so a
   deployed server binary carries its UI and no web root has to be found; `--web-root <dir>` serves
   from disk instead, and the scripts pass `crates/ui/lumacore-webui/web`, so a JS / CSS edit is
   live without rebuilding the server. (`ServerConfig.web_root`'s `"web/dist"` default goes.)
3. **The connect page lives in the WebUI too**; the app depends on `lumacore-webui` for it. The
   client rule (§23 "Step 8") becomes: clients compile only `lumacore-wire` and `lumacore-webui`.
4. **One shell crate, OS code in platform modules.** `apps/lumacore-app` stays one crate (the user
   asked for a split per OS; wry/tao already are the per-OS layer — 4 `cfg` blocks in 3,200 lines,
   and a Linux shell needs WebKitGTK in any crate). OS code lives only in
   `src/platform/{macos,windows,linux}.rs` (tidy: no `cfg(target_os)` elsewhere in the app);
   discovery, prefs, IPC and the local server never touch wry / tao, so a core crate can be cut out
   the day one OS gets its own engine (e.g. Linux on CEF).
5. **The app is out of the default build.** The root `Cargo.toml` gets `default-members` = every
   crate but `lumacore-app`: `cargo build` / `cargo test` and `./scripts/unix/test quick` skip it;
   `./scripts/unix/test` (full, with `clippy --workspace`), `cargo test --workspace` and
   `./scripts/unix/build app|all` include it. CLAUDE.md's build and test lines say so.

**3D: one renderer on the station**

6. **Clients never draw 3D.** Every 3D window is a video stream rendered on the station; the
   client only shows it (and a browser, a phone and the Linux app show the same). Camera gestures
   are command lines / window settings sent to the station. The WebUI uses no WebGL / WebGPU in any
   view — live views (faders, DMX, Realtime) stay DOM and 2D canvas. This also takes WebKitGTK's
   WebGL slow paths and NVIDIA / Wayland breakage out of the picture.
7. **One renderer, two levels.** A wgpu renderer on the station draws both **Basic** (wireframe:
   fixtures, beams, rig geometry, GDTF meshes as edges — never shaded) and **Full** (shaded, the established
   levels). Quality is a **RenderQuality pool** on the profile (a few locked defaults plus
   your own), picked per 3D window; each object has `Render: Basic | Full` and the levels (Beam,
   Body, Gobo, Shadow, scales — the established Beam No Beam … High Fancy, Body None … Ultra as the model).
8. **RenderNode is a station service**, like LumaSession / Shell / HTTP: `Set Interface "en1"
   "RenderNode" Yes|No` (station-local, filtered like the others). A 3D window streams from a
   render node in the LumaSession; with none, it shows that none is reachable. Basic is cheap
   enough for most stations, Full wants a GPU station; the DMX render thread keeps priority on any
   station that is both.
9. **Source: `Output | Blind | Preview`, following** (a reference console has Output | Preview with *Follow
   Preview* on by default and no Blind): a 3D window setting picks the source, and `Follow` (default
   on) shows Blind or Preview while you are in it, with that mode's frame colour. Preview — an
   environment where sequences run without touching the output (there: its own DataPool, Go /
   Go− / Goto there) — is a new feature, hardened as its own § before the viewer needs it.
10. **The stream pipe is recorded, decided later.** It must: carry low-latency one-way video from a
    render node to any client engine (WebView2, WKWebView, WebKitGTK, Android WebView, a browser),
    use the station's hardware encoder where there is one, take camera gestures back as command
    lines, and serve more than 3D later (media previews, cameras). Transport (WebRTC vs WebCodecs
    over `/ws/feeds` vs MSE), codec and decode support per engine are researched when the 3D step
    starts. Earlier notes this builds on: DESK-02 "3D visualizer with Rust GPU rendering streamed to
    web UI" (March), §1's `3D / Environment (independently claimable)`, `visions/notes/client-types.md`
    (3D models loaded lazily, per client role).

**The Linux web view**

11. WebKitGTK stays the Linux engine (no better option today). Chromium in kiosk mode on the
    station's page is the supported Linux fallback (the station serves the whole UI to any browser).
    Roadmap: the e2e suite under Playwright's WebKit on LumaCoreA; re-check Tauri's CEF runtime when
    Tauri 3 is stable; watch wry #1474 (GTK4).

**Decisions 1–5 built (2026-10-18, roadmap 3b).** `crates/ui/lumacore-webui` holds `web/` (index,
connect, kit, vendor — moved with history from `apps/lumacore-app/frontend` and `connect/`) and
embeds it (`FILES`, `get`, `INDEX`, `CONNECT`); `lumacore-http` serves the embedded files, or the
`--web-root` folder when it exists (a missing folder falls back to the built-in files with a sysmon
line); a path without a file extension gets the page, an unknown `/api/` or `/ws/` path stays 404;
`--web-root` has no default any more (the `web/dist` placeholder is gone, `Info UIEngine` shows
"built-in"). The app takes its connect page from the crate (its `build.rs` is gone) and no longer
passes `--web-root` to its local server. The app's OS code is in `src/platform.rs` +
`platform/{macos,linux,windows}.rs` (macOS and Linux share `platform/unix.rs`): the menu bar, the
prefs path (Windows: `%APPDATA%`, it read `$HOME` before), the server's file name, the process
group, Force stop's `ps` / `kill`, the browser command. Tidy: `the_webui_is_only_files`,
`os_code_lives_in_platform`; the frontend rules read `web/`. `default-members` leaves the app out;
`./scripts/unix/test check` and `all` run `cargo test --workspace`. The bench row has Server / CLI /
App columns. Checked: clippy on every target (`-D warnings`), the touched crates' tests, a server
without `--web-root` serving the built-in files, e2e connect + surface + app-mode (53, chromium /
phone / webkit). Not yet: the Windows module compiled (no target here), e2e under WebKitGTK on
LumaCoreA.

## 37. Phone hardening (2026-10-19)

The user, after 4d3b (Edit Patch was squeezed on a phone): "should we do a phone hardening round so
our new ui from all ends work better on phone?" — yes, before 4d4 (drag and the lasso are touch
first). First an audit, then one artifact per problem with the real captures and the options side
by side (the user: "artifacts for each problem and screenshot so we can choose between different
solutions"), one question at a time; decided here, built on a go. Mockups:
`docs/dev/ui-kit/mockups/2026-10-19-phone/`.

**The audit** (`tests/e2e/tests/audit-phone.spec.ts`, run with `AUDIT_OUT=<dir>`; 33 surfaces at
phone 390 × 844, phone 360 × 740, phone landscape 844 × 390, tablet 768 × 1024 and 1024 × 768 —
154 captures, each measuring sideways overflow, touch targets under 44 px and controls past the
screen's edge). No page scrolls sideways; the trouble is height and density. The problems, worst
first: (1) the patch window gives the list one row on a 390 px phone and none on 360 px — the
setup frame stacks every desktop band; (2) phone landscape: the slab takes ~60 % of the height in
every setup menu; (3) slab labels clip and run into each other on tablet and landscape; (4) Insert
fixtures on a phone: a long stacked column, an empty Mode block, a wrapped title; (5) Universes ›
Map on a phone: 13 px blocks, unreadable, the title crumb squeezed to "P"; (6) Edit Patch's slots
24 × 32 px; (7) the context menu: a half sheet hanging from the tree tabs. The edit card the audit
missed (its own selector was wrong: a standard slab cell is `data-act`) is fine on a phone: full
screen, tabs, 44 px fields.

1. **The setup frame on a phone folds its chrome and its slab** (problem 1, `phone-1-patch.html`;
   the user: C). On a phone `<luma-setup>` (every setup menu: Patch, Users & Profiles, Network, DMX
   Protocols, Backup) keeps two rows of chrome — the title with the area's state as a chip (tap:
   what Live / your draft / theirs means) and one band with the tree as a chip ("▾ Patch · all ·
   12", opening the tree with its MVR tree / Class / Tag modes and + Layer / + Group as a sheet) and
   the kind tabs as one segmented row — and the grid's selection bar goes into its header. The
   slab becomes one bar: the actions that work now (by the area's state and the selection), Oops,
   ⋯ More (every other action in a sheet, grouped Patch / Rows; greyed cells say why on a tap) and
   the claim button as a small lock cell (tap: what Live means; hold: claim, as today). The list
   gets about ten rows. A desktop and a tablet keep today's layout; the phone breakpoint and
   landscape are problem 2.
2. **A phone is a screen whose short side is under 500 px; on its side the bar goes to the right**
   (problem 2, `phone-2-landscape.html`; the user: B). The frame decides "phone" by the short side
   (390 × 844 upright and 844 × 390 on its side; a 768 × 1024 tablet is not a phone), never by the
   width alone. A phone on its side keeps decision 1's parts but lays them out for width: the
   chrome is one row (the title, the state chip, the tree chip and the kinds side by side), the
   one-bar slab turns into a column on the right (the actions that work now, Oops, ⋯ More, the lock
   at the bottom), and the sheets (tree, More, the lock's explanation) open from the right. The
   list gets the full height under that row (about eight rows of the patch).
3. **A slab action may declare a short label for a narrow cell** (problem 3, `phone-3-labels.html`;
   the user: B). The registry's `Action` (and a standard cell) gets an optional `short` ("Insert…",
   "Next free", "Sort", "Offset 0"; `{value}` forms keep their value); the slab uses it when its
   cells are narrow (a dialog under about 1000 px — tablets, a narrow pane), and an action without
   one wraps to two lines, a little smaller. A label never runs into its neighbour (it is clipped
   at the cell's edge as the last resort). The full name stays the cell's title and the last
   line's. Phones use the one-bar slab (decisions 1–2), whose cells take the short label too.
4. **Insert fixtures on a phone: one step a screen** (problem 4, `phone-4-insert.html`; the user:
   A). On a phone the dialog's Choose step splits in two — **Type** (the search, one chip row with
   Show / Library / Both, ⚙ More for the other filters as a sheet — No list / Manufacturer · Type ·
   Revision, Used only, Details — and Upload; the list of types) then **Mode** (the picked type's
   modes with their slots; the channel map folded under them, "Channels ▸") — and **Place** is the
   form as today. Each is a full screen with ‹ Back and step dots in a one-line title ("Insert
   fixtures ● ● ○ Mode"); a tap on a type goes on to Mode, a tap on a mode to Place. The footer is
   the line, one line (tap to read it whole), and the step's button (Next · Mode, Next · Place,
   Create). The lines are the same as the desktop's. A tablet keeps the approved three columns.
5. **The Universes map on a phone: a list, tap to zoom** (problem 5, `phone-5-map.html`; the user:
   B, "there can be a lot of universes"). On a phone the Map view is a list of universes, one card
   each: number and name, slots in use of 512, how many fixtures, its conflicts in red, and a thin
   overview strip (the runs by fixture type, parked ones red; colour only, no labels). A tap on a
   card zooms into that universe: its slot grid (`<luma-dmxmap zoom: 'slots'>`, problem 6 sets its
   rows), with ‹ back to the list. The conflict list stays under the cards. A tablet and a desktop
   keep the bars (4d2).
6. **The slot grid on a phone: rows of 10** (problem 6, `phone-6-slots.html`; the user: A). On a
   phone `<luma-dmxmap zoom: 'slots'>` draws ten slots a row (labels `u:001`, `u:011`, …), each at
   least 44 px high and the full width (about 30 px wide), no sideways scroll; 52 rows scroll down.
   Edit Patch's fixtures panel folds to one line on a phone (the current fixture, ‹ › for the next
   of the selection), the universe bar stays. Tablets and desktops keep the established rows of 20. The same
   grid is the Map's zoom on a phone (decision 5) and Insert fixtures' DMX sheet (4c3).
7. **The context menu on a phone is a bottom sheet** (problem 7, `phone-7-context.html`; the user:
   A). Like the value dialog: it rises from the bottom over the slab, a grab handle, the selection
   ("1 Fixture · 104", what Live allows) and ✕ on top, then the common actions, the object's own
   lines, and Set folded to one line ("Set · 14 ▸", tap to open). At most about two-thirds of the
   screen, so the rows it acts on stay in sight; it scrolls inside; a swipe down, a tap outside, ✕
   or Esc closes it. Greyed items keep their reason. One sheet look for every phone pop-up (the
   value dialog, the context menu, decision 1's sheets). A tablet and a desktop keep the menu at
   the pointer.


**Built** (2026-10-19, on the user's "lets build the phone"). The phone rule is one module,
`kit/phone.js` (`phoneMode()` → `up` · `side` · '', `onPhoneMode(fn)`: the short side under 500
px); every phone pop-up uses the one sheet, `kit/sheet.js` (`openSheet`, `grabToClose`; in the
Showroom). Per decision:
1. `<luma-setup>` on a phone (`data-phone="up"`): the title is the name and the area's state chip
   (`Live` · `Draft` · `Claimed · <holder>`; a tap opens the slab's lock sheet: the area bar's
   text and the full claim button); one band — the tree chip (`▾ All · 6`, the sidebar with its
   grouping, + Layer / + Group and, with more than one, the device kinds in a sheet; a picked node
   closes it) and the tab strip. No crumb, rail, sidebar, state bar or last line. The grid's
   selection bar is its No cell (`<luma-grid>` `bar: 'head'`: the count; a tap opens + − Thru All
   None Invert and the one-finger drag setting as a menu). `<luma-slab bar>`: the cells that work
   now (the tab's `phone` actions — the Patch tab's are now Patch to… · Edit Patch… · Insert… ·
   Next free · Multipatch… — then New · Delete · Edit · Copy · Import, those that apply; 2
   upright, 4 on its side), Oops, ⋯ More (a sheet: the tab's cells and bound cells, then the
   rows' ten; greyed cells say why) and the power as a small cell — a claim button is the lock
   cell (a tap: the lock sheet; a hold claims). A plain power toggle (Network's Online) still
   presses at once. Found on the way: the area bar and the grid's selection bar both answered to
   `[data-bar]`, so a state change could replace the grid's bar — the area bar is
   `[data-areabar]` now.
2. On its side (`data-phone="side"`): the tree chip and the tabs join the title row (a `left`
   menu keeps its left column, 120 px), the bar is a 104 px column on the right with the lock at
   the bottom (a size container: its width is given), and the sheets open from the right. The
   setup, Edit Patch and Insert fixtures are full screen on a phone either way.
3. `Action.short` in the registry (`menu_defs::short`, sent as `short`): Insert fixtures… →
   Insert…, Next free address → Next free, Load show… → Load…, Save show → Save, New show… → New…,
   Settings tab → Settings. A slab under 1000 px (a container query) shows the short label; an
   action without one wraps to two lines (clamped); the More sheet shows the full names.
4. Insert fixtures on a phone: Type (search, Show / Library / Both, ⚙ More — No list or
   Manufacturer · Type · Revision as headings, Used only, Details — and Upload; a tap on a type goes
   on) → Mode (the type, its revisions when there are several, the modes with their slots,
   "Channels ▸" folds the channel map with its sort; a tap on a mode goes on) → Place (the desk's
   form). ‹ Back, three dots and the step's name in the title; the footer is the line on one line
   (a tap reads it all) and Next · Mode / Next · Place / Create.
5. `<luma-dmxmap>` on a phone draws a card per universe (name, `used / 512`, fixtures, parked in
   red, a 10 px strip of its runs by type, parked red) and the conflict list under them; a tap is
   the `zoom` event — Edit Patch on that universe (rows of 10). As on the desk's Zoom in, Edit
   Patch gets the map's selection (none from a card: Probe and Find work, Patch waits for a
   fixture).
6. The slot grid is 10 a row on an upright phone (`u:001`, `u:011` … 52 rows), 20 on its side,
   44 px high either way, the full width with no sideways scroll. Edit Patch on a phone: full
   screen, the fixtures one line (`‹ Fixture 105 · 1.057 1 of 6 ›`), the tools one row that
   scrolls sideways; on its side the fixture line and the universe bar share a row and the tools
   are icon + word.
7. The context menu on a phone: the kit's sheet (a scrim, the grab handle's swipe, ✕, a tap
   outside, Esc), at most 62 % of the screen; the object's own lines before Set, Set folded
   (`Set · 10 ›`, a tap opens it in place). The host scrolls the first selected row above the
   sheet (`reveal`; a short list gets the room under it while the sheet is open). On its side it
   opens from the right. The value dialog wears the same sheet (the grab handle; from the right on
   its side).
Tests: `tests/e2e/tests/phone.spec.ts` (one per decision; it sets its own screen) and
`support/slab.ts` (`slabCell`: a cell in the bar, else in the More sheet) for the specs that press
slab cells on the phone project. The audit (`AUDIT_SIZES=…` picks sizes) follows the phone's
Insert steps.

## 38. Drives and the one files element (2026-10-19)

Asked while hardening Import MVR (34g4 part 4): its first question drew the file dialog on the
station's `library/` alone, and the user widened it — "in [established consoles] there is a drives list that links to
the local library and attached external drives (HDD, SSD, USB) that can have a library structure;
if it does not have the library path LumaCore inits it", "a unified general way of loading/saving
files", "root.drives.ssda, root.drives.ssdb, root.drives.local-library, root.drives.usb1,
root.drives.smb1". A reference console was probed live over its API: drives sit at
a drive collection, each a `Drive` with Name, DriveType (Internal · Usb · Network), Path,
FreeSpace, TotalSpace; the local library is one folder tree (`library/shows, fixturetypes,
mvr, media, userprofiles, datapools/{macros,sequences,Pawterns}, inout/…`), the same on a stick.
Artifacts in `docs/dev/ui-kit/mockups/2026-10-19-mvr-import/` (its README). Six decisions, one
question each:

1. **The model: the drive list, found and added** (`drives.html`; the user: C). `Root.Drives` is
   this station's drive list — `local-library` (its own `library/`, always Drive 1), every volume
   the station finds (a mounted external disk or stick), and folders you add — never the raw disk
   (today's `Root.Drives` = `/` goes). Each drive carries the one library tree,
   `LumaCore/library/` with the station library's folders (shows, gdtf, mvr, media, userprofiles,
   datapools, macro, presets; never the station's caches and logs; the backup folders
   `.versions` / `.autosave` / `.trash` beside the shows of whichever drive holds them).
   Station-local, never replicated, like NICs; a show never names a drive; an AssetRoot's station
   map (§8, §30 "12a2") names a drive and a folder on it instead of a bare path. A LumaSession
   member's `usb1` is its own stick.
2. **Ids: kind + ordinal, bound to the volume and remembered** (`drive-names.html`; the user: A).
   The id is the kind the station sees plus an ordinal: `local-library`; `ssda`, `ssdb`, `hdda`
   (fixed disks beyond the system one, letters like Linux); `usb1`, `usb2` (removable, bus USB);
   `folder1` (an added folder); `smb1`, `ftp1`, `sftp1` (LumaCore's own network connections, 6);
   `cloud1` later. The first time the station sees a volume it gets the next free id of its kind
   and the station remembers volume id (UUID / serial) → drive id, kind, label, last seen, in its
   settings: the same stick is `usb1` every day on this station; a new one is `usb2` even while
   `usb1` is out; absent drives stay listed greyed ("out since 14:02"); Forget frees the id. The
   volume's label (KINGSTON) is the display name beside the id, never the id. A kind the OS
   misreports is corrected once (`Set Drive "ssdb" "Kind" "USB"` → the next free `usb` id,
   remembered). Not chosen: slots (the first stick in is `usb1` — a macro's `usb1` would mean
   "whatever is in first"), the label as the id.
3. **Two ways onto a drive: the node is the volume, the library is marked, `~usb1` is the short
   way** (`drive-root-library.html`; the user: A, "we need two ways onto a drive, its root and
   the library path"). `Root.Drives.usb1` lists the whole stick, `LumaCore` first and marked ★
   library; the tree shell gets Unix's home: `cd ~usb1` = that drive's library, `cd ~` = the local
   library (`~` and `~name` as a TREE-PATH start, a grammar edit). Every file dialog opens on the
   drive's library folder of its kind; its crumbs are the jump — the drive's name for the whole
   volume, ★ Library back; a file picked outside the library is copied into the right library
   folder first (asked once), so a line stays a library name. `Info Drive` shows both paths. Not
   chosen: the library at the node with the volume under a reserved child (the folder twice in
   the tree), two entries per drive in the rail.
4. **The lines: a `/Drive` option on every file word** (`drive-lines.html`; the user: A; a lum.md
   discussion, now in lum.md "### Drives", the examples in `PENDING` until built). The file name
   stays relative to the word's library folder; `/Drive "<id>"` says which library, absent = the
   local library: `LoadShow "rig" /Drive "usb1"`, `SaveShow "rig" /Drive "usb1"`, `Import Mvr
   "Rig 2026 v3.mvr" /Drive "usb1"`, `Export UserProfile "Bob" /Drive "usb1"` — one option declared
   once (`F::DRIVE`, like `/Version`), every file word takes it; the id is quoted as every name id
   is (the artifact's bare `usb1` fell to the parser's rule: a bare word is a vocabulary word). The **`Drive` object**: station-
   local keyed rows like `Interface`, declared properties `Label` (text; the label by default),
   `Kind` (Internal · SSD · HDD · USB · Folder · SMB · FTP · SFTP), `Path` (the mount, or the added
   path), `Library` (Switch: Yes when `LumaCore/library/` exists — **Set Yes creates it**; No is
   refused, a setting deletes nothing), `Free`, `Total`, `State` (In · Out), `Since` (read-only);
   `List Drive`, `Info Drive "usb1"`, `Set Drive "usb1" "Label" "…"`; `Store Drive "folder1"
   /Path "/srv/shows"` adds a folder (the next free id of its kind is its name); `Delete Drive
   "usb1"` forgets a found drive (frees its id) or removes an added one, asking when an AssetRoot
   maps to it. `Eject Drive "usb1"` is the round's one new verb, planned until built. The Drives
   menu (`Menu "Drives"`) is a `MenuDef` grid on these properties, every cell a `Set` line; its
   slab: Initialise (= `Set … "Library" Yes`), Eject, Rename, Add folder…, Remove, Info. Not
   chosen: the drive inside the quoted name (`"usb1:rig"` — a syntax inside text, and `:` is what
   file names refuse), file words taking tree paths (only the fence enters `Root.Drives`, §25).
5. **One way to load and save: `<luma-files>` on one feed** (`files-element.html`; the user: A).
   One kit element: the drives rail (the drives, then **This device** — the browser upload — last),
   the crumbs, a `<luma-grid>` of the folder's files with the kind's columns (a show's format /
   saved by / versions as today; an MVR's scene, layers, fixtures, GDTFs embedded or missing; a
   GDTF's manufacturer and modes; a profile's user and date; a picture's pixels), the drop zone,
   and a name field for a save (a taken name is the station's ask, Overwrite / Keep both /
   Cancel, as `SaveShow` asks). The rows come from one `files` feed on `/ws/feeds`
   (`sub('files', { drive, folder, kind })`), built on the state thread from a cache keyed by path
   + mtime (file heads read on `core::background`, the rows fill in). Every file dialog is this
   element with a kind: Backup › Library *is* it at `shows/` with its slab (Load show…, Save show,
   Save as…, Import, Export), Import MVR's File step, Insert fixtures' Library… (its list moves
   onto the grid; Upload stays), Import / Export UserProfile, Import Image, and the station's
   "which file?" ask opens it. The footer is the line. The TUI has the fence (`ls ~usb1.shows`),
   no picker. Not chosen: each dialog keeping its list and gaining the rail (three lists, three
   looks), the picker as a station-held dialog (a sortable grid of hundreds of rows in the ask
   protocol).
6. **The service** (`drive-service.html`; the user: first write, never an ask; network drives are
   LumaCore's own, "never exposed to the OS", not now). A station thread (never the state thread)
   reads the OS's mounts every 2 s — Linux `/proc/mounts` + `/dev/disk/by-uuid` / `by-id`, macOS
   `/Volumes` + `diskutil`, Windows drive letters + volume serials — and sends the state thread one
   message when something changed; the state thread keeps the table (2), publishes the drives in
   `StationView` and the `station` feed (a stick appears in every open dialog at once) and writes a
   sysmon line (`Drives: usb1 "KINGSTON" in (28 GB free, library ready)`, `usb1 out`,
   `usb1 library created`). Free space is read on the tick; a save that would not fit is refused
   before writing (E-CMD-012 with the numbers). **The library tree is created on the first
   write** — a save, an export, a copy-in, a tap on the Library cell — never by a read, never by
   an ask: a stick only looked at stays untouched; a read-only stick's first write fails with the
   reason. **Network drives are LumaCore's own clients, in process, multiplatform — SMB, FTP,
   SFTP and what more comes — never the OS's mounts** (`Store Drive "smb1" /Path "//nas/shows"
   /User … ` with the credentials in the station's keystore; `ftp1`, `sftp1`); decided as the
   direction, **not implemented now** (planned rows). An OS-mounted share is only ever a Folder
   drive someone added by path; the network ids are reserved for LumaCore's own connections.

**Parts** (each committed on its own; the Import MVR hardening resumes after 38b with its Options
step, the review, the lines and the placement asks): **38a** the `Drive` object (the registry row,
the keyed properties, `Store` / `Delete`, `Eject` planned), the watcher and the table, the
`station` feed's drives, the Drives menu, `~` in the fence, `/Drive` on every file word, the
library tree on first write; **38b** the `files` feed and `<luma-files>` (the Showroom first),
Backup › Library and Insert fixtures' library on it, Save as… on it, the station's "which file?"
ask opening it, This device on the one upload route (`POST /api/upload?drive=&folder=&name=`,
512 MB, the feedback pill); **38c** Import MVR's File step on it (34g4 part 4 proper);
**38d** network drives (planned). Tests: `kit-drives.spec.ts` (a temp folder the watcher is told
to treat as a volume, `LUMACORE_DRIVES_PROBE`, so a "stick" can come and go in a test).

**38a built (2026-10-19)**, in a crate of its own at the user's word ("we need a dedicated crate,
lumacore-drives … so we have a better structure that is unified for the future"):
`crates/formats/lumacore-drives` — `probe` (the OS's mounted volumes: Linux `/proc/mounts` +
`/sys/block` + `/dev/disk/by-uuid`, macOS `/Volumes`, Windows the letters; free space from one
`df -kP`; `LUMACORE_DRIVES_PROBE`), `library` (`LumaCore/library/` with the folders and a README,
made by `init_library`), `ids` (`next_id`), `table` (`Drive`, `reconcile`: ids bound to the
volume's key and remembered, present / Out, the sysmon lines). On it: `lumacore_state::drives`
(the live list on `AppState::drives`, `StationSettings::drives` saved with the station settings,
the mount map, `drive_library` / `ensure_library` — the first write makes the tree);
`lumacore-tree` `DriveKind` / `DriveMemory`; the server's `drives` thread (every 2 s, a message
to the state thread only when something changed, `station` feed `drives` rows, sysmon
`Drives : usb1 "KINGSTON" in (…)`, `… out`, `… library created`, `… forgotten`);
`lumacore-cmd-station` `drives.rs` — the `Drive` object as a keyed station-local object
(`Label`, `Kind` (re-ids), `Path`, `Library` (Set Yes makes it), `Free`, `Total`, `State`,
`Since`; typed `List` / `Info` / `Store … /Path` / `Delete`, `Eject` planned) and the Drives
menu (`Menu "Drives"`: Initialise · Eject · Rename… · Add folder… · Remove · Info); the fence's
`~` / `~usb1` (`Home` token, `navigation`), `Root.Drives` = the drive list and
`Root.Drives.<id>` its volume root (`vfs`); `/Drive "usb1"` on `LoadShow` and `SaveShow`
(`backups::DirGuard`: that drive's `shows/` with versions, recovery copies and trash beside it;
Import FixtureType, Import / Export UserProfile and Import Image take it with 38b's files
element). The parser's test vocabulary and the catalog's golden words carry the new words.
Tests: `cargo test -p lumacore-drives`, the state's, `tests/e2e/tests/kit-drives.spec.ts`.

**38b built in parts (2026-10-19)**: **38b-1** the `files` feed (`sub files {drive, folder, kind}`:
a drive's library folder by kind, keyed by file name, folders first, a show's manifest facts on
its row, `error` for a drive unknown or out; rebuilt when the station's drives or the library's
writes move — `Revisions.library` counts `drive`'s writes, never a timer) and the one upload route
`POST /api/upload?drive=&folder=&name=` (512 MB, the body is the file, the first write makes the
tree, a path leaving the library is refused). **38b-2** `<luma-files>` (`kit/files.js`: the drives
rail from the `station` feed with This device at its foot, the crumbs `usb1 › LumaCore › library
› shows` with ★ Library, the folder's rows in a `<luma-grid>`, a dir row navigates, a double-tap
picks, the Name row for a save, the drop; events `pick` / `save` / `select` / `where` / `upload`;
in the Showroom). **38b-3** Backup › Library on it: the tab's content is `Content::Drives(&LIBRARY)`
(the registry's new content kind — a grid with the drives rail, `"drives": true` in the menu
JSON), `<luma-setup>` draws the grid inside the files element's host mode (`update({ host: true
})`: the rail and the crumbs only, the host fills the grid), the rail's `where` resubscribes the
`shows` feed with `drive` (the server builds the same library view under a `DirGuard` on that
drive's `shows/`), `{drive}` in a tab's lines becomes ` /Drive "usb1"` off the local library
(`SaveShow{drive}`, `SaveShow {value}{drive}`, the slab's `Delete Show {rows}{drive}`), the Load
show dialog reads `/api/show-parts?drive=` and its `LoadShow` / `Import Show` line carries the
drive, a double-tap is Load…; the Show object's Delete and Import take `/Drive`, and every
show-file question's answer line carries the drive (`Opts::drive_word` in `confirm`; a
multi-show Delete too). Screenshots `target/screenshots/backup-drives-{desktop,phone}.png`.
**38b-4** the station's "which file?" ask *is* the element: a dialog input kind
`InputKind::File { files: <kind> }` (`Ask::file`; CLIENT_PROTOCOL 21) that the kit's dialog draws
as `<luma-files>` on the kind's folder with the drives rail (the element lives across the
station's updates of the dialog); the value is `<drive>:<file>` and the station renders it into
the answer line as `"file"` plus ` /Drive "usb1"` off the local library (`clean_input`; an empty
pick is "Which file?"); a double-tap presses the primary button. A bare `Import UserProfile`
(the pick list gone), `Import FixtureType` and `Import Image` ask so. `/Drive` on the rest of the
file words through one door, `AppState::library_folder(drive, folder, write)`: Import FixtureType
(and `Store Fixture … /FixtureType "file.gdtf" /Drive "usb1"`), Import / Export UserProfile,
Import Image; `gdtf_library::library_in(dir)` reads any gdtf folder (cached by path), so the
`files` feed's `gdtf` rows carry manufacturer, type, modes and revision (the kind's columns) and
`/api/fixture-types?drive=` / `/api/fixture-type?file=&drive=` serve a drive's files. Insert
fixtures' Library list is the element (its grid shows the GDTF facts; the rail's drive reloads
the list; the Store line carries `/Drive`; Upload stays). Screenshots
`target/screenshots/files-ask-{desktop,phone}.png`, `insert-library-desktop.png`. Not in 38b:
a profile's user / date and a picture's pixels as feed columns (the rows have name, size,
modified), the phone's Insert type list (it keeps the API list), Export on the element's name
field (Save as… is the show's).

## 39. The settings home icons (2026-10-20)

Asked by the user while 34g4 part 4 was built ("remove the 'In the show · every station' string on
all buttons in the whole UI"); mockups `docs/dev/ui-kit/mockups/2026-10-20-home-mark/`. Decided
in two questions:

1. **No words on cells** (`home-mark.html`; the user: B, with three homes and an icon). The home
   mark decided in §30 "12d" ("every cell says so") repeated the same words on every cell of a
   tab, cutting the names short; the tab's header already said it once. The words go from every
   settings cell and every edit-card cell. What stays: the fact, as an icon and a tooltip.
2. **Three homes, one icon each, on every cell** (`home-icons.html`; the user: 1). The one
   property registry's `Home` gains **User**: a value in the show file that is one user's — a
   profile's preferences (the MVR import preset, Suggestions, the Oops settings, the look, a
   window's settings), a user's own (password, rights). So: **Show file** = the show, every
   station (icon `save`) · **User** = yours, in the show (`user`) · **Station** = this station
   only, never in the show (`server`), the kit's own Lucide glyphs. Every cell carries its home
   icon at the right of its label, grey; where a cell's home differs from its tab's it is gold
   (the Network menu's identity cells among the wiring, a mixed edit card). The tooltip (hover,
   hold on touch) spells it out: "Show file · every station", "Your profile · in the show", "This
   station · never in the show". The tab's or menu's header keeps its one line, now with the
   icon. Not chosen: A the words gone with only the header and the tooltip (nothing on the
   cells), C the words kept on mixed tabs; 2 the icon only on the odd ones out (the user: every
   cell, grey, so a cell says its home at a glance).

**What changes when built**: `lumacore_state::props::Home::User` and `word()` = `user`; the
`UserProfile` and `User` properties, the window settings and the screen props declare it (the
rights gate and `Set` don't care); `Tab.home` and a menu's `home` take `user` too; `SettingField.
home` carries the word; the kit's `homeMark(home, tabHome)` draws `<luma-icon>` with the tooltip
(`settings.js`, `editcard.js`) and the hosts pass the tab's home (`setup.js`, the edit card from
its object); `HOME_TEXT` becomes icon + words in the header; `kit-12d.spec.ts` checks the icons
(`data-home`) instead of the words; the Showroom's SetupSettings board shows the three.

**Built (2026-10-20)**: `Home::User` (`word` = `user`) and `Props::home` (an object's home for all
its properties, `home_of`): the `UserProfile` object's props declare it, a user's own settings
and a window's settings say `user`; `EditSetting`'s fields and the grid's columns
(`column_json` `home`) carry the word. The kit: `settings.js` `HOME` = { show: save, user: user,
station: server } with the tooltip words, `homeMark(home, tabHome)` draws `<luma-icon
class="lc-home" data-home>` (gold `.odd` where it differs from the tab's), `homeLine(home)` the
header's icon + words; `<luma-settings>` and the grid take `home` (the tab's) in their spec, the
edit card gets it from the grid, the people element and the Import MVR dialog say `user`;
`setup.js` draws the tab's home line with the icon. The e2e checks (`kit-12d.spec.ts`) read
`data-home` as before; the Showroom's Kinds board has a user cell and the legend.

## 40. The demo rig, nested fixtures and second ids (2026-10-20)

Asked by the user after §39: "expand the demo with more fixtures and groups, analyze ../ and
find a big mvr we can have in the demo" — then "../ExternalData/DemoData is the data i think we
should use" (one console MVR export of The International 2026 and 39 DataPool XML exports,
1.6 GB), and "lets discuss this and harden with artifacts first before we implement anything".
Seventeen questions, one at a time, each on a page with real captures (`docs/dev/ui-kit/mockups/
2026-10-20-demo-rig/`, README there). The desk itself was probed read-only through the user's
API bridge (192.0.2.10, the TI show loaded) wherever the console's own behaviour mattered.

**What the files showed (findings, all measured):**

1. 186 MVR files in `../ExternalData/BombeeRigMetadata/mvr/`, 41 153 fixtures, every one read by
   `lumacore-mvr` without error. The number sits in `<FixtureID>` as text in every file before
   1.6 (Capture, Vectorworks, Depence); the crate read only `FixtureIDNumeric`, so every row of
   every real file came in without a number (fixed while scanning: the text is the number when it
   parses as one; one test).
2. The station's GDTF reader (the `gdtf` 0.2.0 crate) refused 672 of the 1 084 GDTFs embedded in
   those files — for attributes the spec gives defaults for (`PhysicalUnit` 482, `Axis Position`
   73, `Highlight` 45, `Pretty` 43, `Description` 43, NaN colours, malformed gamut points, a
   negative `DMXFrom`, an oversized CRI) that Capture and Depence leave out — while taking 393 of a
   random 400 from the GDTF-DB. **Decided (1): fork the crate** into `vendor/gdtf/` under
   `[patch.crates-io]`, the spec's defaults and lenient value readers, every change marked
   `// LumaCore:` and listed in `vendor/README.md`. Built the same day: 1 084 of 1 084 parse,
   399 of 400 of the GDTF-DB sample (one odd Laser geometry left).
3. Capture's exports carry one stripped GDTF per type whose every channel is the attribute
   `Dummy` — 584 of 1 137 embedded GDTFs, 85 files entirely: a rig imported from them has no
   dimmer, pan or colour. Vectorworks and Depence embed the manufacturer's file (86 files). A
   Capture rig needs its types matched to the library by name — a real Import MVR feature, on the
   roadmap, not for the demo. (The GDTF-DB copy, 13 844 files, indexes by manufacturer and name in
   34 s.)
4. In 172 of 186 files the MVR groups hold trusses and scene objects, not fixtures; only 14 put
   fixtures in groups and one names them. The demo's groups are LumaCore's own.
5. The DemoData MVR: MVR 1.6 from a console export, 1 371 rows = 1 348 fixtures + 23 folders, 22 real
   types, 106 universes, FixtureID and FixtureIDNumeric both set; imported for real in 351 ms,
   show file 8.6 MB. The DataPool exports are whole console pools (Default: 87 groups, 100 sequences,
   25 preset pools, 136 Pawterns, 33 layouts; Event: 809 sequences; Busk1: 663; …). The Default
   pool's groups name this rig one for one (Photon 245/245, Icarus 284/284, Blinder 344/344 …);
   3 713 of their 11 393 items are fixture ids, the rest sub-fixture ids.
6. The console exports its fixture-tree folders as fixtures of type `Generic@Grouping` — the user's
   own tree ("the fixture tree is my own with the grouping fixture"): ALL › Stage 1 Mainstage ›
   ACME Icarus … On the desk a folder is a fixture with **IDType Media, no FID and a CID** (ALL =
   Media 99, Stage 1 Mainstage = Media 1, ACME Icarus = Media 7003 — "i did set it to media
   because i did not want FIDs on the group fixtures"); the file writes the number as `CustomId`
   and no `CustomIdType`. The folders' Media ids 1 … 9, 99 and 7003 are all also fixture numbers
   in the same show. On the desk a parent's children come back in the file's order (849, 852, 854,
   853 …) while each child's index in its address is its own id; sub-fixtures are 1, 2, 3 under
   their fixture with no id type.
7. The demo is a test fixture: 12 e2e files and 30 Rust test files build or load it; the rig
   costs 2.2 s per `demo::build` in a debug build; `demo.luma` has 45 versions and 41 MB of
   history; the show format bumped 27 times in ten days.

**Decided (the user's answers, in order):**

1. **The GDTF reader is the forked crate** (above).
2. **The rig is the DemoData MVR** (the user's choice over the shortlist of real-GDTF rigs).
3. **Nested fixtures, natively** — "it should be possible to nest fixtures as well in lumacore so
   we can natively import files with nested fixtures with a group fixture". A fixture may contain
   fixtures (MVR's `ChildList`, any fixture, Grouping or not); a folder is a fixture of a
   channel-less type with a position of its own. **Sub-fixtures of a fixture are never a thing
   here** — "never embrace subfixtures of fixtures because we have the part/element system later.
   a fixture is a fixture." (lum.md's dotted fixture ids go to that system's page; the export's `849.1`
   items in groups are left out.)
4. **Second ids the MVR way** — "i think this means we need to implement the CIDs the MVR/[console] way
   now?" Yes: the "No CIDs: tags instead" decision (§34 round 3) is reversed; the roadmap's
   "Discuss CIDs against tags" item closes with this section.
5. **The patch's key is an internal slot** ("A sounds right"): `BTreeMap<u32, Fixture>` stays,
   the key given at insert, never shown, never on a line; nothing stored points at it (selections,
   groups, presets, cues hold GUIDs already).
6. **The tree index is the slot plus an order on each node**: every fixture, layer and group keeps
   `parent` + `order` (spaced 10, 20, 30 …); a new node takes the next free order under its
   parent, insert in the middle takes the midpoint, siblings are respaced in that one step when
   neighbours touch; the MVR's order round-trips; the sidebar, the grid's tree view and the export
   walk (parent, order). The desk confirmed the split (finding 6).
7. **A fixture carries the triple**: `fid: Option<u32>` · `id_type` · `cid: Option<u32>`
   (replacing the key-as-FID, `custom_id_type` / `custom_id` and `more_ids`); FID unique in the
   show, CID unique per id type; three properties `"FID"` / `"IDType"` / `"CID"`, each a grid
   column; a folder is (None, type, n); `MoreIds` goes and its roadmap item closes.
8. **The line names a second id inside the fixture's id, as lum.md has it**: `Fixture Media 7003`,
   `Fixture Media 1 Thru 9`; with Fixture as the default object a bare `Media 7003` is written
   out (§33 21). Not a bare `Media 7003` (`Effect` is §28's object; nine more object words
   would enter the one namespace).
9. **An untyped CustomId gets the preset's id type**: a 17th "MVR import" property on the profile,
   `MvrIdType`; `/IdType` on the line overrides once; a typed CustomId is taken as typed; no
   number and no CustomId keeps the `/FidOffset` rule; the review shows the id as it will be.
10. **Generic is the default id type** — "Media is not [the console's] word for a numberless folder, i set it
    to media because there were no other good default alternative in [the console], i want Generic as a
    default ID type for CIDs that we can FOR EXAMPLE use for folders/grouping fixtures". The MVR
    spec numbers its CustomID types `0 Undefined · 1 Fixture · 2 Channel · 4 Houselights · 5
    NonDim · 6 Media · 7 Fog · 8 Effect · 9 Pyro · 10 Marker` and skips 3 — the console's own Universal /
    Generic slot (the desk shows `IDTYPE Generic`); LumaCore's list gains `Generic` = 3, the
    preset's default and a new CID's default. No LumaCore-only word (no "Folder"): this rig's
    folders arrive as `Fixture Media n` because the file says Media; the next rig can say Generic.
11. **A folder on a line is the folder**: `Fixture Media 1` selects only the folder.
12. **A value at a folder fans out** ("sends the value down to the fixtures"): `Fixture Media 1 At
    50` writes 50 into every fixture below, one Oops step; the folder keeps nothing. Later a
    switch on the programmer sends values either directly to the selected object or down into the
    tree (the user; roadmap). Move and Delete carry the children (the one nesting rule; Delete
    asks Keep / Delete the children as MVR nodes do). The sidebar's MVR tree shows nested fixtures
    at any depth; the grid gains a tree view in (parent, order) with a folder's id in grey where a
    FID would be; Insert and Import place new fixtures at the end of the picked node; Export
    writes `ChildList` back in order.
13. **The groups come through a small pool XML reader**: `formats/lumacore-poolxml`, Groups first —
    `Import PoolXml "file" Groups [/Drive] [/DataPool]` makes one LumaCore group per exported group in the
    selected DataPool, members by fixture id (a second-id item by its type), grid cell from X / Y,
    order kept; a group of sub-fixture items only is reported and skipped. Presets, sequences,
    macros are later parts of the same reader, each when its LumaCore object exists.
14. **One show: the rig replaces the demo** — "it is good to have a large dataset now when we
    start to test things". Fixtures 101–106 and universes 1 and 2 exist in the rig, so many tests
    keep their numbers; those asserting six fixtures, AuraXB names or two universes are rewritten.
15. **Two demos** — a **generated demo** built from the DemoData through the station's words
    (`Import Mvr`, `Import PoolXml … Groups`, more as words land), "so we can test how data comes in
    and add more later"; and a **manual demo** from the user's own saves while developing, kept in
    git, "so we can start creating a demo manually"; the generated one takes its UserProfile from
    the manual one "so it keeps up to date with how everything looks". The DemoData the generated
    demo needs is in git so it can build anywhere: `library/mvr/ti2026.mvr` (4.4 MB) and the
    Default pool's Groups element as `ti2026-groups.xml` under `library/` (0.8 MB) now, each further
    pool's extract when its word lands, never the 1.6 GB folder. The built `demo.luma` is not
    committed (gitignored; built when missing or stale by the station's start, the e2e support and
    the scripts — one door); the demo test builds it and checks its counts and a golden of the
    normalized JSON without the GDTF bytes.
16. **The manual demo is lines, not a file**: `library/demo/manual.lum`, a script of command lines
    in git, replayed by gen-demo on top of the generated show before it saves; immune to format
    bumps (§20 stands untouched); the user writes what is to be kept into it (the journal shows
    every line run; a `/Script` export of a session's lines is a small later word); the profile
    travels as `Set UserProfile …` lines plus an imported layout. gen-demo replays the file itself
    for now; a station word that runs a script file is its own discussion (roadmap).
17. **Names stay as the desk had them**; the show is called `Demo`; the files `ti2026.mvr` /
    `ti2026-groups.xml`; one line in the docs says where the rig is from.

**Seen on the way, for the build (not questions):** `Import Mvr 1 Thru 9999 /Apply` refuses on
the first number the review lacks where the one range rule says to visit only existing numbers;
Apply repeats a failing type's error once per row instead of once per type; the import creates
layers only for fixtures it writes (right) and today turns a fixture parent into a nameless
recreated group (goes with 3); PGL Cluj's file has 14 address overlaps and Jireel's 168 — the
reference check lists them, the import takes them as the file says.

**The build, in parts** (each hardened above; tests per step, the gate before each commit):

- 40a the GDTF reader's fork + the FixtureID fallback ✓ built 2026-10-20 (commit 19579cdb).
- 40b ✓ built 2026-10-20: `Slot` (no `Display`, so a slot that reaches text fails to compile),
  the triple on `Fixture.Mvr` (`fid` / `id_type` / `cid`; `IdType` with Generic = 3; `MoreIds`
  gone), `order` on fixtures, layers and groups with `next_order`, format 31, the one door on
  `PatchData` (above), `FixturePool` (the door verbs by number, not slot), `by_number()` for the
  old selection resolver and the walk, `fixture_guids` for a plain `Fixture Media 7003`, the
  properties `"FID"` (renumbers, the slot stays) / `"CID"` (Generic when untyped) with their
  uniqueness, the tree path `Fixtures.<fid>`, the grid rows' `id` / `id_type` / `cid`, the demo
  regenerated. Seen and fixed on the way: the tree shell's "Put back" line and its drive tests
  named host paths under `Root.Drives` (broken since 38a: a drive's id comes first now); an
  election test still claimed the patch with the old `Patch` word; the completer listed slots.
- 40c ✓ built 2026-10-20: a file fixture whose parent is a file fixture nests under it (the row's
  `parent_fixture`, written parents-first — the file is in tree order), no group asked; the
  profile's 17th "MVR import" property `MvrIdType` (Generic) and `/IdType` type an untyped
  CustomId, the review's rows carry `id` / `id_type` / `cid` / `order` and show "Media 7003" in
  the No cell where a number would be (the list, the kit); a numberless fixture with a second id
  is New, not a clash; a folder with no id at all — the console lets a tree node go without a CID
  (four of the TI rig's 23) — gets the next free second id of the preset's type, shown as it
  will be, while an id-less leaf keeps the FidOffset rule; a folder already in the draft is Same
  by uuid and never made twice; `change` finds a matched row by GUID; `Import Mvr 901 Thru 9999`
  visits the rows that exist (the one range rule), a single number must; Apply's reply names a
  reason once with a count ("Stage 1 and 206 more (…)"), not once per row. Tested on the real rig:
  1 371 rows, 1 348 fixtures and 23 folders, ACME Icarus (Media 7003) with 286 children under
  Stage 1 Mainstage (Media 1).
- 40e ✓ built 2026-10-20: `formats/lumacore-poolxml` (quick-xml events, nothing kept but the Groups
  element of the export's `Pool`; the pool's other children counted by name), the `PoolXml` object with
  `Import PoolXml "file" Groups [/Drive "usb1"] [/DataPool n]` (one group per exported group in the
  selected or named DataPool, members by fixture number with their grid cells, the file's order;
  a group of sub-fixture items only is skipped and named, sub-fixture items and unknown numbers
  counted in the reply; `Groups` a value word), a groups extract script (the Groups element of
  a big export as its own file), `ti2026-groups.xml` under `library/` (the Default pool's 87 groups,
  0.8 MB). On the real rig: 77 groups made, 10 of cells only skipped, 2 882 sub-fixture items
  left out, every fixture number found.
- 40b the id model: the slot key, `fid` / `id_type` / `cid`, `Generic`, `parent` + `order` on
  fixtures, layers and groups, format 31, the sweep of the 87 sites behind one door
  (`patch.by_id`, a tidy ratchet), the three properties and columns, the resolver's second-id walk
  on the triple, `MoreIds` out, lum.md's rows (below), the demo regenerated.
- 40c Import / Export MVR with the tree: folders as fixtures, nesting as `ChildList` says, the
  untyped id and `MvrIdType` / `/IdType`, the review's rows for folders, the range rule on Apply,
  one error per type; Export writes `ChildList` and the ids back.
- 40d the tree in the dialog — **hardened 2026-10-20** (four questions, pages
  `docs/dev/ui-kit/mockups/2026-10-20-patch-tree/`, an interactive lab on the rig's 1 371 rows;
  the API bridge answered nothing that evening, the desk's Patch window is still to be
  captured): **1–2** the user's own direction after two mockups — the sidebar shows **nodes only**
  (layers › MVR groups › folder fixtures, nested as the show has them, a folder with its second
  id and its fixture count, Direct | Deep with Deep the default) and **never a fixture**; the
  Live patch pane shows **fixtures only, never a folder or a group** — a flat list scoped by
  the sidebar's pick, in tree order by default (a node's fixtures together, in their order), by
  a column when its header is tapped, with an **MVR node** column (the node's name, the path on
  hover). **3** fixtures get into the tree by **select, then drag onto a sidebar node**: the node
  lights under the pointer, a chip follows ("6 fixtures → Smoke"), the drop runs one line,
  `Assign Fixture 1216 Thru 1221 At Fixture Media 9` — Assign's node list widens from
  MvrLayer / MvrGroup to a folder fixture (E-PATCH-011 grows by a kind); a sidebar node dragged
  onto a node runs `Move Fixture Media 7003 At Fixture Media 2` through the one nesting ask;
  **At a node nests, At a bare number renumbers**, a folder At a number is refused; the rows'
  context menu has **Move into…** (a pick of the nodes) for a phone, where the tree is a sheet;
  Insert fixtures… and `Store Fixture` under a picked node run `Store …; Assign … At <node>` —
  two statements, no new flag; "No MVR node" is not a drop target (a fixture leaves the tree only
  by Delete). **Delete of a node asks with both ways out** (A of three): "Delete Fixture Media 1
  "Stage 1 Mainstage" — 9 folders and 1 246 fixtures under it?" · Delete all | Keep them: move
  up to Fixture Layer | Cancel; `/NoConfirmation` = all, `/KeepChildren` the other (as `Delete
  Tag` has it), what stays moves up one level, its order kept. **4** the words as proposed —
  no change to lum.md's grammar, the folder target joins the rows. Counts are fixtures only
  ("1 348 fixtures · 23 folders"): the sidebar, `ls`, the tree chip. The sidebar node's ≡ menu:
  Move…, Rename…, New folder here, Delete. Found on the rig, to fix with it: `List Fixture Media
  1` listed the whole show, `Info Fixture Media 1` wanted a number, `Delete Fixture Media 7006
  /NoConfirmation` said "Fixture 0 not found", `Move Fixture 102 At Fixture Media 1` renumbered
  102 and 1, a hand-stored fixture landed in no node.
- 40d ✓ built 2026-10-20: the sidebar is **nodes only** — `lumacore_views::patch` adds a `folder`
  node for every fixture without a number or holding fixtures (its second id as `id`, `Media 9`;
  lines select / at / label / move / delete; no child line yet) under its layer, group or folder,
  counts fixtures only along the parent chains (Direct | Deep, Deep the default); the pane lists
  **fixtures only**, in tree order (`tree_key` of the orders), the `MVR node` column names the
  node, a folder's name too. **Sorting is the one lift gesture**: `<luma-grid>` lifts its
  selected rows when `lift` is set (events `lift` / `liftmove` / `drop`), the tree's rows too
  (`spec.lift`); the setup page draws the chip ("2 fixtures → Smoke"), lights the node under the
  pointer (`.drop`) and runs `Assign Fixture <rows> At <node>` — a lifted node runs `Move <node>
  At <dest>` through the nesting ask; the rows' context menu has **Move into…**
  (`lumacore_views::context` `move_into`, value kind `node` → the kit's pick of every node with an
  `at` line); Insert fixtures… under a picked node adds `Assign … At <node>` as its second
  statement (`insert.js` `group`). The words (`objects/patch/devices_meta/nodes.rs`:
  `node_target` takes `MvrLayer n`, `MvrGroup n` or a folder `Fixture Media n`, E-PATCH-011 names
  the three): `Assign … At <node>` places by GUID (never under itself or a descendant, new
  `order`s after the node's last); `Move Fixture` (`nest.rs` `MoveFixture`: At a node → the nest
  ask, then the parents; At a number → `door::relocate`; a folder At a number refused);
  `Delete Fixture` with `/KeepChildren` and `fixture::delete_ask` (Plain when nothing is under;
  "Delete Fixture Media 9 "Smoke" — 6 fixtures under it?" · Delete all | Keep them: move up to
  MvrLayer 1 "Fixture Layer" | Cancel — the kept ones lifted to the parent with new orders);
  `List Fixture Media 1` lists the node's fixtures (`nodes::fixtures_under`), `Info Fixture Media
  1` a sheet (Name, GUID, Type, In, Holds); `ls` and `Info Show` count fixtures with the folders
  beside ("1 348 (23 folders)"); lum.md gained the examples, no grammar change. Found and fixed
  with it: the Edit Patch dialog's list followed the feed's sort, so a fixture just patched jumped
  in the list and "the next one" with it — the list keeps the order it opened with, each row's
  facts follow the feed. The e2e on the rig: the virtual grid draws only the rows in view, so the
  tests scroll to a row (`reveal`, until it is drawn — a redraw puts its saved scroll back) after
  `byNumber`; Insert takes the Icarus (a thumbnail and channels; the Photon's file has none) and
  numbers past the rig, Multipatch a free universe (39), the map test the widest free run, Edit
  Patch its own ground in universe 108 (486 free slots) and a slot outside the Photon's 28; the
  hold helper waits 900 ms, the claim test holds until the station answers; the new test is
  `the MVR tree (§40 40d)`. Not built yet: New folder here on a folder (its child line), Duplicate,
  the desk's Patch window capture (the bridge was down).
- 40e `lumacore-poolxml` + `Import PoolXml … Groups`.
- 40f ✓ built 2026-10-20: `demo::build` is the rig — a `LocalLibrary` drive on the repo's
  `library/` (so the words read it wherever the process runs), `Import Mvr "ti2026.mvr" /IdType
  Media` into the claimed Devices, Apply `/Gone Recreate`, Keep `/Default`, one sACN line per run
  of universes (`demo::runs`: 1–38, 40–41, 43–102, 104–105, 107–108, 187, 255 → `Sacn 1` Thru
  `7`), the user LD, `Import PoolXml "ti2026-groups.xml" Groups`, then every line of
  `library/demo/manual.lum` (`#` comments; today `Set Group 1 "Mode" Scaling`), the show named
  `Demo`. The file is gitignored; **one door** `demo::ensure` builds it when missing or stale (a
  newer source, another format) — the station's start takes it for the demo's path (`app.rs`
  `ensure_demo`), the e2e global setup runs `gen-demo --if-stale`, `gen-demo` alone always
  builds, the catalog test writes the file when the build differs. The test is
  `the_demo_show_builds_loads_and_matches_its_golden`: counts and the SHA-256 of the normalized
  JSON in `tests/catalog/testdata/demo.golden.txt` (`LUMACORE_BLESS=1`). Tests get the rig
  through `demo::fresh_state()` (built once per process, a copy each; the GDTF sources travel
  with it) — **every test runs on the rig** (the user, 2026-10-20: "All tests must run equally
  good on the 1000 fixture+ demo file"): the unit tests that named the six AuraXB now name the
  rig's facts (fixtures 1–9999 with gaps, 77 groups, `A3` = Fixture 1200, ColorAdd_W in Color
  with a sub), the MVR tests put their files on a universe the rig lacks and numbers past it
  (`free_universe`, 20101…, `/UniverseOffset 110`), the goldens that walk the tree cap a pool at
  its first 3 members and a listing at 40 lines and number every GUID (`numbered_guids`), the
  completion golden shows the first 8 of a long offer. **The change path on a big show**, found
  by the rig: a line cost 1.0 s in debug — two JSON walks of the show for the sysmon's "tree
  modified" line (gone: `TreeCounts` is counts, the change path says whether anything moved,
  `sysmon::reported` keeps the old subtrees) and two JSON walks for the fingerprints
  (`lumacore_base::fingerprint`: a serde serializer into a byte buffer hashed once, tags and
  lengths so shapes never collide, the readable form as JSON has it; `finish` hashes once and
  re-hashes all but Devices after the attribute sync) — 82 ms a line now, with
  `[profile.dev.package.lumacore-{base,fixtures,tree,state}] opt-level = 1` (155 ms without:
  the walk is instantiated in lumacore-state). The next lever is structural (the fixture types
  are 1.9 MB of the 3.2 MB Devices JSON and change only on Import) — roadmap. **Two more costs
  the rig found and the build fixed:** `Fixture n` was a scan of the slot map
  (`fixture_by_fid`, since 40b) — `PatchData` now carries a number → slot index (`FidIndex`, a
  lock around a map, never saved, every hit checked against the slot's number, so a fixture
  moved or renumbered behind it can't mislead; `after_load` builds it whole); and a Tab on
  `Fixture 1` took 1.4 s (every candidate rebuilt the object's children and scanned the
  registry for its actions — 2 719 candidates on the rig): the suggest engine builds an object's
  children once per query and keeps each object's action words (`action_words`), 250 ms now
  (what remains is `object_in`'s JSON per candidate — roadmap). The e2e suite on the rig
  (29 of 374 cases rewritten): a test names the rig's facts (Group 1 "Photon", Fixture 1 at
  1.001 an Aeco 30, universe 1 full, 77 groups, `List Sacn` = 7 lines, the tree chip
  "All · 1371") or steps past it (groups 85+, universes 109–112 in the long gap, numbers 20000+,
  MVR offsets `/UniverseOffset 110`); a virtual grid draws the rows in view, so a test takes a
  row it can see (fixture 1, group 90 of its own) and the pool reveals a cell before a tap
  (`reveal`); the map's bars are scrolled into view and left to settle (a lift near the
  scroller's edge scrolls it; the map redraws after an Assign); the tree sidebar's fold button
  got its 44 px touch size (the rig's folders made it visible on a tablet); the suggest list
  reads the station's answer for the whole line (the queued keys first); the sysmon's "tree
  modified" line keeps its old subtrees (`sysmon::reported`), so a gesture stays quiet. **Two
  kit bugs the rig showed:** the value dialog was placed once and grew past the viewport's
  bottom with its hint and free values (a row near the end of a big patch on a tablet: Apply
  out of reach) — a `ResizeObserver` keeps it inside; and on WebKit every paint of a virtual
  grid landed back at the top (the body's `innerHTML` swap empties it for a moment, WebKit
  clamps the scroller to 0 and keeps it: nobody could scroll the rig's patch list in Safari) —
  the rows are swapped in one step (a template, `replaceChildren`) and a scroll that still
  moved is put back once, without a paint of its own.
  A `Screenshot` waits 30 s for a client's capture now, not 10 (the station's page with the
  rig's windows took longer to render itself, `screenshot::WAIT`). **Seen, not fixed
  (roadmap):** `List Mvr` on the rig replies 147 KB (1 348 Lost rows — past
  the command port's E-WIRE-002); after `Group 2 ` the bar's grey hint is `All`, the first of
  the whole object list; the debug render thread misses frames on the rig (Overload episodes
  of one late frame after a patch change); the systemtest's batch budget (2 s) held at 0.7 s;
  the sweep's property tests take 150 s on the rig (a world parses the 5 MB show per case); `ls`
  counts folders as fixtures ("1371 fixtures") — 40d's.
