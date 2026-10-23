---
id: 0041
title: Protocol runtimes start from show lines; only the master sends
area: protocols
status: built
date: 2026-10-11
built_in: step 4b
supersedes: [archive §27 L2645–2660 "the dmx_output_enabled hack"]
superseded_by: —
confirmed_by: [`protocol_plan::desired`, `ProtocolRuntimes::reconcile`]
source: [archive §11 L541–551, archive §11 L603–604, archive §11 L662–665, archive §27 L2645–2660]
---
# Protocol runtimes start from show lines; only the master sends

## Context
Before step 4b the sACN, Art-Net and OSC runtimes were registry engines started at boot, and a `dmx_output_enabled` flag decided who sent. Modelled on established consoles, where only the session master transmits, the protocols became a consequence of the loaded show: a show is prepped with its inputs and outputs, a station only maps its NICs.

## Decision
Nothing protocol-related starts at boot. A protocol runtime starts only when the loaded show has an enabled line of that protocol on a mapped network and stops when the last one goes or the show is unloaded. `protocol_plan::desired` (objects) is the pure plan of drivers per (protocol, NIC) and listeners per line; `lumacore_lines::ProtocolRuntimes::reconcile` makes it so on the state thread after every show change, every station change and on its timer. There is one renderer; output drivers attach to it and never build their own from a startup snapshot. Who sends: a station that never joined a LumaSession, or the session master; an IdleMaster only with the station's "output when alone"; every other member shows "silent (not master)" and opens no input socket either. A line on an unmapped network never sends and no "auto" fallback exists (see station-table). Protocols are not engines (see protocol-crates-declare-themselves for the crates).

## Consequences
A new protocol plugs in as a crate with a row and a driver; it never adds a start-at-boot path or a second "who sends" rule. Line status (sending n universes / not mapped / socket error / silent) comes from the runtime per line. The sweep and tests start no sockets unless a show line asks. A change to the plan is tested against `protocol_plan::desired` as a pure function; the runtime side is `lumacore_lines`.
