# The demo rig, nested fixtures and second ids (2026-10-20)

Asked by the user after §39 ("expand the demo with more fixtures and groups, analyze ../ and find
a big mvr we can have in the demo"; then "../ExternalData/DemoData is the data i think we should
use"; "lets discuss this and harden with artifacts first before we implement anything"). Seventeen
questions, one page each, every number on them measured (the 186 rigs scanned, the candidates
imported for real, a reference desk consulted). Decided in decisions §40.

| File | Artifact | What it asks · the answer |
|---|---|---|
| `01-gdtf-parsing.html` | https://claude.ai/artifact/PTLoG4ZAtd681ZPSBZ8zmw | The 186 rigs scanned; the GDTF reader refused 672 of 1 084 embedded GDTFs. A fork with the spec's defaults · B an own reader · C pre-clean the XML. **A** (built). |
| `02-which-rig.html` | https://claude.ai/artifact/QnsaPBXaSot3sF8a5Gmjdq | Capture's Dummy types; the real-GDTF shortlist imported for real. Superseded by the user's DemoData. |
| `03-demodata.html` | https://claude.ai/artifact/R9v1L1SCRQm3ujxrAFzQuu | The DemoData inventory (the TI 2026 MVR, 39 DataPool exports); the exporter's folders as Grouping fixtures. The user: nested fixtures natively. |
| `04-nested-fixtures.html` | https://claude.ai/artifact/G6izwMHmySgC1Vxfr2pYzc | Nested fixtures and the folder's number (folders are Media-id fixtures without a FID). The user: CIDs as a second id. |
| `05-cids.html` | https://claude.ai/artifact/N2hPRdBr9XfzK1oBph2oUX | The patch's key once a fixture may have no number. **A** an internal slot. |
| `06-tree-index.html` | https://claude.ai/artifact/82V2a1smdsi84NzUHwcWNS | The tree index; the desk's child order against the file's. **A** the slot plus an order on each node. |
| `07-id-shape.html` | https://claude.ai/artifact/CVxnSvEuQ4bCmjg1qahmfY | The ids a fixture carries. **A** the triple (FID · IDType · CID). |
| `08-keyword.html` | https://claude.ai/artifact/EWUQY8bW72sAfohpgk9vgJ | How a line names a second id. **A** `Fixture Media 7003` as lum.md has it. |
| `09-untyped-cid.html` | https://claude.ai/artifact/3ihRMoYQWmZ3K7WiccCegf | A CustomId the file never typed. **A** the preset's `MvrIdType`, `/IdType` overrides. |
| `10-folder-word.html` | https://claude.ai/artifact/6Q7RNuGhZfQF9xFgVvK3z2 | What a folder's id type is called. The user: **Generic** (MVR's reserved 3) as the default id type for CIDs. |
| `11-folder-line.html` | https://claude.ai/artifact/5qFWk7mmfqLnK5nsSmy1u6 | What a folder does on a line. The user: the folder only; values go down; any fixture may hold fixtures; no sub-fixtures. |
| `12-folder-values.html` | https://claude.ai/artifact/CaTKx6TvEFDC1GXPubHCo4 | Where a value at a folder lives. **B** fan-out; the programmer's Direct / Down switch later. |
| `13-groups.html` | https://claude.ai/artifact/B5bcBNTvc3TrxkbavCPjnU | The demo's groups. **A** a small pool XML reader, Groups first. |
| `14-shows.html` | https://claude.ai/artifact/4poU9cZgd6sJ2bQEBeYwh3 | One show or two. The user: one, the rig replaces the demo (a large dataset for testing). |
| `15-commit.html` | https://claude.ai/artifact/8zudFMTaLaHo2khpYfo3bU | What git carries. The user: two demos — generated (from the DemoData in git) and manual (from saves), the generated one taking the manual one's profile. |
| `16-manual-demo.html` | https://claude.ai/artifact/GXRCgxsQRpKp5bQZEx4gio | How the manual demo survives a format bump. **A** a replayable script, `library/demo/manual.lum`. |
| `17-names.html` | https://claude.ai/artifact/AF2P9JD7qd6mpDMF67rjwQ | Names. **A** everything as the desk had it; the show is `Demo`. |
