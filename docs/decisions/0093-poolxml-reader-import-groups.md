---
id: 0093
title: A small pool XML reader; Import PoolXml Groups
area: files
status: built
date: 2026-10-20
built_in: 40e
supersedes: []
superseded_by: —
confirmed_by: [the Import PoolXml reply's counts, tests/catalog/testdata/demo.golden.txt]
source: [archive §40 L11636–11640, archive §40 L11708–11718]
---
# A small pool XML reader; Import PoolXml Groups

## Context
The rig's groups exist only in a DataPool XML export from another console. §40 decided a small reader, Groups first, so the generated demo gets its groups through a station word rather than a hand-made file.

## Decision
`crates/formats/lumacore-poolxml` reads a DataPool XML export (the Pool › Groups element) streaming, with no lumacore dependency; the pool's other children are counted, not read. `Import PoolXml "file" Groups [/Drive "usb1"] [/DataPool n]` makes one LumaCore group per exported group in the selected or named DataPool: members by fixture number (an item carrying a second id by its type), the grid cell from X / Y, the file's order kept. A group of sub-fixture items only is skipped and named; sub-fixture items and unknown numbers are counted in the reply. `Groups` is a value word. `scripts/unix/poolxml_groups` pulls the Groups element out of a big export; the rig's extract is `library/poolxml/ti2026-groups.xml` (see demo-show-test).

## Consequences
Presets, sequences and macros follow as later parts of the same reader, each when its LumaCore object exists — not built. Never a second pool XML parser; a further pool's extract enters git when its word lands, never the whole export folder.
