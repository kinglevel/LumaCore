---
id: 0223
title: Files stay short; the hard limit needs a listed reason
area: structure
status: built
date: 2026-10-15
built_in: 34f1
supersedes: []
superseded_by: —
confirmed_by: [files_stay_short]
source: [archive §34 L8003–8005 (item 9)]
---
# Files stay short; the hard limit needs a listed reason

## Context
The research round counted the forty longest files, most over a thousand lines, with the station crate holding two thirds of the workspace. Long files hid copies and made the Part order meaningless.

## Decision
A file stays under the soft limit tidy declares; over the hard limit it fails `files_stay_short` unless `LONG_FILES` names it with a written reason (a generated table, one state machine). The files that were long on 2026-10-15 sit on that list as a ratchet that only shrinks: a listed file that becomes short, or is gone, fails the test until its row goes. Functions are held by clippy's `too_many_lines` as a warning. A Part that grows past the soft limit, or a file needing both COMMANDS and IO, is split rather than listed.

## Consequences
A new file never enters `LONG_FILES`; a change that pushes an old file over the hard limit splits it or moves the touched part out (see one-folder-per-object-one-file-per-verb). The limits and the list live in `lumacore-tidy`, never in prose. Open: the remaining long files, each taken when its area moves.
