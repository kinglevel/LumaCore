---
id: 0042
title: A protocol is an object, a line is one row with one run
area: lines
status: built
date: 2026-10-17
built_in: 34f7c
supersedes: [archive §11 L552–602 "per-universe routes, Output", archive §11 L627–639 "Output n Input n", archive §33 L7154–7160 "At Output 1", archive §34 L8221–8236 "kept, never sends", archive §34 L9128–9178 "34f7b dormant settings"]
superseded_by: —
confirmed_by: [E-OUTPUT-001, E-OUTPUT-003, E-OUTPUT-005, E-OUTPUT-006, E-OUTPUT-007, E-SHOW-007, E-SHOW-008, `LineOps::check_protocols`]
source: [archive §11 L552–602, archive §11 L627–639, archive §11 L736–770, archive §33 L7154–7160, archive §33 L7745–7757, archive §34 L8221–8236, archive §34 L9128–9224, archive §34 L9225–9333]
---
# A protocol is an object, a line is one row with one run

## Context
Routing went through three shapes: per-universe routes on `Output n` / `Input n` lines with protocol shortcuts (`At sACN 5`), then routes naming the line, then — after studying a reference console's protocol rows — one object per protocol.

## Decision
One `Line` type (`lumacore_protocols::Line`) and one set of handlers (`lumacore_cmd_protocols::line_object!`, `dmx_line_verbs!`) — never duplicated per protocol or direction. Each protocol row is a counted object word: `Sacn 1`, `ArtNet 1`, `Osc 1`, numbered per protocol, at `Root.ShowData.Protocols.DMXProtocols.Sacn.1` / `…DataProtocols.Osc.1`; the state crate never names a protocol. A line has `Mode` Output | Input, set by Assign's order (`Assign Universe 5 Thru 8 At Sacn 1 [/Universe 101]` sends, `Assign Sacn 2 At Universe 40` receives), and one contiguous run `LocalUniverse` + `Amount` of slots by number — never links — starting on the wire at `SacnUniverse` / `PortAddress`. Another run asks Replace / New line / Cancel; a gap is E-OUTPUT-007, a wire number outside the row's range E-OUTPUT-005, a run on a data line E-OUTPUT-006, two outputs sending one protocol universe to one destination E-OUTPUT-003. Overlaps are allowed. What every line of a kind has is declared once (`lumacore_cmd_protocols::props`); what one protocol has lives in `Line.settings`, declared as `SettingDef`s on its `ProtocolDef`. The bare word is the protocol (`Info ArtNet`). No line changes protocol. A show naming a protocol this build lacks (E-SHOW-007) or an undeclared setting (E-SHOW-008) is refused at load. Lines are live and need no claim (see staged-patch-draft).

## Consequences
`Output`, `Input`, `/Protocol`, `Route`, `Range` and E-OUTPUT-004 are gone. A new protocol setting bumps the show format. Not built: `Set ArtNet "<Setting>"` on the protocol itself (no protocol-wide setting exists yet), a protocol's own grid columns.
