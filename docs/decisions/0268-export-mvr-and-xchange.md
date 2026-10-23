---
id: 0268
title: Export MVR writes what the dialog shows; Xchange per NIC
area: mvr
status: decided
date: 2026-10-20
built_in: —
supersedes: [archive §34 L9489–9491 "By MVR layer | By Luma layer"]
superseded_by: —
confirmed_by: none
source: [archive §34 L9485–9498, archive §34 L9519–9520, archive §40 L11695]
---
# Export MVR writes what the dialog shows; Xchange per NIC

## Context
Points 13–14 of the patch hardening (2026-10-17) decided the way back out of the show and the live exchange with planning tools. The export's layout switch (By MVR layer | By Luma layer) was decided while Luma layers were planned and went with them; the tree is exported as it is. §40 added what the tree export must carry.

## Decision
Export MVR writes what the dialog shows — the holder's draft while held, otherwise the live patch — for the whole patch, a node or the selected fixtures; GDTFs are embedded; the tree goes as it is (`ChildList` in order, the ids, `CustomIdType` + `CustomId`, local = parent⁻¹ × world, a multipatch copy as MVR's separate fixture with `multipatch` set); "Commit to Xchange" with a comment is an option; anything made only for the export gets a UUID once, kept in `Devices.Metadata.MVR` so the next export reuses it. MVR-xchange is a station-local service per network interface (`Set Interface "en1" "MvrXchange" Yes`, plus `MvrXchangeGroup`, default "Default"); the import dialog's Xchange side lists the files committed in the group (name, station, comment, time); picking one fetches it into `library/mvr/.xchange/` and then it imports like a file (see import-mvr-reviews-against-the-draft).

## Consequences
Not built: both (the Export MVR and MVR-xchange parts of 34g4; the roadmap carries them). An export never invents a second layout; a station-local service never rides in the show.
