---
id: 0126
title: List, Info and Dump are one command each
area: commands
status: decided
date: 2026-10-15
built_in: —
supersedes: []
superseded_by: —
confirmed_by: none
source: [archive §33 L7709–7744]
---
# List, Info and Dump are one command each

## Context
At 3d the read verbs were dozens of List, Info and Dump handlers, one per object, each with its own output shape. A typed handler per object keeping today's outputs, and deferring the read verbs to part 4, were declined.

## Decision
`List`, `Info` and `Dump` are one command each; the command owns the logic, the objects only declare data. The resolver finds the rows (see one-resolver); columns and fields come from the declared properties (see one-property-registry) and the tree collection; `Dump <target>` is the node's JSON as stored. An object whose information isn't stored rows (Health, Clock) declares a view and a `facts` function on its `ObjectDef`; nothing else keeps a per-object List / Info / Dump handler. `List` shows one row per object, No · Name, then every declared property its own column (mixed kinds add Kind · Count and the union of properties). `Info <target>` shows identity, every declared property with value and kind, the facts, and its links both ways from the reference index. A bare `List` / `Info` / `Dump` follows the prompt: the path `cd` put you on, else your default object's kind (see default-object-per-user); the path wins when both are set. Where you are scopes only the bare form — a named kind resolves as always (`List Group` inside `DataPools.2` is still your selected DataPool's); `cd` never selects and never redirects a line.

## Consequences
Not built — a roadmap item of its own; outputs change shape when it lands (one look for every object). Until then a new object adds no bespoke List / Info output beyond its declared properties; the menus' grids may still show fewer columns for the screen.
