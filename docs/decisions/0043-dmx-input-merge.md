---
id: 0043
title: DMX input merges per channel in the tick
area: lines
status: built
date: 2026-10-12
built_in: step 4c
supersedes: [archive §27 L2549–2599 "external input on top"]
superseded_by: —
confirmed_by: [`InputMerger`, the sACN fuzz target]
source: [archive §11 L640–683, archive §27 L2662–2683, archive §29 L3231–3233, archive §29 L3291–3293]
---
# DMX input merges per channel in the tick

## Context
The festival gateway case (guest sACN in on one NIC, merged into local universes, sent to the rig on another) needed input lines and a merge. A reference console puts merge mode on the universe and input priority on the line; the first playback-merge draft had "external input on top", replaced by "at its input priority".

## Decision
One driver per (protocol, NIC) receives for every input line on it; only the sending station listens (see protocol-runtime-lifecycle). `lumacore_protocols::input::InputMerger` merges the sources of one protocol universe by the E1.31 rules — highest sACN priority wins, equal priority HTP per channel, Art-Net HTP — with the sequence check (late and duplicate packets dropped, a big jump accepted as a restart), synced data held until its sync packet with a no-sync fallback, a loss timeout, and stream-terminated releasing at once; preview packets, our own CID and our own IPs are ignored, so a loop through our output never feeds back. Nothing allocates per received packet (`SourceName`, reused buffers). The result is an `InputLayer` on `RenderHub::inputs`, merged per DMX channel in the tick (`render::render_into`): `Universe.merge_mode` Off / Prio / HTP / LowTP, band = `Line.input_priority`; channels with no fixture pass input through. Park always wins; grand master, blackout and Solo touch only our own output. `DMX In <u>` reads the raw universe; the `dmx` window reads the `dmx` feed.

## Consequences
Input never reaches the state thread; the merge has no second home. The input bands sit on today's layers until the Priority pool exists (see priority-pool), then slot in with no show change. Source loss and seen sources are runtime state (`Runtime.Network.Sources`, line status). A parser change runs the sACN fuzz target; the timeouts live as constants in `lumacore-protocols`.
