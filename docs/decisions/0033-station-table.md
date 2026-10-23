---
id: 0033
title: Rig wiring in the show's station table, identity on the station
area: station
status: built
date: 2026-10-13
built_in: step 4 (networks), 12d part 4 (the table)
supersedes: [archive §10 L459–462 "station-local interfaces subtree", archive §11 L583–585 "saved in station config", archive §30 L4859 "edited remotely through master"]
superseded_by: —
confirmed_by: [stations_tests, AppState::follow_renames (R1-69 Q41)]
source: [archive §11 L581–586, archive §11 L618–620, archive §30 L4712–4714, archive §30 L4856–4863, archive §30 L5098–5123]
---
# Rig wiring in the show's station table, identity on the station

## Context
§11 made the show name its networks and each station map them to NICs in its own config file. 12d took the established split: wiring belongs to the rig and is pre-configurable while a station is offline; identity must never change with a show load.

## Decision
The show names its networks (`ShowData.Protocols.Networks`, `Store Network "Rig"`). Each station maps a network to a NIC in **its row of the station table** `ShowData.Stations` (`lumacore_tree::StationWiring`, `ShowSubtree::Stations`, `Stations.json`, `core::stations`): GUID (`StationWiring::guid_for(station id)`, the same on both sides of a split), number, name, station id (empty = made in advance), network → NIC (`BTreeMap<network GUID, NIC spec>`) and services per NIC. **In the show** (replicated, undone, saved): the wiring. **On the station, never in the show**: identity — name, location, priority, LumaSession, Online, Invite, HoldOutput, OutputWhenAlone, ThemeOverride, MulticastBase, StartupShow — plus asset folders, the sACN CID and tokens. Each station applies only its own row (`apply_own_row`, on every show change) into `Root.StationSettings`, keeping a last-applied copy (`library/station_settings.json`); a show without a row for it changes nothing. Interfaces are their hardware's names, not GUIDs. Lines: `Set Network "Rig" "Interface" "en1"`, `Store Station "Tower"` in advance, `Set Station "Tower" + "FOH" Network "Rig" "Interface" "en1"` (several rows, one Oops step); an identity edit of another station goes to its command port (see remote-station-edits); `Claim LumaSession Station "X"` hands the master role over. **A line on an unmapped network never sends** — no Auto fallback. Station maps are keyed by name and follow an AssetRoot or Network renamed by GUID (`AppState::follow_renames` on publish) — the one exception to "links are GUIDs".

## Consequences
A new per-station setting is either wiring (a table column, replicated) or identity (the station file) — decide first. Protocol lines name a network, never a NIC (see the protocols record).
