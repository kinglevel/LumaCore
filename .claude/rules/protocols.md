---
paths:
  - crates/network/protocols/**
  - crates/model/lumacore-protocols/**
  - crates/commands/objects/lumacore-cmd-protocols/**
  - crates/commands/objects/protocols/**
---

# Protocols and lines

Records: `protocol-crates-declare-themselves`, `protocol-contract-below-the-tree`, `lines-one-type-runs`, `protocol-runtime-lifecycle`, `station-table`, `osc-line-user-required`, `declared-words-written-as-declared`, `dmx-input-merge`.

## Each protocol declares itself

- `lumacore-protocols` (in `model/`, below the tree) is the interface: the `ProtocolDef` catalog row (name, kind `Dmx | Data`, directions, port, capabilities `Sync` / `Discovery` / `SharedPort`, its opener), the `DmxDriver` / `ControlListener` traits, the input merger and `LinePlan`. `Nic` is plain data there; `network/` holds everything with sockets or threads.
- `lumacore-artnet`, `lumacore-sacn`, `lumacore-osc` add their row in `register` (`r.rows(..)`); readers use `lumacore_registry::get().rows::<ProtocolDef>()` and `protocol::find`. Each protocol's object is its own command crate (`lumacore-cmd-sacn`, `-artnet`, `-osc`), which registers its row first; the station's `register` lists them (and the sweep's; test catalogs that load the demo show register the rows as dev-dependencies). The line runtime never names a protocol; the protocol crates never see the renderer. Protocols are not engines.
- A show naming a protocol this build lacks, or a setting its protocol doesn't declare, is refused at load (E-SHOW-007 / 008, `LineOps::check_protocols`). Every station of a LumaSession runs the same build, version and commit (E-LUMA-006).

## A protocol is an object, a line one row

- `Sacn 1`, `ArtNet 1`, `Osc 1`, numbered per protocol, kept under the protocol's node (`Root.ShowData.Protocols.DMXProtocols.Sacn.1`, `…DataProtocols.Osc.1`; `ProtocolsData.dmx_protocols` / `data_protocols` by row name; `Node::DmxProtocols` is `Kind::Named`, a step `Step::Name`; the state crate never names a protocol). `Output`, `Input` and `/Protocol` are gone; a bare protocol word is the protocol (`Info ArtNet`). No line changes protocol.
- A line has `Mode` Output | Input, set by Assign's order: `Assign Universe 5 Thru 8 At Sacn 1 [/Universe 101]` sends, `Assign Sacn 2 At Universe 40` receives. One run of local universes per line, `LocalUniverse` + `Amount` (slots by number, never links), starting on the wire at the protocol's universe (`SacnUniverse`, Art-Net `PortAddress`). Another run asks Replace / New line; more runs are more lines; overlaps are allowed; two outputs never send one protocol universe to one destination (E-OUTPUT-003); a gap is E-OUTPUT-007.
- What every line of a kind has is declared once (`lumacore_cmd_protocols::props`, `line_object!`, `dmx_line_verbs!`); what only one protocol has lives in `Line.settings`, declared on its row as `SettingDef`s (sACN `Priority`, OSC `Port`). `Set ArtNet "<Setting>"` for protocol-wide settings is not built; say so, never fake it.
- Lines, networks and routes are live: they need no Devices claim. The DMX Protocols menu is a `MenuDef` on `lumacore-cmd-station` with `left` tabs from the feed. `Assign Universe … At Sacn n` stays the kernel's.

## Networks and stations

- The show names `Network`s; each station maps a network to a NIC in its row of the station table `ShowData.Stations` (`Set Network "Rig" "Interface" "en1"`, `Set Station "Tower" + "FOH" Network "Rig" "Interface" "en1"`), applied by that station only (`apply_own_row`; last-applied copy in its station settings, `Root.StationSettings.networks`). The map follows GUID renames.
- Per-NIC services (`Set Interface "en1" "LumaSession"|"Shell"|"HTTP" Yes|No`) are station-local (`ServiceAccess`); loopback always serves; `Set Station "Online" Off` stops everything networked but localhost.

## OSC

- An OSC line names the user its commands run as (`Store Osc 1 /Network "Rig" /Port 8000 /User …`; E-RIGHTS-003 without one; any user, Admin too, chosen explicitly, never Admin as a fallback; the user needn't be logged in). Deleting that user asks naming the lines. Nothing listens by default. OSC keeps its own bounded channel into the state thread.

## Words

- Typed values and declared words run and are journalled in their declared spelling (`ArtNet`, `Sacn`, `Osc`); an unparseable value is E-CMD-008, never a silent default.
