---
id: 0136
title: mv, cp and rm run through each provider's door
area: door
status: built
date: 2026-10-15
built_in: 35f (the crate cut in 34g4 cut 2)
supersedes: [archive §35 L11048–11059 "Move's alias mv"]
superseded_by: —
confirmed_by: [`door::shell::write_out`, `Hook::Rewrite`, the tree-shell golden record]
source: [archive §35 L11018–11026, archive §35 L11048–11059]
---
# mv, cp and rm run through each provider's door

## Context
Writing through the tree shell needed the Lum verbs' rules, not a second implementation. PowerShell's providers gave the shape: one word, the door of whatever the path names. Files have no GUIDs, so the drive side cannot share the pool door's swap or Oops.

## Decision
On the show tree `mv`, `cp` and `rm` are written out in dispatch as the Lum line they mean, by `door::shell::write_out` next to `write_out_bare`, pushed as `Hook::Rewrite` by `lumacore-cmd-treeshell` (see dispatch-special-cases-are-hooks): `mv 3 7` in DataPool 1's Groups = `Move DataPool 1 Group 3 At 7`, `--merge` / `--overwrite` = `/Merge` / `/Overwrite`, `rm … -n` = `/NoConfirmation` — before rights, forwarding, the journal and Oops, so the pool door does the work (see one-object-door), a member forwards an absolute line, a stored line runs the Lum line nested. Under `Root.Drives` they run on this station (scope Station, Setup rights, reach `Drive`) through the drive door (`core::drive`, see reach-drive-door): a taken name asks Overwrite / Keep both / Cancel (`--overwrite` / `--keep`); a move across volumes is copy + remove; `rm` never deletes — it moves the file to the volume's `.trash` (else `library/.trash`, kept as for shows) and the reply carries a `Put back: mv …` line; disk changes are not Oops steps. Between `Root.Drives` and the show tree they are refused (Unix's EXDEV): that is `Import` / `Export`. Move's alias `mv` is gone — `mv` is the fence word; a node with no pool refuses.

## Consequences
The sweep runs the drive forms only in its sandbox (see command-sweep-dynamic). A new provider under the tree answers these words through its own door, never a handler inside the tree shell; the walk (`vfs`, `navigation`) stays the kernel's.
